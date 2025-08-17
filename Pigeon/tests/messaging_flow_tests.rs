use secure_p2p_msg::messaging::send::send_now;
use secure_p2p_msg::storage::queue::MessageQueue;

#[tokio::test]
async fn send_enqueues_ciphertext() {
    sodiumoxide::init().unwrap();
    let (pk_a, sk_a) = sodiumoxide::crypto::box_::gen_keypair();
    let (pk_b, _sk_b) = sodiumoxide::crypto::box_::gen_keypair();

    let dir = tempfile::tempdir().unwrap();
    let queue_path = dir.path().to_str().unwrap();

    let id = send_now(queue_path, &sk_a, &pk_b, 42, b"hello").await.unwrap();
    let q = MessageQueue::new(queue_path).unwrap();
    assert_eq!(q.len(), 1);
    let item = q.dequeue().unwrap().unwrap();
    assert_eq!(item.id, id);
    assert!(item.payload.len() > sodiumoxide::crypto::box_::NONCEBYTES);
}


