# Pagenor roadmap

- P11.13: implement the selected 26 newer upstream proposals and six incremental
  updates. All 32 are complete locally as **0.7.0**; all seven gates pass
  (2,574 tests, zero failures, seven ignores; twenty WASM packages).
  Nested undo, Pathfinder, tint/Frame/Text menus, View grouping/document grid,
  Justification, Properties transforms, Tools Color Picker, curved corners, image
  cropping/content transforms, saved clipping paths, color targeting/swatch validation,
  command result IDs, scoped PostScript/IDML font lookup, IDML rotation, indexed
  parent-page text/section markers, anchored inheritance/wrap, CR normalization,
  uppercase hyphenation, continuous decorations, consumed-condition reflow, split
  input/dock release, explicit layout targets, IDML spacing, saved EOF typing,
  native print scaling and Pagenor user guides are validated. Native Mac UI inspected;
  Windows/Linux packages and milestone publication are in progress.
  [queue and per-change notes](docs/pagenor/UPSTREAM_ADOPTION_070.md).

- P11.12: published native Windows and Linux **0.6.0 desktop development
  preview**, with matching universal Mac packages. All three Windows installer/
  association/icon/export/uninstall checks and both Linux/Flatpak CLI checks pass.
  Native Mac/Windows/Linux quality and Rust 1.95 gates pass. Signing and interactive
  desktop/printer validation remain open; [distribution notes](docs/pagenor/DESKTOP_RELEASE_060.md).
  The separate FreeBSD VM now installs DejaVu and checks Fontconfig before
  compilation; its changed VM still needs a native run.

- P11.11: complete the twenty UI-first October proposals through #491 and compatible
  reliability/publishing adaptations. **0.6.0**, local milestone. Preserve Pagenor
  fonts, recovery, inline IME, master workflow and branding; broader import/titlebar
  proposals remain open; native desktop distribution is recorded in P11.12.
  [Implementation notes](docs/pagenor/UPSTREAM_ADOPTION_20261012.md).

- P11.10: Add an experimental optional-helper PageMaker 6.5/7 text-recovery
  import through File → Open and document drops. Ramsha retains five pages
  and five editable frames; a PMD sample retains nine pages and 31 frames.
  Graphics, threading, layers and exact legacy typography remain incomplete.
  Local **0.5.2** preview. [Scope and validation](docs/pagenor/PAGEMAKER_IMPORT.md).

- P11.9: Add stored image colour mode to Links and selected-link ICC profile
  details, with header-only metadata queries and bounded profile inspection.
  Imported preview metadata is identified as such; vector/mixed or unavailable
  information is not guessed. Local **0.5.2** addition.
  [Notes](docs/pagenor/LINKS_COLOR_INFO.md).

- P11.8: Make the bottom-bar Preflight indicator/count clickable and open the
  live issue panel from Window → Output → Preflight. Existing issue clicks
  select the frame and page; shared UI regressions cover both access routes.
  Local **0.5.2** correction. [Notes](docs/pagenor/PREFLIGHT_STATUS.md).

- P11.7: Correct INDD raster-image detection and layer order/empty-layer
  retention; preserve multi-column count/gutter. Portfolio validation retains
  22 pages, 57 image placements, six user layers and exact used story text;
  typography/master/effect fidelity remains incomplete. All seven gates pass:
  1,902 tests, six existing ignores and 20 WASM packages. Windows/Linux crate
  cross-checks pass. Local **0.5.2** correction.
  [Evidence and limits](docs/pagenor/INDD_PORTFOLIO.md).

- P11.6: Recover every INDD story/style leaf and mixed-encoding text segment;
  correct canonical font-style precedence, paragraph spacing, left-justified
  last lines and sequential page labels. Owner-file text/native/PDF checks pass
  without missing glyphs; exact source pagination remains incomplete (31 versus
  33 populated frames). All seven gates pass: 1,898 tests, six existing ignores
  and 20 WASM packages. Local **0.5.2** correction; current session preserved.
  [Evidence and limits](docs/pagenor/INDD_RECOVERY.md).

- P11.5: Improve IDML/INDD Indic import preservation: language-name mapping,
  whole-cluster fallback, bounded strict UTF-16 decoding across chunks and correct
  source style-run offsets. Native save/reopen and PDF regressions are included;
  real-file compatibility remains a separate corpus gate. All seven gates pass:
  1,889 tests, six existing ignores and 20 WASM packages; real installed Telugu
  font renders and exact PDF extraction pass. Local 0.5.2 correction.
  [Scope and validation](docs/pagenor/INDIC_IMPORTS.md).

- P11.4: Add independent one/two-column right panel toolbar toggling, a pinned
  switch, saved preferences/workspaces and shared dock support. **0.5.2**, local
  development. All seven local gates pass (1,878 tests; 20 WASM packages);
  inspected UI renders and a separate macOS build preserve the running session.
  [Notes](docs/pagenor/RIGHT_TOOLBAR_COLUMNS.md).
- P11.3: Master Pages controls, live left/right previews, repeating header/footer
  and numbering frames, inherited guides/snapping, shared margins/columns and
  persistent default masters. **0.5.2**, local development.
  [Usage and evidence](docs/pagenor/MASTER_PAGES.md).

- P11.2: Restore Welcome after document-window Close and widen the right panel
  icon bar. Retain unsaved-close questions, recovery and explicit Quit. Local
  **0.5.1** maintenance correction; validation recorded in
  [Welcome notes](docs/pagenor/WELCOME_RECENT_DOCUMENTS.md).

- P11.1: Improve canvas scrollbars, Arrow table/cell selection and edge formatting,
  functional panel groups, closable vertical panel stacks and status-bar controls.
  All seven checks pass: 1,863 tests passed, 6 ignored; 20 WASM packages and
  12 packaging contracts. **0.5.1**, local development. [Behaviour and evidence](docs/pagenor/EDITOR_USABILITY.md).

- P11: Complete all 47 selected non-CJK upstream proposals and #295/#298 prerequisites.
  Shared editor panels, paragraph/table formatting, effects/wraps, asset handling,
  PDF/IDML, graphics startup and printing are adapted under existing Pagenor safeguards.
  All seven `cargo xtask ci` gates pass: 1,856 tests passed, 6 ignored, no failures; 20 WASM packages. The 12 packaging contracts, ShellCheck and actionlint pass. The local macOS app is built and inspected separately. Windows/Linux native GUI and printing execution, signed/notarized distribution and GitHub publication remain milestone work.
  **0.5.0**, local only. [Per-change notes](docs/pagenor/UPSTREAM_EDITOR_ADOPTION.md).

- P00.9: Inspect all 71 upstream v0.6.0 release changes; 45 already covered,
  7 adapted here, 16 pending, 3 deferred/out of scope. Scoped SVG fonts and
  vector PDF agree; PDF export/CLI reliability and RISC-V packaging advance.
  All seven gates pass: 1,548 tests, 3 ignored; 20 WASM packages; 12 packaging
  contracts. **0.4.0**, local only; native Windows/Linux/RISC-V remain milestone
  checks. [Decisions and remaining work](docs/pagenor/UPSTREAM_060_AUDIT.md).

- P10.3: Adapt upstream #131 installer layout with owner Pagenor artwork,
  reproducible HiDPI background and separate app/document icons. Version hold
  lifted by the owner; local feature checkpoint advances to **0.4.0**.
  All seven gates pass: 1,536 tests, 3 ignored; 20 WASM packages. Nine
  packaging contracts and byte-for-byte installer regeneration pass. Native Mac
  installer/version evidence is recorded in the release adaptation report.
  No GitHub publication. [Installer](packaging/macos/dmg/README.md).

- P07.5: Adapt #347 explicit triple-click APIs and Select Line, retaining richer
  canvas multi-click behavior and quick input. Final integration of #341–347
  passes all seven gates: 1,536 tests, 3 ignored; 20 WASM packages. Word/line
  renders inspected. **0.3.0**, local only. [Details](docs/pagenor/MULTICLICK_SELECTION.md).

- P07.4: Adapt #346 checked start-page bounds and iterative/saturating page
  numbering. Retain preflighted book pagination; protect loaded page labels.
  Final #341–347 integration passes: 1,536 tests, 3 ignored; all seven gates including labels.
  **0.3.0**, local only.

- P09.15: Adapt #345 draggable ruler zero point and double-click reset, per-pane
  drag state and plain-click preservation. Inspected renders and all seven gates
  pass: 1,528 tests, 3 ignored; 20 WASM packages. **0.3.0**, local only.

- P09.14: Adapt #344 shared swatch-list hover/width and Glyphs scroll-bar clearance.
  Two UI regressions, inspected renders and all seven gates pass: 1,526 tests,
  3 ignored; 20 WASM packages. **0.3.0**, local only.

- P07.3: Adapt upstream #343 with Tabs commands, floating ruler and Paragraph
  Style Options controls, shared writer validation and file-load repair. All
  seven gates pass: 1,524 tests, 3 ignored; 20 WASM packages; rendered UI
  inspected. Version **0.3.0**, local only. [Details](docs/pagenor/TAB_STOPS.md).

- P09.13: Adapt upstream #342 Numbering & Section Options with side actions and
  atomic one-step undo. All seven gates pass: 1,510 tests, 3 ignored; 20 WASM
  packages. Rendered UI inspected; version remains **0.3.0**, work stays local.
  [Details](docs/pagenor/NUMBERING_SECTION_OPTIONS.md).

- P07.2: Adapt upstream #341 with document/book chapter style and source, formatted
  variables/cross-references, section-prefix rules and safe open-document updates.
  All seven gates pass: 1,507 tests, 3 ignored; 20 WASM packages. Version remains
  **0.3.0**; changes stay local. IDML chapter mapping and native Windows/Linux
  execution remain separate. [Details](docs/pagenor/CHAPTER_NUMBERING.md).

- P09.12: Adapt open upstream #325 with Recent/Saved/Print/Web/Mobile New Document
  tabs, preset cards, detailed page settings, linked margins/bleed/slug and saved
  engine presets. Preserve existing Welcome recents and dialog behavior. Bound
  preset settings and retain failed Create details; order Save Preset above its
  modal parent. All seven gates pass: 1,500 tests, 3 ignored; 20 WASM packages.
  Native A4 creation/Save and Recent recall after restart pass. Version remains
  **0.3.0**; changes stay local. Native Windows/Linux execution remains pending.
  [Behavior and validation](docs/pagenor/NEW_DOCUMENT_PRESETS.md).

