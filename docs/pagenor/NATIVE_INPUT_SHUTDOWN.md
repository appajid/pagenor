# Native input shutdown — P02.9

Pagenor 0.3.0 now stops canvas input once desktop closing is approved. macOS
also deactivates its text input context before releasing its native window.
Save/discard/cancel prompts and final recovery retain their existing behavior.

## Reproduced failure and cause

PG-0215 records the native 2026-10-10 15:10:56 crash. The macOS report shows
`NSWindow dealloc` → input-context deactivation → dead-key session clearing →
`WinitView::set_marked_text`. The pinned winit 0.30.13 view holds a weak native
window reference; its event delivery expects that reference to remain alive.
The callback during deallocation violated that assumption and aborted across
an Objective-C callback boundary. Catching Rust panics would not fix this.

A new UI regression fails before the change: after Quit is accepted, the canvas
still publishes an IME output, re-enabling native input in the closing frame.

## Focused implementation

The shared desktop UI records approved closing, clears ephemeral composition,
publishes `IMEAllowed(false)` and stops processing canvas input. This prevents
the final UI pass from re-enabling the service and prevents late queued commits
from editing a closing document. Unfinished preedit remains ephemeral; it is
cancelled rather than serialized or added to undo history. Committed edits still
go through the existing save/recovery flow.

The macOS host uses safe objc2/AppKit bindings, already present transitively, as
direct target-specific dependencies. Its exit callback retains native windows
first, discards marked text, deactivates input contexts and releases the first
responder. A shared ownership guard in `main` keeps the native windows alive
until `eframe::run_native` returns. AppKit can deliver a synchronous shutdown
callback while its client window still exists. The guard is then released.
No windowing-library fork, dependency source edit, new unsafe code or keyboard
mapping is introduced. Windows/Linux share the approved-close UI behavior;
the AppKit adapter compiles only for macOS. Browser input behavior is unchanged.

## Validation — 2026-10-10

- The failing-before/passing-after regression covers IME output on approved Quit,
  an unchanged saved document and rejection of a late native commit.
- A second regression covers clean root-window Close and cancelling Quit followed
  by successful Unicode typing. Existing save-all, recovery-failure/retry and
  composition regressions pass.
- All seven quality gates pass: fmt, check, Clippy, tests, assets, layers and WASM.
  Total: **1,476 passing tests, 3 ignored, 20 WASM packages**.
- The ad-hoc-signed bundle is built, verified and installed at
  `/Applications/Pagenor.app`, version **0.3.0**, build `uncommitted-P02.9`.
  Installed app/document ICNS bytes match the previously verified separate icons.
- Native Quit displayed the save prompt for edited work. Cancel returned to the
  document; typing and saving continued. A subsequent saved-document window Close
  with `Option+E` composition unfinished exited successfully. Native inventory
  showed Pagenor stopped; the shutdown log recorded retained windows; no new
  Pagenor crash report appeared. Test edits used a separate document copy; the
  owner's original `Untitled-1.pgnr` hash is unchanged.
- This native run used the currently selected **ABC-India** input source. It does
  not constitute a new VariType physical-key validation. The earlier owner's
  successful inline VariType confirmation is preserved in `KEYBOARD_INPUT.md`.
  Native Windows/Linux shutdown and broader input-method/stress coverage remain
  milestone-host checks.

During the native dead-key check, an injected Cmd+Q first completed composition
without quitting; choosing Quit from the native menu worked. PG-0216 records
this accelerator-delivery observation separately. P02.10 now reproduces and fixes
it locally; see [Quit during composition](QUIT_SHORTCUTS.md) for current evidence.

Signing initially failed with disk space nearly exhausted. Only regenerable
`target/debug/incremental` build cache was cleared; the rebuilt bundle then
signed and verified successfully. No documents, fonts or source files were
removed. No GitHub changes were published.

Relevant API references: Apple's
[discardMarkedText](https://developer.apple.com/documentation/appkit/nstextinputcontext/discardmarkedtext())
and [deactivate](https://developer.apple.com/documentation/appkit/nstextinputcontext/deactivate()),
and the pinned local winit/eframe/AppKit binding sources used in the investigation.
