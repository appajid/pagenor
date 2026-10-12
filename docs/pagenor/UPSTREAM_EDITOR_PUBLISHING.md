# Pagenor upstream editor and publishing updates

Local integration, 2026-10-10. Application version: **0.3.0**.

Twenty-three selected upstream proposals are adapted individually rather than merging their
stacked branches. The eight UI changes were applied first, followed by compatibility and
multipage composition work. All source heads are pinned below. The upstream source uses
MIT OR Apache-2.0; its existing licence notices remain. Pagenor branding and owner artwork,
`.pgnr` saves, legacy reading, document-local fonts, lazy system-font discovery, named
instances, native clipboard handling, recovery, and `unsafe_code = "forbid"` remain.

## Behavior

- Quick Apply opens without executing its first item; rows align left. Menu items use their
  whole row. A6 is a print preset and appears among the welcome-page print presets.
- The Advanced object Control panel uses compact, fixed two-row groups. Horizontal scrolling,
  popup anchoring, keyboard accessibility labels, disabled controls and numeric spinners
  are covered by interaction tests.
- Shortcut recording waits for a non-modifier key. Typography, Interactive for PDF and
  Digital Publishing expose the relevant panels while retaining custom workspace support.
- Paragraph Rules has a dedicated dialog. Edits merge only changed fields with resolved
  paragraph/style rules. Errors reject the whole edit; applying after a document/selection
  change is refused. Existing Paragraph Style Options remains available.
- Menu popups, dialog bodies and preference panes scroll within the viewport at larger UI
  scales, with titles and action buttons outside the scrolling body.
- Windows custom caption buttons stay fixed outside the scrolling menu area. Minimize,
  maximize/restore, drag and resize use safe egui viewport commands. Close uses Pagenor's
  existing recovery checkpoint path; failure keeps an unsaved document open. macOS keeps
  its native traffic lights. Native Windows execution is still required at a milestone.
- Relocated native and IDML packages rebind missing embedded image links to the packaged
  Links folder without replacing embedded bytes. Existing original links stay authoritative.
  Lookup is bounded and image reads retain the asset-size limit. Document Fonts folder
  matching accepts case variants for Linux; the exact spelling wins.
- IDML layer import/export respects stacking order. Family format suffixes (OTF, TT, TTF,
  TTC, OTC, T1) resolve through the existing alias-aware scoped font metadata. An exact
  family name takes priority; menus do not load every font file.
- Lowercase blend-mode names apply as documented, with invalid values rejected. Word
  tracked move destinations import as accepted text. Explicit CLI help uses stdout and
  exit status 0; invalid commands still fail on stderr.
- Autoflow ignores break-shortened frames when estimating capacity. Smart Text Reflow
  preserves intentional page/frame deletions and runs at gesture commit. Predicted empty
  trailing pages are pruned in the same undo step, including after autoflow.
- Text after a spanning paragraph flows in a new band below it. Tab advance participates
  in wrapping; hanging-indent stops are honored. Justification affects only text after the
  final left tab, including the existing mojikumi/kashida behavior. Lines before column,
  frame and page breaks use last-line alignment.
- Break insertion/import ends the paragraph and supports odd/even page breaks. Older raw
  inline breaks remain supported; no native-format version increase is needed.
- Column rules use frame-level settings on screen, PDF and IDML, with editable colour,
  weight, tint, offset and insets. Object-style frame-option inheritance is retained.
- Table and cell styles resolve bounded inheritance and explicit zero/None overrides.
  IDML preserves region styles, per-side borders and edge priorities. Shared boundaries
  choose one winning edge. Row fitting accounts for baselines, descenders and inline objects.
  Unequal alternating stroke phases remain round-trip data, as in the source proposal;
  the renderer uses only supported uniform patterns rather than guessing their phase.

## Provenance

