# Validation record — 2026-10-25

Reviewed scope: source under `v2/` plus the root README discovery link. Branch
`feat/native-v2-rust`; HEAD/source `e40abbce5b84ca2c34b8f6f4becb70a60cd04656`.
Pinned standalone SDK `e955911f0bf5b558f9d3eb5183df0adc95c22af2` (0.1.1, protocol 1).
Host contracts `ea100cee3f69711de12485ba061950c5fc65258f`.

## Executed gates

| Gate | Actual outcome |
| --- | --- |
| Rust stable 1.98.0, native aarch64-apple-darwin | 22 tests passed; one extracted-package test separately run, not silently counted as passed |
| Rust 1.85.0 / SDK declared MSRV 1.85 | all 22 regular tests passed |
| cargo fmt --check | passed |
| Strict Clippy --all-targets -D warnings | passed |
| Windows GNU --all-targets check and strict Clippy | passed as secondary compilation evidence; no Windows execution |
| Pinned SDK's own exact committed snapshot + own Cargo.lock | 17 tests passed (4 helper, 1 launcher, 12 process); no SDK source edits |
| Actual TS/utils/helpers/TSX + locked js-beautify 1.15.1 | 663 vectors: 586 successful result byte comparisons, 61 matching rejections, 16 named deliberate exclusions |
| Actual command callback oracle | 12 foreground callback paths; copy/paste order, Format/View Result, viewer limits, stored text; explicit Raycast/React stubs |
| Disposable exact V1 source/lock | 18 tests and typecheck passed; lock bytes unchanged; V1 files not edited |
| Actual committed Surface/Form/Detail/Markdown | baseline callback/SubmitForm failures reproduced; separate patch paths, Back, draft retention/revisions and production-content safety passed |
| Exact committed schema 3 | source manifest passed; 11 negative cases rejected; empty development artifacts permitted |
| Extracted actual runtime/launcher validators | positive/negative identity, protocol, capabilities, duplicate commands, deadlines, migration/path and inventory cases passed; no validator stubs |
| Actual registry resource lookup | baseline version-directory failure reproduced; separate registry-icon.patch fixes path resolution with original PNG, preferences root unchanged |
| Original PNG | 237176 bytes, cfee8098d945a0541c391777deeae1fc6e3ddee07a8516f1f730b2103e4e8f13 |
| Native ARM64 release executable | built; real SDK --protocol-info / --self-test passed |
| Built ZIP extraction | closed inventory/digests, actual host file validator including tamper/recovery, offline original icon, isolated production JSONL smoke passed |
| Deterministic packaging | two ZIPs from identical built inputs compared byte equal |
| Independent source rebuild | source archive extracted under .work, no local SDK/app path; fresh Cargo cache fetched the immutable HTTPS SDK revision and built release independently |

The real production executable pipe tests cover hello, initialize, commands,
session open/call/dispose, search/refresh responses, cancellation and shutdown;
reverse typed calls, capability denial, null/failed/contradictory acknowledgements,
exact-byte explicit retries, uncertain-paste warnings, copy/hide phases, stale IDs,
queued edits after Copy, private text restart after canceled storage/effect, canceled
open removal/ID fencing, malformed JSON/unpaired-surrogate/CRLF rejection, stderr
framing and EOF cleanup. Host services in these tests are capture peers, not real
OS clipboard/selection/paste execution. Domain tests use the production Adapter;
no duplicate fake adapter replaces it. SDK's own tests also cover latest-search
transactional commits. The extension has no search UI or background work.

`fixtures/host-contract.rs` is generated from exact committed host functions;
imports/test wrappers are isolated from app infrastructure. Vendored host functions
retain their existing lint policy; strict Clippy gates all new extension/harness
code. The extracted test does not claim installer/catalog/profile integration.
The SDK metadata entrypoint called --self-test does not itself prove command QA;
the separate domain/process tests provide that evidence.

`integration/renderer.test.cjs` uses React TestRenderer + JSDOM. Presentation,
unrelated grid/list/color UI, IPC hooks and overlay presentation are documented
stubs. Native GUI/focus/keyboard behavior and a full host rebuild are not asserted.
The patch remains separate and unapplied. Persistent text editing requires its
reserved draft revision metadata; missing metadata returns an explicit error.

## Search, dependency and package evidence

Initial XERJ generation 1 indexed all 12 original source files. The final same-prefix
refresh aborted (exit 1) because frozen dataset schema rejects new
`v2/extension.json` fields. Search coverage is incomplete/stale for new files.
No prefix bypass, schema change, service restart or global mutation was attempted;
exact source fixtures, full Git checks and hash inventories are authoritative.

The local SDK HEAD was captured as e955911f at task start. A later public
`git ls-remote ... HEAD` returned `aaec571673ab64a85a136cf6bd054ef660c4d93c`;
this does not move the explicitly requested immutable dependency. Both Cargo.toml
and Cargo.lock remain pinned to e955911f, and the isolated build fetches that rev.
No claim is made that the mutable SDK default branch still equals the pin.

Local ZIP/digest files are disposable build outputs under `artifacts/`; they are
not a published release or trusted host pins. `scripts/package.py` derives their
archive/executable digests from actual built bytes and inventories every packaged
file except extension.json. All reviewable source fixtures/patches/locks/notices
are Git-visible and included in INDEX. Parent should regenerate/check INDEX after
any final edits/staging. The source manifest still has empty artifact pins.

## Not run / remaining release work

* Native Windows MSVC build/execution and native Windows GUI acceptance.
* Actual macOS/Windows desktop clipboard, selection permissions, focus restoration,
  paste, hide, notices, install/uninstall and launcher icon QA.
* Remote CI; `ci/native.yml` is a reviewable definition awaiting parent activation.
* Parent review/rebase/application of the narrow renderer and icon registry prerequisites against the
  selected current host commit, followed by full host integration verification.
* Approved target releases, trusted catalog acquisition/digests, host registration,
  rebuild/install and release acceptance. No public release/deploy/install is
  authorized or performed by this child.

Finite formatting/literal differences, rejected dynamic syntax and explicit
unpaired-surrogate transport policy are in COMPATIBILITY.md. Checkpoints are bounded
and volatile; a canceled SDK open is removed and cannot be refreshed. Crash/OS
ack uncertainty is not turned into false success or automatic replay.

Final intended Git state: root README modified, new v2/ files untracked; staged
diff empty, HEAD unchanged, clone intact. No commits, pushes, PRs or deletions.
