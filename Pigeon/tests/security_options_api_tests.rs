use secure_p2p_msg::api::Core;

#[test]
fn passphrase_set_unlock_rotate() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());
    // Ensure identity exists
    let _ = core.ensure_identity_and_preview().unwrap();

    // Set passphrase
    core.set_passphrase("p1").unwrap();
    // Unlock works
    core.unlock("p1").unwrap();
    // Rotate key
    core.rotate_at_rest_key("p2").unwrap();
    // Unlock with new passphrase
    core.unlock("p2").unwrap();
}


