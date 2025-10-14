// EnvMesh CLI - Command-line interface for interacting with daemon
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[cfg(unix)]
use std::path::PathBuf;

#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::TcpStream;

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

#[derive(Parser)]
#[command(name = "envmesh-cli")]
#[command(about = "P2P mesh network for environment variable sync", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get an environment variable
    Get {
        /// The key to retrieve
        key: String,
    },
    /// Set an environment variable
    Set {
        /// The key to set (format: KEY=value or just KEY with separate value argument)
        key: String,
        /// The value to set (optional if using KEY=value format)
        value: Option<String>,
    },
    /// Delete an environment variable
    Delete {
        /// The key to delete
        key: String,
    },
    /// List all environment variables
    List,
    /// Export variables in shell format
    Export {
        /// Shell format (bash, zsh, powershell)
        #[arg(short, long, default_value = "bash")]
        shell: String,
    },
    /// Show connected peers
    Peers,
    /// Trigger manual sync
    Sync,
    /// Shutdown the daemon
    Shutdown,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    #[cfg(unix)]
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("envmesh");

    #[cfg(unix)]
    let socket_path = data_dir.join("daemon.sock");

    // Platform-specific connection
    #[cfg(unix)]
    {
        // Check if daemon is running
        if !socket_path.exists() {
            eprintln!("❌ Daemon not running");
            eprintln!("\nStart the daemon first:");
            eprintln!("  envmesh-daemon");
            std::process::exit(1);
        }

        // Connect to daemon
        let stream = UnixStream::connect(&socket_path).await?;
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);

        execute_command(cli.command, socket_path, reader, writer).await?;
    }

    #[cfg(windows)]
    {
        // Connect to TCP daemon
        let stream = match TcpStream::connect("127.0.0.1:37842").await {
            Ok(s) => s,
            Err(_) => {
                eprintln!("❌ Daemon not running");
                eprintln!("\nStart the daemon first:");
                eprintln!("  envmesh-daemon");
                std::process::exit(1);
            }
        };

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        execute_command_windows(cli.command, reader, writer).await?;
    }

    Ok(())
}

#[cfg(unix)]
async fn execute_command(
    cli_command: Commands,
    socket_path: PathBuf,
    mut reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    mut writer: tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<()> {
    // Send command
    let command = match cli_command {
        Commands::Get { key } => Command::Get { key },
        Commands::Set { key, value } => {
            // Parse KEY=value format
            if let Some(val) = value {
                Command::Set { key, value: val }
            } else if let Some(eq_pos) = key.find('=') {
                let (k, v) = key.split_at(eq_pos);
                Command::Set {
                    key: k.to_string(),
                    value: v[1..].to_string(),
                }
            } else {
                eprintln!("❌ Invalid format. Use: envmesh-cli set KEY value");
                eprintln!("   or: envmesh-cli set KEY=value");
                std::process::exit(1);
            }
        }
        Commands::Delete { key } => Command::Delete { key },
        Commands::List => Command::List,
        Commands::Export { shell } => {
            // Handle export locally
            handle_export(socket_path, &shell).await?;
            return Ok(());
        }
        Commands::Peers => Command::Peers,
        Commands::Sync => Command::Sync,
        Commands::Shutdown => Command::Shutdown,
    };

    let cmd_json = serde_json::to_string(&command)?;
    writer.write_all(cmd_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Response = serde_json::from_str(&response_line)?;

    // Handle response
    handle_response(response);

    Ok(())
}

#[cfg(windows)]
async fn execute_command_windows(
    cli_command: Commands,
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    mut writer: tokio::net::tcp::OwnedWriteHalf,
) -> anyhow::Result<()> {
    // Send command
    let command = match cli_command {
        Commands::Get { key } => Command::Get { key },
        Commands::Set { key, value } => {
            // Parse KEY=value format
            if let Some(val) = value {
                Command::Set { key, value: val }
            } else if let Some(eq_pos) = key.find('=') {
                let (k, v) = key.split_at(eq_pos);
                Command::Set {
                    key: k.to_string(),
                    value: v[1..].to_string(),
                }
            } else {
                eprintln!("❌ Invalid format. Use: envmesh-cli set KEY value");
                eprintln!("   or: envmesh-cli set KEY=value");
                std::process::exit(1);
            }
        }
        Commands::Delete { key } => Command::Delete { key },
        Commands::List => Command::List,
        Commands::Export { shell } => {
            // Handle export locally
            handle_export_windows(&shell).await?;
            return Ok(());
        }
        Commands::Peers => Command::Peers,
        Commands::Sync => Command::Sync,
        Commands::Shutdown => Command::Shutdown,
    };

    let cmd_json = serde_json::to_string(&command)?;
    writer.write_all(cmd_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // Read response
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Response = serde_json::from_str(&response_line)?;

    // Handle response
    handle_response(response);

    Ok(())
}

fn handle_response(response: Response) {
    match response {
        Response::Value(Some(value)) => {
            println!("{}", value);
        }
        Response::Value(None) => {
            eprintln!("❌ Key not found");
            std::process::exit(1);
        }
        Response::Success => {
            println!("✓ Success");
        }
        Response::Error(msg) => {
            eprintln!("❌ Error: {}", msg);
            std::process::exit(1);
        }
        Response::List(vars) => {
            if vars.is_empty() {
                println!("No environment variables");
            } else {
                for (key, value) in vars {
                    println!("{}={}", key, value);
                }
            }
        }
        Response::Peers(peers) => {
            if peers.is_empty() {
                println!("No connected peers");
            } else {
                for (id, addr) in peers {
                    println!("{} @ {}", id, addr);
                }
            }
        }
    }
}

#[cfg(unix)]
async fn handle_export(socket_path: PathBuf, shell: &str) -> anyhow::Result<()> {
    // Connect and get list
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let command = Command::List;
    let cmd_json = serde_json::to_string(&command)?;
    writer.write_all(cmd_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Response = serde_json::from_str(&response_line)?;

    match response {
        Response::List(vars) => {
            for (key, value) in vars {
                match shell {
                    "powershell" | "pwsh" => {
                        println!("$env:{}=\"{}\"", key, value);
                    }
                    "fish" => {
                        println!("set -gx {} \"{}\"", key, value);
                    }
                    _ => {
                        // bash, zsh, sh
                        println!("export {}=\"{}\"", key, value);
                    }
                }
            }
        }
        Response::Error(msg) => {
            eprintln!("# Error: {}", msg);
            std::process::exit(1);
        }
        _ => {
            eprintln!("# Unexpected response");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(windows)]
async fn handle_export_windows(shell: &str) -> anyhow::Result<()> {
    // Connect and get list
    let stream = TcpStream::connect("127.0.0.1:37842").await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let command = Command::List;
    let cmd_json = serde_json::to_string(&command)?;
    writer.write_all(cmd_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Response = serde_json::from_str(&response_line)?;

    match response {
        Response::List(vars) => {
            for (key, value) in vars {
                match shell {
                    "powershell" | "pwsh" => {
                        println!("$env:{}=\"{}\"", key, value);
                    }
                    "fish" => {
                        println!("set -gx {} \"{}\"", key, value);
                    }
                    _ => {
                        // bash, zsh, sh
                        println!("export {}=\"{}\"", key, value);
                    }
                }
            }
        }
        Response::Error(msg) => {
            eprintln!("# Error: {}", msg);
            std::process::exit(1);
        }
        _ => {
            eprintln!("# Unexpected response");
            std::process::exit(1);
        }
    }

    Ok(())
}
