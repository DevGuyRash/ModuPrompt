use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

pub const COMMAND_DAEMON_PING: &str = "daemon.ping";
pub const COMMAND_WORKSPACE_CREATE: &str = "workspace.create";
pub const COMMAND_WORKSPACE_LIST: &str = "workspace.list";
pub const COMMAND_PROJECT_CREATE: &str = "project.create";
pub const COMMAND_PROJECT_LIST: &str = "project.list";
pub const COMMAND_EVENTS_READ_FROM: &str = "events.read_from";
pub const COMMAND_EVENTS_SUBSCRIBE: &str = "events.subscribe";

pub const EVENT_WORKSPACE_CREATED: &str = "workspace.created";
pub const EVENT_PROJECT_CREATED: &str = "project.created";
pub const EVENT_COMMAND_REJECTED: &str = "command.rejected";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    ReadOnly,
    StateChanging,
}

pub fn command_kind(command_type: &str) -> Option<CommandKind> {
    match command_type {
        COMMAND_DAEMON_PING
        | COMMAND_WORKSPACE_LIST
        | COMMAND_PROJECT_LIST
        | COMMAND_EVENTS_READ_FROM
        | COMMAND_EVENTS_SUBSCRIBE => Some(CommandKind::ReadOnly),
        COMMAND_WORKSPACE_CREATE | COMMAND_PROJECT_CREATE => Some(CommandKind::StateChanging),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidSchema,
    UnknownCommand,
    IdempotencyKeyRequired,
    ExpectedVersionMismatch,
    ValidationFailed,
    Unauthorized,
    NotFound,
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorCode::InvalidSchema => "invalid_schema",
            ErrorCode::UnknownCommand => "unknown_command",
            ErrorCode::IdempotencyKeyRequired => "idempotency_key_required",
            ErrorCode::ExpectedVersionMismatch => "expected_version_mismatch",
            ErrorCode::ValidationFailed => "validation_failed",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::NotFound => "not_found",
            ErrorCode::Internal => "internal",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    pub kind: String,
    pub id: String,
    pub label: Option<String>,
}

impl Actor {
    pub fn system() -> Self {
        Self {
            kind: "system".to_string(),
            id: "system".to_string(),
            label: Some("mpd".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCreatePayload {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCreatePayload {
    pub workspace_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceCreatedPayload {
    pub name: String,
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCreatedPayload {
    pub workspace_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandRejectedPayload {
    pub command_type: String,
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaemonPingResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceListEntry {
    pub workspace_id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub seq_global: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectListEntry {
    pub project_id: String,
    pub workspace_id: String,
    pub name: String,
    pub created_at: String,
    pub seq_global: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeInfo {
    pub addr: String,
    pub token: String,
    pub pid: u32,
    pub db_path: String,
    pub started_at: String,
}

pub fn new_uuid() -> String {
    Uuid::now_v7().to_string()
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_kind_classifies() {
        assert_eq!(command_kind(COMMAND_DAEMON_PING), Some(CommandKind::ReadOnly));
        assert_eq!(
            command_kind(COMMAND_WORKSPACE_CREATE),
            Some(CommandKind::StateChanging)
        );
        assert_eq!(command_kind("unknown.command"), None);
    }

    #[test]
    fn error_code_display_matches_wire_format() {
        assert_eq!(ErrorCode::InvalidSchema.to_string(), "invalid_schema");
        assert_eq!(ErrorCode::UnknownCommand.to_string(), "unknown_command");
        assert_eq!(ErrorCode::IdempotencyKeyRequired.to_string(), "idempotency_key_required");
        assert_eq!(
            ErrorCode::ExpectedVersionMismatch.to_string(),
            "expected_version_mismatch"
        );
        assert_eq!(ErrorCode::ValidationFailed.to_string(), "validation_failed");
        assert_eq!(ErrorCode::Unauthorized.to_string(), "unauthorized");
        assert_eq!(ErrorCode::NotFound.to_string(), "not_found");
        assert_eq!(ErrorCode::Internal.to_string(), "internal");
    }

    #[test]
    fn actor_rejects_unknown_fields() {
        let json = r#"{"kind":"system","id":"system","label":null,"extra":true}"#;
        let result: Result<Actor, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn new_uuid_is_valid() {
        let id = new_uuid();
        assert!(Uuid::parse_str(&id).is_ok());
    }

    #[test]
    fn now_rfc3339_parses() {
        let ts = now_rfc3339();
        let parsed = OffsetDateTime::parse(&ts, &time::format_description::well_known::Rfc3339);
        assert!(parsed.is_ok());
    }
}
