use secure_p2p_msg::storage::queue::{MessageQueue, MessageStatus, QueuedMessage};
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

#[test]
fn priority_lane_prefers_high() {
    sodiumoxide::init().unwrap();
    let dir = temp_db();
    let path = dir.path().to_str().unwrap();
    let q = MessageQueue::new(path).unwrap();

    // Enqueue a normal message due now
    let normal = QueuedMessage {
        id: Uuid::new_v4(),
        contact_id: 1,
        payload: b"n".to_vec(),
        created: 0,
        priority: 1,
        status: MessageStatus::Pending,
        retry_count: 0,
        next_attempt_at: 0,
        max_retries: 3,
    };
    q.enqueue(normal).unwrap();

    // Enqueue a high-priority message due now
    let high = QueuedMessage {
        id: Uuid::new_v4(),
        contact_id: 1,
        payload: b"h".to_vec(),
        created: 0,
        priority: 0,
        status: MessageStatus::Pending,
        retry_count: 0,
        next_attempt_at: 0,
        max_retries: 3,
    };
    q.enqueue(high).unwrap();

    // First dequeue should fetch high-priority
    let first = q.dequeue().unwrap().unwrap();
    assert_eq!(first.payload, b"h");

    let second = q.dequeue().unwrap().unwrap();
    assert_eq!(second.payload, b"n");
}

#[test]
fn weighted_fairness_pattern() {
    sodiumoxide::init().unwrap();
    let dir = temp_db();
    let path = dir.path().to_str().unwrap();
    let q = MessageQueue::new(path).unwrap();

    // Enqueue 4 high (h1..h4) and 2 normal (n1..n2)
    let make = |b: &str, prio: u8| QueuedMessage {
        id: Uuid::new_v4(),
        contact_id: 1,
        payload: b.as_bytes().to_vec(),
        created: 0,
        priority: prio,
        status: MessageStatus::Pending,
        retry_count: 0,
        next_attempt_at: 0,
        max_retries: 3,
    };
    for name in ["h1", "h2", "h3", "h4"] {
        q.enqueue(make(name, 0)).unwrap();
    }
    for name in ["n1", "n2"] {
        q.enqueue(make(name, 1)).unwrap();
    }

    // Dequeue with manual fairness loop similar to runtime: 2 highs then 1 normal
    let ratio = 2u8;
    let mut high_budget = ratio;
    let mut order = Vec::new();
    loop {
        let pick = if high_budget > 0 {
            match q.dequeue_from_priority(0).unwrap() {
                Some(m) => {
                    high_budget -= 1;
                    Some(m)
                }
                None => q.dequeue_from_priority(1).unwrap(),
            }
        } else {
            match q.dequeue_from_priority(1).unwrap() {
                Some(m) => {
                    high_budget = ratio;
                    Some(m)
                }
                None => q.dequeue_from_priority(0).unwrap(),
            }
        };
        match pick {
            Some(m) => order.push(String::from_utf8(m.payload).unwrap()),
            None => break,
        }
    }

    // Expect first three to follow 2H then 1N pattern (names may differ due to UUID ordering)
    assert!(order.len() >= 3);
    assert!(order[0].starts_with('h'));
    assert!(order[1].starts_with('h'));
    assert!(order[2].starts_with('n'));
}
