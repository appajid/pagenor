# Canvas keyboard input — P02.7 / P02.8

Pagenor accepts the text produced by the operating system's selected input source.
There is no list of permitted languages or keyboard layouts. Direct Unicode text,
dead-key composition and input-method commits use the same existing `text.insert`
command, so selection replacement, typing styles, document font scopes, undo and
recovery remain in the engine.

## Cause and implementation

The custom canvas handled `egui::Event::Text` but did not publish an IME output or
handle `egui::Event::Ime`. In the pinned egui-winit 0.36.2 / winit 0.30.13 macOS
integration, an IME output enables the native text service; without that flag,
macOS skips `interpretKeyEvents`. Layouts using actions/dead keys consequently
could not finish normal native composition on the canvas.

The canvas now enables that service at the focused text caret, including table
cells and notes. Both host rectangles use the caret because the pinned native
integration positions candidates from `IMEOutput.rect`. An overset/empty caret
has a safe canvas fallback. Only the active split-view pane accepts keyboard text.

`canvas_ime.rs` keeps unfinished preedit in ephemeral UI state. P02.8 removes the
separate below-caret preview box: unfinished text appears directly in the normal
paragraph flow at the selected font, size, colour and position. The visual caret
and native candidate anchor follow that text. Contextual shaping, selected-text
replacement and line wrapping use the existing compositor and renderer.

The engine applies the existing insertion command to an isolated copy-on-write
document, preserving pending typing formatting and document-scoped fonts. The
result is cached until preedit changes. Fonts, assets and unchanged collections
remain shared. Automatic page creation/deletion and typographer substitutions
are deferred until commit. Preedit never enters the live document, save data,
journal or undo history. A nonempty
commit calls `text.insert` once. Cancellation does not remove selected text.
Composition keys belong to the input method, so candidate arrows, Backspace,
Escape and Enter do not also perform canvas editing or global shortcuts.
Option/AltGr-generated text takes priority over a matching Alt shortcut.

Canvas focus is requested only when it is not already held. In egui 0.36.2,
even re-requesting the same focus sets `should_interrupt_composition`; doing
that every paint cancels the OS's dead-key state. A regression reproduces this
failure before the correction and verifies continuous commit/new-preedit
batches, retained composition across repaint and candidate navigation, and an
advancing UTF-8 caret afterwards. Tab, arrows and Escape retain canvas focus
while text editing so egui focus navigation does not interrupt the text service.

Composition is invalidated when the document, revision, selection, pane, viewport
or keyboard focus changes. Queued commits for an invalidated target are discarded.
Dialogs, Home and focused UI text fields retain their input ownership. Surrounding
deletion counts Unicode characters and uses the existing edit command.

Preedit is limited to 16 KiB and a commit to 1 MiB. Inline text uses the existing
bounded viewport rendering and texture patching rather than a second preview
page or fixed-size font.
Font discovery, document serialization and shaping are unchanged. The selected
font still needs appropriate glyph coverage; a keyboard layout does not provide
its own font or replace shaping support.

## Local validation — 2026-10-10

Eleven UI regressions cover native-service output, the exact sample
`ఉదయం ఫాంట్‌` (including U+200C), selected-text replacement, single commit,
confirmation without an extra paragraph, undo/redo, cancellation, direct
multilingual text, language switching, stale commits, Alt/Option text, Unicode
surrounding deletion, split views, Home, input bounds and focused text fields.
Two P02.8 regressions additionally verify 36-point pending typing formatting,
retained inline allocation on repaint, identical document/caret geometry on
commit, paragraph wrapping, selected-text replacement and cancellation restoring
the canvas. All seven local quality gates pass; the workspace has 1,473 passing tests and
three ignored tests, and all 20 WASM packages check successfully.

The macOS development bundle was built and launched. macOS preferences identify
the selected input source as `AdityaFonts.com-VariType(Apple)`; its installed
layout uses 55 composition actions. P02.7 corrected the reproduced per-paint
focus interruption and established continuous physical Telugu typing in Udayam
at 36 points. The owner then reported the current character sitting below the
line in the old preview box. P02.8 removes that box. On 2026-10-10 the owner tested
the rebuilt bundle with physical VariType typing and confirmed: “Yes, it stays
in the line.” Native undo/redo and candidate-list variants were not separately
exercised beyond the automated tests.

The macOS development launcher now checks executable mappings (`lsof -a -d txt`)
when preventing a running bundle from being replaced. An ordinary icon reader
had kept the old executable open after exit, causing a false rebuild refusal.
The new check permitted the stopped-app rebuild and detected the relaunched
Pagenor process. Shell syntax, bundle signing and build checks pass.

A separate earlier native Quit produced a winit 0.30.13 `set_marked_text` callback
panic after its window disappeared (`view.rs:835`, “view to have a window”).
P02.9 fixes this host teardown path locally with approved-close input suppression
and safe AppKit deactivation/window retention. Shutdown regressions and native
saved-window Close with unfinished ABC-India dead-key input pass; this does not
claim a new physical VariType quit check. See
[native shutdown evidence](NATIVE_INPUT_SHUTDOWN.md).
The vendor layout and font files are neither copied into the repository nor
modified.

The implementation is shared Rust UI code with no platform-specific text mapping
or unsafe code. Native Windows/Linux IME and keyboard-layout execution, browser
IME execution, long candidate lists and OS-specific candidate positioning remain
milestone-host checks. Pagenor remains 0.3.0; no remote changes were published.
