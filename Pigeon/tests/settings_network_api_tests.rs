use secure_p2p_msg::api::Core;

#[test]
fn log_level_get_set() {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::with_data_dir(dir.path());
    assert_eq!(core.get_log_level(), "info");
    core.set_log_level("debug");
    assert_eq!(core.get_log_level(), "debug");
}

#[cfg(feature = "network")]
#[test]
fn network_settings_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::with_data_dir(dir.path());
    let mut s = core.get_network_settings();
    s.listen_addr = Some("/ip4/0.0.0.0/tcp/4001".to_string());
    s.enable_mdns = true;
    core.set_network_settings(s.clone()).unwrap();
    assert_eq!(core.get_network_settings(), s);
}


