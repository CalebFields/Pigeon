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
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageStatus {
    Pending,
    Transmitting,
    Canceled,
    Delivered(u64), // Delivery timestamp
}

#[allow(dead_code)]
pub struct MessageQueue {
    db: sled::Db,
    messages: Tree,
    by_created: Tree,
    inbox: Tree,
}

#[allow(dead_code)]
impl MessageQueue {
    pub fn new(path: &str) -> Result<Self, super::Error> {
        let db = sled::open(path)?;
        let messages = db.open_tree("messages")?;
        let by_created = db.open_tree("index_by_created")?;
        let inbox = db.open_tree("inbox")?;
        Ok(Self { db, messages, by_created, inbox })
    }

    pub fn enqueue(&self, mut message: QueuedMessage) -> Result<(), super::Error> {
        if message.created == 0 {
            message.created = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        let id_bytes = message.id.as_bytes();
        let message_bytes = bincode::serialize(&message)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        self.messages.insert(id_bytes, message_bytes)?;
        let mut key = Vec::with_capacity(8 + id_bytes.len());
        key.extend_from_slice(&message.created.to_be_bytes());
        key.extend_from_slice(id_bytes);
        self.by_created.insert(key, id_bytes)?;
        Ok(())
    }

    pub fn dequeue(&self) -> Result<Option<QueuedMessage>, super::Error> {
        if let Some(Ok((k, v))) = self.by_created.iter().next() {
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
}