- P02.12: Preserve macOS Finder tags, binary extended attributes, Unix ownership/
  mode and extended ACLs before replacing an existing saved document. Linux
  copies visible xattrs (including POSIX ACLs); Windows keeps the portable path
  with extended metadata pending. Bounded metadata copying fails before commit,
  preserving previous bytes. Mac regressions cover tags, ACLs, differing groups
  and metadata failure/retry. All seven gates pass (1,488 tests, 3 ignored;
  20 WASM packages). Native Cmd+S retains exact metadata while saving the edit;
  writer-only Windows/Linux strict Clippy passes. Evidence is recorded in
  [save metadata](docs/pagenor/NATIVE_SAVE_METADATA.md). Version stays **0.3.0**,
  and all changes remain local. Native Windows/Linux validation remains open.

- P02.11: Stage, sync and replace native Save/Save As/Save a Copy outputs instead
  of truncating the previous file. Five regressions verify partial failure/retry,
  unpublished new saves, file links/modes, read-only command/recovery integrity
  and Copy state. Native read-only Save preserves exact bytes and dirty state;
  retry saves the latest edit and clears the stale error. All seven gates pass:
  1,484 tests, 3 ignored; 20 WASM packages. Version remains **0.3.0**; changes stay
  local. Extended metadata and native Windows/Linux validation remain open.
  [Behavior and validation](docs/pagenor/NATIVE_SAVE_SAFETY.md).

- P02.10: Recognize Quit during canvas composition, including the macOS native
  key-release fallback when AppKit consumes key-down. Final delivered text is
  committed once before save/recovery, and duplicate Quit preserves save choices.
  Three regressions (one failing before), all seven gates (1,479 tests, 3 ignored;
  20 WASM packages), native first Cmd+Q, Cancel/resumed typing and Save/Quit pass.
  Native Windows/Linux and broader input-method coverage remain milestone checks.
  Installed local app remains **0.3.0**, retaining its separate icons.
  [Behavior and validation](docs/pagenor/QUIT_SHORTCUTS.md).

- P02.9: Stop canvas input after approved desktop closing and deactivate macOS
  native input while retaining its window through event-loop teardown (PG-0215).
  A quit regression fails before the correction; both new shutdown regressions
  and all seven gates pass: 1,476 tests, 3 ignored; 20 WASM packages.
  Native save prompt, cancellation/resumed typing and saved-window Close with
  unfinished ABC-India dead-key input pass. Native Windows/Linux execution and
  the separate Cmd+Q composition observation remain further coverage.
  The installed local app stays **0.3.0**, with its existing separate icons.
  [Behavior and validation](docs/pagenor/NATIVE_INPUT_SHUTDOWN.md).

- P09.11: Add one shared font-size preset list to the top bar, right-side
  Character panel, Properties typography and floating text bar, retaining custom
  sizes. Use the supplied document SVG for separate macOS/Windows/Linux file
  icons. Fix corrupted small macOS app icons in the portable ICNS packer, with
  a failing-before/passing-after native decoder regression. All seven local gates
  pass: 1,474 tests, 3 ignored; 20 WASM packages. Eight packaging contracts pass.
  Version stays **0.3.0**; changes stay local. Native Windows/Linux icon display
  remains a milestone check.
  [Behavior and validation](docs/pagenor/FONT_SIZE_DOCUMENT_ICONS.md).

- P02.8: Remove the separate composition preview box. Unfinished OS text now
  sits in the actual paragraph flow at its selected font/size, with contextual
  shaping, wrapping and a matching visual caret. A cached copy-on-write document
  uses the existing insertion command without changing live save/undo/recovery.
  Two new regressions and all seven gates pass: 1,473 tests, 3 ignored;
  20 WASM packages. The owner confirmed physical VariType text stays inline
  in the rebuilt Mac bundle. The launcher no longer mistakes icon readers for
  running executables. Native Windows/Linux/browser execution remains pending.
  Version stays **0.3.0**; changes stay local.
  [Input behavior and evidence](docs/pagenor/KEYBOARD_INPUT.md).

- P02.7: Enable the canvas's OS text-input service and handle composition/commits
  through existing editing commands, preserving styles, font scopes, undo and
  recovery. Preedit remains ephemeral and uses the existing document renderer.
  Keyboard layouts are not language-filtered; composition and Option/AltGr text
  retain input ownership. Nine new regressions and all seven local gates pass:
  1,471 tests, 3 ignored; 20 WASM packages. The macOS bundle was built/launched,
  and continuous Varitype Telugu typing was observed after correcting a
  reproduced per-paint focus-reset failure. Native Windows/Linux/browser checks
  remain separate.
  Version stays **0.3.0**; changes stay local.
  [Input behavior and evidence](docs/pagenor/KEYBOARD_INPUT.md).

- P10.4: GitHub packaging contracts reproduced a runner-only assertion failure:
  Actions warnings are on stdout, local warnings on stderr. The same AppImage
  contract now checks both routes and stale-update removal in both environments.
  No application source, artifact binary or version change. Version stays 0.3.0;
  All seven local gates and eight packaging contracts pass, including a local
  reproduction with GITHUB_ACTIONS=true. The development release remains draft
  while its GitHub packaging check is rerun.

- P10.3: Prepare the owner-authorized GitHub v0.3.0 development prerelease, with
  the verified Apple-silicon Mac bundle, matching source, release notes and SHA-256
  checksums. Version stays 0.3.0. Release notes explicitly retain pending native
  Windows/Linux/FreeBSD package/signing checks; no milestone is declared complete.
  Restore the pre-existing actionlint archive checksum and checkout pin in
  packaging lint before publication. Local seven-step gate and eight packaging
  contracts pass; GitHub check results are reported separately.

- P10.2: Adapted the two remaining upstream v0.4.0 release-note items, #132/#135:
  FreeBSD tar packaging, Flatpak bundles for x86_64/aarch64, AppImage zsync and
  truthful owner downloads/build documentation. All 34 listed items are accounted
  for. Pagenor remains **0.3.0** per the owner’s instruction. Bounded archive
  extraction, host-compatible X11 permissions, owner/bundled-font notices and
  update routing retain existing architecture and identifiers. All seven local
  gates pass: 1,308 tests, 3 ignored; 20 WASM packages. Eight additional packaging
  contracts, ShellCheck and actionlint pass. Native installer/signing/execution
  checks remain in the milestone matrix; no remote publication.
  [Release audit](docs/pagenor/UPSTREAM_040_AUDIT.md) and
  [packaging procedure](docs/releasing.md).

- P00.8: Audited all 34 upstream v0.4.0 release-note changes. The 32 inherited
  application/workflow contributions remain; #132/#135 distribution work follows
  in P10.2. Accumulated local application checkpoint is 0.3.0.
  All seven gates pass: 1,308 tests, 3 ignored; 20 WASM packages.
  [Per-item audit](docs/pagenor/UPSTREAM_040_AUDIT.md) and
  [version policy](docs/pagenor/VERSIONING.md).

- P09.10: Welcome moves About below the large right-side image and removes its
  surrounding card. Recent documents use matching 120 × 150 tiles with first-page
  previews, a bounded local PNG cache and a single background rendering worker.
  Existing importer/font scopes and page rendering remain. Four new regressions
  and all seven local gates pass: 1,462 tests, 3 ignored; 20 WASM packages. Native
  macOS thumbnails and About interaction were inspected. Native Windows/Linux
  execution remains separate. Version stays **0.3.0**; changes stay local.
  [Behavior and limits](docs/pagenor/WELCOME_RECENT_DOCUMENTS.md).

- P09.9: Welcome adds bounded, persisted Recent Documents beneath presets and
  File → Open Recent, including the native macOS menu. Home retains open
  documents and hides editor tools/panels until a document opens or is created.
  Quit asks Save/Don't Save/Cancel for every modified or never-saved document;
  cancelled/failed saves stop quitting and existing recovery guards remain.
  Eleven new regressions and all seven local gates pass: 1,458 tests, 3 ignored;
  20 WASM packages. Native macOS Welcome, restart persistence, restored editor
  controls and the native recent-file menu were inspected. Windows/Linux native
  execution remains separate. Version stays **0.3.0**; changes stay local.
  [Behavior](docs/pagenor/WELCOME_RECENT_DOCUMENTS.md).
- P09.8: Fixed canvas clicks/drags delivered between paints, restoring Type-tool
  frame drawing, caret placement and manual text threading. Four UI regressions
  and all seven local gates pass: 1,447 tests, 3 ignored; 20 WASM packages.
  Native macOS drawing, clipboard paste, out-port loading, page navigation,
  continuation-frame creation and `.pgnr` saving pass in the rebuilt development
  app. Version stays 0.3.0; native Windows/Linux checks remain separate.
  [Cause, verification and manual steps](docs/pagenor/MANUAL_TEXT_FLOW.md).

- P09.7: Adapted 23 new upstream proposals: eight UI improvements, six compatibility
  fixes and nine multipage/composition changes. Quick Apply, compact controls, A6,
  scrolling dialogs, Paragraph Rules and safe Windows caption actions accompany
  scoped Smart Text Reflow, spanning bands, tabs/page breaks, column rules and table
  style/fitting corrections. Font scopes/lazy discovery, `.pgnr`, recovery and version
  **0.3.0** remain. All seven local gates pass: 1,443 tests, 3 ignored; 20 WASM
  packages. Inspected CPU UI and pointer/undo/import regressions pass; native
  Windows/Linux execution remains a milestone check. The local macOS development
  app build, plist and ad-hoc signature verification pass. No publication.
  [Scope, provenance and evidence](docs/pagenor/UPSTREAM_EDITOR_PUBLISHING.md).

