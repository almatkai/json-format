import beautify from "js-beautify";

export function unescapeJsonString(str: string): string {
  return /^[^"]+\\".*$/.test(str)
    ? str.replace(/\\(["\\/bfnrt]|u[0-9a-fA-F]{4})/g, function (match, p1) {
        const unescapeMap: Record<string, string> = {
          '"': '"',
          "\\": "\\",
          "/": "/",
          b: "\b",
          f: "\f",
          n: "\n",
          r: "\r",
          t: "\t",
        };
        if (p1[0] === "u") {
          return String.fromCharCode(parseInt(p1.slice(1), 16));
        } else {
          return unescapeMap[p1] || p1;
        }
      })
    : str;
}

export function isJson(str: string): boolean {
  try {
    JSON.parse(str);
  } catch {
    return false;
  }
  return true;
}

export function isArray(str: string): boolean {
  return str.startsWith("[") && str.endsWith("]");
}

export function isExecuteable(input: string): boolean {
  return /\([\s\S]*?\)|[\w$]\s*`[\s\S]*?`/.test(input);
}

export function tryConvertToJsonString(input: string): string | undefined {
  let processed = input;
  try {
    processed = unescapeJsonString(processed);
  } catch {
    // ignore
  }
  if (isJson(processed)) return processed;
  if (processed.endsWith(";")) processed = processed.slice(0, -1);
  try {
    if (isExecuteable(processed)) throw new Error("executeable");
    const result = Function(`"use strict";return (${processed})`)();
    return JSON.stringify(result);
  } catch {
    return undefined;
  }
}

export function formatWithBeautify(jsonStr: string, indent: string): string {
  const options = {
    indent_size: indent === "tab" ? 1 : parseInt(indent, 10),
    space_in_empty_paren: true,
    indent_with_tabs: indent === "tab",
  };
  return beautify(jsonStr, options);
}

export async function formatJSLogic(text: string, indent: string): Promise<string | undefined> {
  const trimmed = text.trim();
  if (!trimmed) return undefined;
  const firstChar = trimmed[0];
  let json: string | undefined;
  if (firstChar === '"') {
    try {
      const parsed = JSON.parse(trimmed) as string;
      json = tryConvertToJsonString(parsed);
    } catch {
      return undefined;
    }
  } else {
    json = tryConvertToJsonString(trimmed);
  }
  if (!json) return undefined;
  return formatWithBeautify(json, indent);
}

export function formatToJSONLinesPure(input: string): string[] | undefined {
  if (!isJson(input) || !isArray(input)) return undefined;
  const jsonVal = JSON.parse(`{"data":${input}}`) as { data: unknown[] };
  return jsonVal.data.map((v) => JSON.stringify(v));
}

export function formatJSONValue(text = ""): string {
  return JSON.stringify(text);
}
