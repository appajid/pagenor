# Pagenor development phases

Current version: **0.7.0** (P11.13), published desktop development preview. 0.6.0
and v0.3.0 packages remain intact.

P11.13 completes all 26 new selected proposals and six incremental updates.
All seven gates pass: 2,574 tests, zero failures, seven existing ignores; twenty
WASM packages. Native 0.7.0 Mac UI inspection covers saved EOF typing and Print
scaling, in addition to the earlier UI/graphics/split-view checks. Native format 5
reads prior documents; older apps reject the new capability rather than lose it.
Mac/Windows/Linux quality and Rust 1.95 gates pass. All desktop packages are
published and their 20 payload checksums verified; all three Windows installer
checks and both Linux/Flatpak native checks passed. Signing/notarization and
interactive Windows/Linux/printer validation remain open.
[Implementation notes](docs/pagenor/UPSTREAM_ADOPTION_070.md) and
[distribution tracker](docs/pagenor/DESKTOP_RELEASE_070.md).

P11.12 completes native Windows x64/x86/ARM64 MSI/portable and Linux x86_64/ARM64
AppImage/Debian/RPM/tar/Flatpak builds. Windows installation, architecture,
document associations/icons, installed CLI export and uninstall passed for all
three architectures. Linux/Flatpak native CLI execution passed on both.
All seven local Mac and native Windows/Linux quality gates passed; Rust 1.95
checks passed. A test-only Mac OpenSSL compatibility follow-up changes no
production binaries; its hosted validation is tracked separately.
Matching universal Mac DMG/CLI packages passed signature and ARM CLI checks.
Windows signing, Mac notarization, native Intel Mac GUI and interactive
Windows/Linux editor/printer validation remain open. Default-branch integration
is in PR #3; release binaries use the verified tagged build-branch commit.
[Distribution notes and run evidence](docs/pagenor/DESKTOP_RELEASE_060.md).

P11.11 completes the twenty UI-first proposals through #491, with compatible later
reliability/publishing fixes. The implementation batch passes all seven gates
(2,306 tests passed, seven ignored; twenty WASM packages), native Mac UI inspection
and Windows/Linux font/composition/tool compile checks. Broader Affinity/master-panel/
titlebar work remains open; native desktop distribution is recorded in P11.12.
[Implementation and provenance](docs/pagenor/UPSTREAM_ADOPTION_20261012.md).

P11.10 adds experimental PageMaker 6.5/7 opening through the optional installed
`pmd2raw` helper, recovering editable native pages/text frames. Ramsha has five
pages/five frames; a PMD sample has nine pages/31 frames. Full-fidelity graphics,
threading/layers and legacy Unicode mapping remain open.
[Scope and evidence](docs/pagenor/PAGEMAKER_IMPORT.md).

P11.9 adds colour mode and selected-image ICC profile details to Links, using
stored image headers and preserving preview/source distinctions. Shared Rust
queries retain document/undo state. [Notes](docs/pagenor/LINKS_COLOR_INFO.md).

P11.8 makes the bottom-bar error count/indicator open the live Preflight panel,
and connects Window → Output → Preflight to the same details. Issue selection
and page navigation are retained. [Notes](docs/pagenor/PREFLIGHT_STATUS.md).

P11.7 restores portfolio INDD images and the source layer stack, including empty
user layers, and preserves multi-column count/gutter. The owner-file probe
retains 22 pages, 57 image placements, six user layers and all used story text.
Exact typography, master artwork and effects remain reader-subset limits.
[Portfolio evidence](docs/pagenor/INDD_PORTFOLIO.md).

P11.6 corrects INDD multi-leaf/mixed-encoding text recovery, font-style alias
precedence, paragraph spacing/justification and default page labels. The owner
file retains all 34 stored pages, 33 linked frames and all story text without
missing glyphs. Exact pagination remains unfinished: 31 populated frames versus
33 in the source PDF. All seven gates pass (1,898 tests, six existing ignores;
20 WASM packages). [Recovery evidence](docs/pagenor/INDD_RECOVERY.md).

