# P11.20 — upstream review and local Pagenor 0.10.0

Review snapshot: 21 October 2026 (Asia/Kolkata), through upstream #1463.
Base: `afcabc3552bf82c63112e9eb59a4af948e81dedf` (local 0.9.0), committed
15 October 2026 at 21:33:34 UTC. The online open/closed updated-PR scan returned
600 distinct records; 476 were updated after that checkpoint. Metadata screening
is not equivalent to code review or adoption. Sixty-eight candidate patches/file
scopes were inspected: **49 supply local adaptations, two are already covered,
one has no corresponding input path, and 16 remain queued**. All 68 inspected
heads were rechecked unchanged. Other screened proposals remain unadopted until
reviewed; this note does not claim complete upstream parity.

## Visible changes

A0/A1/A2 presets, remembered print units and unit-aware preset cards; better dock
chevrons and floated-panel size; Properties text-wrap offsets, fitting controls,
selection titles, multi-frame text targeting, Align To glyphs and distribution;
Swatches default fill/stroke tint controls and ink/color-mode columns; actual
wrapped-row labels in Story Editor; remembered style preview; an Effects angle
dial; corrected platform shortcut displays and native Paste in Place.

## Compatibility and validation

Keep Rust layers and `unsafe_code = "forbid"`, Pagenor branding and Appaji
Ambarisha’s reserved branding rights, .pgnr/legacy loading and separate icons,
lazy/live/document-scoped fonts, inline Indic input, explicit left/right book
masters, grouped/vertical/dual-column panels, Links color-mode/ICC reporting,
private legacy-font import, and EOF/final-LF/genuine paragraph defaults.
No private document/font/mapping or new translation catalogue is included.

The native document capability stays **6**. 0.10.0 is an editor/import feature
milestone; existing persisted document fields and dependency revisions remain
unchanged. Only a temporary-file test dependency is added from the existing lock.
Retain Pagenor’s conservative failed-OpenGL history rather than weakening it to
match upstream’s copied-profile reset assumption. CDATA still consumes the XML
node budget. Distribution rejects overflowing transforms before any mutation.
PDF placement validates a single whole page. Bad syntax retains the dialog for
correction; numeric pages outside the document follow the existing staged-payload
cancellation path.

All seven workspace gates pass: **2,875 tests**, zero failures, 7 existing ignores; twenty WASM packages. Seven synthetic offscreen views were inspected. The separate optimized local Mac arm64 GUI/CLI build follows the source commit; its artifact-side report records signature, icon and reader-compatibility results. Native Windows/Linux execution and publication remain pending. The owner’s app session stays open.

## Inspected source pins and decisions

Source changes retain the repository’s MIT OR Apache-2.0 provenance. The table
credits the upstream PR authors and immutable reviewed heads; adoption is adapted
to this fork rather than a wholesale branch merge.

