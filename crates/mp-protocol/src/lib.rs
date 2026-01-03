use anyhow::Context;
use mp_kernel::{Actor, Subject};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

const COMMAND_WORKSPACE_CREATE_SCHEMA: &str =
    include_str!("../../../schemas/commands/workspace.create.v1.json");
const COMMAND_PROJECT_CREATE_SCHEMA: &str =
    include_str!("../../../schemas/commands/project.create.v1.json");

const EVENT_WORKSPACE_CREATED_SCHEMA: &str =
    include_str!("../../../schemas/events/workspace.created.v1.json");
const EVENT_PROJECT_CREATED_SCHEMA: &str =
    include_str!("../../../schemas/events/project.created.v1.json");
const EVENT_COMMAND_REJECTED_SCHEMA: &str =
    include_str!("../../../schemas/events/command.rejected.v1.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    #[serde(rename = "type")]
    pub command_type: String,
    pub schema_version: i32,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_version: Option<i64>,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub actor: Actor,
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub subject: Subject,
    pub payload: Value,
    pub schema_version: i32,
    pub seq_global: i64,
    pub seq_stream: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitCommandResponse {
    pub accepted: bool,
    pub events: Vec<EventEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection: Option<CommandRejection>,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandRejection {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdioFrame {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(rename = "type")]
    pub frame_type: String,
    pub schema_version: i32,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdioErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdioAuthPayload {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdioProjectsQuery {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdioEventsSubscribe {
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
}

#[derive(Debug)]
pub struct SchemaRegistry {
    command_schemas: HashMap<(String, i32), Value>,
    event_schemas: HashMap<(String, i32), Value>,
}

#[derive(Debug)]
pub struct SchemaError {
    pub message: String,
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SchemaError {}

impl SchemaRegistry {
    pub fn new() -> anyhow::Result<Self> {
        let mut command_schemas = HashMap::new();
        let mut event_schemas = HashMap::new();

        Self::insert_schema(
            &mut command_schemas,
            "workspace.create",
            1,
            COMMAND_WORKSPACE_CREATE_SCHEMA,
        )?;
        Self::insert_schema(
            &mut command_schemas,
            "project.create",
            1,
            COMMAND_PROJECT_CREATE_SCHEMA,
        )?;

        Self::insert_schema(
            &mut event_schemas,
            "workspace.created",
            1,
            EVENT_WORKSPACE_CREATED_SCHEMA,
        )?;
        Self::insert_schema(
            &mut event_schemas,
            "project.created",
            1,
            EVENT_PROJECT_CREATED_SCHEMA,
        )?;
        Self::insert_schema(
            &mut event_schemas,
            "command.rejected",
            1,
            EVENT_COMMAND_REJECTED_SCHEMA,
        )?;

        Ok(Self {
            command_schemas,
            event_schemas,
        })
    }

    fn insert_schema(
        map: &mut HashMap<(String, i32), Value>,
        command_type: &str,
        schema_version: i32,
        raw_schema: &str,
    ) -> anyhow::Result<()> {
        let schema_json: Value = serde_json::from_str(raw_schema)
            .with_context(|| format!("invalid JSON schema for {command_type} v{schema_version}"))?;
        map.insert((command_type.to_string(), schema_version), schema_json);
        Ok(())
    }

    pub fn validate_command_payload(
        &self,
        command_type: &str,
        schema_version: i32,
        payload: &Value,
    ) -> Result<(), SchemaError> {
        let schema = self
            .command_schemas
            .get(&(command_type.to_string(), schema_version))
            .ok_or_else(|| SchemaError {
                message: format!("schema not found for command {command_type} v{schema_version}"),
            })?;
        Self::validate(schema, payload)
    }

    pub fn validate_event_payload(
        &self,
        event_type: &str,
        schema_version: i32,
        payload: &Value,
    ) -> Result<(), SchemaError> {
        let schema = self
            .event_schemas
            .get(&(event_type.to_string(), schema_version))
            .ok_or_else(|| SchemaError {
                message: format!("schema not found for event {event_type} v{schema_version}"),
            })?;
        Self::validate(schema, payload)
    }

    fn validate(schema: &Value, payload: &Value) -> Result<(), SchemaError> {
        let compiled = jsonschema::JSONSchema::compile(schema).map_err(|err| SchemaError {
            message: format!("failed to compile schema: {err}"),
        })?;
        let result = compiled.validate(payload);
        if let Err(errors) = result {
            let mut messages = Vec::new();
            for error in errors {
                messages.push(error.to_string());
            }
            return Err(SchemaError {
                message: messages.join("; "),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaRegistry;
    use serde_json::json;

    #[test]
    fn schemas_compile() {
        let registry = SchemaRegistry::new();
        assert!(registry.is_ok());
    }

    #[test]
    fn command_schema_rejects_unknown_fields() {
        let registry = SchemaRegistry::new().expect("registry");
        let payload = json!({
            "name": "demo",
            "path": "./demo",
            "extra": "nope"
        });
        let result = registry.validate_command_payload("workspace.create", 1, &payload);
        assert!(result.is_err());
    }

    #[test]
    fn event_schema_rejects_unknown_fields() {
        let registry = SchemaRegistry::new().expect("registry");
        let payload = json!({
            "name": "demo",
            "root_path": "./demo",
            "extra": true
        });
        let result = registry.validate_event_payload("workspace.created", 1, &payload);
        assert!(result.is_err());
    }

    #[test]
    fn command_schema_requires_fields() {
        let registry = SchemaRegistry::new().expect("registry");
        let payload = json!({});
        let result = registry.validate_command_payload("workspace.create", 1, &payload);
        assert!(result.is_err());
    }
}
