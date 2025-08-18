use secure_p2p_msg::api::Core;

#[tokio::test]
async fn inbox_watcher_emits_on_change() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());
    let mut watcher = core.watch_inbox(50);

    // Initially empty; no immediate event guaranteed. Add one inbox item.
    let q = secure_p2p_msg::storage::queue::MessageQueue::new(
        dir.path().join("queue_db").to_str().unwrap(),
    )
    .unwrap();
    let id = uuid::Uuid::new_v4();
    q.store_inbox(id, b"hello".to_vec()).unwrap();
    drop(q);

    // Expect a snapshot within a short time
    let got = tokio::time::timeout(std::time::Duration::from_millis(500), watcher.recv())
        .await
        .ok()
        .flatten()
        .expect("no snapshot");
    assert_eq!(got.len, 1);
    assert!(got.latest.is_some());
}

#[tokio::test]
async fn queue_summaries_present() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());

    // Enqueue a pending message directly via compose
    let _ = core.compose(1, "body").await.unwrap();
    let summaries = core.queue_list_pending_summaries().unwrap();
    assert_eq!(summaries.len(), 1);
    let s = &summaries[0];
    assert_eq!(s.contact_id, 1);
}


