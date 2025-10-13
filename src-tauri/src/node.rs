// EnvMeshNode - Unified node that can be client or server
use anyhow::{anyhow, Result};
use std::time::Duration;

use crate::client::{SyncMessage, WebSocketClient};
use crate::election::{generate_peer_id, Election};
use crate::server::EmbeddedServer;

const DEFAULT_LAN_PORT: u16 = 8765;
const CLOUD_CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const LAN_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub enum NodeMode {
    CloudClient,
    LanClient { server_addr: String },
    LanServer { port: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerMode {
    /// Automatically decide role based on network (default)
    Auto,
    /// Prefer being a server (for cloud/VPS machines)
    ServerPreferred,
    /// Never become a server (client-only mode)
    ClientOnly,
}

impl Default for ServerMode {
    fn default() -> Self {
        Self::Auto
    }
}

pub struct EnvMeshNode {
    mode: NodeMode,
    client: Option<WebSocketClient>,
    server: Option<EmbeddedServer>,
    config: NodeConfig,
    peer_id: String,
}

#[derive(Clone)]
pub struct NodeConfig {
    pub cloud_url: String,
    pub lan_port: u16,
    pub listen_addr: String,
    pub enable_cloud: bool,
    pub enable_lan: bool,
    pub server_mode: ServerMode,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            cloud_url: "ws://localhost:8080".to_string(),
            lan_port: DEFAULT_LAN_PORT,
            listen_addr: "127.0.0.1".to_string(),
            enable_cloud: true,
            enable_lan: true,
            server_mode: ServerMode::default(),
        }
    }
}

impl EnvMeshNode {
    /// Create a new node with automatic failover
    pub async fn new(config: NodeConfig) -> Result<Self> {
        let peer_id = generate_peer_id();
        tracing::info!("Initializing EnvMesh node: {}", peer_id);

        let mut node = Self {
            mode: NodeMode::CloudClient,
            client: None,
            server: None,
            config,
            peer_id,
        };

        // Try to connect with failover
        node.reconnect_with_failover().await?;

        Ok(node)
    }

    /// Try to connect with automatic failover logic
    pub async fn reconnect_with_failover(&mut self) -> Result<()> {
        // Step 1: Try cloud server (if enabled)
        if self.config.enable_cloud {
            tracing::info!("Attempting to connect to cloud server...");
            match tokio::time::timeout(
                CLOUD_CONNECTION_TIMEOUT,
                WebSocketClient::connect(&self.config.cloud_url),
            )
            .await
            {
                Ok(Ok(client)) => {
                    tracing::info!("Connected to cloud server");
                    self.mode = NodeMode::CloudClient;
                    self.client = Some(client);
                    self.server = None;
                    return Ok(());
                }
                Ok(Err(e)) => {
                    tracing::warn!("Cloud server connection failed: {}", e);
                }
                Err(_) => {
                    tracing::warn!("Cloud server connection timeout");
                }
            }
        }

        // Step 2: Try to discover LAN server (if enabled)
        if self.config.enable_lan {
            tracing::info!("Searching for LAN server...");
            let election = Election::new(self.peer_id.clone());

            match tokio::time::timeout(LAN_DISCOVERY_TIMEOUT, election.discover_lan_server()).await
            {
                Ok(Ok(Some(server_info))) => {
                    let lan_url = format!("ws://{}:{}", server_info.address, server_info.port);
                    tracing::info!("Found LAN server at {}", lan_url);

                    match WebSocketClient::connect(&lan_url).await {
                        Ok(client) => {
                            tracing::info!("Connected to LAN server");
                            self.mode = NodeMode::LanClient {
                                server_addr: lan_url.clone(),
                            };
                            self.client = Some(client);
                            self.server = None;
                            return Ok(());
                        }
                        Err(e) => {
                            tracing::warn!("Failed to connect to LAN server: {}", e);
                        }
                    }
                }
                Ok(Ok(None)) => {
                    tracing::info!("No LAN server found");
                }
                Ok(Err(e)) => {
                    tracing::warn!("LAN server discovery error: {}", e);
                }
                Err(_) => {
                    tracing::debug!("LAN server discovery timeout");
                }
            }

            // Step 3: Become LAN server (if allowed by server_mode)
            if self.config.server_mode == ServerMode::ClientOnly {
                return Err(anyhow!(
                    "No server available and server_mode is ClientOnly"
                ));
            }

            tracing::info!("No server available, running election...");

            // ServerPreferred mode: Always try to become server
            let should_become_server = if self.config.server_mode == ServerMode::ServerPreferred {
                tracing::info!("ServerPreferred mode: becoming server immediately");
                true
            } else {
                // Auto mode: Run election
                match election.should_become_server().await {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(anyhow!("Election failed: {}", e));
                    }
                }
            };

            if should_become_server {
                tracing::info!("Elected as LAN server");
                let bind_addr = format!("{}:{}", self.config.listen_addr, self.config.lan_port);
                let server = EmbeddedServer::start(self.config.lan_port).await?;
                let port = server.port();

                // Announce via mDNS
                election.announce_as_server(port).await?;

                self.mode = NodeMode::LanServer { port };
                self.server = Some(server);
                self.client = None;

                tracing::info!("Now running as LAN server on {} (port {})", bind_addr, port);
                return Ok(());
            } else {
                tracing::info!("Lost election, another node is the server");
                // Wait a bit and retry discovery
                tokio::time::sleep(Duration::from_secs(1)).await;
                // Use Box::pin to handle recursive call
                return Box::pin(self.reconnect_with_failover()).await;
            }
        }

        Err(anyhow!(
            "Failed to connect to any server and LAN mode is disabled"
        ))
    }

    /// Send an update to peers (broadcast if server, send if client)
    pub async fn send_update(&mut self, msg: &SyncMessage) -> Result<()> {
        match &mut self.client {
            Some(client) => {
                client.send(msg.clone()).await?;
            }
            None => {
                if let Some(server) = &self.server {
                    server.broadcast(msg).await?;
                } else {
                    return Err(anyhow!("Not connected to any server"));
                }
            }
        }
        Ok(())
    }

    /// Receive updates from the network
    pub async fn receive_update(&mut self) -> Result<Option<SyncMessage>> {
        if let Some(client) = &mut self.client {
            client.receive().await
        } else {
            // Server mode doesn't receive from network, only broadcasts
            Ok(None)
        }
    }

    /// Get current node mode
    pub fn current_mode(&self) -> NodeMode {
        self.mode.clone()
    }

    /// Get connection info for display
    pub fn connection_info(&self) -> String {
        match &self.mode {
            NodeMode::CloudClient => format!("Connected to cloud: {}", self.config.cloud_url),
            NodeMode::LanClient { server_addr } => format!("Connected to LAN server: {}", server_addr),
            NodeMode::LanServer { port } => {
                let active = self.server.as_ref().map(|_| 0).unwrap_or(0);
                format!("Running as LAN server on port {} ({} clients)", port, active)
            }
        }
    }

    /// Get list of connected peers (for UI)
    pub fn get_peers(&self) -> Vec<(String, String)> {
        match &self.mode {
            NodeMode::CloudClient => vec![("cloud".to_string(), self.config.cloud_url.clone())],
            NodeMode::LanClient { server_addr } => {
                vec![("lan-server".to_string(), server_addr.clone())]
            }
            NodeMode::LanServer { port } => {
                vec![("self".to_string(), format!("LAN Server on port {}", port))]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_config_default() {
        let config = NodeConfig::default();
        assert!(config.enable_cloud);
        assert!(config.enable_lan);
        assert_eq!(config.lan_port, DEFAULT_LAN_PORT);
    }
}
