use std::{env, path::PathBuf};
use serial_test::serial;

#[test]
#[serial]
fn loads_default_when_no_file() {
    let cfg = secure_p2p_msg::config::load();
    // Accept either system data dir/pigeon or fallback .\pigeon
    let tail = cfg.data_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    assert_eq!(tail, "pigeon");
    assert_eq!(cfg.log_level, "info");
}

#[test]
#[serial]
fn env_overrides_apply() {
    let temp = tempfile::tempdir().unwrap();
    env::set_var("PIGEON_DATA_DIR", temp.path());
    env::set_var("PIGEON_LOG_LEVEL", "debug");

    let cfg = secure_p2p_msg::config::load();
    assert_eq!(cfg.data_dir, PathBuf::from(temp.path()));
    assert_eq!(cfg.log_level, "debug");

    env::remove_var("PIGEON_DATA_DIR");
    env::remove_var("PIGEON_LOG_LEVEL");
}


