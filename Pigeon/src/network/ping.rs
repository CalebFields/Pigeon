use crate::{crypto, storage};
use libp2p::{
    identity, noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr, PeerId, Swarm,
};
use std::time::Duration;

pub struct NetworkManager {
    swarm: Swarm<protocol::MessageProtocol>,
    local_key: identity::Keypair,
}

impl NetworkManager {
    pub fn new(local_key: identity::Keypair) -> Result<Self, super::Error> {
        let transport = TokioTcpConfig::new()
            .nodelay(true)
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(
                noise::NoiseAuthenticated::xx(&local_key)
                    .expect("Signing libp2p-noise static DH keypair failed"),
            )
            .multiplex(libp2p::yamux::YamuxConfig::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let behaviour = protocol::MessageProtocol::new();
        let peer_id = PeerId::from(local_key.public());
        
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        Ok(Self {
            swarm,
            local_key,
        })
    }

    pub async fn start(&mut self) {
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", crate::config::DATA_PORT)
            .parse()
            .expect("Valid multiaddr");
        
        self.swarm
            .listen_on(addr)
            .expect("Failed to listen on address");

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Listening for data on {}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    log::info!("Connected to peer: {}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    log::info!("Disconnected from peer: {}", peer_id);
                }
                event => {
                    log::debug!("Unhandled SwarmEvent: {:?}", event);
                }
            }
        }
    }
}