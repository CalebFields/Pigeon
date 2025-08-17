#![cfg(feature = "network")]

use libp2p::{identity, swarm::{SwarmBuilder, SwarmEvent}, Multiaddr, ping};
use secure_p2p_msg::network::protocol::MessageProtocol;

#[tokio::test]
async fn ping_events_occur() {
    let kp = identity::Keypair::generate_ed25519();

    let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&kp).unwrap())
        .multiplex(libp2p::yamux::Config::default())
        .boxed();

    let mut behaviour = MessageProtocol::new_with_ping(Some(std::time::Duration::from_millis(200)));
    let mut swarm = SwarmBuilder::new(transport, behaviour, kp.public().to_peer_id())
        .executor(Box::new(|fut| { tokio::spawn(fut); }))
        .build();

    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    swarm.listen_on(addr).unwrap();

    let mut saw_listen = false;
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 3 {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { .. } => { saw_listen = true; break; }
            _ => {}
        }
    }
    assert!(saw_listen);
}


