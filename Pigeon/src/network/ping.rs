use crate::{crypto, storage};
use libp2p::{
    identity,
    swarm::{SwarmBuilder, SwarmEvent},
    Multiaddr, PeerId, Swarm, Transport,
};
use crate::network::protocol::MessageProtocol;
use std::time::Duration;

pub struct NetworkManager {
    swarm: Swarm<protocol::MessageProtocol>,
    local_key: identity::Keypair,
}

impl NetworkManager {
    pub fn new(local_key: identity::Keypair) -> Result<Self, super::Error> {
        let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| super::Error::Handshake(e.to_string()))?)
            .multiplex(libp2p::yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let behaviour = MessageProtocol::new_with_ping(Some(Duration::from_secs(5)));
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
        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0"
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
                SwarmEvent::Behaviour(event) => {
                    match event {
                        crate::network::protocol::MessageProtocolEvent::Ping(pe) => {
                            match pe {
                                libp2p::ping::Event::Success { rtt, .. } => {
                                    log::info!("Ping RTT: {:?}", rtt);
                                }
                                libp2p::ping::Event::Failure { error, .. } => {
                                    log::warn!("Ping failure: {:?}", error);
                                }
                                other => {
                                    log::debug!("Ping event: {:?}", other);
                                }
                            }
                        }
                        other => {
                            log::debug!("Behaviour event: {:?}", other);
                        }
                    }
                }
                event => {
                    log::debug!("Unhandled SwarmEvent: {:?}", event);
                }
            }
        }
    }
}