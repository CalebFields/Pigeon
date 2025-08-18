use std::path::{Path, PathBuf};
use std::net::SocketAddr;

use crate::config::{self, AppConfig};
use crate::storage::contacts::{Contact, ContactStore};
use crate::storage::queue::{DeadLetterRecord, MessageQueue, QueuedMessage};
use crate::ops;
use crate::settings::{self, AccessibilitySettings, AppState};
use uuid::Uuid;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// Thin facade exposing core operations for GUI or other frontends.
#[allow(dead_code)]
pub struct Core {
    cfg: AppConfig,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Core {
    /// Create a Core instance loading config from file/env.
    pub fn new() -> Self {
        Self { cfg: config::load() }
    }

    /// Create a Core with an explicit data directory (useful for tests/GUI).
    pub fn with_data_dir<P: AsRef<Path>>(data_dir: P) -> Self {
        let mut cfg = config::load();
        cfg.data_dir = data_dir.as_ref().to_path_buf();
        Self { cfg }
    }

    fn queue_path(&self) -> PathBuf {
        // Keep queue under data_dir by default
        self.cfg.data_dir.join("queue_db")
    }

    // Contacts
    pub fn contacts_add(
        &self,
        name: &str,
        addr: &str,
        public_key_hex: &str,
    ) -> Result<Contact, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store
            .add(name, addr, public_key_hex)
            .map_err(crate::error::Error::Storage)
    }

