# Separate host prerequisite for parent review

`renderer.patch` is based only on committed host
`ea100cee3f69711de12485ba061950c5fc65258f`. The live host working tree was never
checked out, stashed, edited, built or indexed by this implementation. The patch
has not been applied there and may need rebasing if parent selects a newer commit.

The patch changes only `src/renderer/src/raycast-api/index.tsx`:

1. Every action whose kind is `submit-form` receives current FormRuntime fields,
   including the second View Result action in the overlay. Current baseline sends
   fields only to the first selected submit action.
2. Detail's primary button uses a zero-argument wrapper; its MouseEvent must not
   become the action ID. Keyboard Enter already sends the string ID correctly.
3. Form field callbacks and submissions carry monotonic reserved transport field
   `__tezbarDraftRevision`. The port checks it to fence delayed drafts. Copy uses
   submitted text and that revision; field changes use absolute text + revision.

`renderer.test.cjs` applies the patch to a disposable frozen-source copy inside
`.work/renderer`, never to the host. React **18.3.1**, Marked **18.0.2**,
marked-highlight **2.2.4**, highlight.js **11.11.1** and DOMPurify **3.4.0** match the
committed host lock. The separate npm lock pins the harness as well.

The test runs the actual frozen Surface, FormRuntime, DetailRuntime and Markdown
components, reproduces the baseline failures, then verifies the patched paths:
first/second SubmitForm fields, Detail click identity, Back, omitted-value draft
retention and reserved revisions. It renders actual production process-generated
detail/recovery markdown and verifies no attacker link/image/button appears.

Explicit stubs: presentation primitives, unrelated grid/list/color conversion,
IPC hooks/navigation contexts and action overlay presentation. Overlay onExecute
is invoked as a component callback. DOMPurify uses JSDOM. React TestRenderer runs
component behavior; this is not a browser GUI, keyboard/focus/OS test or full app
build. General shortcut dispatch is outside this narrow patch.

Run from `v2/` after `cargo build --locked`:

```sh
npm ci --ignore-scripts --no-audit --no-fund --prefix integration
npm test --prefix integration
```

## Process resource lookup

`registry-icon.patch` is a second separate prerequisite against the same committed
host. The original collect() keeps the managed package root for preferences but
loses the verified version resource directory, so extension and command icons fail
without a legacy sibling asset. The patch preserves `package.dir` returned by
`process_install::resolve` in a private resource_dir field and uses it only for
icon candidates. The preference-only Installed constructor uses None. No preference
path or trust check is moved. A host unit test is included in the patch.

`icon_contract.py` emits exact before/after icon-path functions into a standalone
Rust test harness. `cargo test --test icons` demonstrates the baseline failure and
patched lookup using the original pinned PNG, with no legacy asset or network.
Only the pure icon path boundary is compiled here; the full host collect/installer
integration still requires parent review and a real host build/installation test.
The live host remains untouched. Regenerate/check using
`python3 integration/icon_contract.py [--check]`.
