use secure_p2p_msg::api::Core;
use secure_p2p_msg::settings::AccessibilitySettings;

#[test]
fn accessibility_settings_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_data_dir(dir.path());

    // Defaults
    let def = core.get_accessibility().unwrap();
    assert_eq!(def, AccessibilitySettings::default());

    let new = AccessibilitySettings { reduce_motion: true, high_contrast: true, larger_text: false };
    core.set_accessibility(new).unwrap();
    let loaded = core.get_accessibility().unwrap();
    assert_eq!(loaded, new);
}


