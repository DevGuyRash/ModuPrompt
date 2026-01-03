use anyhow::Context;
use mp_kernel::{
    DaemonPingResponse, ProjectCreatePayload, ProjectListEntry, RuntimeInfo,
    WorkspaceCreatePayload, WorkspaceListEntry,
};
use mp_dirs::runtime_dir as default_runtime_dir_impl;
use mp_protocol::{CommandEnvelope, SubmitCommandResponse};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client as HttpClient, Response};
use serde_json::Value;
use std::path::{Path, PathBuf};
use url::Url;
use uuid::Uuid;

pub struct Client {
    base_url: Url,
    token: String,
    http: HttpClient,
}

impl Client {
    pub fn from_runtime_dir(runtime_dir: &Path) -> anyhow::Result<Self> {
        let path = runtime_dir.join("daemon.json");
        let data = std::fs::read(&path).with_context(|| {
            format!("failed to read runtime file at {}", path.display())
        })?;
        let info: RuntimeInfo = serde_json::from_slice(&data)?;
        let base_url = Url::parse(&info.addr)?;
        Ok(Self {
            base_url,
            token: info.token,
            http: HttpClient::new(),
        })
    }

    pub fn default_runtime_dir() -> PathBuf {
        default_runtime_dir_impl()
    }

    pub async fn ping(&self) -> anyhow::Result<DaemonPingResponse> {
        let url = self.base_url.join("/v1/daemon/ping")?;
        let resp = self
            .http
            .get(url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn workspace_create(
        &self,
        name: String,
        path: Option<String>,
        idempotency_key: Option<String>,
        expected_version: Option<i64>,
    ) -> anyhow::Result<SubmitCommandResponse> {
        let payload = WorkspaceCreatePayload { name, path };
        self.submit_command(
            "workspace.create",
            1,
            serde_json::to_value(payload)?,
            idempotency_key,
            expected_version,
        )
        .await
    }

    pub async fn project_create(
        &self,
        workspace_id: String,
        name: String,
        idempotency_key: Option<String>,
        expected_version: Option<i64>,
    ) -> anyhow::Result<SubmitCommandResponse> {
        let payload = ProjectCreatePayload { workspace_id, name };
        self.submit_command(
            "project.create",
            1,
            serde_json::to_value(payload)?,
            idempotency_key,
            expected_version,
        )
        .await
    }

    pub async fn workspace_list(&self) -> anyhow::Result<Vec<WorkspaceListEntry>> {
        let url = self.base_url.join("/v1/workspaces")?;
        let resp = self
            .http
            .get(url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn project_list(&self, workspace_id: &str) -> anyhow::Result<Vec<ProjectListEntry>> {
        let url = self
            .base_url
            .join(&format!("/v1/projects?workspace_id={workspace_id}"))?;
        let resp = self
            .http
            .get(url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn events_read_from(
        &self,
        workspace_id: &str,
        from: i64,
    ) -> anyhow::Result<Vec<mp_protocol::EventEnvelope>> {
        let url = self.base_url.join(&format!(
            "/v1/events?workspace_id={workspace_id}&from={from}"
        ))?;
        let resp = self
            .http
            .get(url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn events_stream(
        &self,
        workspace_id: &str,
        from: i64,
    ) -> anyhow::Result<Response> {
        let url = self.base_url.join(&format!(
            "/v1/events/stream?workspace_id={workspace_id}&from={from}"
        ))?;
        let resp = self
            .http
            .get(url)
            .headers(self.auth_headers())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp)
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token = format!("Bearer {}", self.token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
        headers
    }

    async fn submit_command(
        &self,
        command_type: &str,
        schema_version: i32,
        payload: Value,
        idempotency_key: Option<String>,
        expected_version: Option<i64>,
    ) -> anyhow::Result<SubmitCommandResponse> {
        let url = self.base_url.join("/v1/commands/submit")?;
        let envelope = CommandEnvelope {
            command_type: command_type.to_string(),
            schema_version,
            payload,
            idempotency_key: Some(idempotency_key.unwrap_or_else(new_idempotency_key)),
            expected_version,
            trace_id: new_trace_id(),
        };
        let resp = self
            .http
            .post(url)
            .headers(self.auth_headers())
            .json(&envelope)
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}

fn new_idempotency_key() -> String {
    format!("ik_{}", Uuid::now_v7())
}

fn new_trace_id() -> String {
    format!("tr_{}", Uuid::now_v7())
}
