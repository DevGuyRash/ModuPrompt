use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use futures::StreamExt;
use mp_client::{Client, ClientError, StdioAuthMode, StdioClient};
use mp_kernel::{ErrorCode, ProjectListEntry, WorkspaceListEntry};
use mp_protocol::{CommandRejection, ErrorResponse, SubmitCommandResponse};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[command(name = "mpctl", version, about = "ModuPrompt CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    Events {
        #[command(subcommand)]
        command: EventCommands,
    },
}

#[derive(Subcommand)]
enum DaemonCommands {
    Start,
    Status {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    Init {
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
    },
    List {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    Create {
        #[arg(long)]
        workspace: String,
        #[arg(long)]
        name: String,
    },
    List {
        #[arg(long)]
        workspace: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EventCommands {
    Watch {
        #[arg(long)]
        workspace: String,
        #[arg(long, default_value_t = 0)]
        from: i64,
        #[arg(long, value_enum, default_value_t = EventTransport::Sse)]
        transport: EventTransport,
        #[arg(long)]
        mpd_path: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EventTransport {
    Sse,
    Ndjson,
    Stdio,
}

#[derive(Debug, Clone)]
struct CliError {
    error: ErrorResponse,
}

impl CliError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: ErrorResponse {
                code,
                message: message.into(),
                details: None,
                trace_id: None,
            },
        }
    }

    fn from_error_response(error: ErrorResponse) -> Self {
        Self { error }
    }

    fn from_rejection(rejection: CommandRejection, trace_id: String) -> Self {
        Self {
            error: ErrorResponse {
                code: rejection.code,
                message: rejection.message,
                details: None,
                trace_id: Some(trace_id),
            },
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error.code, self.error.message)
    }
}

impl std::error::Error for CliError {}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(client_error) = err.downcast_ref::<ClientError>() {
            return CliError::from_error_response(client_error.error.clone());
        }
        CliError::new(ErrorCode::Unknown, err.to_string())
    }
}

type CliResult<T> = Result<T, CliError>;

fn exit_code_for_error_code(code: ErrorCode) -> i32 {
    match code {
        ErrorCode::InvalidSchema
        | ErrorCode::ValidationFailed
        | ErrorCode::IdempotencyKeyRequired
        | ErrorCode::UnknownCommand => 2,
        ErrorCode::ExpectedVersionMismatch => 3,
        ErrorCode::Unauthorized => 4,
        ErrorCode::NotFound => 5,
        ErrorCode::PolicyDenied => 6,
        ErrorCode::Unknown | ErrorCode::Internal => 1,
    }
}

fn wants_json(cli: &Cli) -> bool {
    match &cli.command {
        Commands::Daemon {
            command: DaemonCommands::Status { json },
        } => *json,
        Commands::Workspace {
            command: WorkspaceCommands::List { json },
        } => *json,
        Commands::Project {
            command: ProjectCommands::List { json, .. },
        } => *json,
        _ => false,
    }
}

fn render_error(err: &CliError, json: bool) {
    if json {
        match serde_json::to_string_pretty(&err.error) {
            Ok(payload) => println!("{payload}"),
            Err(_) => {
                let fallback = serde_json::json!({
                    "code": err.error.code,
                    "message": err.error.message,
                    "details": err.error.details,
                    "trace_id": err.error.trace_id,
                });
                match serde_json::to_string(&fallback) {
                    Ok(payload) => println!("{payload}"),
                    Err(_) => {
                        println!("{{\"code\":\"internal\",\"message\":\"serialization error\"}}")
                    }
                }
            }
        }
        return;
    }

    eprintln!("{}: {}", err.error.code, err.error.message);
    if let Some(trace_id) = &err.error.trace_id {
        eprintln!("trace_id: {trace_id}");
    }
}

fn ensure_command_accepted(response: SubmitCommandResponse) -> CliResult<SubmitCommandResponse> {
    if response.accepted {
        return Ok(response);
    }
    if let Some(rejection) = response.rejection {
        return Err(CliError::from_rejection(rejection, response.trace_id));
    }
    Err(CliError::new(
        ErrorCode::Unknown,
        "command rejected without details",
    ))
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let json = wants_json(&cli);
    if let Err(err) = run(cli).await {
        render_error(&err, json);
        std::process::exit(exit_code_for_error_code(err.error.code));
    }
}

async fn run(cli: Cli) -> CliResult<()> {
    match cli.command {
        Commands::Daemon { command } => match command {
            DaemonCommands::Start => {
                start_daemon()?;
            }
            DaemonCommands::Status { json } => {
                daemon_status(json).await?;
            }
        },
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init { path, name } => {
                let client = ensure_client().await?;
                let resolved_name = name.unwrap_or_else(|| {
                    path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "workspace".to_string())
                });
                let response = client
                    .workspace_create(resolved_name, Some(path.display().to_string()), None, None)
                    .await?;
                let response = ensure_command_accepted(response)?;
                print_json(&response)?;
            }
            WorkspaceCommands::List { json } => {
                let client = ensure_client().await?;
                let workspaces = client.workspace_list().await?;
                if json {
                    print_json(&workspaces)?;
                } else {
                    print_workspaces(&workspaces);
                }
            }
        },
        Commands::Project { command } => match command {
            ProjectCommands::Create { workspace, name } => {
                let client = ensure_client().await?;
                let workspace_id = resolve_workspace_id(&client, &workspace).await?;
                let response = client
                    .project_create(workspace_id, name, None, None)
                    .await?;
                let response = ensure_command_accepted(response)?;
                print_json(&response)?;
            }
            ProjectCommands::List { workspace, json } => {
                let client = ensure_client().await?;
                let workspace_id = resolve_workspace_id(&client, &workspace).await?;
                let projects = client.project_list(&workspace_id).await?;
                if json {
                    print_json(&projects)?;
                } else {
                    print_projects(&projects);
                }
            }
        },
        Commands::Events { command } => match command {
            EventCommands::Watch {
                workspace,
                from,
                transport,
                mpd_path,
                db,
            } => match transport {
                EventTransport::Sse => {
                    let client = ensure_client().await?;
                    let workspace_id = resolve_workspace_id(&client, &workspace).await?;
                    watch_events_sse(&client, &workspace_id, from).await?;
                }
                EventTransport::Ndjson => {
                    let client = ensure_client().await?;
                    let workspace_id = resolve_workspace_id(&client, &workspace).await?;
                    watch_events_ndjson(&client, &workspace_id, from).await?;
                }
                EventTransport::Stdio => {
                    watch_events_stdio(&workspace, from, mpd_path, db).await?;
                }
            },
        },
    }

    Ok(())
}

