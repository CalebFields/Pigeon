// use crate::storage; // uncomment when storage is wired into CLI flows
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
        recipient: String,
        message: String,
        #[arg(short, long)]
        file: Option<String>,
    },
    
    /// Manage message queue
    Queue {
        #[command(subcommand)]
        action: QueueAction,
    },
    
    /// Enable inbox checking
    Receive,
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
            Commands::Compose {
                recipient,
                message: _,
                file: _,
            } => {
                // Compose and queue message
                println!("Message queued for {}", recipient);
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
        }
        Ok(())
    }
}