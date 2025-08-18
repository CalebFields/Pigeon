use secure_p2p_msg::storage::queue::{MessageQueue, MessageStatus, QueuedMessage};
use uuid::Uuid;

#[test]
fn exponential_backoff_and_dead_letter() {
    let dir = tempfile::tempdir().unwrap();
    let q = MessageQueue::new(dir.path().to_str().unwrap()).unwrap();

    let id = Uuid::new_v4();
    let msg = QueuedMessage {
        id,
        contact_id: 7,
        payload: b"x".to_vec(),
        created: 0,
        priority: 1,
        status: MessageStatus::Pending,
        retry_count: 0,
        next_attempt_at: 0,
        max_retries: 2,
    };
    q.enqueue(msg).unwrap();

    // First dequeue should return Some (due immediately)
    let mut m = q.dequeue().unwrap().expect("expected msg");
    // Simulate failure -> requeue with backoff
    q.requeue_with_backoff(m, 1).unwrap();

    // Immediately dequeuing again should yield None due to backoff
    assert!(q.dequeue().unwrap().is_none());

    // Force schedule to now by crafting a due message
    m = QueuedMessage {
        id: Uuid::new_v4(),
        contact_id: 7,
        payload: vec![],
        created: 0,
        priority: 1,
        status: MessageStatus::Pending,
        retry_count: 2,
        next_attempt_at: 0,
        max_retries: 2,
    };
    q.requeue_or_dead_letter(m, 1, "fail").unwrap();
    // Since retry_count >= max_retries, it should be placed into DLQ
    assert_eq!(q.dead_letter_len(), 1);
}
