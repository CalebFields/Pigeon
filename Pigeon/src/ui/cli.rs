use crate::storage::queue::MessageQueue;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "secure-p2p-msg")]
#[command(about = "Secure P2P messaging client", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new contact
    Add {
        name: String,
        #[arg(value_parser = parse_socket_addr)]
        addr: SocketAddr,
        public_key: String,
        #[arg(long)]
        ping_interval: Option<u64>,
    },
    
    /// Compose and queue a message
    Compose {
        recipient_id: u64,
        message: String,
        #[arg(short, long)]
        queue: Option<String>,
    },
    
    /// Manage message queue
    Queue {
        #[command(subcommand)]
        action: QueueAction,
    },
    
    /// Fetch/decrypt inbox (local)
    Fetch {
        #[arg(short, long)]
        queue: Option<String>,
    },

    /// Send a message over libp2p to a peer (requires `network` feature)
    #[cfg(feature = "network")]
    SendNet {
        #[arg(long)]
        to: String, // multiaddr
        #[arg(long = "pubkey_hex")]
        pubkey_hex: String,
        #[arg(long)]
        message: String,
    },

    /// Listen for incoming messages and store to inbox (requires `network` feature)
    #[cfg(feature = "network")]
    ListenNet {
        #[arg(long)]
        port: Option<u16>,
    },
}

#[derive(Subcommand)]
enum QueueAction {
    /// List queued messages
    List,
    /// Cancel a queued message
    Cancel { id: String },
}

fn parse_socket_addr(s: &str) -> Result<SocketAddr, String> {
    s.parse().map_err(|e| format!("Invalid address: {}", e))
}

