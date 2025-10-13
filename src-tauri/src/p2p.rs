// P2P networking module using libp2p
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use std::collections::HashMap;
use std::time::Duration;
use futures::{StreamExt, FutureExt};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "P2PBehaviourEvent")]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
pub enum P2PBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
}

impl From<gossipsub::Event> for P2PBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        P2PBehaviourEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for P2PBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        P2PBehaviourEvent::Mdns(event)
    }
}

pub struct P2PNode {
    swarm: Swarm<P2PBehaviour>,
    topic: gossipsub::IdentTopic,
    connected_peers: HashMap<PeerId, String>,
}

impl P2PNode {
    pub async fn new() -> anyhow::Result<Self> {
        // Generate a keypair
        let id_keys = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());

        tracing::info!("Local peer id: {peer_id}");

        // Set up gossipsub config
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");

        // Build gossipsub behaviour
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        ).map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?;

        // Create a topic for env var sync
        let topic = gossipsub::IdentTopic::new("envmesh-sync");
        gossipsub.subscribe(&topic)?;

        // Set up mDNS for peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Create behaviour
        let behaviour = P2PBehaviour { gossipsub, mdns };

        // Build the Swarm
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on all interfaces
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self {
            swarm,
            topic,
            connected_peers: HashMap::new(),
        })
    }

    pub async fn publish(&mut self, message: Vec<u8>) -> anyhow::Result<()> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), message)?;
        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<(PeerId, String)> {
        self.connected_peers
            .iter()
            .map(|(id, addr)| (*id, addr.clone()))
            .collect()
    }

    pub async fn process_event(&mut self) -> Option<Vec<u8>> {
        if let Some(event) = self.swarm.next().now_or_never() {
            if let Some(event) = event {
                match event {
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message,
                        ..
                    })) => {
                        tracing::debug!("Received message from peer");
                        return Some(message.data);
                    }
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                        for (peer_id, multiaddr) in peers {
                            tracing::info!("Discovered peer: {peer_id} at {multiaddr}");
                            self.connected_peers.insert(peer_id, multiaddr.to_string());

                            // Dial the discovered peer
                            if let Err(e) = self.swarm.dial(multiaddr.clone()) {
                                tracing::warn!("Failed to dial {peer_id}: {e}");
                            }

                            // Subscribe to gossipsub
                            self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .add_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                        for (peer_id, _) in peers {
                            tracing::info!("Peer expired: {peer_id}");
                            self.connected_peers.remove(&peer_id);
                            self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!("Listening on {address}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        tracing::info!("Connected to {peer_id} at {}", endpoint.get_remote_address());
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        tracing::info!("Disconnected from {peer_id}");
                    }
                    _ => {}
                }
            }
        }
        None
    }
}
