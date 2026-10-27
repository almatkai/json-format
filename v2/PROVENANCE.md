# Immutable inputs and scope

Selected clone: `/Users/almat/Desktop/code/Tezbar APP/rust-extension-ports/json-format`.
It was absent, cloned via HTTPS, fetched, confirmed clean/main with one worktree,
and switched to `feat/native-v2-rust` from freshly fetched `origin/main`.
Actual source is `e40abbce5b84ca2c34b8f6f4becb70a60cd04656`; no dependent branch.
The surrounding parent directory is non-Git, as explicitly authorized.

| Input | Immutable revision |
| --- | --- |
| Own https://github.com/almatkai/json-format | e40abbce5b84ca2c34b8f6f4becb70a60cd04656 |
| Independent https://github.com/almatkai/tezbar-extension-sdk-rust.git | e955911f0bf5b558f9d3eb5183df0adc95c22af2 |
| Committed Tezbar host | ea100cee3f69711de12485ba061950c5fc65258f |
| Raycast extensions/json-format | 6734d85992290066fd31a7f5bee3cca3d3545a1a |

The SDK dependency uses that immutable HTTPS Git revision, crate 0.1.1, protocol 1,
MSRV 1.85, with Cargo.lock. No local SDK path or app/Tauri dependency is published.
All host/SDK reads came from `git show <exact-sha>:<path>`; dirty host source was
not consulted. Legacy NATIVE_V2_FORMAT.md/schema 2 describes compiled metadata,
not executable installation. The applicable validator is schema 3 plus committed
`process_install::validate`; `host_contract.py` extracts actual validation,
path/digest/file checking and actual launcher validation functions. No launcher
validator stub is substituted. Imports/test wrappers and crate version environment
are harness-owned; installer/profile/catalog side effects are not exercised.

`fixtures/provenance.json` records byte sizes, SHA256 and Git blob SHA1 for every
frozen own/host/SDK/upstream/beautifier input. `provenance.py --check` verifies it.
`INDEX.json` additionally inventories all review deliverables, including fixtures,
renderer patch, generated contract excerpt, locks, licenses and this documentation.
It excludes itself and disposable build outputs. `index.py` checks Git visibility
so broad ignores do not silently hide fixtures. Parent must rerun it after any
final edits and after staging; no staging/commit occurred in this task.

Original icon, fetched from the pinned upstream raw URL:

```
https://raw.githubusercontent.com/raycast/extensions/6734d85992290066fd31a7f5bee3cca3d3545a1a/extensions/json-format/assets/command-icon.png
237176 bytes
SHA256 cfee8098d945a0541c391777deeae1fc6e3ddee07a8516f1f730b2103e4e8f13
```

It is packaged unchanged at `resources/command-icon.png`, used by extension and
command metadata, inventoried/digest checked and tested after extraction. The
committed registry initially searches the managed root rather than its verified
version resource directory. `integration/registry-icon.patch` preserves that
verified directory for icon lookup without moving preference paths; an exact
before/after function test proves resolution of the original PNG offline. No generated/SVG/WebP substitute or V1 asset
replacement is used. Native install/launcher GUI icon acceptance remains unrun.

Pinned upstream package.json identifies **destiner**, real contributors and MIT;
its real root LICENSE is preserved. Own contributors and almatkai remain credited
in NOTICE. V1 LICENSE bytes contain literal `\n`; they are left untouched, retained
as a fixture, and a newline-decoded copy supplies the native MIT license text.
SDK/host Apache licenses and notices and js-beautify's MIT license are retained.
`licenses.py` copies actual Cargo package license/notice files for the union of
normal dependencies on both intended targets; no invented copyright notices.

## XERJ

The xerj-search skill was read before code exploration. Canonical own prefix is
`ax-ce7075f14d27`, state directory
`~/.local/share/xerj/workspaces/ax-ce7075f14d27`. Initial autoindex completed generation
1 with exit 3 `completed-with-junk`, all 12 source files indexed, no code junked.
Four non-code/junk inputs were skipped. Searches located actual utils first;
exact reads and hashes, plus rg, established exhaustive source facts.

The existing host prefix `ax-865a7399102e` returned obsolete adapter references;
SDK prefix `ax-869a4471df77` returned no match. These were discovery hints only.
The host was not reindexed from dirty files; no sibling was indexed as this project.
A stale, frozen, 429/RSS-limited or incomplete index is never absence evidence.
There was no service restart, schema mutation or prefix bypass. Final own index
refresh outcome is recorded in VALIDATION.md; all deliverables remain independently
checkable by exact reads/hashes regardless of search coverage.

Only new `v2/` files and the short root README discovery link were changed. Host,
SDK, other ports, other worktrees, profiles, catalog and ledger were not edited.
No push, PR, commit, release, deploy, installation or clone deletion was performed.
