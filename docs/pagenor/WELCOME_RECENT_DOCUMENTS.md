# Welcome and recent documents (P09.9–P09.10)

The Welcome screen shows **Recent Documents** below the new-document presets.
Recent entries use 120 × 150 tiles, matching the preset boxes, with a first-page
thumbnail and filename. Full paths appear on hover. About Pagenor sits below the
large branding image on the right; its previous surrounding card is removed.
Narrow windows stack the artwork and button after the scrollable document tiles.

Saved and opened local files remain listed after restarting Pagenor. Newly created
unsaved documents appear while they are open; their entry activates the existing
tab. Saving replaces the unsaved entry with the file. Save As keeps the previous
file in history too. Reopening a recent document already in a tab activates it
without opening a second copy.

**File → Open Recent** uses the same list, including the native macOS menu.
The Home button returns to Welcome without closing documents. Welcome hides the
editor's tools, control/status bars, docked/floating panels and Story Editor.
Opening or creating a document restores the existing panel arrangement.

Quit offers **Save**, **Don't Save**, and **Cancel** for
each modified or never-saved document. Already saved, unchanged documents need
no question. Cancelling a save picker or a failed save keeps the document and
stops quitting. The existing recovery checkpoint/failure checks remain in place;
explicitly discarded documents remove their own recovery entries.

The document window close button closes only the active document through the
same unsaved-changes question as File > Close and the tab ×. It keeps the workspace
alive, returning to Welcome after the last document; closing an empty workspace
keeps Welcome visible. Explicit Quit still prompts for all unsaved documents
and exits. File/tab closing renders Welcome immediately when no documents remain.

History is limited to 20 entries, newest first, with duplicate local paths
removed. Native command boundaries resolve absolute/canonical paths; Windows
path comparisons ignore ASCII case. Drawing the menu reads neither font files
nor document files. History persists paths without session IDs. Welcome also
caches up to 20 first-page PNG previews in local preferences, each bounded to
160 pixels per side; it never persists whole documents. A single background
worker renders previews through the existing importer, document font scopes and
page renderer. Unchanged cached files avoid reopening and rendering. Modified
open documents get fresh runtime previews without replacing their saved-file
cache. Closed-file backfill reads at most 64 MiB; larger documents get previews
when opened. Missing previews use a paper placeholder. Missing/moved files show
the existing open-error dialog. Imports retain their source path until saved as
a Pagenor document. Browser documents
remain session-local because a download name cannot reopen a browser file.

The shared Rust implementation covers desktop macOS, Windows and Linux and
preserves the WASM build. Version remains **0.3.0**. Native Windows/Linux
execution remains a separate milestone check; changes stay local.

## P09.9 local validation (2026-10-10)

All seven `cargo xtask ci` gates pass: 1,458 tests passed, 3 existing tests
ignored, and 20 WASM packages checked. Eleven new regressions cover history
ordering/persistence/deduplication, failed and cancelled operations, imports and
recovery, narrow-window scrolling, Home/editor transitions, multi-document quit,
cancelled/failed saves and Escape cancellation. Existing shutdown/recovery tests
now require the save question before accepting a close.

The rebuilt 0.3.0 macOS development bundle was inspected through native UI:
Welcome hides tools/panels, saved files survive restart, File → Open Recent
lists the same documents, and opening a document restores its editor controls.
The native Save dialog was observed during quitting. Windows/Linux GUI execution
and OS termination outside the cancellable application close path were not
tested in this task.

## P09.10 local validation (2026-10-10)

Four new regressions cover actual first-page artwork, bounded preview preference
round-trips, cache reuse and refresh after saving, stale background results,
bounded file reads without changing open tabs, and the right-side About button
and matching tile sizes. All seven local `cargo xtask ci` gates pass: 1,462 tests
passed, 3 existing tests ignored, and 20 WASM packages checked. The rebuilt
0.3.0 macOS development bundle shows actual magazine and text-document thumbnails;
the relocated About button opens its dialog correctly. Native Windows/Linux
execution remains separate. Changes stay local.

## P11.2 maintenance correction — 2026-10-11

Version remains 0.5.1. Native inspection reproduced a black window after the
document window close button: it entered approved shutdown, which stops UI
drawing, instead of closing the active document. The shared desktop close guard
now cancels viewport termination, handles each window-close event only once per
frame, checkpoints through the existing recovery guard, and uses the existing
document-close command and question. Empty workspaces continue displaying Welcome.
Explicit Quit and its save-all/recovery/native-input teardown remain separate.

Closing the final tab also requests a fresh layout immediately; Welcome clears
ephemeral canvas composition and hides editor tools and panels. The right-side
icon rail is 44 logical pixels wide in both legacy and shared docks, with 32-pixel
buttons and matching flyout placement.

All seven `cargo xtask ci` gates pass: 1,867 tests passed, 6 existing cases ignored,
no failures; all 20 WASM packages checked. Four added regressions cover immediate
Welcome rendering and editor reopening, the native window-close render path,
single-document handling despite repeated host calls, and active-document
save/cancel/discard behavior. Existing recovery and IME tests now verify that a
document window close disables canvas input and retains Welcome; explicit Quit
continues using its existing shutdown checks. Physical Windows/Linux execution
remains milestone work; no GitHub publication.

The arm64 Mac development bundle passes plist and strict/deep ad-hoc signature
checks. Native inspection confirms the wider icon bar; actual egui renders show
Welcome after the final document closes. The native close-button interaction
was left unfinished when the owner requested that the current session stay open.
The closed /Applications/Pagenor.app copy was backed up and updated separately,
with build ID local-0.5.1-welcome-close, matching executable bytes and distinct
owner-supplied app/document icons. The running development session was preserved.