impl Cli {
    pub async fn execute(self) -> Result<(), crate::error::Error> {
        match self.command {
            Commands::Add {
                name,
                addr: _,
                public_key,
                ping_interval: _,
            } => {
                let _pub_key_bytes = hex::decode(&public_key)
                    .map_err(|e| crate::error::Error::Config(e.to_string()))?;
                
                // Create and store contact
                println!("Added contact: {}", name);
            }
            Commands::Compose { recipient_id, message, queue } => {
                let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                let id = crate::messaging::compose::compose_message(recipient_id, &message, &queue_path).await?;
                println!("Queued message {} for {}", id, recipient_id);
            }
            Commands::Queue { action } => match action {
                QueueAction::List => {
                    // List messages
                    println!("Listing queued messages");
                }
                QueueAction::Cancel { id } => {
                    // Cancel message
                    println!("Canceled message {}", id);
                }
            },
            Commands::Fetch { queue } => {
                let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                let q = MessageQueue::new(&queue_path).map_err(crate::error::Error::Storage)?;
                for (id, bytes) in q.list_inbox().map_err(crate::error::Error::Storage)? {
                    let preview = String::from_utf8_lossy(&bytes);
                    println!("{}: {}", id, preview);
                }
            }
            #[cfg(feature = "network")]
            Commands::SendNet { to, pubkey_hex, message } => {
                use libp2p::{identity, Multiaddr, swarm::SwarmEvent, Transport};

                let remote_pk_bytes = hex::decode(pubkey_hex)
                    .map_err(|e| crate::error::Error::Config(e.to_string()))?;
                let _remote_pk = sodiumoxide::crypto::box_::PublicKey::from_slice(&remote_pk_bytes)
                    .ok_or_else(|| crate::error::Error::Config("invalid pubkey".to_string()))?;

                let local_key = identity::Keypair::generate_ed25519();
                let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
                    .upgrade(libp2p::core::upgrade::Version::V1)
                    .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| crate::error::Error::Network(crate::network::Error::Handshake(e.to_string())))?)
                    .multiplex(libp2p::yamux::Config::default())
                    .boxed();
                let cfg = libp2p::request_response::Config::default();
                let protocols: Vec<(String, libp2p::request_response::ProtocolSupport)> = vec![("/pigeon/1".to_string(), libp2p::request_response::ProtocolSupport::Full)];
                let behaviour: libp2p::request_response::Behaviour<crate::network::rr::PigeonCodec> = libp2p::request_response::Behaviour::new(protocols, cfg);
                let peer_id = local_key.public().to_peer_id();
                use libp2p::swarm::SwarmBuilder;
                let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

                let addr: Multiaddr = to.parse().map_err(|e: libp2p::multiaddr::Error| crate::error::Error::Config(e.to_string()))?;
                libp2p::Swarm::dial(&mut swarm, addr).map_err(|e| crate::error::Error::Network(crate::network::Error::Connection(format!("dial: {}", e))))?;
                let mut _sent = false;
                let mut done = false;
                use libp2p::futures::StreamExt;
                while !done {
                    match swarm.select_next_some().await {
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            let data = message.clone().into_bytes();
                            let _ = swarm.behaviour_mut().send_request(&peer_id, data);
                        }
                        SwarmEvent::Behaviour(libp2p::request_response::Event::<Vec<u8>, Vec<u8>>::Message { message, .. }) => {
                            match message {
                                libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Response { response, .. } => {
                                    let txt = String::from_utf8_lossy(&response);
                                    println!("response: {}", txt);
                                    done = true;
                                }
                                libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Request { .. } => {}
                            }
                        }
                        _ => {}
                    }
                }
                println!("sent over libp2p");
            }
            #[cfg(feature = "network")]
            Commands::ListenNet { port } => {
                use libp2p::{identity, Multiaddr, Transport};
                use crate::network::rr;
                let local_key = identity::Keypair::generate_ed25519();
                let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
                    .upgrade(libp2p::core::upgrade::Version::V1)
                    .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| crate::error::Error::Network(crate::network::Error::Handshake(e.to_string())))?)
                    .multiplex(libp2p::yamux::Config::default())
                    .boxed();
                let cfg = libp2p::request_response::Config::default();
                let protocols: Vec<(String, libp2p::request_response::ProtocolSupport)> = vec![("/pigeon/1".to_string(), libp2p::request_response::ProtocolSupport::Full)];
                let behaviour: libp2p::request_response::Behaviour<crate::network::rr::PigeonCodec> = libp2p::request_response::Behaviour::new(protocols, cfg);
                let peer_id = local_key.public().to_peer_id();
                use libp2p::swarm::SwarmBuilder;
                let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

                let p = port.unwrap_or(0);
                let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p).parse().map_err(|e: libp2p::multiaddr::Error| crate::error::Error::Config(e.to_string()))?;
                libp2p::Swarm::listen_on(&mut swarm, addr).map_err(|e| crate::error::Error::Network(crate::network::Error::Connection(format!("listen: {}", e))))?;

                println!("Listening (libp2p rr)... [Ctrl+C to exit]");
                use libp2p::futures::StreamExt;
                loop {
                    match swarm.select_next_some().await {
                        libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {}", address),
                        libp2p::swarm::SwarmEvent::Behaviour(libp2p::request_response::Event::<Vec<u8>, Vec<u8>>::Message { message, peer, .. }) => {
                            match message {
                                libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Request { request, channel, .. } => {
                                    let txt = String::from_utf8_lossy(&request);
                                    println!("received: {}", txt);
                                    let _ = swarm.behaviour_mut().send_response(channel, b"ACK".to_vec());
                                }
                                libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Response { .. } => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            /*
            Commands::Receive => {
                #[cfg(feature = "network")]
                {
                    use libp2p::identity;
                    use crate::network::NetworkManager;
                    let local_key = identity::Keypair::generate_ed25519();
                    let mut nm = NetworkManager::new(local_key).map_err(|e| crate::error::Error::Network(e.to_string()))?;
                    println!("Listening for messages... [Ctrl+C to exit; ping enabled]");
                    tokio::select! {
                        _ = nm.start() => {}
                        _ = tokio::signal::ctrl_c() => {}
                    }
                }
                #[cfg(not(feature = "network"))]
                {
                    println!("Listening for messages... [Ctrl+C to exit] (network feature disabled)");
                    tokio::signal::ctrl_c().await?;
                }
            }
            */
        }
        Ok(())
    }
}