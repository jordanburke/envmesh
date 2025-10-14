// Embedded WebSocket server that runs when client becomes the LAN server
use anyhow::{anyhow, Result};
use futures_util::SinkExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::client::SyncMessage;

type WsStream = WebSocketStream<TcpStream>;

pub struct EmbeddedServer {
    connections: Arc<Mutex<Vec<WsStream>>>,
    port: u16,
    _shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl EmbeddedServer {
    pub async fn start(port: u16) -> Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", addr, e))?;

        // Get the actual bound port (important when port=0 for random port)
        let actual_port = listener.local_addr()?.port();

        tracing::info!("LAN server listening on 0.0.0.0:{}", actual_port);

        let connections = Arc::new(Mutex::new(Vec::new()));
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        // Spawn connection acceptor
        let conns = Arc::clone(&connections);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                tracing::info!("Client connected: {}", addr);
                                if let Err(e) = Self::handle_connection(stream, addr, Arc::clone(&conns)).await {
                                    tracing::error!("Connection error: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            connections,
            port: actual_port,
            _shutdown_tx: shutdown_tx,
        })
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        connections: Arc<Mutex<Vec<WsStream>>>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream)
            .await
            .map_err(|e| anyhow!("WebSocket handshake failed: {}", e))?;

        tracing::info!("WebSocket connection established: {}", addr);

        // Add to connections list
        connections.lock().await.push(ws_stream);

        Ok(())
    }

    pub async fn broadcast(&self, msg: &SyncMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        let message = Message::Text(json);

        let mut conns = self.connections.lock().await;
        let mut i = 0;

        // Remove closed connections and send to active ones
        while i < conns.len() {
            match conns[i].send(message.clone()).await {
                Ok(_) => {
                    i += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to send to client, removing: {}", e);
                    conns.remove(i);
                }
            }
        }

        tracing::debug!("Broadcasted to {} clients", conns.len());
        Ok(())
    }

    pub async fn active_connections(&self) -> usize {
        self.connections.lock().await.len()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for EmbeddedServer {
    fn drop(&mut self) {
        tracing::info!("Embedded server shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts() {
        let server = EmbeddedServer::start(0).await.unwrap(); // Port 0 = random
        assert!(server.port() > 0);
        assert_eq!(server.active_connections().await, 0);
    }
}
