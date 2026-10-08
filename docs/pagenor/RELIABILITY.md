# Phase 02 — clipboard and document reliability

Updated: 2026-10-09. P02.1–P02.3 are complete locally; Phase 02 and M0 are in progress.
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

## P02.2 — retain recovery checkpoints across restarts

PG-0203 was reproduced before the fix: opening recovery deleted its only snapshot,
so another immediate exit before the next timer/manual save recovered nothing.
A second failing regression showed that saving a new document with a reused
process-local UID overwrote a pending, unreadable recovery entry.

Recovery now retains the original checkpoint and records its ownership in native
document session state. Recovery keys are independent of runtime UIDs. New entries
reserve an unused numeric filename with `create_new`; corrupt snapshots and orphan
metadata remain untouched. Repeated recovery-open calls in the same session skip
owned entries, including paths with different spellings of the same directory.
Updates keep their original keys. Successful save, revert and explicit close remove
only that document's checkpoints, including explicitly supplied recovery folders.
Copy-save and failed save/revert keep the checkpoints. Serialization errors propagate
rather than writing an empty archive. No document format or dependency changed.

Checkpoint data and metadata are staged and flushed with `sync_all` before rename;
metadata replacement precedes document replacement. A staging or metadata failure
keeps the previous document bytes usable. A failure of the final document rename can
leave newer title/timestamp metadata alongside the previous valid snapshot. Unique
staging files avoid truncating leftovers from an interrupted write. Failed staging
files and new empty reservations are removed; a failed existing checkpoint can retry
under its original key. Collision searches are bounded to 4096 recovery-key candidates
and 64 staging candidates; exhaustion returns an error and preserves pending data.
Mutex poisoning is handled and `unsafe_code = "forbid"` is preserved.

Seven new synthetic regressions cover retention, repeated open, ownership and
cleanup, reused/extreme UIDs, failed replacement and retry, explicit directories,
undo after save, copy-save, multiple entries and stale staging files. A separate
Mac probe uses five child processes with abrupt exits (no Session destruction or
normal exit-save). Two immediate restarts retain identical dirty document models,
Unicode text and original paths without a recovery save. Later edits use the original
keys; a new document with runtime UID 3 skips corrupt key 3 and orphan metadata 4,
using key 5. Another process recovers all three documents and verifies copy-save,
save and close cleanup. All fixtures are isolated temporary data; user files and
default recovery folders are not accessed.

## P02.3 — recovery scheduling, clean undo and retry

Two timer regressions failed against the original behavior. The timer checked
only dirty documents, so undo returning to the saved document left its obsolete
checkpoint indefinitely. `recoveryMinutes: 1e308` was accepted by `prefs.set`;
multiplying by 60 overflowed to infinity, preventing any timed recovery save.

The engine now exposes whether a session has dirty documents or owned checkpoints
awaiting cleanup. The existing `file.recovery.save` command handles both on the next
scheduled pass; unclaimed files do not schedule recovery for a clean session. The
UI timer requests its own repaint, independently of font polling and caret animation.
Intervals are bounded to five seconds through the preferences UI's one-hour maximum,
before multiplication. Nonfinite intervals fall back to 30 seconds; a reset or
nonfinite previous clock is rebased, and an invalid current clock defers the attempt.
Exact deadlines count as due. Failed attempts are throttled to the same interval,
retain their previous valid snapshot and show the existing recovery error. A later
successful timer pass replaces that failure status with “Recovery is up to date.”
Normal edits still have the periodic checkpoint window; this is not per-keystroke
recovery. No new dependency, document format or unsafe code was introduced.

Seven new UI tests cover clean undo, oversized/default/minimum/nonfinite intervals,
clock reset, independent idle wakeup, disabled recovery, multiple inactive documents,
unknown files and a failed replacement followed by a scheduled retry. A real-time
headless egui fixture honors repaint delays using actual elapsed time, without
control requests or manual recovery saves. Three fixture processes plus three fresh
verification processes pass: inactive dirty documents checkpoint at about six
seconds; clean undo removes its snapshot on the second pass (about 12 seconds);
an injected metadata-replacement failure appears on the second pass, preserves the
previous snapshot and retries successfully on the third (about 18 seconds). Fresh
processes verify exact document/Unicode/path data and cleanup. CPU-rendered failure
and retry images were inspected; the current edit stays visible and intact.

A live eframe window fixture was also attempted with isolated data, but timed out
before reaching its timer checks while AppKit reported invalid message connections.
It was terminated by its test runner and its temporary files removed. Live native
window validation therefore remains pending; headless evidence does not establish
native event-loop behavior. A separate source audit found that the desktop `on_exit`
hook saves preferences only: a final recovery pass for edits newer than the last
periodic snapshot is the next local task (PG-0206), requiring quit/close validation.

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
| Immediate second restart before recovery timer/save | Passed (P02.2). Retained checkpoint; synthetic sessions and separate processes with abrupt exits. |
| Browser / TextEdit / Pages / Word actual clipboard transfer | Pending. Require real local application transfer evidence. |
| Multiple entries, corrupt pending files and process UID reuse | Passed. Ownership regression and fresh-process probe; unknown entries remain intact. |
| Checkpoint write failure, retry, save/revert/close cleanup | Passed for synthetic replacement/staging failures and failed save/revert. Prior snapshot remains usable; copy-save retains it. |
| Recovery timer deadlines, idle wakeup, clean undo and retry | Passed (P02.3). Seven UI tests and a real-time headless egui loop; fresh-process recovery matches. |
| Recovery error and successful-retry display | Passed. CPU failure/retry images inspected; latest document remains intact. |
| Live native timer/window exit and actual disk-full/read-only storage | Pending. Live window fixture timed out with AppKit connection errors. Normal exit hook has no final recovery pass (PG-0206). |
| Native Windows/Linux runtime matrix | Deferred to milestone publication. |

Validation: `cargo xtask ci` passes all seven steps; 835 tests pass, none fail,
three performance tests are ignored and all 16 WASM packages check. The Mac app
builds. Strict UI-library Clippy, including engine dependencies, passes for Windows GNU and Linux GNU.
No GitHub upload or remote CI was performed. Cross-target checks do not
establish native clipboard/recovery runtime behavior.

Remaining coverage includes real external-application clipboard transfer, final
shutdown protection and the live native recovery timer/exit matrix. Power-loss durability (including directory
sync), cleanup-denied storage and simultaneous app processes claiming the same
checkpoint are not established by these tests. Key reservation prevents new-entry
collisions; it does not add inter-process ownership locking. Keep these limits
separate from the verified immediate-restart and reused-UID fixes.
