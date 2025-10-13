// Health monitoring and automatic failover/failback
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;

use crate::node::{EnvMeshNode, NodeMode};

pub struct HealthMonitor {
    cloud_url: String,
    check_interval: Duration,
    failure_threshold: u32,
}

impl HealthMonitor {
    pub fn new(cloud_url: String) -> Self {
        Self {
            cloud_url,
            check_interval: Duration::from_secs(30),
            failure_threshold: 3,
        }
    }

    /// Start monitoring in the background
    pub fn start_monitoring(self, node: Arc<Mutex<EnvMeshNode>>) {
        tokio::spawn(async move {
            self.monitor_loop(node).await;
        });
    }

    async fn monitor_loop(&self, node: Arc<Mutex<EnvMeshNode>>) {
        let mut interval = interval(self.check_interval);
        let mut failure_count = 0;

        loop {
            interval.tick().await;

            let current_mode = {
                let n = node.lock().await;
                n.current_mode()
            };

            match current_mode {
                NodeMode::CloudClient => {
                    // Check if cloud is still healthy
                    if !self.is_cloud_healthy().await {
                        failure_count += 1;
                        tracing::warn!(
                            "Cloud server health check failed ({}/{})",
                            failure_count,
                            self.failure_threshold
                        );

                        if failure_count >= self.failure_threshold {
                            tracing::error!("Cloud server down, initiating failover");
                            if let Err(e) = self.failover_to_lan(Arc::clone(&node)).await {
                                tracing::error!("Failover failed: {}", e);
                            }
                            failure_count = 0;
                        }
                    } else {
                        failure_count = 0;
                    }
                }
                NodeMode::LanClient { .. } | NodeMode::LanServer { .. } => {
                    // Check if cloud came back online
                    if self.is_cloud_healthy().await {
                        tracing::info!("Cloud server restored, initiating failback");
                        if let Err(e) = self.failback_to_cloud(Arc::clone(&node)).await {
                            tracing::error!("Failback failed: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn is_cloud_healthy(&self) -> bool {
        // Try to connect to cloud server with timeout
        match tokio::time::timeout(
            Duration::from_secs(5),
            crate::client::WebSocketClient::connect(&self.cloud_url),
        )
        .await
        {
            Ok(Ok(_)) => {
                tracing::debug!("Cloud server is healthy");
                true
            }
            Ok(Err(e)) => {
                tracing::debug!("Cloud server unreachable: {}", e);
                false
            }
            Err(_) => {
                tracing::debug!("Cloud server connection timeout");
                false
            }
        }
    }

    async fn failover_to_lan(&self, node: Arc<Mutex<EnvMeshNode>>) -> Result<()> {
        let mut n = node.lock().await;
        n.reconnect_with_failover().await?;
        tracing::info!("Failover to LAN completed");
        Ok(())
    }

    async fn failback_to_cloud(&self, node: Arc<Mutex<EnvMeshNode>>) -> Result<()> {
        let mut n = node.lock().await;

        // Sync local state to cloud first if we're the LAN server
        if matches!(n.current_mode(), NodeMode::LanServer { .. }) {
            tracing::info!("Syncing local state to cloud before failback");
            // TODO: Implement state sync
        }

        // Reconnect to cloud
        n.reconnect_with_failover().await?;
        tracing::info!("Failback to cloud completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new("ws://localhost:8080".to_string());
        assert_eq!(monitor.failure_threshold, 3);
        assert_eq!(monitor.check_interval, Duration::from_secs(30));
    }
}
