//! Real production executable over JSONL OS pipes. The reverse host is a capture
//! fixture, not a replacement adapter and not evidence of OS clipboard access.
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tezbar_json_format::{COMMANDS, ID};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Process {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    dir: tempfile::TempDir,
}
impl Process {
    async fn new() -> Self {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-data");
        std::fs::create_dir_all(&base).unwrap();
        let dir = tempfile::tempdir_in(base).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tezbar-json-format"))
            .current_dir(dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut p = Self {
            child,
            input,
            output,
            dir,
        };
        let hello = p.recv().await;
        assert_eq!(hello["type"], "hello");
        assert_eq!(hello["id"], ID);
        assert_eq!(hello["sdkVersion"], "0.1.1");
        assert_eq!(hello["protocol"], 1);
        p
    }
    async fn send(&mut self, v: Value) {
        let mut bytes = serde_json::to_vec(&v).unwrap();
        bytes.push(b'\n');
        self.input
            .as_mut()
            .unwrap()
            .write_all(&bytes)
            .await
            .unwrap();
    }
    async fn recv(&mut self) -> Value {
        let mut line = String::new();
        assert!(
            tokio::time::timeout(Duration::from_secs(5), self.output.read_line(&mut line))
                .await
                .expect("protocol deadline")
                .unwrap()
                > 0,
            "unexpected EOF"
        );
        assert!(line.ends_with('\n'));
        assert!(!line.contains('\r'));
        serde_json::from_str(&line).expect("stdout contains non-protocol output")
    }
    async fn request(&mut self, method: &str, params: Value) -> String {
        let n = SERIAL.fetch_add(1, Ordering::SeqCst);
        let id = format!("h:00000000-0000-4000-8000-{n:012x}");
        self.send(json!({"type":"request","id":id,"method":method,"params":params}))
            .await;
        id
    }
    async fn invoke(&mut self, method: &str, params: Value) -> Value {
        let id = self.request(method, params).await;
        self.response(&id).await
    }
    async fn response(&mut self, id: &str) -> Value {
        let v = self.recv().await;
        assert_eq!(v["type"], "response");
        assert_eq!(v["id"], id);
        v
    }
    async fn reply(&mut self, request: &Value, result: Value) {
        assert_eq!(request["type"], "request");
        assert!(request["id"].as_str().unwrap().starts_with("e:"));
        self.send(json!({"type":"response","id":request["id"],"result":result}))
            .await;
    }
    async fn init(&mut self, grants: Value) {
        let v=self.invoke("initialize",json!({"protocol":1,"extensionId":ID,"platform":"macos","packageDir":self.dir.path(),"dataDir":self.dir.path().join("data"),"tempDir":self.dir.path().join("temp"),"grantedCapabilities":grants,"reason":"test","migrationPaths":{}})).await;
        assert_eq!(v["result"]["ready"], true);
    }
    async fn full_init(&mut self) {
        self.init(json!([
            "clipboard.read",
            "clipboard.write",
            "clipboard.pasteText",
            "selection.read",
            "window.visibility"
        ]))
        .await;
    }
    async fn open(&mut self, sid: &str, command: &str, prefs: Value) -> Value {
        let v=self.invoke("session.open",json!({"sessionId":sid,"commandName":command,"preferences":prefs,"argumentValues":{}})).await;
        assert!(v.get("error").is_none(), "{v}");
        v["result"].clone()
    }
    async fn action(&mut self, sid: &str, id: &Value, fields: Value) -> Value {
        self.invoke("session.call",json!({"sessionId":sid,"operation":"extension:invoke-action","payload":{"actionId":id,"formValues":fields}})).await
    }
    async fn action_start(&mut self, sid: &str, id: &Value, fields: Value) -> String {
        self.request("session.call",json!({"sessionId":sid,"operation":"extension:invoke-action","payload":{"actionId":id,"formValues":fields}})).await
    }
    async fn refresh(&mut self, sid: &str) -> Value {
        self.invoke(
            "session.call",
            json!({"sessionId":sid,"operation":"extension:refresh-session","payload":{}}),
        )
        .await
    }
    async fn stop(&mut self) {
        let v = self.invoke("shutdown", json!({})).await;
        assert!(v.get("error").is_none());
        drop(self.input.take());
        assert!(
            tokio::time::timeout(Duration::from_secs(5), self.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
        let mut stderr = String::new();
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .await
            .unwrap();
        assert!(stderr.is_empty(), "{stderr}");
    }
}
fn first(v: &Value) -> Value {
    v["actions"][0]["id"].clone()
}
fn markdown(v: &Value) -> &str {
    v["root"]["props"]["markdown"].as_str().unwrap()
}
#[tokio::test]
async fn automatic_launch_effect_order_and_null_ack_retry() {
    for command in ["formatClipboard", "formatSelection"] {
        for paste in [false, true] {
            let mut p = Process::new().await;
            p.full_init().await;
            let id=p.request("session.open",json!({"sessionId":"auto","commandName":command,"preferences":{"autopaste":paste,"indent":"2"}})).await;
            let read = p.recv().await;
            assert_eq!(
                read["method"],
                if command == "formatClipboard" {
                    "host.clipboard.read"
                } else {
                    "host.selection.read"
                }
            );
            assert_eq!(read["params"], json!({}));
            p.reply(&read, json!("{a:1,}")).await;
            let write = p.recv().await;
            assert_eq!(
                write["method"],
                if paste {
                    "host.clipboard.pasteText"
                } else {
                    "host.clipboard.write"
                }
            );
            assert_eq!(write["params"], json!({"text":"{\n  \"a\": 1\n}"}));
            p.reply(&write, Value::Null).await;
            let failed = p.response(&id).await["result"].clone();
            assert_eq!(failed["title"], "Delivery not confirmed");
            assert_eq!(failed["refreshMs"], Value::Null);
            if paste {
                assert!(markdown(&failed).contains("twice"));
            }
            let retry = p
                .action_start(
                    "auto",
                    &first(&failed),
                    json!({"input":"changed should not be read"}),
                )
                .await;
            let repeated = p.recv().await;
            assert_eq!(repeated["params"], write["params"]);
            assert_eq!(repeated["method"], write["method"]);
            p.reply(&repeated, json!({"ok":true})).await;
            if !paste {
                let close = p.recv().await;
                assert_eq!(close["method"], "host.window.visibility");
                assert_eq!(close["params"], json!({"visible":false}));
                p.reply(&close, json!({"ok":true})).await;
            }
            let done = p.response(&retry).await;
            assert_eq!(done["result"]["mode"], "no-view");
            assert!(p
                .action("auto", &first(&failed), json!({}))
                .await
                .get("error")
                .is_some());
            p.stop().await;
        }
    }
}
#[tokio::test]
async fn both_form_actions_back_and_whole_jsonlines() {
    let mut p = Process::new().await;
    p.full_init().await;
    for command in ["index", "formatJsonLines"] {
        let form = p.open(command, command, json!({"indent":"2"})).await;
        assert_eq!(form["actions"][0]["title"], "Format");
        assert_eq!(form["actions"][1]["title"], "View Result");
        assert_eq!(form["actions"][1]["kind"], "submit-form");
        let input = if command == "index" {
            "{a:1,b:'hi'}"
        } else {
            "[{\"z\":1,\"2\":2,\"1\":1,\"z\":3},-0,1e400,9007199254740993]"
        };
        let detail = p
            .action(command, &form["actions"][1]["id"], json!({"input":input}))
            .await["result"]
            .clone();
        assert_eq!(detail["root"]["type"], "Detail");
        let copy = p.action_start(command, &first(&detail), json!({})).await;
        let rpc = p.recv().await;
        assert_eq!(rpc["method"], "host.clipboard.write");
        if command == "formatJsonLines" {
            assert_eq!(
                rpc["params"]["text"],
                "{\"1\":1,\"2\":2,\"z\":3}\n0\nnull\n9007199254740992"
            );
        }
        p.reply(&rpc, json!({"ok":true})).await;
        let detail2 = p.response(&copy).await["result"].clone();
        assert!(p
            .action(command, &first(&detail), json!({}))
            .await
            .get("error")
            .is_some());
        let back = detail2["root"]["props"]["backActionId"].clone();
        let form2 = p.action(command, &back, json!({})).await["result"].clone();
        assert_eq!(form2["root"]["children"][0]["props"]["value"], input);
        let send = p
            .action_start(command, &first(&form2), json!({"input":input}))
            .await;
        let rpc = p.recv().await;
        p.reply(&rpc, json!({"ok":true})).await;
        let close = p.recv().await;
        assert_eq!(close["method"], "host.window.visibility");
        p.reply(&close, json!({"ok":true})).await;
        assert_eq!(p.response(&send).await["result"]["root"]["type"], "Form");
    }
    p.stop().await;
}
#[tokio::test]
async fn selected_preview_negative_load_all_copy_full_and_stale_session() {
    let mut p = Process::new().await;
    p.full_init().await;
    let id=p.request("session.open",json!({"sessionId":"selected","commandName":"formatSelectionAndShow","preferences":{"maxInitialLines":"-1","indent":"2"}})).await;
    let rpc = p.recv().await;
    assert_eq!(rpc["method"], "host.selection.read");
    p.reply(&rpc, json!("{\"a\":1,\"b\":2}")).await;
    let detail = p.response(&id).await["result"].clone();
    assert_eq!(detail["actions"][0]["title"], "Copy Full JSON");
    assert_eq!(detail["actions"][1]["title"], "Load All Lines");
    assert!(markdown(&detail).contains("Showing -1 of 4"));
    let id = p.action_start("selected", &first(&detail), json!({})).await;
    let rpc = p.recv().await;
    assert_eq!(rpc["params"]["text"], "{\n  \"a\": 1,\n  \"b\": 2\n}");
    p.reply(&rpc, json!({"ok":true})).await;
    let detail = p.response(&id).await["result"].clone();
    let all = p
        .action("selected", &detail["actions"][1]["id"], json!({}))
        .await["result"]
        .clone();
    assert_eq!(all["actions"].as_array().unwrap().len(), 1);
    assert!(!markdown(&all).contains("Showing"));
    assert!(p
        .invoke("session.dispose", json!({"sessionId":"selected"}))
        .await
        .get("error")
        .is_none());
    assert!(p
        .action("selected", &first(&all), json!({}))
        .await
        .get("error")
        .is_some());
    p.stop().await;
}
#[tokio::test]
async fn cancellation_discards_sdk_state_but_never_replays_paste() {
    let mut p = Process::new().await;
    p.full_init().await;
    let form = p.open("cancel", "index", json!({"autopaste":true})).await;
    let id = p
        .action_start("cancel", &first(&form), json!({"input":"[1,2]"}))
        .await;
    let paste = p.recv().await;
    assert_eq!(paste["method"], "host.clipboard.pasteText");
    p.send(json!({"type":"cancel","requestId":id,"reason":"test"}))
        .await;
    let mut cancelled = false;
    let mut response = false;
    for _ in 0..2 {
        let v = p.recv().await;
        if v["type"] == "cancel" {
            assert_eq!(v["requestId"], paste["id"]);
            cancelled = true;
        } else {
            assert_eq!(v["id"], id);
            assert!(v.get("error").is_some());
            response = true;
        }
    }
    assert!(cancelled && response);
    // A late acknowledgement is tolerated by the real SDK, never a new effect.
    p.reply(&paste, json!({"ok":true})).await;
    let recovered = p.refresh("cancel").await["result"].clone();
    assert!(markdown(&recovered).contains("twice"));
    let next = p.refresh("cancel").await["result"].clone();
    assert_eq!(first(&recovered), first(&next));
    let retry = p.action_start("cancel", &first(&next), json!({})).await;
    let again = p.recv().await;
    assert_eq!(again["method"], paste["method"]);
    assert_eq!(again["params"], paste["params"]);
    p.reply(&again, json!({"ok":true})).await;
    assert_eq!(p.response(&retry).await["result"]["root"]["type"], "Form");
    p.stop().await;
}
#[tokio::test]
async fn cancelled_close_retries_close_without_copying() {
    let mut p = Process::new().await;
    p.full_init().await;
    let form = p.open("close", "index", json!({})).await;
    let id = p
        .action_start("close", &first(&form), json!({"input":"0"}))
        .await;
    let copy = p.recv().await;
    p.reply(&copy, json!({"ok":true})).await;
    let close = p.recv().await;
    assert_eq!(close["method"], "host.window.visibility");
    p.send(json!({"type":"cancel","requestId":id})).await;
    for _ in 0..2 {
        p.recv().await;
    }
    let recovery = p.refresh("close").await["result"].clone();
    assert_eq!(recovery["actions"][0]["title"], "Retry Close");
    let retry = p.action_start("close", &first(&recovery), json!({})).await;
    let close2 = p.recv().await;
    assert_eq!(close2["method"], "host.window.visibility");
    p.reply(&close2, json!({"ok":true})).await;
    assert_eq!(p.response(&retry).await["result"]["root"]["type"], "Form");
    p.stop().await;
}
#[tokio::test]
async fn storage_restart_queued_drafts_and_preferences_are_snapshots() {
    let mut p = Process::new().await;
    p.full_init().await;
    let form = p
        .open("text", "formatToJsonValue", json!({"autopaste":true}))
        .await;
    let edit = form["root"]["children"][0]["props"]["actionId"].clone();
    let a = p
        .action_start(
            "text",
            &edit,
            json!({"value":"old","__tezbarDraftRevision":1}),
        )
        .await;
    let b = p
        .action_start(
            "text",
            &edit,
            json!({"value":"latest\n\"😀\"","__tezbarDraftRevision":2}),
        )
        .await;
    let one = p.recv().await;
    let two = p.recv().await;
    assert!([a.clone(), b.clone()].contains(&one["id"].as_str().unwrap().to_owned()));
    assert!([a, b].contains(&two["id"].as_str().unwrap().to_owned()));
    let result =
        if two["result"]["root"]["children"][1]["props"]["text"] == "\"latest\\n\\\"😀\\\"\"" {
            &two["result"]
        } else {
            &one["result"]
        };
    assert!(result["root"]["children"][0]["props"]
        .get("value")
        .is_none());
    let copy = p
        .action_start(
            "text",
            &first(result),
            json!({"input":"submitted latest","__tezbarDraftRevision":2}),
        )
        .await;
    let rpc = p.recv().await;
    assert_eq!(rpc["method"], "host.clipboard.write");
    assert_eq!(rpc["params"]["text"], "\"submitted latest\"");
    p.reply(&rpc, json!({"ok":true})).await;
    p.response(&copy).await;
    let store = p.dir.path().join("data/json-format/storage.json");
    let data: Value = serde_json::from_slice(&std::fs::read(&store).unwrap()).unwrap();
    assert_eq!(data["text"], "submitted latest");
    let restored = p.open("again", "formatToJsonValue", json!({})).await;
    assert_eq!(
        restored["root"]["children"][0]["props"]["value"],
        "submitted latest"
    );
    // Forged payload preferences do not replace SDK's frozen session snapshot.
    let form = p
        .open("snapshot", "index", json!({"autopaste":false}))
        .await;
    let id=p.request("session.call",json!({"sessionId":"snapshot","operation":"extension:invoke-action","payload":{"actionId":first(&form),"formValues":{"input":"1"},"preferences":{"autopaste":true}}})).await;
    let rpc = p.recv().await;
    assert_eq!(rpc["method"], "host.clipboard.write");
    p.reply(&rpc, json!({"ok":true})).await;
    let close = p.recv().await;
    p.reply(&close, json!({"ok":true})).await;
    p.response(&id).await;
    p.stop().await;
}
#[tokio::test]
async fn capability_errors_and_contradictory_ack_are_never_success() {
    let mut p = Process::new().await;
    p.init(json!([])).await;
    let denied = p
        .invoke(
            "session.open",
            json!({"sessionId":"denied","commandName":"formatClipboard"}),
        )
        .await;
    assert!(denied["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not granted"));
    p.stop().await;
    for bad in [
        json!({"ok":false}),
        json!({"ok":true,"error":"failed"}),
        json!({"ok":true,"success":false}),
        Value::Null,
    ] {
        let mut p = Process::new().await;
        p.full_init().await;
        let form = p.open("v", "formatToJsonValue", json!({})).await;
        let id = p
            .action_start(
                "v",
                &first(&form),
                json!({"input":"x","__tezbarDraftRevision":0}),
            )
            .await;
        let rpc = p.recv().await;
        p.reply(&rpc, bad).await;
        assert_eq!(
            p.response(&id).await["result"]["title"],
            "Delivery not confirmed"
        );
        p.stop().await;
    }
}
#[test]
fn protocol_info_and_self_test_are_real_sdk_entrypoints() {
    for flag in ["--protocol-info", "--self-test"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_tezbar-json-format"))
            .arg(flag)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let v: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(v["id"], ID);
        assert_eq!(v["commands"], json!(COMMANDS));
    }
}
#[tokio::test]
async fn cancelled_open_is_removed_by_sdk_and_id_reuse_is_fenced() {
    let mut p = Process::new().await;
    p.full_init().await;
    let id=p.request("session.open",json!({"sessionId":"orphan","commandName":"formatClipboard","preferences":{"autopaste":true}})).await;
    let read = p.recv().await;
    p.reply(&read, json!("[1,2]")).await;
    let paste = p.recv().await;
    p.send(json!({"type":"cancel","requestId":id})).await;
    for _ in 0..2 {
        p.recv().await;
    }
    assert!(p.refresh("orphan").await.get("error").is_some());
    p.reply(&paste, json!({"ok":true})).await;
    // The host must not reuse IDs; defensive reopening only exposes recovery and
    // must not read, reconvert or replay an already attempted automatic effect.
    let reopened = p.open("orphan", "formatClipboard", json!({})).await;
    assert_eq!(reopened["title"], "Delivery not confirmed");
    assert!(markdown(&reopened).contains("twice"));
    p.stop().await;
}
#[tokio::test]
async fn cancelled_storage_effect_is_durable_across_real_process_restart() {
    let mut p = Process::new().await;
    p.full_init().await;
    let form = p.open("persist", "formatToJsonValue", json!({})).await;
    let id = p
        .action_start(
            "persist",
            &first(&form),
            json!({"input":"stored despite cancelled call","__tezbarDraftRevision":0}),
        )
        .await;
    let copy = p.recv().await;
    assert_eq!(copy["method"], "host.clipboard.write");
    p.send(json!({"type":"cancel","requestId":id})).await;
    for _ in 0..2 {
        p.recv().await;
    }
    let recovered = p.refresh("persist").await["result"].clone();
    assert_eq!(recovered["title"], "Delivery not confirmed");
    let data = p.dir.path().join("data");
    p.stop().await;
    // Keep first TempDir alive, so the second executable sees the same real store.
    let mut restarted = Process::new().await;
    let v=restarted.invoke("initialize",json!({"protocol":1,"extensionId":ID,"platform":"macos","packageDir":restarted.dir.path(),"dataDir":data,"tempDir":restarted.dir.path().join("temp"),"grantedCapabilities":["clipboard.write"],"reason":"restart","migrationPaths":{}})).await;
    assert_eq!(v["result"]["ready"], true);
    let form = restarted
        .open("restarted", "formatToJsonValue", json!({}))
        .await;
    assert_eq!(
        form["root"]["children"][0]["props"]["value"],
        "stored despite cancelled call"
    );
    restarted.stop().await;
}
#[tokio::test]
async fn malformed_wire_surrogates_and_framing_fail_on_stderr() {
    for line in [
        b"not json\n".as_slice(),
        b"{\"type\":\"event\",\"data\":\"\\ud800\"}\n",
        b"{}\r\n",
    ] {
        let mut p = Process::new().await;
        p.input.as_mut().unwrap().write_all(line).await.unwrap();
        drop(p.input.take());
        assert!(
            !tokio::time::timeout(Duration::from_secs(5), p.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
        let mut error = String::new();
        p.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut error)
            .await
            .unwrap();
        assert!(error.starts_with("json-format:"));
        let mut rest = String::new();
        p.output.read_to_string(&mut rest).await.unwrap();
        assert!(rest.is_empty(), "stdout must remain protocol only");
    }
}
#[tokio::test]
async fn eof_cancels_pending_reverse_without_replay_or_hang() {
    let mut p = Process::new().await;
    p.full_init().await;
    p.request(
        "session.open",
        json!({"sessionId":"eof","commandName":"formatClipboard"}),
    )
    .await;
    let read = p.recv().await;
    assert_eq!(read["method"], "host.clipboard.read");
    drop(p.input.take());
    assert!(tokio::time::timeout(Duration::from_secs(5), p.child.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
}
#[tokio::test]
async fn queued_edit_after_copy_keeps_stable_edit_id_and_persists_newer_text() {
    let mut p = Process::new().await;
    p.full_init().await;
    let form = p.open("editing", "formatToJsonValue", json!({})).await;
    let edit = form["root"]["children"][0]["props"]["actionId"].clone();
    let copy_id = p
        .action_start(
            "editing",
            &first(&form),
            json!({"input":"copied snapshot","__tezbarDraftRevision":0}),
        )
        .await;
    let rpc = p.recv().await;
    assert_eq!(rpc["params"]["text"], "\"copied snapshot\"");
    let edit_id = p
        .action_start(
            "editing",
            &edit,
            json!({"value":"new draft after copy","__tezbarDraftRevision":1}),
        )
        .await;
    p.reply(&rpc, json!({"ok":true})).await;
    let a = p.recv().await;
    let b = p.recv().await;
    assert!([copy_id.clone(), edit_id.clone()].contains(&a["id"].as_str().unwrap().to_owned()));
    assert!([copy_id, edit_id].contains(&b["id"].as_str().unwrap().to_owned()));
    let file = p.dir.path().join("data/json-format/storage.json");
    let stored: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert_eq!(stored["text"], "new draft after copy");
    p.stop().await;
}
