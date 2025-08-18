use secure_p2p_msg::api::Core;

#[tokio::test]
async fn core_facade_compose_and_inbox_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());

    // Compose stores plaintext into queue as Pending (pre-encryption stage)
    let id = core.compose(1, "hello world").await.unwrap();

    // Simulate delivery by directly moving to inbox via storage API for smoke test
    let q = secure_p2p_msg::storage::queue::MessageQueue::new(
        dir.path().join("queue_db").to_str().unwrap(),
    )
    .unwrap();
    // Pull one pending and store to inbox (mimicking receive logic)
    if let Some(msg) = q.dequeue().unwrap() {
        q.store_inbox(msg.id, msg.payload).unwrap();
    }
    drop(q); // release sled lock before next Core call

    // Ensure inbox shows the message
    let items = core.inbox_list().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, id);
    assert!(String::from_utf8_lossy(&items[0].1).contains("hello"));

    // Search finds it
    let found = core.inbox_search("WORLD", None).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, id);

    // Export works
    let out = dir.path().join("msg.txt");
    core.inbox_export(id, &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("hello"));

    // Stats are populated
    let stats = core.queue_stats().unwrap();
    assert_eq!(stats.inbox, 1);
}