| PR | Author | Reviewed head | Decision |
|---|---|---|---|
| [#1083](https://github.com/storytold/designcraft/pull/1083) | @moppeeer | `5aed65ff238109960cc0b583d2e54f8b1529b338` | queued: inline object renaming must be ported into Pagenor’s grouped, draggable Layers rows; the source assumes a flat rename-capable listing. |
| [#1089](https://github.com/storytold/designcraft/pull/1089) | @skylartaylor | `42df4d6440f9085a82bd50d8d4c59a30ef85f14a` | adapted: Reject export pages before narrowing; retain existing saturating PPTX diagnostics. |
| [#1090](https://github.com/storytold/designcraft/pull/1090) | @marcoarnulfo | `9f204f57eedd3a226e5fcba753a21e26b88c85a5` | adapted: Snapshot text-field focus before panels; Enter/Tab confirmation cannot leak into canvas text. Preserve inline Indic IME and split-pane event ordering. |
| [#1099](https://github.com/storytold/designcraft/pull/1099) | @harrisonoconnorhover | `fbf3132ad05768d23131064dcbe31c0d61642bc3` | adapted: Add A0/A1/A2 to document and Properties presets. |
| [#1117](https://github.com/storytold/designcraft/pull/1117) | @Silvertree2010 | `230b29be60d6506f91c0f3ee7d4fa792750196f0` | adapted: Reject header/body/footer row moves across section boundaries, including cell-drag commands. |
| [#1123](https://github.com/storytold/designcraft/pull/1123) | @moppeeer | `f47cb6bd6a633f6c1cf6397fe66fb48942286dbc` | adapted: Cap UI-dragged floating panel height at 400 points without changing explicit automation layouts. |
| [#1136](https://github.com/storytold/designcraft/pull/1136) | @harrisonoconnorhover | `f78bf721e86d2357180c2578fc76ad400906a526` | queued: master deletion must preserve Pagenor’s explicit default left/right book-master assignments and existing document guards. |
| [#1141](https://github.com/storytold/designcraft/pull/1141) | @moppeeer | `83e789340c8244c3ab0595ea62df109f9e244e0d` | adapted: Color/Swatches show and edit saved new-shape fill/stroke defaults; add shared fill/stroke target and tint controls without recoloring old objects. |
| [#1148](https://github.com/storytold/designcraft/pull/1148) | @y-az | `ec86a8fd3c29f2fa0061c4bf0a104333e6c7bb8e` | queued: assumes the newer overlong-word breaker pass, absent here; no partial breaker or glyph changes retained. |
| [#1149](https://github.com/storytold/designcraft/pull/1149) | @y-az | `e6878edac280597e06274fa7dd04d29e96a36bd3` | adapted: Keep Arabic contextual alternates active when the Latin contextual-alternates switch is off. |
| [#1163](https://github.com/storytold/designcraft/pull/1163) | @MaridWSH | `affce22f4fc0117ff05ff14c0f27f430f19804f8` | covered: Type → Rescan Installed Fonts refreshes the live, scoped, lazy catalogue, invalidates native tables and composition; retain existing refresh/activation regressions instead of replacing the font database. |
| [#1164](https://github.com/storytold/designcraft/pull/1164) | @Hamza0101 | `653f83b8cb21f8ce2cdc873ca712a58c73628352` | covered: retain Pagenor’s stronger OpenGL failure history across successful launches; add a regression proving a failed non-candidate fallback stays disabled. Do not re-enable it just because it is outside the primary candidate list. |
| [#1169](https://github.com/storytold/designcraft/pull/1169) | @Silvertree2010 | `4b520066e3a7d502f46fc4c042b46fb21f00fa33` | queued: depends on a newer primary-frame autoflow/empty-tail model. Existing smart reflow and deliberately deleted-page policy retained. |
| [#1177](https://github.com/storytold/designcraft/pull/1177) | @Silvertree2010 | `81cc1404b8c7335d5838e5a694d1815bfdec7dbb` | adapted: XLSX shared/inline strings exclude phonetic annotation text; retain existing output budgets. |
| [#1182](https://github.com/storytold/designcraft/pull/1182) | @Silvertree2010 | `61387304b66fca571d16f40de32c89c32db1cf06` | adapted: Ignore non-finite Tagged Text sizes. |
| [#1187](https://github.com/storytold/designcraft/pull/1187) | @Silvertree2010 | `3823d7d9420b74c661578db75c0ff2168ef4190d` | adapted: Constrain imported RTF sizes to supported positive bounds. |
| [#1188](https://github.com/storytold/designcraft/pull/1188) | @Silvertree2010 | `2cc21e19b2a0ea6ac48b0ca4f0fa098ccd5fd727` | adapted: Ignore non-finite DOCX spacing/indents and sizes; retain parser limits. |
| [#1193](https://github.com/storytold/designcraft/pull/1193) | @Silvertree2010 | `ec4fbb270fe7555a55c06141a8139fb89fae81bf` | adapted: Filter non-finite IDML numeric properties, tab positions, leading and stroke inputs; native save/reopen regression. |
| [#1196](https://github.com/storytold/designcraft/pull/1196) | @seanth | `8f1f20f24aa773eeb4537dceee4b3cc87d6a8da6` | adapted: Tool view units follow the active document. Pagenor’s Page tool has no upstream width/height overlay to change. |
| [#1199](https://github.com/storytold/designcraft/pull/1199) | @Silvertree2010 | `485a964e94e24f846b6fefa3063750bd9694a20c` | adapted: CLI rejects non-positive/non-finite export scale before creating output. |
| [#1215](https://github.com/storytold/designcraft/pull/1215) | @Silvertree2010 | `77625733445b99760df3773faa8a48318479b6ea` | adapted: Replacing moved math uses local path bounds and preserves the frame transform. |
| [#1216](https://github.com/storytold/designcraft/pull/1216) | @MarekSpodBiedry | `dcb0417b63f58fbbc845adde61c27db0cfcd8720` | adapted: Make dock headers clickable to collapse/expand, retaining grouped panels and dual icon columns. |
| [#1218](https://github.com/storytold/designcraft/pull/1218) | @MarekSpodBiedry | `b13b1ec40860efe2487a0a34d22e945fb49c185b` | adapted: Mapped first imported paragraph is retained at an originally empty insertion point; preserve replacement-paragraph/EOF policies. |
| [#1271](https://github.com/storytold/designcraft/pull/1271) | @Hamza0101 | `0857671f4881499fd8eea0a669f3ab9bf5e26b95` | queued: preview clipping/slug guides need reconciliation with the current spread/canvas transforms; no partial preview implementation retained. |
| [#1272](https://github.com/storytold/designcraft/pull/1272) | @marcoarnulfo | `526eeda545c205014783cbb22d924f67d14b0e18` | adapted: Document Setup interprets bare measures in document ruler units. |
| [#1284](https://github.com/storytold/designcraft/pull/1284) | @Silvertree2010 | `103d7eb3ff87f04fc42fe346c101fcd7e624cd28` | adapted: Clamp stroke/gap tint before storage and native save. |
| [#1289](https://github.com/storytold/designcraft/pull/1289) | @MarekSpodBiedry | `4df5d1639cff3b88ce85145b56daa870051989aa` | adapted: Story Editor style labels follow actual laid-out rows instead of character-count estimates. |
| [#1290](https://github.com/storytold/designcraft/pull/1290) | @MarekSpodBiedry | `6e0b50a819ca24ac21808753d7656aafa971dbd1` | adapted: Preset cards display dimensions in the dialog’s selected units. |
| [#1291](https://github.com/storytold/designcraft/pull/1291) | @MarekSpodBiedry | `551886c133030f5c2878b8114e49f7198d8f8cee` | adapted: Stroke panel edits supported text outline weight/color through type.char, preserving cell handling. Stroke Behind Fill is omitted until its model/renderer backend exists. |
| [#1292](https://github.com/storytold/designcraft/pull/1292) | @MarekSpodBiedry | `0d0fa585a0e49d333dc1d8f8fc92b0a0fbb6af20` | adapted: Reject invalid or overflowing UTF-8 story replacement ranges atomically. |
| [#1297](https://github.com/storytold/designcraft/pull/1297) | @Silvertree2010 | `3b3544477d19f385b3c78a100e233f41cc3ff3a5` | queued: anchored-image page/PPI reporting needs a bounded placement walk compatible with Pagenor’s existing Links color-mode and ICC reporting. |
| [#1299](https://github.com/storytold/designcraft/pull/1299) | @KnottyDyes | `157035aa9e8d26edf8edc1b199a57eecd2ed80cf` | adapted: Validate stroke-type structure before editing; malformed input preserves the document. |
| [#1301](https://github.com/storytold/designcraft/pull/1301) | @Silvertree2010 | `49fc2c120924b88e3d087adece26c57dbf32a12a` | adapted: Clamp ASE RGB/CMYK/Gray values while retaining non-finite rejection. |
| [#1308](https://github.com/storytold/designcraft/pull/1308) | @Silvertree2010 | `352abea9fce5ef73d01761f32ddd9d80150bac72` | adapted: Bound ASE UTF-16 names without leaving half a surrogate or overflowing their length. |
| [#1310](https://github.com/storytold/designcraft/pull/1310) | @Silvertree2010 | `a5ff17356d7c3ffe91f74a970b5aa44e3955a3dc` | adapted: Ignore out-of-range DOCX half-point sizes. |
| [#1318](https://github.com/storytold/designcraft/pull/1318) | @andeecollard | `04f2e8fcb9f7fa63c9134a29b9e2f97e486d726f` | adapted: Mac Settings/Frame Edges/High Quality shortcuts avoid native Hide collisions; metadata remains shared in the engine. |
| [#1319](https://github.com/storytold/designcraft/pull/1319) | @Silvertree2010 | `177ac8674195323f9b248e88aa3c10293b384c6d` | adapted: Retain XML CDATA text with node/event/depth limits and fallible decoding. |
| [#1346](https://github.com/storytold/designcraft/pull/1346) | @aiongg | `03a6043dc524c6f28c11f6c8b1e8ed8c5c17aee1` | adapted: Route native Paste in Place using existing ordered modifier events and Pagenor’s clipboard bridge; preserve native menu deduplication and rebinding. |
| [#1366](https://github.com/storytold/designcraft/pull/1366) | @con3code | `92bcf7c1ae5fa8f7b48ae82bc56b7a868435bd4f` | adapted: Round-trip IDML Q units as Q; warn and fall back to points for unsupported units. |
| [#1368](https://github.com/storytold/designcraft/pull/1368) | @Silvertree2010 | `9887272efbba2d5fb842f57388b43cb63bc271d2` | adapted: Avoid negative-zero pica formatting using the actual displayed rounded value; fix the upstream patch’s unrounded-remainder check. |
| [#1369](https://github.com/storytold/designcraft/pull/1369) | @Silvertree2010 | `0e7e5c3f04d339e2dbc6461aa1e703599c47aaaf` | adapted: Clamp polygon counts before narrowing to u32. |
| [#1376](https://github.com/storytold/designcraft/pull/1376) | @Silvertree2010 | `7aba0e15af4f808818b0094c21ae7365a40c0fbf` | adapted: Saturate large alternating-fill/index count inputs before narrowing. |
| [#1378](https://github.com/storytold/designcraft/pull/1378) | @Silvertree2010 | `d271298362af27a67c419d3a06caeda58d9a5fb5` | adapted: Reject PDF page lists/ranges and invalid/fractional/out-of-range page numbers; retain dialog on error. |
| [#1383](https://github.com/storytold/designcraft/pull/1383) | @Silvertree2010 | `c35dd44f8227b41a98d6d7275987348d3253e643` | adapted: Remember style Preview choice without losing source-document identity guards. |
| [#1389](https://github.com/storytold/designcraft/pull/1389) | @moppeeer | `be9ee05a58a99dafcce72b8432c999019d92318f` | queued: drag opacity slider assumes UI live-preview lifecycle helpers not present here; preserve the current transactional Effects controls. |
| [#1390](https://github.com/storytold/designcraft/pull/1390) | @moppeeer | `9b48a538d5ad0745ffaaedfadc8ce88e95d2da55` | adapted: Properties offers linked/unlinked text-wrap offset controls in document units. |
| [#1394](https://github.com/storytold/designcraft/pull/1394) | @moppeeer | `c0bf260b5731f14044d5f871ad873ce9bb22381d` | adapted: Properties fitting options button and Auto-Fit control; accessible checkbox metadata. |
| [#1396](https://github.com/storytold/designcraft/pull/1396) | @Silvertree2010 | `6641e7c29fe8be823044adfec847d7342a4852e1` | adapted: Handle malformed RTF Unicode integers without overflow or losing fallback handling. |
| [#1397](https://github.com/storytold/designcraft/pull/1397) | @Silvertree2010 | `e1e9923471aaa32ef1a964dd39c487f5d4968cab` | adapted: Escape/unescape special characters in Tagged Text style names. |
| [#1406](https://github.com/storytold/designcraft/pull/1406) | @moppeeer | `83f2f252c7330bc1cef68694600b54761611b63a` | queued: corner-dialog preview assumes shared step-and-repeat/modal preview infrastructure absent here; existing corner command/dropdowns remain. |
| [#1409](https://github.com/storytold/designcraft/pull/1409) | @moppeeer | `39dfbdb36b006f66499c8d0030672089cb2c409a` | adapted: Display centimeter precision to four decimals, preserving existing inch behavior. |
| [#1412](https://github.com/storytold/designcraft/pull/1412) | @Silvertree2010 | `3e8dd4d4dbfd8e3f1254e18d0d9c987dbc8d2932` | adapted: Remember last-created print units while saved/recent presets retain their own units. |
| [#1413](https://github.com/storytold/designcraft/pull/1413) | @moppeeer | `aa66d3455790710109185fb2eb1875c6f40f758e` | queued: broad menu reorganization needs a dedicated review against Pagenor’s added commands and panel groups. |
| [#1414](https://github.com/storytold/designcraft/pull/1414) | @moppeeer | `f66621ff5f8703973677069bce040b002a0b0532` | queued UI follow-up: preview dropdown glyphs and accessible labels require coordinated changes to shared dropdown callers; existing stroke/corner controls retained. |
| [#1419](https://github.com/storytold/designcraft/pull/1419) | @moppeeer | `21320df6f8657d0b72266cf95f0dcc617c483573` | adapted: Properties names text-frame/image/mixed multi-selections and distinguishes a single linked image. |
| [#1420](https://github.com/storytold/designcraft/pull/1420) | @moppeeer | `076de16622e599e2b88e7559671d7e566ded546e` | adapted: Apply To Frame/Text is available for several selected text frames. |
| [#1422](https://github.com/storytold/designcraft/pull/1422) | @Silvertree2010 | `6630d4cf078b89b61275537458fd89c554dc1629` | not applicable: parse_iso_date does not exist in Pagenor’s current variable implementation; no corresponding input path to fix. |
| [#1430](https://github.com/storytold/designcraft/pull/1430) | @moppeeer | `5e1fd65dfc50d64e848ac8ac8fa7b98f29068e12` | queued: anchor point controls depend on newer point-selection/tool state; avoid adding UI without its backend. |
| [#1431](https://github.com/storytold/designcraft/pull/1431) | @moppeeer | `d24ce2cfd37e7b5024c0e1ff2da8b76c25570fd3` | adapted: Swatches show editability, ink type and color-mode glyphs drawn in code; clip long labels before the columns and retain default tints. |
| [#1434](https://github.com/storytold/designcraft/pull/1434) | @moppeeer | `0fd7eb468eb194f0f81260f894342d13b3b4f47f` | adapted: Format platform shortcut modifiers without duplicate Control labels. |
| [#1436](https://github.com/storytold/designcraft/pull/1436) | @moppeeer | `f882053c855252b95f0ed999eda5a441ac629e38` | adapted: Properties supports object-edge/center distribution and fixed spacing; validate derived transforms and keep locked objects/layers unchanged. |
| [#1437](https://github.com/storytold/designcraft/pull/1437) | @moppeeer | `72541208031cd1898d7614a8176a5da13c1e3132` | adapted: Align To button shows a distinct target glyph. |
| [#1439](https://github.com/storytold/designcraft/pull/1439) | @moppeeer | `fa76d6bd7ebda340e86cfdb67c2125c5060f219b` | queued: Queued with #1297: reconcile Links row/details redesign with existing linked-image color mode/ICC, relink safety and panel grouping. |
| [#1441](https://github.com/storytold/designcraft/pull/1441) | @moppeeer | `8c122f8c68759dad3c76b859238bad9262e3fcd5` | queued: reconcile hierarchical Preflight UI with Pagenor’s existing clickable status/error navigation and grouped panels. |
| [#1450](https://github.com/storytold/designcraft/pull/1450) | @moppeeer | `065a3c6efb45472e93bda7af930a24a105585897` | queued UI follow-up: reconcile Effects menus/clear actions with existing Effects panel controls and document-scoped preview lifecycle. |
| [#1451](https://github.com/storytold/designcraft/pull/1451) | @moppeeer | `75d72c69b59a52acc14a1766a534a3f570bb4b87` | queued rendering milestone: per-effect blend modes change persisted effect semantics; require format capability, rendering and interoperability validation. |
| [#1452](https://github.com/storytold/designcraft/pull/1452) | @moppeeer | `161b359c37ce2545724c86fa7985df186c167567` | adapted: Effects Angle fields gain an accessible angle dial using existing dialog transactions. |
| [#1453](https://github.com/storytold/designcraft/pull/1453) | @moppeeer | `d5978356fad6569f8f2181a16798053bb53538d0` | queued rendering milestone: noise/knockout/technique changes require new persisted fields, render verification and import/export coverage. |

## Review evidence

The local output folder contains the screened inventory, per-file blob pins and
patch hashes, unchanged-head recheck, gate logs, UI images and build/signature
evidence. Queued items are not included in the app. In particular, Links/Preflight
redesigns, corner/opacity preview infrastructure, master deletion, point controls
and new persisted rendering effects require separate integration work.
