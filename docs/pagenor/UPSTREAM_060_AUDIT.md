# Pagenor upstream v0.6.0 audit and first adaptations

Historical local audit at **0.4.0**, 2026-10-11. Subsequent P11 work at 0.5.0 is recorded in [the editor adoption notes](UPSTREAM_EDITOR_ADOPTION.md). Upstream release head:
`a9124a0517594ce9872f73637e7b841a4967bcf9`.

All 71 distinct release-note changes were inspected against their individual merge-commit
patches and the current fork. Contributor acknowledgements repeat 16 PR links and are not
additional changes. This audit identifies remaining work; **the v0.6.0 adoption is not complete**.
Forty-five changes are already covered, seven are adapted in this pass, sixteen remain pending,
and three are owner-deferred or outside the fork's scope. A matching function name alone is
not an integration verdict: source changes, existing fork adaptations and validation are used.

Pagenor keeps its branding, owner artwork, separate document icons, `.pgnr` native saves,
legacy imports, safe-save behavior, canvas IME, lazy native font catalogue, document scopes,
variable-font/named-instance text-frame support and unsafe-code prohibition. These adaptations
retain the Rust workspace layers and are local only; no GitHub release or push is performed.

## This pass

- SVG labels resolve installed, memory/native-service and document fonts through the existing
  FontDb/ScopedFonts APIs. Each selected face holds its existing bytes in an Arc; menu discovery
  is unchanged. Aliases and case use Pagenor's resolver. Unselected collection faces cannot
  displace document replacements. Screen image-cache entries include font scope/generation;
  closing a scope invalidates its former SVG glyphs. PDF vector export uses the same scope.
- SVG variation settings remain usvg CSS settings on the original variable font. Pagenor
  named-instance coordinates are not injected into SVG CSS. Existing text-frame named instances
  are preserved; this pass does not claim a new SVG named-instance selector.
- Mixed RGB/CMYK/gray gradient output spaces are normalized to RGB with an export warning,
  including mixed spaces introduced by midpoint expansion. Homogeneous device spaces remain.
- `pagenor-cli validate-pdf FILE.pdf` exposes the existing exporter-owned PDF/X-4 checks as JSON;
  invalid/missing arguments and failed checks exit nonzero. Reads are capped at 256 MiB. These
  checks are not a full independent PDF/X certification.
- Missing linked images are reported as missing, with their link; corrupt images retain their
  decoding warning. Facing-page data-merge regressions preserve Pagenor's single-record defaults
  and existing refusal of unsupported facing-page multi-record tiling.
- Installed CLI MCP instructions use Pagenor executable names. PR #349 is covered by the
  stronger existing Unicode/structured clipboard bridge; native menu routes now join the
  existing once-only/reversible Copy/Cut regression. No duplicate clipboard pipeline is added.
- Linux riscv64 gets a tar-only cross-build job on Ubuntu 24.04 (glibc 2.39 baseline), explicit
  Rust/C/ring/pkg-config target routing and a QEMU CLI smoke-test hook. The script rejects
  inconsistent architecture/target labels and unsupported formats before staging. Foreign
  binaries are never executed directly on the host. Mac/Windows/existing Linux formats remain.

## Per-item decisions

