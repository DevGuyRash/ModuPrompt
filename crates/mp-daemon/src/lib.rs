use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{sse::Event, sse::Sse},
    Json,
};
use futures::StreamExt;
use base64::Engine;
use mp_kernel::{
    command_kind, now_rfc3339, Actor, CommandKind, CommandRejectedPayload, DaemonPingResponse,
    ErrorCode, ProjectCreatePayload, RuntimeInfo, WorkspaceCreatePayload, EVENT_COMMAND_REJECTED,
    EVENT_PROJECT_CREATED, EVENT_WORKSPACE_CREATED,
};
use mp_protocol::{CommandEnvelope, CommandRejection, SchemaRegistry, SubmitCommandResponse};
use mp_storage::{CommandMeta, EventStore, NewEvent, ProjectionReader};
use mp_storage_sqlite::SqliteStore;
use rand::RngCore;
use serde::Deserialize;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone)]
pub struct DaemonConfig {
    pub db_path: PathBuf,
    pub addr: SocketAddr,
    pub runtime_dir: PathBuf,
    pub safe_mode: bool,
}

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<SqliteStore>>,
    schema_registry: Arc<SchemaRegistry>,
    broadcaster: broadcast::Sender<mp_protocol::EventEnvelope>,
    token: String,
    safe_mode: bool,
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    workspace_id: String,
    #[serde(default)]
    from: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProjectsQuery {
    workspace_id: String,
}

pub async fn run_daemon(config: DaemonConfig) -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let store = SqliteStore::open(&config.db_path)?;
    if !config.safe_mode {
        store.rebuild_projections()?;
    }

    let registry = SchemaRegistry::new()?;
    let (tx, _) = broadcast::channel(1024);

    let token = generate_token()?;
    let state = AppState {
        store: Arc::new(Mutex::new(store)),
        schema_registry: Arc::new(registry),
        broadcaster: tx,
        token: token.clone(),
        safe_mode: config.safe_mode,
    };

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    let local_addr = listener.local_addr()?;
    write_runtime_file(&config.runtime_dir, &token, local_addr, &config.db_path)?;

    let app = axum::Router::new()
        .route("/v1/daemon/ping", axum::routing::get(handle_ping))
        .route("/v1/commands/submit", axum::routing::post(handle_submit))
        .route("/v1/workspaces", axum::routing::get(handle_list_workspaces))
        .route("/v1/projects", axum::routing::get(handle_list_projects))
        .route("/v1/events", axum::routing::get(handle_events_read))
        .route("/v1/events/stream", axum::routing::get(handle_events_stream))
        .with_state(state);

    tracing::info!("mpd listening on {}", local_addr);

    let server = axum::serve(listener, app.into_make_service());

    tokio::select! {
        result = server => {
            if let Err(err) = result {
                tracing::error!("server error: {err}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutdown requested");
        }
    }

    cleanup_runtime_file(&config.runtime_dir);
    Ok(())
}

async fn handle_ping(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DaemonPingResponse>, StatusCode> {
    authorize(&state, &headers)?;
    Ok(Json(DaemonPingResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: now_rfc3339(),
    }))
}

async fn handle_list_workspaces(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<mp_kernel::WorkspaceListEntry>>, StatusCode> {
    authorize(&state, &headers)?;
    let store = state.store.lock().await;
    let workspaces = store
        .list_workspaces()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(workspaces))
}

async fn handle_list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProjectsQuery>,
) -> Result<Json<Vec<mp_kernel::ProjectListEntry>>, StatusCode> {
    authorize(&state, &headers)?;
    let store = state.store.lock().await;
    let projects = store
        .list_projects(&query.workspace_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(projects))
}