- P09.6: Completed ten selected UI proposals sequentially: PDF export options,
  clickable swatches/colour pickers, canvas context actions, paragraph rules,
  stable control-bar transitions, Interactive PDF workspace, shortcut handling
  and undoable table-row distribution. Italian translation #218 is deferred.
  All seven local gates pass: 1,308 tests, 3 ignored; 20 WASM packages.
  Inspected CPU UI and pointer checks pass.
  [Behavior and local evidence](docs/pagenor/EDITOR_UI_UPDATES.md).

- Scope update (2026-10-09): The owner cancelled Adobe/dongle activation
  validation. It is no longer pending or a milestone requirement, and is not
  claimed as tested. Existing native font support is retained; other font
  lifecycle, platform and Indic validation tasks remain open.

- P00.7: Adapted the performance idea from the now-withdrawn upstream #172,
  resolving threaded frame locations in one iterative document walk per story.
  Nested/parent locations, embedded frames, font scope and cache invalidation
  remain. Four regressions and equivalent-output warm-cache benchmarks pass;
  1,000-page/50-frame stories improve from 333.221 ms to 28.524 ms (11.68×).
  The previous deep-group locator stack failure is reproduced in isolation;
  the new small-stack regression passes. All seven local gates pass: 1,286 tests,
  3 ignored; 20 WASM packages. Windows/Linux production document/composition
  library strict Clippy and the macOS executable build pass. Native platform
  execution remains a milestone check. [Scope and evidence](docs/pagenor/COMPOSITION_LOOKUP.md).

- P00.6: Adapted upstream #122 for desktop stderr and rotating diagnostic logs,
  retaining settings paths, control authentication/off-by-default and existing
  panic/recovery hooks. Record, pending-byte and file caps handle storage failures
  safely; tokens bypass file logging. Fresh-process closed-stderr, rotation and
  token-exclusion tests pass. All seven local gates pass: 1,282 tests, 3 ignored;
  20 WASM packages. Production logger-only Windows/Linux strict Clippy passes;
  native platform execution remains a milestone check. [Scope and evidence](docs/pagenor/DESKTOP_LOGGING.md).

- P00.5: Adapted upstream #151 for authenticated desktop control, loopback-only
  clients and bounded framing, connections, requests, UI queue and pointer work.
  Automation remains off by default. Lost replies never replay document edits;
  explicit later calls can reconnect. Pagenor recovery, scoped fonts, branding,
  .pgnr and shared Rust layers remain. All seven local gates pass: 1,262 tests,
  3 ignored; 20 WASM packages. Native Windows/Linux execution remains a milestone
  check. [Behavior and evidence](docs/pagenor/CONTROL_HARDENING.md).

- P09.5: Adapted closed #54 with native AI services, persisted review records,
  docked/floating Chat/Suggestions/Settings and proofreading, acceptance, undo
  and report controls. Pagenor format 3 and scoped document fonts stay. Input,
  record, request and worker bounds preserve documents on errors; secrets use
  platform credential storage. All seven local gates pass: 1,249 tests, 3 ignored,
  20 WASM packages. CPU pointer review/undo and save/reopen/report checks pass.
  Live account and native Windows/Linux execution
  remain unverified. Cross-builds for this group require C toolchains absent on
  this Mac. [Behavior and limits](docs/ai-assistant.md).

- P08.2: Adapted #98 as a standalone INDD subset reader feeding the existing
  IDML importer. Open/Place filters and visible warnings preserve stable import
  targets, document fonts and .pgnr. Bounded input/model/XML handling retains
  failed-import state. All seven local gates pass: 1,220 tests, 3 ignored,
  19 WASM packages. Windows/Linux strict UI-library Clippy checks pass.
  CPU warning/layout inspection uses a synthetic Rust fixture; real-file
  version breadth/fidelity and native platform execution remain unverified.
  [Supported subset](docs/indd.md).

- P08.1: Adapted #150/#170 EPS/Links and editable PowerPoint export. The Links
  picker lists graphics; replacement batches prepare before mutation. EPS retains
  source and uses previews. Export options/warnings preserve document font scopes
  and native format migration; new saves use format 3. Local gate: 1,207 passed,
  3 ignored; 18 WASM packages, all seven steps. Windows/Linux strict UI-library
  Clippy checks pass. Inspected CPU Links/dialogs and pointer Reflow/embedding
  choices produce editable slide XML. PostScript interpretation, native PowerPoint
  and native platform/browser execution remain milestone checks.
  [Scope](docs/pagenor/EXPORT_INTERCHANGE.md).

- P09.4: Adapted upstream #128/#118 for RTL table defaults/panel controls,
  document-binding story defaults, PDF/A raster interpolation and typing/story/
  page contracts. Pagenor recovery-aware quit and damaged-raster validation stay.
  Local gate: 1,167 passed, 3 ignored; 17 WASM packages. Windows/Linux strict
  UI-library Clippy checks pass. Inspected CPU pointer direction and undo pass;
  native platform/browser execution remain milestone checks.
  [Behavior](docs/pagenor/TABLE_CONTROL.md).

- P04.1: Adapted upstream #126/#129 for scoped composite resolution, generated
  labels, localized aliases, glyph coverage, nested objects/preflight and repeated
  missing-font replacement. Bounded shared metadata preserves lazy menus, live
  refresh, cached rendering and newly installed exact styles. Open dialog lists
  refresh with catalogue changes/rescans. Local gate: 1,157 passed, 3 ignored;
  17 WASM packages. CPU pointer replacement/undo/Done and inspected dialogs pass;
  Windows/Linux strict UI-library Clippy checks pass. Indic exit gates remain pending. [Scope](docs/pagenor/FONT_REPLACEMENT.md).

- P07.1: Dependent upstream typography/layout proposals #83/#84/#81/#106/#96
  add drop caps/selection, baseline/wrap/defaults, threading, snapping, family
  style linking, text/frame formatting, kerning and empty-paragraph fixes.
  Version-2 migration preserves version-1 layout with bounded indexed style
  walks. Pagenor clipboard/recovery and scoped lazy fonts are retained.
  Local gate: 1,118 passed, 3 ignored; 17 WASM packages. CPU panel/layout
  inspection and Windows/Linux strict UI-library Clippy checks are included
  in the completed local integration group; native/browser execution remains
  separate.

- P09.3: Advanced data-merge backends and panel from pinned upstream #109/#125.
  Multiple sources, JSON/Excel values, filters/sorts, grids and warning handling
  preserve template state, undo, scoped fonts and .pgnr. Joins/affixes/combined
  sorting are command interfaces. Local parsing, formula, transfer and planning
  bounds retain the import policies. Source/grid/lower-control CPU inspection
  and UI interaction regressions pass. Full local gate: 1,022 passed, 3 ignored;
  17 WASM packages. Windows/Linux strict UI-library Clippy checks pass. Native
  execution on those platforms and browser interaction remain milestone checks.
  [Behavior and limits](docs/pagenor/DATA_MERGE.md).

- P09.2: Adapted nine pinned upstream editor UI proposals after reviewing both
  open and closed requests. Responsive bars, two control rows, movable/pinned
  task bar, ruler units, language, richer style editors/deletion and stable
  asynchronous import targets preserve Pagenor branding, .pgnr, lazy scoped
  fonts, clipboard and recovery. Local replacement/rename validation prevents
  broken style references and inheritance cycles. New feature backends and
  their UI remain in scope for subsequent local integration groups. Full local
  gate: 972 passed, 3 ignored; 17 WASM packages. The rebuilt Mac app was
  inspected for branding, both control rows and character-style sections.
  Windows/Linux strict UI-library Clippy checks pass. Native drag delivery and
  browser execution remain separate validation gaps.

- P00.4: Adapted the package portion of upstream #155: portable Unicode names,
  redirected-output checks and staged individual file replacement. Pagenor
  additions prepare exports/preflight paths before writes, check existing parent
  redirects and preserve short extensions when shortening names. .pgnr, scoped
  fonts, fallback/licence handling and live document state are retained. Full
  local gate: 916 passed, 3 ignored; 17 WASM packages. Windows/Linux strict
  UI-library Clippy checks pass. Folder-wide rollback and hostile concurrent
  directory renaming remain outside this task; [scope](docs/pagenor/PACKAGE_OUTPUT.md).
  No milestone exit or GitHub push.

- P00.3: Adapted upstream #154 for bounded ZIP/ZIP64, native and Office
  import/output processing, with matching native save limits and XML 0.41
  compatibility. Added padded-table/root/duplicate/read guards and propagated
  native serialization errors before writes; failed save/recovery checks retain
  existing files, document state and checkpoints. Full local gate: 907 passed,
  3 ignored; 17 WASM packages and Windows/Linux strict UI-library checks pass.
  Import limits and remaining streaming,
  package-path and IDML confinement work are recorded in
  [the policy](docs/pagenor/IMPORT_LIMITS.md). Cross-platform native execution
  remains pending; no milestone exit/push.

- P00.2: Triaged 53 open upstream PRs and adapted reliability changes from 12,
  with pinned revisions/attribution and a decision register. Fixes cover no-break
  composition, UTF-16 imports, count bounds, damaged-image PDF export, unsaved
  close, typed dialogs, macOS Edit routing, Windows backend defaults, export
  escaping, web startup/Print and CLI option order. Shared font search omits dot
  families per the owner’s preference while retaining backend access. Local gate:
  883 passed, 3 ignored; 16 WASM packages, Mac bundle and Windows/Linux strict
  UI-library checks pass. Larger conflicting/dependent features remain candidates
  for separate tasks; native platform and browser execution gaps remain open.

- P02.6: New native documents, copies, packages and web downloads default to
  `.pgnr`. Shared extension detection keeps `.pagenor` and case variants readable;
  saving an existing/explicit path preserves it. ZIP contents/version/MIME and
  private recovery checkpoints are unchanged. Desktop filters and platform package
  metadata recognize both extensions. Six new regressions and actual macOS Save
  without a suffix/Open pass. Local gate: 854 passed, 3 ignored; 16 WASM packages,
  Mac bundle and Windows/Linux strict UI-library checks pass. Native installer
  and OS file-association validation awaits the milestone matrix.

