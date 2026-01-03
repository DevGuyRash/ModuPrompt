use mp_kernel::{
    ProjectCreatedPayload, WorkspaceCreatedPayload, EVENT_PROJECT_CREATED, EVENT_WORKSPACE_CREATED,
};
use mp_protocol::EventEnvelope;
use serde_json::from_value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("projection error: {0}")]
    Apply(String),
}

pub trait ProjectionWriter {
    fn reset(&self) -> Result<(), ProjectionError>;
    fn upsert_workspace(
        &self,
        workspace_id: &str,
        name: &str,
        root_path: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError>;
    fn upsert_project(
        &self,
        project_id: &str,
        workspace_id: &str,
        name: &str,
        created_at: &str,
        seq_global: i64,
    ) -> Result<(), ProjectionError>;
    fn set_meta(&self, workspace_id: &str, seq_global: i64) -> Result<(), ProjectionError>;
}

pub fn apply_event<W: ProjectionWriter>(
    writer: &W,
    event: &EventEnvelope,
) -> Result<(), ProjectionError> {
    match event.event_type.as_str() {
        EVENT_WORKSPACE_CREATED => {
            let payload: WorkspaceCreatedPayload =
                from_value(event.payload.clone()).map_err(|err| {
                    ProjectionError::Apply(format!("invalid workspace.created payload: {err}"))
                })?;
            writer.upsert_workspace(
                &event.workspace_id,
                &payload.name,
                &payload.root_path,
                &event.timestamp,
                event.seq_global,
            )?;
            writer.set_meta(&event.workspace_id, event.seq_global)?;
        }
        EVENT_PROJECT_CREATED => {
            let payload: ProjectCreatedPayload =
                from_value(event.payload.clone()).map_err(|err| {
                    ProjectionError::Apply(format!("invalid project.created payload: {err}"))
                })?;
            let project_id = event
                .project_id
                .as_ref()
                .ok_or_else(|| ProjectionError::Apply("project_id missing".to_string()))?;
            writer.upsert_project(
                project_id,
                &payload.workspace_id,
                &payload.name,
                &event.timestamp,
                event.seq_global,
            )?;
            writer.set_meta(&event.workspace_id, event.seq_global)?;
        }
        _ => {
            // Ignore events that do not affect projections.
        }
    }
    Ok(())
}

pub fn rebuild_projections<W, I>(writer: &W, events: I) -> Result<(), ProjectionError>
where
    W: ProjectionWriter,
    I: IntoIterator<Item = EventEnvelope>,
{
    writer.reset()?;
    for event in events {
        apply_event(writer, &event)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_kernel::{Actor, Subject};
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct RecordingWriter {
        resets: Cell<usize>,
        workspaces: RefCell<Vec<(String, String, String, String, i64)>>,
        projects: RefCell<Vec<(String, String, String, String, i64)>>,
        metas: RefCell<Vec<(String, i64)>>,
    }

    impl ProjectionWriter for RecordingWriter {
        fn reset(&self) -> Result<(), ProjectionError> {
            self.resets.set(self.resets.get() + 1);
            self.workspaces.borrow_mut().clear();
            self.projects.borrow_mut().clear();
            self.metas.borrow_mut().clear();
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
            self.workspaces.borrow_mut().push((
                workspace_id.to_string(),
                name.to_string(),
                root_path.to_string(),
                created_at.to_string(),
                seq_global,
            ));
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
            self.projects.borrow_mut().push((
                project_id.to_string(),
                workspace_id.to_string(),
                name.to_string(),
                created_at.to_string(),
                seq_global,
            ));
            Ok(())
        }

        fn set_meta(&self, workspace_id: &str, seq_global: i64) -> Result<(), ProjectionError> {
            self.metas
                .borrow_mut()
                .push((workspace_id.to_string(), seq_global));
            Ok(())
        }
    }

    fn workspace_event(seq_global: i64) -> EventEnvelope {
        EventEnvelope {
            event_id: "e1".to_string(),
            event_type: EVENT_WORKSPACE_CREATED.to_string(),
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            actor: Actor::system(),
            workspace_id: "w1".to_string(),
            project_id: None,
            subject: Subject {
                kind: "workspace".to_string(),
                id: "w1".to_string(),
            },
            payload: serde_json::json!({
                "name": "demo",
                "root_path": "/tmp/demo"
            }),
            schema_version: 1,
            seq_global,
            seq_stream: 1,
            trace_id: None,
        }
    }

    fn project_event(seq_global: i64) -> EventEnvelope {
        EventEnvelope {
            event_id: "e2".to_string(),
            event_type: EVENT_PROJECT_CREATED.to_string(),
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            actor: Actor::system(),
            workspace_id: "w1".to_string(),
            project_id: Some("p1".to_string()),
            subject: Subject {
                kind: "project".to_string(),
                id: "p1".to_string(),
            },
            payload: serde_json::json!({
                "workspace_id": "w1",
                "name": "core"
            }),
            schema_version: 1,
            seq_global,
            seq_stream: 1,
            trace_id: None,
        }
    }

    #[test]
    fn apply_event_records_workspace() {
        let writer = RecordingWriter::default();
        let event = workspace_event(3);
        apply_event(&writer, &event).expect("apply event");

        let workspaces = writer.workspaces.borrow();
        assert_eq!(workspaces.len(), 1);
        assert_eq!(
            workspaces[0],
            (
                "w1".to_string(),
                "demo".to_string(),
                "/tmp/demo".to_string(),
                "2020-01-01T00:00:00Z".to_string(),
                3
            )
        );

        let metas = writer.metas.borrow();
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0], ("w1".to_string(), 3));
    }

