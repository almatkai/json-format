import assert from "node:assert/strict";
import test, { describe, it } from "node:test";
import {
  unescapeJsonString,
  isJson,
  isArray,
  isExecuteable,
  tryConvertToJsonString,
  formatWithBeautify,
  formatJSLogic,
  formatToJSONLinesPure,
  formatJSONValue,
} from "../src/lib/json";

describe("unescapeJsonString", () => {
  it("leaves normal JSON untouched", () => {
    assert.equal(unescapeJsonString('{"a":1}'), '{"a":1}');
  });
  it("unescapes when key quote is escaped", () => {
    // payload like {\"a\":1} -> should unescape
    const escaped = '{\\"a\\":1}';
    // /^[^"]+\\".*$/.test should be true for string starting with { then escaped quote
    const out = unescapeJsonString(escaped);
    assert.equal(out, '{"a":1}');
  });
  it("handles unicode escape", () => {
    const escaped = '{\\"a\\": "\\u0041"}';
    const out = unescapeJsonString(escaped);
    assert.equal(out, '{"a": "A"}');
  });
});

describe("isJson / isArray", () => {
  it("detects valid and invalid JSON", () => {
    assert.equal(isJson('{"a":1}'), true);
    assert.equal(isJson("{a:1}"), false);
    assert.equal(isJson("[]"), true);
    assert.equal(isJson("not json"), false);
  });
  it("detects array strings", () => {
    assert.equal(isArray("[1,2]"), true);
    assert.equal(isArray('{"a":1}'), false);
    assert.equal(isArray("[]"), true);
  });
});

describe("isExecuteable", () => {
  it("flags IIFE and tagged templates", () => {
    assert.equal(isExecuteable("(function(){})()"), true);
    assert.equal(isExecuteable("tag`template`"), true);
    assert.equal(isExecuteable('{"a":1}'), false);
    assert.equal(isExecuteable("[1,2,3]"), false);
  });
});

describe("tryConvertToJsonString", () => {
  it("returns already-valid JSON", () => {
    assert.equal(tryConvertToJsonString('{"x":10}'), '{"x":10}');
  });
  it("converts JS object literal with single quotes", () => {
    const out = tryConvertToJsonString("{a:1, b: 'hi'}");
    assert.ok(out);
    const parsed = JSON.parse(out!);
    assert.equal(parsed.a, 1);
    assert.equal(parsed.b, "hi");
  });
  it("strips trailing semicolon", () => {
    const out = tryConvertToJsonString('{"a":1};');
    assert.equal(out, '{"a":1}');
  });
  it("rejects executable payloads", () => {
    assert.equal(tryConvertToJsonString("(function(){})()"), undefined);
  });
});

describe("formatWithBeautify", () => {
  it("beautifies with tab and spaces", () => {
    const src = '{"a":1,"b":[2,3]}';
    const tab = formatWithBeautify(src, "tab");
    assert.match(tab, /\t/);
    const two = formatWithBeautify(src, "2");
    assert.match(two, /  "a"/);
    const four = formatWithBeautify(src, "4");
    assert.match(four, /    "a"/);
  });
});

describe("formatJSLogic", () => {
  it("formats plain JSON", async () => {
    const out = await formatJSLogic('{"a":1}', "2");
    assert.ok(out);
    assert.match(out!, /"a": 1/);
  });
  it("handles stringified JSON", async () => {
    // Input is a JSON string that itself contains stringified JSON, e.g. '"{\\"a\\":1}"'
    const inner = JSON.stringify('{"a":1}'); // '"{\\"a\\":1}"' ? Actually JSON.stringify('{"a":1}') wraps
    // Simpler: pass '"{\\"a\\":1}"' as stringified payload? formatJS handles firstChar === '"'
    const payload = '"{\\"a\\":1}"'; // trimmed first char is "
    // convert will parse then convert inner
    const out = await formatJSLogic(payload, "2");
    // formatJSLogic should parse the outer JSON string then convert inner
    // outer parse yields '{"a":1}' -> tryConvert returns same -> beautified
    assert.ok(out);
    assert.match(out!, /"a": 1/);
  });
  it("returns undefined for invalid", async () => {
    const out = await formatJSLogic("not json at all {{{", "2");
    // tryConvert will fail, but may return undefined due to executable? Let's see, it will try Function and fail => undefined
    assert.equal(out, undefined);
  });
});

describe("formatToJSONLinesPure", () => {
  it("splits array to JSONLines", () => {
    const out = formatToJSONLinesPure('[1, {"a":2}, "hi"]');
    assert.deepEqual(out, ['1', '{"a":2}', '"hi"']);
  });
  it("returns undefined for non-array or invalid", () => {
    assert.equal(formatToJSONLinesPure('{"a":1}'), undefined);
    assert.equal(formatToJSONLinesPure("not json"), undefined);
  });
});

describe("formatJSONValue", () => {
  it("stringifies text as JSON value", () => {
    assert.equal(formatJSONValue("hello"), '"hello"');
    assert.equal(formatJSONValue('a "quote"'), '"a \\"quote\\""');
    assert.equal(formatJSONValue(""), '""');
  });
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ?? (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/json-format");
  assert.equal(tezbar.upstream.path, "extensions/json-format");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  assert.equal(commands.length, 6);
  for (const cmd of commands) {
    assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
  }
  const deps = (data as { dependencies: Record<string, string> }).dependencies;
  assert.ok(deps["@raycast/api"]);
  assert.ok(deps["js-beautify"]);
});
