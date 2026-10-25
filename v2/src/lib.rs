pub mod convert;
mod ui;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};
use tezbar_extension_sdk::{
    read_store, update_store, BoxFuture, CoreResult, Extension, ExtensionContext,
};

pub const ID: &str = "com.tezbar.native.json-format";
pub const COMMANDS: &[&str] = &[
    "formatClipboard",
    "formatSelection",
    "formatSelectionAndShow",
    "index",
    "formatJsonLines",
    "formatToJsonValue",
];
const STORE: &str = "json-format";
const STORE_LIMIT: usize = 2 * 1024 * 1024;
const RECEIPT_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Session {
    sid: String,
    command: String,
    token: u64,
    edit_token: u64,
    page: String,
    draft: String,
    seed: String,
    output: String,
    entries: Vec<String>,
    autopaste: bool,
    indent: String,
    max_lines: String,
    show_all: bool,
    #[serde(default)]
    receipt_done: String,
}
impl Session {
    fn new(sid: &str, command: &str, prefs: &Value, token: u64) -> Result<Self, String> {
        let autopaste = match prefs.get("autopaste") {
            None => false,
            Some(Value::Bool(v)) => *v,
            _ => return Err("autopaste must be boolean".into()),
        };
        let indent = match prefs.get("indent") {
            None => "tab",
            Some(Value::String(v)) if ["tab", "2", "4", "8"].contains(&v.as_str()) => v,
            _ => return Err("indent must be tab, 2, 4 or 8".into()),
        };
        let max = match prefs.get("maxInitialLines") {
            None => "10000",
            Some(Value::String(v)) if v.len() <= 128 => v,
            _ => return Err("maxInitialLines must be a string of at most 128 bytes".into()),
        };
        Ok(Self {
            sid: sid.into(),
            command: command.into(),
            token,
            edit_token: token,
            page: "form".into(),
            draft: String::new(),
            seed: String::new(),
            output: String::new(),
            entries: Vec::new(),
            autopaste,
            indent: indent.into(),
            max_lines: max.into(),
            show_all: false,
            receipt_done: String::new(),
        })
    }
    pub(crate) fn action(&self, kind: &str) -> String {
        let token = if kind == "change" {
            self.edit_token
        } else {
            self.token
        };
        format!("json-format:{}:{}:{kind}", self.sid, token)
    }
}
#[derive(Clone)]
struct Receipt {
    snapshot: Session,
    origin: String,
    method: &'static str,
    close: bool,
    phase: Phase,
    retry: String,
    error: String,
}
#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Sending,
    Delivered,
    Closing,
    Complete,
}
#[derive(Default)]
pub struct Adapter {
    sequence: AtomicU64,
    // A bounded, session-owned write-ahead checkpoint. Unlike staged SDK state,
    // it survives cancellation after an OS effect. Never persisted or replayed.
    receipts: Mutex<HashMap<String, Receipt>>,
    drafts: Mutex<HashMap<String, u64>>,
}
impl Adapter {
    fn next(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst) + 1
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, Receipt>>, String> {
        self.receipts
            .lock()
            .map_err(|_| "Checkpoint lock poisoned".into())
    }
    fn checkpoint(&self, r: Receipt) -> Result<(), String> {
        let mut all = self.lock()?;
        let size = |r: &Receipt| {
            r.snapshot.output.len()
                + r.snapshot.draft.len()
                + r.snapshot.seed.len()
                + r.snapshot.entries.iter().map(String::len).sum::<usize>()
                + r.error.len()
                + r.origin.len()
                + r.retry.len()
                + 512
        };
        let used: usize = all
            .iter()
            .filter(|(id, _)| **id != r.snapshot.sid)
            .map(|(_, r)| size(r))
            .sum();
        if used + size(&r) > RECEIPT_LIMIT
            || (!all.contains_key(&r.snapshot.sid) && all.len() >= 128)
        {
            return Err("Session checkpoint budget reached; close other Format JSON views".into());
        }
        all.insert(r.snapshot.sid.clone(), r);
        Ok(())
    }
    fn draft(&self, sid: &str, payload: &Value, copy: bool) -> Result<bool, String> {
        let revision = payload["formValues"]["__tezbarDraftRevision"]
            .as_u64()
            .filter(|n| *n <= 9_007_199_254_740_991)
            .ok_or("Renderer draft revision is required; apply integration prerequisite")?;
        let mut drafts = self.drafts.lock().map_err(|_| "Draft lock poisoned")?;
        if !drafts.contains_key(sid) && drafts.len() >= 128 {
            return Err("Draft session limit reached".into());
        }
        let current = drafts.entry(sid.to_owned()).or_default();
        if revision < *current || (!copy && revision == *current) {
            return Ok(false);
        }
        *current = revision;
        Ok(true)
    }
    async fn store(&self, ctx: &ExtensionContext, text: Option<String>) -> CoreResult {
        let file = ctx.paths.data_dir.join(STORE).join("storage.json");
        if let Ok(m) = std::fs::metadata(&file) {
            if m.len() > STORE_LIMIT as u64 {
                return Err("Private text store exceeds 2 MiB".into());
            }
        }
        let current = read_store(ctx, STORE).await?;
        validate_store(&current)?;
        self.sequence
            .fetch_max(current["revision"].as_u64().unwrap_or(0), Ordering::SeqCst);
        let Some(text) = text else { return Ok(current) };
        if text.len() > convert::MAX_INPUT {
            return Err("Input exceeds 1 MiB".into());
        }
        let revision = self.next();
        update_store(ctx, STORE, move |value| {
            // Validate inside the atomic transaction, not only before it.
            validate_store(value)?;
            if value["revision"].as_u64().unwrap_or(0) < revision {
                value["text"] = json!(text);
                value["revision"] = json!(revision);
            }
            if serde_json::to_vec(value).map_err(|e| e.to_string())?.len() > STORE_LIMIT {
                return Err("Private text store exceeds 2 MiB".into());
            }
            Ok(())
        })
        .await
    }
    fn render(&self, s: &Session, initial: bool) -> CoreResult {
        ui::render(s, initial)
    }
    fn recovery(&self, mut r: Receipt, state: &mut Value) -> CoreResult {
        let mut s = r.snapshot.clone();
        if r.phase == Phase::Complete {
            s.receipt_done = r.origin.clone();
            s.token = self.next();
            *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
            if matches!(s.command.as_str(), "formatClipboard" | "formatSelection") {
                return Ok(tezbar_extension_sdk::no_view(
                    if r.method == "host.clipboard.pasteText" {
                        "Pasted to foremost application"
                    } else {
                        "Copied to clipboard"
                    },
                ));
            }
            let mut v = self.render(&s, false)?;
            v["message"] = json!(if r.method == "host.clipboard.pasteText" {
                "Pasted to foremost application"
            } else {
                "Copied to clipboard"
            });
            return Ok(v);
        }
        if r.retry.is_empty() {
            r.retry = format!("{}:retry:{}", r.origin, self.next());
            self.checkpoint(r.clone())?;
        }
        s.page = "recovery".into();
        *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
        Ok(ui::recovery(
            &s,
            &r.retry,
            &r.error,
            r.method == "host.clipboard.pasteText",
            matches!(r.phase, Phase::Delivered | Phase::Closing),
        ))
    }
    async fn deliver(
        &self,
        ctx: &ExtensionContext,
        mut r: Receipt,
        state: &mut Value,
    ) -> CoreResult {
        r.retry.clear();
        r.error.clear();
        // Commit the checkpoint BEFORE the await. Cancellation can drop this
        // future after clipboard mutation but before SDK state is committed.
        self.checkpoint(r.clone())?;
        if r.phase == Phase::Sending {
            let result = ctx.host(r.method, json!({"text":r.snapshot.output})).await;
            match result.and_then(ack) {
                Ok(_) => {
                    r.phase = if r.close {
                        Phase::Delivered
                    } else {
                        Phase::Complete
                    };
                    self.checkpoint(r.clone())?;
                }
                Err(e) => {
                    r.error = format!("Delivery not confirmed: {e}");
                    self.checkpoint(r.clone())?;
                    return self.recovery(r, state);
                }
            }
        }
        if matches!(r.phase, Phase::Delivered | Phase::Closing) {
            r.phase = Phase::Closing;
            self.checkpoint(r.clone())?;
            match ctx
                .host("host.window.visibility", json!({"visible":false}))
                .await
                .and_then(ack)
            {
                Ok(_) => r.phase = Phase::Complete,
                Err(e) => {
                    r.error = format!(
                        "Clipboard copy was confirmed; closing the window was not confirmed: {e}"
                    );
                    self.checkpoint(r.clone())?;
                    return self.recovery(r, state);
                }
            }
        }
        self.checkpoint(r.clone())?;
        self.recovery(r, state)
    }
    async fn perform(
        &self,
        ctx: &ExtensionContext,
        s: &Session,
        origin: String,
        automatic: bool,
        state: &mut Value,
    ) -> CoreResult {
        let method = if s.autopaste && s.command != "formatToJsonValue" {
            "host.clipboard.pasteText"
        } else {
            "host.clipboard.write"
        };
        let r = Receipt {
            snapshot: s.clone(),
            origin,
            method,
            close: automatic && method == "host.clipboard.write",
            phase: Phase::Sending,
            retry: String::new(),
            error: String::new(),
        };
        self.deliver(ctx, r, state).await
    }
    async fn run(
        &self,
        ctx: &ExtensionContext,
        command: &str,
        operation: &str,
        payload: &Value,
        state: &mut Value,
    ) -> CoreResult {
        let sid = payload["sessionId"]
            .as_str()
            .filter(|v| !v.is_empty() && v.len() <= 256)
            .ok_or("Missing or oversized session ID")?;
        if operation == "extension:dispose-session" {
            self.lock()?.remove(sid);
            self.drafts
                .lock()
                .map_err(|_| "Draft lock poisoned")?
                .remove(sid);
            return Ok(json!({}));
        }
        if operation == "extension:run-command" {
            // Failed opens are removed by the SDK without a dispose callback.
            // Defensively fence accidental ID reuse; never read/convert/replay it.
            let orphan = self.lock()?.get(sid).cloned();
            if let Some(r) = orphan {
                if r.snapshot.command != command {
                    return Err("Checkpoint session ownership mismatch".into());
                }
                return self.recovery(r, state);
            }
            let mut s = Session::new(sid, command, &payload["preferences"], self.next())?;
            match command {
                "formatClipboard" | "formatSelection" | "formatSelectionAndShow" => {
                    let method = if command == "formatClipboard" {
                        "host.clipboard.read"
                    } else {
                        "host.selection.read"
                    };
                    let text = ctx.host(method, json!({})).await?;
                    let text = text.as_str().ok_or("Host read must return a text string")?;
                    s.output = convert::format(text, &s.indent)?;
                    s.page = "detail".into();
                    *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
                    if command != "formatSelectionAndShow" {
                        let id = s.action("launch");
                        return self.perform(ctx, &s, id.to_owned(), true, state).await;
                    }
                }
                "formatToJsonValue" => {
                    let store = self.store(ctx, None).await?;
                    s.draft = store["text"].as_str().unwrap_or("").to_owned();
                    s.seed = s.draft.clone();
                    s.output = convert::quote(&s.draft)?;
                }
                "index" | "formatJsonLines" => {}
                _ => return Err("Unknown command".into()),
            }
            *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
            return self.render(&s, true);
        }
        let existing = self.lock()?.get(sid).cloned();
        if let Some(r) = existing {
            if state["receipt_done"] != r.origin {
                if command == "formatToJsonValue"
                    && operation == "extension:invoke-action"
                    && payload["actionId"] == r.snapshot.action("change")
                {
                    if self.draft(sid, payload, false)? {
                        let text = payload["formValues"]["value"]
                            .as_str()
                            .ok_or("Expected text change value")?;
                        self.store(ctx, Some(text.to_owned())).await?;
                    }
                    return self.recovery(r, state);
                }
                if operation == "extension:invoke-action"
                    && !r.retry.is_empty()
                    && payload["actionId"] == r.retry
                {
                    return self.deliver(ctx, r, state).await;
                }
                // Even with discarded SDK state, refresh can recover the exact
                // prepared bytes. A stale action cannot trigger another effect.
                return self.recovery(r, state);
            }
        }
        let mut s: Session = serde_json::from_value(state.clone())
            .map_err(|_| "Session did not finish initialization; reopen command")?;
        if s.sid != sid || s.command != command {
            return Err("Session ownership mismatch".into());
        }
        match operation {
            "extension:refresh-session" => return Ok(tezbar_extension_sdk::unchanged()),
            "extension:search-text-changed" => return Ok(tezbar_extension_sdk::unchanged()),
            "extension:load-more" => {
                if s.page != "detail" || command != "formatSelectionAndShow" {
                    return Err("Load All Lines is unavailable".into());
                }
                s.show_all = true;
            }
            "extension:invoke-action" => {
                let id = payload["actionId"].as_str().ok_or("Missing actionId")?;
                if id == s.action("back")
                    && s.page == "detail"
                    && matches!(command, "index" | "formatJsonLines")
                {
                    s.page = "form".into();
                    s.seed = s.draft.clone();
                    s.output.clear();
                    s.entries.clear();
                    s.token = self.next();
                } else if id == s.action("load")
                    && s.page == "detail"
                    && command == "formatSelectionAndShow"
                {
                    s.show_all = true;
                } else if command == "formatToJsonValue" && id == s.action("change") {
                    if !self.draft(sid, payload, false)? {
                        // Action responses need a complete root: the current
                        // renderer only special-cases unchanged on refresh/search.
                        return self.render(&s, false);
                    }
                    let text = payload["formValues"]["value"]
                        .as_str()
                        .ok_or("Expected text change value")?;
                    let stored = self.store(ctx, Some(text.into())).await?;
                    s.draft = stored["text"].as_str().unwrap_or("").into();
                    s.output = convert::quote(&s.draft)?;
                    // Keep field token stable for already queued absolute drafts.
                } else if command == "formatToJsonValue" && id == s.action("copy") {
                    if !self.draft(sid, payload, true)? {
                        return Err("Stale text submission; use the latest draft".into());
                    }
                    let text = form_text(payload)?;
                    let stored = self.store(ctx, Some(text.into())).await?;
                    s.draft = stored["text"].as_str().unwrap_or("").into();
                    s.output = convert::quote(&s.draft)?;
                    return self.perform(ctx, &s, id.to_owned(), false, state).await;
                } else if s.page == "form"
                    && matches!(command, "index" | "formatJsonLines")
                    && (id == s.action("format") || id == s.action("view"))
                {
                    s.draft = form_text(payload)?.to_owned();
                    let output = if command == "index" {
                        convert::format(&s.draft, &s.indent)
                    } else {
                        convert::lines(&s.draft).map(|v| {
                            s.entries = v;
                            s.entries.join("\n")
                        })
                    };
                    match output {
                        Ok(v) => s.output = v,
                        Err(e) => {
                            *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
                            let mut v = self.render(&s, false)?;
                            v["message"] = json!(format!("Failed · {e}"));
                            return Ok(v);
                        }
                    }
                    if id == s.action("format") {
                        return self.perform(ctx, &s, id.to_owned(), true, state).await;
                    }
                    s.page = "detail".into();
                    s.token = self.next();
                } else if s.page == "detail" && id == s.action("deliver") {
                    return self.perform(ctx, &s, id.to_owned(), false, state).await;
                } else {
                    return Err("Stale or unavailable action; refresh this session".into());
                }
            }
            _ => return Err("Unsupported operation".into()),
        }
        *state = serde_json::to_value(&s).map_err(|e| e.to_string())?;
        self.render(&s, false)
    }
}
fn form_text(payload: &Value) -> Result<&str, String> {
    let s = payload["formValues"]["input"]
        .as_str()
        .ok_or("Submitted input field is required (renderer integration prerequisite)")?;
    if s.len() > convert::MAX_INPUT {
        return Err("Input exceeds 1 MiB".into());
    }
    Ok(s)
}
fn validate_store(v: &Value) -> Result<(), String> {
    if !v.is_object()
        || v.get("text")
            .is_some_and(|v| v.as_str().is_none_or(|t| t.len() > convert::MAX_INPUT))
        || v.get("revision")
            .is_some_and(|v| v.as_u64().is_none_or(|n| n > 9_007_199_254_740_991))
    {
        return Err("Private text store has malformed typed fields".into());
    }
    Ok(())
}
fn ack(v: Value) -> CoreResult {
    let Some(object) = v.as_object() else {
        return Err("Host did not return an acknowledgement object".into());
    };
    if v["ok"] != true
        || object.keys().any(|k| k != "ok" && k != "message")
        || object.get("message").is_some_and(|m| !m.is_string())
    {
        return Err("Host acknowledgement is failed, malformed or contradictory".into());
    }
    Ok(v)
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        ID
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn commands(&self) -> &'static [&'static str] {
        COMMANDS
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(self.run(ctx, command, operation, payload, state))
    }
    fn shutdown<'a>(&'a self, _ctx: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            self.lock()?.clear();
            self.drafts
                .lock()
                .map_err(|_| "Draft lock poisoned")?
                .clear();
            Ok(json!({}))
        })
    }
}