| PR | Release change | Decision |
|---|---|---|
| [#136](https://github.com/storytold/designcraft/pull/136) | Images: PSD decode passes clippy on Rust 1.98 | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#115](https://github.com/storytold/designcraft/pull/115) | Compose: don't panic when a hyphenated paragraph starts with a no-break letter | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#80](https://github.com/storytold/designcraft/pull/80) | Show an alert when a file can't be opened or placed | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#111](https://github.com/storytold/designcraft/pull/111) | UI: text fields take Select All, Copy, Cut, Paste, Undo and Redo from the Mac menu (#30) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#112](https://github.com/storytold/designcraft/pull/112) | UI: toolbars that don't fit the window switch to two columns or scroll (#57) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#116](https://github.com/storytold/designcraft/pull/116) | Images: set SVG text in the shared fonts when the bundled ones lack it | Adapted here: Scoped SVG font resolution and zero-copy face data; document isolation, cache refresh, collection selection and screen/PDF regressions. |
| [#117](https://github.com/storytold/designcraft/pull/117) | Fix browser printing, startup flags, and document MIME | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#118](https://github.com/storytold/designcraft/pull/118) | Honor MCP typing, story, and page contracts | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#120](https://github.com/storytold/designcraft/pull/120) | Preserve typed values in generic command dialogs | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#127](https://github.com/storytold/designcraft/pull/127) | CLI: export options after the last export are an error; MCP and README docs match the code | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#128](https://github.com/storytold/designcraft/pull/128) | PDF/A keeps placed images; right-to-left tables and columns | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#137](https://github.com/storytold/designcraft/pull/137) | Commands: huge row and column counts can't exhaust memory | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#138](https://github.com/storytold/designcraft/pull/138) | Web: AltGr and other named keys no longer type their names (#134) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#139](https://github.com/storytold/designcraft/pull/139) | Docs, About: the Illustrator-class sibling is VectorCraft (#59) | Deferred / not applicable: Upstream sibling branding is outside Pagenor product scope. |
| [#145](https://github.com/storytold/designcraft/pull/145) | UI: ruler units by right-click, Language in Properties | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#149](https://github.com/storytold/designcraft/pull/149) | Import: RTF and Tagged Text keep characters past U+FFFF | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#152](https://github.com/storytold/designcraft/pull/152) | Apply contextual escaping to HTML and EPUB exports | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#257](https://github.com/storytold/designcraft/pull/257) | Word import: keep tracked moved text (w:moveTo) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#266](https://github.com/storytold/designcraft/pull/266) | Autoflow: add only the pages breaks ask for | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#275](https://github.com/storytold/designcraft/pull/275) | fix(engine): object.opacity accepts its documented lowercase blend modes | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#287](https://github.com/storytold/designcraft/pull/287) | Fonts: the Document Fonts folder is found whatever its case | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#239](https://github.com/storytold/designcraft/pull/239) | UI: Quick Apply opens without running an entry; rows are left-aligned | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#264](https://github.com/storytold/designcraft/pull/264) | UI: the shortcut recorder waits for the key; Interactive for PDF and Digital Publishing open their panels | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#142](https://github.com/storytold/designcraft/pull/142) | UI: the Control panel shows its second row | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#147](https://github.com/storytold/designcraft/pull/147) | PDF: a damaged PNG, GIF or WebP is skipped, not fatal | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#254](https://github.com/storytold/designcraft/pull/254) | fix(cli): --help prints usage to stdout and exits 0 | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#122](https://github.com/storytold/designcraft/pull/122) | Desktop: write log records to standard error and a rotating log file | Already covered: Pagenor logger and its renamed filter/control-port tests preserve opt-in local automation. |
| [#224](https://github.com/storytold/designcraft/pull/224) | Distribute selected table rows evenly and keep the edit undoable | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#229](https://github.com/storytold/designcraft/pull/229) | Prevent mixed gradient color spaces from aborting PDF export | Adapted here: Mixed-output-space gradients normalize to RGB with a deduplicated warning; plain/PDF-X/PDF-A/booklet matrix. |
| [#230](https://github.com/storytold/designcraft/pull/230) | Preserve text decoration color resets, geometry and paint order | Pending: Text-decoration resets, centered geometry and paint order. |
| [#244](https://github.com/storytold/designcraft/pull/244) | Rebind embedded image links when opening relocated packages (#235) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#247](https://github.com/storytold/designcraft/pull/247) | IDML: layers import and export in stacking order; font names with a format suffix | Already covered: Pagenor scoped alias-aware family resolver replaces upstream global exact-family helpers; stacking-order round trips already tested. |
| [#269](https://github.com/storytold/designcraft/pull/269) | docs: register the MCP server from an installed release | Adapted here: Installed CLI MCP instructions use Pagenor executable and retain opt-in authenticated local control. |
| [#249](https://github.com/storytold/designcraft/pull/249) | fix(ui): Make menu entries selectable across the full row | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#200](https://github.com/storytold/designcraft/pull/200) | Add PDF/X-4 validation command | Adapted here: Expose existing PDF/X-4 checks through a strict, bounded CLI command with JSON and exit status. |
| [#276](https://github.com/storytold/designcraft/pull/276) | Text: text after a paragraph that spans columns flows below it | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#283](https://github.com/storytold/designcraft/pull/283) | Text: the line before a column, frame or page break is a last line | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#268](https://github.com/storytold/designcraft/pull/268) | chore: bump github actions to fix Node 20 deprecation warnings | Pending: Actions runtime upgrades across the existing packaging/release workflows. |
| [#236](https://github.com/storytold/designcraft/pull/236) | Add Ukrainian interface translation | Pending: Ukrainian locale registration and translation. |
| [#272](https://github.com/storytold/designcraft/pull/272) | i18n: translate the whole interface into Japanese | Pending: Complete Japanese locale table and its validation. |
| [#143](https://github.com/storytold/designcraft/pull/143) | UI: one searchable font menu everywhere; the Contextual Task Bar can be moved | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#119](https://github.com/storytold/designcraft/pull/119) | Preserve browser import intent and document targets | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#294](https://github.com/storytold/designcraft/pull/294) | i18n: a new English string doesn't fail the Ukrainian catalog test | Pending: Ukrainian partial-catalog policy depends on #236. |
| [#144](https://github.com/storytold/designcraft/pull/144) | Styles: Character Style Options dialog | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#199](https://github.com/storytold/designcraft/pull/199) | Swatch menus: make the whole row (chip included) clickable and show chips in the Contextual Task Bar | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#248](https://github.com/storytold/designcraft/pull/248) | fix(ui): Add A6 sheet preset to the welcome page | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#222](https://github.com/storytold/designcraft/pull/222) | Hyphenation: Spanish text is hyphenated with Javier Bezos's patterns | Pending: Spanish hyphenation dependency, patterns and attribution. |
| [#223](https://github.com/storytold/designcraft/pull/223) | PDF/X-4: RGB images carry sRGB, no image interpolation, pdf:Trapped in XMP | Pending: PDF/X-4 RGB image profiles, interpolation and XMP trapping. |
| [#263](https://github.com/storytold/designcraft/pull/263) | Paragraph Rules: a dialog for the selected paragraphs and a section in Paragraph Style Options | Already covered: Pagenor sparse Paragraph Rules edits and style dialog tests cover the behavior. |
| [#296](https://github.com/storytold/designcraft/pull/296) | i18n: one Tint: row per catalog | Already covered: Exactly one Tint row in each configured Arabic/Portuguese catalog; Italian excluded, unconfigured catalogs remain pending separately. |
| [#274](https://github.com/storytold/designcraft/pull/274) | Text frames: column rules on screen, in PDF and in IDML; Column Rules in Text Frame Options | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#282](https://github.com/storytold/designcraft/pull/282) | Text: a line with a tab wraps at the column edge | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#146](https://github.com/storytold/designcraft/pull/146) | UI: closing a document with unsaved changes asks first (#141) | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#284](https://github.com/storytold/designcraft/pull/284) | Text: a justified line with a tab justifies the text after its last tab | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#218](https://github.com/storytold/designcraft/pull/218) | I18n italian | Deferred / not applicable: Italian translation deferred explicitly by the owner. |
| [#258](https://github.com/storytold/designcraft/pull/258) | test(datamerge): pin facing-page record placement (#198 no longer reproduces) | Adapted here: Facing-page merge regression fixtures, preserving Pagenor single-record defaults and explicit tiling restrictions. |
| [#285](https://github.com/storytold/designcraft/pull/285) | Text: a column, frame or page break ends its paragraph; odd and even page breaks | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#153](https://github.com/storytold/designcraft/pull/153) | Harden temporary files and release inputs | Pending: Audit and adapt the remaining print/recovery temporary-file hardening against Pagenor native-save protections. |
| [#280](https://github.com/storytold/designcraft/pull/280) | Set a text selection as warichu inside the parent line | Pending: Warichu layout, hit testing and selection geometry. |
| [#288](https://github.com/storytold/designcraft/pull/288) | IDML: resolve table styles and correct cell height fitting | Already covered: Existing implementation/earlier Pagenor adaptation, with corresponding source and tests inspected. |
| [#281](https://github.com/storytold/designcraft/pull/281) | Keep menus and dialogs usable at larger UI scales | Already covered: Pagenor scrollable dialogs and its separate new-document UI have existing larger-scale/RTL tests. |
| [#300](https://github.com/storytold/designcraft/pull/300) | PDF: reuse image conversions and index page objects (from #176) | Pending: Raster conversion reuse and indexed PDF page traversal. |
| [#299](https://github.com/storytold/designcraft/pull/299) | PDF: a placed image whose file is missing is reported as missing | Adapted here: Empty linked image data reports missing and includes the original link; corrupt data still reports decode failure. |
| [#298](https://github.com/storytold/designcraft/pull/298) | Lists: numbering formats and expressions from IDML; an empty paragraph gets no bullet or number | Pending: List formatting/metacharacters and empty-paragraph suppression. |
| [#297](https://github.com/storytold/designcraft/pull/297) | Desktop: pick the graphics backend before the window exists; DirectX 12 on Windows | Pending: Graphics-backend selection and persistent startup fallback. |
| [#293](https://github.com/storytold/designcraft/pull/293) | Text: a word longer than the line breaks at the column edge; a line that exactly fills the measure fits | Pending: Emergency breaks for overlong words and exact-fit tolerance. |
| [#196](https://github.com/storytold/designcraft/pull/196) | PDF export: options dialog with remembered settings | Pending: Complete PDF options UI with remembered settings. |
| [#295](https://github.com/storytold/designcraft/pull/295) | Text: Split Columns sets a paragraph in sub-columns within its column | Pending: Balanced split-column paragraphs. |
| [#155](https://github.com/storytold/designcraft/pull/155) | Validate engine work counts and package filenames | Pending: Remaining guide/step-repeat work limits; package/import limits already have Pagenor protections. |
| [#355](https://github.com/storytold/designcraft/pull/355) | Progress docs: full re-measure against InDesign 2026 21.6 (craftrules standard) | Deferred / not applicable: Upstream parity/readiness claims cannot be inherited as fork validation. |
| [#398](https://github.com/storytold/designcraft/pull/398) | Release prep: version 0.6.0; Linux riscv64 tar.gz in releases | Adapted here: Linux riscv64 tar-only cross-build/release routing, target C/ring tooling, QEMU CLI hook and packaging contracts. |

## Evidence and boundaries

Pinned source blobs for #116 and #398 are recorded in `PAGENOR_UPSTREAM.md`; the full audit
records every individual merge commit. Code adaptations preserve the upstream MIT/Apache
licences. No upstream product artwork, contributor-registry edits or parity percentages are
copied. Existing owner brand notices remain authoritative.

Tests cover named/shared memory and installed fonts, Hangul fallback, exact collection index,
zero-copy/deduplication, two competing document scopes, cached vs cold screen rendering,
memory-font changes, scope closure and the same vector SVG in PDF. The inspected synthetic
screen/PDF proofs use the same SVG character rendered through different document glyphs.
Packaging tests use synthetic binaries/tool shims; they are not native RISC-V execution.
Native Windows/Linux/RISC-V GUI execution and genuine cross artifacts remain milestone checks.

All seven local verification gates pass: **1,548 tests, 3 existing ignores, 20 WASM
packages**, plus **12 packaging contracts**. ShellCheck and actionlint pass.
The macOS 0.4.0 development bundle is rebuilt and installed locally; strict/deep
ad-hoc signature checks and separate app/document icon/association checks pass.
This is not a signed/notarized release. No version bump or GitHub publication.

## P11 follow-up

The 0.5.0 local checkpoint implements #295 (split columns), #297 (graphics startup/fallback) and #298 (list expressions/empty-paragraph suppression), previously pending in the audit table above. Those rows are historical. Thirteen other pending audit changes remain outside the selected P11 queue; this does not claim complete v0.6.0 adoption.
