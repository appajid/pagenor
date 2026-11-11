# Pagenor queued upstream adaptations — 2026-11-11

Local milestone **0.14.0 / P11.25**, following 0.13.1, authorized by Appaji Ambarisha. Preserve Pagenor architecture, branding, native fonts, Indic input, master pages, and existing document compatibility. No application installation or publication.

## Source pins and execution order

| Proposal | Reviewed source head | Status |
| --- | --- | --- |
| [#2395](https://github.com/storytold/designcraft/pull/2395) | `99d9926c4e2dcfbd77dabc54216d754713388fa2` | Implemented; 3 focused recovery tests pass |
| [#2567](https://github.com/storytold/designcraft/pull/2567) | `89b20ba97870ab57f270ec57cf8ec66f326ebe4d` | Implemented; 33 engine and 6 UI regressions pass |
| [#2553](https://github.com/storytold/designcraft/pull/2553) | `95ba558d0df76405e0c73756e34e20f0c6ac6584` | Adapted; final seven-gate check passes |
| [#2554](https://github.com/storytold/designcraft/pull/2554) | `5fe5ff89f6b77c64a5a3893eb82c022ba36d2620` | Adapted; final seven-gate check passes |
| [#2555](https://github.com/storytold/designcraft/pull/2555) | `a99523bef93257a77f40cc1996ea06e03d4faaf2` | Adapted; final seven-gate check passes |
| [#2418](https://github.com/storytold/designcraft/pull/2418) | `2da1bce1cefd5ab7af20dd8d48cdf66ecb53183a` | Adapted; final seven-gate check passes |
| [#2382](https://github.com/storytold/designcraft/pull/2382) | `894209e5a58dc0991dc686f07d55a21c7090b2aa` | Adapted; final seven-gate check passes |
| [#1150](https://github.com/storytold/designcraft/pull/1150) | `0f68281bc1e9631a204a6e8d6c51cd5d56421aec` | Adapted; final seven-gate check passes |
| [#2333](https://github.com/storytold/designcraft/pull/2333) | `45d225c962caa8208be2e87c900bebf6ae862c2b` | Adapted; final seven-gate check passes |
| [#2412](https://github.com/storytold/designcraft/pull/2412) | `cd315908aae3aa7b13f89225cda1af0d69f842a7` | Adapted; final seven-gate check passes |
| [#2544](https://github.com/storytold/designcraft/pull/2544) | `bd51962160aa63c8ccad2fb4d1c35f5637058d1c` | Adapted; final seven-gate check passes |

## Validation notes

- #2395: both ZIP and legacy JSON loaders prune only dangling or foreign story-frame references before validation. Text stays intact and unplaced-story preflight warnings remain available. Linear ownership index includes nested and anchored objects. Three focused regressions pass.
- #2567: adapting wrap/reveal/Enter to the existing view history and find-scope APIs; 33 engine and 6 UI regressions pass, including parent/document transitions, once-only wrapping, Enter behavior and retained field focus. Find/Change inputs now expose their visible accessibility labels.

- #2553: nested-group ancestry and Direct Selection clicks/marquees adapted. All document/tools library tests pass; the combined 98-test canvas suite also passes.

- #2554: grouped text click/drag/multiple-click reaches the existing caret and selection commands, preserving standalone locked-frame text editing. All 111 tools tests pass. The newer upstream table-border drag helper is absent locally, so that unrelated path is not introduced.
- #2555: group drill-down, previous-tool return, Escape parent navigation and matching context-menu targets adapted. Marquee traversal is iterative/depth-bounded and selects shown states only. 112 document tests, 113 tools tests, the engine transition regression and all 98 canvas tests pass.

- #2418: retain fitting on empty frames, apply it on file/place-cursor placement, clear it explicitly and remove it for media/data merge. IDML owns/style-inherits the options. Native capability advances to 8 using Pagenor’s existing current-version writer; older documents still read. 31 native-format, 112 IDML and 11 engine fitting tests pass.

- #2382: Keep with Previous model, IDML and both Keep Options UIs. The older monolithic layout receives the pinned source’s small tracked-deletion helpers instead of a layout rewrite; existing table force-column handling is retained. All 281 composition tests and 112 IDML tests pass (one existing composition ignore). No interface translation catalogs added.

- #1150 prerequisite: paint-spanning shaping uses character clusters when needed, preserves Arabic ligatures, keeps tracking off base/mark boundaries and excludes those boundaries from caret/hit positions. Pagenor’s scoped/native fonts, Indic fallback and composite-note paths remain. Warichu-only hit paths do not exist locally and are outside the requested CJK scope. 290 composition and 109 font tests pass (one existing composition ignore).

- #2333: shape feature-only style changes together, pass per-run ranged features through script/direction/vertical slices, bypass word-cache entries when features are ranged. All 291 composition and 110 font tests pass (one existing composition ignore). Existing Kana behavior is preserved without adding CJK UI features.

- #2412: white/Paper overprint preflight for existing object fills, strokes and stroke gaps, following PDF’s displayed states, left/right masters, anchored objects and exclusion rules. Shared traversal helpers preserve the existing exporter. CJK adornment-overprint fields do not exist locally and are not introduced. Pagenor still warns that PDF overprint flags are unsupported; the warning therefore says “may disappear when overprint is honored,” rather than promising that current PDF export applies them. All eight preflight regressions and 32 PDF library tests pass.

- #2544: per-placement Relink and Go To with strict exclusive selectors, conservative asset-reference counting, source reuse/orphan cleanup and undo. The Links panel follows the selected use after a split, labels source-wide Update All/Embed All, and retains Pagenor’s async status worker, document-bound dialogs, color/ICC metadata and separate linked-story workflow. Go To reveals document/master placements. The initial 13-test Links engine suite and the additional anchored-reference/UI regressions pass in the final workspace run. The new UI regression found stale placement rows during pending worker checks; previews now recompute uses from the current document without filesystem I/O.

## Pagenor-specific completion work

- Added Type → Keep Options using the existing local paragraph dialog, alongside the paragraph-style control. Captured document/selection guards prevent applying Keep/Fitting changes to a different target.
- Added native fitting/paragraph-setting round trips, fitting-dialog target-change refusal, UI Relink target scope and anchored-source preservation regressions.
- Version 0.14.0 is a local source milestone. Native capability 8 protects empty-frame fitting and Keep with Previous from silent loss in older readers. `.pgnr` and legacy `.pagenor` reading remain.
- No external dependency pins changed. No fonts, artwork or user documents added. No app installation, bundle, upload or release publication.
- All eleven reviewed source heads were rechecked online and are unchanged; #2412 is closed, the other ten remain open. Cumulative group/shaping branches are reconciled once rather than duplicated.

## Final verification

All seven workspace gates pass: **3,157 tests, zero failures, seven existing skips; twenty WASM packages**. All 658 UI tests pass (one existing UI skip). Five synthetic offscreen views are inspected. Windows GNU and Linux GNU compile checks pass for the six shared libraries listed in the compact local verification report; native Windows/Linux app execution remains pending.

Temporary test targets, generated fixture documents and screenshots are removed after inspection; compact logs, hashes and provenance remain in this ledger and the local output report.

- Offscreen review: fitting, both Keep Options controls and Links placement/relink views inspected. An unsupported arrow glyph in placement-row labels is replaced by explicit English “Placement 1/2” labels; final verification is rerun after that correction. Arabic lam-alif color-change regression also passes with installed fonts, with no skipped-font diagnostic.
