// CLI module for envmesh command-line tool
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "envmesh")]
#[command(about = "P2P mesh network for environment variable sync", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get an environment variable
    Get {
        /// The key to retrieve
        key: String,
    },
    /// Set an environment variable
    Set {
        /// The key to set
        key: String,
        /// The value to set
        value: String,
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
    /// Start the background daemon
    Daemon,
}
