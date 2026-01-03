use anyhow::Context;
use futures::StreamExt;
use clap::{Parser, Subcommand};
use mp_client::Client;
use mp_kernel::{ProjectListEntry, WorkspaceListEntry};
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
            EventCommands::Watch { workspace, from } => {
                let client = ensure_client().await?;
                let workspace_id = resolve_workspace_id(&client, &workspace).await?;
                watch_events(&client, &workspace_id, from).await?;
            }
        },
    }

    Ok(())
}

fn start_daemon() -> anyhow::Result<()> {
    let child = Command::new("mpd")
        .arg("start")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start mpd")?;
    let _ = child.id();
    Ok(())
}

async fn daemon_status(json: bool) -> anyhow::Result<()> {
    match try_client().await {
        Ok(client) => {
            let resp = client.ping().await?;
            if json {
                print_json(&resp)?;
            } else {
                println!("daemon: {} (version {} @ {})", resp.status, resp.version, resp.timestamp);
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

async fn ensure_client() -> anyhow::Result<Client> {
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
    Err(anyhow::anyhow!("daemon failed to start"))
}

async fn try_client() -> anyhow::Result<Client> {
    let runtime_dir = Client::default_runtime_dir();
    let client = Client::from_runtime_dir(&runtime_dir)?;
    client.ping().await?;
    Ok(client)
}

async fn resolve_workspace_id(client: &Client, selector: &str) -> anyhow::Result<String> {
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
    Err(anyhow::anyhow!(
        "workspace not found or ambiguous: {selector}"
    ))
}

async fn watch_events(client: &Client, workspace_id: &str, from: i64) -> anyhow::Result<()> {
    let resp = client.events_stream(workspace_id, from).await?;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
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

fn print_json<T: serde::Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}
