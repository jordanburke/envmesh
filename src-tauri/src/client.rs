// WebSocket client for connecting to cloud or LAN servers
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
    pub machine_id: String,
    pub deleted: bool,
}

pub struct WebSocketClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    server_url: String,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self> {
        tracing::info!("Connecting to server: {}", url);

        let (stream, _) = connect_async(url)
            .await
            .map_err(|e| anyhow!("Failed to connect to {}: {}", url, e))?;

        tracing::info!("Connected to server: {}", url);

        Ok(Self {
            stream,
            server_url: url.to_string(),
        })
    }

    pub async fn send(&mut self, msg: SyncMessage) -> Result<()> {
        let json = serde_json::to_string(&msg)?;
        self.stream
            .send(Message::Text(json))
            .await
            .map_err(|e| anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<SyncMessage>> {
        match self.stream.next().await {
            Some(Ok(Message::Text(text))) => {
                let msg: SyncMessage = serde_json::from_str(&text)?;
                Ok(Some(msg))
            }
            Some(Ok(Message::Close(_))) => {
                tracing::warn!("Server closed connection");
                Ok(None)
            }
            Some(Err(e)) => Err(anyhow!("WebSocket error: {}", e)),
            None => Ok(None),
            _ => Ok(None),
        }
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    pub async fn ping(&mut self) -> Result<()> {
        self.stream
            .send(Message::Ping(vec![]))
            .await
            .map_err(|e| anyhow!("Ping failed: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_message_serialization() {
        let msg = SyncMessage {
            key: "TEST_KEY".to_string(),
            value: "test_value".to_string(),
            timestamp: 1234567890,
            machine_id: "machine-1".to_string(),
            deleted: false,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: SyncMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.key, deserialized.key);
        assert_eq!(msg.value, deserialized.value);
    }
}