| PR | Author | Inspected source head | Scope |
|---|---|---|---|
| [#239](https://github.com/storytold/designcraft/pull/239) | @aiongg | `3751d0346c3466eaabfb105e9eee9ba22bd312fb` | UI: Quick Apply opens without running an entry; rows are left-aligned |
| [#249](https://github.com/storytold/designcraft/pull/249) | @moppeeer | `218dc42e5d47b6e6dfba52247dd77c34835b4cfa` | fix(ui): Make menu entries selectable across the full row |
| [#248](https://github.com/storytold/designcraft/pull/248) | @moppeeer | `84be68d9c204b172ae1d434eeb2abe7ade150ec5` | fix(ui): Add A6 sheet preset to the welcome page |
| [#255](https://github.com/storytold/designcraft/pull/255) | @moppeeer | `88dab50978d75cc71afb2d90792f65c4fde1c1c1` | fix(ui): Compact the Advanced object Control panel into aligned rows |
| [#264](https://github.com/storytold/designcraft/pull/264) | @aiongg | `0d491dbdd9c8340949189777f4bc0e2690a78245` | UI: the shortcut recorder waits for the key; Interactive for PDF and Digital Publishing open their panels |
| [#263](https://github.com/storytold/designcraft/pull/263) | @aiongg | `cbe960ce55c675e5bc22dfa90f62fd5dbd3711c3` | Paragraph Rules: a dialog for the selected paragraphs and a section in Paragraph Style Options |
| [#281](https://github.com/storytold/designcraft/pull/281) | @skylartaylor | `d5c1755329f00cb9b31f361de7aae8d64b561c62` | Keep menus and dialogs usable at larger UI scales |
| [#253](https://github.com/storytold/designcraft/pull/253) | @moppeeer | `1f53e115e5325153b8d5f6b5b02250214a2a3586` | fix(ui): Integrate Windows window controls into the menu bar |
| [#287](https://github.com/storytold/designcraft/pull/287) | @aiongg | `e1cef6ecc126d768503aebce41e0240a09bb2bb0` | Fonts: the Document Fonts folder is found whatever its case |
| [#275](https://github.com/storytold/designcraft/pull/275) | @kubuntugamer | `a2dbe793f3a6ce0fcc05e27709cb0c26bd8b5503` | fix(engine): object.opacity accepts its documented lowercase blend modes |
| [#257](https://github.com/storytold/designcraft/pull/257) | @thaynej | `c812c83d95b7055a9e5b1325c7be1523914d3b9e` | Word import: keep tracked moved text (w:moveTo) |
| [#254](https://github.com/storytold/designcraft/pull/254) | @kubuntugamer | `80c0adaaa3dfa71bf8ab091f9c1d0b217692f68b` | fix(cli): --help prints usage to stdout and exits 0 |
| [#244](https://github.com/storytold/designcraft/pull/244) | @DWAA1660 | `658d8e17842eb31ffe66cc615b163f33c3153ec9` | Rebind embedded image links when opening relocated packages (#235) |
| [#247](https://github.com/storytold/designcraft/pull/247) | @aiongg | `40eaaea9db06307872fa787168d7ce4a930c76c9` | IDML: layers import and export in stacking order; font names with a format suffix |
| [#266](https://github.com/storytold/designcraft/pull/266) | @thaynej | `7b3afb0b219244803363fa248ddb3fdd94e8923a` | Autoflow: add only the pages breaks ask for |
| [#241](https://github.com/storytold/designcraft/pull/241) | @noah1400 | `5e4fd17383464036d2623683a31fc32f28d2a1ee` | Smart Text Reflow: deleted pages and frames stay deleted |
| [#276](https://github.com/storytold/designcraft/pull/276) | @aiongg | `1443c3a8a86c62ee5bd46f76084850e5af6af396` | Text: text after a paragraph that spans columns flows below it |
| [#282](https://github.com/storytold/designcraft/pull/282) | @aiongg | `a33e597399f414a964fb0c85f65b9bd3fcba58ca` | Text: a line with a tab wraps at the column edge |
| [#283](https://github.com/storytold/designcraft/pull/283) | @aiongg | `54a4f167599b17e6ce9c6779a9843bc397a69b92` | Text: the line before a column, frame or page break is a last line |
| [#284](https://github.com/storytold/designcraft/pull/284) | @aiongg | `58388e6ec5049300aa0c8e5d4ded4409d78dd531` | Text: a justified line with a tab justifies the text after its last tab |
| [#285](https://github.com/storytold/designcraft/pull/285) | @aiongg | `69f8d0fcdd0fbded09356eb87893d2fe64ebe025` | Text: a column, frame or page break ends its paragraph; odd and even page breaks |
| [#274](https://github.com/storytold/designcraft/pull/274) | @aiongg | `7d0bdbc916a13b202add175f40a5f1eb559109f8` | Text frames: column rules on screen, in PDF and in IDML; Column Rules in Text Frame Options |
| [#288](https://github.com/storytold/designcraft/pull/288) | @skylartaylor | `7b15baa215a1951c4c7ef90b5c8525ed0b8ed728` | IDML: resolve table styles and correct cell height fitting |

## Validation

All seven local `CARGO_NET_OFFLINE=true cargo xtask ci` steps pass: fmt, workspace/all-target
check, strict Clippy, tests, assets, layers and WASM. **1,443 tests pass, 3 existing tests are
ignored, and all 20 WASM packages check.** The UI library has 193 passing tests and one of
those existing ignores. Pagenor's full engine suite has 407 passing tests; composition has
209 passing tests and its existing corpus-dependent ignore. IDML has 54 passing tests.

CPU renders were inspected for A6/welcome, the two-row Advanced object controls, Quick Apply,
Paragraph Rules, column rules on the canvas and their options dialog, Preferences at 150%
scale, and fixed Windows captions at 900 × 560. Pointer tests cover scrolling, popup clipping,
caption actions, workspace changes, sparse rules, numeric grids and unsaved-close failure.
The CPU harness supplies logical viewport dimensions adjusted for UI zoom before rasterizing
at the output pixel scale; it does not establish GPU or native Windows execution.

Additional combined-integration regressions cover page-start options with multiple frames
on one page and section-number parity, cached/cold layouts, gesture completion, intentional
page/frame deletion, reflow after undo, same-step pruning of predicted empty pages,
package relocation without pixel replacement, partial table/cell inheritance, explicit clears,
shared edges, minimum heights and legacy/full-style native round trips. Existing font,
clipboard, recovery, data-merge, RTL/CJK/vertical layout and file-limit regressions also pass.
The reflow fixtures use a hidden half and a 60 pt frame to exercise an actual capacity change
with Pagenor's font metrics. The IDML default-fill assertion now checks resolved table output
because style records retain partial attributes for inheritance.

Published GitHub v0.3.0 source/tag/assets are unchanged. Native Windows/Linux GUI, installer
and browser execution remain host/milestone validation; no new native-platform execution
claim is made. The macOS development build passes via `packaging/macos/dev.sh --no-launch`; bundle
plist validation and strict/deep ad-hoc signature verification pass. It is available at
`target/macos-dev/Pagenor.app`. Build metadata is explicitly `uncommitted-P09.7`, not a
published source revision. This is a local development bundle, not a signed/notarized release.
It has not been launched as part of this task; CPU UI verification is separate evidence.

Deferred review items remain deferred: Italian translation, the Affinity/theme drafts,
additional distribution targets, and macOS Finder/Dock handling that would violate the
unsafe-code policy. Native Windows/Linux GUI and installer execution require their hosts.
Adobe/dongle activation validation remains cancelled by the owner.
