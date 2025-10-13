// Application state management
use crate::storage::EnvStorage;
use crate::p2p::P2PNode;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use uuid::Uuid;

pub struct AppState {
    pub storage: Arc<Mutex<EnvStorage>>,
    pub p2p: Arc<Mutex<P2PNode>>,
    pub machine_id: String,
}

impl AppState {
    pub async fn new(db_path: std::path::PathBuf) -> Result<Self> {
        let storage = EnvStorage::new(db_path)?;
        let p2p = P2PNode::new().await?;
        let machine_id = Uuid::new_v4().to_string();

        Ok(Self {
            storage: Arc::new(Mutex::new(storage)),
            p2p: Arc::new(Mutex::new(p2p)),
            machine_id,
        })
    }
}
