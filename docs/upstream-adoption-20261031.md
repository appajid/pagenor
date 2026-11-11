# Pagenor 0.13.0 — upstream adaptations, 31 October 2026

Status: implemented and verified locally as P11.23 / 0.13.0. Local Mac packaging is recorded separately in BUILD_STATUS.json; no publication.
Base: `4bfe980582b84d117ff554328392b7ab05e0b1c0` (0.12.0).

The review screened 546 updated open/closed records since 22 October (220 open,
326 closed), through #2086. Thirty-two current proposals and six older scopes
supply the adaptations below. These are selected changes, with local implementation
where Pagenor's architecture differs; whole upstream branches were not merged.
All 38 source heads were rechecked unchanged in the recorded review snapshot.
The original MIT OR Apache-2.0 licences, copyrights and third-party notices remain.
Author logins below identify source provenance; generated contributor records are
untouched. Owner branding remains Pagenor.

## Per-proposal ledger

| PR | Source author | Inspected immutable head | Local adaptation |
|---|---|---|---|
| [#1444](https://github.com/storytold/designcraft/pull/1444) | @AndreRese | `66ad56ba414135930eb4b3f9c9407ebaf1a11c0b` | Background graphic-link checks, owned results, bounded workers, streamed external-byte comparisons and adaptive polling. |
| [#1496](https://github.com/storytold/designcraft/pull/1496) | @moppeeer | `d865a7cca15fa7ed81d18e0bf6e2bad41e47a153` | Find/Change formatting criteria and format-only replacement. |
| [#1497](https://github.com/storytold/designcraft/pull/1497) | @moppeeer | `cd84bd903ae92530eb0ce6cf248629d491b51e20` | Change and Change/Find actions, stable selection scope and table-cell traversal. |
| [#1498](https://github.com/storytold/designcraft/pull/1498) | @moppeeer | `620f83d44a08c0b21fcbc1352a8440f71182c3df` | Bounded GREP features and format controls; retain Pagenor EOF/default formatting and named replacement syntax. |
| [#1531](https://github.com/storytold/designcraft/pull/1531) | @PiMinusMax | `cc5efba46d1980d4656cfa62b79c631c71ee849e` | Per-document Links search, file information, color/ICC metadata, counts, Update All, Copy Path and native Show in Folder. |
| [#1537](https://github.com/storytold/designcraft/pull/1537) | @zuujiscom | `19d03eee278df648b6bcf47929a583fbc91af79e` | Shared pure Gregorian timestamp arithmetic for variables, PDF/PPTX metadata and logging; retain existing EPUB behavior. |
| [#1961](https://github.com/storytold/designcraft/pull/1961) | @aiongg | `688c41a2e08d7513fcd06e5521d9ac04df91f1ca` | Pinned craft-dialogs native asynchronous dialogs across Open, Place, Save/Copy, exports, books, libraries, profiles and panel imports. Document-bound continuations, replace confirmation and Save/Quit cancellation. |
| [#1975](https://github.com/storytold/designcraft/pull/1975) | @VoiceLessQ | `b2d8d8809af48016e579e28f245d5f75c1cbab85` | Decoded RGBA allocation budget, including TIFF/CMYK/PDF image paths; retain stricter importer budgets. |
| [#1968](https://github.com/storytold/designcraft/pull/1968) | @VoiceLessQ | `ba1ee2329b971bb2dc7dde669846f44618290be4` | Fresh table, anchored-object and nested-story IDs in copy/data-merge paths. |
| [#1843](https://github.com/storytold/designcraft/pull/1843) | @dotCooCoo | `9702c9676a1985cd47df78af194565d16e0aeb55` | Common bounded object/story cloning, retargeted anchors and independent cell/note/endnote content. |
| [#2055](https://github.com/storytold/designcraft/pull/2055) | @ZIDAN44 | `3372e3f3cf34739d45883af568524a1535ef8a51` | Independent horizontal/vertical ruler units in rectangle-size dialogs. |
| [#2052](https://github.com/storytold/designcraft/pull/2052) | @texjer | `99f90cbe2bf43511535644c3221097aee9c516aa` | Imported story identity map and warning-frame navigation; exact IDML Self or Pagenor INDD decimal uID, never an invented item ID. |
| [#2035](https://github.com/storytold/designcraft/pull/2035) | @janwalenda | `75c818d4748c4150173958e60d1c358f6b65e9da` | Grouped File → Export menu; package/booklet actions preserved. |
| [#2031](https://github.com/storytold/designcraft/pull/2031) | @tareq-halaby | `61687039eceb45a854afa885b304189325e73787` | Option/Alt-click tool cycling and modified double-click guard; preserve flyouts and toolbar columns. |
| [#1866](https://github.com/storytold/designcraft/pull/1866) | @Zaxaerith | `0e833a37fa615e94e400849acd32a8be9a8a7209` | Fitting submenu for text frames. |
| [#1834](https://github.com/storytold/designcraft/pull/1834) | @dotCooCoo | `92e03d09658e00fe8ca5733f27ea51696beb1c13` | Duplicate Layer preserves settings, grouped/anchored objects, text threading, nested cell/note objects, guides, page overrides and hyperlink sources. |
| [#1638](https://github.com/storytold/designcraft/pull/1638) | @marcoarnulfo | `3ea645965189027b7a0ffd39b9d1acb6cdee57ca` | Per-document page history including master sides, stable PageIds and Back/Forward commands. |
| [#2033](https://github.com/storytold/designcraft/pull/2033) | @abishtakan | `4a69c39e2488398fef53c9149d72ee14037f427a` | World-Ready paragraph and single-line composer identities, native/IDML round trips and Pagenor shaping. |
| [#1864](https://github.com/storytold/designcraft/pull/1864) | @Hamza0101 | `93e9fcab194226b49568e3bc5669af43be32514f` | Named variable instance PostScript-name construction and synthetic fvar coverage. |
| [#1969](https://github.com/storytold/designcraft/pull/1969) | @bantjes | `a2f2e11e30ad1739b7d21dcbf3f95aa2ed32e2ef` | RIBBI style-alias filtering in eager, lazy and native catalogues. |
| [#2024](https://github.com/storytold/designcraft/pull/2024) | @Zaxaerith | `9ee25b1561b57644cc9f91e557b76f10f8e2bbac` | Relative document paths locate Document Fonts in the current directory. |
| [#1826](https://github.com/storytold/designcraft/pull/1826) | @ZJHeepfixer | `c6c4a038b22ae1212ebea8a6c2c95b366685b291` | UI font fallbacks are loaded on demand for observed scripts, retaining asynchronous live font generations and scoped fonts. |
| [#1654](https://github.com/storytold/designcraft/pull/1654) | @makesomethingshit | `b6f82deb11606d0022f428c77f36d6e4863d015f` | Page Tool and Properties resize a selected master side with nine anchor positions, margins and guides. |
| [#2071](https://github.com/storytold/designcraft/pull/2071) | @texjer | `06d29fe3a63f4447879dc4928f85bcf87a5aec36` | Positional master choice in single-sided multi-page spreads; facing-page left/right policy retained. |
| [#2057](https://github.com/storytold/designcraft/pull/2057) | @seanth | `06a5199e1340148fca6b0e8b4fba34fc1ec6dfae` | Running-header placeholders while editing masters, actual values on document pages. |
| [#1629](https://github.com/storytold/designcraft/pull/1629) | @marcoarnulfo | `e7d16aeaaae68e61dda386776f62c02f67fc41a4` | Manual Autoflow consumes existing following columns/pages before adding pages and removes only newly created empty tails. |
| [#1972](https://github.com/storytold/designcraft/pull/1972) | @VoiceLessQ | `8867f312e91ff3e2d8b580b7bb309afd12b37dde` | Conservative bounded Autoflow estimates around Start Paragraph; existing explicit-break handling and primary reflow retained. |
| [#1971](https://github.com/storytold/designcraft/pull/1971) | @KnottyDyes | `ed88f3e4b5f6775a92e2ab4a2e7fa7d993477ff6` | Column/full-width footnote reservation around spanning content, actual cell-note height and vertical-justification cleanup. |
| [#1766](https://github.com/storytold/designcraft/pull/1766) | @alichaudhry55 | `48f2e64e8e0c898bb24fe2e1863815a18d75efdd` | Spanning tables in multi-column text frames. |
| [#1774](https://github.com/storytold/designcraft/pull/1774) | @alichaudhry55 | `f1cd2c9cba52c24a2d98d16c249548e62b7060c5` | Keep-all spanning tables/headings move to a destination that can accommodate them. |
| [#2054](https://github.com/storytold/designcraft/pull/2054) | @danielveresbelgium | `797edbaaf7274de986b745d176e227020d9dc8ad` | Native book-wide TOC scope, source-style controls, chapter reordering, isolated candidates, up to four closed-book stabilization passes and persisted semantic freshness checks before export. |
| [#2029](https://github.com/storytold/designcraft/pull/2029) | @gnacef-hub | `5498755a1d33f397155590f3853339556eb41e81` | New flat book covers and spine-aware cover resizing, printer page-count rules or explicit widths, guides, fitting, undo and UI. |
| [#1923](https://github.com/storytold/designcraft/pull/1923) | @hvfrancesco | `35b6102096eb87fd5721458561aef5a2412aa494` | Document-owned asynchronous page/master thumbnails, bounded pending work/texture storage and stale-result rejection. |
| [#2051](https://github.com/storytold/designcraft/pull/2051) | @texjer | `82370d41111f059da58cc21c9d3a39dbac17626c` | Canvas image proxies on bounded workers; synchronous full-resolution export paths retained, scoped SVG cache keys and completion repaint. |
| [#2036](https://github.com/storytold/designcraft/pull/2036) | @r31lin | `41ab9484dc0b406289e4157e1a9b14e7f450995c` | Composition cache includes only used numbered lists; shared story-order traversal carries group transforms. |
| [#1815](https://github.com/storytold/designcraft/pull/1815) | @ZJHeepfixer | `fffa9e824c392bd010f80eea113880f4c6896116` | Skip unchanged recovery checkpoints using weak document identity and owned file stamps; retry if the checkpoint changed externally. |
| [#2073](https://github.com/storytold/designcraft/pull/2073) | @texjer | `c0a919a668c684bcefa10a629c15bf64263e9f02` | Atomic bounded multi-image loading, document-owned queue, click/frame/grid placement, contact sheets, Escape cancellation and cursor preview/count. |
| [#2016](https://github.com/storytold/designcraft/pull/2016) | @onion3130 | `61388fdc556e45c9f22cb2c04d919441fcbb0553` | Raster resolution metadata determines natural placement size when sane. |

## Compatibility and scope

- Workspace version is 0.13.0. Native capability version 7 preserves the new
  composer enum values and book TOC scope. Older Pagenor documents remain readable;
  readers limited to format 6 must refuse new files rather than discard fields.
- Preserve `.pgnr` and legacy `.pagenor` opening; distinct owner-supplied app/document
  icons, lazy/native font catalogue, dynamic-font refresh, dot-family menu filtering,
  document font scopes, inline Indic input and EOF formatting, welcome/recent files,
  explicit left/right masters, grouped vertical panels and existing Links color/ICC.
- Add pinned `craft-dialogs` at `1a11a4897865f7410c40d25ca6e452ce399a1a2c`
  for the native host and bounded fancy-regex 0.16.2. Existing pinned craft-ui,
  craft-print and egui remain unchanged. No broad dependency upgrade was adopted.
- Native dialogs are bound to the original document UID. Closing that document
  cancels its answer; unrelated egui modals delay it. Save/Quit waits for a successful
  save; cancellation/unavailable dialogs cancel quitting. Browser inbox/download
  services remain separate. Tests use dialog overrides and never launch OS dialogs.
- Book TOCs use native chapters and book order. The active chapter may be edited in
  memory only with fixed TOC space and current numbering; other participants must
  be closed. Closed-book Refresh numbers and composes isolated candidates (maximum
  four passes) before writing. Exactly one Entire Book TOC, at most 4096 chapters,
  128 MiB input, 256 MiB prepared output, 100000 entries and 4 MiB freshness state.
  Refresh rejects external book-manifest changes before writing; relative chapter
  paths resolve beside the book. Export also composes TOC-only style edits to catch
  newly introduced overset. Per-file atomic replacement compensates earlier writes on failure and reports
  any failed restoration. This is not a filesystem-wide transaction or a lock on
  concurrent external editors. No cross-document bookmark destinations or arbitrary
  imported-layout writes; save IDML chapters as `.pgnr` first. Existing IDML TOC
  definition interchange is not extended; scope is preserved in native documents.
- Multi-image placement takes at most 1000 images / 512 MiB encoded input, preserving
  selection order. A corrupt source leaves the document and old queue intact.
  File → Contact Sheet exposes rows, columns, gutter and page margin for the loaded
  queue; File → New Book Cover and Resize Book Cover expose cover commands.
  Multi-selection is graphics-only; single PDF and text/layout import options retain
  their own workflows. Switching/closing documents or changing tools clears the queue.
- Warning Show is available only when a warning names an actual imported story with
  a surviving frame. Current package/ruler/global INDD warnings have no fabricated
  destination. Pagenor's converter emits decimal `u` identifiers, not upstream's hex.
- Recovery already avoids large checkpoint previews; #1923's applicable change is
  page/master thumbnail ownership. #1537 reuses shared dates only in matching
  consumers. #1972 retains the existing explicit-break estimator and adds bounded
  Start Paragraph estimates. Performance optimizations are bounded by construction;
  no comparative speedup percentage is claimed without a dedicated benchmark.
- Lulu's standard 444-PPI paperback formula and hardback bands were verified against
  its [published reference](https://help.api.lulu.com/en/support/solutions/articles/64000254616-how-is-spine-width-calculated-)
  on 31 October. Users can override spine width for other paper/binding products;
  ambiguous published band boundaries use the first matching band.

## Already covered / intentionally excluded

#2049 native Quit, #1667 welcome scrolling, #1751 bounded link handling, #1757 strict
asset validation, #1675 finite IDML handling, #1674 Shift+Tab, #2045 relative font
folder duplicate, #1505/#1508/#1521 and existing overset/hyperlink work were retained.
Exclude wholesale merges #2056/#2067, unpinned CI #2018, egui 0.37 migration #1810,
bulk dependency changes #2020/#1929, translation/CJK-only proposals and unrelated
Android/server/web-only work. This task does not modify Vectnor or the owner's
running app, and does not publish or upload source or release assets.

## Validation

`CARGO_INCREMENTAL=0 cargo xtask ci` passes all seven steps: fmt, locked workspace
check, strict Clippy, tests, assets, layers and WASM. **3053 passed, zero failed,
seven existing skips; all twenty WASM packages; zero compiler warnings.**

Twenty-two synthetic offscreen views were inspected: Welcome, pages/history,
TOC style/scope controls, new/resize cover options and printer rules, Find/Change
and format criteria, Links metadata, duplicated layers, queued/grid placement,
left/right master sizing, File menu and Contact Sheet options/result. Dialog
continuation tests cover cancellation, Save/Quit and stale/closed owners without
launching a native OS dialog. The Contact Sheet regression supplies a real canvas
viewport and verifies destination navigation, queue consumption and one-step undo.

The sample's duplicated wrapping quote produces the same mutual-wrap overset as
0.12.0 when an identical frame is copied at the same coordinates; this is recorded
as unchanged behavior, not suppressed. Regression coverage also checks independent
nested IDs, closed-book stabilization and external-manifest refusal, TOC-only style
overset, relative paths, stale worker rejection and release of encoded image bytes.
Image caches retain weak encoded owners and bounded decoded pixels.

Evidence is in the local `pagenor-upstream-20261031` output directory: `ci-release.log`,
`VALIDATION.json`, `QA_SUMMARY.txt`, twenty-two PNGs/scripts in `qa/`, and separate
Mac bundle/signature/version/format-compatibility results. Packaging is local arm64,
ad-hoc signed, without notarization or installation. The owner's open session is
preserved. Native Windows/Linux execution and interactive OS dialog checks remain
separate validation work; no such execution is implied by Mac or WASM checks.
