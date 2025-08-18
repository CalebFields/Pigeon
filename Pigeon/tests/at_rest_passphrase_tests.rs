use serial_test::serial;

#[test]
#[serial]
fn set_passphrase_creates_encrypted_keyfile() {
    sodiumoxide::init().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path();

    // Seal with passphrase
    secure_p2p_msg::storage::at_rest::set_passphrase_and_seal(data_dir, "test-pass").unwrap();

    let enc = data_dir.join("at_rest.key.enc");
    let plain = data_dir.join("at_rest.key");
    assert!(enc.exists());
    assert!(!plain.exists());

    // Check magic header
    let bytes = std::fs::read(enc).unwrap();
    assert!(bytes.len() > 4 + 16 + 24);
    assert_eq!(&bytes[0..4], b"PGN1");
}

#[test]
#[serial]
fn unlock_with_passphrase_roundtrip() {
    sodiumoxide::init().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path();

    secure_p2p_msg::storage::at_rest::set_passphrase_and_seal(data_dir, "pw").unwrap();
    let key = secure_p2p_msg::storage::at_rest::unlock_with_passphrase(data_dir, "pw").unwrap();

    let msg = b"hello at-rest".to_vec();
    let enc = secure_p2p_msg::storage::at_rest::encrypt(&key, &msg).unwrap();
    let dec = secure_p2p_msg::storage::at_rest::decrypt(&key, &enc).unwrap();
    assert_eq!(dec, msg);
}

#[test]
#[serial]
fn load_or_create_requires_unlock_if_locked() {
    sodiumoxide::init().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path();

    // Simulate presence of a sealed keyfile without providing passphrase
    std::fs::write(data_dir.join("at_rest.key.enc"), []).unwrap();
    let res = secure_p2p_msg::storage::at_rest::AtRestKey::load_or_create(data_dir);
    assert!(res.is_err(), "expected locked error without passphrase env");
}

#[test]
#[serial]
fn load_or_create_env_unlocks() {
    sodiumoxide::init().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path();

    secure_p2p_msg::storage::at_rest::set_passphrase_and_seal(data_dir, "pw-env").unwrap();
    std::env::set_var("PIGEON_PASSPHRASE", "pw-env");
    let key = secure_p2p_msg::storage::at_rest::AtRestKey::load_or_create(data_dir).unwrap();

    // Roundtrip
    let msg = b"env unlock".to_vec();
    let enc = secure_p2p_msg::storage::at_rest::encrypt(&key, &msg).unwrap();
    let dec = secure_p2p_msg::storage::at_rest::decrypt(&key, &enc).unwrap();
    assert_eq!(dec, msg);

    // Cleanup env to avoid leaking into other tests
    std::env::remove_var("PIGEON_PASSPHRASE");
}
