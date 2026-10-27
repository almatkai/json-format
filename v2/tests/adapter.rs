use serde_json::{json, Value};
use tezbar_extension_sdk::{Extension, ExtensionPaths, TestContext};
use tezbar_json_format::Adapter;
fn context() -> (tempfile::TempDir, TestContext) {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-data");
    std::fs::create_dir_all(&base).unwrap();
    let dir = tempfile::tempdir_in(base).unwrap();
    let ctx = TestContext::new(
        ExtensionPaths {
            package_dir: dir.path().into(),
            data_dir: dir.path().join("data"),
            temp_dir: dir.path().join("temp"),
        },
        &["clipboard.write"],
    );
    (dir, ctx)
}
#[tokio::test]
async fn typed_atomic_store_rejects_corruption_preserves_unknown_and_fences_drafts() {
    let (dir, t) = context();
    let file = dir.path().join("data/json-format/storage.json");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        b"{\"text\":\"old\",\"revision\":4,\"unknown\":{\"preserve\":true}}",
    )
    .unwrap();
    let adapter = Adapter::default();
    let mut state = json!({});
    let payload = json!({"sessionId":"direct","preferences":{}});
    let initial = adapter
        .handle(
            &t.context,
            "formatToJsonValue",
            "extension:run-command",
            &payload,
            &mut state,
        )
        .await
        .unwrap();
    let change = initial["root"]["children"][0]["props"]["actionId"].clone();
    adapter.handle(&t.context,"formatToJsonValue","extension:invoke-action",&json!({"sessionId":"direct","actionId":change,"formValues":{"value":"new","__tezbarDraftRevision":2}}),&mut state).await.unwrap();
    let ignored=adapter.handle(&t.context,"formatToJsonValue","extension:invoke-action",&json!({"sessionId":"direct","actionId":change,"formValues":{"value":"stale","__tezbarDraftRevision":1}}),&mut state).await.unwrap();
    assert_eq!(ignored["root"]["type"], "Form");
    assert_eq!(ignored["root"]["children"][1]["props"]["text"], "\"new\"");
    let store: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(store["text"], "new");
    assert_eq!(store["unknown"], json!({"preserve":true}));
    for (i, bad) in [
        b"{\"text\":false}".as_slice(),
        b"{\"revision\":\"bad\"}",
        b"[]",
        b"broken",
        b"{\"revision\":18446744073709551615}",
    ]
    .into_iter()
    .enumerate()
    {
        std::fs::write(&file, bad).unwrap();
        assert!(adapter.handle(&t.context,"formatToJsonValue","extension:invoke-action",&json!({"sessionId":"direct","actionId":change,"formValues":{"value":"write","__tezbarDraftRevision":3+i}}),&mut state).await.is_err());
        assert_eq!(std::fs::read(&file).unwrap(), bad);
    }
    assert!(t.calls().is_empty());
}
#[tokio::test]
async fn actual_sdk_test_context_does_not_treat_default_null_as_copy_success() {
    let (_dir, t) = context();
    let adapter = Adapter::default();
    let mut state = json!({});
    let form = adapter
        .handle(
            &t.context,
            "formatToJsonValue",
            "extension:run-command",
            &json!({"sessionId":"ack","preferences":{}}),
            &mut state,
        )
        .await
        .unwrap();
    let result=t.run(adapter.handle(&t.context,"formatToJsonValue","extension:invoke-action",&json!({"sessionId":"ack","actionId":form["actions"][0]["id"],"formValues":{"input":"hi","__tezbarDraftRevision":0}}),&mut state)).await.unwrap();
    assert_eq!(result["title"], "Delivery not confirmed");
    assert_eq!(t.calls().len(), 1);
    assert_eq!(t.calls()[0]["method"], "host.clipboard.write");
}
