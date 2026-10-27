# Finite source compatibility

Behavior comes from own commit `e40abbce5b84ca2c34b8f6f4becb70a60cd04656`, not a
reconstruction of upstream or `src/lib/json.ts` alone. The executable oracle
transpiles the frozen **actual `src/utils.ts` and all command/view TSX files**, and
executes callbacks with the genuine npm-locked **js-beautify 1.15.1**. Its original
tarball is retained with lockfile SHA512 integrity. Raycast services, React hooks,
`useForm`, `useLocalStorage` and `usePromise` are explicit capture stubs, not native
Raycast or GUI execution. The trusted finite oracle runs in a separate VM context
without process/environment access; never feed it arbitrary untrusted expressions.

`fixtures/oracle.json` has 663 deterministic vectors over four indent settings.
The Rust differential test compares actual result bytes and accept/reject behavior,
with 16 explicit exclusions (four unsupported forms × four indents). Diagnostics,
source toasts, runtime exception strings and arbitrary JavaScript/beautifier
behavior are not universally reproduced. `fixtures/action-oracle.json` records
12 actual command callback paths, viewer limits, stored text and action ordering.
This is a bounded compatibility port, not universal JavaScript/js-beautify parity.

## Valid JSON and stringify

* Valid JSON is validated but its original number and string tokens, property
  order and duplicate properties are retained for formatting. Thus `-0`, `1.00`,
  `1E+3`, `1e400` and a large integer keep their spelling. Object/array indentation,
  inline primitive arrays, nested arrays and preserved source newlines are covered
  by the finite beautifier corpus. No arbitrary comments/options/JS syntax are
  passed to a JavaScript beautifier at runtime.
* Fallback data literals and JSONLines use JS f64 semantics and `ryu-js` numeric
  rendering: `-0` becomes `0`, large integers round, non-finite numbers stringify
  as `null`. Duplicates keep their first insertion position and last value;
  canonical array-index keys sort before other own keys. The fallback literal's
  `__proto__` setter is not an own property; strict JSON `"__proto__"` is retained.
* Strings use UTF-16 internally, including decoded surrogate pairs and escaped
  unpaired surrogates. Stringify emits lowercase `\ud800` for unpaired units and
  literal Unicode for pairs. Valid JSON formatting retains original escapes/case.
  `formatToJsonValue` stringifies raw text; it does not parse it.
* **Transport policy:** protocol strings must be well-formed UTF-8 Unicode. The
  pinned SDK's serde_json rejects a wire string containing an unpaired surrogate.
  To format JSON containing one, send the six ASCII characters `\ud800` *inside*
  the input text (double escape at the JSONL transport layer). The UTF-16 stringify
  oracle tests unpaired units separately. An unescape recovery or outer string
  that would itself produce an unpaired Rust transport string returns an explicit
  error; no U+FFFD replacement is silently substituted.

## Bounded literal recovery

Supported data forms include single-quoted strings, ASCII unquoted identifier and
numeric keys, trailing commas, array holes, `undefined`/`NaN`/`Infinity`, unary
numeric signs, decimal fractions/exponents, hexadecimal/binary/octal integers up
to 128 bits before f64 rounding, plain untagged templates without interpolation,
standard string escapes, `\xNN` and `\u{...}`. The own escape heuristic and one
trailing semicolon removal are retained. The source's loose parentheses/tagged
string guard also applies before fallback, including its false positives inside
quoted data. Strict JSON is accepted before that guard, as in the source.

Arithmetic (`1+2`), spread (`[...[]]`), computed keys (`{['a']:1}`) and numeric
separators (`1_000`) are the four explicit corpus exclusions. Member access,
identifiers outside the closed constants, calls, getters, functions, assignments,
interpolation, comments and other executable/dynamic expressions are rejected.
No embedded JS engine, JS/Bun subprocess, eval or execution of input is present.
Unsupported data syntax gets a concrete error instead of speculative recovery.

## Actual utils versus the newer helper

The helper's `formatToJSONLinesPure` uses `.map(v => JSON.stringify(v))`; actual
command-used `formatToJSONLines` uses **`.map(JSON.stringify)`**. The oracle executes
the latter directly. For JSON.parse-created arrays, its numeric callback index
is ignored as a replacer and the callback's original array is ignored as `space`
(only string/number space arguments qualify). Observed entries stay compact;
no invented pretty-printing based on the callback index is introduced. Tested
outputs agree, while their source/callback signatures and failure toasts differ.
JSONLines accepts only a valid JSON array with literal `[`/`]` endpoints, so outer
whitespace and JS trailing commas are rejected even though Format JSON accepts
other recovery forms.

The helper explicitly returns early for empty input and catches an invalid outer
JSON string. Actual utils can emit a failure toast for empty input and can throw
TypeError after a failed outer parse passes undefined into convert. The port
returns recoverable errors without copying. Source `Function` can evaluate some
expressions that its regex misses; those are intentionally rejected here.

## Viewer and form behavior

Only selected-and-show uses `maxInitialLines`, matching its actual `JsonTreeView`.
`parseInt(...,10) || 10000` and `slice(0,maxLines)` behavior includes zero/NaN
fallback, signs, numeric prefixes and negative limits (within the explicit 128-byte
preference bound). The full result is retained for Copy/Paste; Load All Lines
changes presentation only. Index/JSONLines detail retains the complete result and
JSONLines entry boundaries, without adding the selected-view truncation rule.

Forms preserve Format then View Result; Back restores the submitted draft. The
text-value form remains editable/persistent and Copy always uses submitted fields.
Draft revisions guard delayed field events, including cancellation and edits queued
behind Copy. Safe indented code replaces unsafe source triple-backtick interpolation;
actual committed Markdown rendering is tested against production-generated detail
and host-error content. User text cannot create links, images or arbitrary buttons.
The host may add its standard fixed code-block Copy button; the extension primary
Copy/Paste remains the full-output action. Cmd+L shortcut metadata is retained;
the current generic Surface has no general shortcut dispatcher, so use Load All
Lines in the action menu. Native key-binding/OS GUI behavior is not claimed.
