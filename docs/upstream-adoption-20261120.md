# Pagenor 0.15.0 — upstream adaptations, 20 November 2026

Status: implemented and validated locally as P11.29. Local branch `codex/upstream-20261120`.
Base: `c0abe4f91cff43b5917e7a79cfa91442004da545` (verified 0.14.2 Windows-candidate completion notes).

The online scan covers **440 distinct open/closed PRs updated since 11 November 2026**, through **#3048**. Metadata and changed-file/patch inventories for **84 candidates** were inspected against Pagenor. Thirty compatible source scopes are adapted here; this is not an audit of every changed line in all 440 proposals or an assertion that every remaining candidate is unsuitable.

Original MIT OR Apache-2.0 licences and copyright notices remain applicable. Author logins and immutable inspected heads below identify upstream provenance, without changing the generated contributor registry. Pagenor branding, owner asset rights and native capability 8 remain unchanged. No Vectnor edits, user publications, private fonts or Adobe application/install resources are involved.

## Adapted scopes

| PR | Author | Inspected head | Pagenor adaptation |
| --- | --- | --- | --- |
| [#2842](https://github.com/storytold/designcraft/pull/2842) | @MMexicann | `8746a0b2f91c57213bf4f9c50b71f6ddb1a2a4c4` | Reject duplicate article names on rename without mutation. |
| [#2853](https://github.com/storytold/designcraft/pull/2853) | @MMexicann | `72c646323b6bf34b35903281f402a43e3ca27b71` | Require a string for a state rename; malformed input cannot erase its name. |
| [#2854](https://github.com/storytold/designcraft/pull/2854) | @MMexicann | `162edf914c510dbf7d14d6d80bee269161d45242` | Sort and deduplicate transition targets before applying them. |
| [#3033](https://github.com/storytold/designcraft/pull/3033) | @MMexicann | `bc2ca1714ff728a1324af320cf517aa9aef58e71` | Keep attached combining marks inside title-case words; preserve existing Unicode case policy. |
| [#2599](https://github.com/storytold/designcraft/pull/2599) | @autocraft-bot[bot] | `eb9a2d11a99be6155c001cbc13fb7dbbb2b02fb1` | Home/End at a soft-wrap boundary belongs to the following line. |
| [#2588](https://github.com/storytold/designcraft/pull/2588) | @autocraft-bot[bot] | `6450481566e1fcf5c6c30f875135af5f73044353` | Document Find/Change includes footnotes while retaining formatted searches and Find Next state. |
| [#2645](https://github.com/storytold/designcraft/pull/2645) | @dotCooCoo | `19a77da7027d2b9893b329a21636efd0d2f7cc34` | Require the alignment parameter instead of silently applying Align Left. |
| [#2706](https://github.com/storytold/designcraft/pull/2706) | @dotCooCoo | `a89bdce8490b78e6dab182c38ef578e84252ca12` | Footnote carets do not enable or implicitly target outer tables; explicit table IDs retain their existing behavior. |
| [#2982](https://github.com/storytold/designcraft/pull/2982) | @MMexicann | `073d0f92418b3913d82a4bd3c67b7f1ddfe2c9fb` | Reject an entire malformed page-size target list instead of resizing only its numeric entries. |
| [#2773](https://github.com/storytold/designcraft/pull/2773) | @dotCooCoo | `faae2639bb60e93e6e49d86d7be92dcc79e02938` | Advance the document revision when live-caption refresh replaces the snapshot. |
| [#2836](https://github.com/storytold/designcraft/pull/2836) | @MMexicann | `326247b2964bbf0fc477b814f66b6256e533e1f6` | Keep-going scripts run subsequent steps but retain the first failure and exit unsuccessfully. |
| [#2429](https://github.com/storytold/designcraft/pull/2429) | @albarago | `2d01de82ee1be6f8cb732524d2b6ccfb28ad4f79` | Accept commands --json without treating it as a search filter; reject unknown flags. |
| [#2837](https://github.com/storytold/designcraft/pull/2837) | @MMexicann | `14648be35694498fdcee5366b52592b63fe9fb79` | Check all run options before an export can write a file. |
| [#2839](https://github.com/storytold/designcraft/pull/2839) | @MMexicann | `0af31ca1f649e89b47511cc76d9b70dc22de020f` | Return an error when benchmarking a document with no spreads. |
| [#2643](https://github.com/storytold/designcraft/pull/2643) | @dylanmtaylor | `1d7d00ea19077bbea0c4b9c84b33d0a0069a5c34` | Keep at most four PPTX reverse-font maps; preserve lookup results. |
| [#2606](https://github.com/storytold/designcraft/pull/2606) | @r31lin | `ec5e16025653ad4ee4e7c9c445378d5d6d29c348` | A table-edge priority without an actual rule cannot suppress neighboring formatted edges. |
| [#2639](https://github.com/storytold/designcraft/pull/2639) | @dylanmtaylor | `a9796a69c0414e36ff26e0a24a3792a85809938e` | Bound retained glyph outline elements; discard an incomplete clipped outline. |
| [#2037](https://github.com/storytold/designcraft/pull/2037) | @r31lin | `a2d745db009da671102f4ddff99e094cc5ae4dee` | Request the adapter-supported texture/buffer sizes; retain the existing graphics fallback and error handling. |
| [#2845](https://github.com/storytold/designcraft/pull/2845) | @gmrmad1 | `b325589db4566b161b3a1cb7508e936e8036b198` | Frame Fitting crops display and parse in the active ruler units; retain target guards. |
| [#2848](https://github.com/storytold/designcraft/pull/2848) | @gmrmad1 | `625e803d8a2f78477266e8bdf7e9f5c804dc128a` | Move composer options and checkboxes outside the justification percentage grid; retain World-Ready choices. |
| [#2846](https://github.com/storytold/designcraft/pull/2846) | @gmrmad1 | `c7fafdf8d14b5a8e9c1ce14b2006cff84ddb0b67` | Initially focus Columns and support arrow/+/- adjustments; preserve raw dialog edits, mixed-selection intent and the 216-column cap. |
| [#3048](https://github.com/storytold/designcraft/pull/3048) | @dileep2in | `673ee6cb9ac1606e87f658733c618786d684f6d0` | Add eleven Indic pattern languages with complete-grapheme/cluster boundaries and no added visible hyphen; retain the English dictionaries. |
| [#2665](https://github.com/storytold/designcraft/pull/2665) | @dotCooCoo | `ed9980fec243e286368011f7fff16c9a55dd6bd2` | Adapt the embedded raster command, bounded async native Edit > Paste and control payload. Extend ordinary story carets to inline images. Bitmap-only keyboard events require a later frontend adaptation. |
| [#2783](https://github.com/storytold/designcraft/pull/2783) | @dotCooCoo | `2ab4b60f89903bb2e620ed8fc3c1f77f2322f224` | Size pasted image files from their stored resolution, with one point per pixel as fallback; cap both page dimensions. |
| [#3030](https://github.com/storytold/designcraft/pull/3030) | @stillpica | `492df053cb27f3141dc69637351b866160c6cbed` | A group wraps outside text without pushing its own text frames out. |
| [#3031](https://github.com/storytold/designcraft/pull/3031) | @stillpica | `6719aac647cdec9b26a1bb771054b25f2b527db3` | Ignore spaces before a forced line break in both paragraph and single-line alignment. |
| [#3014](https://github.com/storytold/designcraft/pull/3014) | @stillpica | `60dd1b17b96995fa9a13c0fe0e2a801be765e253` | Resolve exact IDML BasedOn references before trimming whitespace. |
| [#2784](https://github.com/storytold/designcraft/pull/2784) | @dotCooCoo | `b61e433e48358c5772f2686791c742ce6b7adc24` | A text area with zero height cannot contain a line; preserve positive-height fixed-baseline behavior. |
| [#2814](https://github.com/storytold/designcraft/pull/2814) | @dotCooCoo | `008f1b613563eb5aeaff83ed85e4009c6d639eaf` | Reject character stroke weights outside 0–800 pt at the shared text/style boundary. |
| [#2808](https://github.com/storytold/designcraft/pull/2808) | @dotCooCoo | `ea2ffb8ee8a8e1bf04ac02eb1c1ede8019d13598` | Match the Character stroke field to the supported 0–800 pt range. |

## Compatibility decisions

- Clipboard: the native menu reads images on one bounded worker, never waits on the interface thread, caps pixels/files, checks the exact document/selection/layer/master-editing mode and releases a timed-out document snapshot. Empty or failed image reads do not delete selected text. Pagenor objects and ordinary Unicode text retain their existing clipboard routes.
- Image paste at an ordinary story caret embeds one inline object as one undo step. Table-cell and footnote image insertion report unsupported atomically. Empty-frame placement respects frame/layer locks; embedded bytes are deduplicated. Stored resolution controls physical size, capped to 60% of page width and height.
- The pinned egui 0.36 frontend does not deliver `Event::PasteImage`: **bitmap-only Ctrl/Cmd+V is not claimed**. Use native Edit > Paste for bitmaps; URL-backed native paste requests and the control image payload are supported. No wholesale frontend upgrade is introduced.
- Indic document languages are English-labelled text language choices, not translated interfaces. Only the eleven MIT Indic pattern sets in exact hypher 0.1.8 are enabled; full pattern permission is in NOTICE. The original English hyphenation dictionaries/cache behavior remains in use. Grapheme and shaped-cluster boundaries constrain Indic points.
- Keep `.pgnr`, legacy opening, scoped/live fonts, lazy catalogues, inline IME, left/right masters, guarded dialogs, grouped panels and the 0.14.2 Windows packaging fixes. No new persisted field requires a native format increase.
- Local source advances to **0.15.0**. Previously verified 0.14.2 Windows candidates remain pinned and unchanged. This pass does not install an app, publish binaries or upload source. The current Mac session stays open.

## Already covered or partial overlap

| PR | Inspected head | Finding |
| --- | --- | --- |
| [#2274](https://github.com/storytold/designcraft/pull/2274) | `90a680b4cb3cfe8552ab7edc1b726a2e3a342b13` | Existing Pagenor page-thumbnails worker already renders asynchronously with exact document/font ownership. |
| [#2832](https://github.com/storytold/designcraft/pull/2832) | `6f21260c1c8779baf8ad76561c2ef37c3d8abebf` | Existing page-thumbnails keys do not depend on caret/selection revisions. |
| [#2607](https://github.com/storytold/designcraft/pull/2607) | `a65597d6d1b26460a92cf8feba9e2583e7856e2b` | Pagenor already supports named table targets without requiring a table selection. |
| [#2638](https://github.com/storytold/designcraft/pull/2638) | `1187cffde1d409069e1f080a0d49a3d46970dc71` | Existing bounded RIFF scanning checks ICC lengths before decoding and caps profile allocations. |
| [#2730](https://github.com/storytold/designcraft/pull/2730) | `063910938832f566a0d73c42a73553ef38844de6` | The lazy system catalogue queries exact installed family/style records; existing live-font tests cover rescans. |
| [#2731](https://github.com/storytold/designcraft/pull/2731) | `f0262ca98584765b2a0bb2ea0fb3b6e3e1688931` | Existing scoped Document Fonts lookup combines styles with installed fonts instead of replacing the entire family. |
| [#2986](https://github.com/storytold/designcraft/pull/2986) | `02474307c1680834fe9d1a94a8594609878bd534` | Existing new-document input clamps to 1–9999 before the usize conversion. |
| [#2733](https://github.com/storytold/designcraft/pull/2733) | `2ecda520c4f83379799975350ba9b7471bd432b1` | Existing TFO caption links are retained, including Columns; this is not a claim that the entire proposal is ported. |

## Reviewed candidates queued for separate adaptation

These remain follow-up work, not completed features. Some are useful but need additional architecture or behavior checks beyond this batch. Translation-only/CJK-only additions, upstream branding/version updates and broad dependency/library migrations were excluded from this selection.

| PR | Author | Inspected head | Reason / follow-up |
| --- | --- | --- | --- |
| [#1370](https://github.com/storytold/designcraft/pull/1370) | @Hamza0101 | `437ff77df093cc3ec9ac1027c594bc93fab2d70f` | Queue with #2696: Pagenor has independent masters, pasteboard content and preview overlays; visibility changes need joint geometry checks. |
| [#1938](https://github.com/storytold/designcraft/pull/1938) | @ashokram-si | `94a890250c7da5f50b01b048c177b56757f66aae` | Selection-behind follow-up with nested groups and Pagenor modifier routing. |
| [#1953](https://github.com/storytold/designcraft/pull/1953) | @alejo-c | `faf9dd8edaf22cd706d61d911c282296c2c4cec4` | Queue a separate timing change with IME/caret regressions; current inline Indic composition must stay intact. |
| [#2229](https://github.com/storytold/designcraft/pull/2229) | @ljkx | `788bb42c566f97ff38dd00adbf19aa8810da8164` | Style-reference rename/delete follow-up across running heads and list definitions. |
| [#2475](https://github.com/storytold/designcraft/pull/2475) | @soro-82 | `f9e48a0f49042e9709e01568ad4fad362541d265` | Partial thumbnail invalidation needs Pagenor-specific worker/cache changes; the existing asynchronous worker is retained. |
| [#2587](https://github.com/storytold/designcraft/pull/2587) | @autocraft-bot[bot] | `9dffb692dea4417650828c3186467b4cff5f4a37` | Useful follow-up: upstream cell parsing differs from Pagenor’s bounded DOCX output accounting and placement style mapping; preserve those limits while adapting it separately. |
| [#2623](https://github.com/storytold/designcraft/pull/2623) | @r31lin | `8fa104cb68461ee4e214eb897c0d387a7ee620ac` | Non-breaking hyphen fallback follow-up with scoped font and export tests. |
| [#2636](https://github.com/storytold/designcraft/pull/2636) | @dylanmtaylor | `9bedaaf8c31146acc74c8175bfc4dac5951eba0b` | Data-merge range expansion follow-up with existing row limits and atomic parse tests. |
| [#2640](https://github.com/storytold/designcraft/pull/2640) | @dylanmtaylor | `a993e4d4d1e7b4a8d061d935dd5e80d95dec91db` | Renderer/filter follow-up; requires image-output checks for the pinned renderer rather than an unchecked patch. |
| [#2650](https://github.com/storytold/designcraft/pull/2650) | @dotCooCoo | `7809f5e9da01c84672333a4f0840233567b6ba83` | Additional paragraph range validation requires the shared style/import boundary audit; not silently applied in this pass. |
| [#2653](https://github.com/storytold/designcraft/pull/2653) | @dotCooCoo | `670ada58c22ffae76c16f53af212e89cdce0cade` | Same thumbnail invalidation work as #2475; do not replace the current document-owned worker. |
| [#2654](https://github.com/storytold/designcraft/pull/2654) | @dotCooCoo | `6d8f26a6beff4c0c5e63e5acf5b5678baefdd69d` | Depends on renderer invalidation changes; retain existing dirty-region behavior in this batch. |
| [#2661](https://github.com/storytold/designcraft/pull/2661) | @dotCooCoo | `6dd47dce6fc09bb9af331e245caa8eb19d1754ed` | Useful Properties layout follow-up; preserve Pagenor’s grouped dock and multi-selection values. |
| [#2672](https://github.com/storytold/designcraft/pull/2672) | @Hamza0101 | `85098772b110362ae4623fa6a23b092e15795f41` | Font substitution diagnostics need tests against Pagenor’s independent scoped and memory-backed font resolver. |
| [#2696](https://github.com/storytold/designcraft/pull/2696) | @dotCooCoo | `5858ccfec1e467459c7498d1190315bd135bf825` | Queue with #1370; do not cull independent master/overflow content using upstream page-only assumptions. |
| [#2701](https://github.com/storytold/designcraft/pull/2701) | @jayzed80 | `3d99096adb6463b39153d3ba86d9942a9576217d` | Save/import follow-up: verify Pagenor’s lazy asset validation and embedded raster limits together. |
| [#2703](https://github.com/storytold/designcraft/pull/2703) | @dotCooCoo | `8ee3c1f7d001f03df18df7ce0ce3affd180b8987` | TFO multi-selection follow-up; preserve existing hydration/confirmation and stale-target safeguards. |
| [#2715](https://github.com/storytold/designcraft/pull/2715) | @jayzed80 | `a8c38eae15305117e15132da1b3a26fda33c8515` | Symbol-font bullet conversion requires local font-resolution and provenance-safe synthetic cases. |
| [#2716](https://github.com/storytold/designcraft/pull/2716) | @jayzed80 | `cc8e08a8f157a5965f89ca10e22b2b3cb9383570` | Endnote formatting follow-up with child-story round trips. |
| [#2729](https://github.com/storytold/designcraft/pull/2729) | @jayzed80 | `ef98f07efe9b3b4fb500d1bd4e8bf25700aebc75` | Inline frame-fit follow-up; retain existing baseline/overflow contracts until targeted checks are available. |
| [#2746](https://github.com/storytold/designcraft/pull/2746) | @AlPaka1234 | `982f845808aa3018a4a2f83846cffa5909b8c5a3` | Window/workspace follow-up; preserve Pagenor’s independent grouped, removable, vertical panel docking. |
| [#2747](https://github.com/storytold/designcraft/pull/2747) | @dotCooCoo | `3893f1017c0ff940387b2b0e644803bb5c02bbdc` | Baseline policy needs coordinated positive-height, rounded-frame, note and vertical-frame checks. |
| [#2762](https://github.com/storytold/designcraft/pull/2762) | @jayzed80 | `d28f479e7e1c8d4d2d8bdb92f934ecc46ac0928f` | IDML transform follow-up with clipped/grouped frame checks. |
| [#2763](https://github.com/storytold/designcraft/pull/2763) | @llsoftware | `5d250f9723347af1b4617cd44b316c8bcf311e7b` | Lab spot-color follow-up across native format, IDML and PDF output. |
| [#2769](https://github.com/storytold/designcraft/pull/2769) | @dotCooCoo | `45be8f53424bd78ae8e690ad8cc11bb98041c884` | Coordinate with #2747 rather than replacing Pagenor’s current baseline-fit policy piecemeal. |
| [#2774](https://github.com/storytold/designcraft/pull/2774) | @dotCooCoo | `e7657b2cf0feb90898f0a2786daf8dde24880840` | IDML object-style inheritance follow-up with cycle/depth limits and native round trips. |
| [#2776](https://github.com/storytold/designcraft/pull/2776) | @dotCooCoo | `d84b569a83d092a275d7c0ee940442232d1d1a2d` | Object stroke validation follow-up; text stroke range is covered by #2814 here. |
| [#2777](https://github.com/storytold/designcraft/pull/2777) | @dotCooCoo | `52e3e8b7bb3d2e7e915eb4110e02642ceea50ded` | Caption association audit across the remaining number fields; existing TFO labelled controls are retained. |
| [#2778](https://github.com/storytold/designcraft/pull/2778) | @dotCooCoo | `0a0039c94da01a79310b1cbe04e990f8054039d3` | No-op table undo follow-up with mixed-cell and explicit-ID tests. |
| [#2786](https://github.com/storytold/designcraft/pull/2786) | @dotCooCoo | `0c2c13b5d894a3f5c0810fb889235781ec5c3db9` | Gesture caption refresh must be reconciled with Pagenor’s guard/history boundary; command revision correction is adapted here. |
| [#2787](https://github.com/storytold/designcraft/pull/2787) | @dotCooCoo | `b5a4e784c8503b2fca6dd6bd82ee9ece0904565e` | Object stroke range/swatch validation follow-up, coordinated with style creation/edit paths. |
| [#2813](https://github.com/storytold/designcraft/pull/2813) | @jayzed80 | `fe4ff124272ab8d3fceee3e5366e2527f512d1d6` | Direct-selection pointer ordering follow-up with nested-group and modifier tests. |
| [#2817](https://github.com/storytold/designcraft/pull/2817) | @jayzed80 | `bbc57df233520bfa996c69e87be5a697d68e4776` | New-page master inheritance needs Pagenor’s automatic left/right master rules preserved. |
| [#2826](https://github.com/storytold/designcraft/pull/2826) | @jayzed80 | `ac66b4ff11ec550d1d58595517be1af1182f4bf5` | Vector gradient paste requires a larger paint/swatches adaptation and export checks. |
| [#2841](https://github.com/storytold/designcraft/pull/2841) | @MMexicann | `d54cccd3847c9324955f59096e877cde6907856c` | Book style-source identity follow-up with chapter-edit and save/reopen tests. |
| [#2851](https://github.com/storytold/designcraft/pull/2851) | @MMexicann | `cf43ba518ba71a252ab5ee9b93b85f1a4e07da8b` | Tracked-change selection follow-up with caret/IME and undo regressions. |
| [#2881](https://github.com/storytold/designcraft/pull/2881) | @AlPaka1234 | `01434ab89959b18634c44d52dc0997b0c7c5afbb` | Delimiter sniffing also changes persisted data-source state; a separate format/merge-panel adaptation is required. |
| [#2921](https://github.com/storytold/designcraft/pull/2921) | @jayzed80 | `a20b4284a73fc4f9afb85d2c6f704dfcc7b91ed5` | Guide release follow-up with focus-loss and live-drag event tests. |
| [#2923](https://github.com/storytold/designcraft/pull/2923) | @jayzed80 | `59b10cae180c2a932803a4d562841ef6f3b6bdbb` | Embedded WMF bitmap support requires importer budgets and malformed-data coverage. |
| [#2987](https://github.com/storytold/designcraft/pull/2987) | @RIESTERSupport | `e4292abe314d361b16aa9ecb6433741115afa8d2` | Tab line-breaking follow-up with caret/hit-test and export checks. |
| [#2988](https://github.com/storytold/designcraft/pull/2988) | @RIESTERSupport | `3ddb9c5250ed9fd68e6c288f25116e8bdc8164a1` | Tab alignment follow-up coordinated with #2987. |
| [#2989](https://github.com/storytold/designcraft/pull/2989) | @MMexicann | `94d340a1227f1b5cb10f5dac0dc1db81c9cba078` | Unicode batch-name limits need platform-specific filesystem and collision tests. |
| [#2990](https://github.com/storytold/designcraft/pull/2990) | @Pazaak | `1baa5ac4c1a0a484a2263e622980d963f653273f` | Large background-export state/panel change based on newer upstream structure; preserve current guarded export/dialog lifecycle. |
| [#2993](https://github.com/storytold/designcraft/pull/2993) | @dotCooCoo | `6a96ef00e92592a53408825efac63f0e8a36ebbb` | Even-odd vector paste needs geometry/boolean-path adaptation and export checks. |
| [#3020](https://github.com/storytold/designcraft/pull/3020) | @RIESTERSupport | `5765bbaca18d5e8459241fcb5af94fad11156999` | Missing-font open diagnostics need reconciliation with existing import warning UI and scoped fonts. |
| [#3040](https://github.com/storytold/designcraft/pull/3040) | @MMexicann | `40978d8700e53dc504b5090585d02475e9e39fb9` | Upstream’s automatic extension append differs from Pagenor’s save path; adapt only after save/async picker checks. |

## Verification

Baseline engine library: 958 passed, zero failures, one existing ignore. New title-case, article-rename, state-rename and transition-target regressions were observed failing before their production corrections.

All seven workspace gates pass: **3,212 tests passed, zero failures, 7 existing ignores; twenty WASM packages**. Both Windows GNU and Linux GNU checks pass for eight shared libraries (doc, fonts, compose, images, format, render, IDML and PPTX). All 19 packaging contracts pass.

Four offscreen views were inspected: focused Columns with increment/decrement controls, compact justification, millimeter Frame Fitting fields, and the embedded raster filling an empty frame. Visual inspection found clipped crop fields; their grid minimum was widened and the final view was checked again.

Integration corrections retain explicit mixed-selection intent when Columns is edited back to its original value, preserve the existing small-import raster limits through a separate bounded clipboard decoder, and cancel delayed image reads when master editing mode changes. Native/control/SVG placement uses the current page/master context. Footnote scope tests retain explicit parent-story isolation while document scope includes notes.

The Mac native test suite and WASM build check pass. The Windows/Linux results above are **library compile checks**, not full desktop builds or native execution; those runtime checks and the affected Windows 10 user's responsiveness retest remain pending. The existing cross-compiler limitation for the full engine's C dependencies remains outside this pass. No installation, GitHub upload or release publication occurs.