- P09.1: Desktop launch now shows the supplied splash artwork once for about
  1.2 seconds from its first UI frame, including launches with documents already
  opened or recovered. The transient shared UI state preserves documents and
  preferences; the web keeps its loader. Two timing/document/recovery regressions,
  inspected CPU renders and an observed macOS splash-to-welcome transition pass.
  Local gate: 848 tests pass, 3 ignored; 16 WASM packages and Windows/Linux strict
  UI-library checks pass. Native Windows/Linux execution awaits the milestone matrix.

- P10.1: Local macOS builds now have a build-and-launch command that assembles
  `target/macos-dev/Pagenor.app` with the supplied owner icon and release bundle
  metadata. It ad-hoc signs the current binary and refuses to rebuild a running
  bundle. Finder icon display, native launch, icon decoding/hash, signature and
  CLI version pass locally; the workspace gate passes (846 tests, 3 ignored).
  Windows/Linux packaging stays intact; distribution/signing gates remain open.

- P02.5: Text Copy/Cut now reach the system clipboard through the shared UI
  command path, including menus/control and canvas events. Same-batch Copy/Paste
  uses queued text; internal formatting and undo remain intact. Four UI regressions
  and actual bidirectional TextEdit/native Pagenor transfers pass. Native control
  quit, fresh native recovery and screenshot inspection pass. Other application
  transfers, native menu/keyboard delivery and timer/storage/OS-exit cases remain open.

- P02.4: Desktop Quit and root-window close checkpoint current edits before
  closing, cancel on write/cleanup failure, and allow retry. macOS Quit dispatches
  through the shared command; the exit hook makes a best-effort final save. Eight
  UI regressions, six isolated process probes and inspected CPU images pass.
  Live native shutdown, actual storage failures and external clipboard transfer
  remain pending.

- P02.3: The recovery timer handles owned checkpoints after clean undo, bounds
  oversized intervals before multiplication, schedules its own wakeup and reports
  successful retry after failure. Seven UI regressions, real-time headless timer
  scenarios and CPU failure/retry images pass. Live window testing remains pending
  after an AppKit connection timeout; shared shutdown protection follows in P02.4.

- P02.2: Recovery keeps its checkpoint when reopened and tracks owned recovery
  keys independently of runtime document IDs. New entries avoid pending files;
  save/revert/close clean up only owned checkpoints. Staged, flushed writes keep
  previous snapshots usable on failure. Seven regressions and a fresh-process
  restart probe cover repeat exits, UID reuse, multiple entries and cleanup.
  Real external clipboard transfer and native timer/exit validation remain open.

- P02.1: External/plain paste creates one undo step and one journal entry.
  It reuses text insertion without dispatching a second command; undo captures
  the selection before edits. Unicode/formatting persistence and UI bridge tests
  pass. P02.2 addresses the immediate repeat-crash recovery gap.

- P01.10: Unicode family-name matching now follows the menu grouping key.
  Case variants resolve the same lazy installed/memory/document fonts, duplicate
  faces are excluded and document fonts keep precedence during composition.
  Accents, punctuation, spacing and original names remain distinct/intact.

- P01.9: Font diagnostics recognize style aliases and custom variable base styles
  using the existing lookup normalization, without loading glyph data for checks.
  An independent macOS inventory comparison covers all 272 SFNT families; two
  legacy Multiple Master families expose no SFNT tables and remain unsupported.
  Document scopes and original requested style names are preserved.

- P01.8: The background worker revalidates one native table source per check.
  Unused sources request metadata; loaded sources compare bounded current tables.
  Changed glyphs/menus and recovered table access trigger the existing rescan.
  Physical-source transitions and retained metadata expansion have regressions.
  Native notifications remain pending.

- P01.7: Manual rescan invalidates native table caches even when registered
  identities stay unchanged. Data reloads lazily; old requests cannot publish
  after the rescan. Variable instances, document scopes and held layout handles
  have regression coverage. P01.8 adds automatic same-identity change detection;
  notification-driven detection remains open.

- P01.6: Replaced installed-font files now reload lazily using published file
  versions. New glyphs, collection styles and named/custom variable instances
  resolve current data; existing layout/document handles remain valid.
  Identical bytes preserve identity, and stale loader results are rejected.
  Windows/Linux private-memory API limits are documented; no new backend is claimed.

- P01.5: macOS native font identities now support fonts without readable files.
  Core Text supplies bounded metadata for menus and glyph tables on demand;
  activation epochs prevent reuse after observed withdrawal/reactivation.
  Real memory-font shaping/outline tests pass. Windows/Linux memory-font
  backends remain pending; work stays local.

- P01.4: Newly discovered exact styles and variable instances resolve before a
  previously loaded closest style. macOS also checks registered PostScript names;
  hidden paths/families and explicit extensionless font files have regression
  coverage. Development/testing stay local until a completed milestone.

- P01.3: Installed picker families/styles follow the refreshed catalogue even
  after a face was loaded. Cached rendering handles and document-local fonts
  remain intact; the removal/restoration regression and full local gate pass.
  Native adapter execution and activation evidence: `docs/pagenor/FONT_DISCOVERY.md`.

- P01.2: Windows installed-font registry and Linux/FreeBSD Fontconfig locations
  now feed the shared lazy catalogue. XDG folders and bounded configuration
  includes are rescanned for new files. Cross-target compilation and Clippy pass;
  native host comparison tests and the Windows path-assertion correction await CI.
  Details: `docs/pagenor/FONT_DISCOVERY.md`.

- P01.1: Installed-font diagnostics and manual rescan are available through Type
  and engine commands. Native background refresh discovers new file-backed fonts,
  including Core Text activation locations on macOS, without eagerly loading font data.
  Rescans clear stale fallback misses and retain document-local fonts. Validation
  and remaining activation/platform gaps: `docs/pagenor/FONT_DISCOVERY.md`.

- P00.1: Phase-based validation now starts with a locked workspace quality gate,
  macOS/Windows/Linux CI, Rust 1.95 metadata and a web-app WASM check. The local
  baseline and remaining ownership/release findings are recorded in
  `PHASE_STATUS.md` and `docs/pagenor/PHASE_00_REPORT.md`; Phase 00 is still in progress.

- AR1.1: Kashida justification tests verify space fallback without Tatweel fonts using an isolated font database, while retaining elongation assertions when fonts support it.

- AR1: Arabic IDML controls now retain character direction, kashida switches, diacritic offsets/forms, paragraph policy values and independent story/table directions. Shaping preserves joining context and script/language selection; paragraph-level bidi, contextual digit conversion, mark-safe elongation, RTL columns/tables and physical column insertion have regression coverage. Vendor-specific justification and diacritic presets remain explicit preflight warnings; see docs/arabic-typography.md.
- CJK1: IDML composite fonts now retain character mappings and metrics; explicit aki, tsume, jidori, em alignment/leading, named/custom kinsoku boundaries, hanging punctuation, tate-chu-yoko offsets and kenten symbols flow through composition and IDML export. Automatic kerning no longer imports inactive numeric values as enormous manual spacing. Composite definitions are editable through commands with undo/cache invalidation. Mojikumi definitions are retained and unsupported spacing-table / push-in-push-out policies are reported by preflight. Full InDesign CJK composition parity, frame-grid layout and detailed ruby/warichu remain unfinished.

Pagenor aims at full Adobe InDesign parity — and to be better: faster, open (documented JSON format + IDML), scriptable by agents (MCP), and available on the web.

## Status (2026-10-01)