async fn handle_events_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<mp_protocol::EventEnvelope>>, StatusCode> {
    authorize(&state, &headers)?;
    let from = query.from.unwrap_or(0);
    let store = state.store.lock().await;
    let events = store
        .read_from(&query.workspace_id, from, None)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn handle_events_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>>, StatusCode>
{
    authorize(&state, &headers)?;
    let from = query.from.unwrap_or(0);
    let workspace_id = query.workspace_id.clone();

    let initial_events = {
        let store = state.store.lock().await;
        store
            .read_from(&workspace_id, from, None)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let mut last_seq = initial_events.last().map(|e| e.seq_global).unwrap_or(from);
    let rx = state.broadcaster.subscribe();
    let stream = async_stream::stream! {
        for event in initial_events {
            let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            last_seq = event.seq_global;
            yield Ok(Event::default().data(data));
        }

        let mut broadcast_stream = BroadcastStream::new(rx);
        while let Some(item) = broadcast_stream.next().await {
            if let Ok(event) = item {
                if event.workspace_id == workspace_id && event.seq_global > last_seq {
                    let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                    last_seq = event.seq_global;
                    yield Ok(Event::default().data(data));
                }
            }
        }
    };

    Ok(Sse::new(stream))
}

async fn handle_submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(command): Json<CommandEnvelope>,
) -> Result<Json<SubmitCommandResponse>, StatusCode> {
    authorize(&state, &headers)?;

    if state.safe_mode {
        let rejection = CommandRejection {
            code: "safe_mode".to_string(),
            message: "daemon running in safe mode".to_string(),
        };
        return Ok(Json(SubmitCommandResponse {
            accepted: false,
            events: Vec::new(),
            rejection: Some(rejection),
            trace_id: command.trace_id,
        }));
    }

    let command_type = command.command_type.clone();
    let command_kind = match command_kind(&command_type) {
        Some(kind) => kind,
        None => {
            return Ok(Json(
                reject_command(&state, &command, ErrorCode::UnknownCommand, "unknown command type")
                    .await?,
            ));
        }
    };

    if command_kind == CommandKind::ReadOnly {
        return Ok(Json(
            reject_command(
                &state,
                &command,
                ErrorCode::ValidationFailed,
                "read-only commands must use query endpoints",
            )
            .await?,
        ));
    }

    if command.idempotency_key.is_none() {
        return Ok(Json(
            reject_command(
                &state,
                &command,
                ErrorCode::IdempotencyKeyRequired,
                "idempotency_key required",
            )
            .await?,
        ));
    }

    if let Err(err) = state
        .schema_registry
        .validate_command_payload(&command_type, command.schema_version, &command.payload)
    {
        return Ok(Json(
            reject_command(&state, &command, ErrorCode::InvalidSchema, &err.message).await?,
        ));
    }

    let actor = Actor::system();
    let events = match command_type.as_str() {
        mp_kernel::COMMAND_WORKSPACE_CREATE => {
            let payload: WorkspaceCreatePayload = serde_json::from_value(command.payload.clone())
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            if let Some(expected_version) = command.expected_version {
                if expected_version != 0 {
                    return Ok(Json(
                        reject_command(
                            &state,
                            &command,
                            ErrorCode::ExpectedVersionMismatch,
                            &format!(
                                "expected version {expected_version} does not match 0 for new workspace"
                            ),
                        )
                        .await?,
                    ));
                }
            }
            let workspace_id = mp_kernel::new_uuid();
            let root_path = payload
                .path
                .clone()
                .unwrap_or_else(|| payload.name.clone());
            let payload_json = serde_json::to_value(mp_kernel::WorkspaceCreatedPayload {
                name: payload.name,
                root_path,
            })
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            vec![NewEvent {
                event_type: EVENT_WORKSPACE_CREATED.to_string(),
                schema_version: 1,
                actor,
                workspace_id: workspace_id.clone(),
                project_id: None,
                subject: mp_kernel::Subject {
                    kind: "workspace".to_string(),
                    id: workspace_id,
                },
                payload: payload_json,
                trace_id: Some(command.trace_id.clone()),
                stream_id: None,
            }]
        }
        mp_kernel::COMMAND_PROJECT_CREATE => {
            let payload: ProjectCreatePayload = serde_json::from_value(command.payload.clone())
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            if let Some(expected_version) = command.expected_version {
                let current = {
                    let store = state.store.lock().await;
                    store
                        .head_seq(&payload.workspace_id)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                };
                if current != expected_version {
                    return Ok(Json(
                        reject_command(
                            &state,
                            &command,
                            ErrorCode::ExpectedVersionMismatch,
                            &format!(
                                "expected version {expected_version} does not match {current}"
                            ),
                        )
                        .await?,
                    ));
                }
            }

            let project_id = mp_kernel::new_uuid();
            let payload_json = serde_json::to_value(mp_kernel::ProjectCreatedPayload {
                workspace_id: payload.workspace_id.clone(),
                name: payload.name,
            })
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            vec![NewEvent {
                event_type: EVENT_PROJECT_CREATED.to_string(),
                schema_version: 1,
                actor,
                workspace_id: payload.workspace_id.clone(),
                project_id: Some(project_id.clone()),
                subject: mp_kernel::Subject {
                    kind: "project".to_string(),
                    id: project_id,
                },
                payload: payload_json,
                trace_id: Some(command.trace_id.clone()),
                stream_id: None,
            }]
        }
        _ => {
            return Ok(Json(
                reject_command(&state, &command, ErrorCode::UnknownCommand, "unsupported command")
                    .await?,
            ));
        }
    };

    let meta = CommandMeta {
        command_type: command_type.clone(),
        idempotency_key: command.idempotency_key.clone(),
        expected_version: command.expected_version,
        trace_id: command.trace_id.clone(),
    };

    for event in &events {
        if let Err(err) = state
            .schema_registry
            .validate_event_payload(&event.event_type, event.schema_version, &event.payload)
        {
            return Ok(Json(
                reject_command(&state, &command, ErrorCode::InvalidSchema, &err.message).await?,
            ));
        }
    }

    let mut store = state.store.lock().await;
    let append_result = match store.append(&meta, events) {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("append failed: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    for event in &append_result.events {
        let _ = state.broadcaster.send(event.clone());
    }

    let rejection = extract_rejection(&append_result.events);
    Ok(Json(SubmitCommandResponse {
        accepted: rejection.is_none(),
        events: append_result.events,
        rejection,
        trace_id: command.trace_id,
    }))
}

fn extract_rejection(events: &[mp_protocol::EventEnvelope]) -> Option<CommandRejection> {
    events.iter().find_map(|event| {
        if event.event_type == EVENT_COMMAND_REJECTED {
            let payload: mp_kernel::CommandRejectedPayload =
                serde_json::from_value(event.payload.clone()).ok()?;
            Some(CommandRejection {
                code: payload.code.to_string(),
                message: payload.message,
            })
        } else {
            None
        }
    })
}

async fn reject_command(
    state: &AppState,
    command: &CommandEnvelope,
    code: ErrorCode,
    message: &str,
) -> Result<SubmitCommandResponse, StatusCode> {
    let workspace_id = command
        .payload
        .get("workspace_id")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "global".to_string());

    let payload = CommandRejectedPayload {
        command_type: command.command_type.clone(),
        code: code.clone(),
        message: message.to_string(),
        details: None,
    };
    let payload_json = serde_json::to_value(payload).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = NewEvent {
        event_type: EVENT_COMMAND_REJECTED.to_string(),
        schema_version: 1,
        actor: Actor::system(),
        workspace_id,
        project_id: None,
        subject: mp_kernel::Subject {
            kind: "command".to_string(),
            id: command.trace_id.clone(),
        },
        payload: payload_json,
        trace_id: Some(command.trace_id.clone()),
        stream_id: None,
    };

    let meta = CommandMeta {
        command_type: command.command_type.clone(),
        idempotency_key: command.idempotency_key.clone(),
        expected_version: command.expected_version,
        trace_id: command.trace_id.clone(),
    };

    let mut store = state.store.lock().await;
    let append_result = match store.append(&meta, vec![event]) {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("append failed: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    for event in &append_result.events {
        let _ = state.broadcaster.send(event.clone());
    }

    Ok(SubmitCommandResponse {
        accepted: false,
        events: append_result.events,
        rejection: Some(CommandRejection {
            code: code.to_string(),
            message: message.to_string(),
        }),
        trace_id: command.trace_id.clone(),
    })
}

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Ok(auth) = value.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let expected = format!("Bearer {}", state.token);
    if auth != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

fn generate_token() -> anyhow::Result<String> {
    let mut buf = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf))
}

fn runtime_file_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("daemon.json")
}

fn write_runtime_file(
    runtime_dir: &Path,
    token: &str,
    addr: SocketAddr,
    db_path: &Path,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(runtime_dir)?;
    set_private_dir_perms(runtime_dir)?;
    let info = RuntimeInfo {
        addr: format!("http://{}", addr),
        token: token.to_string(),
        pid: std::process::id(),
        db_path: db_path.display().to_string(),
        started_at: now_rfc3339(),
    };
    let payload = serde_json::to_vec_pretty(&info)?;
    let path = runtime_file_path(runtime_dir);
    std::fs::write(&path, payload)?;
    set_private_file_perms(&path)?;
    Ok(())
}

fn cleanup_runtime_file(runtime_dir: &Path) {
    let path = runtime_file_path(runtime_dir);
    let _ = std::fs::remove_file(path);
}

fn set_private_dir_perms(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn set_private_file_perms(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

pub fn default_runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("moduprompt");
    }
    home_dir().join(".moduprompt").join("run")
}

pub fn default_db_path() -> PathBuf {
    home_dir().join(".moduprompt").join("state").join("mpd.sqlite")
}

fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    PathBuf::from(".")
}
