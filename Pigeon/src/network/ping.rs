use libp2p::ping;
use libp2p::{identity, swarm::SwarmEvent, Multiaddr, PeerId, Swarm, Transport};
use std::time::Duration;

pub struct NetworkManager {
    swarm: Swarm<ping::Behaviour>,
    #[allow(dead_code)]
    local_key: identity::Keypair,
}

impl NetworkManager {
    pub fn new(local_key: identity::Keypair) -> Result<Self, super::Error> {
        let transport =
            libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(
                    libp2p::noise::Config::new(&local_key)
                        .map_err(|e| super::Error::Handshake(e.to_string()))?,
                )
                .multiplex(libp2p::yamux::Config::default())
                .timeout(Duration::from_secs(20))
                .boxed();

        let behaviour = ping::Behaviour::default();
        let peer_id = PeerId::from(local_key.public());

        let swarm = libp2p::Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        Ok(Self { swarm, local_key })
    }

    pub async fn start_with_port(&mut self, port: Option<u16>) {
        let port = port.unwrap_or(0);
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port)
            .parse()
            .expect("Valid multiaddr");

        self.swarm
            .listen_on(addr)
            .expect("Failed to listen on address");

        use libp2p::futures::StreamExt;
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
                SwarmEvent::Behaviour(ev) => match ev {
                    ping::Event { result: Ok(s), .. } => log::info!("Ping OK: {:?}", s),
                    ping::Event { result: Err(e), .. } => log::warn!("Ping failure: {:?}", e),
                },
                event => {
                    log::debug!("Unhandled SwarmEvent: {:?}", event);
                }
            }
        }
    }

    pub async fn start(&mut self) {
        self.start_with_port(None).await
    }
}
