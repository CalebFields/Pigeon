use serde::{Deserialize, Serialize};
use sled::IVec;
use uuid::Uuid;

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
}

#[allow(dead_code)]
impl MessageQueue {
    pub fn new(path: &str) -> Result<Self, super::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn enqueue(&self, message: &QueuedMessage) -> Result<(), super::Error> {
        let id_bytes = message.id.as_bytes();
        let message_bytes = bincode::serialize(message)
            .map_err(|e| super::Error::Serialization(e.to_string()))?;
        self.db.insert(id_bytes, message_bytes)?;
        Ok(())
    }

    pub fn update_status(&self, message_id: Uuid, status: MessageStatus) -> Result<(), super::Error> {
        let id_bytes = message_id.as_bytes();
        if let Some(mut message_bytes) = self.db.get(id_bytes)? {
            let mut message: QueuedMessage = bincode::deserialize(&message_bytes)
                .map_err(|e| super::Error::Serialization(e.to_string()))?;
            message.status = status;
            message_bytes = IVec::from(
                bincode::serialize(&message)
                    .map_err(|e| super::Error::Serialization(e.to_string()))?
            );
            self.db.insert(id_bytes, message_bytes)?;
        }
        Ok(())
    }

    pub fn get_pending_messages(&self) -> Result<Vec<QueuedMessage>, super::Error> {
        self.db
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
}