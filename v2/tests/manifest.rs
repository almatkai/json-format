#[rustfmt::skip]
#[path = "../fixtures/host-contract.rs"]
mod host;
use serde_json::{json, Value};
#[test]
fn actual_committed_runtime_validator_and_negative_cases() {
    let source: Value = serde_json::from_str(include_str!("../extension.json")).unwrap();
    host::validate(&source).unwrap();
    assert!(host::artifact(&source, "aarch64-apple-darwin").is_err());
    for (field, bad) in [
        ("schemaVersion", json!(2)),
        ("protocolVersion", json!(2)),
        ("runtime", json!("javascript")),
        ("execution", json!("builtin")),
        ("id", json!("com.tezbar.native.other")),
        ("capabilities", json!(["core.any"])),
    ] {
        let mut m = source.clone();
        m[field] = bad;
        assert!(host::validate(&m).is_err(), "{field}");
    }
    let mut duplicate = source.clone();
    duplicate["commands"]
        .as_array_mut()
        .unwrap()
        .push(source["commands"][0].clone());
    assert!(host::validate(&duplicate).is_err());
    for field in ["timeoutMs", "refreshMs"] {
        let mut m = source.clone();
        m["commands"][0][field] = json!(1);
        if field == "refreshMs" {
            assert!(host::validate(&m).is_err());
        }
    }
    let mut bad = source.clone();
    bad["migrationPaths"] = json!({"escape":"../data"});
    assert!(host::validate(&bad).is_err());
    let icon = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/command-icon.png"
    ))
    .unwrap();
    use sha2::{Digest, Sha256};
    assert_eq!(icon.len(), 237176);
    assert_eq!(
        format!("{:x}", Sha256::digest(icon)),
        "cfee8098d945a0541c391777deeae1fc6e3ddee07a8516f1f730b2103e4e8f13"
    );
}
#[test]
fn package_member_validation_is_actual_host_code() {
    use sha2::{Digest, Sha256};
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-data");
    std::fs::create_dir_all(&base).unwrap();
    let dir = tempfile::tempdir_in(base).unwrap();
    std::fs::create_dir(dir.path().join("bin")).unwrap();
    std::fs::write(dir.path().join("bin/json-format"), b"fixture bytes").unwrap();
    let hash = format!("{:x}", Sha256::digest(b"fixture bytes"));
    let mut m = json!({"files":{"bin/json-format":hash}});
    let a = json!({"executable":"bin/json-format","executableSha256":hash});
    host::files(dir.path(), &m, &a).unwrap();
    m["files"]["../escape"] = json!("0".repeat(64));
    assert!(host::files(dir.path(), &m, &a).is_err());
}
#[test]
#[ignore = "run with JSON_FORMAT_PACKAGE_DIR pointing at an extracted built ZIP"]
fn extracted_package_against_actual_host_validator() {
    let dir = std::path::PathBuf::from(
        std::env::var_os("JSON_FORMAT_PACKAGE_DIR").expect("extracted package path required"),
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir.join("extension.json")).unwrap()).unwrap();
    host::validate(&manifest).unwrap();
    let exe = if cfg!(windows) {
        "bin/json-format.exe"
    } else {
        "bin/json-format"
    };
    let artifact = json!({"executable":exe,"executableSha256":manifest["files"][exe]});
    host::files(&dir, &manifest, &artifact).unwrap();
    let icon = dir.join("resources/command-icon.png");
    let bytes = std::fs::read(&icon).unwrap();
    std::fs::write(&icon, b"tampered").unwrap();
    assert!(host::files(&dir, &manifest, &artifact).is_err());
    std::fs::write(&icon, bytes).unwrap();
    host::files(&dir, &manifest, &artifact).unwrap();
}
