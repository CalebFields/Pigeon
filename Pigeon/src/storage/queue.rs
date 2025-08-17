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
}

#[allow(dead_code)]
impl MessageQueue {
    pub fn new(path: &str) -> Result<Self, super::Error> {
        let db = sled::open(path)?;
        let messages = db.open_tree("messages")?;
        let by_created = db.open_tree("index_by_created")?;
        Ok(Self { db, messages, by_created })
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
}