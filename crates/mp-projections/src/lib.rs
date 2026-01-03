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

pub fn apply_event<W: ProjectionWriter>(writer: &W, event: &EventEnvelope) -> Result<(), ProjectionError> {
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
            let payload: ProjectCreatedPayload = from_value(event.payload.clone()).map_err(|err| {
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
