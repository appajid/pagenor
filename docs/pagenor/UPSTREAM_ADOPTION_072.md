# P11.16 — local upstream adaptations, Pagenor 0.7.2

Reviewed open and closed upstream PR metadata on 13 October 2026, through #748.
200 latest-created PR records were fetched; 97 new proposals were found since the
previous #565 checkpoint. Inspected source patches for 33 candidates. This batch
adapts 19 compatible corrections; the inventory below distinguishes unimplemented
proposals from adopted work. All 19 source heads were rechecked after patch download
and were unchanged. Source repository: https://github.com/storytold/designcraft
(MIT OR Apache-2.0); preserve its existing legal provenance. Pagenor owner: Appaji Ambarisha.

## Adapted source pins

| PR | Source author | Inspected head | Local change |
|---|---|---|---|
| [#567](https://github.com/storytold/designcraft/pull/567) | @dotCooCoo | `45f2a2090ba37db7601452e9e2ce0b60b21b10e6` | Swatches: Load Swatches refuses .ase files over 100,000 colors and merges in linear time |
| [#576](https://github.com/storytold/designcraft/pull/576) | @vxkj1211 | `670830655e49cff0890c99ae964cbaa2d3e7210d` | Fix: white text invisible when selected |
| [#634](https://github.com/storytold/designcraft/pull/634) | @aiongg | `84158e36d34003a84206eeba695289e2342f1186` | Document tabs: long names are shortened so the close button stays visible |
| [#637](https://github.com/storytold/designcraft/pull/637) | @webdevtodayjason | `98f1340f61a44f3681d8f2d3cac6298b10be5912` | fix(index): keep level 3 and 4 topics in a run-in index |
| [#638](https://github.com/storytold/designcraft/pull/638) | @webdevtodayjason | `d01fe5049cdbbe932b67716e31dc17cd6650465e` | fix(color): mirror gradient midpoints when reversing |
| [#639](https://github.com/storytold/designcraft/pull/639) | @marcoarnulfo | `de31bd5647f86d0a2c7c45ec1d08bbc7a3eeef73` | fix(fonts): fall back to a font made for Malayalam (part of #628) |
| [#641](https://github.com/storytold/designcraft/pull/641) | @webdevtodayjason | `4dd72b3b0f378012eb95b0633c8b051522f6ee2b` | fix(textimport): keep a leading empty paragraph in Tagged Text |
| [#666](https://github.com/storytold/designcraft/pull/666) | @sanyamsingh001 | `7abcbc745f33daf175af8b09e52ae12400ce5ec8` | Honor quoted fields when detecting data-source delimiters and records |
| [#667](https://github.com/storytold/designcraft/pull/667) | @sanyamsingh001 | `f7adb11cdc31b8997389c0fc7cb4fafab8cb4e7f` | Capture selected formatting when creating a character style |
| [#670](https://github.com/storytold/designcraft/pull/670) | @sanyamsingh001 | `b3ec8dc908bebba44812cce36ccca80a8983bc42` | Match EPUB stylesheet selectors to exported style tags and classes |
| [#672](https://github.com/storytold/designcraft/pull/672) | @sanyamsingh001 | `5ca6a474b93c35c1a42ba2e9755501144d782b1f` | Split every originally selected table cell exactly once |
| [#673](https://github.com/storytold/designcraft/pull/673) | @storybored379 | `5e22a3dbeef4d634a91955dab21e1781e173db43` | fix(import): preserve XLSX numeric and cached text values |
| [#677](https://github.com/storytold/designcraft/pull/677) | @marcoarnulfo | `966b637796bc9a7cf175c066ded41521b7c72e0f` | fix(ui): open the dock's panel menu from its three-line button |
| [#678](https://github.com/storytold/designcraft/pull/678) | @sanyamsingh001 | `fa9c7b08b255bef19577c4830da546f285e44387` | Keep locked objects fixed during distribution |
| [#690](https://github.com/storytold/designcraft/pull/690) | @haranishi | `ed35dc7beb2a719908b75686ef153297dcfbdaf2` | fix: bundle the CLI with the macOS app |
| [#695](https://github.com/storytold/designcraft/pull/695) | @haranishi | `dafc99833ee117f9594fbe49810a82534ee77c15` | fix: validate Move Pages input and prevent panics with fixed spreads |
| [#702](https://github.com/storytold/designcraft/pull/702) | @haranishi | `1b6a03e350e91e73ca9816b1fa449d21640273c0` | fix: use group transforms for HTML and EPUB reading order |
| [#703](https://github.com/storytold/designcraft/pull/703) | @haranishi | `27302d8ea33b44460bca8642584a2bb32da111a8` | fix: show source paths in the Links panel |
| [#704](https://github.com/storytold/designcraft/pull/704) | @noah1400 | `65bb4e3f661fc6541432fbd99d7d40ea33ba4e49` | fix(tables): keep each table anchor in its own paragraph |

## Pagenor integration decisions

- Document tabs retain horizontal scrolling, bidi/Indic labels and existing close/save behavior; long titles use an ellipsis and a full-title tooltip.
- Links rows show full source paths on hover/selection while retaining Pagenor's color-mode and ICC details. Name/path rows both select the same asset.
- Dock menus retain Close Panel, Float Panel and grouping, alongside the close button, in legacy, tabbed, floating and vertical stack layouts.
- Character styles capture selected/caret attributes, without baking paragraph baseline formatting into the style. Existing blank-EOF staging remains.
- Malayalam fallback applies to complete clusters as well as characters, retaining lazy system catalogs and document-scoped fonts.
- Table edit repair retains surviving table identities and gives each anchor a paragraph. Pasting uses fresh IDs, including files with u64::MAX IDs. Regression reproduced the pre-fix invalid story; edit/save/reopen now covered.
- Multi-cell splitting visits each original owner once, preserves adjacent geometry/formatting and row roles, and bounds grid growth before allocating. Undo/Redo and native serialization are covered.
- Move Pages uses one-based displayed positions in the dialog, while engine/control API remains zero-based. Invalid values retain the dialog/document/history. Fixed multi-page spreads return an explicit unsupported error before mutation; index conversion is checked for 32-bit targets.
- ASE import caps colors and merges linearly. Gradient reversal mirrors midpoints; run-in indexes retain levels three/four.
- CSV detection/record parsing honors quoted fields and embedded line endings. XLSX retains tiny numbers and cached formula strings; Tagged Text retains leading empty paragraphs.
- EPUB/HTML style selectors follow exported tags/classes; reading order includes group transforms while preserving explicit article order.
- Selected white text retains glyph color over a dark translucent highlight.
- Mac bundles include and sign the matching CLI before signing the enclosing app. Windows/Linux packaging stays intact.

## Inventory and disposition

“Further evaluation” means metadata reviewed, but not adopted in this batch; it is
not a claim that the proposal has been fully tested or rejected. Larger features
need separate integration and UI verification against Pagenor's existing additions.

| PR | State at review | Proposal | Disposition |
|---|---|---|---|
| [#567](https://github.com/storytold/designcraft/pull/567) | open | Swatches: Load Swatches refuses .ase files over 100,000 colors and merges in linear time | Adapted locally in P11.16 |
| [#570](https://github.com/storytold/designcraft/pull/570) | open | Blend modes don't blend with the paper | Further evaluation; not implemented in this batch |
| [#571](https://github.com/storytold/designcraft/pull/571) | open | Styles: Load Styles from another document or IDML file | Further evaluation; not implemented in this batch |
| [#572](https://github.com/storytold/designcraft/pull/572) | closed | Eyedropper: sample a placed image's colour into the selected text or objects | Further evaluation; not implemented in this batch |
| [#573](https://github.com/storytold/designcraft/pull/573) | closed | Step and Repeat: InDesign's dialog with Create as a grid and Preview | Further evaluation; not implemented in this batch |
| [#575](https://github.com/storytold/designcraft/pull/575) | closed | Effects panel: the level row, Isolate Blending and Knockout Group | Further evaluation; not implemented in this batch |
| [#576](https://github.com/storytold/designcraft/pull/576) | open | Fix: white text invisible when selected | Adapted locally in P11.16 |
| [#577](https://github.com/storytold/designcraft/pull/577) | open | Gradient feather on a group: fade it on screen, and don't hide it in PDF | Further evaluation; not implemented in this batch |
| [#578](https://github.com/storytold/designcraft/pull/578) | open | i18n(es): whole-interface Spanish table (the shared column was mostly English) | Deferred: translation-only change |
| [#580](https://github.com/storytold/designcraft/pull/580) | open | feat(desktop): portable mode, portable.txt next to the executable (#574) | Further evaluation; not implemented in this batch |
| [#583](https://github.com/storytold/designcraft/pull/583) | open | IDML: tracked deleted text takes no room in the layout | Further evaluation; not implemented in this batch |
| [#584](https://github.com/storytold/designcraft/pull/584) | open | Footnotes: notes stack baseline to baseline | Further evaluation; not implemented in this batch |
| [#585](https://github.com/storytold/designcraft/pull/585) | closed | Hyphenation Zone applies to the Single-line Composer only | Further evaluation; not implemented in this batch |
| [#588](https://github.com/storytold/designcraft/pull/588) | open | CMYK blend space: blend modes blend in CMYK on screen | Further evaluation; not implemented in this batch |
| [#589](https://github.com/storytold/designcraft/pull/589) | open | (fix)Small frame wrap for overset small text boxes | Further evaluation; not implemented in this batch |
| [#590](https://github.com/storytold/designcraft/pull/590) | closed | Move a placed graphic inside its frame: content grabber and Direct Selection | Further evaluation; not implemented in this batch |
| [#591](https://github.com/storytold/designcraft/pull/591) | open | Double-click a frame handle to fit the frame to its content; Fit Frame to Content fits text frames | Further evaluation; not implemented in this batch |
| [#592](https://github.com/storytold/designcraft/pull/592) | open | Toolbox Fill/Stroke proxy works on text; Shift+X, D, X, / and J | Further evaluation; not implemented in this batch |
| [#598](https://github.com/storytold/designcraft/pull/598) | open | Canvas: arrow keys and Tab keep keyboard focus on the canvas | Further evaluation; not implemented in this batch |
| [#599](https://github.com/storytold/designcraft/pull/599) | open | Place: with no frame selected, the graphic loads into the place cursor | Further evaluation; not implemented in this batch |
| [#600](https://github.com/storytold/designcraft/pull/600) | closed | Toolbox: Shift+X, D, X and / for the Fill/Stroke proxy | Further evaluation; not implemented in this batch |
| [#601](https://github.com/storytold/designcraft/pull/601) | open | Tables: drag row and column borders with the Type tool | Further evaluation; not implemented in this batch |
| [#604](https://github.com/storytold/designcraft/pull/604) | open | MCP: confine file access with --automation-read-root and --automation-write-root | Further evaluation; not implemented in this batch |
| [#606](https://github.com/storytold/designcraft/pull/606) | open | Styles: selection capture accepts deleted-swatch fills | Further evaluation; not implemented in this batch |
| [#607](https://github.com/storytold/designcraft/pull/607) | closed | MCP: reconnect closed sockets before sending the next request | Further evaluation; not implemented in this batch |
| [#608](https://github.com/storytold/designcraft/pull/608) | open | UI: spring-loaded zoom (Cmd+Space / Cmd+Alt+Space) | Further evaluation; not implemented in this batch |
| [#610](https://github.com/storytold/designcraft/pull/610) | open | Links: placed Word, RTF and text files can stay linked | Further evaluation; not implemented in this batch |
| [#611](https://github.com/storytold/designcraft/pull/611) | open | Add a Nix flake (NixOS, GNOME and Hyprland) | Further evaluation; not implemented in this batch |
| [#613](https://github.com/storytold/designcraft/pull/613) | closed | fix(save): preserve live state when persistence fails | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#614](https://github.com/storytold/designcraft/pull/614) | closed | fix(export): replace desktop and CLI outputs atomically | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#615](https://github.com/storytold/designcraft/pull/615) | closed | fix(book): stage chapter updates before saving | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#616](https://github.com/storytold/designcraft/pull/616) | closed | fix(recovery): retain recovered checkpoints until save or close | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#617](https://github.com/storytold/designcraft/pull/617) | closed | fix(recovery): preserve documents on orderly app exit | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#618](https://github.com/storytold/designcraft/pull/618) | open | Paste vector art from Illustrator, VectorCraft or Figma as editable paths | Further evaluation; not implemented in this batch |
| [#623](https://github.com/storytold/designcraft/pull/623) | closed | fix(macos): make Dock Quit cancelable | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#627](https://github.com/storytold/designcraft/pull/627) | closed | test(recovery): make failed republishing fixtures deterministic | Further evaluation: compare with Pagenor persistence/recovery/quit safeguards before rewriting |
| [#629](https://github.com/storytold/designcraft/pull/629) | open | Complete the Arabic (العربية) catalog: every missing key translated | Deferred: translation-only change |
| [#630](https://github.com/storytold/designcraft/pull/630) | closed | Pen tool: continue an open path from either end point; Reverse Path | Further evaluation; not implemented in this batch |
| [#631](https://github.com/storytold/designcraft/pull/631) | open | Windows: dragging across monitors with different scales doesn't grow or crash the window | Further evaluation: host/dependency changes need native hardware/Windows testing |
| [#632](https://github.com/storytold/designcraft/pull/632) | open | Text frames: auto-size grows and shrinks frames to fit their text | Further evaluation; not implemented in this batch |
| [#633](https://github.com/storytold/designcraft/pull/633) | open | Text frames: Balance Columns evens out the last frame's columns | Further evaluation; not implemented in this batch |
| [#634](https://github.com/storytold/designcraft/pull/634) | closed | Document tabs: long names are shortened so the close button stays visible | Adapted locally in P11.16 |
| [#635](https://github.com/storytold/designcraft/pull/635) | open | Pages panel: double-clicking a parent shows that parent; parent thumbnails | Further evaluation; not implemented in this batch |
| [#636](https://github.com/storytold/designcraft/pull/636) | closed | Type on a path: drag the start and end brackets | Further evaluation; not implemented in this batch |
| [#637](https://github.com/storytold/designcraft/pull/637) | open | fix(index): keep level 3 and 4 topics in a run-in index | Adapted locally in P11.16 |
| [#638](https://github.com/storytold/designcraft/pull/638) | closed | fix(color): mirror gradient midpoints when reversing | Adapted locally in P11.16 |
| [#639](https://github.com/storytold/designcraft/pull/639) | closed | fix(fonts): fall back to a font made for Malayalam (part of #628) | Adapted locally in P11.16 |
| [#640](https://github.com/storytold/designcraft/pull/640) | closed | feat(type): kern the pair at the caret with Alt+Left/Right | Further evaluation; not implemented in this batch |
| [#641](https://github.com/storytold/designcraft/pull/641) | closed | fix(textimport): keep a leading empty paragraph in Tagged Text | Adapted locally in P11.16 |
| [#642](https://github.com/storytold/designcraft/pull/642) | closed | fix(idml): text strokes aligned outside stay behind the fill | Further evaluation; not implemented in this batch |
| [#643](https://github.com/storytold/designcraft/pull/643) | open | The Effects dialog: Object › Effects and the panel's fx button, with Preview | Further evaluation; not implemented in this batch |
| [#645](https://github.com/storytold/designcraft/pull/645) | open | feat(i18n): translate the French interface | Deferred: translation-only change |
| [#647](https://github.com/storytold/designcraft/pull/647) | closed | Type on a path: the Type tool clicks and selects along the curve; caret and highlight follow it | Further evaluation; not implemented in this batch |
| [#648](https://github.com/storytold/designcraft/pull/648) | open | Eyedropper: a sampled colour goes to the stroke when the Stroke box is in front | Further evaluation; not implemented in this batch |
| [#649](https://github.com/storytold/designcraft/pull/649) | closed | feat(text): select sentence, paragraph and story on 3, 4 and 5 clicks | Further evaluation; not implemented in this batch |
| [#650](https://github.com/storytold/designcraft/pull/650) | closed | Fix Intel Mac crash: probe Metal shader language version support | Further evaluation: host/dependency changes need native hardware/Windows testing |
| [#655](https://github.com/storytold/designcraft/pull/655) | open | Autoflow: one frame per column, as the page's columns are set | Further evaluation; not implemented in this batch |
| [#656](https://github.com/storytold/designcraft/pull/656) | open | Type: Alt+Left/Right kerns at the caret and tracks a selection | Further evaluation; not implemented in this batch |
| [#657](https://github.com/storytold/designcraft/pull/657) | open | Text: stroke behind fill | Further evaluation; not implemented in this batch |
| [#658](https://github.com/storytold/designcraft/pull/658) | open | Word import: bullets and numbering from numbering.xml | Further evaluation; not implemented in this batch |
| [#659](https://github.com/storytold/designcraft/pull/659) | open | Images: CMYK and profiled photos show in their colours | Further evaluation; not implemented in this batch |
| [#660](https://github.com/storytold/designcraft/pull/660) | open | Color Picker and Color panel: typed values stay; colours are read on a fixed scale | Further evaluation; not implemented in this batch |
| [#661](https://github.com/storytold/designcraft/pull/661) | open | Text frames: lines follow an oval or polygon frame's shape | Further evaluation; not implemented in this batch |
| [#662](https://github.com/storytold/designcraft/pull/662) | closed | fix(cmd): reject unknown spread values instead of falling back to page 1 | Further evaluation; not implemented in this batch |
| [#663](https://github.com/storytold/designcraft/pull/663) | closed | fix(cli): accept a UTF-8 BOM in scripts and document the syntax in script --help | Further evaluation; not implemented in this batch |
| [#664](https://github.com/storytold/designcraft/pull/664) | open | macOS: open the window on Intel Metal by skipping indirect-call validation | Further evaluation: host/dependency changes need native hardware/Windows testing |
| [#666](https://github.com/storytold/designcraft/pull/666) | closed | Honor quoted fields when detecting data-source delimiters and records | Adapted locally in P11.16 |
| [#667](https://github.com/storytold/designcraft/pull/667) | closed | Capture selected formatting when creating a character style | Adapted locally in P11.16 |
| [#668](https://github.com/storytold/designcraft/pull/668) | closed | Move objects past same-layer siblings with Arrange | Further evaluation; not implemented in this batch |
| [#669](https://github.com/storytold/designcraft/pull/669) | closed | Draw inherited leaders for right-indent tabs | Further evaluation; not implemented in this batch |
| [#670](https://github.com/storytold/designcraft/pull/670) | closed | Match EPUB stylesheet selectors to exported style tags and classes | Adapted locally in P11.16 |
| [#671](https://github.com/storytold/designcraft/pull/671) | closed | Preserve IDML index reference source sort keys | Further evaluation; not implemented in this batch |
| [#672](https://github.com/storytold/designcraft/pull/672) | closed | Split every originally selected table cell exactly once | Adapted locally in P11.16 |
| [#673](https://github.com/storytold/designcraft/pull/673) | open | fix(import): preserve XLSX numeric and cached text values | Adapted locally in P11.16 |
| [#674](https://github.com/storytold/designcraft/pull/674) | open | Parents: a parent can be based on another parent | Further evaluation; not implemented in this batch |
| [#675](https://github.com/storytold/designcraft/pull/675) | open | Styles: object styles carry effects | Further evaluation; not implemented in this batch |
| [#677](https://github.com/storytold/designcraft/pull/677) | closed | fix(ui): open the dock's panel menu from its three-line button | Adapted locally in P11.16 |
| [#678](https://github.com/storytold/designcraft/pull/678) | closed | Keep locked objects fixed during distribution | Adapted locally in P11.16 |
| [#680](https://github.com/storytold/designcraft/pull/680) | closed | Check transformed image resolution during preflight | Further evaluation; not implemented in this batch |
| [#682](https://github.com/storytold/designcraft/pull/682) | open | Embed craft-fonts by default via .cargo/config.toml [env] | Declined as proposed: retain optional craft-fonts build input |
| [#683](https://github.com/storytold/designcraft/pull/683) | open | New Document: Preview | Further evaluation; not implemented in this batch |
| [#687](https://github.com/storytold/designcraft/pull/687) | open | Bump quick-xml to 0.41: linear-time attribute check on .docx/.xlsx import (RUSTSEC-2026-0194, -0195) | Already covered: workspace uses quick-xml 0.41 |
| [#688](https://github.com/storytold/designcraft/pull/688) | open | Create Outlines makes one compound path per line, so Ungroup separates the lines | Further evaluation; not implemented in this batch |
| [#689](https://github.com/storytold/designcraft/pull/689) | open | Ungroup leaves frames with pasted-in content whole; name them by their shape | Further evaluation; not implemented in this batch |
| [#690](https://github.com/storytold/designcraft/pull/690) | open | fix: bundle the CLI with the macOS app | Adapted locally in P11.16 |
| [#691](https://github.com/storytold/designcraft/pull/691) | open | Copy a selected placed graphic on its own, so Paste Into puts the whole image in place | Further evaluation; not implemented in this batch |
| [#692](https://github.com/storytold/designcraft/pull/692) | open | test: make the MCP reconnect test socket blocking on macOS | Further evaluation; not implemented in this batch |
| [#693](https://github.com/storytold/designcraft/pull/693) | open | docs: add contribution coordination guidance | Further evaluation; not implemented in this batch |
| [#695](https://github.com/storytold/designcraft/pull/695) | open | fix: validate Move Pages input and prevent panics with fixed spreads | Adapted locally in P11.16 |
| [#698](https://github.com/storytold/designcraft/pull/698) | open | Effects apply to frames with pasted-in content (Inner Glow, Basic Feather, shadows…) | Further evaluation; not implemented in this batch |
| [#699](https://github.com/storytold/designcraft/pull/699) | open | Resize a selected placed graphic by its own handles (blocked by #415, #550) | Further evaluation; not implemented in this batch |
| [#701](https://github.com/storytold/designcraft/pull/701) | open | docs: fix the README font integration link | Further evaluation; not implemented in this batch |
| [#702](https://github.com/storytold/designcraft/pull/702) | open | fix: use group transforms for HTML and EPUB reading order | Adapted locally in P11.16 |
| [#703](https://github.com/storytold/designcraft/pull/703) | open | fix: show source paths in the Links panel | Adapted locally in P11.16 |
| [#704](https://github.com/storytold/designcraft/pull/704) | open | fix(tables): keep each table anchor in its own paragraph | Adapted locally in P11.16 |
| [#746](https://github.com/storytold/designcraft/pull/746) | closed | VIB-11819 chore: release a Linux x86_64 tarball on tag | Declined: fork-specific Linux-only release replaces cross-platform pipeline |
| [#748](https://github.com/storytold/designcraft/pull/748) | open | Effects panel: Object/Stroke/Fill/Text levels, fx badge, Clear All and Remove Effects (blocked by #643) | Further evaluation: depends on #643 Effects dialog; do not import stacked branch wholesale |

## Validation and delivery

All seven workspace gates passed: fmt, check, Clippy with warnings denied, tests, assets, layers and WASM. 2,647 tests passed, zero failed, 7 existing skips; twenty WASM packages checked. Bash syntax checks passed for both Mac packaging scripts.

Native Mac inspection used an independent profile/endpoint and synthetic content: long Latin/Telugu document tabs retain visible close controls, selected white text remains readable, and same-named PNG Links rows show source paths plus RGB metadata. No proprietary owner document was edited. The running app windows remain open.

Offscreen full-window renders of the final shared-menu code were inspected: tabbed and vertical layouts, the open Pages dropdown (page insertion, deletion, master actions, Close/Float/Group), and the Move Pages dialog with Page (1–3) hints. The docking regression explicitly checks insertion and closing in both layouts; all twenty-eight docking tests pass. Native serialization and Undo/Redo regressions cover table identity, splitting and page movement. The document suite reproduced #704's invalid-anchor error before the fix.

An optimized Apple-silicon Mac bundle with the matching CLI is being assembled from the final local commit. Initial local bundle strict/deep signature verification passed; final commit-pinned verification will be recorded in BUILD_STATUS.json. This is a local build; native Windows/Linux execution and distribution are not part of this pass.

No upstream workflow, private source documents/fonts, translation catalogs or new
branding were imported. Proprietary owner sessions are left open. No GitHub push,
PR update or public release is part of this local request. Native Windows/Linux
execution is not claimed by a macOS build or WASM compilation.

The separate Anu6/Anu7 Unicode-conversion work remains pending validation of matching
mappings; the older owner-authored Anupama tables are not applied to these versions.
