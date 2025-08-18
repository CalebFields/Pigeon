use secure_p2p_msg::api::Core;

#[test]
fn contacts_crud_and_find() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());

    // Add
    let c = core
        .contacts_add(
            "Alice",
            "/ip4/127.0.0.1/tcp/1234",
            &hex::encode([1u8; 32]),
        )
        .unwrap();
    assert_eq!(c.name, "Alice");
    // List
    let list = core.contacts_list().unwrap();
    assert_eq!(list.len(), 1);
    // Find by name (case-insensitive)
    let found = core.contacts_find_by_name("alice").unwrap().unwrap();
    assert_eq!(found.id, c.id);
    // Update
    let c2 = core
        .contacts_update(
            c.id,
            "Alice Updated",
            "/ip4/127.0.0.1/tcp/4321",
            &hex::encode([2u8; 32]),
        )
        .unwrap();
    assert_eq!(c2.name, "Alice Updated");
    // Remove
    assert!(core.contacts_remove(c.id).unwrap());
    assert!(core.contacts_list().unwrap().is_empty());
}

#[tokio::test]
async fn inbox_list_limited_works() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());
    let queue_path = dir.path().join("queue_db");
    let q = secure_p2p_msg::storage::queue::MessageQueue::new(queue_path.to_str().unwrap()).unwrap();
    // store two items
    let id1 = uuid::Uuid::new_v4();
    q.store_inbox(id1, b"one".to_vec()).unwrap();
    let id2 = uuid::Uuid::new_v4();
    q.store_inbox(id2, b"two".to_vec()).unwrap();
    drop(q);

    let all = core.inbox_list().unwrap();
    assert_eq!(all.len(), 2);
    let limited = core.inbox_list_limited(1).unwrap();
    assert_eq!(limited.len(), 1);
}


