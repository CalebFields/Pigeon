#![cfg(feature = "network")]

use libp2p::futures::StreamExt;
use libp2p::{identity, swarm::SwarmEvent, Multiaddr, Transport as _};

#[tokio::test]
async fn two_swarms_connect_locally() {
    let kp1 = identity::Keypair::generate_ed25519();
    let kp2 = identity::Keypair::generate_ed25519();

    let transport1 =
        libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&kp1).unwrap())
            .multiplex(libp2p::yamux::Config::default())
            .boxed();
    let behaviour1 = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());
    let mut swarm1 = libp2p::Swarm::new(
        transport1,
        behaviour1,
        kp1.public().to_peer_id(),
        libp2p::swarm::Config::with_tokio_executor(),
    );

    let transport2 =
        libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&kp2).unwrap())
            .multiplex(libp2p::yamux::Config::default())
            .boxed();
    let behaviour2 = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());
    let mut swarm2 = libp2p::Swarm::new(
        transport2,
        behaviour2,
        kp2.public().to_peer_id(),
        libp2p::swarm::Config::with_tokio_executor(),
    );

    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let _ = swarm1.listen_on(addr).unwrap();

    // Wait for listen address
    let listen_addr = loop {
        if let Some(SwarmEvent::NewListenAddr { address, .. }) = swarm1.next().await {
            break address;
        }
    };

    swarm2.dial(listen_addr.clone()).unwrap();

    // Drive both swarms until we see a connection
    let mut connected = false;
    let start = std::time::Instant::now();
    while !connected && start.elapsed().as_secs() < 10 {
        tokio::select! {
            event = swarm1.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = event { connected = true; }
            }
            event = swarm2.select_next_some() => {
                if let SwarmEvent::ConnectionEstablished { .. } = event { connected = true; }
            }
        }
    }
    assert!(connected, "peers failed to connect within timeout");
}
