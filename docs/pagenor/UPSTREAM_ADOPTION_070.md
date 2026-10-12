# P11.13 — next upstream adaptation batch

Owner authorised implementation on 12 October 2026, with no routine confirmation pauses.
Start source: `f9c469fab92f022a3bb31c7e4f8d803ecf610d73`.
Work is local on `development/upstream-20261012`; published 0.6.0 artifacts remain intact.
The next completed feature batch will receive a new version before packaging.

The queue is 26 selected new proposals and six incremental updates, after the review
through #565. CJK, Italian, optional language patterns and SEA dictionary integration
remain deferred; already-covered features are not rewritten. The owner Mac session
and proprietary documents/fonts are preserved. Rust layering, unsafe_code=forbid,
scoped/lazy fonts, inline Indic input, .pgnr/legacy documents and branding remain required.

## Queue and pinned provenance

| Task | PR / source author | Inspected source head | Status |
|---|---|---|---|
| P11.13.1 | [#411](https://github.com/storytold/designcraft/pull/411) / @noah1400 | `60f463f03cbe35396e49f3601c42fdf46d662fc2` | Complete — all seven gates passed |
| P11.13.2 | [#524](https://github.com/storytold/designcraft/pull/524) / @moppeeer | `191594897be1c3225b5b3661b848fb97b036c6ba` | Complete — all seven gates and native UI inspection passed |
| P11.13.3 | [#525](https://github.com/storytold/designcraft/pull/525) / @moppeeer | `60bf43b9e842ca1e8477c5e0b9b5129a281f13c1` | Complete — all seven gates and native UI inspection passed |
| P11.13.4 | [#527](https://github.com/storytold/designcraft/pull/527) / @moppeeer | `15d2b998f028cd6b926227f1bd5fcba99df8bda6` | Complete — all seven gates and native UI inspection passed |
| P11.13.5 | [#532](https://github.com/storytold/designcraft/pull/532) / @aiongg | `61504dae488c9bbc0c9b876e37174cf003c6cd2f` | Complete — all seven gates and native UI inspection passed |
| P11.13.6 | [#533](https://github.com/storytold/designcraft/pull/533) / @aiongg | `3f752312d6229acc6e9f36346bb69b2c77a00bf3` | Complete — all seven gates and native UI inspection passed |
| P11.13.7 | [#541](https://github.com/storytold/designcraft/pull/541) / @moppeeer | `65ddf1f119e84972208a661b138cccd3c6301020` | Complete — all seven gates and native UI inspection passed |
| P11.13.8 | [#552](https://github.com/storytold/designcraft/pull/552) / @moppeeer | `3e9881c29bb2410dcb7a46261f91c977a681b7d4` | Complete — all seven gates and native UI inspection passed |
| P11.13.9 | [#519](https://github.com/storytold/designcraft/pull/519) / @moppeeer | `c6d285668875179ad804eb9b3adc5ee6d1e9ab89` | Complete — all seven gates and native graphics inspection passed |
| P11.13.10 | [#528](https://github.com/storytold/designcraft/pull/528) / @aiongg | `54b607525e1614f574778409e686dda76e2069ad` | Complete — all seven gates and native graphics inspection passed |
| P11.13.11 | [#550](https://github.com/storytold/designcraft/pull/550) / @moppeeer | `2db2da0bbcd70548581c6fd574b57b9c2aae3a86` | Complete — all seven gates and native graphics inspection passed |
| P11.13.12 | [#561](https://github.com/storytold/designcraft/pull/561) / @aiongg | `e111bb74f76a438f7cf3fa488c4e1cc8421bf599` | Complete — all seven gates and native graphics inspection passed |
| P11.13.13 | [#551](https://github.com/storytold/designcraft/pull/551) / @moppeeer | `ce9b269df28bb94b124419af54b4969ce687600c` | Complete — all seven gates passed |
| P11.13.14 | [#520](https://github.com/storytold/designcraft/pull/520) / @moppeeer | `23bafc9e53a0c29a25230e0a23246d9de380244e` | Complete — all seven gates passed |
| P11.13.15 | [#521](https://github.com/storytold/designcraft/pull/521) / @moppeeer | `db63e203188c06bd7c6452b86e7b0dc63b70e820` | Complete — all seven gates passed |
| P11.13.16 | [#548](https://github.com/storytold/designcraft/pull/548) / @itrycodinganditsbad | `7968afa39f88471796771a50bc0e770a1c7ed94e` | Complete — all seven gates passed |
| P11.13.17 | [#560](https://github.com/storytold/designcraft/pull/560) / @aiongg | `9061475831df145f5d57577b6ae6c09b91a755b9` | Complete — all seven gates passed |
| P11.13.18 | [#529](https://github.com/storytold/designcraft/pull/529) / @aiongg | `a8fc6a521999ee7688675acb5167a006fdb3eb2a` | Complete |
| P11.13.19 | [#523](https://github.com/storytold/designcraft/pull/523) / @seanth | `5898c084b6d5d65d98e177c44dc6a2db23b27647` | Complete |
| P11.13.20 | [#559](https://github.com/storytold/designcraft/pull/559) / @aiongg | `406f1b3d267fa17617a96d75324d0d709a875361` | Complete |
| P11.13.21 | [#562](https://github.com/storytold/designcraft/pull/562) / @aiongg | `ab231cde62348a3aa428286e2bad88476b58ccb9` | Complete |
| P11.13.22 | [#563](https://github.com/storytold/designcraft/pull/563) / @aiongg | `6e9517b053d053a460691b7f4bd0754d3816828b` | Complete |
| P11.13.23 | [#522](https://github.com/storytold/designcraft/pull/522) / @moppeeer | `95a5317b0ba95403c9e505f9e4baa6b78279d79c` | Complete |
| P11.13.24 | [#538](https://github.com/storytold/designcraft/pull/538) / @moppeeer | `a01bfcde7611da3622a9ec6a5cb1d6cfe11d648f` | Complete |
| P11.13.25 | [#539](https://github.com/storytold/designcraft/pull/539) / @moppeeer | `1e34026a791390908adb4e3b561860eecb4c5851` | Complete |
| P11.13.26 | [#459](https://github.com/storytold/designcraft/pull/459) / @sanyamsingh001 | `1c7f7ac13212488ba1ff999967cb518fd4b7fdd8` | Complete |
| P11.13.27 | [#478](https://github.com/storytold/designcraft/pull/478) / @sanyamsingh001 | `8f33ef36d446af17ab01955928fee1b657c47ad4` | Complete |
| P11.13.28 | [#468](https://github.com/storytold/designcraft/pull/468) / @soro-82 | `5f535ef60f87b03dd69a65d9e64e6322e4816331` | Complete |
| P11.13.29 | [#473](https://github.com/storytold/designcraft/pull/473) / @MarekSpodBiedry | `11b9a0f1041593638edd238a88565c8043625331` | Complete |
| P11.13.30 | [#542](https://github.com/storytold/designcraft/pull/542) / @sanyamsingh001 | `c08b3ddc685ece9534850884567fa3dbac6784a4` | Pending |
| P11.13.31 | [#452](https://github.com/storytold/designcraft/pull/452) / @skylartaylor | `957890223a0c84070278557bac529d6da848ebe6` | Pending |
| P11.13.32 | [#553](https://github.com/storytold/designcraft/pull/553) / @itrycodinganditsbad | `fb90a508b1ccb64ba8cea2e575e015659417aa85` | Pending |

## P11.13.1 — nested undo after a caught panic (#411)

A command continuing after an inner command's caught panic previously lost its outer
undo-step ownership, so subsequent edits recorded extra undo entries. The guard now
restores the saved ownership after recovery as well as before it. Top-level failures
still restore false, allowing later independent commands to record their own undo.

The new synthetic regression fails before the fix, then checks the continuing frame
edit is owned by the outer step and a later independent frame edit records undo.
All seven gates pass: 2,307 tests passed, 0 failed, 7 ignored; twenty WASM packages. No working recovery or command-dispatch subsystem is replaced.

## P11.13.2 — Properties Pathfinder and accessibility (#524)

Properties now shows five original vector Pathfinder controls for multiple selected
objects, using the existing object.pathfinder command. Sections and icon buttons
expose their names and actual enabled/selected state to accessibility. Existing
menu tests distinguish command rows from identically named panel toggles.

Synthetic component interaction verifies one-object hiding, all five buttons and
Intersect geometry. The separate native QA profile's screenshot was inspected at
1440×900; the row fits the Properties panel. Only the disposable QA app was closed.
All seven gates pass: 2,308 tests passed, zero failed, 7 ignored; twenty WASM packages.

## P11.13 — related UI group (#525, #527, #532, #533, #541, #552)

Properties swatch menus now expose tint and explicit Frame/Text targets. Tint-only
object edits preserve each swatch, and do not introduce a missing stroke. Opening
Color Picker retains the chosen text target. Shared swatch lists and lazy fonts remain.

View controls move into Screen Mode, Extras, Grids & Guides and Structure groups;
W/Shift+W commands remain registered. Document Grid renders the same page-origin
positions used by snapping, on the nearest spread's pasteboard band. It follows
front/back settings, hides in preview/export, thins dense subdivisions and caps lines.
Properties and the authenticated UI control channel use the same visibility command.

Type and the Paragraph menu gain Justification. It shares the existing style controls,
shows mixed values empty, edits only changed fields, validates finite ranges and
minimum/desired/maximum ordering, and keeps invalid dialogs open. Pagenor's typing
attributes and selected-text preview remain. The dialog records document and selection
ownership, rejecting stale edits after selection changes.

Properties gains rotation, shear, X/Y scale, quarter-turn and flip controls via existing
transform commands and original drawn icons. Tools Fill/Stroke double-click opens
Color Picker with the selection's color and current text-formatting target.

Each pinned source and author is listed above. Missing optional upstream translation
modules and Italian additions were not introduced; Pagenor's inline translation table,
extra paragraph options and native menu behavior are retained. All seven gates pass: 2,322 tests passed, zero failed, seven ignored; twenty WASM
packages. Native QA screenshots were inspected at 1440×900: grid/transform controls, the
swatch menu (Frame/Text and 35% tint), and the complete Justification dialog. The
disposable QA app was closed; the owner application/session was untouched. Both Tools
boxes, Frame/Text targets and undo were checked by component interaction tests.

## P11.13 — placed graphics and geometry (#519, #528, #550, #561)

Live corner options trim curved sides by arc length, preserving arcs and leaving smooth
points unchanged. Nine focused geometry tests pass, including exact bevel distances.
Image placement fits once into an existing frame and leaves Auto-Fit off; the placement
regression confirms that moving a frame edge crops without moving/scaling the image.
Pagenor's validated PDF crop geometry and graphic wrap metadata remain in place.

transform.set acts on placed content when Direct Selection has selected that content,
with an explicit content override. Explicit item IDs keep frame semantics. Frame and
content measurements remain separate, and Properties shows the selected graphic's
bounds/angle/scales. Singular/overflowing geometry is rejected through the existing
copy-on-write edit transaction. Three focused content-transform regressions pass.

Saved graphic clipping paths have a defaulted optional model field. IDML import/export,
CPU rendering and PDF export retain the path and inverted fill rule. Imports cap the
outline at 100,000 anchors and reject non-finite transforms/coordinates. Options can
report, invert or remove an existing saved path; the existing tracing command remains.
Seven focused clipping tests pass, including normal/inverted pixel and PDF checks.

Native format 4 identifies the retained clipping capability; prior native versions
continue to load. A new native round-trip verifies geometry/options, transform and
format metadata, preventing older readers from silently discarding the new field.
All Pagenor-specific graphic constructors preserve their other metadata and initialise
absent clips to None. All seven gates pass: 2,337 tests passed, zero failed, seven ignored; twenty WASM
packages. Native QA inspected upright selected content in a tilted frame, retained
triangle clipping and its inverse at 1440×900, using generated pixels and a synthetic
native document. Only disposable QA instances were closed; the owner session remained.

Native inspection also caught the legacy selection outline measuring intermediate
axis-aligned bounds twice. Canvas now composes frame and graphic matrices first,
then measures the final bounds. Its painted-output regression fails before the fix
(600×600 pt outline for upright 400×200 pt content), passes afterwards, and the corrected
native screenshots were inspected. This is a Pagenor integration correction for #550.

## P11.13 — colors, command results and font lookup (#551, #520, #521, #548, #560)

Unnamed colors apply to selected table-cell text, including empty cells, while explicit
object IDs retain frame targeting. Existing caret typing, explicit Formatting Affects
Text, swatch reuse and one-step undo remain. Both focused target regressions pass.
Fill/stroke/gap-stroke names are checked against document swatches before mutation;
unknown names get bracket/case hints and leave the document untouched. Tint-only
edits remain supported, including explicit single IDs. The focused validation passes.
Moves, copies and Step and Repeat return their object IDs; Pagenor's content-move
branch also returns its targets, preserving nested movement logic. Copy-ID tests pass.

SVGs resolve static/default PostScript names in family-list order, honoring document
font scopes and exact face style. The catalogue reads only existing small name tables;
PostScript lookup does not load an installed test face. Retained metadata includes the
new string in its size budget. Native activation eligibility remains checked. Existing
SVG variable-font CSS handling is retained; base names are not incorrectly assigned to
named instances. Synthetic PostScript and two-document/scope-closure tests pass.

Exact font-family spaces are preserved in IDML and remain distinct when a font has that
name. Only missing names use trimmed lookup, then existing suffix/prefix/alias rules.
Composite-font references and document precedence remain. FontStyle Nothing/$ID/Nothing
inherits rather than inventing a style. The focused IDML import/export test passes.
All seven gates pass: 2,345 tests passed, zero failed, seven ignored; twenty WASM
packages. Existing synthetic resize fixtures now name [Black]/[None], and the old UI
format-target fixture creates Cyan before applying it, so validation checks real inks.
No font file is added, no UI subsystem is replaced, and the owner session is untouched.

## P11.13 — import and anchored composition group (#529, #523, #559, #562, #563)

IDML spread rotations accept only finite quarter-turn transforms and round-trip without
rotating page/item geometry. Parent-page text caches use absolute page indices, so
restarted numbering cannot share the wrong running header. Section markers resolve
from the actual section, including zero-width blank markers, and edits invalidate text.
Pagenor's PPTX, PDF and rendering paths use the same indexed cache.

Anchored settings inherit per attribute through the existing bounded object-style
chain; left-aligned IDML X offsets change sign at the import/export boundary. Existing
anchored text frames, groups, footnotes and baseline rules remain supported.

Anchored wrapping reuses Pagenor's contour/side and nested graphic wrap geometry.
Own-story wraps start after the anchor's line; other stories share spread obstacles.
Four bounded settling passes and independent neighbor compositions prevent cycles.
The cache holds neighbor story/item allocations and invalidates when an anchor moves;
renderer damage includes affected neighbor frames. Synthetic regressions cover
geometry, ignore-wrap, movement, same-story settling and mutually wrapping stories.

Validation: focused rotation, parent-page cache, section-marker and anchored tests
passed. All seven gates pass: 2,359 tests, zero failures, seven ignored; twenty WASM packages.

## P11.13 — paragraph input and decoration group (#522, #538, #539)

All text-bearing editing commands now share the existing CR/CRLF-to-LF helper,
including story replacement, editorial notes, footnotes/endnotes, cells and path text.
The scripts regression verifies paragraph counts and note text after replacement.
Uppercase words use the existing language hyphenator when capitalized-word
hyphenation is enabled; disabling it still leaves those words intact.

The existing native decoration renderer now merges each continuous underline or
strike independently, preserving Pagenor's rule metrics and allowing either writing
direction. A fractional-zoom pixel test checks the underline has no lighter seams.
Anchored wrap also tracks global marker/settings/visibility dependencies, covered
by a neighbor section-marker edit regression. No new font or asset is introduced.

Focused line-ending and uppercase tests passed. All seven gates pass: 2,361 tests,
zero failures, seven ignored; twenty WASM packages.

## P11.13 — incremental reflow and UI group (#459, #478, #468, #473)

The existing explicit-deletion reflow guard now distinguishes consumed condition
visibility from unused or fully overridden GREP references, including overset text.
It shares segment resolution with contextual shaping and preserves drop caps.
Unused/shadowed conditions retain deleted pages; a printed overset condition can
resume primary growth. Both behaviors preserve undo/redo in synthetic regressions.
Native empty clipboard completions are delivered only for a usable retained rich
payload, after current input, preserving Pagenor's clipboard markers and shutdown.

Split views dispatch one chronological input stream before either view paints.
Pagenor's IME, triple-click, Alt-hand, ruler-origin, menu and scrollbar paths remain;
focused canvas geometry also follows the active pane. Flyouts retain press origin
through release and cancel tearing on Escape. Split/paste/click/tear/IME tests pass.

Guide and margin dialogs show captured page/master descriptions and page/spread
choices, plus all-document margins. Existing unit capture, finite/count validation
and error retention remain. Left/right master defaults stay independent; whole-master
spread changes require the explicit choice. Tests cover units and all target paths.
The existing explicit spacing-off export now writes IDML Nothing; the test inspects
actual Story XML as well as importing it back. Other legacy import values remain.

Focused tests and all seven gates pass: 2,383 tests, zero failures, seven ignored;
twenty WASM packages. Native 1440×900 screenshots were inspected for split views,
scrollbars and target dialogs. Only the independent QA process was closed.
