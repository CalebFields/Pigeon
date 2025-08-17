use crate::storage::queue::{MessageQueue, MessageStatus, QueuedMessage};
use uuid::Uuid;

#[allow(dead_code)]
pub async fn compose_message(recipient_id: u64, body: &str, queue_path: &str) -> Result<Uuid, crate::error::Error> {
    let plaintext = body.as_bytes().to_vec();
    // For M0-060 we do not have contacts wired; store plaintext as payload placeholder
    let id = Uuid::new_v4();
    let msg = QueuedMessage {
        id,
        contact_id: recipient_id,
        payload: plaintext,
        created: 0,
        priority: 1,
        status: MessageStatus::Pending,
    };
    let q = MessageQueue::new(queue_path).map_err(|e| crate::error::Error::Storage(e))?;
    q.enqueue(msg).map_err(|e| crate::error::Error::Storage(e))?;
    Ok(id)
}


