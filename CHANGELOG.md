# Pagenor changelog

The current 0.6.0 source is a local development milestone, not a completed cross-platform
release milestone. GitHub release notes record the available artifacts and checks.

## 0.6.0 — local upstream adaptation milestone, 2026-10-12

- Add installed Indic and other interface font fallbacks, refreshing with activated fonts.
- Repair malformed copied note references and isolate live app recovery checkpoints.
- Extend atomic writes to exports, snippets, books, libraries, swatches and MCP/CLI images.
- Preserve fractional placed graphic sizes, same-style paragraph spacing and single-page masters.
- Add optical margin alignment, interactive command search, Presentation shortcuts,
  Option/Alt wheel zoom and corrected centered resize gestures and selection arrows.
- Add expanded font picker, theme favorites, Document Setup presets and explicit mixed-frame edits.
- Add styled Markdown placement, custom IDML wrap paths, ICC registration foundations,
  vector PDF content bounds, RTL extraction fixes and case-insensitive CLI output formats.
- Improve tool cursors, incremental text repaint bounds and document-unit guide/margin dialogs.
- Preserve current owner sessions and inline Indic input while processing canvas events in order.
- Complete the twenty UI-first proposals through #491; record broader-queue and distribution limits. [Change and source notes](docs/pagenor/UPSTREAM_ADOPTION_20261012.md).
- Add native Windows/Linux milestone release builds, MSI installation and file
  association checks, Linux/Flatpak smoke tests and portable HTTPS/path fixtures.
  Fix WiX duplicate legacy-extension registry entries while retaining document icons.

## 0.5.2 — local development, 2026-10-11

- Add one/two-column switching to the right panel toolbar, with pinned chevrons
  and a Window menu command. Remember the independent choice in preferences and
  custom workspaces; preserve grouping, scrolling and docking.
  [Right toolbar notes](docs/pagenor/RIGHT_TOOLBAR_COLUMNS.md).

- Expose Master Pages with live thumbnails, direct left/right master editing,
  shared header/footer and current-page-number frames, and page settings dialogs.
- Inherit master guides for drawing/snapping and shared margins/columns by page
  side, including book binding and repagination. Preserve explicit local settings.
- Persist the selected default master for document growth and correct guide/dialog
  targeting while editing masters. Retain native document and master item overrides.
- Preserve the document canvas view when switching to/from masters. Keep the owner’s
  current app session open; local build and validation only.
  [Usage and change notes](docs/pagenor/MASTER_PAGES.md).

## 0.5.1 — local editor usability, 2026-10-11

- Keep the workspace open when the document window's close button is used; close
  the active document with the existing unsaved-changes question and return to
  Welcome after the last document. Explicit Quit retains save-all and shutdown.
- Widen the right-hand panel icon bar to 44 pixels with 32-pixel controls.
- Widen both canvas scrollbars to 16 logical pixels and increase thumb contrast.
- Select table cells with the Arrow tool; drag or Shift-click to extend the range.
  Cmd/Ctrl-click selects the enclosing frame; double-click enters text editing.
- Expose horizontal/vertical cell borders, weight, swatch colour, tint, stroke type,
  gap colour and cell fill in the Table panel, using existing undoable commands.
- Group panel menus and the right-hand icon rail by function. Newly opened panels
  stack vertically; long stacks scroll. Add visible close controls, menu toggles,
  and Arrange Vertically for existing custom docks. The AI Assistant shares the
  vertical dock; existing floating-window preferences remain.
- Raise the bottom status bar from 17 to 28 pixels with larger page navigation
  and error text. Preserve `.pgnr`, branding, inline input and safe Rust layering.
- This is a local development build; native Windows/Linux runtime checks and
  GitHub publication remain milestone work.

## 0.5.0 — local development, 2026-10-11

- Complete the selected 47 non-CJK upstream changes plus two prerequisite changes,
  with pinned source credits and per-change adaptation notes.
- Add shared panel docking, System appearance, canvas scrollbars/context menus,
  layer/object reordering, live frame corners and direct graphic movement.
- Expand paragraph style/local formatting: tabs, borders/shading, list expressions,
  split columns, table stroke gaps/dashes, inherited effects and text-wrap contours.
- Improve grouped/content wrapping, small-cap PDF metadata, RTL extraction, optical
  kerning, No Break justification, terminal paragraph leading and RTL parent pages.
- Deduplicate same-source assets while retaining independent relink paths; compact
  native saves and retain all placed asset uses in links/package output.
- Improve graphics startup/fallback, oversized texture handling, native printing,
  IDML package/font/crop metadata, packaging coverage and Russian captions.
- Preserve `.pgnr` and legacy document compatibility, native format 3, inline Telugu
  input, scoped/lazy system fonts, Welcome/recent documents and distinct owner icons.
- Keep new CJK work and Italian deferred. This local checkpoint is not a published
  or signed/notarized cross-platform release; native Windows/Linux checks remain.

See [per-change notes](docs/pagenor/UPSTREAM_EDITOR_ADOPTION.md).

## 0.4.0 — local development, 2026-10-11

- Add Welcome recent-document thumbnails, File → Open Recent, contextual editor
  tools, About placement and save prompts for every open document on quit.
- Improve custom keyboard/IME text entry on canvas, including inline Telugu
  composition, quick mouse input and manual text threading across pages.
- Add shared font-size presets and separate document icons across the packaging
  targets, preserving the owner’s app icon and `.pgnr`/legacy file compatibility.
- Add New Document preset cards/settings; chapter and numbering-section options;
  Tabs panel and paragraph-style tab controls; bounded page numbering;
  swatch/Glyphs improvements; draggable ruler origin; word/line multi-click selection.
- Preserve native-save metadata and staged-save reliability improvements.
- Adapt the upstream v0.5.0 installer contribution (#131) with Pagenor artwork,
  HiDPI background, reproducible Finder layout and versioned package filenames.
- Audit all 71 upstream v0.6.0 changes and adapt SVG installed/document font
  resolution with scope-aware screen/PDF caches; mixed-gradient export; built-in
  PDF/X-4 CLI checks; missing-image warnings; facing-page merge regressions;
  installed CLI MCP guidance; and tar-only Linux RISC-V cross packaging.
- Keep document format 3, shared Rust architecture and `unsafe_code = "forbid"`.
  Native Windows/Linux matrix and signed/notarized distribution remain milestone
  validation; 0.4.0 is not published by this local update.

## 0.3.0 — 2026-10-10

- Record the accumulated Pagenor work since 0.2.1: lazy native/system font refresh,
  scoped/variable-font resolution, clipboard and recovery reliability, `.pgnr`,
  supplied owner branding/splash, editor/style/PDF controls, advanced data merge,
  typography/threading, INDD subset import, EPS/PowerPoint and reviewed AI edits.
- Retain and audit all 32 inherited application/workflow contributions listed in
  the upstream v0.4.0 release. Adapt distribution/documentation #132/#135
  under the same 0.3.0 version, as requested by the owner.
- Add FreeBSD tar packaging, Flatpak bundles for x86_64/aarch64 and AppImage
  zsync update routing to Pagenor’s repository, plus download/build documentation.
  Package notices include the owner branding rights and bundled-font licences.
  Native package execution and publication remain milestone checks.
- Keep the Rust architecture, unsafe-code prohibition and native document format 3.

See [release audit](docs/pagenor/UPSTREAM_040_AUDIT.md),
[phase evidence](PHASE_STATUS.md) and [version policy](docs/pagenor/VERSIONING.md).
