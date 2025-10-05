use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub machine_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub address: String,
    pub last_seen: i64,
}

#[tauri::command]
pub async fn get_env_var(key: String) -> Result<Option<EnvVar>, String> {
    // TODO: Implement storage retrieval
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn set_env_var(key: String, value: String) -> Result<(), String> {
    // TODO: Implement storage and sync
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn delete_env_var(key: String) -> Result<(), String> {
    // TODO: Implement deletion and sync
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn list_env_vars() -> Result<Vec<EnvVar>, String> {
    // TODO: Implement listing
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn get_peers() -> Result<Vec<Peer>, String> {
    // TODO: Implement peer discovery
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn trigger_sync() -> Result<(), String> {
    // TODO: Implement manual sync trigger
    Err("Not implemented yet".to_string())
}
