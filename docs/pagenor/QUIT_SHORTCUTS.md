# Quit during native composition — P02.10

Pagenor 0.3.0 now recognizes Quit while canvas text composition is unfinished.
A final text commit delivered in the same UI pass is inserted once before the
save prompt and recovery checkpoint. Cancel returns to normal typing.

## Cause and focused change

PG-0216 was reproduced on the installed Mac app: Option+E started a dead key;
the first Cmd+Q committed its accent but left the app editing. Ordinary Cmd+Q
from Welcome worked. The pinned winit 0.30.13 macOS view calls AppKit's
`interpretKeyEvents` first and suppresses key-down after an IME commit. Its
Command key-up path still delivers the release. Existing shared shortcut
routing also returned early while composition owned input.

The shared desktop shortcut handler now checks the configured primary-modifier
Quit shortcut before the composition/text-field barrier. On macOS with the
native menu, a matching key release is a fallback for the swallowed key-down.
Other commands and unmodified/Option/AltGr input keep their existing ownership.
The native Quit route drains already-delivered canvas text through the existing
editing commands before deciding whether to prompt. Unfinished preedit remains
ephemeral. Duplicate Quit requests preserve an existing save/discard choice.
No dependency, vendor source, language mapping or unsafe code was added.

## Validation — 2026-10-10

- A synthetic macOS commit/key-release regression failed before the fix and now
  passes: prompt, one insertion, recovery, duplicate-request choice retention
  and Cancel/resumed typing.
- Two further regressions cover candidate/layout key ownership, clean Quit with
  unfinished preedit, and native-menu delivery of a same-pass commit with undo.
  All 16 composition tests pass.
- Native computer-use testing in build `uncommitted-P02.10` reproduced the old
  behavior, then verified the corrected first Cmd+Q displays Unsaved Changes.
  Cancel permitted another character; a second Quit and Save persisted the
  isolated fixture and exited. Native inventory showed Pagenor stopped; its
  shutdown log records retained windows, with no new Pagenor crash report.
- The native run used ABC-India and Udayam 36 pt. It is an automated native-input
  check, not a new human physical VariType test. Native Windows/Linux and broader
  input-method coverage remain milestone checks. WASM keeps its existing routing.
- Version remains 0.3.0. The signed local bundle at `/Applications/Pagenor.app`
  retains the distinct verified app/document icon bytes. Test edits used only
  `Shortcut check.pgnr`, a separate fixture copy. Changes remain local.

All seven gates pass: fmt, check, strict Clippy, workspace tests, assets, layers
and WASM. Total: **1,479 passing tests, 3 ignored, 20 WASM packages**.
