use crate::storage::queue::{MessageQueue, MessageStatus, QueuedMessage};
use crate::storage::contacts::ContactStore;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SendLoopConfig {
    pub queue_path: String,
    pub data_dir: std::path::PathBuf,
    pub base_backoff_secs: u64,
    pub interval_ms: u64,
}

/// Drain the queue periodically. For each due message, try to send over the network
/// if contact info is available; otherwise requeue with backoff or dead-letter.
pub async fn run(config: SendLoopConfig) -> Result<(), crate::error::Error> {
    loop {
        let q = MessageQueue::new(&config.queue_path).map_err(crate::error::Error::Storage)?;
        while let Some(msg) = q.dequeue().map_err(crate::error::Error::Storage)? {
            if !try_send_one(&config, &q, msg).await? {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(config.interval_ms)).await;
    }
}

async fn try_send_one(config: &SendLoopConfig, q: &MessageQueue, msg: QueuedMessage) -> Result<bool, crate::error::Error> {
    // Lookup contact
    let store = ContactStore::open_in_dir(&config.data_dir).map_err(crate::error::Error::Storage)?;
    let contact_opt = store.get(msg.contact_id).map_err(crate::error::Error::Storage)?;
    let Some(contact) = contact_opt else {
        let _ = q.requeue_or_dead_letter(msg, config.base_backoff_secs, "missing contact")?;
        return Ok(true);
    };

    // Build a minimal one-shot rr client and send bytes
    use libp2p::{Multiaddr, Transport};
    let cfg = crate::config::load();
    let id = crate::identity::Identity::load_or_generate(&cfg.data_dir)?;
    let local_key = id.libp2p;
    let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&local_key).map_err(|e| crate::error::Error::Network(crate::network::Error::Handshake(e.to_string())))?)
        .multiplex(libp2p::yamux::Config::default())
        .boxed();
    let rr_cfg = libp2p::request_response::Config::default();
    let protocols: Vec<(String, libp2p::request_response::ProtocolSupport)> = vec![("/pigeon/1".to_string(), libp2p::request_response::ProtocolSupport::Full)];
    let behaviour: libp2p::request_response::Behaviour<crate::network::rr::PigeonCodec> = libp2p::request_response::Behaviour::new(protocols, rr_cfg);
    let peer_id = local_key.public().to_peer_id();
    let mut swarm = libp2p::Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());

    let addr: Multiaddr = contact.addr.parse().map_err(|e: libp2p::multiaddr::Error| crate::error::Error::Config(e.to_string()))?;
    libp2p::Swarm::dial(&mut swarm, addr).map_err(|e| crate::error::Error::Network(crate::network::Error::Connection(format!("dial: {}", e))))?;

    use libp2p::futures::StreamExt;
    let mut done = false;
    let mut delivered = false;
    while !done {
        match swarm.select_next_some().await {
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                // Send the already-encrypted payload as request
                let _ = swarm.behaviour_mut().send_request(&peer_id, msg.payload.clone());
            }
            libp2p::swarm::SwarmEvent::Behaviour(libp2p::request_response::Event::<Vec<u8>, Vec<u8>>::Message { message, .. }) => {
                match message {
                    libp2p::request_response::Message::<Vec<u8>, Vec<u8>>::Response { response, .. } => {
                        let _ = response; // ACK ignored
                        delivered = true;
                        done = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if delivered {
        // mark delivered by storing minimal wrapper in inbox for consistency
        q.update_status(msg.id, MessageStatus::Delivered(now_secs()))?;
        Ok(true)
    } else {
        let _requeued = q.requeue_or_dead_letter(msg, config.base_backoff_secs, "send failed")?;
        Ok(true)
    }
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}


