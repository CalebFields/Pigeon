use serde::{Deserialize, Serialize};
use sled::{IVec, Tree};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct QueuedMessage {
    pub id: Uuid,
    pub contact_id: u64,
    pub payload: Vec<u8>,
    pub created: u64, // Unix timestamp
    pub priority: u8,
    pub status: MessageStatus,
    pub retry_count: u32,
    pub next_attempt_at: u64, // Unix timestamp when eligible for retry/dequeue
    pub max_retries: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageStatus {
    Pending,
    Transmitting,
    Canceled,
    Delivered(u64), // Delivery timestamp
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeadLetterRecord {
    pub id: Uuid,
    pub contact_id: u64,
    pub payload: Vec<u8>,
    pub failed_at: u64,
    pub attempts: u32,
    pub last_error: String,
}

#[allow(dead_code)]
pub struct MessageQueue {
    db: sled::Db,
    messages: Tree,
    by_created: Tree,
    inbox: Tree,
    dead_letter: Tree,
}

#[allow(dead_code)]
impl MessageQueue {
    pub fn new(path: &str) -> Result<Self, super::Error> {
        let db = sled::open(path)?;
        let messages = db.open_tree("messages")?;
        let by_created = db.open_tree("index_by_created")?;
        let inbox = db.open_tree("inbox")?;
        let dead_letter = db.open_tree("dead_letter")?;
        Ok(Self { db, messages, by_created, inbox, dead_letter })
    }

    pub fn enqueue(&self, mut message: QueuedMessage) -> Result<(), super::Error> {
        if message.created == 0 {
            message.created = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        if message.next_attempt_at == 0 {
            message.next_attempt_at = message.created;
        }
        let id_bytes = message.id.as_bytes();
        let message_bytes = bincode::serialize(&message)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        self.messages.insert(id_bytes, message_bytes)?;
        // Index by next_attempt_at to support exponential backoff scheduling
        let mut key = Vec::with_capacity(8 + id_bytes.len());
        key.extend_from_slice(&message.next_attempt_at.to_be_bytes());
        key.extend_from_slice(id_bytes);
        self.by_created.insert(key, id_bytes)?;
        Ok(())
    }

    pub fn dequeue(&self) -> Result<Option<QueuedMessage>, super::Error> {
        if let Some(Ok((k, v))) = self.by_created.iter().next() {
            // Only dequeue if the message is due (next_attempt_at <= now)
            if k.len() >= 8 {
                let mut ts_bytes = [0u8; 8];
                ts_bytes.copy_from_slice(&k[0..8]);
                let scheduled = u64::from_be_bytes(ts_bytes);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if scheduled > now {
                    return Ok(None);
                }
            }
            let id_bytes = v.as_ref();
            if let Some(bytes) = self.messages.get(id_bytes)? {
                let msg: QueuedMessage = bincode::deserialize(&bytes)
                    .map_err(|e| super::Error::Serialization(e.to_string()))?;
                self.messages.remove(id_bytes)?;
                self.by_created.remove(k)?;
                return Ok(Some(msg));
            }
        }
        Ok(None)
    }

    pub fn update_status(&self, message_id: Uuid, status: MessageStatus) -> Result<(), super::Error> {
        let id_bytes = message_id.as_bytes();
        if let Some(mut message_bytes) = self.messages.get(id_bytes)? {
            let mut message: QueuedMessage = bincode::deserialize(&message_bytes)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            message.status = status;
            message_bytes = IVec::from(
                bincode::serialize(&message)
                    .map_err(|e| super::Error::Serialization(e.to_string()))?
            );
            self.messages.insert(id_bytes, message_bytes)?;
        }
        Ok(())
    }

    pub fn get_pending_messages(&self) -> Result<Vec<QueuedMessage>, super::Error> {
        self.messages
            .iter()
            .filter_map(|item| item.ok())
            .filter(|(_, v)| {
                if let Ok(msg) = bincode::deserialize::<QueuedMessage>(v) {
                    matches!(msg.status, MessageStatus::Pending)
                } else {
                    false
                }
            })
            .map(|(_, v)| bincode::deserialize::<QueuedMessage>(&v).map_err(|e| super::Error::Serialization(e.to_string())))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.by_created.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn store_inbox(&self, message_id: Uuid, plaintext: Vec<u8>) -> Result<(), super::Error> {
        self.inbox.insert(message_id.as_bytes(), plaintext)?;
        Ok(())
    }

    pub fn get_inbox(&self, message_id: Uuid) -> Result<Option<Vec<u8>>, super::Error> {
        if let Some(v) = self.inbox.get(message_id.as_bytes())? {
            Ok(Some(v.to_vec()))
        } else {
            Ok(None)
        }
    }

    pub fn inbox_len(&self) -> usize {
        self.inbox.len()
    }

    pub fn list_inbox(&self) -> Result<Vec<(Uuid, Vec<u8>)>, super::Error> {
        self.inbox
            .iter()
            .filter_map(|item| item.ok())
            .map(|(k, v)| {
                let mut id_bytes = [0u8; 16];
                id_bytes.copy_from_slice(&k);
                let id = Uuid::from_bytes(id_bytes);
                Ok((id, v.to_vec()))
            })
            .collect()
    }

    pub fn requeue_with_backoff(&self, mut message: QueuedMessage, base_backoff_secs: u64) -> Result<(), super::Error> {
        // Exponential backoff: base * 2^(retry_count)
        message.retry_count = message.retry_count.saturating_add(1);
        let exp = message.retry_count.saturating_sub(1);
        // backoff = base * 2^exp, with capping exp to 20 to avoid overflow
        let pow2 = if exp > 20 { 1u64 << 20 } else { 1u64 << exp };
        let backoff = base_backoff_secs.saturating_mul(pow2);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        message.next_attempt_at = now.saturating_add(backoff);
        message.status = MessageStatus::Pending;
        self.enqueue(message)
    }

    pub fn dead_letter(&self, message: QueuedMessage, reason: &str) -> Result<(), super::Error> {
        let failed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = DeadLetterRecord {
            id: message.id,
            contact_id: message.contact_id,
            payload: message.payload,
            failed_at,
            attempts: message.retry_count,
            last_error: reason.to_string(),
        };
        let bytes = bincode::serialize(&record)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        self.dead_letter.insert(record.id.as_bytes(), bytes)?;
        Ok(())
    }

    pub fn requeue_or_dead_letter(&self, message: QueuedMessage, base_backoff_secs: u64, reason: &str) -> Result<bool, super::Error> {
        if message.retry_count >= message.max_retries {
            self.dead_letter(message, reason)?;
            Ok(false)
        } else {
            self.requeue_with_backoff(message, base_backoff_secs)?;
            Ok(true)
        }
    }

    pub fn dead_letter_len(&self) -> usize {
        self.dead_letter.len()
    }

    pub fn list_dead_letters(&self) -> Result<Vec<DeadLetterRecord>, super::Error> {
        self.dead_letter
            .iter()
            .filter_map(|item| item.ok())
            .map(|(_, v)| bincode::deserialize::<DeadLetterRecord>(&v).map_err(|e| super::Error::Serialization(e.to_string())))
            .collect()
    }
}