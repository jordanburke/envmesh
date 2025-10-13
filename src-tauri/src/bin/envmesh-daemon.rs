// EnvMesh daemon - runs without GUI (perfect for WSL)
use envmesh::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("EnvMesh Daemon Starting...");

    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("envmesh");

    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("envmesh.db");

    println!("Database: {}", db_path.display());

    // Initialize storage and P2P
    let storage = storage::EnvStorage::new(db_path)?;
    let mut p2p = p2p::P2PNode::new().await?;

    println!("✓ Storage initialized");
    println!("✓ P2P node initialized");
    println!("\nDaemon running. Press Ctrl+C to stop.");
    println!("\nCommands:");
    println!("  Use another terminal to interact with EnvMesh");
    println!("  Example: envmesh-cli set MY_VAR=value");

    // Keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
