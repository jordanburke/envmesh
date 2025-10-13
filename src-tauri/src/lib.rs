// Library exports for CLI and daemon binaries
pub mod api;
pub mod cli;
pub mod client;
pub mod config;
pub mod crypto;
pub mod election;
pub mod health;
pub mod node;
pub mod server;
pub mod state;
pub mod storage;

// Re-export for convenience
pub use config::Config;
pub use crypto::Crypto;
pub use node::{EnvMeshNode, NodeConfig};
pub use state::AppState;
pub use storage::EnvStorage;