fn start_daemon() -> CliResult<()> {
    let child = Command::new("mpd")
        .arg("start")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start mpd")?;
    let _ = child.id();
    Ok(())
}

async fn daemon_status(json: bool) -> CliResult<()> {
    match try_client().await {
        Ok(client) => {
            let resp = client.ping().await?;
            if json {
                print_json(&resp)?;
            } else {
                println!(
                    "daemon: {} (version {} @ {})",
                    resp.status, resp.version, resp.timestamp
                );
            }
        }
        Err(err) => {
            if json {
                print_json(&serde_json::json!({"status": "down", "error": err.to_string()}))?;
            } else {
                println!("daemon: down ({err})");
            }
        }
    }
    Ok(())
}

async fn ensure_client() -> CliResult<Client> {
    if let Ok(client) = try_client().await {
        return Ok(client);
    }

    start_daemon()?;
    for _ in 0..20 {
        sleep(Duration::from_millis(150)).await;
        if let Ok(client) = try_client().await {
            return Ok(client);
        }
    }
    Err(CliError::new(ErrorCode::Unknown, "daemon failed to start"))
}

async fn try_client() -> CliResult<Client> {
    let runtime_dir = Client::default_runtime_dir();
    let client = Client::from_runtime_dir(&runtime_dir)?;
    client.ping().await?;
    Ok(client)
}

async fn resolve_workspace_id(client: &Client, selector: &str) -> CliResult<String> {
    let workspaces = client.workspace_list().await?;
    if workspaces.iter().any(|ws| ws.workspace_id == selector) {
        return Ok(selector.to_string());
    }
    let mut matches = workspaces
        .iter()
        .filter(|ws| ws.name == selector || ws.root_path == selector)
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        return Ok(matches.pop().unwrap().workspace_id.clone());
    }
    Err(CliError::new(
        ErrorCode::NotFound,
        format!("workspace not found or ambiguous: {selector}"),
    ))
}

async fn watch_events_sse(client: &Client, workspace_id: &str, from: i64) -> CliResult<()> {
    let resp = client.events_stream(workspace_id, from).await?;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| CliError::new(ErrorCode::Unknown, err.to_string()))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end().to_string();
            buffer = buffer[pos + 1..].to_string();
            if line.starts_with("data:") {
                let data = line.trim_start_matches("data:").trim();
                if !data.is_empty() {
                    println!("{data}");
                }
            }
        }
    }
    Ok(())
}

async fn watch_events_ndjson(client: &Client, workspace_id: &str, from: i64) -> CliResult<()> {
    let resp = client.events_stream_ndjson(workspace_id, from).await?;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| CliError::new(ErrorCode::Unknown, err.to_string()))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end().to_string();
            buffer = buffer[pos + 1..].to_string();
            if !line.is_empty() {
                println!("{line}");
            }
        }
    }
    Ok(())
}

