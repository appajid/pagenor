# P11.21 — upstream review and local Pagenor 0.11.0

Review snapshot: 22 October 2026 (Asia/Kolkata), through upstream #1573.
Base: local 0.10.0, `39a312f05304c2bba455c110c9c1be27cadc68f0`.
The open/closed updated-PR scan returned 300 records reaching back to 18 October;
186 were updated since the previous review snapshot (168 open, 18 closed).
Metadata screening is not code adoption. Thirty-eight candidate scopes and revised
heads were checked for compatibility: **20 supply adaptations, eight are covered,
one has no matching control, and nine remain queued**. All 38 heads were rechecked
unchanged. Other metadata-only proposals remain unadopted; this is not a claim of
complete upstream parity. The previous 0.10.0 deferred rendering/import work remains
pending unless specifically listed here.

## Visible changes

- Appearance choices preview immediately; Cancel restores the original theme.
  Retain Pagenor’s System mode and favorite themes. Preferences includes a Tools
  single/double column choice and a separate UI Scaling section. Right-side panel
  columns and grouping are preserved.
- Effects color choices show their actual swatch chips. Angle dials accept arrow
  keys, Shift for ten-degree steps, and accessibility value/increment actions.
- Multi-object wrap edits preserve every object’s mode and untouched offsets.
  Linked-offset state belongs to its document/selection. Distribution buttons
  require three objects unless a spacing is chosen; malformed spacing is refused.
- Properties distinguishes embedded images from linked files and mixed selections;
  Paper swatch editability follows its value even under an imported/localized name.
- PDF and PowerPoint exports warn about overset/unplaced story text. Warnings name
  the page/frame/count and follow selected export pages. Reflowable exports retain
  full text. `story.get` exposes `oversetChars` independently of its byte offset.
- PowerPoint font embedding is opt-in in both the command and export dialog. The
  existing licence checks and full export-option forwarding remain in place.
- Default dashed strokes stay dashed on canvas, table borders, PDF and PowerPoint;
  underlines/strikes span tabs. Rule Above uses the baseline for its offset.
- IDML preserves literal colons/backslashes in paragraph, character, table, cell
  and object style names. Invalid page/table counts, hyperlink page destinations
  and duplicate/blank anchor names are refused before editing. Repeated export
  pages are included once. Common English words and curly-apostrophe contractions
  are accepted by the existing spelling checker.

## Compatibility and validation

Preserve Rust layering and `unsafe_code = "forbid"`, Pagenor branding and owner
rights, `.pgnr` and legacy loading, distinct app/document icons, lazy/live/scoped
fonts, inline Indic input, EOF/final-LF/genuine paragraph defaults, explicit
left/right book masters, Welcome/recent documents and grouped/vertical/dual-column
panels, Links color-mode/ICC reporting and private legacy-font import. No private
fonts, documents or mapping tables, new assets, translation catalogues or upstream
branding identifiers are added. Vectnor files and its chat are not touched.

The native document capability stays **6**; new anchor hyperlink variants and
persisted effect changes remain deferred. Existing external dependency pins stay
unchanged. Keep Pagenor’s popup-aware color picker, document-owned Effects preview,
dotted raster correction, dash expansion budgets, atomic writers and distribution
overflow/locked-item safeguards rather than replacing their subsystems.

All seven workspace gates pass: 2,894 tests, zero failures, seven existing skips; twenty WASM packages. Six synthetic offscreen views are inspected. The separate optimized local Mac GUI/CLI bundle and artifact-side signature/icon/reader report follow the source commit. Native Windows/Linux execution and publication remain pending; the owner’s running session stays open.

## Source pins and decisions

Adapted source retains MIT OR Apache-2.0 provenance. Immutable PR heads and
upstream authors are credited below. A partial adaptation is explicitly labelled.

