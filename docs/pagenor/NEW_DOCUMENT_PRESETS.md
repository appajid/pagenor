# New Document presets — P09.12

Pagenor adapts upstream pull request #325 in its shared Rust UI and engine.
Version remains **0.3.0**. Changes are local; no GitHub publication is part of this task.

## Use

Choose File → New → Document, Cmd+N on macOS, or Ctrl+N on Windows/Linux.
Welcome's New file button opens the same window. Recent shows factory defaults
and up to twelve recently used sets of New Document settings. These are separate
from the existing recent-file thumbnails on Welcome and File → Open Recent.

Print, Web and Mobile show the existing built-in document presets as proportioned
page cards. Click one to fill the details; double-click to create its document.
Details include document name, width/height, units, portrait/landscape, page count,
starting page number, facing pages, primary text frame, columns and gutter.
Margins, bleed and slug can be linked or edited per edge. Facing pages use
Inside/Outside; otherwise the controls show Left/Right. Changing units reformats
lengths without changing them. Fields have arrow buttons and accept Up/Down keys.

The save-preset button beside Document Name opens a name popup. Enter saves the
preset and leaves New Document open. Escape closes the popup first; a second
Escape closes New Document. Saved names replace case-insensitively, with built-in
names protected. Saved-tab cards offer Delete Preset in their context menu.
Presets and recent settings use the existing preference persistence at app exit.
Create uses the existing document engine. An out-of-range page size keeps the
window and entered details available for correction.

Small windows stack details above presets and retain scrolling. The right-to-left
interface mirrors the two-pane layout. PDF/PPTX Export and Find Font retain their
existing specialized confirmation/cancellation labels.

## Commands and bounds

- `file.savePreset {name, …settings}` saves/replaces an engine preference preset.
- `file.deletePreset {name}` removes a saved preset, reporting unknown names.
- `file.presets` lists built-in and saved presets (`saved: true` for the latter).
- `file.new {preset: name, …overrides}` uses a saved or built-in preset.

Names are limited to 100 characters; at most 500 presets may be saved. A failed
capacity check preserves the previous list. Loaded page/column/start counts and
edge widths are bounded, page counts clamp before a 32-bit cast, page dimensions
and other geometry are checked for finite values, and the untitled counter cannot
overflow. Engine errors propagate through the shared command route.

A native macOS check exposed first-frame popup layering after the parent dialog
was interacted with. The popup is explicitly ordered directly above its modal
parent. A pointer/layer regression covers reopening after preset selection and
clicking the settled name field.

## Provenance and scope

See [PAGENOR_UPSTREAM.md](../../PAGENOR_UPSTREAM.md) for the pinned #325 head,
author and licensing. Seven reused Rust source-file blob identities were verified.
The latest upstream Enter/Escape and engine-preset corrections are included.
Italian translations are deferred per the owner. No dependencies, unsafe blocks,
platform adapters or native document-format changes are introduced. Welcome
thumbnails, scoped lazy fonts, clipboard, inline composition, recovery, staged
saving, branding and separate app/document icons remain in their existing paths.

## Validation

All seven final local gates pass: **1,500 tests passed, 3 ignored, zero failed;
20 WASM packages**. The signed Mac app is installed at `/Applications/Pagenor.app`
as `uncommitted-P09.12`, version **0.3.0**. Native A4 selection, popup layering and
Escape, two-page creation starting at 10 with primary text frames, Save, Recent
recall and actual quit/restart persistence pass. Saved archive settings and shared
primary story were independently verified. App/document ICNS hashes match their
previously verified distinct assets. The app is left at Welcome with no document
open. Owner documents were not edited; no QA saved preset was added to preferences.

Shared engine tests
cover saved preset commands, persistence round-trip, setting overrides, intent,
units, page numbering, bleed/slug, capacity failure and hostile loaded settings.
UI tests cover real preset selection/double-click, linked fields, unit conversion,
popup Enter/Escape/layering, correction after failed Create, resizing/scaling and
Arabic field reachability. Existing Welcome and save/input regressions remain.

The offscreen wgpu harness has no available GPU adapter in this session. A scratch
CPU renderer produces review images from the actual egui mesh/texture output;
native macOS interaction provides the application check. Native Windows/Linux
execution remains a milestone task. No Preview or template-search feature is
introduced by this proposal.
