# Phase 02 — clipboard and document reliability

Updated: 2026-10-09. P02.1 is complete locally; Phase 02 and M0 are in progress.
Adobe/dongle activation validation remains pending at the owner’s request.

## P02.1 — one reversible paste operation

External/plain paste previously dispatched `text.insert` through `Session::execute`
inside the already undoable Paste command. Both commands recorded undo and journal
entries. Replaying those entries could insert the clipboard contents twice. The
shared command history also stored the selection after the edit: undo of a paste
replacing a short range with longer text restored a caret beyond the original text
rather than the original selection.

Paste now calls the existing insertion handler directly. Its outer command owns
undo, journaling, smart reflow and caption refresh. Raw insertion, newline handling,
insertion formatting and internal formatted clipboard handling remain in the
existing subsystems. History entries capture the document and selection together
before each edit; redo retains the post-edit state through the existing machinery.
No clipboard format, native API, serialization format or dependency was added.

Two new regressions failed before the fix and now pass. They check external and
plain Unicode replacement as one history/journal operation, replay of the paste
segment without duplication, and formatted paste restoring its selected range and
bold text on undo/redo. Additional tests check Unicode, combining accents, emoji
ZWJ sequences and multiple scripts through copy-save, first recovery, save and
reopen, plus the injected UI clipboard bridge and explicit-text bypass. A CPU UI
probe renders paste, undo and redo and checks their model state. The visual fixture
uses Latin/accented text; Indic/emoji tests establish data preservation, not shaping
parity or real third-party clipboard transfer.

## Validation matrix

| Scenario | Evidence / status |
|---|---|
| Engine external Unicode/CRLF/CR paste | Passed. Existing/new synthetic tests. |
| Formatted internal copy, footnotes and plain insertion format | Passed. Existing test and formatted undo/redo regression. |
| External/plain paste undo/journal/replay | Passed. One outer operation, exact document snapshots and one replayed paste. |
| Selection before edit; redo caret/format | Passed. Original range and post-paste caret restored respectively. |
| Host callback and explicit supplied text | Passed with injected `Services::clipboard_text`; real OS formats not exercised. |
| CPU UI paste/undo/redo | Passed and inspected; no native user document/clipboard was changed. |
| Copy-save preserves dirty/path; Unicode and format bytes | Passed. Temporary `.pagenor` round-trip. |
| First restart recovery; save/reopen | Passed. Temporary documents and fresh engine sessions. |
| Immediate second restart before recovery timer/save | Failed probe, PG-0203 open. Recovery currently deletes its source snapshot on reopen. |
| Browser / TextEdit / Pages / Word actual clipboard transfer | Pending. Require real local application transfer evidence. |
| Recovery timer, disk/write failure, multiple entries/process UID reuse | Pending. Resolve PG-0203 first. |
| Native Windows/Linux runtime matrix | Deferred to milestone publication. |

Validation: `cargo xtask ci` passes all seven steps; 821 tests pass, none fail,
three performance tests are ignored and all 16 WASM packages check. The Mac app
builds. Strict engine-library Clippy passes for Windows GNU and Linux GNU.
No GitHub upload or remote CI was performed. Cross-target checks do not
establish native clipboard/recovery runtime behavior.

The repeat-crash probe uses isolated temporary data and three engine sessions.
The first restart opens one dirty document and leaves zero snapshots. A second
restart without a timer/manual save opens zero documents. The next task must
retain a valid checkpoint and prevent restarted UID allocation from overwriting
another pending recovery entry. User files and default recovery folders were
not accessed by these probes.
