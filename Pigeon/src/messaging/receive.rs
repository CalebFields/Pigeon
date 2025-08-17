use crate::storage::queue::{MessageQueue, MessageStatus};
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use crate::crypto;

#[allow(dead_code)]
pub async fn receive_and_ack(
    queue_path: &str,
    sender_pk: &PublicKey,
    receiver_sk: &SecretKey,
) -> Result<(), crate::error::Error> {
    let q = MessageQueue::new(queue_path).map_err(crate::error::Error::Storage)?;
    if let Some(msg) = q.dequeue().map_err(crate::error::Error::Storage)? {
        let plaintext = crypto::decrypt_message(&msg.payload, sender_pk, receiver_sk)
            .map_err(crate::error::Error::Crypto)?;
        q.store_inbox(msg.id, plaintext).map_err(crate::error::Error::Storage)?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        q.enqueue(crate::storage::queue::QueuedMessage { status: MessageStatus::Delivered(ts), ..msg })
            .map_err(crate::error::Error::Storage)?;
    }
    Ok(())
}


