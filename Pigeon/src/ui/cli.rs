use crate::ops;
use crate::storage::queue::MessageQueue;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "secure-p2p-msg")]
#[command(about = "Secure P2P messaging client", long_about = None)]
pub struct Cli {
    /// Override data directory (highest precedence)
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    /// Override log level (trace, debug, info, warn, error)
    #[arg(long, global = true)]
    log_level: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show local identity information
    Identity,
    /// Contacts management
    Contacts {
        #[command(subcommand)]
        action: ContactsAction,
    },
    /// Security operations
    Security {
        #[command(subcommand)]
        action: SecurityAction,
    },
    /// Ops and observability
    Ops {
        #[command(subcommand)]
        action: OpsAction,
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

    /// Inbox browsing
    Inbox {
        #[command(subcommand)]
        action: InboxAction,
    },

    /// Background send loop (drains queue with backoff)
    #[cfg(feature = "network")]
    SendLoop {
        #[arg(short, long, default_value = "queue_db")]
        queue: String,
        /// Base backoff in seconds
        #[arg(long, default_value_t = 2)]
        base_backoff: u64,
        /// Poll interval in milliseconds
        #[arg(long, default_value_t = 500u64)]
        interval_ms: u64,
        /// Number of high-priority messages to send for each normal message
        #[arg(long, default_value_t = 3u8)]
        high_ratio: u8,
    },
    /// Send a message over libp2p to a peer (requires `network` feature)
    #[cfg(feature = "network")]
    SendNet {
        /// Multiaddr or contact name/id if --contact is used
        #[arg(long)]
        to: Option<String>,
        /// Sodium pubkey hex; ignored if --contact is used
        #[arg(long = "pubkey_hex")]
        pubkey_hex: Option<String>,
        /// Send to a saved contact by name or id
        #[arg(long)]
        contact: Option<String>,
        #[arg(long)]
        message: String,
        /// Maximum reconnect attempts
        #[arg(long, default_value_t = 5u32)]
        retries: u32,
        /// Initial backoff in milliseconds between reconnect attempts
        #[arg(long, default_value_t = 500u64)]
        backoff_ms: u64,
        /// Maximum backoff in milliseconds
        #[arg(long, default_value_t = 5000u64)]
        max_backoff_ms: u64,
        /// Overall attempt timeout in milliseconds (connect+send+ack)
        #[arg(long, default_value_t = 5000u64)]
        timeout_ms: u64,
    },

    /// Listen for incoming messages and store to inbox (requires `network` feature)
    #[cfg(feature = "network")]
    ListenNet {
        #[arg(long)]
        port: Option<u16>,
        /// Override listen multiaddr (e.g. "/ip4/0.0.0.0/tcp/4001"). If set, overrides --port and config/env.
        #[arg(long)]
        listen_addr: Option<String>,
        /// Enable mDNS LAN discovery
        #[arg(long)]
        mdns: bool,
    },
}

#[derive(Subcommand)]
enum QueueAction {
    /// List queued messages
    List,
    /// Cancel a queued message
    Cancel { id: String },
}

#[derive(Subcommand)]
enum ContactsAction {
    /// Add a new contact
    Add {
        name: String,
        /// Multiaddr string, e.g. "/ip4/127.0.0.1/tcp/4001"
        addr: String,
        /// Sodium box public key hex (64 hex chars)
        #[arg(long = "pubkey_hex")]
        pubkey_hex: String,
    },
    /// List contacts
    List,
    /// Show a contact by name or id
    Show { sel: String },
    /// Remove a contact by id
    Remove { id: u64 },
}

#[derive(Subcommand)]
enum SecurityAction {
    /// Derive and print a KDF key preview from a passphrase (dev aid)
    PreviewKey { passphrase: String },
    /// Set/rotate a passphrase and seal the at-rest key
    SetPassphrase { passphrase: String },
    /// Unlock an existing sealed at-rest key for this session
    Unlock { passphrase: String },
}

#[derive(Subcommand)]
enum OpsAction {
    /// Serve /metrics on an HTTP socket (Prometheus exposition)
    Serve {
        /// Host:port to bind, e.g. 127.0.0.1:9090
        #[arg(long, default_value = "127.0.0.1:9090")]
        addr: String,
    },
}

#[derive(Subcommand)]
enum InboxAction {
    /// List inbox messages (preview lines)
    List {
        #[arg(short, long)]
        queue: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show a full message by UUID
    Show {
        #[arg(short, long)]
        queue: Option<String>,
        id: String,
    },
    /// Export a message to a file
    Export {
        #[arg(short, long)]
        queue: Option<String>,
        id: String,
        #[arg(long, value_name = "PATH")]
        out: PathBuf,
    },
    /// Search inbox messages by substring (case-insensitive)
    Search {
        #[arg(short, long)]
        queue: Option<String>,
        term: String,
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[allow(dead_code)]
fn parse_socket_addr(s: &str) -> Result<SocketAddr, String> {
    s.parse().map_err(|e| format!("Invalid address: {}", e))
}

impl Cli {
    pub async fn execute(self) -> Result<(), crate::error::Error> {
        // Apply global overrides early so all commands see them in config loader
        if let Some(dir) = &self.data_dir {
            std::env::set_var("PIGEON_DATA_DIR", dir);
        }
        if let Some(level) = &self.log_level {
            std::env::set_var("PIGEON_LOG_LEVEL", level);
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", level);
            }
        }
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        // initialize logger after overrides
        let _ = pretty_env_logger::try_init();
        match self.command {
            Commands::Identity => {
                let cfg = crate::config::load();
                let id = crate::identity::Identity::load_or_generate(&cfg.data_dir)?;
                let path = cfg.data_dir.join("identity.bin");
                println!("identity file: {}", path.display());
                let sodium_pk_hex = hex::encode(id.sodium_box_pk.0);
                println!("sodium box pk: {}", sodium_pk_hex);
                println!("sign pk: {}", hex::encode(id.sign_pk.0));
                #[cfg(feature = "network")]
                {
                    let peer_id = libp2p::PeerId::from(id.libp2p.public());
                    println!("peer id: {}", peer_id);
                }
                #[cfg(not(feature = "network"))]
                {
                    println!("peer id: (network feature disabled)");
                }
            }
            Commands::Contacts { action } => {
                let cfg = crate::config::load();
                let store = crate::storage::contacts::ContactStore::open_in_dir(&cfg.data_dir)
                    .map_err(crate::error::Error::Storage)?;
                match action {
                    ContactsAction::Add {
                        name,
                        addr,
                        pubkey_hex,
                    } => {
                        let c = store
                            .add(&name, &addr, &pubkey_hex)
                            .map_err(crate::error::Error::Storage)?;
                        println!("added contact {} (id: {}) -> {}", c.name, c.id, c.addr);
                    }
                    ContactsAction::List => {
                        let list = store.list().map_err(crate::error::Error::Storage)?;
                        for c in list {
                            println!(
                                "{}\t{}\t{}\t{}",
                                c.id,
                                c.name,
                                c.addr,
                                hex::encode(c.public_key)
                            );
                        }
                    }
                    ContactsAction::Show { sel } => {
                        let found = if let Ok(id) = sel.parse::<u64>() {
                            store.get(id).map_err(crate::error::Error::Storage)?
                        } else {
                            let list = store.list().map_err(crate::error::Error::Storage)?;
                            list.into_iter().find(|c| c.name.eq_ignore_ascii_case(&sel))
                        };
                        match found {
                            Some(c) => {
                                println!(
                                    "id: {}\nname: {}\naddr: {}\npubkey: {}",
                                    c.id,
                                    c.name,
                                    c.addr,
                                    hex::encode(c.public_key)
                                );
                            }
                            None => println!("not found: {}", sel),
                        }
                    }
                    ContactsAction::Remove { id } => {
                        let existed = store.remove(id).map_err(crate::error::Error::Storage)?;
                        if existed {
                            println!("removed {}", id);
                        } else {
                            println!("not found: {}", id);
                        }
                    }
                }
            }
            Commands::Compose {
                recipient_id,
                message,
                queue,
            } => {
                let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                let id =
                    crate::messaging::compose::compose_message(recipient_id, &message, &queue_path)
                        .await?;
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
            Commands::Inbox { action } => match action {
                InboxAction::List { queue, limit } => {
                    let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                    let q = MessageQueue::new(&queue_path).map_err(crate::error::Error::Storage)?;
                    let mut items = q.list_inbox().map_err(crate::error::Error::Storage)?;
                    if let Some(n) = limit {
                        items.truncate(n);
                    }
                    for (id, bytes) in items {
                        let preview = String::from_utf8_lossy(&bytes);
                        println!("{}\t{}", id, preview);
                    }
                }
                InboxAction::Show { queue, id } => {
                    let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                    let q = MessageQueue::new(&queue_path).map_err(crate::error::Error::Storage)?;
                    let uid = uuid::Uuid::parse_str(&id)
                        .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
                    match q.get_inbox(uid).map_err(crate::error::Error::Storage)? {
                        Some(bytes) => match String::from_utf8(bytes) {
                            Ok(txt) => println!("{}", txt),
                            Err(_) => println!("<binary>"),
                        },
                        None => println!("not found: {}", id),
                    }
                }
                InboxAction::Export { queue, id, out } => {
                    let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                    let q = MessageQueue::new(&queue_path).map_err(crate::error::Error::Storage)?;
                    let uid = uuid::Uuid::parse_str(&id)
                        .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
                    match q.get_inbox(uid).map_err(crate::error::Error::Storage)? {
                        Some(bytes) => {
                            std::fs::write(&out, &bytes).map_err(|e| {
                                crate::error::Error::Storage(crate::storage::Error::Serialization(
                                    e.to_string(),
                                ))
                            })?;
                            println!("exported {} bytes to {}", bytes.len(), out.display());
                        }
                        None => println!("not found: {}", id),
                    }
                }
                InboxAction::Search { queue, term, limit } => {
                    let queue_path = queue.unwrap_or_else(|| "queue_db".to_string());
                    let q = MessageQueue::new(&queue_path).map_err(crate::error::Error::Storage)?;
                    let items = q.list_inbox().map_err(crate::error::Error::Storage)?;
                    let needle = term.to_lowercase();
                    let mut count = 0usize;
                    for (id, bytes) in items {
                        if let Ok(txt) = String::from_utf8(bytes) {
                            if txt.to_lowercase().contains(&needle) {
                                println!("{}\t{}", id, txt);
                                count += 1;
                                if let Some(max) = limit {
                                    if count >= max {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Commands::Security { action } => {
                match action {
                    SecurityAction::PreviewKey { passphrase } => {
                        let salt = b"pigeon-dev-salt"; // dev-only static salt preview
                        let key =
                            crate::storage::at_rest::derive_key_from_passphrase(&passphrase, salt)
                                .map_err(crate::error::Error::Storage)?;
                        println!("kdf key preview: {}", hex::encode(key.0));
                    }
                    SecurityAction::SetPassphrase { passphrase } => {
                        let cfg = crate::config::load();
                        crate::storage::at_rest::set_passphrase_and_seal(
                            &cfg.data_dir,
                            &passphrase,
                        )
                        .map_err(crate::error::Error::Storage)?;
                        println!("sealed at-rest key");
                    }
                    SecurityAction::Unlock { passphrase } => {
                        let cfg = crate::config::load();
                        let _ = crate::storage::at_rest::unlock_with_passphrase(
                            &cfg.data_dir,
                            &passphrase,
                        )
                        .map_err(crate::error::Error::Storage)?;
                        println!("unlocked for this session");
                    }
                }
            }
            Commands::Ops { action } => match action {
                OpsAction::Serve { addr } => {
                    let addr: std::net::SocketAddr =
                        addr.parse().map_err(|e: std::net::AddrParseError| {
                            crate::error::Error::Config(e.to_string())
                        })?;
                    let metrics = ops::Metrics::default();
                    println!("serving /metrics on http://{}", addr);
                    ops::serve(addr, metrics)
                        .await
                        .map_err(crate::error::Error::Io)?;
                }
            },
            #[cfg(feature = "network")]
            Commands::SendLoop {
                queue,
                base_backoff,
                interval_ms,
                high_ratio,
            } => {
                let cfg = crate::config::load();
                let conf = crate::messaging::send_loop::SendLoopConfig {
                    queue_path: queue,
                    data_dir: cfg.data_dir.clone(),
                    base_backoff_secs: base_backoff,
                    interval_ms,
                    high_to_normal_ratio: high_ratio,
                };
                println!("Starting send loop (queue: {}, backoff: {}s, interval: {}ms, high:normal={}:{})...",
                    conf.queue_path, conf.base_backoff_secs, conf.interval_ms, conf.high_to_normal_ratio, 1);
                crate::messaging::send_loop::run(conf).await?;
            }
            #[cfg(feature = "network")]
            Commands::SendNet {
                to,
                pubkey_hex,
                contact,
                message,
                retries,
                backoff_ms,
                max_backoff_ms,
                timeout_ms,
            } => {
                use libp2p::{swarm::SwarmEvent, Multiaddr, Transport};
                let cfg = crate::config::load();
                // Resolve target: from contact or explicit args via helper
                let (addr_str, remote_pk) = crate::ui::resolve_contact_or_args(
                    &cfg.data_dir,
                    contact.as_deref(),
                    to.as_deref(),
                    pubkey_hex.as_deref(),
                )?;

                let id = crate::identity::Identity::load_or_generate(&cfg.data_dir)?;
                let local_key = id.libp2p;
                let transport = libp2p::tcp::tokio::Transport::new(
                    libp2p::tcp::Config::default().nodelay(true),
                )
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| {
                    crate::error::Error::Network(crate::network::Error::Handshake(e.to_string()))
                })?)
                .multiplex(libp2p::yamux::Config::default())
                .boxed();
                let cfg = libp2p::request_response::Config::default();
                let protocols: Vec<(String, libp2p::request_response::ProtocolSupport)> = vec![(
                    "/pigeon/1".to_string(),
                    libp2p::request_response::ProtocolSupport::Full,
                )];
                let behaviour: libp2p::request_response::Behaviour<
                    crate::network::rr::PigeonCodec,
                > = libp2p::request_response::Behaviour::new(protocols, cfg);
                let peer_id = local_key.public().to_peer_id();
                let mut swarm = libp2p::Swarm::new(
                    transport,
                    behaviour,
                    peer_id,
                    libp2p::swarm::Config::with_tokio_executor(),
                );

                let addr: Multiaddr = addr_str.parse().map_err(|e: libp2p::multiaddr::Error| {
                    crate::error::Error::Config(e.to_string())
                })?;
                let mut attempt: u32 = 0;
                let mut delay = backoff_ms;
                let mut done = false;
                let mut need_dial = true;
                use libp2p::futures::StreamExt;
                while !done && attempt <= retries {
                    if need_dial {
                        if let Err(e) = libp2p::Swarm::dial(&mut swarm, addr.clone()) {
                            attempt += 1;
                            eprintln!("dial failed: {} (attempt {} of {})", e, attempt, retries);
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                            delay = (delay.saturating_mul(2)).min(max_backoff_ms);
                            continue;
                        }
                        need_dial = false;
                    }
                    let step_timeout =
                        tokio::time::sleep(std::time::Duration::from_millis(timeout_ms));
                    tokio::pin!(step_timeout);
                    match swarm.select_next_some().await {
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            // Build envelope v1 with sealedbox ciphertext
                            let plaintext = message.clone().into_bytes();
                            let nonce = sodiumoxide::crypto::box_::gen_nonce();
                            let ciphertext = sodiumoxide::crypto::box_::seal(
                                &plaintext,
                                &nonce,
                                &remote_pk,
                                &id.sodium_box_sk,
                            );
                            // sign over (version|sender_id=0|recipient_id=0|nonce|ciphertext)
                            let mut to_sign = Vec::new();
                            to_sign.push(1u8);
                            to_sign.extend_from_slice(&0u64.to_be_bytes());
                            to_sign.extend_from_slice(&0u64.to_be_bytes());
                            to_sign.extend_from_slice(nonce.as_ref());
                            to_sign.extend_from_slice(&ciphertext);
                            let sig =
                                sodiumoxide::crypto::sign::sign_detached(&to_sign, &id.sign_sk);
                            // Convert nonce to fixed array for envelope
                            let mut nonce_arr = [0u8; 24];
                            nonce_arr.copy_from_slice(nonce.as_ref());
                            let env = crate::messaging::message::EnvelopeV1::new(
                                0,
                                0,
                                nonce_arr,
                                ciphertext,
                                sig.to_bytes().to_vec(),
                            );
                            let data = bincode::serialize(&env)
                                .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
                            let _ = swarm.behaviour_mut().send_request(&peer_id, data);
                        }
                        SwarmEvent::Behaviour(libp2p::request_response::Event::<
                            Vec<u8>,
                            Vec<u8>,
                        >::Message {
                            message,
                            ..
                        }) => match message {
                            libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Response {
                                response,
                                ..
                            } => {
                                let txt = String::from_utf8_lossy(&response);
                                println!("response: {}", txt);
                                done = true;
                            }
                            libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Request {
                                ..
                            } => {}
                        },
                        SwarmEvent::ConnectionClosed { .. } => {
                            attempt += 1;
                            if attempt > retries {
                                break;
                            }
                            need_dial = true;
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                            delay = (delay.saturating_mul(2)).min(max_backoff_ms);
                        }
                        _ => {}
                    }
                    if step_timeout.is_elapsed() && !done {
                        attempt += 1;
                        if attempt > retries {
                            break;
                        }
                        need_dial = true;
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        delay = (delay.saturating_mul(2)).min(max_backoff_ms);
                    }
                }
                if done {
                    println!("sent over libp2p");
                } else {
                    eprintln!("send failed after {} attempts", attempt);
                }
            }
            #[cfg(feature = "network")]
            Commands::ListenNet {
                port,
                listen_addr,
                mdns,
            } => {
                use libp2p::{Multiaddr, Transport};
                // use crate::network::rr; // unused here
                let cfg = crate::config::load();
                let id = crate::identity::Identity::load_or_generate(&cfg.data_dir)?;
                let local_key = id.libp2p;
                let transport = libp2p::tcp::tokio::Transport::new(
                    libp2p::tcp::Config::default().nodelay(true),
                )
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| {
                    crate::error::Error::Network(crate::network::Error::Handshake(e.to_string()))
                })?)
                .multiplex(libp2p::yamux::Config::default())
                .boxed();
                // Build behaviour: request-response + mdns
                #[derive(libp2p::swarm::NetworkBehaviour)]
                struct NodeBehaviour {
                    request_response:
                        libp2p::request_response::Behaviour<crate::network::rr::PigeonCodec>,
                    mdns: libp2p::mdns::tokio::Behaviour,
                }
                let rr_cfg = libp2p::request_response::Config::default();
                let protocols: Vec<(String, libp2p::request_response::ProtocolSupport)> = vec![(
                    "/pigeon/1".to_string(),
                    libp2p::request_response::ProtocolSupport::Full,
                )];
                let rr_behaviour: libp2p::request_response::Behaviour<
                    crate::network::rr::PigeonCodec,
                > = libp2p::request_response::Behaviour::new(protocols, rr_cfg);
                let mdns_behaviour = libp2p::mdns::tokio::Behaviour::new(
                    libp2p::mdns::Config::default(),
                    local_key.public().to_peer_id(),
                )?;
                let behaviour = NodeBehaviour {
                    request_response: rr_behaviour,
                    mdns: mdns_behaviour,
                };
                let peer_id = local_key.public().to_peer_id();
                let mut swarm = libp2p::Swarm::new(
                    transport,
                    behaviour,
                    peer_id,
                    libp2p::swarm::Config::with_tokio_executor(),
                );

                // Determine listen address precedence: --listen-addr > config/env > --port > default
                let addr_str = if let Some(cli_addr) = listen_addr {
                    cli_addr
                } else if let Some(cfg_addr) = cfg.listen_addr.clone() {
                    cfg_addr
                } else if let Some(p) = port {
                    format!("/ip4/0.0.0.0/tcp/{}", p)
                } else {
                    "/ip4/0.0.0.0/tcp/0".to_string()
                };
                let addr: Multiaddr = addr_str.parse().map_err(|e: libp2p::multiaddr::Error| {
                    crate::error::Error::Config(e.to_string())
                })?;
                libp2p::Swarm::listen_on(&mut swarm, addr).map_err(|e| {
                    crate::error::Error::Network(crate::network::Error::Connection(format!(
                        "listen: {}",
                        e
                    )))
                })?;

                println!("Listening (libp2p rr)... [Ctrl+C to exit]");
                use libp2p::futures::StreamExt;
                loop {
                    match swarm.select_next_some().await {
                        libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {}", address)
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(ev) => {
                            match ev {
                                // Events from our derived behaviour enum
                                NodeBehaviourEvent::RequestResponse(
                                    libp2p::request_response::Event::<Vec<u8>, Vec<u8>>::Message {
                                        message,
                                        ..
                                    },
                                ) => {
                                    match message {
                                        libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Request { request, channel, .. } => {
                                            // Try decode as EnvelopeV1 and compute a single response
                                            let response = if let Ok(env) = bincode::deserialize::<crate::messaging::message::EnvelopeV1>(&request) {
                                                // verify signature
                                                let mut to_verify = Vec::new();
                                                to_verify.push(env.version);
                                                to_verify.extend_from_slice(&env.sender_id.to_be_bytes());
                                                to_verify.extend_from_slice(&env.recipient_id.to_be_bytes());
                                                to_verify.extend_from_slice(&env.nonce);
                                                to_verify.extend_from_slice(&env.payload);
                                                if env.signature.len() != 64 {
                                                    println!("received: <invalid signature>");
                                                    b"NACK".to_vec()
                                                } else {
                                                    let mut arr = [0u8; 64];
                                                    arr.copy_from_slice(&env.signature);
                                                    let sig = ed25519_dalek::Signature::from_bytes(&arr);
                                                    if let Ok(vk) = ed25519_dalek::VerifyingKey::from_bytes(&id.sign_pk.0) {
                                                        if vk.verify_strict(&to_verify, &sig).is_ok() {
                                                        // replay protection via nonce store
                                                        let db = sled::open(crate::config::load().data_dir.join("queue_db")).map_err(|e| crate::error::Error::Storage(crate::storage::Error::Db(e)))?;
                                                        let ns = crate::storage::nonce_store::NonceStore::open(&db).map_err(crate::error::Error::Storage)?;
                                                        if ns.insert_if_fresh(env.sender_id, &env.nonce).map_err(crate::error::Error::Storage)? {
                                                            // decrypt
                                                            let nonce = sodiumoxide::crypto::box_::Nonce::from_slice(&env.nonce).unwrap();
                                                            if let Ok(plaintext) = sodiumoxide::crypto::box_::open(&env.payload, &nonce, &id.sodium_box_pk, &id.sodium_box_sk) {
                                                                // Store to inbox (best-effort)
                                                                if let Ok(q) = crate::storage::queue::MessageQueue::new("queue_db") {
                                                                    let _ = q.store_inbox(uuid::Uuid::new_v4(), plaintext.clone());
                                                                }
                                                                // Increment received metric (process-local)
                                                                static METRICS: once_cell::sync::Lazy<crate::ops::Metrics> = once_cell::sync::Lazy::new(Default::default);
                                                                METRICS.received_messages.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                                let txt = String::from_utf8_lossy(&plaintext);
                                                                println!("received: {}", txt);
                                                            } else {
                                                                println!("received: <failed to decrypt>");
                                                            }
                                                            b"ACK".to_vec()
                                                        } else {
                                                            println!("replay detected (nonce)");
                                                            b"REPLAY".to_vec()
                                                        }
                                                        } else {
                                                            println!("received: <signature verify failed>");
                                                            b"NACK".to_vec()
                                                        }
                                                    } else {
                                                        println!("received: <invalid verify key>");
                                                        b"NACK".to_vec()
                                                    }
                                                }
                                            } else {
                                                // Fallback to plain text
                                                let txt = String::from_utf8_lossy(&request);
                                                println!("received: {}", txt);
                                                b"ACK".to_vec()
                                            };
                                            let _ = swarm.behaviour_mut().request_response.send_response(channel, response);
                                        }
                                        libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Response { .. } => {}
                                    }
                                }
                                NodeBehaviourEvent::Mdns(event) => {
                                    if mdns || cfg.enable_mdns {
                                        match event {
                                            libp2p::mdns::Event::Discovered(list) => {
                                                for (peer, addr) in list {
                                                    println!("mdns: discovered {peer} at {addr}");
                                                }
                                            }
                                            libp2p::mdns::Event::Expired(list) => {
                                                for (peer, addr) in list {
                                                    println!("mdns: expired {peer} at {addr}");
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            } /*
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
