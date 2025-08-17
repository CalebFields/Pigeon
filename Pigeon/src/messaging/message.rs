use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvelopeV1 {
    pub version: u8,
    pub sender_id: u64,
    pub recipient_id: u64,
    pub payload: Vec<u8>, // nonce || ciphertext
}

impl EnvelopeV1 {
    pub fn new(sender_id: u64, recipient_id: u64, payload: Vec<u8>) -> Self {
        Self { version: 1, sender_id, recipient_id, payload }
    }
}