async fn watch_events_stdio(
    workspace: &str,
    from: i64,
    mpd_path: Option<PathBuf>,
    db: Option<PathBuf>,
) -> CliResult<()> {
    let mpd_path = mpd_path.unwrap_or_else(|| PathBuf::from("mpd"));
    let mut client = StdioClient::spawn(&mpd_path, db, StdioAuthMode::None).await?;
    let result = async {
        let workspace_id = resolve_workspace_id_stdio(&mut client, workspace).await?;
        client.subscribe_events(&workspace_id, from).await?;

        loop {
            tokio::select! {
                event = client.next_event() => {
                    let event = event?;
                    let json = serde_json::to_string(&event)
                        .map_err(|err| CliError::new(ErrorCode::Unknown, err.to_string()))?;
                    println!("{json}");
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }
        Ok(())
    }
    .await;

    let shutdown_result = client.shutdown().await;
    if result.is_err() {
        let _ = shutdown_result;
        return result;
    }
    shutdown_result?;
    result
}

async fn resolve_workspace_id_stdio(client: &mut StdioClient, selector: &str) -> CliResult<String> {
    let workspaces = client.list_workspaces().await?;
    if workspaces.iter().any(|ws| ws.workspace_id == selector) {
        return Ok(selector.to_string());
    }
    let mut matches = workspaces
        .iter()
        .filter(|ws| ws.name == selector || ws.root_path == selector)
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        return Ok(matches.pop().unwrap().workspace_id.clone());
    }
    Err(CliError::new(
        ErrorCode::NotFound,
        format!("workspace not found or ambiguous: {selector}"),
    ))
}

fn print_workspaces(workspaces: &[WorkspaceListEntry]) {
    if workspaces.is_empty() {
        println!("no workspaces");
        return;
    }
    for workspace in workspaces {
        println!(
            "{}\t{}\t{}",
            workspace.workspace_id, workspace.name, workspace.root_path
        );
    }
}

fn print_projects(projects: &[ProjectListEntry]) {
    if projects.is_empty() {
        println!("no projects");
        return;
    }
    for project in projects {
        println!("{}\t{}", project.project_id, project.name);
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> CliResult<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| CliError::new(ErrorCode::Unknown, err.to_string()))?;
    println!("{json}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_kernel::ErrorCode;

    #[test]
    fn parse_events_watch_defaults() {
        let cli =
            Cli::try_parse_from(["mpctl", "events", "watch", "--workspace", "w1"]).expect("parse");
        match cli.command {
            Commands::Events { command } => match command {
                EventCommands::Watch {
                    workspace,
                    from,
                    transport,
                    mpd_path,
                    db,
                } => {
                    assert_eq!(workspace, "w1");
                    assert_eq!(from, 0);
                    assert!(matches!(transport, EventTransport::Sse));
                    assert!(mpd_path.is_none());
                    assert!(db.is_none());
                }
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn parse_events_watch_ndjson() {
        let cli = Cli::try_parse_from([
            "mpctl",
            "events",
            "watch",
            "--workspace",
            "w1",
            "--transport",
            "ndjson",
            "--from",
            "5",
        ])
        .expect("parse");
        match cli.command {
            Commands::Events { command } => match command {
                EventCommands::Watch {
                    transport, from, ..
                } => {
                    assert!(matches!(transport, EventTransport::Ndjson));
                    assert_eq!(from, 5);
                }
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn parse_events_watch_stdio() {
        let cli = Cli::try_parse_from([
            "mpctl",
            "events",
            "watch",
            "--workspace",
            "w1",
            "--transport",
            "stdio",
        ])
        .expect("parse");
        match cli.command {
            Commands::Events { command } => match command {
                EventCommands::Watch { transport, .. } => {
                    assert!(matches!(transport, EventTransport::Stdio));
                }
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn exit_code_mapping_is_stable() {
        assert_eq!(exit_code_for_error_code(ErrorCode::InvalidSchema), 2);
        assert_eq!(exit_code_for_error_code(ErrorCode::ValidationFailed), 2);
        assert_eq!(
            exit_code_for_error_code(ErrorCode::IdempotencyKeyRequired),
            2
        );
        assert_eq!(exit_code_for_error_code(ErrorCode::UnknownCommand), 2);
        assert_eq!(
            exit_code_for_error_code(ErrorCode::ExpectedVersionMismatch),
            3
        );
        assert_eq!(exit_code_for_error_code(ErrorCode::Unauthorized), 4);
        assert_eq!(exit_code_for_error_code(ErrorCode::NotFound), 5);
        assert_eq!(exit_code_for_error_code(ErrorCode::PolicyDenied), 6);
        assert_eq!(exit_code_for_error_code(ErrorCode::Unknown), 1);
        assert_eq!(exit_code_for_error_code(ErrorCode::Internal), 1);
    }
}
