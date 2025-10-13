// Configuration module for EnvMesh
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::node::{NodeConfig, ServerMode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub client: ClientConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server mode: auto, server-preferred, or client-only
    #[serde(default)]
    pub mode: String,

    /// Address to listen on (e.g., "127.0.0.1" for local, "0.0.0.0" for public)
    #[serde(default = "default_listen_addr")]
    pub listen: String,

    /// Port to listen on
    #[serde(default = "default_lan_port")]
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    /// URL of cloud server to connect to
    #[serde(default = "default_cloud_url")]
    pub cloud_url: String,

    /// Enable cloud server connection
    #[serde(default = "default_true")]
    pub enable_cloud: bool,

    /// Enable LAN server discovery and creation
    #[serde(default = "default_true")]
    pub enable_lan: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            client: ClientConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            listen: default_listen_addr(),
            port: default_lan_port(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            cloud_url: default_cloud_url(),
            enable_cloud: true,
            enable_lan: true,
        }
    }
}

fn default_listen_addr() -> String {
    "127.0.0.1".to_string()
}

fn default_lan_port() -> u16 {
    8765
}

fn default_cloud_url() -> String {
    "ws://localhost:8080".to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Try to load configuration from default locations
    pub fn load_default() -> Result<Self> {
        // Try ~/.envmesh/config.toml
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(".envmesh").join("config.toml");
            if config_path.exists() {
                tracing::info!("Loading config from {}", config_path.display());
                return Self::from_file(&config_path);
            }
        }

        // Try system config directory
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("envmesh").join("config.toml");
            if config_path.exists() {
                tracing::info!("Loading config from {}", config_path.display());
                return Self::from_file(&config_path);
            }
        }

        // Return default config if no file found
        tracing::info!("No config file found, using defaults");
        Ok(Self::default())
    }

    /// Convert to NodeConfig
    pub fn to_node_config(&self) -> NodeConfig {
        let server_mode = match self.server.mode.to_lowercase().as_str() {
            "server-preferred" | "server_preferred" => ServerMode::ServerPreferred,
            "client-only" | "client_only" => ServerMode::ClientOnly,
            _ => ServerMode::Auto,
        };

        NodeConfig {
            cloud_url: self.client.cloud_url.clone(),
            lan_port: self.server.port,
            listen_addr: self.server.listen.clone(),
            enable_cloud: self.client.enable_cloud,
            enable_lan: self.client.enable_lan,
            server_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.mode, "auto");
        assert_eq!(config.server.listen, "127.0.0.1");
        assert_eq!(config.server.port, 8765);
        assert!(config.client.enable_cloud);
        assert!(config.client.enable_lan);
    }

    #[test]
    fn test_server_mode_parsing() {
        let config = Config {
            server: ServerConfig {
                mode: "server-preferred".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let node_config = config.to_node_config();
        assert_eq!(node_config.server_mode, ServerMode::ServerPreferred);
    }
}
