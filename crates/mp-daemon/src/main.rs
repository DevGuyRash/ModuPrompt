use clap::{Parser, Subcommand, ValueEnum};
use mp_daemon::{
    default_db_path, default_runtime_dir, run_daemon, run_stdio, DaemonConfig, StdioAuth,
    StdioConfig,
};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mpd", version, about = "ModuPrompt daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(long, default_value = "127.0.0.1:7331")]
        addr: SocketAddr,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        runtime_dir: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        safe_mode: bool,
    },
    ServeStdio {
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        runtime_dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = AuthMode::Token)]
        auth: AuthMode,
        #[arg(long)]
        token: Option<String>,
        #[arg(long, default_value_t = false)]
        safe_mode: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum AuthMode {
    Token,
    None,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start {
            addr,
            db,
            runtime_dir,
            safe_mode,
        } => {
            let config = DaemonConfig {
                db_path: db.unwrap_or_else(default_db_path),
                addr,
                runtime_dir: runtime_dir.unwrap_or_else(default_runtime_dir),
                safe_mode,
            };
            run_daemon(config).await?;
        }
        Commands::ServeStdio {
            db,
            runtime_dir,
            auth,
            token,
            safe_mode,
        } => {
            let auth = match auth {
                AuthMode::None => StdioAuth::None,
                AuthMode::Token => {
                    let token =
                        token.ok_or_else(|| anyhow::anyhow!("--token required for auth=token"))?;
                    StdioAuth::Token(token)
                }
            };
            let config = DaemonConfig {
                db_path: db.unwrap_or_else(default_db_path),
                addr: "127.0.0.1:0".parse::<SocketAddr>()?,
                runtime_dir: runtime_dir.unwrap_or_else(default_runtime_dir),
                safe_mode,
            };
            run_stdio(config, StdioConfig { auth }).await?;
        }
    }
    Ok(())
}
