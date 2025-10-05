// P2P networking module using libp2p
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct P2PNode {
    swarm: Swarm<P2PBehaviour>,
    topic: gossipsub::IdentTopic,
}

impl P2PNode {
    pub async fn new() -> anyhow::Result<Self> {
        // Generate a keypair
        let id_keys = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());

        println!("Local peer id: {peer_id}");

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
        )?;

        // Create a topic for env var sync
        let topic = gossipsub::IdentTopic::new("envctl-sync");
        gossipsub.subscribe(&topic)?;

        // Set up mDNS for peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Create behaviour
        let behaviour = P2PBehaviour { gossipsub, mdns };

        // Build the Swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self { swarm, topic })
    }

    pub async fn listen_on(&mut self, addr: &str) -> anyhow::Result<()> {
        self.swarm.listen_on(addr.parse()?)?;
        Ok(())
    }

    pub async fn publish(&mut self, message: Vec<u8>) -> anyhow::Result<()> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), message)?;
        Ok(())
    }
}
