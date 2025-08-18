use secure_p2p_msg::storage::contacts::ContactStore;

#[test]
fn resolves_contact_by_name_and_id() {
    sodiumoxide::init().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();

    // Insert a contact
    let c = {
        let store = ContactStore::open_in_dir(&data_dir).unwrap();
        let pk = hex::encode([1u8; 32]);
        let c = store.add("Alice", "/ip4/127.0.0.1/tcp/4001", &pk).unwrap();
        // Release sled lock before reopening in resolver
        drop(store);
        c
    };

    // Resolve by name
    let (addr1, pk1) = secure_p2p_msg::ui::resolve_contact_or_args(
        &data_dir, Some("alice"), None, None,
    ).unwrap();
    assert_eq!(addr1, c.addr);
    assert_eq!(pk1.0.as_slice(), c.public_key.as_slice());

    // Resolve by id
    let (addr2, pk2) = secure_p2p_msg::ui::resolve_contact_or_args(
        &data_dir, Some(&c.id.to_string()), None, None,
    ).unwrap();
    assert_eq!(addr2, c.addr);
    assert_eq!(pk2.0.as_slice(), c.public_key.as_slice());

    // Resolve explicit args fallback
    let pk_hex = hex::encode([2u8; 32]);
    let (addr3, _pk3) = secure_p2p_msg::ui::resolve_contact_or_args(
        &data_dir, None, Some("/ip4/1.2.3.4/tcp/1234"), Some(&pk_hex),
    ).unwrap();
    assert_eq!(addr3, "/ip4/1.2.3.4/tcp/1234");
}


