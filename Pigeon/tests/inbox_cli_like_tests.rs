use secure_p2p_msg::storage::queue::MessageQueue;

#[test]
fn inbox_export_roundtrip() {
    // No crypto required here; we're only exercising storage/inbox
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue_db");
    let q = MessageQueue::new(path.to_str().unwrap()).unwrap();

    let id = uuid::Uuid::new_v4();
    let body = b"hello world".to_vec();
    q.store_inbox(id, body.clone()).unwrap();

    let fetched = q.get_inbox(id).unwrap().expect("inbox present");
    assert_eq!(fetched, body);

    // Simulate CLI export to file
    let outfile = dir.path().join("out.txt");
    std::fs::write(&outfile, &fetched).unwrap();
    let disk = std::fs::read(&outfile).unwrap();
    assert_eq!(disk, body);
}

#[test]
fn inbox_search_case_insensitive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue_db");
    let q = MessageQueue::new(path.to_str().unwrap()).unwrap();

    let msgs = [
        (uuid::Uuid::new_v4(), "Hello there".to_string()),
        (uuid::Uuid::new_v4(), "world stage".to_string()),
        (uuid::Uuid::new_v4(), "other".to_string()),
    ];
    for (id, s) in &msgs {
        q.store_inbox(*id, s.clone().into_bytes()).unwrap();
    }

    let items = q.list_inbox().unwrap();
    let needle = "HELLO".to_lowercase();
    let mut hits: Vec<String> = items
        .into_iter()
        .filter_map(|(_id, bytes)| String::from_utf8(bytes).ok())
        .filter(|txt| txt.to_lowercase().contains(&needle))
        .collect();
    hits.sort();
    assert_eq!(hits, vec!["Hello there".to_string()]);
}