    #[test]
    fn apply_event_records_project() {
        let writer = RecordingWriter::default();
        let event = project_event(7);
        apply_event(&writer, &event).expect("apply event");

        let projects = writer.projects.borrow();
        assert_eq!(projects.len(), 1);
        assert_eq!(
            projects[0],
            (
                "p1".to_string(),
                "w1".to_string(),
                "core".to_string(),
                "2020-01-01T00:00:00Z".to_string(),
                7
            )
        );

        let metas = writer.metas.borrow();
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0], ("w1".to_string(), 7));
    }

    #[test]
    fn apply_event_rejects_invalid_payload() {
        let writer = RecordingWriter::default();
        let mut event = workspace_event(1);
        event.payload = serde_json::json!({
            "name": "demo"
        });

        let result = apply_event(&writer, &event);
        assert!(result.is_err());
    }

    #[test]
    fn apply_event_requires_project_id() {
        let writer = RecordingWriter::default();
        let mut event = project_event(2);
        event.project_id = None;

        let result = apply_event(&writer, &event);
        assert!(result.is_err());
    }

    #[test]
    fn rebuild_resets_and_applies() {
        let writer = RecordingWriter::default();
        let events = vec![workspace_event(1), project_event(2)];
        let result = rebuild_projections(&writer, events);
        assert!(result.is_ok());
        assert_eq!(writer.resets.get(), 1);
        assert_eq!(writer.workspaces.borrow().len(), 1);
        assert_eq!(writer.projects.borrow().len(), 1);
    }

    #[test]
    fn rebuild_ignores_unknown_events() {
        let writer = RecordingWriter::default();
        let event = EventEnvelope {
            event_id: "e2".to_string(),
            event_type: "unknown.event".to_string(),
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            actor: Actor::system(),
            workspace_id: "w1".to_string(),
            project_id: None,
            subject: Subject {
                kind: "unknown".to_string(),
                id: "x1".to_string(),
            },
            payload: serde_json::json!({}),
            schema_version: 1,
            seq_global: 1,
            seq_stream: 1,
            trace_id: None,
        };

        let result = rebuild_projections(&writer, vec![event]);
        assert!(result.is_ok());
    }
}
