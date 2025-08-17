use secure_p2p_msg::storage::queue::{MessageQueue, QueuedMessage, MessageStatus};
use uuid::Uuid;

fn temp_db() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn enqueue_dequeue_persists() {
    let dir = temp_db();
    {
        let q = MessageQueue::new(dir.path().to_str().unwrap()).unwrap();
        let msg = QueuedMessage {
            id: Uuid::new_v4(),
            contact_id: 1,
            payload: b"payload".to_vec(),
            created: 0,
            priority: 1,
            status: MessageStatus::Pending,
            retry_count: 0,
            next_attempt_at: 0,
            max_retries: 3,
        };
        q.enqueue(msg).unwrap();
        assert_eq!(q.len(), 1);
    }
    {
        let q = MessageQueue::new(dir.path().to_str().unwrap()).unwrap();
        let out = q.dequeue().unwrap();
        assert!(out.is_some());
        assert_eq!(q.len(), 0);
    }
}


