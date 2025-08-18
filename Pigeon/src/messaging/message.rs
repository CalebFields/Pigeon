use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvelopeV1 {
    pub version: u8,
    pub sender_id: u64,
    pub recipient_id: u64,
    pub nonce: [u8; 24],
    pub payload: Vec<u8>, // ciphertext
    pub signature: Vec<u8>, // ed25519 signature over (version|sender|recipient|nonce|payload)
}

impl EnvelopeV1 {
    #[allow(dead_code)]
    pub fn new(sender_id: u64, recipient_id: u64, nonce: [u8;24], payload: Vec<u8>, signature: Vec<u8>) -> Self {
        Self { version: 1, sender_id, recipient_id, nonce, payload, signature }
    }
}