**Working today**
- Document model: facing/non-facing spreads, parent pages with page-number markers, sections, layers, frames (text / graphic / unassigned), threaded stories, paragraph/character/object styles with based-on, swatches (process/spot/tints/gradients), corner options, text wrap, drop shadow, opacity/blend.
- Text engine: Knuth–Plass paragraph composer and single-line composer, dictionary hyphenation (public-domain Moby list + our own trained Liang patterns), justification with word/letter/glyph-scaling ranges, keep options (keep with next, keep lines together, widows/orphans), balance ragged lines, optical margin alignment, hyphenation zone/limit, columns, threading across frames and spreads, tabs, bullets/numbering, rules, shading, baseline grid, first-baseline options, vertical justification, wrap exclusions, overset detection, caret/hit testing.
- Rendering: vello_cpu (SIMD, multithreaded) — pages, items, images with mip levels, gradients, composed text.
- UI (egui): application bar + menus, Control panel (object and text modes), Tools panel with flyouts, document tabs, rulers, pasteboard, guides (margins, columns, bleed, baseline grid), frame edges, selection handles, text ports and threads, Properties / Pages / Layers dock, Swatches, Styles, Character, Paragraph, Stroke, Text Wrap panels, New Document and Preferences dialogs, Quick Apply (⌘Return), 4 brightness themes.
- Pen tool, Direct Selection anchor/handle editing, rotate from corners, Rotate/Scale/Shear tools, Eyedropper, snapping & smart guides, Align/Distribute, Find/Change (text + GREP), Story Editor, Paragraph Style Options, Effects panel, hidden characters, native macOS menu bar, measured InDesign 2026 look (Medium Dark): Properties sections per selection state, 21 pt spinner fields with arithmetic, Contextual Task Bar, inverse (black) text selection, Pages panel with drag-and-drop reorder / parent apply, Layers panel with per-object rows.
- Tools: Selection (click, marquee, move, Alt-duplicate, resize handles), Direct Selection, Type (draw frame, click caret, select, type), Rectangle/Ellipse/Polygon (+ frame variants), Line, Hand, Zoom.
- PDF export (krilla): real selectable text with embedded font subsets, DeviceCMYK/RGB + spot Separations, bleed boxes, crop/bleed marks + page info, pages or spreads, PDF/A-2b (PDF/X-4 output intent pending) — `file.exportPdf`, File › Export PDF…, `pagenor-cli run --export out.pdf`.
- IDML interchange (`pagenor-idml`): export and import of swatches, styles, fonts, preferences, parent spreads, spreads/pages, frames, groups, images (embedded or linked), formatted threaded stories — opens in InDesign 2026 and round-trips InDesign-exported files (`file.exportIdml`, `file.openIdml`, File → Export IDML…, CLI `--in x.idml` / `--export x.idml`).
- Tables (M8): tables anchored in stories (header/footer/body rows, merged cells, per-edge strokes, fills + alternating fills, insets, vertical justification, at-least/exact row heights); composed into the text column (columns scale to fit, rows break across columns/frames with repeating headers/footers, overset); rendered, exported to PDF (real text) and IDML (export + import); Table menu, Table panel, Create Table dialog, caret/typing/Tab navigation and cell selection in cells; `table.*` commands.
- Hyperlinks (text or frames → URL / e-mail / page) and bookmarks, exported as PDF link annotations and outline.
- EPUB 3 (reflowable) export: stories in reading order, CSS from paragraph/character styles, images, navigation (`file.exportEpub`, CLI `--export x.epub`).
- Data Merge (one linked CSV, TSV, text, or Excel file; text, image, QR, and hyperlink placeholders; preview; a new document with one record per page or records tiled on a single page), spell checking (public-domain Moby list + document dictionary, suggestions), Step and Repeat (count or grid), snippets, object styles, Numbering & Section Options.
- Footnotes: Type › Insert Footnote (caret moves into the note), Document Footnote Options (numbering style incl. symbols, start/restart per page/spread/section, prefix/suffix, reference position/character style, paragraph style, separator, spacing, first baseline, rule above); notes composed at the bottom of the referencing column with the body text making room; edited in place on the canvas; rendered, exported to PDF (real text) and IDML (InDesign's structure, verified opening in InDesign 2026) and imported from IDML; `footnote.*` commands.
- Cross-references and text anchors: Insert Cross-Reference (paragraph by style, or text anchor), 11 formats with InDesign's building blocks (full/partial paragraph, paragraph text/number, page number, anchor name, chapter, file name) plus user formats; resolved live at composition — never out of date, no Update step; unresolved destinations show `??`; PDF link annotations to the destination page; IDML export/import as InDesign cross-reference sources, hyperlinks, text destinations and formats (verified in InDesign 2026); `xref.*` / `anchor.*` commands.
- Index: page references (topic from the selection or up to 4 levels, sort keys; current page, to end of story, next n paragraphs, suppressed, See / See also), Generate / Update Index (section headings, nested or run-in, page ranges merged, Index Title / Section Head / Level 1–4 styles); IDML export/import as InDesign topics, page references and topic cross-references (verified in InDesign 2026); `index.*` commands.
- Text clipboard keeps formatting (character/paragraph formats, footnotes, cross-references, index markers, tables) and falls back to plain text when the system clipboard changed; Paste without Formatting (⇧⌘V); Change Case (UPPERCASE, lowercase, Title Case, Sentence case — formatting kept); Type › Insert Special Character / White Space / Break Character submenus.
- Anchored objects: items flow in the text inline (on the baseline, with Y offset; lines grow to fit) or above the line (left/center/right, space before/after); paste copied items into text to anchor them; Anchored Object Options and Release (back onto the page where shown); rendered, exported to PDF and to/from IDML (InDesign anchored object settings; verified in InDesign 2026); copy/paste and threading carry them; `anchored.*` commands.
- Ruler guides: drag out of the rulers (page guides, or spread guides on the pasteboard), drag to move, drop on a ruler to delete; Layout › Create Guides (rows/columns with gutters, fit to margins or page); Delete All Guides on Spread; `guide.*` commands.
- Links panel: every placed graphic with status (OK / Modified / Missing / Embedded, checked against the file on disk), page and effective PPI (low resolution flagged); Relink…, Go To, Update, Embed; `links.*` commands.
- Missing fonts: highlighted pink on screen (never in output), listed by Preflight; Type › Find/Replace Font (fonts in the document, missing first; replace in text, cells, footnotes and styles); `font.list` / `font.replace`.
- Missing glyphs, as in InDesign: a character the applied font lacks is drawn as that font's missing-glyph box (its .notdef; a drawn box when the font's is empty), on screen and in PDF (as a path, so PDF/A and PDF/UA exports take it), and Preflight lists the characters by font. Preferences › Composition › Draw Missing Glyphs from Fallback Fonts (`document.preferences {glyphFallback}`, stored with the document) draws them from other fonts instead; it is off in new documents and IDML imports, on in documents saved before it existed. A missing font's substitute always uses fallback fonts.
- Damage-region repaint: after an edit the canvas re-renders only the frames whose composed text (or the items that) changed and patches its texture — pixel-identical to a full render; typing in a frame repaints in under a millisecond instead of a full-viewport render (22 ms measured on the sample).
- Crash recovery: unsaved documents are written to the recovery folder every 30 s and reopened (unsaved, remembering their files) after a crash; File › Revert and Save a Copy…; `file.recovery.*`, `file.revert`, `file.saveACopy`.
- Edit › Paste Into (copied objects become a frame's content, clipped by it; rendered, PDF and IDML both ways — verified in InDesign 2026); Polygon Settings (sides, star inset; double-click the Polygon tool).
- Color panel (fill/stroke proxy, CMYK/RGB/Lab sliders with channel ramps, tint slider for swatches, spectrum ramp, Add to Swatches) and Color Picker (double-click the proxy); mixed colours are unnamed (not listed in Swatches until added); `object.color`, `swatch.addToSwatches`, `swatch.addUnnamed`.
- Place text files: Word (.docx: styles by name with their attributes, bold/italic/underline/size/font, footnotes, tables, tabs, breaks), RTF and plain text, into the insertion point, the selected frame or a new frame; autoflow adds pages and threaded frames until the text fits; Remove Styles option (`pagenor-textimport`).
- Gradient Feather (⇧G tool, `object.gradientFeather`): objects fade along a linear or radial opacity gradient, on screen and as a soft mask in PDF.
- Script Label (`object.label`, `object.findByLabel`), alt text (`object.altText`), Isolate Blending / Knockout Group (`object.transparencyGroup`; on screen, PDF isolates), Table ▸ Sort (`table.sortRows`, numeric-aware, header/footer fixed).
- File ▸ Export Text (`file.exportText`): a story as Text Only or RTF (fonts, styles, alignment, indents, tables as tab-separated rows); Export EPUB now asks for a path in the UI.
- Tagged PDF (`file.exportPdf {tagged}`, on by default from the UI): structure tree with stories as paragraphs in reading order, figures carrying their alt text, parent-page items and printer's marks as artifacts.
- Global Light (`object.globalLight`, `globalLight` on drop/inner shadows; Use Global Light in the Effects panel) and Generate Static Caption (`object.caption`: template over name, path, alt text, label, effective ppi, dimensions, format; any side, offset, style).
- Control panel scale X/Y, rotation and shear fields (`transform.info`, absolute `transform.set {scaleX, scaleY, rotation, shear}` about the reference point) and Preferences › Transformations are Totals for nested objects and placed graphics.
- Variable fonts: each named instance is a style (Font menu, shaping with feature variations, metrics, outlines) and embeds at its axis settings in PDF; free axis sliders still to come.
- Object ▸ Select (first/next above, next/last below, container, content, previous/next in group), full Object ▸ Convert Shape (`object.convertShape`), Table ▸ Split Cell Horizontally/Vertically.
- Attributes panel (`object.attributes`: Overprint Fill/Stroke/Gap, Nonprinting), with Overprint Gap in Overprint Preview and IDML.
- Page Numbering View (Preferences: section or absolute): page labels everywhere follow it; Go to Page takes section names, numbers or `+n`.
- Preferences › Advanced Type (document `advancedType`: superscript/subscript size and position; IDML TextPreference); subscripts now drop 33.3% like InDesign.
- Preferences › Composition › Highlight: keep violations, H&J violations (three shades), custom tracking/kerning, substituted fonts.
- File ▸ Export HTML (`file.exportHtml`): one self-contained page in reading order, styles as CSS, images inline, alt text from Object Export Options (also in EPUB).
- Swatches panel colour groups (`swatch.newColorGroup`, `moveToGroup`, `ungroupColorGroup`, `renameColorGroup`; folders and a row context menu; IDML ColorGroup).
- Object ▸ Generate QR Code (`object.qrCode`: web link, text, SMS, email, business card; vector modules as a compound path, into a frame or re-encoded in place).
- Conditional text (`condition.new/options/delete/apply/list`, Conditional Text panel): hidden conditions drop their text from layout and exports, indicators underline on screen, IDML Condition/AppliedConditions.
- Endnotes (`endnote.insert/edit/delete/list/options`, Type ▸ Insert Endnote): references number through the document, the endnote frame on a new last page lists them under a heading; IDML keeps the listed text only.
- Editorial notes (`note.new/edit/delete/convertToText/list`, Notes panel): anchored in text, flagged on screen, never printed or exported.
- File ▸ Print Booklet (`file.printBooklet`): saddle-stitch or 2-up consecutive printer spreads as PDF, padded with blanks, with a gap between pages.
- Edit ▸ Transparency Blend Space (`edit.transparencyBlendSpace`; RGB for web/mobile documents; IDML TransparencyPreference) — exported PDFs give pages with transparency a DeviceCMYK or DeviceRGB page group, so viewers and RIPs blend in that space. On screen, spreads with transparency in a CMYK blend space are shown through the working CMYK profile (their colours keep to its gamut, as when composited in CMYK).
- High Contrast interface theme (fifth Interface Color Theme: black chrome, white text, yellow selection).
- Place Excel workbooks (.xlsx): the first worksheet's used range becomes a table (shared/inline strings, numbers, booleans).
- Object Library (`library.new/open/add/place/remove/list/json/close`, Library panel): `.plib` files of snippets that keep stories, styles, swatches and images.
- Ink Manager (`ink.list`, `ink.options`; Swatches panel menu): All Spots to Process, per-ink conversion and ink aliases in PDF output; IDML ConvertToProcess/AliasInkName.
- Appearance of Black complete: Printing/Exporting rich black for RGB output (PNG), Overprint [Black] Swatch at 100% (document, IDML OverprintBlack).
- Hyperlinks and Bookmarks panels (Window ▸ Interactive): new from the selection, edit (`hyperlink.edit`), go to source (`hyperlink.goToSource`), rename bookmarks (`bookmark.rename`), go to page, delete.
- Nested line styles (`nestedLineStyles` paragraph attribute, Paragraph Style Options › Drop Caps and Nested Styles, IDML AllNestedLineStyles): the first lines restyle until the line breaks settle.
- User dictionary hyphenation exceptions (`hyphenation.addException/removeException/list`, Edit ▸ Spelling ▸ User Dictionary): `ex~am~ple` breaks only at `~`, a plain word never hyphenates.
- Frame Fitting Options (`object.fittingOptions`, Object ▸ Fitting ▸ Frame Fitting Options…): Auto-Fit refits on resize, fitting, Align From reference point, crop amounts; IDML FrameFittingOption.
- Paragraph borders (per-side weights, colour, tint, offsets) and shading offsets, drawn per column part of split paragraphs (also when overset); Paragraph panel controls; IDML ParagraphBorder*/ParagraphShading*Offset.
- Font menu: search, favourites (stars, Show Favorites Only; `favoriteFonts` preference) and each family previewed in its own face. Western (and other-script) families come first, then Japanese, Simplified Chinese, Traditional Chinese and Korean ones, each group alphabetical and set off by a separator; the group comes from the font's own data (`meta` languages, family names in a CJK language, OS/2 code pages, else what it covers), read once (installed fonts by the font scan, without loading them). CJK families show their native names, the English name on hover; Preferences › Type › Show Font Names in English (`showFontNamesInEnglish`) shows English names throughout. Search matches either name; documents store the English name. Every family list (Character panel, Control bar, Properties, style dialogs, Find Font, Glyphs) follows this order.
- Application preferences (engine `Prefs`) now persist across launches (desktop: prefs.json beside ui.json).
- View ▸ Fit Selection in Window (⌥⌘=) and Find/Change ▸ Object (`find.objects`, `find.changeObjects`: by fill, stroke, weight, opacity, kind, layer, label).
- Custom workspaces (`window.newWorkspace/deleteWorkspace/resetWorkspace`; workspace switcher shows the current one): bars and panel arrangement saved by name.
- Word/RTF Import Options (File ▸ Place with Import Options…, `place.styles`, `file.place {styleMap, styleConflicts}`): map imported styles onto the document's, use existing / redefine / auto-rename on conflicts, or remove formatting.
- Custom anchored objects (`anchored.insert/options {position: custom}`): placed relative to the anchor, frame, column, page margins or page edge with reference points and offsets, no space in the text, keep within column; IDML Anchored position.
- Variable font axes: Character panel sliders; any axis values (`fontStyle: "Bold {wght:650}"`) make that instance for layout, screen and PDF.
- Image Import Options › Crop to for placed PDFs (`file.place {pdfCrop: crop|trim|bleed|art|media}`, Place PDF dialog): the frame shows that page box.
- Stroke styles: striped (thick-thin… and custom bands), wavy and straight-hash strokes now draw on screen and in PDF; named custom stripe/dash/dot styles (`strokeStyle.new/delete/list`, Stroke panel Type list; IDML Striped/Dashed/DottedStrokeStyle).
- Object ▸ Primary Text Frame (`object.primaryTextFrame`): any text frame's story can become (or stop being) the primary story Smart Text Reflow follows.
- Place a page of an IDML or Pagenor document (`file.place {layoutPage}`): its objects, stories, styles and swatches as one group (INDD files are not readable).
- Articles panel (`article.new/add/remove/options/delete/list`): reading order and content of EPUB and HTML exports.
- Object ▸ Clipping Path (`object.clippingPath {type: alpha|edges, threshold, tolerance}`): traces the placed image and makes the outline the frame.
- Gridify: arrow keys while dragging a frame or shape tool add columns/rows (`frame.grid`), one undo step.
- Gap tool (U, `gap.move`): drag the space between objects (or an object and the page edge) and both sides resize.
- XML basics (`xml.newTag/deleteTag/tag/mapStyle/structure`, `file.exportXml`, `file.importXml`; Tags panel with structure; View ▸ Show Tagged Frames): tagged frames export in reading order, mapped paragraph styles become elements, imported elements flow into frames with matching tags. Inline text tagging (`xml.tagText`, an `xmlTag` character attribute, exported as nested elements) and tag markers (View ▸ Structure ▸ Show Tag Markers) too. DTDs (`xml.loadDtd`, `xml.validate`, `xml.deleteDtd`; Tags panel): loading one adds its elements as tags and names the root; validation checks content models (sequences, choices, `?*+`, mixed content, EMPTY/ANY, parameter entities) and required attributes, and lists each problem with its element path.
- Adjust Layout / Layout Adjustment (`adjustLayout` on Document Setup and Margins and Columns): objects follow each page's margin box to its new size and position.
- Content Collector and Placer tools (B; `conveyor.collect/place/list/clear`): copies of objects (with stories, styles, images) ride a conveyor between pages and documents; Alt-click keeps them on it.
- Linked stories (Edit ▸ Place and Link, `story.placeAndLink/links/updateLink/unlink`): child stories copy a parent, show out of date in the Links panel and update.
- Edit ▸ Menus (`window.hideMenuItem`): hide menu items; menus with hidden items end in Show All Menu Items.
- View ▸ Separations Preview (`view.separations`): one process plate as ink density, or areas over an ink limit in red (a GCR preview of the rendered view; spot plates and the flattener preview aren't there).
- Separations Preview plates now come from each object's own colour (CMYK as authored; RGB and images through the colour settings, images via a cached 17³ table), so pure black shows only on the black plate.
- Language (Character panel ▸ Language, `type.char {language}`): text in other languages isn't hyphenated or spell-checked with the English dictionaries, typographer's quotes follow the language (German „…“, French and Russian «…», Swiss, Spanish, Dutch, Nordic, Japanese and Traditional Chinese 「…」, Simplified Chinese “…”), the font's localized forms (OpenType `locl`: Turkish i, Bulgarian Cyrillic, Japanese / Chinese / Korean glyph variants) follow it too, and so does the fallback for CJK characters the font lacks (with Draw Missing Glyphs from Fallback Fonts on): Japanese, Simplified Chinese, Traditional Chinese and Korean each try their own installed system fonts (Japanese starts with the craft-fonts Japanese faces, Mincho first, when built with them; Hangul, kana and Bopomofo pick their language's fonts whatever the language). Language names from IDML keep their spelling; locale codes as InDesign writes them for some languages (`de_DE_2006`, `nl_NL_2005`; any `ll`, `ll_CC`, `ll-CC`) are their language and region for all of the above, and Chinese is also recognised in its common spellings ("Simplified Chinese", "Chinese (Traditional)", `zh_CN`, `zh-Hant`). Hyphenation dictionaries for other languages aren't bundled yet.
- Books (`book.new/open/add/remove/list/styleSource/paginate/syncStyles/exportPdf`, Book panel): `.pbook` files, continuous page numbering, style sync from a style source, one merged PDF.
- Track Changes (`changes.track/list/acceptAll/rejectAll`, Type ▸ Track Changes): typing is marked added (highlighted on screen), deletions are kept but hidden from layout and exports until accepted or rejected. Per-change accept/reject and authors aren't there yet.
- Track Changes: per-change Accept / Reject (`changes.accept`, `changes.reject`) and a Track Changes panel listing every change.
- Move table rows and columns (`table.moveRow`, `table.moveColumn`, Table panel buttons; with whole rows or columns selected, drag them with the Type tool — `table.dropCells`).
- Object States (`states.create/show/rename/release/list`, Object States panel): multi-state objects show one state on screen and in PDF.
- Buttons (`button.set/clear/list`, Buttons and Forms panel): go to page / next / previous / first / last / URL as PDF link annotations. Form fields too (`form.set/clear/list`: text fields, check boxes, combo and list boxes, signature fields; required, multiline, defaults) exported as AcroForm widgets through an incremental update (not in PDF/X).
- Power Zoom (Hand tool: press and hold still for half a second, or Alt-press): zoom out to the spread, aim the red view rectangle, release to zoom back in there.
- Graphic cells (`table.placeGraphic`, `table.textCell`, Table › Convert Cell Type, Table panel): images in table cells, fitted or filled, clipped to the cell on screen and in PDF.
- Live Distribute (Space while dragging a selection handle; `transform.resize {distribute}`): objects keep their size and spread with the bounds.
- Liquid Layout (`liquid.pageRule/object`, `guide.liquid`, `layout.createAlternate`, Liquid Layout panel): scale / re-center / guide-based / object-based page rules applied on page resize and Document Setup; dashed liquid guides; alternate layouts as resized page copies in a named section. Text in an alternate layout stays linked to the original stories (Links panel shows them out of date; `story.updateLink`).
- Split Window / New Window (`window.split`, `window.newWindow`, Window ▸ Arrange): two side-by-side views of the document with their own zoom, scroll and render cache. New Window puts the second view in its own OS window (an egui viewport; a floating window on the web).
- Transparency flattener (`file.exportPdf {flatten: high|medium|low|ppi}`, Edit ▸ Transparency Flattener Presets): each object involving transparency becomes an opaque image of its area over what's beneath it, so everything else stays vector (spreads with transparent parent items are rasterised whole).
- Video and sound (place .mp4/.mov/.webm/.mp3/.wav…, `media.options/get`, Media panel): media frames show a poster or placeholder on screen, in print and in PDF, and export as HTML5 `<video>`/`<audio>` in EPUB. Interactive PDF export (`file.exportPdf {media: true}`, File ▸ Export PDF (Interactive)) embeds each file and plays it in a Screen annotation through a rendition action (controls, loop, play on page load). No in-app playback.
- Right-to-left text (World-Ready basics): right-to-left runs are shaped right to left, lines are reordered by the Unicode Bidirectional Algorithm (matched against the reference implementation), Paragraph Direction (`type.para {direction}`, Paragraph panel, IDML ParagraphDirection, EPUB `dir`). Digits (`digits`: Default, Arabic, Hindi, Farsi, Native by language; Character panel for right-to-left text; IDML DigitsType) draw 0–9 in another script. Kashidas (`kashidas` paragraph attribute, on by default): justified Arabic lines stretch at one join per word (after seen/sad when possible, never inside lam–alef) with a drawn tatweel before word spaces widen. Carets follow the drawing: the caret before a right-to-left glyph sits at its right edge, arrow keys move as drawn (by words in the line's direction), clicks find the nearest drawn caret position and selections highlight the selected glyphs. Binding: Right to Left (Document Setup, `layout.documentSetup {binding}`, IDML PageBinding) makes page 1 a left page and lays spreads out right to left.
- Interface language (Edit ▸ Interface Language, `app.language`): German, French, Spanish, Japanese, Simplified Chinese and Arabic menu titles, common menu items and panel names (our own translations). Common dialog, panel and tooltip labels are also localized, with English fallback for untranslated strings. Arabic translations cover every key in the shared interface translation table, including menu commands and utility labels. The Type menu's Story Direction, Horizontal/Vertical, Tate-Chu-Yoko, Ruby… and Kenten are translated too. Arabic labels use Unicode bidi run ordering and contextual shaping; welcome and new-document form layouts follow RTL direction independently of document binding. CJK and Arabic glyphs come from [craft-fonts](https://github.com/storytold/craft-fonts) when built with `CRAFT_FONTS_DIR` (all releases): BIZ UDPGothic and Noto Sans CJK SC for the UI (Chinese forms first in the Chinese interface), Noto Sans Arabic, and Mincho for Japanese documents (the web build embeds BIZ UDPGothic only); otherwise from system fonts.
- Page transitions (`page.transition/transitions`, Page Transitions panel): twelve transition types with speed and direction per spread, written into exported PDFs as `/Trans` entries through an incremental update.
- PDF/X-4 export (`file.exportPdf {standard: "x4"}`): an output intent with our own Generic CMYK ICC profile (lut16 tables generated from the parametric press model), GTS_PDFXVersion in Info and XMP, and built-in PDF/X-4 checks reported as warnings. qpdf finds no errors in the result.
- EPS place: placed at its (high-resolution) bounding box and shown, printed and exported through its TIFF preview, or a placeholder when it has none. PostScript itself isn't interpreted.
- Preferences: Dictionary (user dictionary; `spelling.setWords/words`), Spelling (Dynamic Spelling: red squiggles on the canvas, also Edit ▸ Spelling), Autocorrect (word list, applied as you type), Notes, Track Changes, Story Editor Display and File Handling (recovery interval) sections.
- Colour management exposed: Edit ▸ Color Settings (`color.settings`: RGB/CMYK working spaces, intent, black-point compensation; `color.loadProfile` for ICC files; `color.convert` through the working spaces, with gamut checks) and View ▸ Proof Setup / Proof Colors (press, sRGB, legacy Mac RGB, colour-blindness simulations, simulate paper) on the canvas.
- Keyboard: Tab / Shift+Tab hide and show all panels / all but Tools (`window.hidePanels`, `window.hidePanelsExceptTools`); Cmd+F6 / Cmd+Shift+F6 cycle documents; panel fields are reached with Tab and Escape returns to the layout.
- Smart guides and snapping (View ▸ Snap to Guides / Snap to Document Grid / Smart Guides, `view.snapPreferences` and Preferences ▸ Guides & Pasteboard): moving, drawing, resizing, rotating and pen/anchor drags snap in order to the document and baseline grids, ruler guides, margins and columns, other objects' edges and centres (parent-page items too, by visible stroked bounds), equal spacing, matching dimensions and matching rotation, with guides, gap marks and measurements drawn on the canvas. Shift and Cmd constrain the gesture.
- View ▸ Rotate Spread (`view.rotateSpread`, `layout.rotateSpreadView`): the spread in view turns in quarter turns on screen (stored with the document; output is unaffected). The canvas maps each spread through its own transform, so tools, snapping, resize handles, guides, overlays and rendering all follow the turned spread.
- Vertical type (Vertical Type Tool, Type ▸ Story Direction `type.storyDirection`, IDML StoryOrientation): the direction belongs to the story, so every frame of a thread is vertical or horizontal together and a frame threaded onto a story takes its direction (`object.textFrameOptions {vertical}` sets the frames' stories); lines run top to bottom and follow each other right to left; the arrow keys follow them (Down and Up along the line, Left and Right to the next and previous line, keeping the position along it); CJK characters stay upright and other text turns, on screen and in PDF; CJK line breaking with basic kinsoku (also in horizontal text); Korean breaks at spaces, word by word (Hangul in any language, hanja in Korean text), or between syllables when the paragraph asks for it (`type.para {koreanCharBreaks}`, Paragraph Style Options ▸ Justification; not written to IDML). Upright text is shaped top to bottom: punctuation and brackets take their vertical forms (OpenType `vert`; `vkrn` and `vpal` through OpenType features), glyphs advance by their vertical metrics (`vmtx`) and hang from their vertical origins (`VORG`) on the centre of the ideographic em box (`BASE`, else OS/2), which the first line's ascent follows; turned text keeps its horizontal forms. Tate-chu-yoko (Type ▸ Tate-Chu-Yoko, `type.tateChuYoko`, IDML `Tatechuyoko`) sets a run such as "12" side by side within one em of the line. Ruby (Type ▸ Ruby…, `type.ruby`, IDML RubyString; group ruby spread 1-2-1 or centred) and kenten emphasis dots (`type.kenten`, IDML KentenKind) are set over their text — to its right in vertical frames — without changing line breaks. Warichu and ruby options (per-character ruby, positions, overhang) aren't there yet.
- Math expressions (Type ▸ Insert Math Expression…, `math.insert`, `math.svg`): a LaTeX subset (fractions, scripts, roots, big operators with limits, Greek, relations and operators with math spacing, `\left…\right`, `\text`) typeset by our own engine into SVG, placed inline at the cursor or as a frame; the LaTeX is kept as alt text and can be edited.
- Object Layer Options for placed PDFs (`object.pdfLayers`, `object.layerOptions`, Object ▸ Object Layer Options…): the file's optional-content layers listed and shown or hidden per frame on screen; in exported PDFs such graphics go out as 300 ppi images of their visible layers.
- Effects: Inner Glow (edge or centre), Bevel and Emboss (inner bevel lit from the light angle), Satin (`object.innerGlow`, `object.bevel`, `object.satin`); PDF export now keeps soft effects: objects with shadows, glows, feathers, bevels or satin go out as 300 ppi transparent images of their appearance (threaded text frames excepted).
- Directional Feather (`object.directionalFeather {widths: [top, left, bottom, right]}`): each side fades over its own width.
- View ▸ Flattener Preview (`view.flattenerPreview`): objects involving transparency highlighted in red; knockout groups export as images (PDF output has no knockout groups).
- Live captions (`object.caption {live: true}`, Object ▸ Captions ▸ Generate Live Caption): the caption text is re-filled from the source object's metadata after every edit (same undo step).
- Style Export Tagging (`style.exportTag`, Paragraph Style Options ▸ Export Tagging): paragraph and character styles choose their HTML element and class in EPUB/HTML.
- Tagged Text: export (`file.exportText {format: "tagged"}`) and place (detected by its header; ASCII, UTF-8 or UTF-16) with paragraph and character styles, font, typeface and size, and `<0x…>` characters.
- Tagged PDF: one structure element per paragraph (H1–H6 and BlockQuote from Export Tagging, else P) in reading order, frame fills, strokes, rules and tables as artifacts, figures with alt text.
- EPUB: cover image from the first page (`file.exportEpub {cover}`), and Fixed Layout export (`file.exportFixedEpub`, File ▸ Export EPUB (Fixed Layout)…): pre-paginated pages as 144 ppi images with their text kept for search and read-aloud.
- Interactive PDF options (`file.exportPdf {fullScreen, bookmarksPanel, pageLayout, view, advanceSeconds}`, File ▸ Export PDF (Interactive)…): page mode, layout, opening view and page timing in the catalog and pages.
- Object Export Options (`object.exportOptions`): tagged-PDF artifact, EPUB/HTML rasterize (the object as an image), alignment and page break before, alt text.
- Scripts panel (Window ▸ Utilities ▸ Scripts, `script.run`): saved command scripts (with `$N.path` references to earlier results) run as one undo step; edit, add, load and delete them in the panel. Scripts use Pagenor's command language, not ExtendScript.
- Drag and drop text editing: drag selected text to move it (Alt copies) with its formatting (`text.release`).
- Image Import Options: place any page of a multi-page PDF (`file.place {pdfPage}`; the app asks which page), shown on screen and embedded as that page in PDF export.
- Define Lists: named numbered lists that continue across stories in page order (`list.define`, paragraph `listName` / `startAt`), round-tripped through IDML; `null` in `type.para` / `type.char` now removes an override.
- View › Overprint Preview (⌥⇧⌘Y): overprinting fills, strokes and 100% [Black] text mix with what's beneath on screen.
- Type on a Path (⇧T tool, `type.onPath`): a story along any line, curve or shape on screen and in PDF; Type on a Path Options (start, flip, baseline/ascender/descender/centre) and Delete Type from Path. Round-trips through IDML as TextPath; on-path caret drawing is still to come.
- Preferences › Appearance of Black: 100% K on screen accurately (dark grey) or as rich black (`window.richBlack`); the printing/export side is still to come.
- Spreads of up to 10 pages: Add to Previous Spread and Allow Spread to Shuffle (Pages panel; `layout.pages.toSpread`, `layout.spreadShuffle`); inserted pages go around kept spreads.
- Page tool (⇧P) and `layout.pageSize`: pages of their own size (or a preset); the spine stays put and objects keep their place on their page.
- File › Print (⌘P): printer, copies, page range, spreads, marks, bleed; printed as PDF through the system spooler (`file.print`, `file.printers`, `dryRun` for agents). Shortcuts now act like choosing the menu item (dialogs open), and UI commands win shortcut ties (⌘N opens New Document).
- Nested styles (through/up to N sentences, words, characters, letters, digits, tabs, breaks, spaces or given characters) and GREP styles (regex → character style), edited in Paragraph Style Options and composed under local formatting; `ui.dialog.open` for agents. Nested line styles are still to come; nested and GREP styles round-trip through IDML.
- Table and cell styles (round-trip through IDML): cell styles (fill, insets, vertical justification, strokes, paragraph style) and table styles (region cell styles, border, alternating rows, spacing); apply from the Table panel, edits re-apply to every user; `style.cell.*`, `style.table.*`.
- Style groups: paragraph and character styles shown in group folders, Move to Group (new, existing, none); `style.group` renames every use (styles, stories, table cells, footnotes, object styles).
- Scrubby zoom: drag the Zoom tool left or right to zoom continuously around the press point.
- Guides belong to the active layer: hidden with it (or with its Show Guides off) and locked with it.
- Pencil tool (N): freehand strokes simplified to smooth paths (Alt closes); Smooth and Erase tools re-fit or remove the stretch you drag along (`path.smooth`, `path.erase`).
- Edit › Keyboard Shortcuts: every command, click and press the new keys, conflicts shown, per-command and global reset; `window.setShortcut`.
- Primary Text Frame (New Document) and Smart Text Reflow: pages are added while the primary story oversets and empty ones at the end removed, in the same undo step (Preferences › Type).
- Glyphs panel: every character of any font (rendered by our renderer), search by character or U+code, click to insert, recently used.
- UI scaling (Preferences › Interface, 50–200%; `window.uiScale`); the window title steps aside when the bar is crowded.
- Underline / Strikethrough Options: weight, offset, colour and tint (Character panel; screen, PDF and IDML).
- Relink to Folder and Relink File Extension (`links.relinkFolder`; missing links found by name).
- File › Package (⌥⇧⌘P): the document relinked to a Links folder, the placed files, the font files that draw its text in a Document Fonts folder (the fonts it names and any fallback fonts that draw characters those lack; except fonts whose licence restricts it, OS/2 `fsType`), an IDML copy, an optional PDF and a report (fonts, fallback fonts, links, preflight, instructions); Copy Links To (`links.copyTo`).
- Document fonts: opening a `.pagenor` or `.idml` file loads the fonts in the `Document Fonts` folder beside it (as Package writes it), ahead of installed fonts with the same name, so a packaged document opens with its fonts on another machine. Damaged, oversized (over 256 MB per file, 512 MB in all) and surplus (over 200) font files are skipped and listed in `file.open`'s `warnings`; `font.list` gives each font's `source`. Document fonts belong to their document, as in InDesign: other open documents don't see them (each document with a same-named font draws with its own), and closing the document (or reverting it, or a book chapter's export ending) makes them unavailable. Their files stay in memory until Pagenor quits, so reopening a document doesn't read or load them again; on the web there is no folder to read.
- Load Swatches / Save Swatches for Exchange (.ase: CMYK, RGB, Lab, Gray; spot or process) from the Swatches panel menu; `swatch.load` / `swatch.save`.
- Pathfinder (Object › Pathfinder and a Pathfinder panel): Add, Subtract, Intersect, Exclude Overlap, Minus Back; curves stay curves (flo_curves); `object.pathfinder`.
- Make / Release Compound Path (⌘8; nested paths become holes), Create Outlines (⇧⌘O; one path per text colour), Scissors tool (C) with `path.split`.
- Transform Again / Individually / Sequence Again (⌥⌘3; a moved copy repeats like Step and Repeat), Clear Transformations; Layers: Merge, Delete Unused, Hide/Lock Others, Show/Unlock All; Break Link to Style (paragraph and character).
- Floating panels: tear a panel off the dock (its float button, or drag its header away) into a movable, resizable window; Dock puts it back; positions persist; `window.floatPanel` / `window.dockPanel`.
- Place SVG (vector in PDF export via krilla-svg, resvg on screen, text in the bundled fonts), Photoshop (.psd/.psb composite in its colour mode — Bitmap, Grayscale, Duotone, Indexed, RGB, CMYK, Lab, 8/16-bit — with merged transparency; extra alpha and spot channels don't change the image), BMP, and Illustrator files saved with PDF compatibility (.ai). New `pagenor-images` crate (L2) for sniffing, sizes and decoding. EPS is still missing.
- View › Display Performance: Fast (grey boxes, no effects, ⌥⇧⌘Z), Typical (72 ppi proxies, ⌥⌘Z), High Quality (⌥⌘H, our default — the renderer is fast enough).
- OpenType menu (Character panel): discretionary ligatures, fractions, ordinal, swash, titling, contextual alternates, slashed zero, the four figure styles and stylistic sets 1–20; `type.openType`; mapped to IDML's OTF attributes.
- Add Anchor Point (=), Delete Anchor Point (-) and Convert Direction Point (⇧C) tools (click toggles smooth/corner, drag pulls out handles), with `path.addAnchor`, `path.deleteAnchor`, `path.convertAnchor`.
- Gradients: the Gradient Swatch tool (G) drags a gradient's start and end across objects (Shift: 45°); the Gradient panel sets type, angle, reverse and edits stops on a ramp (drag, add, drag off to remove, location, colour). Edited gradients are unnamed swatches; `object.gradient`; the vector round-trips through IDML.
- Document Setup (⌥⌘P): intent, number of pages, start page # (an even start begins with a left page), facing pages, size presets and orientation, bleed and slug per edge; `layout.documentSetup {}` reports the setup.
- Stroke panel: cap, join, miter limit, alignment, type, Start/End arrowheads (12 kinds, drawn on screen and in PDF; the path is shortened under pointed heads) and gap colour under dashes and dots.
- Preferences (⌘K: General, Type, Units & Increments, Grids, Guides & Pasteboard, Display Performance) backed by `prefs.set` (application) and `document.preferences` (units, increments, grids — undoable, saved with the document); scaling applies to content and scales stroke weights (Include Stroke Weight), and X/Y/W/H measure the stroke's outer edge (Dimensions Include Stroke Weight). Quick Apply (⌘Return) finds paragraph, character and object styles as well as commands.
- Parent item overrides: Cmd+Shift-click a parent item on a page (or Override All Parent Page Items, ⌥⇧⌘L) makes an editable local copy that hides the parent's; Remove All Local Overrides and Detach All Objects from Parent in the Pages panel menu.
- Place PDF: placed PDF pages are sized by their crop box, drawn on screen through hayro (with mip levels) and embedded in exported PDFs as vector form XObjects (krilla), `<PDF>` in IDML.
- Menus expose the engine's features (TOC, numbering & sections, text variables, hyperlinks, cross-references, footnotes, step and repeat, spelling, fitting/content, effects, Data Merge, Preflight, EPUB); any menu command with parameters gets a dialog generated from its parameter documentation.
- Table of contents (generate/update, dot leaders) and text variables (running headers, last page number, chapter number, file name, dates, custom) resolved per page.
- Soft effects: blurred drop shadow, inner shadow, outer glow, basic feather.
- ~200 commands, all reachable through the JSON control channel; headless CLI rendering to PNG.
- MCP server (`pagenor-cli mcp [--connect PORT]`, docs/mcp.md): headless engine or the running app; commands, batch, document/story inspection, page renders as images, window screenshots, pointer/keyboard/dialog input.
- Web build (`apps/pagenor-web`, trunk): the same UI on WebGPU with a WebGL2 fallback; open/place via the browser file picker or drag-and-drop, save/export as downloads.

**Next (in order):** books · New Window / split view · Ink Manager and Separations Preview · XML structure, tags and export · buttons and forms · PDF/X-4 output intent (needs krilla support) · EPS place.

## How far from full parity (estimate, 2026-10-04)

**Breadth: ~99% weighted** (P0 core 99%, P1 100%, P2 97%) over the 275 features of the InDesign catalogue, scored
row by row in [docs/parity.md](docs/parity.md) (`cargo xtask parity` recomputes it). Many features scored done still
lack some of InDesign's options or dialog details, so **overall parity including depth is about 83%**.

**Remaining work: about 170 wall-clock hours of a single Claude Opus 5.5 agent** (±30%), or roughly 60–85 hours with
four agents in parallel on separate crates:

| Work | Estimate |
|---|---|
| Open P0 (2: PDF/X-4 validation against a certified checker, PostScript interpretation for EPS) | 8 h |
| Open P1 (none partial) | 0 h |
| Open P2 (4 partial: localization breadth, INDD pages (not readable clean-room), in-app media playback, warichu) | 14 h |
| Depth and pixel fidelity of every dialog, panel and menu against InDesign 2026 | 90 h |
| Performance (incremental composition, GPU raster) and interchange hardening (IDML/PDF corpus) | 55 h |

Basis: since the first estimate this effort landed about 45 arcs (from parent overrides, Preferences, Stroke panel,
gradients and Pathfinder to table/cell styles, nested/GREP styles and Print) in roughly 14 hours on a heavily loaded
machine — about 1–2 hours per catalogue row.

## Agents: CLI and MCP

Every command is reachable from `pagenor-cli` (`run`, `script` with `$N.path` result references, `app` for the
running window, `describe`, `commands`), from MCP (`pagenor-cli mcp`, tools including `execute` and `batch` with the
same references), and from the app's JSON control channel. See [docs/agents.md](docs/agents.md).

## Milestones

| # | Milestone | State |
|---|---|---|
| M0 | Skeleton + vertical slice | ✅ |
| M1 | Selection, transform, layers, pages, MCP | in progress |
| M2 | Type I (Type tool, threading, Character/Paragraph, composer) | in progress |
| M3 | Styles (nested/GREP, bullets, keeps, span columns) | started |
| M4 | Color & effects | started |
| M5 | Graphics & links | started |
| M6 | Files & export (native, IDML, PDF, PNG/JPEG) | in progress (IDML ✅, PDF export) |
| M7 | Long documents (sections, TOC, index, footnotes, books) | in progress (sections, TOC, text variables/running heads, hyperlinks, bookmarks, footnotes, cross-references, index) |
| M8 | Tables | ✅ core (table/cell styles, rotation, diagonal lines pending) |
| M9 | Performance (MT composition, tiles) | in progress (glyph path cache, bounded RTL label layout cache, culling, parallel compose 6.7× faster, perf harness) |
| M10 | Find/Change, spelling, Preflight | in progress (Find/Change ✅, spelling ✅, Preflight) |
| M11 | Layout power features (liquid/alternate layouts, data merge) | started (advanced data merge, step & repeat, snippets) |
| M12 | Interactive & digital (EPUB, HTML, interactive PDF) | started (EPUB, PDF links/bookmarks) |
| M13 | Automation (scripts, batch) | |
| M14 | 1.0 polish & packaging | |
