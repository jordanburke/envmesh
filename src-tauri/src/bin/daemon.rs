// EnvMesh Daemon - Headless mode for WSL and servers
use envmesh::{EnvStorage, EnvMeshNode, Config};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use serde::{Deserialize, Serialize};
use clap::Parser;

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
    node: Arc<Mutex<EnvMeshNode>>,
    machine_id: String,
}

#[derive(Parser, Debug)]
#[command(name = "envmesh-daemon")]
#[command(about = "EnvMesh background daemon for environment variable sync", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command-line arguments
    let args = Args::parse();

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

    // Load configuration
    let config = if let Some(config_path) = args.config {
        println!("📄 Loading config from: {}", config_path.display());
        Config::from_file(&config_path)?
    } else {
        Config::load_default()?
    };

    // Initialize storage and node
    let storage = EnvStorage::new(db_path)?;
    let node_config = config.to_node_config();

    println!("⚙️  Configuration:");
    println!("   Server mode: {:?}", node_config.server_mode);
    println!("   Listen address: {}:{}", node_config.listen_addr, node_config.lan_port);
    println!("   Cloud enabled: {}", node_config.enable_cloud);
    if node_config.enable_cloud {
        println!("   Cloud URL: {}", node_config.cloud_url);
    }

    let node = EnvMeshNode::new(node_config).await?;
    let machine_id = uuid::Uuid::new_v4().to_string();

    let state = Arc::new(DaemonState {
        storage: Arc::new(Mutex::new(storage)),
        node: Arc::new(Mutex::new(node)),
        machine_id,
    });

    println!("✓ Storage initialized");
    println!("✓ Node initialized with failover support");
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
            let node = state.node.lock().await;
            let peers = node.get_peers();
            Response::Peers(peers)
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