P11.5 adds local IDML/INDD Indic import corrections on the existing 0.5.2 baseline:
language labels, whole-cluster fallback and bounded Unicode/style-run decoding.
All seven local gates pass (1,889 tests, six existing ignores; 20 WASM packages).
Both importers pass a real installed Telugu-font render/native-round-trip probe
and exact PDF Unicode extraction, including joiners. Real-source compatibility
and native Windows/Linux execution remain separate validation requirements.
Importers remain built-in Rust modules using the shared text engine. See
[the import notes](docs/pagenor/INDIC_IMPORTS.md) for validation and remaining limits.
PageMaker/Anu legacy-font mapping awaits the owner's font/sample; no new format
support or phase completion is claimed.

Updated: 2026-10-12. Source: Appaji Ambarisha’s `Pagenor_Development_Phases.docx`.
This tracker records verified work against the supplied roadmap. Existing features
in `ROADMAP.md` are implementation history, not proof that a phase exit gate passes.

P00.9 audits upstream v0.6.0: 45 changes already covered, 7 adapted, 16 still
pending and 3 deferred/out of scope. All seven gates pass (1,548 tests, 3 ignored;
20 WASM packages), plus 12 packaging contracts. The v0.6.0 adoption is not
complete. [Audit and next items](docs/pagenor/UPSTREAM_060_AUDIT.md).

