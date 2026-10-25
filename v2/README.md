# Format JSON: standalone Rust executable

This `v2/` source implements **schema 3 / execution process / protocol 1**, using
`tezbar-extension-sdk` 0.1.1 at immutable revision
`e955911f0bf5b558f9d3eb5183df0adc95c22af2`. It builds independently of Tezbar,
Tauri, Node and sibling repositories. The directory name retains the port-workflow
convention; this is not a compiled-in schema-2 adapter.

All six original command names and their metadata are in `extension.json`:
`formatClipboard`, `formatSelection`, `formatSelectionAndShow`, `index`,
`formatJsonLines`, `formatToJsonValue`. Clipboard/selection commands deliver
automatically on launch; forms offer Format then View Result. The text-value form
persists editable text and copies a JSON string. No ordinary delivery/confirmation
screen was added. Preferences remain `autopaste=false`, `indent=tab` (2/4/8 also
supported), `maxInitialLines=10000`.

## Build and verify

```sh
cd v2
cargo fetch --locked
cargo test --locked --all-targets
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.85.0 test --locked --all-targets
cargo build --release --locked
./target/release/tezbar-json-format --protocol-info
./target/release/tezbar-json-format --self-test
python3 scripts/provenance.py --check
python3 scripts/host_contract.py --check
```

Both command-line probes use the real SDK entrypoint and report protocol/command
metadata; the Cargo tests exercise domain behavior and real JSONL pipes.
Production needs no JavaScript runtime and never evaluates input as code.
[COMPATIBILITY.md](COMPATIBILITY.md) defines the finite compatibility surface and
intentional differences. [LIFECYCLE.md](LIFECYCLE.md) covers acknowledgements,
cancellation, drafts, storage and resource bounds.

Development-only oracle and renderer reproduction:

```sh
python3 scripts/reproduce_oracle.py  # frozen V1 source + exact npm lock
npm ci --ignore-scripts --no-audit --no-fund --prefix integration
npm test --prefix integration       # requires cargo build/test first
python3 -m venv .work/py
.work/py/bin/pip install -r scripts/requirements-lock.txt
.work/py/bin/python scripts/validate_schema.py
```

Use `PYTHON=python` where the interpreter is named `python`. The renderer test
applies [integration/renderer.patch](integration/renderer.patch) only to a disposable
copy of committed host source. The patch is a **host prerequisite for parent
review**, covering all SubmitForm payloads, Detail button callbacks and monotonic
draft revisions. The separate `integration/registry-icon.patch` fixes verified version-directory
icon resolution. Both patches remain unapplied to the host. See
[integration/README.md](integration/README.md).

## Packaging and release boundary

```sh
python3 scripts/licenses.py
python3 scripts/package.py --binary target/release/tezbar-json-format \
  --target aarch64-apple-darwin
python3 scripts/package_test.py artifacts/json-format-0.2.0-aarch64-apple-darwin.zip
```

The script checks native executable headers, bundles the original PNG and actual
license/notice files, inventories every member except the manifest itself, and
writes deterministic ZIP and digest metadata. `--url https://...` additionally
creates a reviewable release reference from **built bytes**. It does not publish,
install or change source/catalog pins. Given identical input files, packaging is
byte deterministic; Rust compiler reproducibility across toolchains is not claimed.

The source manifest intentionally has empty `artifacts`, permitted by the actual
schema/runtime validator. It cannot be installed as an executable without a
verified target archive and trusted reference. [ci/native.yml](ci/native.yml)
contains isolated macOS ARM64 and Windows MSVC x64 gates; parent must explicitly
activate it in the repository's workflow directory. It has not run remotely.
Windows native execution and real desktop clipboard/selection/focus/paste QA on
both platforms remain not-run. No host registration/rebuild/install, release,
push or deployment was performed.

See [VALIDATION.md](VALIDATION.md), [PROVENANCE.md](PROVENANCE.md), [NOTICE](NOTICE)
and [INDEX.json](INDEX.json). After final review/staging, parent should rerun
`python3 scripts/index.py` and `python3 scripts/index.py --check` before committing.