    pub fn contacts_list(&self) -> Result<Vec<Contact>, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store.list().map_err(crate::error::Error::Storage)
    }

    pub fn contacts_get(&self, id: u64) -> Result<Option<Contact>, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store.get(id).map_err(crate::error::Error::Storage)
    }

    pub fn contacts_remove(&self, id: u64) -> Result<bool, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store.remove(id).map_err(crate::error::Error::Storage)
    }

    pub fn contacts_update(
        &self,
        id: u64,
        name: &str,
        addr: &str,
        public_key_hex: &str,
    ) -> Result<Contact, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store
            .update(id, name, addr, public_key_hex)
            .map_err(crate::error::Error::Storage)
    }

    pub fn contacts_find_by_name(&self, name: &str) -> Result<Option<Contact>, crate::error::Error> {
        let store = ContactStore::open_in_dir(&self.cfg.data_dir)
            .map_err(crate::error::Error::Storage)?;
        store
            .find_by_name_case_insensitive(name)
            .map_err(crate::error::Error::Storage)
    }

    // Messaging
    pub async fn compose(&self, recipient_id: u64, body: &str) -> Result<Uuid, crate::error::Error> {
        crate::messaging::compose::compose_message(
            recipient_id,
            body,
            self.queue_path().to_str().unwrap_or("queue_db"),
        )
        .await
    }

    /// Encrypt immediately and enqueue for sending using local identity as sender.
    pub async fn send_encrypt_and_enqueue(
        &self,
        recipient_pubkey_hex: &str,
        recipient_id: u64,
        body: &str,
        high_priority: bool,
    ) -> Result<Uuid, crate::error::Error> {
        sodiumoxide::init().map_err(|_| crate::error::Error::Crypto(crate::crypto::Error::Encryption("sodium init failed".into())))?;
        let mut pk_bytes = vec![];
        pk_bytes.extend_from_slice(&hex::decode(recipient_pubkey_hex).map_err(|e| {
            crate::error::Error::Storage(crate::storage::Error::Validation(format!(
                "invalid recipient pk hex: {e}"
            )))
        })?);
        if pk_bytes.len() != 32 {
            return Err(crate::error::Error::Storage(crate::storage::Error::Validation(
                "recipient pk must be 32 bytes".into(),
            )));
        }
        let recipient_pk = sodiumoxide::crypto::box_::PublicKey::from_slice(&pk_bytes)
            .ok_or_else(|| crate::error::Error::Storage(crate::storage::Error::Validation("bad pk".into())))?;
        let id = {
            let ident = crate::identity::Identity::load_or_generate(&self.cfg.data_dir)?;
            crate::messaging::send::send_now(
                self.queue_path().to_str().unwrap_or("queue_db"),
                &ident.sodium_box_sk,
                &recipient_pk,
                recipient_id,
                body.as_bytes(),
            )
            .await?
        };
        if high_priority {
            // Move message to high lane by re-enqueueing with priority=0
            let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
                .map_err(crate::error::Error::Storage)?;
            if let Some(mut msg) = q
                .dequeue()
                .map_err(crate::error::Error::Storage)?
                .filter(|m| m.id == id)
            {
                msg.priority = 0;
                q.enqueue(msg).map_err(crate::error::Error::Storage)?;
            }
        }
        Ok(id)
    }

    // Inbox helpers
    pub fn inbox_list(&self) -> Result<Vec<(Uuid, Vec<u8>)>, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        q.list_inbox().map_err(crate::error::Error::Storage)
    }

    pub fn inbox_list_limited(
        &self,
        limit: usize,
    ) -> Result<Vec<(Uuid, Vec<u8>)>, crate::error::Error> {
        let mut items = self.inbox_list()?;
        if items.len() > limit {
            items.truncate(limit);
        }
        Ok(items)
    }

    pub fn inbox_show(&self, id: Uuid) -> Result<Option<Vec<u8>>, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        q.get_inbox(id).map_err(crate::error::Error::Storage)
    }

    pub fn inbox_search(
        &self,
        term: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(Uuid, Vec<u8>)>, crate::error::Error> {
        let lower = term.to_ascii_lowercase();
        let mut out = Vec::new();
        for (id, bytes) in self.inbox_list()? {
            let s = String::from_utf8_lossy(&bytes);
            if s.to_ascii_lowercase().contains(&lower) {
                out.push((id, bytes));
                if let Some(max) = limit {
                    if out.len() >= max {
                        break;
                    }
                }
            }
        }
        Ok(out)
    }

    pub fn inbox_export(&self, id: Uuid, out_path: &Path) -> Result<(), crate::error::Error> {
        let Some(bytes) = self.inbox_show(id)? else {
            return Err(crate::error::Error::Storage(crate::storage::Error::Serialization(
                "message not found".to_string(),
            )));
        };
        std::fs::write(out_path, bytes)
            .map_err(|e| crate::error::Error::Storage(crate::storage::Error::Serialization(e.to_string())))
    }

    // Queue views for GUI
    pub fn queue_list_pending(&self) -> Result<Vec<QueuedMessage>, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        q.get_pending_messages().map_err(crate::error::Error::Storage)
    }

    pub fn queue_list_dead_letters(&self) -> Result<Vec<DeadLetterRecord>, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        q.list_dead_letters().map_err(crate::error::Error::Storage)
    }

    pub fn queue_stats(&self) -> Result<QueueStats, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        Ok(QueueStats {
            pending: q.len() as u64,
            inbox: q.inbox_len() as u64,
            dead_letters: q.dead_letter_len() as u64,
        })
    }

    /// UI-friendly summaries of pending queue items.
    pub fn queue_list_pending_summaries(&self) -> Result<Vec<QueueItemSummary>, crate::error::Error> {
        let q = MessageQueue::new(self.queue_path().to_str().unwrap_or("queue_db"))
            .map_err(crate::error::Error::Storage)?;
        let items = q
            .get_pending_messages()
            .map_err(crate::error::Error::Storage)?;
        Ok(items.into_iter().map(QueueItemSummary::from).collect())
    }

    /// Start a lightweight inbox watcher that emits snapshots on change.
    pub fn watch_inbox(&self, interval_ms: u64) -> InboxWatcher {
        let queue_path = self.queue_path();
        let (tx, rx) = mpsc::channel(8);
        let handle: JoinHandle<()> = tokio::spawn(async move {
            let mut last_len: usize = 0;
            loop {
                let q = match MessageQueue::new(queue_path.to_str().unwrap_or("queue_db")) {
                    Ok(v) => v,
                    Err(_) => {
                        sleep(Duration::from_millis(interval_ms)).await;
                        continue;
                    }
                };
                let len = q.inbox_len();
                if len != last_len {
                    let snapshot = match q.list_inbox() {
                        Ok(mut items) => {
                            let latest = items.pop();
                            InboxSnapshot { len, latest }
                        }
                        Err(_) => InboxSnapshot { len, latest: None },
                    };
                    let _ = tx.send(snapshot).await;
                    last_len = len;
                }
                sleep(Duration::from_millis(interval_ms)).await;
            }
        });
        InboxWatcher { rx, handle }
    }

    /// Attempt to decrypt one queued message (simulated receive path) and store to inbox.
    pub async fn try_receive_once(&self, sender_pubkey_hex: &str) -> Result<(), crate::error::Error> {
        sodiumoxide::init().map_err(|_| crate::error::Error::Crypto(crate::crypto::Error::Encryption("sodium init failed".into())))?;
        let sender_pk_bytes = hex::decode(sender_pubkey_hex)
            .map_err(|e| crate::error::Error::Storage(crate::storage::Error::Validation(e.to_string())))?;
        let sender_pk = sodiumoxide::crypto::box_::PublicKey::from_slice(&sender_pk_bytes)
            .ok_or_else(|| crate::error::Error::Storage(crate::storage::Error::Validation("bad pk".into())))?;
        let ident = crate::identity::Identity::load_or_generate(&self.cfg.data_dir)?;
        crate::messaging::receive::receive_and_ack(
            self.queue_path().to_str().unwrap_or("queue_db"),
            &sender_pk,
            &ident.sodium_box_sk,
        )
        .await
    }

    /// Start the ops metrics HTTP server on the given address.
    pub fn start_ops_server(&self, addr: SocketAddr) -> JoinHandle<std::io::Result<()>> {
        let metrics = ops::Metrics::default();
        tokio::spawn(async move { ops::serve(addr, metrics).await })
    }

    /// Check latest available version from a URL returning a plain semver string.
    pub async fn check_for_update(&self, url: &str) -> Result<Option<String>, crate::error::Error> {
        let resp = reqwest::get(url)
            .await
            .map_err(|e| crate::error::Error::Io(std::io::Error::other(e)))?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let text = resp
            .text()
            .await
            .map_err(|e| crate::error::Error::Io(std::io::Error::other(e)))?;
        let s = text.trim();
        if s.is_empty() { return Ok(None); }
        Ok(Some(s.to_string()))
    }

    // Onboarding & Identity
    /// Returns true if identity file is missing, indicating first run.
    pub fn first_run_required(&self) -> bool {
        let path = crate::identity::Identity::file_path(&self.cfg.data_dir);
        !path.exists()
    }

    /// Ensure identity exists (generate if missing) and return a public preview.
    pub fn ensure_identity_and_preview(&self) -> Result<IdentityPreview, crate::error::Error> {
        let id = crate::identity::Identity::load_or_generate(&self.cfg.data_dir)?;
        Ok(IdentityPreview::from_identity(&id))
    }

    /// Import a previously exported identity file.
    pub fn import_identity_from_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), crate::error::Error> {
        let dest = crate::identity::Identity::file_path(&self.cfg.data_dir);
        std::fs::create_dir_all(self.cfg.data_dir.as_path())?;
        std::fs::copy(path, &dest)?;
        // Validate loadable
        let _ = crate::identity::Identity::load_or_generate(&self.cfg.data_dir)?;
        Ok(())
    }

    /// Set a passphrase to protect the at-rest key file.
    pub fn set_passphrase(&self, passphrase: &str) -> Result<(), crate::error::Error> {
        crate::storage::at_rest::set_passphrase_and_seal(&self.cfg.data_dir, passphrase)
            .map_err(crate::error::Error::Storage)
    }

    /// Unlock the at-rest key with the passphrase.
    pub fn unlock(&self, passphrase: &str) -> Result<(), crate::error::Error> {
        let _ = crate::storage::at_rest::unlock_with_passphrase(&self.cfg.data_dir, passphrase)
            .map_err(crate::error::Error::Storage)?;
        Ok(())
    }

    /// Rotate at-rest key and seal with passphrase
    pub fn rotate_at_rest_key(&self, passphrase: &str) -> Result<(), crate::error::Error> {
        crate::storage::at_rest::rotate_key_and_seal(&self.cfg.data_dir, passphrase)
            .map_err(crate::error::Error::Storage)
    }

    // Accessibility settings
    pub fn get_accessibility(&self) -> Result<AccessibilitySettings, crate::error::Error> {
        settings::load_accessibility_settings(&self.cfg.data_dir).map_err(crate::error::Error::Io)
    }

    pub fn set_accessibility(
        &self,
        s: AccessibilitySettings,
    ) -> Result<(), crate::error::Error> {
        settings::save_accessibility_settings(&self.cfg.data_dir, &s)
            .map_err(crate::error::Error::Io)
    }

    // App state (onboarding flag)
    pub fn get_app_state(&self) -> Result<AppState, crate::error::Error> {
        settings::load_app_state(&self.cfg.data_dir).map_err(crate::error::Error::Io)
    }

    pub fn set_app_state(&self, s: AppState) -> Result<(), crate::error::Error> {
        settings::save_app_state(&self.cfg.data_dir, &s).map_err(crate::error::Error::Io)
    }

    // Settings - general
    pub fn get_log_level(&self) -> String {
        self.cfg.log_level.clone()
    }

    pub fn set_log_level(&mut self, level: &str) {
        self.cfg.log_level = level.to_string();
    }

    // Settings - network (feature-gated)
    #[cfg(feature = "network")]
    pub fn get_network_settings(&self) -> NetworkSettings {
        NetworkSettings {
            listen_addr: self.cfg.listen_addr.clone(),
            enable_mdns: self.cfg.enable_mdns,
        }
    }

    #[cfg(feature = "network")]
    pub fn set_network_settings(
        &mut self,
        settings: NetworkSettings,
    ) -> Result<(), crate::error::Error> {
        if let Some(addr) = &settings.listen_addr {
            use std::str::FromStr;
            // Validate multiaddr format
            libp2p::Multiaddr::from_str(addr).map_err(|e| {
                crate::error::Error::Config(format!("invalid listen_addr: {e}"))
            })?;
        }
        self.cfg.listen_addr = settings.listen_addr;
        self.cfg.enable_mdns = settings.enable_mdns;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueStats {
    pub pending: u64,
    pub inbox: u64,
    pub dead_letters: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityPreview {
    pub sodium_box_pk_hex: String,
    pub sign_pk_hex: String,
    #[cfg(feature = "network")]
    pub libp2p_peer_id: String,
}

impl IdentityPreview {
    fn from_identity(id: &crate::identity::Identity) -> Self {
        #[cfg(feature = "network")]
        let peer_id = libp2p::PeerId::from(id.libp2p.public()).to_string();
        Self {
            sodium_box_pk_hex: hex::encode(id.sodium_box_pk.0.as_slice()),
            sign_pk_hex: hex::encode(id.sign_pk.0.as_slice()),
            #[cfg(feature = "network")]
            libp2p_peer_id: peer_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueItemSummary {
	pub id: Uuid,
	pub contact_id: u64,
	pub priority: u8,
	pub retry_count: u32,
	pub next_attempt_at: u64,
}

impl From<QueuedMessage> for QueueItemSummary {
	fn from(m: QueuedMessage) -> Self {
		Self {
			id: m.id,
			contact_id: m.contact_id,
			priority: m.priority,
			retry_count: m.retry_count,
			next_attempt_at: m.next_attempt_at,
		}
	}
}

pub struct InboxWatcher {
	rx: mpsc::Receiver<InboxSnapshot>,
	handle: JoinHandle<()>,
}

impl InboxWatcher {
	pub async fn recv(&mut self) -> Option<InboxSnapshot> {
		self.rx.recv().await
	}

	pub fn try_recv(&mut self) -> Option<InboxSnapshot> {
		self.rx.try_recv().ok()
	}
}

impl Drop for InboxWatcher {
	fn drop(&mut self) {
		self.handle.abort();
	}
}

#[derive(Debug, Clone)]
pub struct InboxSnapshot {
	pub len: usize,
	pub latest: Option<(Uuid, Vec<u8>)>,
}

#[cfg(feature = "network")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSettings {
	pub listen_addr: Option<String>,
	pub enable_mdns: bool,
}


