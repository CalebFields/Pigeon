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
        // Do not re-enqueue delivered messages; queue should drain on successful receive
    }
    Ok(())
}


