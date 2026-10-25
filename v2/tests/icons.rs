#[rustfmt::skip]
#[path="../fixtures/icon-contract.rs"]mod registry;
use serde_json::json;
#[test]
fn actual_registry_before_after_resolves_original_process_png_offline() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-data");
    std::fs::create_dir_all(&base).unwrap();
    let dir = tempfile::tempdir_in(base).unwrap();
    let managed = dir.path().join("packages/com.tezbar.native.json-format");
    let version = managed.join("versions/0.2.0-fixture");
    std::fs::create_dir_all(version.join("resources")).unwrap();
    let icon = version.join("resources/command-icon.png");
    let bytes = include_bytes!("../resources/command-icon.png");
    std::fs::write(&icon, bytes).unwrap();
    let m = json!({"icon":"resources/command-icon.png"});
    let c = m.clone();
    let before = registry::before::icons(managed.clone(), version.clone(), m.clone(), c.clone());
    assert_eq!(before.0, None);
    assert_eq!(before.1, None);
    let after = registry::after::icons(managed.clone(), version, m, c);
    assert_eq!(after.0.as_deref(), icon.to_str());
    assert_eq!(after.1.as_deref(), icon.to_str());
    assert_eq!(after.2, managed);
    assert_eq!(std::fs::read(after.0.unwrap()).unwrap().as_slice(), bytes);
}
