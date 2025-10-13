// Library exports for CLI and daemon binaries
pub mod client;
pub mod server;
pub mod node;
pub mod election;
pub mod health;
pub mod storage;
pub mod crypto;
pub mod cli;
pub mod state;
pub mod api;
pub mod config;

// Re-export for convenience
pub use node::{EnvMeshNode, NodeConfig};
pub use storage::EnvStorage;
pub use crypto::Crypto;
pub use state::AppState;
pub use config::Config;
