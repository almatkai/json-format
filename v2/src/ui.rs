use crate::Session;
use serde_json::{json, Value};
use tezbar_extension_sdk::{view, CoreResult};
fn action(s: &Session, kind: &str, title: &str, submit: bool) -> Value {
    json!({"id":s.action(kind),"title":title,"kind":if submit{"submit-form"}else{"action"}})
}
fn static_view(title: &str, root: Value, actions: Vec<Value>) -> Value {
    let mut v = view(title, root, actions);
    v["refreshMs"] = Value::Null;
    v
}
// Indented code is literal in the committed Markdown renderer: fences, links,
// images and HTML from input cannot escape it. Empty lines stay in the block.
pub fn code(text: &str) -> String {
    text.split('\n')
        .map(|l| format!("    {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
fn max_lines(s: &str) -> f64 {
    let s = s.trim_start_matches(crate::convert::js_whitespace);
    let (sign, s) = if let Some(s) = s.strip_prefix('-') {
        (-1.0, s)
    } else {
        (1.0, s.strip_prefix('+').unwrap_or(s))
    };
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    let n = digits.parse::<f64>().unwrap_or(f64::NAN) * sign;
    if n.is_nan() || n == 0.0 {
        10000.0
    } else {
        n
    }
}
pub fn preview(text: &str, max: &str, show_all: bool) -> (String, bool, f64, usize) {
    if text.is_empty() {
        return (String::new(), false, 0.0, 0);
    }
    let lines: Vec<_> = text.split('\n').collect();
    let count = lines.len();
    let limit = max_lines(max);
    if show_all || count as f64 <= limit {
        return (text.into(), false, count as f64, count);
    }
    let end = if limit < 0.0 {
        (count as f64 + limit).max(0.0)
    } else {
        limit.min(count as f64)
    } as usize;
    (lines[..end].join("\n"), true, limit, count)
}
pub(crate) fn render(s: &Session, initial: bool) -> CoreResult {
    if s.page == "form" {
        let persistent = s.command == "formatToJsonValue";
        let mut field = json!({"type":"Form.TextArea","props":{"id":"input","title":"Input","placeholder":if persistent{"Enter text to format..."}else if s.command=="formatJsonLines"{"Paste JSON Array here…"}else{"Paste JSON here…"}}});
        // After the first persistent render omit value; FormRuntime then keeps
        // queued drafts despite stale responses, even with actionId present.
        if !persistent || initial {
            field["props"]["value"] = json!(s.seed);
        }
        if persistent {
            field["props"]["actionId"] = json!(s.action("change"));
        }
        let mut children = vec![field];
        let actions = if persistent {
            children.push(
                json!({"type":"Form.Description","props":{"title":"Formatted","text":s.output}}),
            );
            vec![action(s, "copy", "Copy", true)]
        } else {
            vec![
                action(s, "format", "Format", true),
                action(s, "view", "View Result", true),
            ]
        };
        return Ok(static_view(
            if persistent {
                "Format Text into Valid JSON Value"
            } else if s.command == "formatJsonLines" {
                "Format Array of JSON to JSONLines"
            } else {
                "Format JSON"
            },
            json!({"type":"Form","children":children}),
            actions,
        ));
    }
    if s.page != "detail" {
        return Err("Invalid view state".into());
    }
    let selected = s.command == "formatSelectionAndShow";
    let mut markdown = String::new();
    let mut truncated = false;
    let label = if s.autopaste {
        if selected {
            "Paste Full JSON"
        } else {
            "Paste"
        }
    } else if selected {
        "Copy Full JSON"
    } else {
        "Copy to Clipboard"
    };
    if selected {
        let (display, trunc, n, total) = preview(&s.output, &s.max_lines, s.show_all);
        truncated = trunc;
        markdown = code(&display);
        if trunc {
            let n = crate::convert::js_number(n);
            markdown.push_str(&format!(
                "\n\n---\n⚠️ **Showing {n} of {total} lines.** Load All Lines to display the rest."
            ));
        }
    } else if s.command == "formatJsonLines" {
        for (i, entry) in s.entries.iter().enumerate() {
            if i > 0 {
                markdown.push_str("\n\n");
            }
            markdown.push_str(&code(entry));
        }
    } else {
        markdown = code(&s.output);
    }
    let mut root = json!({"type":"Detail","props":{"markdown":markdown}});
    if matches!(s.command.as_str(), "index" | "formatJsonLines") {
        root["props"]["backActionId"] = json!(s.action("back"));
    }
    let mut actions = vec![action(s, "deliver", label, false)];
    if truncated {
        let mut load = action(s, "load", "Load All Lines", false);
        load["shortcut"] = json!({"modifiers":["cmd"],"key":"l"});
        actions.push(load);
    }
    Ok(static_view(
        if s.command == "formatJsonLines" {
            "Formatted JSON Lines"
        } else {
            "Formatted JSON"
        },
        root,
        actions,
    ))
}
pub(crate) fn recovery(
    _s: &Session,
    retry: &str,
    error: &str,
    paste: bool,
    only_close: bool,
) -> Value {
    let message = if error.is_empty() {
        "The operation was interrupted. Its result was not confirmed."
    } else {
        error
    };
    let warning = if only_close {
        "Clipboard copy was confirmed. Retry closes the window without copying again."
    } else if paste {
        "The previous paste may already have happened. Retrying can paste the same text twice. Check the target application first."
    } else {
        "The clipboard may already contain the formatted text. Retry writes the exact prepared bytes without reading the source again."
    };
    static_view(
        "Delivery not confirmed",
        json!({"type":"Detail","props":{"markdown":format!("{}\n\n{warning}",code(message))}}),
        vec![
            json!({"id":retry,"title":if only_close{"Retry Close"}else if paste{"Retry Paste (may duplicate)"}else{"Retry Copy"},"kind":"action"}),
        ],
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_and_parseint() {
        assert_eq!(
            preview("1\n2\n3\n4", "-1", false),
            ("1\n2\n3".into(), true, -1.0, 4)
        );
        assert_eq!(preview("1\n2\n3\n4", "  +2suffix", false).0, "1\n2");
        assert!(!preview("a", "0", false).1);
        assert_eq!(preview("1\n2", "-99", false).0, "");
    }
}
