use serde_json::Value;
use tezbar_json_format::convert;
#[test]
fn frozen_actual_utils_and_beautifier() {
    let fixture: Value = serde_json::from_str(include_str!("../fixtures/oracle.json")).unwrap();
    let mut differences = Vec::new();
    let mut supported = 0;
    let mut deliberate = 0;
    for v in fixture["vectors"].as_array().unwrap() {
        let src = v["input"].as_str().unwrap_or("");
        let expected = &v["actual"]["value"];
        let result: Result<Value, String> = match v["kind"].as_str().unwrap() {
            "format" => convert::format(src, v["indent"].as_str().unwrap()).map(Value::String),
            "lines" => convert::lines(src).map(|v| serde_json::to_value(v).unwrap()),
            "value" => {
                if let Some(units) = v["inputUtf16"].as_array() {
                    Ok(Value::String(convert::quote_utf16(
                        &units
                            .iter()
                            .map(|n| n.as_u64().unwrap() as u16)
                            .collect::<Vec<_>>(),
                    )))
                } else {
                    convert::quote(src).map(Value::String)
                }
            }
            _ => unreachable!(),
        };
        // These accepted JS expressions are intentionally outside a data-only port.
        if v["kind"] == "format" && ["1+2", "[...[]]", "{['a']:1}", "1_000"].contains(&src) {
            assert!(result.is_err(), "dynamic expression was accepted");
            deliberate += 1;
            continue;
        }
        let got = result.unwrap_or(Value::Null);
        if got != *expected {
            differences.push(format!(
                "{} {:?} indent {}: expected {} got {}",
                v["kind"], src, v["indent"], expected, got
            ));
        } else {
            supported += 1;
        }
    }
    assert!(
        differences.is_empty(),
        "{} mismatches:\n{}",
        differences.len(),
        differences.join("\n")
    );
    assert!(supported >= 247);
    assert_eq!(deliberate, 16);
}
#[test]
fn resource_bounds_and_data_only() {
    assert!(convert::format(&" ".repeat(convert::MAX_INPUT + 1), "2").is_err());
    assert!(convert::format(&format!("{}0{}", "[".repeat(66), "]".repeat(66)), "2").is_err());
    for src in [
        "{a:process.env}",
        "{get a(){return 1}}",
        "[...[]]",
        "new Date()",
        "1+2",
        "{a:`${secret}`}",
    ] {
        assert!(convert::format(src, "2").is_err(), "{src}");
    }
}
#[test]
fn explicit_output_and_node_limits_and_unicode_dont_panic() {
    let deep = format!(
        "{}{}{}",
        "[".repeat(63),
        vec!["0"; 20000].join(",\n"),
        "]".repeat(63)
    );
    assert!(matches!(convert::format(&deep,"8"),Err(e)if e.contains("4 MiB")));
    let nodes = format!("[{}]", vec!["0"; 100001].join(","));
    assert!(convert::format(&nodes, "2").is_err());
    let holes = format!("[{}]", ",".repeat(100001));
    assert!(convert::format(&holes, "2").is_err());
    let alphabet = [
        '{', '}', '[', ']', '\'', '"', '\\', 'é', '😀', '\u{2028}', '\0', '0', ',', ':', 'x', '-',
    ];
    let mut seed = 42_u64;
    for _ in 0..1000 {
        let mut s = String::new();
        for _ in 0..32 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            s.push(alphabet[(seed >> 32) as usize % alphabet.len()]);
        }
        let _ = convert::format(&s, "2");
    }
}
