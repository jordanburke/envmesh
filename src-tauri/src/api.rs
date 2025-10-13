use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMessage {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub machine_id: String,
    pub deleted: bool,
}

#[tauri::command]
pub async fn get_env_var(key: String, state: State<'_, AppState>) -> Result<Option<EnvVar>, String> {
    let storage = state.storage.lock().await;

    match storage.get(&key) {
        Ok(Some((value, timestamp, machine_id))) => Ok(Some(EnvVar {
            key,
            value,
            timestamp,
            machine_id,
        })),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get env var: {}", e)),
    }
}

#[tauri::command]
pub async fn set_env_var(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.storage.lock().await;

    storage.set(&key, &value, &state.machine_id)
        .map_err(|e| format!("Failed to set env var: {}", e))?;

    // Broadcast change to peers
    let timestamp = chrono::Utc::now().timestamp();
    let msg = SyncMessage {
        key: key.clone(),
        value: value.clone(),
        timestamp,
        machine_id: state.machine_id.clone(),
        deleted: false,
    };

    let msg_bytes = serde_json::to_vec(&msg)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    let mut p2p = state.p2p.lock().await;
    p2p.publish(msg_bytes).await
        .map_err(|e| format!("Failed to broadcast: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_env_var(key: String, state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.storage.lock().await;

    storage.delete(&key, &state.machine_id)
        .map_err(|e| format!("Failed to delete env var: {}", e))?;

    // Broadcast deletion to peers
    let timestamp = chrono::Utc::now().timestamp();
    let msg = SyncMessage {
        key: key.clone(),
        value: String::new(),
        timestamp,
        machine_id: state.machine_id.clone(),
        deleted: true,
    };

    let msg_bytes = serde_json::to_vec(&msg)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    let mut p2p = state.p2p.lock().await;
    p2p.publish(msg_bytes).await
        .map_err(|e| format!("Failed to broadcast: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_env_vars(state: State<'_, AppState>) -> Result<Vec<EnvVar>, String> {
    let storage = state.storage.lock().await;

    let vars = storage.list_all()
        .map_err(|e| format!("Failed to list env vars: {}", e))?;

    Ok(vars.into_iter().map(|(key, value, timestamp, machine_id)| EnvVar {
        key,
        value,
        timestamp,
        machine_id,
    }).collect())
}

#[tauri::command]
pub async fn get_peers(state: State<'_, AppState>) -> Result<Vec<Peer>, String> {
    let p2p = state.p2p.lock().await;

    let peers = p2p.get_connected_peers();

    Ok(peers.into_iter().map(|(id, addr)| Peer {
        id: id.to_string(),
        address: addr,
        last_seen: chrono::Utc::now().timestamp(),
    }).collect())
}

#[tauri::command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.storage.lock().await;
    let changes = storage.get_changes_since(0)
        .map_err(|e| format!("Failed to get changes: {}", e))?;

    drop(storage);

    for (key, value, timestamp, machine_id, deleted) in changes {
        let msg = SyncMessage {
            key,
            value,
            timestamp,
            machine_id,
            deleted,
        };

        let msg_bytes = serde_json::to_vec(&msg)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let mut p2p = state.p2p.lock().await;
        p2p.publish(msg_bytes).await
            .map_err(|e| format!("Failed to broadcast: {}", e))?;
    }

    Ok(())
}
