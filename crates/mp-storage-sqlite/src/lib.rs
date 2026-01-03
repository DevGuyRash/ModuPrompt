use mp_kernel::{now_rfc3339, Actor, ProjectListEntry, Subject, WorkspaceListEntry};
use mp_projections::{apply_event, rebuild_projections, ProjectionError, ProjectionWriter};
use mp_protocol::EventEnvelope;
use mp_storage::{AppendResult, CommandMeta, EventStore, NewEvent, ProjectionReader, StoreError};
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| StoreError::Internal(format!("failed to create db dir: {err}")))?;
        }
        let conn = Connection::open(path)
            .map_err(|err| StoreError::Internal(format!("failed to open db: {err}")))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn rebuild_projections(&self) -> Result<(), StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT workspace_id, seq_global, stream_id, seq_stream, event_id, event_type, ts, actor_json, project_id, subject_kind, subject_id, schema_version, payload_json, trace_id
                 FROM events
                 ORDER BY workspace_id, seq_global",
            )
            .map_err(map_sql_err)?;
        let events_iter = stmt
            .query_map([], |row| row_to_event(row))
            .map_err(map_sql_err)?;
        let mut events = Vec::new();
        for event in events_iter {
            events.push(event.map_err(map_sql_err)?);
        }

        let writer = SqliteProjectionWriterConn { conn: &self.conn };
        rebuild_projections(&writer, events).map_err(map_proj_err)?;
        Ok(())
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn
            .execute_batch(MIGRATION_0001)
            .map_err(map_sql_err)?;
        Ok(())
    }

    fn current_seq_in_tx(tx: &Transaction<'_>, workspace_id: &str) -> Result<i64, StoreError> {
        let seq: Option<i64> = tx
            .query_row(
                "SELECT MAX(seq_global) FROM events WHERE workspace_id = ?1",
                params![workspace_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(map_sql_err)?;
        Ok(seq.unwrap_or(0))
    }

    fn current_stream_seq_in_tx(
        tx: &Transaction<'_>,
        workspace_id: &str,
        stream_id: &str,
    ) -> Result<i64, StoreError> {
        let seq: Option<i64> = tx
            .query_row(
                "SELECT MAX(seq_stream) FROM events WHERE workspace_id = ?1 AND stream_id = ?2",
                params![workspace_id, stream_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(map_sql_err)?;
        Ok(seq.unwrap_or(0))
    }

    fn lookup_idempotency(
        &self,
        idempotency_key: &str,
        command_type: &str,
    ) -> Result<Option<IdempotencyRecord>, StoreError> {
        let record = self
            .conn
            .query_row(
                "SELECT workspace_id, idempotency_key, command_type, trace_id, first_seq_global, last_seq_global, status_code
                 FROM idempotency_keys
                 WHERE idempotency_key = ?1 AND command_type = ?2",
                params![idempotency_key, command_type],
                |row| {
                    Ok(IdempotencyRecord {
                        workspace_id: row.get(0)?,
                        idempotency_key: row.get(1)?,
                        command_type: row.get(2)?,
                        trace_id: row.get(3)?,
                        first_seq_global: row.get(4)?,
                        last_seq_global: row.get(5)?,
                        status_code: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(map_sql_err)?;
        Ok(record)
    }

    fn load_event_range(
        &self,
        workspace_id: &str,
        first: i64,
        last: i64,
    ) -> Result<Vec<EventEnvelope>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT workspace_id, seq_global, stream_id, seq_stream, event_id, event_type, ts, actor_json, project_id, subject_kind, subject_id, schema_version, payload_json, trace_id
                 FROM events
                 WHERE workspace_id = ?1 AND seq_global BETWEEN ?2 AND ?3
                 ORDER BY seq_global",
            )
            .map_err(map_sql_err)?;
        let events_iter = stmt
            .query_map(params![workspace_id, first, last], |row| row_to_event(row))
            .map_err(map_sql_err)?;
        let mut events = Vec::new();
        for event in events_iter {
            events.push(event.map_err(map_sql_err)?);
        }
        Ok(events)
    }
}

impl EventStore for SqliteStore {
    fn append(
        &mut self,
        meta: &CommandMeta,
        events: Vec<NewEvent>,
    ) -> Result<AppendResult, StoreError> {
        if events.is_empty() {
            return Ok(AppendResult {
                events: Vec::new(),
                idempotent: false,
            });
        }

        if let Some(key) = &meta.idempotency_key {
            if let Some(existing) = self.lookup_idempotency(key, &meta.command_type)? {
                let events = self.load_event_range(
                    &existing.workspace_id,
                    existing.first_seq_global,
                    existing.last_seq_global,
                )?;
                return Ok(AppendResult {
                    events,
                    idempotent: true,
                });
            }
        }

        let workspace_id = events[0].workspace_id.clone();
        let tx = self.conn.transaction().map_err(map_sql_err)?;
        let mut seq_global = Self::current_seq_in_tx(&tx, &workspace_id)?;
        let mut stream_seq_cache: HashMap<String, i64> = HashMap::new();
        let mut appended = Vec::new();
        let mut first_seq = None;
        let mut last_seq = None;

        for event in events {
            let stream_id = event
                .stream_id
                .clone()
                .unwrap_or_else(|| event.subject.id.clone());
            let seq_stream = match stream_seq_cache.get_mut(&stream_id) {
                Some(current) => {
                    *current += 1;
                    *current
                }
                None => {
                    let mut current =
                        Self::current_stream_seq_in_tx(&tx, &workspace_id, &stream_id)?;
                    current += 1;
                    stream_seq_cache.insert(stream_id.clone(), current);
                    current
                }
            };

            seq_global += 1;
            let event_id = mp_kernel::new_uuid();
            let timestamp = now_rfc3339();
            let envelope = EventEnvelope {
                event_id,
                event_type: event.event_type.clone(),
                timestamp,
                actor: event.actor.clone(),
                workspace_id: event.workspace_id.clone(),
                project_id: event.project_id.clone(),
                subject: event.subject.clone(),
                payload: event.payload.clone(),
                schema_version: event.schema_version,
                seq_global,
                seq_stream,
                trace_id: event.trace_id.clone(),
            };

            tx.execute(
                "INSERT INTO events (workspace_id, seq_global, stream_id, seq_stream, event_id, event_type, ts, actor_json, project_id, subject_kind, subject_id, schema_version, payload_json, trace_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    envelope.workspace_id,
                    envelope.seq_global,
                    stream_id,
                    envelope.seq_stream,
                    envelope.event_id,
                    envelope.event_type,
                    envelope.timestamp,
                    serde_json::to_string(&envelope.actor).map_err(map_serde_err)?,
                    envelope.project_id,
                    envelope.subject.kind,
                    envelope.subject.id,
                    envelope.schema_version,
                    serde_json::to_string(&envelope.payload).map_err(map_serde_err)?,
                    envelope.trace_id,
                ],
            )
            .map_err(map_sql_err)?;

            let writer = SqliteProjectionWriterTx { tx: &tx };
            apply_event(&writer, &envelope).map_err(map_proj_err)?;

            if first_seq.is_none() {
                first_seq = Some(seq_global);
            }
            last_seq = Some(seq_global);
            appended.push(envelope);
        }

        if let Some(key) = &meta.idempotency_key {
            tx.execute(
                "INSERT INTO idempotency_keys (workspace_id, idempotency_key, command_type, trace_id, first_seq_global, last_seq_global, status_code)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    workspace_id,
                    key,
                    meta.command_type,
                    meta.trace_id,
                    first_seq.unwrap_or(0),
                    last_seq.unwrap_or(0),
                    "accepted",
                ],
            )
            .map_err(map_sql_err)?;
        }

        tx.commit().map_err(map_sql_err)?;

        Ok(AppendResult {
            events: appended,
            idempotent: false,
        })
    }

    fn read_from(
        &self,
        workspace_id: &str,
        from_seq: i64,
        limit: Option<i64>,
    ) -> Result<Vec<EventEnvelope>, StoreError> {
        let mut sql = "SELECT workspace_id, seq_global, stream_id, seq_stream, event_id, event_type, ts, actor_json, project_id, subject_kind, subject_id, schema_version, payload_json, trace_id
                   FROM events
                   WHERE workspace_id = ?1 AND seq_global > ?2
                   ORDER BY seq_global".to_string();
        if limit.is_some() {
            sql.push_str(" LIMIT ?3");
        }
        let mut stmt = self.conn.prepare(&sql).map_err(map_sql_err)?;
        let rows = if let Some(limit_val) = limit {
            stmt.query_map(params![workspace_id, from_seq, limit_val], row_to_event)
        } else {
            stmt.query_map(params![workspace_id, from_seq], row_to_event)
        }
        .map_err(map_sql_err)?;
        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(map_sql_err)?);
        }
        Ok(events)
    }

    fn head_seq(&self, workspace_id: &str) -> Result<i64, StoreError> {
        let seq: Option<i64> = self
            .conn
            .query_row(
                "SELECT MAX(seq_global) FROM events WHERE workspace_id = ?1",
                params![workspace_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(map_sql_err)?;
        Ok(seq.unwrap_or(0))
    }
}

impl ProjectionReader for SqliteStore {
    fn list_workspaces(&self) -> Result<Vec<WorkspaceListEntry>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT workspace_id, name, root_path, created_at, seq_global
                 FROM proj_workspaces
                 ORDER BY name",
            )
            .map_err(map_sql_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(WorkspaceListEntry {
                    workspace_id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    created_at: row.get(3)?,
                    seq_global: row.get(4)?,
                })
            })
            .map_err(map_sql_err)?;
        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(row.map_err(map_sql_err)?);
        }
        Ok(workspaces)
    }

    fn list_projects(&self, workspace_id: &str) -> Result<Vec<ProjectListEntry>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT project_id, workspace_id, name, created_at, seq_global
                 FROM proj_projects
                 WHERE workspace_id = ?1
                 ORDER BY name",
            )
            .map_err(map_sql_err)?;
        let rows = stmt
            .query_map(params![workspace_id], |row| {
                Ok(ProjectListEntry {
                    project_id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    seq_global: row.get(4)?,
                })
            })
            .map_err(map_sql_err)?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row.map_err(map_sql_err)?);
        }
        Ok(projects)
    }
}

