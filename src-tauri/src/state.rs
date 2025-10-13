// Application state management
use crate::storage::EnvStorage;
use crate::node::{EnvMeshNode, NodeConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use uuid::Uuid;

pub struct AppState {
    pub storage: Arc<Mutex<EnvStorage>>,
    pub node: Arc<Mutex<EnvMeshNode>>,
    pub machine_id: String,
}

impl AppState {
    pub async fn new(db_path: std::path::PathBuf) -> Result<Self> {
        let storage = EnvStorage::new(db_path)?;

        // Configure node (use default config for now)
        let config = NodeConfig::default();
        let node = EnvMeshNode::new(config).await?;

        let machine_id = Uuid::new_v4().to_string();

        Ok(Self {
            storage: Arc::new(Mutex::new(storage)),
            node: Arc::new(Mutex::new(node)),
            machine_id,
        })
    }
}
