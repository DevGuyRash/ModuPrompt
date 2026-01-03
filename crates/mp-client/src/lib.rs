use anyhow::Context;
use mp_dirs::runtime_dir as default_runtime_dir_impl;
use mp_kernel::{
    DaemonPingResponse, ProjectCreatePayload, ProjectListEntry, RuntimeInfo,
    WorkspaceCreatePayload, WorkspaceListEntry,
};
use mp_protocol::{
    CommandEnvelope, StdioAuthPayload, StdioErrorPayload, StdioEventsSubscribe, StdioFrame,
    StdioProjectsQuery, SubmitCommandResponse,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client as HttpClient, Response};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command as TokioCommand};
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
        let data = std::fs::read(&path)
            .with_context(|| format!("failed to read runtime file at {}", path.display()))?;
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

    pub async fn events_stream(&self, workspace_id: &str, from: i64) -> anyhow::Result<Response> {
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

    pub async fn events_stream_ndjson(
        &self,
        workspace_id: &str,
        from: i64,
    ) -> anyhow::Result<Response> {
        let url = self.base_url.join(&format!(
            "/v1/events/stream-ndjson?workspace_id={workspace_id}&from={from}"
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

#[derive(Debug, Clone)]
pub enum StdioAuthMode {
    None,
    Token(String),
}

pub struct StdioClient {
    child: Child,
    reader: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    writer: BufWriter<tokio::process::ChildStdin>,
}

impl StdioClient {
    pub async fn spawn(
        mpd_path: &Path,
        db: Option<PathBuf>,
        auth: StdioAuthMode,
    ) -> anyhow::Result<Self> {
        let mut cmd = TokioCommand::new(mpd_path);
        cmd.arg("serve-stdio");
        match &auth {
            StdioAuthMode::None => {
                cmd.arg("--auth").arg("none");
            }
            StdioAuthMode::Token(token) => {
                cmd.arg("--auth").arg("token");
                cmd.arg("--token").arg(token);
            }
        }
        if let Some(db_path) = db {
            cmd.arg("--db").arg(db_path);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().context("failed to spawn mpd serve-stdio")?;
        let stdin = child.stdin.take().context("stdio stdin missing")?;
        let stdout = child.stdout.take().context("stdio stdout missing")?;

        let mut client = Self {
            child,
            reader: BufReader::new(stdout).lines(),
            writer: BufWriter::new(stdin),
        };

        if let StdioAuthMode::Token(token) = auth {
            client.auth(token).await?;
        }

        Ok(client)
    }

    pub async fn submit_command(
        &mut self,
        command: CommandEnvelope,
    ) -> anyhow::Result<SubmitCommandResponse> {
        let payload = serde_json::to_value(command)?;
        let response = self.request("command.submit", payload).await?.payload;
        Ok(serde_json::from_value(response)?)
    }

    pub async fn list_workspaces(&mut self) -> anyhow::Result<Vec<WorkspaceListEntry>> {
        let response = self
            .request("query.workspaces", serde_json::json!({}))
            .await?
            .payload;
        Ok(serde_json::from_value(response)?)
    }

    pub async fn list_projects(
        &mut self,
        workspace_id: &str,
    ) -> anyhow::Result<Vec<ProjectListEntry>> {
        let payload = serde_json::to_value(StdioProjectsQuery {
            workspace_id: workspace_id.to_string(),
        })?;
        let response = self.request("query.projects", payload).await?.payload;
        Ok(serde_json::from_value(response)?)
    }

    pub async fn subscribe_events(&mut self, workspace_id: &str, from: i64) -> anyhow::Result<()> {
        let payload = serde_json::to_value(StdioEventsSubscribe {
            workspace_id: workspace_id.to_string(),
            from: Some(from),
        })?;
        let _ = self.request("events.subscribe", payload).await?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> anyhow::Result<mp_protocol::EventEnvelope> {
        loop {
            let frame = self.read_frame().await?;
            if frame.frame_type == "events.event" {
                return Ok(serde_json::from_value(frame.payload)?);
            }
        }
    }

    pub async fn shutdown(mut self) -> anyhow::Result<()> {
        let _ = self.child.kill().await;
        Ok(())
    }

    async fn auth(&mut self, token: String) -> anyhow::Result<()> {
        let payload = serde_json::to_value(StdioAuthPayload { token })?;
        let _ = self.request("auth", payload).await?;
        Ok(())
    }

    async fn request(&mut self, frame_type: &str, payload: Value) -> anyhow::Result<StdioFrame> {
        let request_id = new_request_id();
        let frame = StdioFrame {
            request_id: Some(request_id.clone()),
            frame_type: frame_type.to_string(),
            schema_version: 1,
            payload,
        };
        self.send_frame(frame).await?;

        loop {
            let frame = self.read_frame().await?;
            if frame.request_id.as_deref() == Some(request_id.as_str()) {
                return Ok(frame);
            }
        }
    }

    async fn send_frame(&mut self, frame: StdioFrame) -> anyhow::Result<()> {
        let mut line = serde_json::to_string(&frame)?;
        line.push('\n');
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.flush().await?;
        Ok(())
    }

    async fn read_frame(&mut self) -> anyhow::Result<StdioFrame> {
        while let Some(line) = self.reader.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let frame: StdioFrame = serde_json::from_str(trimmed)?;
            if frame.frame_type == "error" {
                let payload: StdioErrorPayload = serde_json::from_value(frame.payload.clone())?;
                return Err(anyhow::anyhow!(payload.message));
            }
            return Ok(frame);
        }
        Err(anyhow::anyhow!("stdio connection closed"))
    }
}

fn new_request_id() -> String {
    format!("rq_{}", Uuid::now_v7())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_kernel::RuntimeInfo;
    use tempfile::TempDir;

    #[test]
    fn id_prefixes_are_applied() {
        let idempotency = new_idempotency_key();
        assert!(idempotency.starts_with("ik_"));
        assert!(Uuid::parse_str(&idempotency[3..]).is_ok());

        let trace = new_trace_id();
        assert!(trace.starts_with("tr_"));
        assert!(Uuid::parse_str(&trace[3..]).is_ok());

        let request = new_request_id();
        assert!(request.starts_with("rq_"));
        assert!(Uuid::parse_str(&request[3..]).is_ok());
    }

    #[test]
    fn auth_headers_include_bearer_token() {
        let client = Client {
            base_url: Url::parse("http://localhost:8080").expect("url"),
            token: "secret".to_string(),
            http: HttpClient::new(),
        };
        let headers = client.auth_headers();
        let token = headers
            .get(AUTHORIZATION)
            .expect("authorization")
            .to_str()
            .expect("header str");
        assert_eq!(token, "Bearer secret");
    }

    #[test]
    fn from_runtime_dir_reads_runtime_file() {
        let dir = TempDir::new().expect("tempdir");
        let info = RuntimeInfo {
            addr: "http://127.0.0.1:4242".to_string(),
            token: "tok_test".to_string(),
            pid: 123,
            db_path: "/tmp/mpd.sqlite".to_string(),
            started_at: "2020-01-01T00:00:00Z".to_string(),
        };
        let path = dir.path().join("daemon.json");
        std::fs::write(&path, serde_json::to_vec(&info).expect("json")).expect("write");

        let client = Client::from_runtime_dir(dir.path()).expect("client");
        assert_eq!(client.token, "tok_test");
        assert_eq!(client.base_url.as_str(), "http://127.0.0.1:4242/");
    }
}
