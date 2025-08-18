use secure_p2p_msg::messaging::{receive::receive_and_ack, send::send_now};
use secure_p2p_msg::storage::queue::MessageQueue;

#[tokio::test]
async fn round_trip_and_drain_on_reconnect() {
    sodiumoxide::init().unwrap();
    let (pk_a, sk_a) = sodiumoxide::crypto::box_::gen_keypair();
    let (pk_b, sk_b) = sodiumoxide::crypto::box_::gen_keypair();

    let dir_b = tempfile::tempdir().unwrap();
    let queue_b = dir_b.path().to_str().unwrap();

    // A sends two messages to B (persisted in B's queue)
    let _ = send_now(queue_b, &sk_a, &pk_b, 100, b"hello")
        .await
        .unwrap();
    let _ = send_now(queue_b, &sk_a, &pk_b, 100, b"world")
        .await
        .unwrap();

    // Simulate reconnect by draining with successive opens to avoid sled lock contention
    loop {
        let empty = {
            let qtmp = MessageQueue::new(queue_b).unwrap();
            qtmp.is_empty()
        };
        if empty {
            break;
        }
        // Receiver uses sender's public key (A) and its own secret key (B)
        receive_and_ack(queue_b, &pk_a, &sk_b).await.unwrap();
    }

    // Inbox should contain both plaintext messages
    let q = MessageQueue::new(queue_b).unwrap();
    let inbox = q.list_inbox().unwrap();
    let texts: Vec<String> = inbox
        .into_iter()
        .map(|(_, v)| String::from_utf8_lossy(&v).to_string())
        .collect();
    assert!(texts.contains(&"hello".to_string()));
    assert!(texts.contains(&"world".to_string()));
}
