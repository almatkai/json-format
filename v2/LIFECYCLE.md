# Process lifecycle, effects and storage

The production main calls the pinned independent SDK `serve(Adapter)`. Stdout is
protocol-only UTF-8 JSONL; errors go to stderr and produce a failing exit. No app,
Tauri or sibling crate dependency exists. There are no background tasks, workers,
leases or refresh polling in this extension; views set `refreshMs: null`.

The host and SDK references are frozen Git objects listed in `fixtures/refs.json`.
The host services used are only:

| Method | Parameters | Required successful response |
| --- | --- | --- |
| host.clipboard.read | `{}` | text string |
| host.selection.read | `{}` | text string |
| host.clipboard.write | `{text}` | `{ok:true}` |
| host.clipboard.pasteText | `{text}` | `{ok:true,message:"Pasted text"}` (message optional in validation) |
| host.window.visibility | `{visible:false}` | `{ok:true}` |

Acknowledgements must be objects with `ok:true` and at most an optional string
message. Null, false, errors and additional contradictory fields are not success.
The SDK TestContext's default null is explicitly tested as a failure. The committed
host pasteText writes the clipboard, hides/restores focus and pastes; the extension
sends no duplicate copy/hide for paste. Foreground source copyFormattedJs closes
the window after copy, so automatic copy performs that separately after its write
acknowledgement. Detail Copy has no extra hide. Source HUD/toast feedback becomes
the ordinary process response message; no invented HUD/core channel is used.

## Cancellation and checkpoints

SDK session state is cloned for **every operation**, and commits only on success.
Cancellation can happen after OS mutation or an atomic storage write. Neither can
be rolled back by discarding staged Value state.

Immediately before an OS delivery await, the adapter stores an immutable in-memory
checkpoint of the exact result, owning session/command, action and phase. This
checkpoint is separate from staged state. Sending, delivered, closing and complete
phases distinguish an uncertain write/paste from an acknowledged copy whose hide
is still pending. Completion is recorded before the response. Retry IDs are issued
only for an error/recovery view and consumed before another await. No refresh,
search, old action, late reverse response or process startup replays an effect.
An explicit retry uses the same prepared bytes without rereading/reformatting.
Uncertain paste says it may already have happened and warns of duplication.
Retry Close repeats only visibility, never the acknowledged copy.

The original form remains the response surface after Format, with fresh action IDs;
the host has already hidden it after successful delivery. This is necessary because
the committed renderer keeps old root/action IDs on an action's no-view response.
The actual no-view Clipboard/Selection launch commands return no-view success.
Ordinary launch/form flows never add a delivery/confirmation screen.

A canceled **open** is removed by the pinned SDK without a dispose callback; its
session cannot be refreshed. Its bounded orphan checkpoint remains until shutdown,
and defensively fences an accidental reuse of that ID rather than rereading or
replaying. Host IDs must not be reused. There is no claimed recovery UI for a
removed/canceled open. A new user launch is a new explicit operation and rereads
its source. Process crash loses all checkpoints; there is no persistent effect
replay or claim that an uncertain OS action can be reconstructed after a crash.

SDK serialization, latest-search ownership, cancellation and disposal enforce the
outer session scope. IDs additionally bind session, command and view epoch. Edits
have a stable per-session ID so queued text survives a Copy response. The separate
bounded draft lane uses the renderer's monotonic `__tezbarDraftRevision`; older
values cannot overwrite newer text, even if SDK handler scheduling reorders them.
Copy takes current submitted input, not a stale server description. Index/JSONLines
fields use unchanged seed values; persistent fields omit `value` after first render,
so actual FormRuntime retains local queued drafts while descriptions update.

## Private storage

Only the source's persistent text is saved, at extension-private
`dataDir/json-format/storage.json`, together with an internal monotonic revision.
Preferences come from the SDK's frozen session snapshot, not this private store.
There is no V1/profile migration or access to undeclared files. Unknown keys are
preserved. Typed text/revision fields are validated on read **and inside** the SDK
atomic update closure; malformed JSON, wrong types or oversized stores fail without
a reset. Revision-stamped closures prevent an old canceled storage job from
replacing a newer accepted write. Restart reads the persisted text even when the
previous session's staged state was discarded. One host process/profile is assumed,
as required by the SDK; cross-process writers are not a supported store protocol.

## Explicit bounds

* Input text: 1 MiB UTF-8; formatted/stringified/joined output: 4 MiB. Exceeding
  either returns an error, with no partial copy or formatting truncation.
* Parser depth: 64; nodes (including keys/holes): 100,000; scanner work: 8 MiB
  byte visits per parse. Fallback reparses are a fixed finite sequence. Stringify
  and formatting are bounded by these counts/output limits; property duplicate
  resolution/order uses bounded O(n log n) maps/sorting, not executable input.
* Max Initial Lines preference: at most 128 bytes; indent is tab/2/4/8; autopaste
  is strictly boolean. Unknown preferences remain in the host's store untouched.
* Store: pre-read 2 MiB file metadata check, typed string <=1 MiB, and 2 MiB encoded
  check inside each atomic update. The SDK internally reads the private file; its
  API does not make the metadata check atomic against external file tampering.
* Checkpoints: at most 128 and 16 MiB aggregate logical payload, released on
  dispose/shutdown. Snapshot clones and SDK staged states add bounded transient
  memory; this budget is not an OS RSS guarantee. Draft lanes: at most 128.
* Pinned SDK: 16 MiB frame, 128 sessions, bounded handlers/queues/reverse requests,
  120s reverse timeout, cleanup deadlines, scoped cancellation. The extension
  starts no tasks. Process grants are service permissions, not an OS sandbox.

No global secrets, machine profile scanning, automatic effects after restart,
unbounded receipts or application-domain RPC channels are present.
