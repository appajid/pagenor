# Native document save safety — P02.11

Save, Save As and Save a Copy now write to a unique temporary file in the
destination directory. Pagenor writes the complete serialized document, retains
portable permissions, syncs the file, then replaces the destination through the
existing tempfile dependency. Live saved state/path and recovery cleanup still
change only after a successful Save; Save a Copy retains the original state.

## Reproduced failure and implementation

The previous native command path used `std::fs::write`, which truncates an
existing file before its write completes. A synthetic failure after 32 archive
bytes reproduced loss of the previous readable document. The same regression
now passes: previous bytes remain exact and reopen successfully, no staging file
remains, and retry installs the complete new document.

A small engine-owned native writer serves the three existing commands. Package
confinement and recovery ownership retain their existing subsystems. Serialization,
data-merge path preparation, `.pgnr`/legacy extensions, history, document fonts,
UI commands and WASM download behavior retain their existing ownership. No
new dependency, unsafe code, document-format change or version bump is introduced.

Existing file symlinks resolve to their target before staging so saving retains
the link. Dangling links return an error. Non-regular destinations and read-only
files are refused before replacement. Opening an existing destination for write
without truncating checks its access permissions/ACL; the handle is closed before
Windows replacement. Unix mode bits are retained; new Unix files use 0666 restricted
by the process umask, matching the previous writer. Replacement staging stays private while bytes are written. Staging has an RAII owner.
Successful UI Save/Save As/Save a Copy clears a stale error from an earlier failure.

## Validation

Five new regressions cover partial-write preservation and retry, an unpublished
new document after failure, Unix file links/modes, all three read-only command
routes preserving document/path/dirty state and recovery, and replacement Copy
retaining original bytes/history/recovery. The existing serialization-refusal
regression also passes. All seven quality gates pass: fmt, check, strict Clippy, workspace tests, assets,
layers and WASM. Total: **1,484 passing tests, 3 ignored, 20 WASM packages**.

Native computer-use testing on the final signed Mac bundle verified File Open,
canvas editing and Cmd+S on an isolated read-only fixture. The unsaved star and
read-only error remain after failure; the on-disk SHA-256 matches its prior save.
After restoring write access, Cmd+S saves the latest edit, clears the unsaved star
and removes the stale error. ZIP/document JSON inspection confirms the new text
and no remaining staging files. Closing the saved fixture returns to Welcome.
The automation encountered Open/accessibility timeouts; subsequent native window
inspection showed the document open and later interactions succeeded. No new
Pagenor crash report appeared.

Installed bundle: `/Applications/Pagenor.app`, version **0.3.0**, build
`uncommitted-P02.11`. Strict signature verification passes; app/document ICNS
hashes match the separately verified icons. Test edits used only the separate
`Save safety check.pgnr` fixture; owner documents were not edited. Only regenerable
Rust incremental cache was cleared to make room; final gates/build ran with
`CARGO_INCREMENTAL=0`. No GitHub changes were published.

## Limits

File sync precedes replacement; parent-directory sync and full power-loss
durability are not established. Replacement changes file identity; extra
metadata was not copied by the portable permission API in P02.11.
[P02.12](NATIVE_SAVE_METADATA.md) now preserves macOS tags/xattrs/ACLs and Unix
ownership, with Linux visible xattr/ACL copying; Windows extended metadata and
other platform-specific metadata remain open. Native Windows/Linux execution, network/cloud
filesystem behavior and hostile concurrent path changes remain milestone checks.
The partial full-disk failure is deterministic fault injection, not an actual
full-disk native run. Source changes and validation stay local, version 0.3.0.
