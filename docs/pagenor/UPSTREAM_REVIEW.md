# Upstream pull-request integration review — P00.2 / P00.3 / P00.4

Scope update (2026-10-09): The owner removed Adobe/dongle activation validation
from pending work and milestone requirements. Earlier validation reports below
retain their historical scope; those pending references are superseded. Existing
native font support is retained, without claiming vendor activation was tested.

Snapshot: 2026-10-09; 53 open pull requests retrieved through GitHub’s API. The public HTML page was older than the API snapshot. This is a bounded snapshot, not a monitor or a claim about future changes. Open PRs are proposals, not upstream releases.

Twelve PRs contribute the initial P00.2 batch. P00.3 adds #154; P00.4 selectively adds the package portion of #155. Adaptations and limits are recorded in provenance and the policy notes. Source patches and regressions were inspected for the adopted changes; the remaining proposals were triaged by their descriptions, changed files, dependencies and conflict with Pagenor work. This table does not claim a full implementation/security audit of every deferred branch. Larger feature branches remain candidates, not automatically rejected.

Pagenor keeps its product names, owner artwork, `.pgnr` default/legacy compatibility, shared Rust layers and `unsafe_code = "forbid"`. Native font sources and recovery state are not replaced. Dot-prefixed families are omitted from pickers per the owner’s updated preference, while backend lookup remains available. No contributor profiles or upstream instruction changes were copied. No GitHub publishing or remote CI is performed.

