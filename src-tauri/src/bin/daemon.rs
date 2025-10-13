// EnvMesh Daemon - Headless mode for WSL and servers
use envmesh::{EnvStorage, P2PNode};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    List,
    Peers,
    Sync,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
enum Response {
    Value(Option<String>),
    Success,
    Error(String),
    List(Vec<(String, String)>),
    Peers(Vec<(String, String)>),
}

struct DaemonState {
    storage: Arc<Mutex<EnvStorage>>,
    p2p: Arc<Mutex<P2PNode>>,
    machine_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 EnvMesh Daemon Starting...");

    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("envmesh");

    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("envmesh.db");
    let socket_path = data_dir.join("daemon.sock");

    // Remove old socket if exists
    let _ = std::fs::remove_file(&socket_path);

    println!("📁 Database: {}", db_path.display());
    println!("🔌 Socket: {}", socket_path.display());

    // Initialize storage and P2P
    let storage = EnvStorage::new(db_path)?;
    let p2p = P2PNode::new().await?;
    let machine_id = uuid::Uuid::new_v4().to_string();

    let state = Arc::new(DaemonState {
        storage: Arc::new(Mutex::new(storage)),
        p2p: Arc::new(Mutex::new(p2p)),
        machine_id,
    });

    println!("✓ Storage initialized");
    println!("✓ P2P node initialized");
    println!("\n📡 Daemon running. Use 'envmesh-cli' to interact.");
    println!("Press Ctrl+C to stop.\n");

    // Setup Unix socket listener
    let listener = UnixListener::bind(&socket_path)?;

    // Handle connections
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: Arc<DaemonState>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let cmd: Command = match serde_json::from_str(&line) {
            Ok(cmd) => cmd,
            Err(e) => {
                let resp = Response::Error(format!("Invalid command: {}", e));
                writer.write_all(serde_json::to_string(&resp)?.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                line.clear();
                continue;
            }
        };

        let response = handle_command(cmd, &state).await;
        writer.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        line.clear();
    }

    Ok(())
}

async fn handle_command(cmd: Command, state: &DaemonState) -> Response {
    match cmd {
        Command::Get { key } => {
            let storage = state.storage.lock().await;
            match storage.get(&key) {
                Ok(Some((value, _, _))) => Response::Value(Some(value)),
                Ok(None) => Response::Value(None),
                Err(e) => Response::Error(format!("Failed to get: {}", e)),
            }
        }
        Command::Set { key, value } => {
            let storage = state.storage.lock().await;
            match storage.set(&key, &value, &state.machine_id) {
                Ok(_) => Response::Success,
                Err(e) => Response::Error(format!("Failed to set: {}", e)),
            }
        }
        Command::Delete { key } => {
            let storage = state.storage.lock().await;
            match storage.delete(&key, &state.machine_id) {
                Ok(_) => Response::Success,
                Err(e) => Response::Error(format!("Failed to delete: {}", e)),
            }
        }
        Command::List => {
            let storage = state.storage.lock().await;
            match storage.list_all() {
                Ok(vars) => {
                    let list: Vec<(String, String)> = vars
                        .into_iter()
                        .map(|(k, v, _, _)| (k, v))
                        .collect();
                    Response::List(list)
                }
                Err(e) => Response::Error(format!("Failed to list: {}", e)),
            }
        }
        Command::Peers => {
            let p2p = state.p2p.lock().await;
            let peers = p2p.get_connected_peers();
            Response::Peers(peers.into_iter().map(|(id, addr)| (id.to_string(), addr)).collect())
        }
        Command::Sync => {
            // TODO: Implement sync
            Response::Success
        }
        Command::Shutdown => {
            std::process::exit(0);
        }
    }
}
