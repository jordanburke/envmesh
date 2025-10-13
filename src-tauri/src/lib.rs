// Library exports for CLI and daemon binaries
pub mod p2p;
pub mod storage;
pub mod crypto;
pub mod cli;
pub mod state;
pub mod api;

// Re-export for convenience
pub use p2p::P2PNode;
pub use storage::EnvStorage;
pub use crypto::Crypto;
pub use state::AppState;
