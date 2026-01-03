use futures::StreamExt;
use mp_client::Client;
use mp_daemon::{run_daemon, DaemonConfig};
use std::net::SocketAddr;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

async fn wait_for_client(runtime_dir: &std::path::Path) -> anyhow::Result<Client> {
    for _ in 0..20 {
        if let Ok(client) = Client::from_runtime_dir(runtime_dir) {
            if client.ping().await.is_ok() {
                return Ok(client);
            }
        }
        sleep(Duration::from_millis(150)).await;
    }
    Err(anyhow::anyhow!("daemon did not become ready"))
}

#[tokio::test(flavor = "multi_thread")]
async fn workspace_persists_across_restart() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mpd.sqlite");
    let runtime_dir = temp.path().join("run");

    let config = DaemonConfig {
        db_path: db_path.clone(),
        addr: "127.0.0.1:0".parse::<SocketAddr>()?,
        runtime_dir: runtime_dir.clone(),
        safe_mode: false,
    };

    let handle = tokio::spawn(run_daemon(config.clone()));
    let client = wait_for_client(&runtime_dir).await?;

    let create = client
        .workspace_create("demo".to_string(), Some("./demo".to_string()), None, None)
        .await?;
    assert!(create.accepted);

    let list = client.workspace_list().await?;
    assert_eq!(list.len(), 1);

    handle.abort();
    sleep(Duration::from_millis(200)).await;

    let handle2 = tokio::spawn(run_daemon(config));
    let client2 = wait_for_client(&runtime_dir).await?;
    let list2 = client2.workspace_list().await?;
    assert_eq!(list2.len(), 1);

    handle2.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn idempotency_reuses_events() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mpd.sqlite");
    let runtime_dir = temp.path().join("run");

    let config = DaemonConfig {
        db_path,
        addr: "127.0.0.1:0".parse::<SocketAddr>()?,
        runtime_dir: runtime_dir.clone(),
        safe_mode: false,
    };

    let handle = tokio::spawn(run_daemon(config));
    let client = wait_for_client(&runtime_dir).await?;

    let first = client
        .workspace_create(
            "demo".to_string(),
            Some("./demo".to_string()),
            Some("ik_test".to_string()),
            None,
        )
        .await?;
    let second = client
        .workspace_create(
            "demo".to_string(),
            Some("./demo".to_string()),
            Some("ik_test".to_string()),
            None,
        )
        .await?;

    assert_eq!(first.events.len(), 1);
    assert_eq!(second.events.len(), 1);
    assert_eq!(first.events[0].event_id, second.events[0].event_id);

    handle.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn expected_version_mismatch_rejects() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mpd.sqlite");
    let runtime_dir = temp.path().join("run");

    let config = DaemonConfig {
        db_path,
        addr: "127.0.0.1:0".parse::<SocketAddr>()?,
        runtime_dir: runtime_dir.clone(),
        safe_mode: false,
    };

    let handle = tokio::spawn(run_daemon(config));
    let client = wait_for_client(&runtime_dir).await?;

    client
        .workspace_create("demo".to_string(), Some("./demo".to_string()), None, None)
        .await?;
    let workspaces = client.workspace_list().await?;
    let workspace_id = workspaces[0].workspace_id.clone();

    let response = client
        .project_create(workspace_id, "core".to_string(), None, Some(999))
        .await?;
    assert!(!response.accepted);
    assert!(response.rejection.is_some());

    handle.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn event_stream_catches_up() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mpd.sqlite");
    let runtime_dir = temp.path().join("run");

    let config = DaemonConfig {
        db_path,
        addr: "127.0.0.1:0".parse::<SocketAddr>()?,
        runtime_dir: runtime_dir.clone(),
        safe_mode: false,
    };

    let handle = tokio::spawn(run_daemon(config));
    let client = wait_for_client(&runtime_dir).await?;

    let create = client
        .workspace_create("demo".to_string(), Some("./demo".to_string()), None, None)
        .await?;
    let workspace_id = create.events[0].workspace_id.clone();

    let response = client.events_stream(&workspace_id, 0).await?;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut saw_event = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim_end().to_string();
            buffer = buffer[pos + 1..].to_string();
            if line.starts_with("data:") {
                let data = line.trim_start_matches("data:").trim();
                if data.contains("workspace.created") {
                    saw_event = true;
                    break;
                }
            }
        }
        if saw_event {
            break;
        }
    }

    assert!(saw_event);
    handle.abort();
    Ok(())
}
