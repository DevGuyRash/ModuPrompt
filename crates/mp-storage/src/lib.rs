use mp_kernel::{Actor, ProjectListEntry, Subject, WorkspaceListEntry};
use mp_protocol::EventEnvelope;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CommandMeta {
    pub command_type: String,
    pub idempotency_key: Option<String>,
    pub expected_version: Option<i64>,
    pub trace_id: String,
}

#[derive(Debug, Clone)]
pub struct NewEvent {
    pub event_type: String,
    pub schema_version: i32,
    pub actor: Actor,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub subject: Subject,
    pub payload: Value,
    pub trace_id: Option<String>,
    pub stream_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppendResult {
    pub events: Vec<EventEnvelope>,
    pub idempotent: bool,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("internal: {0}")]
    Internal(String),
}

pub trait EventStore {
    fn append(
        &mut self,
        meta: &CommandMeta,
        events: Vec<NewEvent>,
    ) -> Result<AppendResult, StoreError>;
    fn read_from(
        &self,
        workspace_id: &str,
        from_seq: i64,
        limit: Option<i64>,
    ) -> Result<Vec<EventEnvelope>, StoreError>;
    fn head_seq(&self, workspace_id: &str) -> Result<i64, StoreError>;
}

pub trait ProjectionReader {
    fn list_workspaces(&self) -> Result<Vec<WorkspaceListEntry>, StoreError>;
    fn list_projects(&self, workspace_id: &str) -> Result<Vec<ProjectListEntry>, StoreError>;
}
