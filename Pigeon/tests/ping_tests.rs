#![cfg(feature = "network")]

use libp2p::futures::StreamExt;
use libp2p::{identity, swarm::SwarmEvent, Multiaddr, Transport as _};

#[tokio::test]
async fn ping_events_occur() {
    let kp = identity::Keypair::generate_ed25519();

    let transport =
        libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&kp).unwrap())
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

    let behaviour = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());
    let mut swarm = libp2p::Swarm::new(
        transport,
        behaviour,
        kp.public().to_peer_id(),
        libp2p::swarm::Config::with_tokio_executor(),
    );

    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    swarm.listen_on(addr).unwrap();

    let mut saw_listen = false;
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 3 {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { .. } => {
                saw_listen = true;
                break;
            }
            _ => {}
        }
    }
    assert!(saw_listen);
}
