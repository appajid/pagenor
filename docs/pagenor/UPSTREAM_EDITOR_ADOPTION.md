# P11 — editor and publishing upstream adoption

Owner-authorized queue, 11 October 2026. Work proceeds locally, one change at a time. The complete selected queue is implemented and verified as a **0.5.0 local development checkpoint**. This is not a signed/notarized cross-platform release. CJK-specific work and Italian remain deferred. Existing owner branding, document icons, .pgnr compatibility, lazy native fonts, document scopes and inline IME are retained. #400 is excluded because it relaxes the unsafe-code policy.

| PR | Change | State |
|---|---|---|
| [#447](https://github.com/storytold/designcraft/pull/447) | UI: Return and Escape in the font menu stay in the menu, never reach the dialog under it | Implemented; full local gates passed |
| [#399](https://github.com/storytold/designcraft/pull/399) | fix(ui): Color Picker keeps the typed hex text and commits typed RGB values (#323) | Implemented; full local gates passed |
| [#388](https://github.com/storytold/designcraft/pull/388) | fix(ui): Show the Pencil stroke while drawing | Implemented; full local gates passed |
| [#408](https://github.com/storytold/designcraft/pull/408) | Align panel: code-drawn icons for align and distribute | Implemented; full local gates passed |
| [#446](https://github.com/storytold/designcraft/pull/446) | UI: document views have scrollbars, and Alt pans while editing text | Implemented; full local gates passed |
| [#373](https://github.com/storytold/designcraft/pull/373) | UI: a context menu on the canvas for objects, text and the page | Implemented; full local gates passed |
| [#416](https://github.com/storytold/designcraft/pull/416) | Swatches and layers: Swatch Options and Layer Options dialogs rename them | Implemented; full local gates passed |
| [#378](https://github.com/storytold/designcraft/pull/378) | Layers: drag layers and objects to reorder them | Implemented; full local gates passed |
| [#415](https://github.com/storytold/designcraft/pull/415) | Tools: move a placed graphic inside its frame | Implemented; full local gates passed |
| [#444](https://github.com/storytold/designcraft/pull/444) | Tools: live corners round a frame's corners by dragging | Implemented; full local gates passed |
| [#409](https://github.com/storytold/designcraft/pull/409) | Adopt shared panel docking and preserve workspace commands | Implemented; full local gates passed |
| [#445](https://github.com/storytold/designcraft/pull/445) | macOS: Settings… is in the app menu | Implemented; full local gates passed |
| [#442](https://github.com/storytold/designcraft/pull/442) | macOS: the window buttons sit centred in the title bar | Implemented; full local gates passed |
| [#448](https://github.com/storytold/designcraft/pull/448) | feat(theme): add System interface appearance | Implemented; full local gates passed |
| [#393](https://github.com/storytold/designcraft/pull/393) | UI: no crash on textures past the GPU limit | Implemented; full local gates passed |
| [#434](https://github.com/storytold/designcraft/pull/434) | Objects: group siblings inside a group; refuse mixed levels and spreads | Implemented; full local gates passed |
| [#411](https://github.com/storytold/designcraft/pull/411) | Undo: a command that runs another is one undo step | Implemented; full local gates passed |
| [#420](https://github.com/storytold/designcraft/pull/420) | Assets: one asset per image, and saves leave out unused ones | Implemented; full local gates passed |
| [#458](https://github.com/storytold/designcraft/pull/458) | Preserve editorial clipboard payloads and retrieve text before menu Paste | Implemented; full local gates passed |
| [#305](https://github.com/storytold/designcraft/pull/305) | Preserve explicit nullable text defaults when reopening documents | Implemented; full local gates passed |
| [#456](https://github.com/storytold/designcraft/pull/456) | Wrap saved story revision counters consistently | Implemented; full local gates passed |
| [#457](https://github.com/storytold/designcraft/pull/457) | Preserve table dash patterns in IDML and render dotted borders | Implemented; full local gates passed |
| [#402](https://github.com/storytold/designcraft/pull/402) | Place: an explicit x, y, spread or width puts the image in a new frame | Implemented; full local gates passed |
| [#403](https://github.com/storytold/designcraft/pull/403) | Text: a character fill must name a swatch in the document | Implemented; full local gates passed |
| [#369](https://github.com/storytold/designcraft/pull/369) | Paste: copied objects land on the spread in view | Implemented; full local gates passed |
| [#379](https://github.com/storytold/designcraft/pull/379) | Tables: cell edge strokes in the Stroke panel and Cell Options, with gap colour and tint | Implemented; full local gates passed |
| [#359](https://github.com/storytold/designcraft/pull/359) | UI: Paragraph Style Options gains the remaining sections | Implemented; full local gates passed |
| [#360](https://github.com/storytold/designcraft/pull/360) | Paragraph borders, shading, span space and list levels: every setting from model to dialog | Implemented; full local gates passed |
| [#381](https://github.com/storytold/designcraft/pull/381) | Layout: text wrap set on a placed graphic wraps text | Implemented; full local gates passed |
| [#384](https://github.com/storytold/designcraft/pull/384) | Layout: wrap both sides of an object and around its contour | Implemented; full local gates passed |
| [#433](https://github.com/storytold/designcraft/pull/433) | PDF: items with soft effects keep their text wrap | Implemented; full local gates passed |
| [#422](https://github.com/storytold/designcraft/pull/422) | PDF: generated text keeps its characters under small caps | Implemented; full local gates passed |
| [#421](https://github.com/storytold/designcraft/pull/421) | PDF: right-to-left text extracts in reading order | Implemented; full local gates passed |
| [#427](https://github.com/storytold/designcraft/pull/427) | Text: Optical kerning spaces pairs from their outlines | Implemented; full local gates passed |
| [#372](https://github.com/storytold/designcraft/pull/372) | Compose: No Break runs and nonbreaking spaces stretch on justified lines | Implemented; full local gates passed |
| [#357](https://github.com/storytold/designcraft/pull/357) | Text: the paragraph mark's size counts in its line's auto leading | Implemented; full local gates passed |
| [#418](https://github.com/storytold/designcraft/pull/418) | IDML: frames take corners, effects and text wrap from their object style | Implemented; full local gates passed |
| [#419](https://github.com/storytold/designcraft/pull/419) | IDML: gradient and directional feathers import and export | Implemented; full local gates passed |
| [#356](https://github.com/storytold/designcraft/pull/356) | IDML: part names match leniently, and missing parts are reported | Implemented; full local gates passed |
| [#362](https://github.com/storytold/designcraft/pull/362) | IDML: `$ID/` font names, placed PDF page and crop, package IDML without embedded images | Implemented; full local gates passed |
| [#382](https://github.com/storytold/designcraft/pull/382) | Layout: right-to-left binding puts parent page items on the right side | Implemented; full local gates passed |
| [#397](https://github.com/storytold/designcraft/pull/397) | macOS: the menu bar follows the interface language at once | Implemented; full local gates passed |
| [#297](https://github.com/storytold/designcraft/pull/297) | Native GPU startup prerequisite | Implemented; full local gates passed |
| [#380](https://github.com/storytold/designcraft/pull/380) | Startup: pick the GPU that drives the display, fall back across adapters and backends, and log startup errors | Implemented; full local gates passed |
| [#452](https://github.com/storytold/designcraft/pull/452) | M6: Adopt craft-print and support Windows PDF printing | Implemented; full local gates passed |
| [#455](https://github.com/storytold/designcraft/pull/455) | CI: FreeBSD and Windows ARM64 nightly and on release, not per PR | Implemented; full local gates passed |
| [#374](https://github.com/storytold/designcraft/pull/374) | feat(i18n): apply russian language | Implemented; full local gates passed |
| [#295](https://github.com/storytold/designcraft/pull/295) | Split-column layout prerequisite for Paragraph Style Options | Implemented; full local gates passed |
| [#298](https://github.com/storytold/designcraft/pull/298) | List expressions and empty-paragraph numbering prerequisite | Implemented; full local gates passed |

## Adaptation notes

- #447: font popup consumes navigation/Return/Escape and exposes ComboBox accessibility data. Implemented; UI checks passed.
- #408: code-drawn alignment/distribution icons use existing Pagenor icon painters. Implemented; UI checks passed.
- #388: Pencil preview follows pointer in active-layer colour. Implemented; tool/UI regression checks passed.
- #399: closed upstream patch inspected along with maintainer feedback. Unlike its initial version, hex text commits/reverts on focus loss or Return, avoiding partial shorthand recolouring. RGB rounding retained. Implemented; UI checks passed.

## Validation

Each implementation was adapted against the existing dirty working tree, which was backed up separately. Original contributor registries were retained. Exact upstream heads, contributor credits and local differences are recorded in PAGENOR_UPSTREAM.md. The final versioned build follows the full local checks below.


The first ten editor changes passed 275 UI tests (one existing ignore), focused tool/engine regressions and context-menu clipboard checks. The full integrated workspace gates, native build and visual review are recorded below.

- #446 preserves inline composition and ruler-origin behavior; all four scrollbar/Alt-panning checks pass.
- #373 retains Duplicate/Delete and composition/overlay guards; native Unicode Paste and undo regressions pass.
- #416 retains paragraph-rule actions, bounded swatch migration and shared text/object colour routing.
- #378 retains deferred Layer Options alongside strict sibling reordering.
- #415 retains frame fitting and supports both native counted clicks and synthetic DoubleClick.
- #444 uses one undo step per live-corner gesture; paint tests inspect widget transitions without requiring a headless GPU.
- #409/#445/#442/#448 preserve Welcome, recent documents, safe preferences recovery and cancellable save-all Quit.

- #379: table edge selection, gap colour/tint and Cell Options; retained concrete IDML dash patterns and bounded dash expansion.
- #359/#360: complete paragraph style and local paragraph sections with borders, shading, corners, list levels and list glyph styling. Preserved existing Paragraph Rules safeguards, bounded Tabs and inline input. #295 split-column layout and #298 list expressions are required prerequisites.
- #420/#458: independent linked sources keep their own relink identity; same-source placements share assets. Anchored clipboard copies receive fresh object identities. Regression fixtures still test archive limits after unused assets are removed.

- #381/#384/#433: content, grouped and contour wraps share the bounded exclusion collector; both-side segments retain Jump Object and grid behavior. Soft-effect PDF rasters retain wrapping. Contour point allocation stops at the existing cap.
- #422/#421/#427/#372/#357: generated small-cap text metadata, RTL extraction order, outline-based optical kerning, No Break justification and terminal paragraph-mark leading. Retained drop-cap hits, empty-paragraph metrics, existing spacing and decoration artifacts.
- #418/#419/#356/#362: merge inherited effects without losing cycle-safe style chains, legacy corner defaults or complete frame options; feathers, lenient package parts, PDF crop/page metadata and `$ID/` font normalization. System font enumeration remains lazy and scoped. Packaging preflights exports and links IDML images without embedding copies.
- #382: RTL page binding maps parent items to the proper side.
- #397: macOS menus rebuild immediately for language changes and retain recent documents and the save-all quit path.
- #452: shared printing boundary preserves PDF export options/crop geometry and surfaces spooler errors; no real print job is submitted by tests. Windows execution remains a platform validation requirement.
- #455: FreeBSD/Windows ARM64 nightly/release gates plus packaging-triggered PR checks. Pagenor's font-specific FreeBSD PR coverage is retained.
- #374: Russian captions and contextual/count forms are integrated; no Italian or unrelated Ukrainian/CJK features are pulled in.


## Integration corrections

- Register the existing Tabs panel with shared docking, preserve its minimum width and frame-relative placement, and retain the open/close shortcut.
- Route both legacy `rule` and new `rules` Paragraph Style Options sections to the existing rule controls.
- Keep the existing translations when newly imported sections overlap their keys. Russian menu captions are validated through their actual lookup, including captions absent from the legacy multilingual table. Remove unrelated upstream product links.
- Make the expanded Paragraph Style Options content scrollable; identify the tab ruler in accessibility metadata and test clicks against its actual rectangle.
- Update fixtures for intentional image asset sharing, strict character swatch names, nullable native-format migration and the new table gap-colour fields.

- Remember failed OpenGL in-process fallback on macOS even when Metal is the only primary candidate; the startup regression verifies it is excluded on retry.
- Shared-asset link listings expose every frame use. Package test paths are canonicalized so macOS `/var` aliases do not weaken redirected-directory refusal.
- Graphics error help targets Pagenor's own repository and documented overrides. Tabs closes fully from its command and keeps a frame-relative floating position under shared docking.

- Restore composite-font identity resolution alongside `$ID/` normalization, retaining the existing composite/footnote IDML round trip. This preserves baseline behavior without adopting deferred CJK proposals.
- Hide idle live-corner widgets during loaded threading and marquee gestures; keep their drag controls active during corner editing.
- Retain separate relink identity for byte-identical files with different source paths in IDML; multiple placements of the same source still share one asset.
- Save-corruption fixtures reference their asset so compaction retains it. Nullable-default fixtures compare both current ZIP and fully migrated legacy JSON documents.

- Fix the height of the paragraph style section container while keeping long content scrollable. Keep the paragraph section viewport bounded and scrollable. Wrap long checkbox captions so their complete wording remains visible. Keep the shutdown-specific clipboard guard native-only and omit the unavailable native script-file picker on WASM. Pointer regressions wait for modal sizing before reading widget rectangles, then test font-menu search, tab alignment and ruler editing.

## Completed checkpoint — 0.5.0

All seven `cargo xtask ci` gates pass: 1,856 tests passed, 6 ignored, no failures; 20 WASM packages. The 12 packaging contracts, ShellCheck and actionlint pass. The local macOS app is built and inspected separately. Windows/Linux native GUI and printing execution, signed/notarized distribution and GitHub publication remain milestone work.

All 47 selected non-CJK proposals and the two required prerequisites (#295/#298) are adapted. The separately credited #271 Linux portal implementation supports the printing boundary. Source commits, contributors and local adjustments are recorded in PAGENOR_UPSTREAM.md. New CJK work, Italian and #400 remain excluded. The native document format remains **3**, and the Rust workspace retains `unsafe_code = "forbid"`. Existing stronger Pagenor font, save, clipboard, IME and provenance behavior is retained.

Inspected actual egui renders: Welcome, editor, floating/docked panels, paragraph Tabs, Basic Character Formats and Paragraph Border. The app bundle carries separate owner-supplied application/document icons and 0.5.0 version metadata.

The final local 0.5.0 app is installed at `/Applications/Pagenor.app`; its executable matches the packaged build byte-for-byte. Strict/deep signature checks, 0.5.0 executable/plist versions and separate supplied app/document icon resources pass. The previous 0.4.0 app is retained in a recoverable local backup. No GitHub publication.
