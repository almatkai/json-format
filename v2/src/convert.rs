//! Finite data-literal compatibility, not a JavaScript evaluator.
//! Strict JSON retains lexical tokens for beautification; fallback literals use
//! ECMAScript f64/stringification and own-property ordering. See COMPATIBILITY.md.
use std::fmt::Write;

pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_OUTPUT: usize = 4 * 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 100_000;
const MAX_WORK: usize = 8 * MAX_INPUT;

#[derive(Clone, Debug)]
enum Data {
    Null,
    Bool(bool),
    Number(f64),
    String(Vec<u16>),
    Array(Vec<Node>),
    Object(Vec<(Node, Node)>),
    Undefined,
}
#[derive(Clone, Debug)]
struct Node {
    data: Data,
    token: String,
    newlines: usize,
    closing_newlines: usize,
}
impl Node {
    fn simple(data: Data) -> Self {
        Self {
            data,
            token: String::new(),
            newlines: 0,
            closing_newlines: 0,
        }
    }
}
struct Parser<'a> {
    source: &'a str,
    pos: usize,
    literal: bool,
    nodes: usize,
    work: usize,
}
impl<'a> Parser<'a> {
    fn parse(source: &'a str, literal: bool) -> Result<Node, String> {
        if source.len() > MAX_INPUT {
            return Err("Input exceeds 1 MiB".into());
        }
        let mut p = Self {
            source,
            pos: 0,
            literal,
            nodes: 0,
            work: 0,
        };
        let node = p.value(0)?;
        p.space()?;
        if p.pos != source.len() {
            return Err("Unsupported expression or invalid data literal".into());
        }
        Ok(node)
    }
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }
    fn step(&mut self) -> Result<u8, String> {
        self.work += 1;
        if self.work > MAX_WORK {
            return Err("Conversion work limit exceeded".into());
        }
        let c = self.peek().ok_or("Unexpected end of input")?;
        self.pos += 1;
        Ok(c)
    }
    fn space(&mut self) -> Result<usize, String> {
        let mut n = 0;
        while let Some(c) = self.source[self.pos..].chars().next() {
            let ws = if self.literal {
                js_whitespace(c)
            } else {
                matches!(c, ' ' | '\t' | '\n' | '\r')
            };
            if !ws {
                break;
            }
            if c == '\n' {
                n += 1;
            }
            self.pos += c.len_utf8();
            self.work += c.len_utf8();
            if self.work > MAX_WORK {
                return Err("Conversion work limit exceeded".into());
            }
        }
        Ok(n)
    }
    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.step()? != c {
            return Err("Invalid data literal".into());
        }
        Ok(())
    }
    fn value(&mut self, depth: usize) -> Result<Node, String> {
        if depth > MAX_DEPTH {
            return Err("Nesting exceeds 64 levels".into());
        }
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("Node limit exceeded".into());
        }
        let newlines = self.space()?;
        let start = self.pos;
        let mut closing_newlines = 0;
        let data = match self.peek().ok_or("Empty or invalid JSON/JS literal")? {
            b'"' => Data::String(self.string()?),
            b'\'' | b'`' if self.literal => Data::String(self.string()?),
            b'[' => {
                self.step()?;
                let mut entries = Vec::new();
                let saved = self.pos;
                closing_newlines = self.space()?;
                if self.peek() != Some(b']') {
                    self.pos = saved;
                    loop {
                        let saved = self.pos;
                        let nl = self.space()?;
                        if self.literal && self.peek() == Some(b']') {
                            closing_newlines = nl;
                            break;
                        }
                        if self.literal && self.peek() == Some(b',') {
                            self.nodes += 1;
                            if self.nodes > MAX_NODES {
                                return Err("Node limit exceeded".into());
                            }
                            let mut hole = Node::simple(Data::Undefined);
                            hole.newlines = nl;
                            entries.push(hole);
                            self.step()?;
                            continue;
                        }
                        self.pos = saved;
                        entries.push(self.value(depth + 1)?);
                        closing_newlines = self.space()?;
                        if self.peek() == Some(b']') {
                            break;
                        }
                        self.expect(b',')?;
                        let saved = self.pos;
                        self.space()?;
                        if self.literal && self.peek() == Some(b']') {
                            break;
                        }
                        self.pos = saved;
                    }
                }
                self.expect(b']')?;
                Data::Array(entries)
            }
            b'{' => {
                self.step()?;
                let mut entries = Vec::new();
                let mut proto = false;
                let saved = self.pos;
                closing_newlines = self.space()?;
                if self.peek() != Some(b'}') {
                    self.pos = saved;
                    loop {
                        self.nodes += 1;
                        if self.nodes > MAX_NODES {
                            return Err("Node limit exceeded".into());
                        }
                        let nl = self.space()?;
                        let key_start = self.pos;
                        let key = match self.peek() {
                            Some(b'"') => self.string()?,
                            Some(b'\'') if self.literal => self.string()?,
                            Some(c)
                                if self.literal
                                    && (c.is_ascii_alphabetic() || b"_$".contains(&c)) =>
                            {
                                while self.peek().is_some_and(|b| {
                                    b.is_ascii_alphanumeric() || b"_$".contains(&b)
                                }) {
                                    self.step()?;
                                }
                                self.source[key_start..self.pos].encode_utf16().collect()
                            }
                            Some(c) if self.literal && c.is_ascii_digit() => {
                                let number = self.number()?;
                                if number == f64::INFINITY {
                                    "Infinity".encode_utf16().collect()
                                } else {
                                    js_number(number).encode_utf16().collect()
                                }
                            }
                            _ => {
                                return Err(
                                    "Expected quoted JSON key or supported literal key".into()
                                )
                            }
                        };
                        let mut k = Node::simple(Data::String(key.clone()));
                        k.newlines = nl;
                        k.token = self.source[key_start..self.pos].to_owned();
                        self.space()?;
                        self.expect(b':')?;
                        let value = self.value(depth + 1)?;
                        if self.literal && key == "__proto__".encode_utf16().collect::<Vec<_>>() {
                            if proto {
                                return Err("Duplicate literal __proto__ setter".into());
                            }
                            proto = true;
                        } else {
                            entries.push((k, value));
                        }
                        closing_newlines = self.space()?;
                        if self.peek() == Some(b'}') {
                            break;
                        }
                        self.expect(b',')?;
                        let saved = self.pos;
                        self.space()?;
                        if self.literal && self.peek() == Some(b'}') {
                            break;
                        }
                        self.pos = saved;
                    }
                }
                self.expect(b'}')?;
                Data::Object(entries)
            }
            b'-' | b'0'..=b'9' => Data::Number(self.number()?),
            b'+' | b'.' if self.literal => Data::Number(self.number()?),
            _ => {
                while self
                    .peek()
                    .is_some_and(|b| b.is_ascii_alphanumeric() || b"_$".contains(&b))
                {
                    self.step()?;
                }
                match &self.source[start..self.pos] {
                    "null" => Data::Null,
                    "true" => Data::Bool(true),
                    "false" => Data::Bool(false),
                    "undefined" if self.literal => Data::Undefined,
                    "NaN" if self.literal => Data::Number(f64::NAN),
                    "Infinity" if self.literal => Data::Number(f64::INFINITY),
                    _ => {
                        return Err(
                            "Unsupported executable/dynamic expression or invalid literal".into(),
                        )
                    }
                }
            }
        };
        Ok(Node {
            data,
            token: self.source[start..self.pos].to_owned(),
            newlines,
            closing_newlines,
        })
    }
    fn string(&mut self) -> Result<Vec<u16>, String> {
        let quote = self.step()?;
        let mut out = Vec::new();
        loop {
            let c = self.step()?;
            if c == quote {
                return Ok(out);
            }
            if c == b'\\' {
                let e = self.step()?;
                match e {
                    b'"' | b'\\' | b'/' => out.push(e as u16),
                    b'\'' if self.literal => out.push(e as u16),
                    b'b' => out.push(8),
                    b'f' => out.push(12),
                    b'n' => out.push(10),
                    b'r' => out.push(13),
                    b't' => out.push(9),
                    b'u' => {
                        if self.literal && self.peek() == Some(b'{') {
                            self.step()?;
                            let start = self.pos;
                            while self.peek().is_some_and(|b| b.is_ascii_hexdigit()) {
                                self.step()?;
                            }
                            if self.pos - start > 6 || self.pos == start {
                                return Err("Invalid Unicode escape".into());
                            }
                            let val = u32::from_str_radix(&self.source[start..self.pos], 16)
                                .map_err(|_| "Invalid Unicode escape")?;
                            self.expect(b'}')?;
                            if val <= 0xffff {
                                out.push(val as u16);
                            } else if val <= 0x10ffff {
                                let v = val - 0x10000;
                                out.push(0xd800 + (v >> 10) as u16);
                                out.push(0xdc00 + (v & 1023) as u16);
                            } else {
                                return Err("Invalid Unicode escape".into());
                            }
                        } else {
                            out.push(self.hex(4)?);
                        }
                    }
                    b'x' if self.literal => out.push(self.hex(2)?),
                    b'v' if self.literal => out.push(11),
                    b'0' if self.literal => {
                        if self.peek().is_some_and(|b| b.is_ascii_digit()) {
                            return Err("Legacy octal escape unsupported".into());
                        }
                        out.push(0);
                    }
                    b'\n' if self.literal => {}
                    b'\r' if self.literal => {
                        if self.peek() == Some(b'\n') {
                            self.step()?;
                        }
                    }
                    _ if self.literal && !e.is_ascii_digit() => {
                        if e.is_ascii() {
                            out.push(e as u16);
                        } else {
                            self.pos -= 1;
                            let ch = self.source[self.pos..]
                                .chars()
                                .next()
                                .ok_or("Invalid UTF-8")?;
                            self.pos += ch.len_utf8();
                            if !matches!(ch, '\u{2028}' | '\u{2029}') {
                                let mut units = [0; 2];
                                out.extend_from_slice(ch.encode_utf16(&mut units));
                            }
                        }
                    }
                    _ => return Err("Invalid string escape".into()),
                }
            } else {
                if c < 32 && !(self.literal && quote == b'`') {
                    return Err("Unescaped string control character".into());
                }
                if self.literal && quote == b'`' && c == b'$' && self.peek() == Some(b'{') {
                    return Err("Template interpolation unsupported".into());
                }
                if self.literal && quote == b'`' && c == b'\r' {
                    if self.peek() == Some(b'\n') {
                        self.step()?;
                    }
                    out.push(10);
                } else if c.is_ascii() {
                    out.push(c as u16);
                } else {
                    self.pos -= 1;
                    let ch = self.source[self.pos..]
                        .chars()
                        .next()
                        .ok_or("Invalid UTF-8")?;
                    self.pos += ch.len_utf8();
                    let mut units = [0; 2];
                    out.extend_from_slice(ch.encode_utf16(&mut units));
                }
            }
        }
    }
    fn hex(&mut self, n: usize) -> Result<u16, String> {
        let start = self.pos;
        for _ in 0..n {
            if !self.step()?.is_ascii_hexdigit() {
                return Err("Invalid hex escape".into());
            }
        }
        u16::from_str_radix(&self.source[start..self.pos], 16)
            .map_err(|_| "Invalid hex escape".into())
    }
    fn number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        let negative = self.peek() == Some(b'-');
        if negative || (self.literal && self.peek() == Some(b'+')) {
            self.step()?;
        }
        if self.literal && self.source[self.pos..].starts_with("Infinity") {
            self.pos += 8;
            return Ok(if negative {
                -f64::INFINITY
            } else {
                f64::INFINITY
            });
        }
        if self.literal && self.peek() == Some(b'0') {
            let base = match self.source.as_bytes().get(self.pos + 1) {
                Some(b'x' | b'X') => 16,
                Some(b'b' | b'B') => 2,
                Some(b'o' | b'O') => 8,
                _ => 0,
            };
            if base != 0 {
                self.pos += 2;
                let st = self.pos;
                while self.peek().is_some_and(|b| (b as char).is_digit(base)) {
                    self.step()?;
                }
                // Exact integer then one f64 rounding. Longer radix literals are
                // deliberately rejected instead of accumulating rounded partials.
                let n = u128::from_str_radix(&self.source[st..self.pos], base)
                    .map_err(|_| "Radix literal exceeds 128-bit compatibility bound")?;
                return Ok(if negative { -(n as f64) } else { n as f64 });
            }
        }
        let int_start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.step()?;
        }
        let int_len = self.pos - int_start;
        if !self.literal
            && (int_len == 0 || (int_len > 1 && self.source.as_bytes()[int_start] == b'0'))
        {
            return Err("Invalid JSON number".into());
        }
        if self.literal && int_len > 1 && self.source.as_bytes()[int_start] == b'0' {
            return Err("Legacy octal/leading zero literals unsupported".into());
        }
        let mut fraction = 0;
        if self.peek() == Some(b'.') {
            self.step()?;
            let st = self.pos;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.step()?;
            }
            fraction = self.pos - st;
            if !self.literal && fraction == 0 {
                return Err("Invalid fraction".into());
            }
        }
        if int_len + fraction == 0 {
            return Err("Invalid number".into());
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.step()?;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.step()?;
            }
            let st = self.pos;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.step()?;
            }
            if st == self.pos {
                return Err("Invalid exponent".into());
            }
        }
        self.source[start..self.pos]
            .parse::<f64>()
            .map_err(|_| "Invalid number".into())
    }
}
pub(crate) fn js_whitespace(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
pub(crate) fn js_number(n: f64) -> String {
    if !n.is_finite() {
        "null".into()
    } else if n == 0.0 {
        "0".into()
    } else {
        ryu_js::Buffer::new().format(n).to_owned()
    }
}
pub fn quote_utf16(units: &[u16]) -> String {
    let mut out = String::from("\"");
    for ch in char::decode_utf16(units.iter().copied()) {
        match ch {
            Ok('"') => out.push_str("\\\""),
            Ok('\\') => out.push_str("\\\\"),
            Ok('\u{8}') => out.push_str("\\b"),
            Ok('\u{c}') => out.push_str("\\f"),
            Ok('\n') => out.push_str("\\n"),
            Ok('\r') => out.push_str("\\r"),
            Ok('\t') => out.push_str("\\t"),
            Ok(c) if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            Ok(c) => out.push(c),
            Err(e) => {
                let _ = write!(out, "\\u{:04x}", e.unpaired_surrogate());
            }
        }
    }
    out.push('"');
    out
}
pub fn quote(text: &str) -> Result<String, String> {
    if text.len() > MAX_INPUT {
        return Err("Input exceeds 1 MiB".into());
    }
    bounded(quote_utf16(&text.encode_utf16().collect::<Vec<_>>()))
}
fn bounded(s: String) -> Result<String, String> {
    if s.len() > MAX_OUTPUT {
        Err("Output exceeds 4 MiB".into())
    } else {
        Ok(s)
    }
}
fn index_key(key: &[u16]) -> Option<u32> {
    let s = String::from_utf16(key).ok()?;
    let n = s.parse::<u32>().ok()?;
    if n != u32::MAX && n.to_string() == s {
        Some(n)
    } else {
        None
    }
}
fn stringify(node: &Node, out: &mut String) -> Result<(), String> {
    if out.len() > MAX_OUTPUT {
        return Err("Output exceeds 4 MiB".into());
    }
    match &node.data {
        Data::Null | Data::Undefined => out.push_str("null"),
        Data::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        Data::Number(v) => out.push_str(&js_number(*v)),
        Data::String(v) => out.push_str(&quote_utf16(v)),
        Data::Array(v) => {
            out.push('[');
            for (i, n) in v.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                stringify(n, out)?;
            }
            out.push(']');
        }
        Data::Object(v) => {
            let mut unique: Vec<(&[u16], &Node)> = Vec::new();
            // Bounded O(n log n) duplicate resolution, preserving first insertion.
            let mut positions = std::collections::BTreeMap::new();
            for (k, n) in v {
                let Data::String(key) = &k.data else {
                    unreachable!()
                };
                if let Some(&idx) = positions.get(key) {
                    unique[idx] = (key.as_slice(), n);
                } else {
                    positions.insert(key.clone(), unique.len());
                    unique.push((key, n));
                }
            }
            unique.sort_by_key(|(k, _)| index_key(k).map_or((1, 0), |n| (0, n)));
            out.push('{');
            let mut comma = false;
            for (k, n) in unique {
                if matches!(n.data, Data::Undefined) {
                    continue;
                }
                if comma {
                    out.push(',');
                }
                comma = true;
                out.push_str(&quote_utf16(k));
                out.push(':');
                stringify(n, out)?;
            }
            out.push('}');
        }
    }
    if out.len() > MAX_OUTPUT {
        return Err("Output exceeds 4 MiB".into());
    }
    Ok(())
}
fn unescape_heuristic(s: &str) -> Result<String, String> {
    // JS /^[^"]+\\".*$/ is intentionally loose; '.' excludes line terminators.
    let Some(quote) = s.find('"') else {
        return Ok(s.to_owned());
    };
    if quote < 2
        || s.as_bytes()[quote - 1] != b'\\'
        || s[quote + 1..].contains(['\n', '\r', '\u{2028}', '\u{2029}'])
    {
        return Ok(s.to_owned());
    }
    let mut out = Vec::new();
    let mut p = Parser {
        source: s,
        pos: 0,
        literal: false,
        nodes: 0,
        work: 0,
    };
    while p.pos < s.len() {
        if p.peek() == Some(b'\\') {
            let save = p.pos;
            p.pos += 1;
            let val = match p.peek() {
                Some(b'"') => Some(34),
                Some(b'\\') => Some(92),
                Some(b'/') => Some(47),
                Some(b'b') => Some(8),
                Some(b'f') => Some(12),
                Some(b'n') => Some(10),
                Some(b'r') => Some(13),
                Some(b't') => Some(9),
                Some(b'u') => {
                    p.pos += 1;
                    let r = p.hex(4).ok();
                    if let Some(v) = r {
                        out.push(v);
                        continue;
                    }
                    p.pos = save;
                    None
                }
                _ => None,
            };
            if let Some(v) = val {
                p.pos += 1;
                out.push(v);
                continue;
            }
            p.pos = save;
        }
        let ch = s[p.pos..].chars().next().ok_or("Invalid text")?;
        p.pos += ch.len_utf8();
        let mut u = [0; 2];
        out.extend_from_slice(ch.encode_utf16(&mut u));
    }
    String::from_utf16(&out).map_err(|_| {
        "Unescape recovery produced an unpaired surrogate; use escaped JSON instead".into()
    })
}
fn executable_regex(s: &str) -> bool {
    // Match the source's loose pair-of-parentheses or identifier-tag template.
    if let Some(open) = s.find('(') {
        if s[open + 1..].contains(')') {
            return true;
        }
    }
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_alphanumeric() || *c == '_' || *c == '$' {
            let mut j = i + 1;
            while j < chars.len() && js_whitespace(chars[j]) {
                j += 1;
            }
            if chars.get(j) == Some(&'`') && chars[j + 1..].contains(&'`') {
                return true;
            }
        }
    }
    false
}
pub fn format(text: &str, indent: &str) -> Result<String, String> {
    if text.len() > MAX_INPUT {
        return Err("Input exceeds 1 MiB".into());
    }
    let trimmed = text.trim_matches(js_whitespace);
    if trimmed.is_empty() {
        return Err("Please copy a valid JSON/JS Object".into());
    }
    let inner;
    let src = if trimmed.starts_with('"') {
        let node = Parser::parse(trimmed, false).map_err(|_| "Please enter a valid JSON string")?;
        let Data::String(units) = node.data else {
            return Err("Please enter a valid JSON string".into());
        };
        inner = String::from_utf16(&units)
            .map_err(|_| "Stringified payload contains an unpaired surrogate")?;
        inner.as_str()
    } else {
        trimmed
    };
    let processed = unescape_heuristic(src)?;
    let strict = Parser::parse(&processed, false);
    let node = if let Ok(node) = strict {
        node
    } else {
        let literal = processed.strip_suffix(';').unwrap_or(&processed);
        if executable_regex(literal) {
            return Err("Please copy a valid JSON/JS Object (source executable guard)".into());
        }
        let parsed = Parser::parse(literal, true)?;
        if matches!(parsed.data, Data::Undefined) {
            return Err("Literal has no JSON representation".into());
        }
        let mut compact = String::new();
        stringify(&parsed, &mut compact)?;
        Parser::parse(&compact, false)?
    };
    let unit = match indent {
        "tab" => "\t",
        "2" => "  ",
        "4" => "    ",
        "8" => "        ",
        _ => return Err("Unsupported indentation preference".into()),
    };
    let mut out = String::new();
    pretty(&node, unit, 0, &mut out)?;
    bounded(out)
}
pub fn lines(text: &str) -> Result<Vec<String>, String> {
    if !text.starts_with('[') || !text.ends_with(']') {
        return Err("Please enter a valid JSON/JS Array Object".into());
    }
    let node = Parser::parse(text, false)?;
    let Data::Array(entries) = node.data else {
        return Err("Please enter a valid JSON/JS Array Object".into());
    };
    // Actual .map(JSON.stringify): numeric index is ignored as replacer, the
    // original Array is ignored as space (only String/Number wrappers qualify).
    let mut size = 0;
    let mut result = Vec::new();
    for n in entries {
        let mut out = String::new();
        stringify(&n, &mut out)?;
        size += out.len() + 1;
        if size > MAX_OUTPUT {
            return Err("Output exceeds 4 MiB".into());
        }
        result.push(out);
    }
    Ok(result)
}
fn newline(out: &mut String, n: usize, unit: &str, depth: usize) {
    out.push_str(&"\n".repeat(n.max(1)));
    out.push_str(&unit.repeat(depth));
}
fn pretty(node: &Node, unit: &str, depth: usize, out: &mut String) -> Result<(), String> {
    if out.len() > MAX_OUTPUT {
        return Err("Output exceeds 4 MiB".into());
    }
    match &node.data {
        Data::Object(entries) => {
            out.push('{');
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, k.newlines, unit, depth + 1);
                out.push_str(&k.token);
                out.push_str(": ");
                pretty(v, unit, depth + 1, out)?;
            }
            if !entries.is_empty() {
                newline(out, node.closing_newlines, unit, depth);
            }
            out.push('}');
        }
        Data::Array(entries) => {
            out.push('[');
            let forced = |i: usize, v: &Node| {
                matches!(v.data, Data::Array(_))
                    && (i == 0 || matches!(entries[i - 1].data, Data::Array(_) | Data::Object(_)))
            };
            let multiline = entries.iter().enumerate().any(|(i, v)| {
                forced(i, v) || (v.newlines > 0 && !(i == 0 && matches!(v.data, Data::Object(_))))
            });
            for (i, v) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let nl = forced(i, v)
                    || (v.newlines > 0 && !(i == 0 && matches!(v.data, Data::Object(_))));
                if nl {
                    newline(out, v.newlines, unit, depth + 1);
                } else if i > 0 {
                    out.push(' ');
                }
                pretty(v, unit, if multiline { depth + 1 } else { depth }, out)?;
            }
            if !entries.is_empty() && multiline {
                newline(out, node.closing_newlines, unit, depth);
            }
            out.push(']');
        }
        _ => out.push_str(&node.token),
    }
    if out.len() > MAX_OUTPUT {
        return Err("Output exceeds 4 MiB".into());
    }
    Ok(())
}
