use clap::{Parser, Subcommand};
use mp_daemon::{default_db_path, default_runtime_dir, run_daemon, DaemonConfig};
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
    }
    Ok(())
}