| PR | Author | Reviewed head | Decision |
|---|---|---|---|
| [#1318](https://github.com/storytold/designcraft/pull/1318) | @andeecollard | `fe9f0432758db2e3ff9f21a4fe673b3fe998b39d` | covered: Changed head adds tests for high-quality shortcut already implemented in Pagenor. |
| [#1390](https://github.com/storytold/designcraft/pull/1390) | @moppeeer | `a1bbf1c7a105aeca70bef3856128018688eb3e7f` | adapted: Incremental: preserve each object’s wrap mode and untouched offsets in multi-selection edits; key link state by document and selection. |
| [#1394](https://github.com/storytold/designcraft/pull/1394) | @moppeeer | `19513160700adb99ca8ed8d798d3fb1b9d14459d` | covered: Production patch unchanged apart from hunk offsets. |
| [#1409](https://github.com/storytold/designcraft/pull/1409) | @moppeeer | `3ae5284acc684f13e2957536535a46882749b34b` | covered: Documentation change only; centimeter precision already implemented. |
| [#1419](https://github.com/storytold/designcraft/pull/1419) | @moppeeer | `cf2e1a458eaab7327bc003a418c3d5c8b6017fb3` | adapted: Incremental: distinguish linked/embedded/mixed image selections; extend existing title regression. |
| [#1420](https://github.com/storytold/designcraft/pull/1420) | @moppeeer | `324084d4490a0ed3e48cd7c20cb8156cd77108ab` | covered: Production patch unchanged apart from hunk offsets. |
| [#1431](https://github.com/storytold/designcraft/pull/1431) | @moppeeer | `67e40d758d5d0e711c90d83da558280de0aaded1` | adapted: Incremental: Paper editability follows swatch value instead of its translated/imported name. |
| [#1434](https://github.com/storytold/designcraft/pull/1434) | @moppeeer | `5b813d4f03afc5df189299e775334fc82d44caf9` | covered: Production patch unchanged apart from hunk offsets. |
| [#1436](https://github.com/storytold/designcraft/pull/1436) | @moppeeer | `37d4afe1802e32abacc4e0f7057cb2e77218ffc9` | adapted: Incremental: reject malformed fixed spacing; disable two-object distribution until spacing is chosen; retain local overflow/locked-item guards. |
| [#1437](https://github.com/storytold/designcraft/pull/1437) | @moppeeer | `ef8165422950b2cce64eef803847ae64383da15c` | covered: Production patch unchanged apart from hunk offsets. |
| [#1444](https://github.com/storytold/designcraft/pull/1444) | @AndreRese | `66ad56ba414135930eb4b3f9c9407ebaf1a11c0b` | queued: Background link checks change document/status/threading behavior; need bounded asynchronous lifecycle and network-path tests. |
| [#1452](https://github.com/storytold/designcraft/pull/1452) | @moppeeer | `d10ede80258bc9787bddc2cb86d39245ec0ee1b9` | adapted: Incremental: keyboard and accessibility value/step actions for angle dial, preserving current document-owned Effects transactions. |
| [#1488](https://github.com/storytold/designcraft/pull/1488) | @Hamza0101 | `26235ecbad4b5588c54330045a8c4f9b50d323ba` | adapted: Resolve empty dashed patterns in composition, CPU rendering, table borders, PDF and PPTX; retain dotted raster and expansion-budget guards. |
| [#1489](https://github.com/storytold/designcraft/pull/1489) | @midasdf | `db0bfe445b9c2244535eaaaa4e9f61bc2d647fc1` | adapted: Make permitted PPTX font embedding opt-in in engine and UI options, preserving Pagenor’s full option forwarding and atomic writer. |
| [#1491](https://github.com/storytold/designcraft/pull/1491) | @Hamza0101 | `0fc845750aafaf596591f3e48e2eac1632b0e943` | adapted: Carry underline/strike bars over tabs without drawing tab glyphs, on canvas and PDF; retain PDF tagging. |
| [#1493](https://github.com/storytold/designcraft/pull/1493) | @Hamza0101 | `ca21ba7cabda91359120d6a9b3513d379f08f0f4` | adapted: Report overset/unplaced story text in page-based PDF/PPTX exports, scoped to selected pages; expose oversetChars separately from the byte offset. |
| [#1496](https://github.com/storytold/designcraft/pull/1496) | @moppeeer | `e33398a3c4474e3a224d53020d29610116dc733f` | queued: Change Format depends on newer formatting-match and style-application lifecycle. |
| [#1497](https://github.com/storytold/designcraft/pull/1497) | @moppeeer | `f615508ea59f28b0606d69aac4d6d6e8b5247b9f` | queued: Find Format depends on newer formatting-match and style-application lifecycle. |
| [#1498](https://github.com/storytold/designcraft/pull/1498) | @moppeeer | `94e414459e3a5e773be9510fbeec92bdd27422d0` | queued: Find Change/Change-Find depends on newer formatting-match and style-application lifecycle. |
| [#1499](https://github.com/storytold/designcraft/pull/1499) | @moppeeer | `7067744340ef40b337ca0cd6b0afd1a4e855909c` | covered: Existing popup-aware Fill/Stroke double-click picker and tests already cover frame/text targets. |
| [#1505](https://github.com/storytold/designcraft/pull/1505) | @moppeeer | `e19b000a30507619e909a4f7eb4da9e8833b11fa` | queued: Page/master navigation box needs reconciliation with Pagenor enlarged status bar and explicit left/right master view. |
| [#1508](https://github.com/storytold/designcraft/pull/1508) | @moppeeer | `ebb428619a6bf469e987840c9c344e43833bd835` | queued: Insert Pages dialog needs its parent-reference backend reconciled with explicit book-master defaults. |
| [#1509](https://github.com/storytold/designcraft/pull/1509) | @moppeeer | `8fb00c5a26e5527a092cee9565ce957156b336ef` | adapted: Recognize common English words and curly-apostrophe contractions; retain user dictionary and data-optional builds. |
| [#1512](https://github.com/storytold/designcraft/pull/1512) | @moppeeer | `d9e8438cbbae386c5dfca0bedf1f2f638e16e33d` | adapted: Separate UI Scaling; preview Appearance with Cancel restoration and explicit Tools column choice, retaining Pagenor System mode, favorite themes and right-panel columns. |
| [#1513](https://github.com/storytold/designcraft/pull/1513) | @moppeeer | `d13f6cf4c1db4a7ffcd26ca41707e7f63f7ed056` | not applicable: Pagenor uses its existing recent-document implementation; this upstream recentFilesMax field is absent. No decimal control to fix. |
| [#1521](https://github.com/storytold/designcraft/pull/1521) | @moppeeer | `259cd54243c0d35a6ae045cf7af74fd392bf20cc` | queued: Linked paragraph sides/corners require coordinated style-dialog integration and preview/cancel coverage. |
| [#1524](https://github.com/storytold/designcraft/pull/1524) | @PiMinusMax | `c089ab5fcd4315820ccba9f921cf205057360a23` | adapted: Show Effects swatch color chips using existing bounded, document-owned preview transactions. Shadow units already follow the document through the dialog; no duplicate legacy panel controls. |
| [#1531](https://github.com/storytold/designcraft/pull/1531) | @PiMinusMax | `55469c487212574c52a5fece319bd3335d38d20a` | queued: Links redesign requires reconciliation with Pagenor color-mode/ICC reporting and grouped panels. |
| [#1535](https://github.com/storytold/designcraft/pull/1535) | @aiongg | `7cbb5b973608074596e00f726bce41646a065dd1` | adapted: Position Rule Above relative to the first baseline rather than subtracting ascent twice. |
| [#1537](https://github.com/storytold/designcraft/pull/1537) | @zuujiscom | `19d03eee278df648b6bcf47929a583fbc91af79e` | queued: Shared date/EPUB refactor spans package serialization and logging; keep as separate compatibility-tested maintenance work. |
| [#1547](https://github.com/storytold/designcraft/pull/1547) | @Silvertree2010 | `fa81009be2450dcfa10ea3b858f5139d6f13a1ab` | adapted: Escape literal colons and backslashes in IDML style names; all five style kinds round-trip. |
| [#1550](https://github.com/storytold/designcraft/pull/1550) | @Silvertree2010 | `35e5b39874a40550404c61337f5e0ee3539a24d0` | adapted: Clamp text-frame column counts before narrowing. |
| [#1556](https://github.com/storytold/designcraft/pull/1556) | @con3code | `2ea4ac7d45ec5c0efa56aaee7a0434531d0b9098` | covered: Current Effects panel uses the dialog; its distance/size fields already use document units. Obsolete inline fields do not apply. |
| [#1562](https://github.com/storytold/designcraft/pull/1562) | @Hamza0101 | `e7ef29d9d1826add4ee8a55374898989ccaef4e1` | adapted: Deduplicate array export page lists with a seen set, preserving first occurrence and existing document-order output. |
| [#1564](https://github.com/storytold/designcraft/pull/1564) | @Hamza0101 | `97820ca26925a4810192b35924f086c869f42dda` | adapted: Partial adaptation: reject invalid page hyperlink destinations on create/edit before mutation. Anchor destinations/new persisted enum variants remain deferred to a format milestone. |
| [#1567](https://github.com/storytold/designcraft/pull/1567) | @Hamza0101 | `2946f2af6202a744dcc2162a2259066b1a4d62ac` | adapted: Reject impossible Insert Pages counts before editing; retain append positions and explicit left/right book-master defaults. |
| [#1568](https://github.com/storytold/designcraft/pull/1568) | @Hamza0101 | `3d099d181ce1028fb94655e7af2bec2e890e0d68` | adapted: Reject impossible table body/header/footer/column counts; preserve cell/EOF insertion semantics. |
| [#1571](https://github.com/storytold/designcraft/pull/1571) | @Hamza0101 | `c212c89bab697768c9839292cb3d2e098f474dd3` | adapted: Refuse blank and duplicate document-wide text-anchor names. |

## Review evidence

The local `outputs/pagenor-upstream-20261022` folder holds the screened metadata
inventory, immutable source/file/patch pins, head recheck, full quality logs, UI
images and separate Mac bundle/signature/version/icon/reader checks. The running
owner session stays open. Native Windows/Linux execution and remote publication
remain separate work; shared Rust source and WASM checks do not certify installers.