P11 completes the selected non-CJK queue (47 proposals plus #295/#298 prerequisites),
with per-change provenance and adaptation notes. All seven `cargo xtask ci` gates pass: 1,856 tests passed, 6 ignored, no failures; 20 WASM packages. The 12 packaging contracts, ShellCheck and actionlint pass. The local macOS app is built and inspected separately. Windows/Linux native GUI and printing execution, signed/notarized distribution and GitHub publication remain milestone work.
[Completed local queue](docs/pagenor/UPSTREAM_EDITOR_ADOPTION.md). The earlier
v0.6.0 audit is historical; #295/#297/#298 are now completed, leaving 13 of its
previously pending changes outside this queue. No entire roadmap phase is declared complete.

P11.1 continues editor usability with visible scrolling, Arrow table selection,
cell borders, functional/closable vertical panels and a larger bottom bar.
[Local change notes](docs/pagenor/EDITOR_USABILITY.md).

P11.2 corrects document-window closing to return to Welcome and widens the right
panel icon bar. [Welcome maintenance notes](docs/pagenor/WELCOME_RECENT_DOCUMENTS.md).

P11.3 makes existing master pages usable for shared content, guides, page numbers,
headers/footers and side-specific margins/columns. Left/right masters repeat on
matching book pages; a chosen default persists for document growth. Existing local
page overrides are retained. All seven gates pass (1,875 tests passed, 6 ignored; 20 WASM packages), plus
12 packaging contracts. A separate macOS arm64 build preserves the running
session. [Master-page notes](docs/pagenor/MASTER_PAGES.md).

P11.4 adds an independent one/two-column right toolbar switch, remembered in
preferences and custom workspaces, with the same grouped panels and scrolling.
All seven local gates pass (1,878 tests, six ignored; 20 WASM packages).
One/two-column renders and a separate macOS build were checked.
[Right toolbar notes](docs/pagenor/RIGHT_TOOLBAR_COLUMNS.md).

Owner workflow: make and test changes locally on macOS, keeping the shared code
compatible with Windows/Linux and WASM. Publish to GitHub and run the native
macOS/Windows/Linux matrix at a completed milestone; routine task commits stay
local. On 2026-10-10 the owner additionally authorized publishing the current
0.3.0 development preview and source to GitHub; native package and phase exit
gates remain open. A task completion does not by itself complete a roadmap milestone.

| Phase | Priority | Work | Status / evidence |
|---|---|---|---|
| 00 | P0 | Repository, security and provenance | In progress. P00.8 audits all 34 upstream v0.4.0 release-note changes and records the accumulated application checkpoint as 0.3.0. The 32 inherited application/workflow changes remain; #132/#135 distribution work is adapted in P10.2, completing source coverage of all 34 listed items while keeping version 0.3.0 per the owner. [Per-item audit](docs/pagenor/UPSTREAM_040_AUDIT.md). Local quality baseline and CI setup: [report](docs/pagenor/PHASE_00_REPORT.md). P00.2 triages 53 upstream PRs and adapts selected reliability changes from 12 with pinned provenance; [review decisions](docs/pagenor/UPSTREAM_REVIEW.md). P00.3 adds upstream #154 for bounded native/Office imports and matching save limits, with error propagation that preserves files/recovery; [policy and scope](docs/pagenor/IMPORT_LIMITS.md). P00.4 adapts the package portion of #155 for portable filenames, redirect refusal and staged individual files, with preflight/preparation before writes; [scope](docs/pagenor/PACKAGE_OUTPUT.md). Full local gate for P00.4: 916 passed, 3 ignored; 17 WASM packages. P00.5 adapts upstream #151 with authenticated desktop/CLI/MCP control, bounded requests/UI queue and no replay after lost replies. All seven local gates pass: 1,262 tests, 3 ignored; 20 WASM packages. Native Windows/Linux execution remains a milestone check; [scope and evidence](docs/pagenor/CONTROL_HARDENING.md). Windows/Linux strict UI-library Clippy checks pass locally; native platform execution remains separate. P00.6 adapts upstream #122 with bounded desktop diagnostics and rotating files; control tokens stay outside file logs and existing recovery hooks remain. All seven local gates pass: 1,282 tests, 3 ignored; 20 WASM packages. Logger-only production strict Clippy passes for Windows/Linux; native execution remains a milestone check. [Logging scope](docs/pagenor/DESKTOP_LOGGING.md). P00.7 adapts the performance idea from now-withdrawn #172 using iterative batched frame lookup, preserving embedded frames/font scope/cache semantics. Four regressions and equivalent-output synthetic warm-cache measurements pass (11.68× at 1,000 pages with 50 frames/story). All seven local gates pass: 1,286 tests, 3 ignored; 20 WASM packages. Windows/Linux document/composition library strict Clippy and the macOS executable build pass; native platform execution remains a milestone check. [Lookup scope](docs/pagenor/COMPOSITION_LOOKUP.md). Own repository and local remote are configured. Remote CI passed macOS, Linux, Rust 1.95, packaging lint and FreeBSD. Windows exposed a path-separator assertion in the data-merge test; the assertion now compares paths and awaits its rerun. Private-repository branch protection and release supply-chain checks remain open. |
| 01 | P0 | Font discovery and catalogue | In progress. Core Text paths, all SFNT extensions, lazy named instances and duplicate handling implemented. Host comparison covered by tests; Diagnostics/rescan commands and automatic native refresh are implemented and pass the local gate. Windows registry and Linux/FreeBSD Fontconfig adapters are implemented (P01.2); cross-target compile and strict Clippy checks pass. Native Linux adapter tests passed. P01.3 removes stale loaded families/styles from the picker while preserving rendering/document handles. P01.4 fixes newly discovered style/instance resolution and supplements Core Text collections with PostScript-name discovery; hidden-path/extensionless-file coverage passes locally. P01.5 adds lazy Core Text table access for native fonts without readable files, with withdrawal/reactivation cache tests. P01.6 fixes lazy replacement of loaded installed files, including collection/style changes, identical-byte reuse and stale loader rejection. P01.7 makes forced/manual rescans invalidate unchanged native identities for lazy reload, preserving existing handles and rejecting stale requests. P01.8 adds bounded polling revalidation for unchanged native identities, including menu/table changes and recovered access. P01.9 aligns missing-style diagnostics with lookup normalization and custom base styles; independent macOS inventory coverage finds all 272 SFNT families, with two legacy non-SFNT families unsupported. P01.10 aligns Unicode family lookup/deduplication with menu grouping and preserves document precedence through composition. P00.2 shares searchable pickers and omits dot-prefixed families from the menu per the owner’s updated preference; backend lookup remains available. Adobe/dongle activation validation was removed from scope by the owner on 2026-10-09; it was not performed. Notification-driven detection and Windows/Linux memory-font adapters remain open. Cross-platform native validation is deferred to the next milestone; Font Book activation lifecycle remains open. |
| 02 | P0 | Clipboard, input and document reliability | In progress. P02.12 adds macOS Finder-tag/xattr/ACL and Unix ownership preservation before replacement; Linux copies visible xattrs/ACLs through safe descriptor APIs. Metadata errors retain the previous save. Four regressions and all seven gates pass (1,488 tests, 3 ignored; 20 WASM packages); native tagged/ACL Save retains exact metadata and saves the edit. Writer-only Windows/Linux strict Clippy passes. Windows extended metadata and native Windows/Linux execution remain open. [Scope and evidence](docs/pagenor/NATIVE_SAVE_METADATA.md). P02.11 stages/syncs native Save/Save As/Save a Copy before replacement (PG-0217), preserving last saved bytes on partial-write failure and current document/recovery state on refusal. Five regressions, all seven gates (1,484 tests, 3 ignored; 20 WASM packages) and native read-only failure/retry with cleared error pass. Extended file metadata (PG-0218) and native Windows/Linux execution remain open. [Save safety evidence](docs/pagenor/NATIVE_SAVE_SAFETY.md). P02.10 fixes PG-0216 with a Quit-only native macOS release fallback and same-pass text checkpointing before save/recovery. Three regressions (one failing before), all seven gates (1,479 tests, 3 ignored; 20 WASM packages), native first Cmd+Q during ABC-India dead-key input, Cancel/resumed typing and Save/Quit pass. Version remains 0.3.0; native Windows/Linux and broader input-method checks remain open. [Quit behavior and evidence](docs/pagenor/QUIT_SHORTCUTS.md). P02.9 fixes the tracked macOS input callback teardown locally: approved close disables canvas input, and safe AppKit deactivation retains windows through event-loop shutdown. Two new regressions (one failing before), all seven gates (1,476 tests, 3 ignored; 20 WASM packages), native save/cancel/resumed typing and saved-window Close with unfinished ABC-India input pass. Native Windows/Linux and broader input-method execution remain milestone checks. [Shutdown behavior and evidence](docs/pagenor/NATIVE_INPUT_SHUTDOWN.md). P02.8 removes the separate composition preview box and renders unfinished text in the paragraph at the selected font/size without changing the live document. Two new regressions and all seven gates pass (1,473 tests, 3 ignored; 20 WASM packages). The owner confirmed physical VariType typing stays in the line in the rebuilt Mac bundle. Native Windows/Linux/browser checks remain open; the separately observed native Quit callback failure (PG-0215) is addressed by P02.9. P02.7 enables shared native canvas composition/commits without language filtering, with ephemeral rendered preedit and existing command-backed editing/undo. Nine new regressions and all seven gates pass (1,471 tests, 3 ignored; 20 WASM packages). Native macOS ordinary typing and continuous Varitype Telugu entry were inspected after correcting a reproduced per-paint focus-reset failure; native Windows/Linux/browser execution remains pending. [Behavior and evidence](docs/pagenor/KEYBOARD_INPUT.md). P02.1 fixes duplicate external/plain paste undo/journal entries and restores pre-edit selection; Unicode/formatting, replay, host-bridge and CPU UI probes pass. P02.2 retains recovery snapshots across reopen and prevents reused runtime IDs from overwriting pending entries (PG-0203); multiple-entry/failure/retry/cleanup and fresh-process tests pass. P02.3 fixes stale checkpoints after clean undo (PG-0204) and oversized intervals disabling timed saves (PG-0205); seven UI tests, real-time headless scenarios and inspected failure/retry images pass. P02.4 adds final recovery on desktop quit/root close (PG-0206), cancels closing on failure and retries cleanup errors; seven UI tests, six isolated processes and inspected CPU images pass. P02.5 fixes menu/control Copy/Cut missing OS clipboard output (PG-0207); four UI tests and actual bidirectional TextEdit/native Pagenor transfer pass. Native fixture control quit, fresh native recovery and inspected native screenshot pass. P02.6 defaults new native saves/copies/packages/downloads to `.pgnr`, preserves legacy `.pagenor` reading and saved paths, and updates shared routing/platform metadata. Six new regressions and actual macOS suffixless Save/Open pass. Local gate: 854 passed, 3 ignored; 16 WASM packages, Mac bundle and Windows/Linux strict UI-library checks pass. Native installer/file-association testing remains in the milestone matrix. P00.2 ports upstream unsaved-close confirmation, focused macOS Edit routing and typed generic dialogs without replacing recovery ownership. Headless interaction regressions pass; direct native accelerator and browser execution remain unverified. Earlier direct-launch AppKit timeout is recorded; the desktop-launched clipboard fixture now processes frames. Other application transfers, native menu/keyboard delivery, live timer/failure/OS-exit coverage, actual storage failures and native platform matrices remain open. |
| 03 | P0 | Indic corpus and font presets | Planned. Requires stable font discovery, versioned fixtures and font licence/size review. |
| 04 | P0 | Script-aware fallback and composite fonts | In progress. P04.1 adapts #126/#129 composite/generated-label resolution, localized aliases, coverage and nested-object/preflight handling, plus repeated missing-font replacement. Metadata remains bounded, shared and lazy; live catalogue refresh, cached rendering and document precedence are retained. Local gate: 1,157 passed, 3 ignored; 17 WASM packages. Inspected CPU dialogs and pointer replacement/undo/Done pass; Windows/Linux strict UI-library Clippy checks pass. This is partial supporting work; Indic/script corpus gates remain pending. Adobe/dongle validation was removed from scope by the owner on 2026-10-09. |
| 05 | P0 | Indic editing and shaping | Planned. Cluster-safe editing and independent shaping comparisons required. |
| 06 | P1 | Indic composition and language support | Planned. |
| 07 | P1 | Professional typography and page tools | In progress. P07.5 adapts #347 explicit triple-click APIs and Select Line while retaining richer canvas multi-click and quick native input. Final #341–347 integration passes all seven gates: 1,536 tests, 3 ignored; 20 WASM packages. Word/line renders and native temporary-file numbering/Tabs/ruler/selection checks pass; final 0.3.0 bundle installed locally with separate icons. Documents/Codex native font-folder probing stalled; OS/filesystem cause remains unresolved and recorded. Windows/Linux native execution remains pending. 0.3.0, local only. [Details](docs/pagenor/MULTICLICK_SELECTION.md). P07.4 adapts #346 start-page bounds and iterative/saturating page numbering, with hostile-file label safeguards. Final #341–347 integration passes all seven gates: 1,536 tests, 3 ignored, including the additional label safeguard. 0.3.0, local only. P07.3 adapts #343 Tabs commands, panel and style ruler, writer validation and native/IDML repair. All seven gates pass: 1,524 tests, 3 ignored; 20 WASM packages; rendered UI inspected. Version remains 0.3.0, local only. [Details](docs/pagenor/TAB_STOPS.md). P07.2 adapts upstream #341 chapter numbering for documents/books and section-prefix validation. All seven gates pass: 1,507 tests, 3 ignored; 20 WASM packages. Open-document synchronization and safe book writes are covered. Version remains 0.3.0; IDML chapter mapping and native Windows/Linux execution remain separate. [Details](docs/pagenor/CHAPTER_NUMBERING.md). P07.1 adapts the dependent #83/#84/#81/#106/#96 typography/layout group: drop caps/selection, first baselines, wrap/defaults, threading/snapping, family style linking and text/frame formatting, kerning and empty-paragraph mark formatting. Version-2 migration pins prior paragraph defaults using indexed bounded style walks; Pagenor clipboard/recovery and document-scoped fonts remain. Full local gate: 1,118 passed, 3 ignored; 17 WASM packages. CPU panel/layout inspection and Windows/Linux strict UI-library Clippy checks are recorded with the group. Native platform/browser execution and Indic exit gates remain separate. |
| 08 | P1 | Import, export and print | In progress. P08.1 adapts #150/#170 EPS source/previews, Links relinking and editable PowerPoint backend/options/warnings. Bounded actual reads/export structure, atomic relink preparation, document fonts and native migration remain. New saves use format 3. All seven local gates pass: 1,207 tests, 3 ignored, 18 WASM packages; Windows/Linux strict UI-library Clippy checks pass. Inspected CPU dialogs/Links and pointer option changes produce editable slide XML. PostScript interpretation, native PowerPoint compatibility and native platform/browser execution remain milestone checks. P08.2 adds #98 INDD subset Open/Place with visible limitations and bounded conversion. All seven local gates pass: 1,220 tests, 3 ignored, 19 WASM packages; Windows/Linux strict UI-library Clippy checks pass. CPU warning/layout inspection uses a synthetic fixture; real-file compatibility remains unverified. AI supporting feature integration continues locally. |
| 09 | P1 | UI, accessibility and performance | In progress. P09.15 adapts #345 draggable ruler origin/reset, per-pane drag state and single-click preservation. Inspected renders and all seven gates pass: 1,528 tests, 3 ignored; 20 WASM packages. 0.3.0, local only. P09.14 adapts #344 swatch hover/width and Glyphs grid clearance. Two render-geometry regressions, inspected renders and all seven gates pass: 1,526 tests, 3 ignored; 20 WASM packages. 0.3.0, local only. P09.13 adapts #342 numbering window with atomic apply and one-step undo. All seven gates pass: 1,510 tests, 3 ignored; 20 WASM packages. Normal/small/RTL action visibility and rendered window inspected. Version remains 0.3.0; work stays local. [Details](docs/pagenor/NUMBERING_SECTION_OPTIONS.md). P09.12 adapts open upstream #325 with New Document preset tabs/cards, detailed settings and reusable engine presets. Popup Enter/Escape/layering, retained failed Create details, input bounds and persistence have regression coverage. All seven gates pass: 1,500 tests, 3 ignored; 20 WASM packages. Native macOS A4 creation/Save and Recent recall after app restart pass. Version remains 0.3.0 and changes stay local; native Windows/Linux execution remains a milestone check. [Behavior and validation](docs/pagenor/NEW_DOCUMENT_PRESETS.md). P09.11 adds shared font-size presets across the top bar, Character/Properties panels and floating text bar with custom entry preserved. Separate document artwork and corrected native small-icon encoding are packaged for macOS, Windows and Linux. All seven gates pass: 1,474 tests, 3 ignored; 20 WASM packages; eight packaging contracts pass. Native top/Character dropdowns, fresh/original document artwork and Login Items app artwork are verified together; the bundle is installed locally in Applications. Version remains 0.3.0 and changes stay local. [Behavior and validation](docs/pagenor/FONT_SIZE_DOCUMENT_ICONS.md). P09.10 moves About below the right-side artwork and adds matching recent-document preview tiles with a bounded background PNG cache. Four new regressions and all seven local gates pass: 1,462 tests, 3 ignored; 20 WASM packages. Native macOS previews and About interaction pass; Windows/Linux execution remains separate. Version remains 0.3.0; changes stay local. P09.9 adds Welcome Recent Documents and File → Open Recent, hides editor tools/panels on Home, and prompts to save each modified/never-saved document before quitting. Eleven new regressions and all seven local gates pass: 1,458 tests, 3 ignored; 20 WASM packages. Native macOS Welcome, restart persistence, restored editor controls and native recent-file menu inspection pass; Windows/Linux native execution remains separate. Version stays 0.3.0; changes stay local. [Behavior and validation](docs/pagenor/WELCOME_RECENT_DOCUMENTS.md). P09.8 fixes quick canvas clicks/drags. Four UI regressions and all seven local gates pass: 1,447 tests, 3 ignored; 20 WASM packages. Actual macOS drawing, paste, manual threading across pages and `.pgnr` saving pass in the rebuilt development app; native Windows/Linux remain unverified. [Cause and steps](docs/pagenor/MANUAL_TEXT_FLOW.md). P09.7 adapts 23 new proposals across UI, compatibility and multipage composition. All seven local gates pass: 1,443 tests, 3 ignored; 20 WASM packages. CPU UI and interaction/undo/import regressions pass; version remains 0.3.0 and changes stay local. Native Windows/Linux execution remains separate. [Scope and pinned provenance](docs/pagenor/UPSTREAM_EDITOR_PUBLISHING.md). P09.6 completes ten selected UI proposals sequentially. All seven local gates pass: 1,308 tests, 3 ignored; 20 WASM packages. Inspected CPU UI/pointer and interaction/undo regressions pass; native Windows/Linux/browser execution remains separate. Italian #218 is deferred. [Behavior and evidence](docs/pagenor/EDITOR_UI_UPDATES.md). P09.2 adapts nine upstream editor UI proposals with command-backed controls, richer style editors/deletion and stable asynchronous import targets. Full local gate: 972 passed, 3 ignored; 17 WASM packages. Rebuilt macOS UI inspection covers branding, two control rows and character-style sections. P09.3 adds advanced data-merge backends and panel controls from #109/#125, with bounded input/formula/grid/transfer handling. Full local gate: 1,022 passed, 3 ignored; 17 WASM packages. CPU source/grid/scrolled-control inspection and UI command/undo interactions pass. Windows/Linux strict UI-library Clippy checks pass; native execution remains separate. P09.4 adds #128/#118 RTL table defaults/controls, binding story defaults, PDF/A images and typing/story/page contracts. Full local gate: 1,167 passed, 3 ignored; 17 WASM packages. Inspected CPU pointer direction and undo pass, as do Windows/Linux strict UI-library Clippy checks. P09.5 adapts closed #54 with AI transport/credential services, persisted review records and docked/floating review/settings UI. Full local gate: 1,249 passed, 3 ignored; 20 WASM packages. CPU pointer review/undo and native save/reopen/report checks pass. Offline review, bounds and cancellation are tested; live account sign-in/inference and native Windows/Linux credential integration remain unverified. New TLS dependencies require Windows/Linux C cross-compilers absent on this Mac. Native drag delivery and browser execution remain separate validation gaps. Partial branding evidence (P09.1): shared desktop launch splash uses the supplied artwork, independent of whether documents are open/recovered. Two timing/document/recovery tests, inspected CPU renders and an actual macOS splash-to-welcome transition pass; full local gate is 848 passed, 3 ignored, with 16 WASM packages and Windows/Linux strict UI-library checks passing. The phase's broader UI/accessibility/performance gates and native Windows/Linux validation remain planned. |
| 10 | P1 | Cross-platform distribution | P10.3 adapts upstream #131 with owner-branded HiDPI DMG artwork and reproducible Finder layout; local version advances to 0.4.0 with owner authorization. All seven gates pass (1,536 tests, 3 ignored; 20 WASM packages), nine packaging contracts and ShellCheck pass. The layout allows newer Finder tab/path/status bars without changing user preferences. No new GitHub publication. P09.11 packages the supplied document icon independently of app artwork for macOS, Windows and Linux and corrects small ICNS decoding. Apple-native regression and eight packaging contracts pass; native Windows/Linux installation/display remains a milestone check. P10.2 adapts FreeBSD tar packaging, x86_64/aarch64 Flatpak bundles, AppImage zsync and owner download documentation from #132/#135. Version stays 0.3.0. All seven local gates pass (1,308 tests, 3 ignored; 20 WASM packages), plus eight synthetic packaging contracts, ShellCheck and actionlint. Actual new-package creation/install/execution and signing remain milestone-host checks; no publication. [Procedure](docs/releasing.md). Partial local development evidence (P10.1): current macOS executable is bundled with the owner icon through `packaging/macos/dev.sh`; Finder display, native launch, independent macOS icon decoding, signature and CLI version pass. Rebuilding a running bundle is refused. Native macOS, Windows and Linux CI starts in Phase 00; Windows ARM64 unsigned MSI build/install/architecture/CLI-version/uninstall smoke passed in run 37776034034. Release packaging breadth, GUI/file-open, installation and signing gates remain open. |
| 11 | P1 | Public beta and 1.0 | Planned. |
| 12 | P2 | Post-1.0 improvements | Planned. |

Milestones: M0 = 00–02; Indic Alpha = 03–05; Publishing Alpha = 06–08;
Beta = 09–10; 1.0 = 11. No milestone is complete yet.

Next: continue the local Phase 02 native menu/keyboard delivery, other application
clipboard transfers, and live timer/failure/OS-exit and storage matrix, followed by
Phase 00 release checks. Adobe/dongle activation validation is no longer a
pending task or milestone requirement, per the owner’s 2026-10-09 instruction.
Font Book deactivation evidence remains open. Run
the native platform matrix when publishing the next completed milestone. The
Indic corpus remains planned until its font/reliability prerequisites are met.
Open findings and their completion evidence belong in [the issue register](docs/pagenor/ISSUES.md).

All platforms retain the shared Rust core and `unsafe_code = "forbid"`.
New platform discovery belongs behind native adapters; document font scopes,
family/style grouping and lazy font data must remain intact. WASM remains a build gate.

P11.13 progress (12 October 2026): first eight queue items are implemented locally,
including the six related UI adaptations. The UI group passes all seven gates
(2,322 tests, no failures, seven ignored; twenty WASM packages) and native visual
inspection in an isolated synthetic QA profile. The rest of the queue remains in
progress; published 0.6.0 packages and the owner's running session remain intact.

P11.13 graphics group: #519, #528, #550 and #561 are complete. All seven gates
pass with 2,337 tests, no failures and seven ignored; twenty WASM packages. Native
inspection verifies content transforms, tight selection bounds and normal/inverted
saved masks. Native format 4 retains clipping paths and reads prior formats. Twelve
queue items are now complete; color/font and remaining publishing work continues.

P11.13 color/font group (#551, #520, #521, #548, #560): all seven gates passed,
2,345 tests/zero failures/seven ignored, twenty WASM packages. Seventeen queue items
are complete. Table-cell text color, validated swatches, returned object IDs, scoped/lazy
PostScript SVG lookup and exact IDML family/inherited-style handling are recorded in
UPSTREAM_ADOPTION_070.md; the remaining publishing/reliability work continues.

P11.13 import/composition group (#529, #523, #559, #562, #563): all seven gates
pass, 2,359 tests/zero failures/seven ignored; twenty WASM packages. Twenty-two
queue items are complete. Pagenor retains rich contour/side/graphic wrapping and
its anchored text/group support; synthetic wrap/cycle/cache/damage regressions pass.
Owner's running document session remains untouched.

P11.13 paragraph/decoration group (#522, #538, #539): all seven gates passed,
2,361 tests, zero failures, seven ignored; twenty WASM packages. Twenty-five queue
items are complete. Scripts/notes/cells share line-ending normalization; capitalized
settings control uppercase hyphenation; native underline pixels are continuous.
A section-marker edit regression additionally covers neighbor anchored-wrap caches.

P11.13 incremental UI/reflow group (#459, #478, #468, #473): all seven gates pass,
2,383 tests/zero failures/seven ignored; twenty WASM packages. Twenty-nine queue
items are complete. Native split-pane/scrollbar and layout-target screenshots were
inspected. Left/right masters remain independent by default; existing IME, clipboard,
validation and shutdown paths remain. Only synthetic QA documents were used.
