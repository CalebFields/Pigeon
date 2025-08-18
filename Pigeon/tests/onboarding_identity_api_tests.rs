use secure_p2p_msg::api::Core;

#[test]
fn first_run_and_identity_preview_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());
    assert!(core.first_run_required());

    let preview = core.ensure_identity_and_preview().unwrap();
    assert!(!preview.sodium_box_pk_hex.is_empty());
    assert!(!preview.sign_pk_hex.is_empty());
    assert!(!core.first_run_required());
}

#[test]
fn passphrase_lock_unlock() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());
    // Ensure identity exists
    let _ = core.ensure_identity_and_preview().unwrap();

    core.set_passphrase("test-pass").unwrap();
    // After sealing, loading at-rest key without unlock should fail; ensure unlock works
    core.unlock("test-pass").unwrap();
}


