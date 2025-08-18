use crate::crypto;
use crate::storage::queue::{MessageQueue, MessageStatus, QueuedMessage};
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use uuid::Uuid;

#[allow(dead_code)]
pub async fn send_now(
    queue_path: &str,
    sender_sk: &SecretKey,
    recipient_pk: &PublicKey,
    recipient_id: u64,
    plaintext: &[u8],
) -> Result<Uuid, crate::error::Error> {
    let ciphertext = crypto::encrypt_message(sender_sk, recipient_pk, plaintext)
        .map_err(crate::error::Error::Crypto)?;
    let id = Uuid::new_v4();
    let q = MessageQueue::new(queue_path).map_err(crate::error::Error::Storage)?;
    let msg = QueuedMessage {
        id,
        contact_id: recipient_id,
        payload: ciphertext,
        created: 0,
        priority: 1,
        status: MessageStatus::Transmitting,
        retry_count: 0,
        next_attempt_at: 0,
        max_retries: 5,
    };
    q.enqueue(msg).map_err(crate::error::Error::Storage)?;
    Ok(id)
}