| PR | Proposal | Decision | Reason |
|---|---|---|---|
| [#178](https://github.com/storytold/designcraft/pull/178) | Compose: reuse previous layouts for eligible undo and redo | Not adopted in this batch | Draft; depends on #177 and stable font publication epochs. Requires measured cache/undo integration with Pagenor’s live fonts. |
| [#177](https://github.com/storytold/designcraft/pull/177) | Memory: release retired layouts and reserve text growth exactly | Not adopted in this batch | Depends on #175; changes cache lifetime, glyph ownership and text allocation. Requires Pagenor document-font/recovery ownership controls. |
| [#176](https://github.com/storytold/designcraft/pull/176) | PDF: reuse image conversions and index page objects | Not adopted in this batch | Depends on #175; overlaps the damaged-image export fix. Needs combined image-cache/per-asset diagnostics and PDF equivalence tests. |
| [#175](https://github.com/storytold/designcraft/pull/175) | Images: use fixed-size chunks in PSD decoding | Already present | Already present: both PSD loops use as_chunks; no duplicate patch. |
| [#172](https://github.com/storytold/designcraft/pull/172) | perf(compose): resolve a story's frames in one walk, not one per frame | Adapted locally in P00.7 from a withdrawn proposal | Current API marks it closed/unmerged and renamed withdrawn (2026-10-10). Independently reviewed: iterative shared traversal, preserved embedded-frame/font/cache behavior, lookup and cache regressions; synthetic warm-cache benchmark improves 11.68× at 1,000 pages. See COMPOSITION_LOOKUP.md. |
| [#171](https://github.com/storytold/designcraft/pull/171) | Paragraph and Character Styles: wire save and delete into the UI (#169) | Not adopted in this batch | Overlaps #144; choose one style-edit/delete design and test nested style references before integrating. |
| [#170](https://github.com/storytold/designcraft/pull/170) | feat(pptx): export documents as editable PowerPoint presentations | Not adopted in this batch | New PowerPoint exporter/crate and embedded-font work; Phase 08 feature, with fidelity and font-licence gates. |
| [#168](https://github.com/storytold/designcraft/pull/168) | Fix Windows startup crash by defaulting to DirectX 12 | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#156](https://github.com/storytold/designcraft/pull/156) | Confine IDML resources and bound package parsing | Not adopted in this batch | Draft, depends on #154. Rejects external IDML resources on non-Unix hosts; unsuitable unchanged for the required Windows workflow. |
| [#155](https://github.com/storytold/designcraft/pull/155) | Validate engine work counts and package filenames | Package portion adopted in P00.4 | Portable names and staged individual output files; preserves .pgnr, fallible serialization and scoped fonts. Adds preparation/preflight, existing-ancestor checks and extension/suffix bounds. Guide/work-count portions remain deferred. See PACKAGE_OUTPUT.md. |
| [#154](https://github.com/storytold/designcraft/pull/154) | Bound native and Office document parsing | Adopted in P00.3 | Shared bounded ZIP/native/Office parsing and matching save limits; quick-xml compatibility tested. Adds Pagenor table/duplicate/root/read budgets and save-error propagation. See IMPORT_LIMITS.md. |
| [#153](https://github.com/storytold/designcraft/pull/153) | Harden temporary files and release inputs | Not adopted in this batch | Recommended next hardening; recovery rewrite overlaps P02.2–P02.4 owned entries/retained snapshots. Port without replacing tested recovery semantics; release inputs separately. |
| [#152](https://github.com/storytold/designcraft/pull/152) | Apply contextual escaping to HTML and EPUB exports | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#151](https://github.com/storytold/designcraft/pull/151) | Harden the authenticated control channel | Adapted locally in P00.5 | Token/framing bounds across desktop/CLI/MCP; Pagenor retains automation off by default, bounds the UI queue and prevents edit replay after lost replies. |
| [#150](https://github.com/storytold/designcraft/pull/150) | Fix relink atomicity and EPS source retention | Not adopted in this batch | Recommended next hardening; extensive links/EPS/format/IDML/PDF changes need combined atomicity and source-retention fixtures. |
| [#149](https://github.com/storytold/designcraft/pull/149) | Import: RTF and Tagged Text keep characters past U+FFFF | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#148](https://github.com/storytold/designcraft/pull/148) | #59: AGENTS.md and CLAUDE.md name the VectorCraft sibling ../vectorcraft | Not adopted in this batch | Not applicable: upstream sibling directory naming does not describe Pagenor ownership or local sibling projects. |
| [#147](https://github.com/storytold/designcraft/pull/147) | PDF: a damaged PNG, GIF or WebP is skipped, not fatal | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#146](https://github.com/storytold/designcraft/pull/146) | UI: closing a document with unsaved changes asks first (#141) | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#145](https://github.com/storytold/designcraft/pull/145) | UI: ruler units by right-click, Language in Properties | Not adopted in this batch | Useful later UI work; ruler/language controls require independent interaction and document undo tests after this reliability batch. |
| [#144](https://github.com/storytold/designcraft/pull/144) | Styles: Character Style Options dialog | Not adopted in this batch | Useful Phase 07 style editor; overlaps #171 and needs nested style propagation/clear-override validation. |
| [#143](https://github.com/storytold/designcraft/pull/143) | UI: one searchable font menu everywhere; the Contextual Task Bar can be moved | Adopted (selected portions) | Shared searchable font menu only; keep backend hidden-font access, document scopes and lazy preview. Dot families omitted per owner preference. Task-bar dragging/pinning deferred. |
| [#142](https://github.com/storytold/designcraft/pull/142) | UI: the Control panel shows its second row | Not adopted in this batch | Useful UI fix; validate object/text second-row layout separately, with responsive toolbar #112. |
| [#139](https://github.com/storytold/designcraft/pull/139) | Docs, About: the Illustrator-class sibling is VectorCraft (#59) | Not adopted in this batch | Not applicable: sibling product promotion/naming is upstream documentation/branding. |
| [#138](https://github.com/storytold/designcraft/pull/138) | Web: AltGr and other named keys no longer type their names (#134) | Not adopted in this batch | Needs adaptation: heuristic drops any multi-character capitalized ASCII alphanumeric Text event. Validate legitimate browser/IME text before filtering. |
| [#137](https://github.com/storytold/designcraft/pull/137) | Commands: huge row and column counts can't exhaust memory | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#136](https://github.com/storytold/designcraft/pull/136) | Images: PSD decode passes clippy on Rust 1.98 | Already present | Already present, same fixed-size PSD chunk substitutions as #175. |
| [#131](https://github.com/storytold/designcraft/pull/131) | macOS DMG: branded window background and icon layout | Not adopted in this batch | Do not copy upstream branded binary artwork or layout metadata. A Pagenor DMG can reuse the packaging approach with owner assets in Phase 10. |
| [#129](https://github.com/storytold/designcraft/pull/129) | Fix IDML Mojikumi spacing, kinsoku composition, and nested text preflight | Not adopted in this batch | Large CJK/IDML/preflight stack overlaps #126. Isolate changes and validate CJK plus planned Indic composition before adopting. |
| [#128](https://github.com/storytold/designcraft/pull/128) | PDF/A keeps placed images; right-to-left tables and columns | Not adopted in this batch | Useful PDF/A and RTL corrections; table/path/layout/PDF semantics need dedicated bidi and image-colour oracles. |
| [#127](https://github.com/storytold/designcraft/pull/127) | CLI: export options after the last export are an error; MCP and README docs match the code | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#126](https://github.com/storytold/designcraft/pull/126) | Fix composite font resolution, missing-glyph diagnostics, and document-wide font replacement | Not adopted in this batch | Useful composite-font/preflight work, but directly overlaps Pagenor’s lazy dynamic/variable/document font fixes. Requires careful independent port in Phase 04. |
| [#125](https://github.com/storytold/designcraft/pull/125) | Data merge phase 3: skip warnings, joins, Excel, and the grid tool | Not adopted in this batch | Large data-merge phase-3 stack depending on phase 2; not a drop-in fix for current phase. Schedule feature/compatibility work. |
| [#124](https://github.com/storytold/designcraft/pull/124) | IDML: preserve anchored text frames and footnote artwork | Not adopted in this batch | Useful IDML anchored text/footnote preservation; spans composition/render/export and needs synthetic interchange/fidelity tests. |
| [#122](https://github.com/storytold/designcraft/pull/122) | Desktop: write log records to standard error and a rotating log file | Adapted locally in P00.6 | Bounded stderr/rotating files, existing panic/recovery diagnostics, closed-stderr safety and token exclusion; no new dependency. |
| [#121](https://github.com/storytold/designcraft/pull/121) | README: install on Gentoo from the ::snakebyte overlay | Not adopted in this batch | Upstream Gentoo package and someone else’s contributor entry; no verified Pagenor package. Do not copy attribution entries. |
| [#120](https://github.com/storytold/designcraft/pull/120) | Preserve typed values in generic command dialogs | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#119](https://github.com/storytold/designcraft/pull/119) | Preserve browser import intent and document targets | Not adopted in this batch | Recommended next reliability integration: asynchronous import intent/target UID and PDF dialog ownership overlap current .pgnr drop/inbox routing. |
| [#118](https://github.com/storytold/designcraft/pull/118) | Honor MCP typing, story, and page contracts | Not adopted in this batch | Recommended next reliability integration: MCP/caret/page contracts need connected and headless parity testing. |
| [#117](https://github.com/storytold/designcraft/pull/117) | Fix browser printing, startup flags, and document MIME | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#116](https://github.com/storytold/designcraft/pull/116) | Images: set SVG text in the shared fonts when the bundled ones lack it | Not adopted in this batch | Useful SVG font resolver, but native memory fonts and variable coordinates need Pagenor-specific tests before sharing source bytes with usvg. |
| [#115](https://github.com/storytold/designcraft/pull/115) | Compose: don't panic when a hyphenated paragraph starts with a no-break letter | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#112](https://github.com/storytold/designcraft/pull/112) | UI: toolbars that don't fit the window switch to two columns or scroll (#57) | Not adopted in this batch | Useful responsive toolbar/accessibility fix; integrate with #142 and inspect multiple desktop/web viewport sizes. |
| [#111](https://github.com/storytold/designcraft/pull/111) | UI: text fields take Select All, Copy, Cut, Paste, Undo and Redo from the Mac menu (#30) | Adopted with Pagenor adaptation | Independent reliability fix adapted to Pagenor names/current architecture; source and regression tests reviewed. |
| [#109](https://github.com/storytold/designcraft/pull/109) | Data merge phase 2: JSON, several sources, filters, and record grids | Not adopted in this batch | Large phase-2 data-merge stack; isolate from its prerequisites and preserve current grid/save/recovery behavior before feature integration. |
| [#106](https://github.com/storytold/designcraft/pull/106) | Compose: fit horizontal text frames by the last baseline | Not adopted in this batch | Stack includes ~5,500 changed lines, beyond its baseline-fit fix. Isolate final behavior from historical commits and verify descenders/keeps/paths/notes. |
| [#98](https://github.com/storytold/designcraft/pull/98) | INDD: clean-room InDesign document (.indd) import | Not adopted in this batch | New INDD parser with clean-room claims; Phase 08 feature. Review parser bounds/provenance and synthetic cross-platform fixtures before adding. |
| [#96](https://github.com/storytold/designcraft/pull/96) | Compose: an empty paragraph takes its mark's character format | Not adopted in this batch | Stack includes ~5,600 changed lines, beyond empty-paragraph fix. Isolate the mark-format correction and check it against existing composition behavior. |
| [#84](https://github.com/storytold/designcraft/pull/84) | Text editing, snapping and text threading (#23) | Not adopted in this batch | Large stacked tool/editing change overlaps local clipboard/undo fixes; isolate individual gestures and preserve current journal semantics. |
| [#83](https://github.com/storytold/designcraft/pull/83) | Layout: first baselines, Jump Object wrap, drop caps, IDML defaults | Not adopted in this batch | Large baseline/wrap/drop-cap/IDML stack; requires layout oracle and existing font metric compatibility before adoption. |
| [#81](https://github.com/storytold/designcraft/pull/81) | Text: kerning survives a colour change (#29) | Not adopted in this batch | Stack includes ~5,600 changed lines, beyond paint-only shaping. Isolate the run-coalescing change and test Indic/Arabic cluster colours. |
| [#80](https://github.com/storytold/designcraft/pull/80) | Show an alert when a file can't be opened or placed | Not adopted in this batch | Useful failure alerts; drop-routing changes overlap P02.6 and async target fix #119. Integrate together to avoid inconsistent routing. |
| [#65](https://github.com/storytold/designcraft/pull/65) | docs: add star history chart to README | Not adopted in this batch | Not applicable: chart advertises upstream star counts rather than Pagenor. |

Selected PR heads/authors and adaptation details are recorded in `PAGENOR_UPSTREAM.md`. A machine-readable snapshot and local evidence are provided with the review deliverable. Deferred hardening should be addressed through coordinated tasks, beginning with parser bounds, control/MCP contracts and recovery-path protection. No roadmap phase exit is implied.

## Local validation

`cargo xtask ci` passes all seven steps: formatting, checks, strict Clippy, tests, asset licences, layers and WASM. Host totals are 883 passed, 0 failed, 3 ignored (29 additional tests); all 16 WASM packages compile. Strict UI-library Clippy checks for Windows GNU and Linux GNU pass. The current macOS development bundle builds and verifies. Two browser Print regressions compile for WASM; a browser runner was unavailable, so execution remains pending. Windows/Linux checks do not establish native platform or AMD-driver behavior.

Before adaptation, synthetic tests reproduced the composition panic, supplementary-character loss in both RTF and Tagged Text, and damaged-image PDF failure. Those regressions now pass. The selected 12 head commits were fetched and all 44 source-file blob identities matched the inspected API patches.

CPU-rendered unsaved-close and shared font-menu layouts were inspected. The actual macOS font picker displayed search, favourites and previews; a native input sequence was interrupted by app-state guards, so native keyboard/accelerator delivery remains unverified. The GPU offscreen renderer could not obtain an adapter; the CPU images are headless verification, not native screenshots. An initial focused build exhausted disk space; removing only generated incremental caches allowed the complete gate to pass. No user documents or recovery files were removed. Actual Adobe/dongle activation testing stays pending, as requested.

## P00.3 follow-up validation

The full local gate passes all seven steps: 907 tests passed, 0 failed, 3 ignored
(24 net additional tests), and 17 WASM packages compile. Windows/Linux strict
UI-library Clippy checks pass. The new archive
foundation and native/DOCX/XLSX budgets are covered by synthetic regressions;
26 existing IDML compatibility tests pass after XML-library unification.
Native save/error/recovery regressions preserve bytes and session state.
[Import limits and scope](IMPORT_LIMITS.md) record the 512 MiB initial archive
policy and the remaining host-buffering, streaming, package and IDML tasks.
A test compilation initially exhausted disk space; generated caches/examples
were removed only. A Unicode fixture's numeric reference was corrected before
the final gate. Deferred PRs remain candidates; no GitHub push or phase exit.

## P00.4 package validation

The selected #155 package portion passes 12 focused macOS tests and the complete
local gate: 916 passed, 0 failed, 3 ignored; all seven steps and 17 WASM packages
pass. Windows/Linux strict UI-library Clippy checks pass; these are compile
checks, not native execution. A synthetic title-escape regression overwrote an owned outside sentinel on
the original implementation and passes after the adaptation. Tests also cover
Unicode/device names, existing/missing redirected parents, report redirects,
regular overwrites/modes, failed staging cleanup, serialization refusal and
licensed/named/fallback font copies. The Windows-specific redirect test remains
unexecuted on macOS. See [behavior and limits](PACKAGE_OUTPUT.md).

The pinned 53-PR snapshot now has 14 selected/adopted proposals (including the
partial #155 adaptation), two already present and 37 deferred. #155's guide and
work-count portions remain candidates. Only stale generated compiler objects
were removed to recover build disk space; no user documents/recovery were removed.
No GitHub push or milestone exit.

## P09.6 editor UI follow-up (2026-10-10)

Reviewed current open and closed requests. Adapted #196/#199/#221/#228/#233/
#226/#231/#217/#219/#224 sequentially, with specific focused tests. Italian #218
is deferred by the owner. Stacked duplicates of previously handled clipboard,
close, styles and toolbar changes were excluded. Source blob identities match
the inspected heads. [Scope and evidence](EDITOR_UI_UPDATES.md); provenance is
in PAGENOR_UPSTREAM.md. Multipage requests are the next review after this group.