struct SqliteProjectionWriterTx<'a> {
    tx: &'a Transaction<'a>,
}

impl ProjectionWriter for SqliteProjectionWriterTx<'_> {
    fn reset(&self) -> Result<(), ProjectionError> {
        Err(ProjectionError::Apply(
            "reset not supported in transaction writer".to_string(),
        ))
    }

    fn upsert_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        root_path: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError> {
        self.tx
            .execute(
                "INSERT INTO proj_workspaces (workspace_id, name, root_path, created_at, seq_global)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(workspace_id) DO UPDATE SET name = excluded.name, root_path = excluded.root_path, created_at = excluded.created_at, seq_global = excluded.seq_global",
                params![workspace_id, name, root_path, created_at, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }

    fn upsert_project(
        &self,
        project_id: &str,
        workspace_id: &str,
        name: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError> {
        self.tx
            .execute(
                "INSERT INTO proj_projects (project_id, workspace_id, name, created_at, seq_global)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(project_id) DO UPDATE SET workspace_id = excluded.workspace_id, name = excluded.name, created_at = excluded.created_at, seq_global = excluded.seq_global",
                params![project_id, workspace_id, name, created_at, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }

    fn set_meta(&self, workspace_id: &str, seq_global: i64) -> Result<(), ProjectionError> {
        self.tx
            .execute(
                "INSERT INTO proj_meta (workspace_id, last_seq_global)
                 VALUES (?1, ?2)
                 ON CONFLICT(workspace_id) DO UPDATE SET last_seq_global = excluded.last_seq_global",
                params![workspace_id, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }
}

struct SqliteProjectionWriterConn<'a> {
    conn: &'a Connection,
}

impl ProjectionWriter for SqliteProjectionWriterConn<'_> {
    fn reset(&self) -> Result<(), ProjectionError> {
        self.conn
            .execute_batch(
                "DELETE FROM proj_workspaces; DELETE FROM proj_projects; DELETE FROM proj_meta;",
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }

    fn upsert_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        root_path: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError> {
        self.conn
            .execute(
                "INSERT INTO proj_workspaces (workspace_id, name, root_path, created_at, seq_global)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(workspace_id) DO UPDATE SET name = excluded.name, root_path = excluded.root_path, created_at = excluded.created_at, seq_global = excluded.seq_global",
                params![workspace_id, name, root_path, created_at, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }

    fn upsert_project(
        &self,
        project_id: &str,
        workspace_id: &str,
        name: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError> {
        self.conn
            .execute(
                "INSERT INTO proj_projects (project_id, workspace_id, name, created_at, seq_global)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(project_id) DO UPDATE SET workspace_id = excluded.workspace_id, name = excluded.name, created_at = excluded.created_at, seq_global = excluded.seq_global",
                params![project_id, workspace_id, name, created_at, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }

    fn set_meta(&self, workspace_id: &str, seq_global: i64) -> Result<(), ProjectionError> {
        self.conn
            .execute(
                "INSERT INTO proj_meta (workspace_id, last_seq_global)
                 VALUES (?1, ?2)
                 ON CONFLICT(workspace_id) DO UPDATE SET last_seq_global = excluded.last_seq_global",
                params![workspace_id, seq_global],
            )
            .map_err(|err| ProjectionError::Apply(err.to_string()))?;
        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct IdempotencyRecord {
    workspace_id: String,
    idempotency_key: String,
    command_type: String,
    trace_id: String,
    first_seq_global: i64,
    last_seq_global: i64,
    status_code: String,
}

fn row_to_event(row: &Row<'_>) -> Result<EventEnvelope, rusqlite::Error> {
    let actor_json: String = row.get(7)?;
    let payload_json: String = row.get(12)?;
    let actor: Actor = serde_json::from_str(&actor_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })?;
    let payload: Value = serde_json::from_str(&payload_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })?;

    Ok(EventEnvelope {
        workspace_id: row.get(0)?,
        seq_global: row.get(1)?,
        seq_stream: row.get(3)?,
        event_id: row.get(4)?,
        event_type: row.get(5)?,
        timestamp: row.get(6)?,
        actor,
        project_id: row.get(8)?,
        subject: Subject {
            kind: row.get(9)?,
            id: row.get(10)?,
        },
        schema_version: row.get(11)?,
        payload,
        trace_id: row.get(13)?,
    })
}

fn map_sql_err(err: rusqlite::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

fn map_serde_err(err: serde_json::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

fn map_proj_err(err: ProjectionError) -> StoreError {
    StoreError::Internal(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_kernel::{
        Actor, ProjectCreatedPayload, WorkspaceCreatedPayload, EVENT_PROJECT_CREATED,
        EVENT_WORKSPACE_CREATED,
    };
    use mp_storage::{CommandMeta, NewEvent};
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn temp_store() -> (TempDir, SqliteStore) {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("mpd.sqlite");
        let store = SqliteStore::open(&db_path).expect("open store");
        (dir, store)
    }

    fn command_meta(command_type: &str, idempotency_key: Option<&str>) -> CommandMeta {
        CommandMeta {
            command_type: command_type.to_string(),
            idempotency_key: idempotency_key.map(|s| s.to_string()),
            expected_version: None,
            trace_id: "tr_test".to_string(),
        }
    }

    fn workspace_event(workspace_id: &str, name: &str, root_path: &str) -> NewEvent {
        NewEvent {
            event_type: EVENT_WORKSPACE_CREATED.to_string(),
            schema_version: 1,
            actor: Actor::system(),
            workspace_id: workspace_id.to_string(),
            project_id: None,
            subject: Subject {
                kind: "workspace".to_string(),
                id: workspace_id.to_string(),
            },
            payload: serde_json::to_value(WorkspaceCreatedPayload {
                name: name.to_string(),
                root_path: root_path.to_string(),
            })
            .expect("payload"),
            trace_id: Some("tr_evt".to_string()),
            stream_id: None,
        }
    }

    fn project_event(workspace_id: &str, project_id: &str, name: &str) -> NewEvent {
        NewEvent {
            event_type: EVENT_PROJECT_CREATED.to_string(),
            schema_version: 1,
            actor: Actor::system(),
            workspace_id: workspace_id.to_string(),
            project_id: Some(project_id.to_string()),
            subject: Subject {
                kind: "project".to_string(),
                id: project_id.to_string(),
            },
            payload: serde_json::to_value(ProjectCreatedPayload {
                workspace_id: workspace_id.to_string(),
                name: name.to_string(),
            })
            .expect("payload"),
            trace_id: Some("tr_evt".to_string()),
            stream_id: None,
        }
    }

    #[test]
    fn append_and_read_from_persists_events() {
        let (_dir, mut store) = temp_store();
        let meta = command_meta("workspace.create", None);
        let events = vec![
            workspace_event("w1", "alpha", "/tmp/alpha"),
            project_event("w1", "p1", "core"),
        ];

        let result = store.append(&meta, events).expect("append");
        assert_eq!(result.events.len(), 2);
        assert!(!result.idempotent);
        assert_eq!(result.events[0].seq_global, 1);
        assert_eq!(result.events[1].seq_global, 2);
        assert_eq!(result.events[0].workspace_id, "w1");
        assert_eq!(result.events[1].workspace_id, "w1");
        assert!(!result.events[0].event_id.is_empty());
        assert!(!result.events[0].timestamp.is_empty());

        let read = store.read_from("w1", 0, None).expect("read");
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].seq_global, 1);
        assert_eq!(read[1].seq_global, 2);
        assert_eq!(store.head_seq("w1").unwrap(), 2);
    }

    #[test]
    fn append_idempotency_replays_events() {
        let (_dir, mut store) = temp_store();
        let meta = command_meta("workspace.create", Some("ik_test"));
        let events = vec![workspace_event("w1", "alpha", "/tmp/alpha")];

        let first = store.append(&meta, events).expect("append");
        assert_eq!(first.events.len(), 1);
        assert!(!first.idempotent);

        let second = store
            .append(&meta, vec![workspace_event("w1", "beta", "/tmp/beta")])
            .expect("append idempotent");
        assert!(second.idempotent);
        assert_eq!(second.events.len(), 1);
        assert_eq!(first.events[0].event_id, second.events[0].event_id);
        assert_eq!(first.events[0].seq_global, second.events[0].seq_global);
        assert_eq!(store.head_seq("w1").unwrap(), 1);
    }

    #[test]
    fn read_from_respects_limit() {
        let (_dir, mut store) = temp_store();
        let meta = command_meta("workspace.create", None);
        let events = vec![
            workspace_event("w1", "alpha", "/tmp/alpha"),
            project_event("w1", "p1", "core"),
            project_event("w1", "p2", "api"),
        ];
        store.append(&meta, events).expect("append");

        let read = store.read_from("w1", 1, Some(1)).expect("read");
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].seq_global, 2);
    }

    #[test]
    fn projections_update_and_rebuild() {
        let (dir, mut store) = temp_store();
        let meta = command_meta("workspace.create", None);
        let events = vec![
            workspace_event("w1", "alpha", "/tmp/alpha"),
            project_event("w1", "p1", "core"),
        ];
        store.append(&meta, events).expect("append");

        let workspaces = store.list_workspaces().expect("workspaces");
        assert_eq!(workspaces.len(), 1);
        assert_eq!(workspaces[0].name, "alpha");

        let projects = store.list_projects("w1").expect("projects");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "core");

        let db_path = dir.path().join("mpd.sqlite");
        let conn = Connection::open(db_path).expect("conn");
        conn.execute(
            "UPDATE proj_workspaces SET name = ?1 WHERE workspace_id = ?2",
            params!["corrupt", "w1"],
        )
        .expect("corrupt");

        store.rebuild_projections().expect("rebuild");
        let workspaces = store.list_workspaces().expect("workspaces");
        assert_eq!(workspaces[0].name, "alpha");
    }
}
