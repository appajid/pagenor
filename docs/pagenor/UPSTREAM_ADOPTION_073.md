# P11.17 — local Pagenor 0.7.3 upstream adaptations

Review date: 15 October 2026. Base: `726ddd4994f4927b38882a47ccbdd6f358737bc8` (0.7.2).
Read open/closed PR metadata from 300 latest-created and 100 latest-updated records.
The new-proposal inventory extends through #1054 (202 proposals after #748).
Fetched 41 candidate diffs; adopted 26 compatible corrections after inspecting
Pagenor's existing architecture and tests. All 26 selected source heads were
rechecked after adaptation and remained unchanged. #1054 appeared during this
review and was included after source inspection. The final head-only check saw
later #1056 (paper while canvas rendering is pending); it is queued for the next
source review, outside this batch’s #1054 inventory cutoff.

This is local work on `development/upstream-20261012`; no remote write or publication.
Pagenor branding, owner-supplied assets, .pgnr/legacy loading, unsafe_code=forbid,
Rust layering and WASM remain. Private owner documents/fonts are not fixtures or
release assets. Owner app sessions are left open. Source code remains MIT OR
Apache-2.0; upstream legal provenance is retained. Pagenor owner: Appaji Ambarisha.

## Adapted source pins

| PR | Source author | Inspected source head | Change |
|---|---|---|---|
| [#1045](https://github.com/storytold/designcraft/pull/1045) | @moppeeer | `32146348b8894201fd3fdabee8d8cd8db72705e2` | UI: Tab, Shift+Tab and Presentation hide the flyout and floating panels too |
| [#1043](https://github.com/storytold/designcraft/pull/1043) | @moppeeer | `e001b4122101043109b1ab24811a27b7a18ec19b` | Layout: First/Previous/Next/Last Page and Next/Previous Spread, with InDesign's shortcuts |
| [#1042](https://github.com/storytold/designcraft/pull/1042) | @Silvertree2010 | `61d625bd903e2c1dda2923f1193cf62e2292abc3` | Keep '-' after an RTF control word when no digit follows |
| [#1040](https://github.com/storytold/designcraft/pull/1040) | @sanyamsingh001 | `f9bd971fff33163b0f876911e3c08ec11846e5bd` | Reject threading that would create linked-story ancestor cycles |
| [#1038](https://github.com/storytold/designcraft/pull/1038) | @Silvertree2010 | `61d5f3a1ef986021a5eb3464906aee46d2b02f3f` | Fix \rightarrow closing a \left fence early in math expressions |
| [#1029](https://github.com/storytold/designcraft/pull/1029) | @midasmatterhorn | `937e948a3dab9048e79883bd389076fc78172ea8` | docx: an explicit off imports as off |
| [#1028](https://github.com/storytold/designcraft/pull/1028) | @midasmatterhorn | `ff3ae9f277d8ab069321453481fbfe0454ffe8e2` | format: refuse to save a document that can't be read back |
| [#1015](https://github.com/storytold/designcraft/pull/1015) | @Kerrrunchhh | `0a836edb7df62023d79d970ed2d4883352372e7e` | fix: IDML links in a subfolder beside the package load on another machine |
| [#1001](https://github.com/storytold/designcraft/pull/1001) | @wstock | `02bc1e38aa0d31d8c494e2bfa0c2329e2e4832e1` | Importers: XML element nesting and DTD group nesting are bounded (#712) |
| [#985](https://github.com/storytold/designcraft/pull/985) | @Silvertree2010 | `ec510b90695e66b9fc1edf25c38cf115726d7b24` | End the RTF \uN fallback skip at the end of its group |
| [#982](https://github.com/storytold/designcraft/pull/982) | @Silvertree2010 | `26b5578291b0229b6d0f960740c3f55a24b253e2` | Skip .ase swatches with NaN or infinite colour components |
| [#968](https://github.com/storytold/designcraft/pull/968) | @guillehr2 | `939d5676cb5ef970650f6aa1704611097fd8f202` | Color panel: the Tint field keeps typed text and applies it on Enter or leaving the field |
| [#967](https://github.com/storytold/designcraft/pull/967) | @midasmatterhorn | `e8dd2636f5e8c2d6f953bdd4949bc6ace6875631` | guard: restore the whole command transaction on a panic |
| [#966](https://github.com/storytold/designcraft/pull/966) | @midasmatterhorn | `a393ab2018e845e0303da60205b80904a8b16ecc` | script.run: roll a failed script back instead of stranding its edits |
| [#965](https://github.com/storytold/designcraft/pull/965) | @midasmatterhorn | `59bdb371ff82d15318132fa3ab234151a9061e43` | dtd: cap parameter-entity expansion instead of trusting a pass count |
| [#963](https://github.com/storytold/designcraft/pull/963) | @midasmatterhorn | `8cfaa90dcb036698ea54fa6c719b8d79c66b8fa5` | idml: read PDFAttribute PageNumber so placed PDFs show their page |
| [#954](https://github.com/storytold/designcraft/pull/954) | @midasmatterhorn | `e45bb864ed426199297ed693acc697edc152f9bb` | math: cap LaTeX expression nesting depth so hostile input can't overflow the stack |
| [#952](https://github.com/storytold/designcraft/pull/952) | @Silvertree2010 | `2302b7ec7f01d6301365383275b2001a2f7a3714` | Keep viewBox-only placed SVGs at viewBox size on IDML import |
| [#951](https://github.com/storytold/designcraft/pull/951) | @Silvertree2010 | `8b834d053383d6a3ac95f5a5ef240755f2d28877` | Fix overflow in layout.pages.insert when `after` is u64::MAX |
| [#933](https://github.com/storytold/designcraft/pull/933) | @moppeeer | `11b1566c099aff00974010a756bc8204c240a106` | Type tool: a press over a locked empty frame draws a new text frame instead of converting it |
| [#928](https://github.com/storytold/designcraft/pull/928) | @tydude001 | `a65a83b765ba9db4ed2919c64d89d45621f82f95` | packaging: RPM requires library sonames, so openSUSE can install it |
| [#914](https://github.com/storytold/designcraft/pull/914) | @marcoarnulfo | `b12e94c11d0e302adc20fa8f353e8d0f42fc7b6a` | Package: a missing link stays missing, with no empty file in Links/ |
| [#1054](https://github.com/storytold/designcraft/pull/1054) | @Silvertree2010 | `108427e216c14f5f3bfdf003d3589b89f40aabb0` | Keep text of inline content controls in DOCX import |
| [#863](https://github.com/storytold/designcraft/pull/863) | @noah1400 | `554019cdd582207835a2f895344483389a016cb0` | Tables: table.options limits header and footer rows |
| [#837](https://github.com/storytold/designcraft/pull/837) | @noah1400 | `ba4519c3821a5dfb15d27ff5819fd7a2c8527e84` | Preferences: Story Editor size stays finite and in range |
| [#971](https://github.com/storytold/designcraft/pull/971) | @Silvertree2010 | `8421c6c47601f08d41117ddf4a00d7f1ff8c3940` | Bound decompressed size and entry count of native and IDML ZIP imports |

## Pagenor-specific integration

- Tab/Shift+Tab and Presentation suppress drawing flyout/floating palettes, including the Pagenor AI palette, without removing their saved layout. The Welcome-screen and Story Editor behavior stay intact.
- Layout gains First/Previous/Next/Last Page and Previous/Next Spread, with shortcuts. Page position, zoom and the existing left/right master navigation remain; tests use platform-neutral command modifiers.
- The Color-panel tint field retains text while focused, then commits on Enter/focus loss and clamps to its range; its slider still applies on release.
- Locked empty frames remain unconverted when clicked by the Type tool; drawing a new text frame above one remains possible. Existing text editing and multi-click/Indic input paths remain.
- Panic recovery restores every document/history, active document and command journal, plus pending caret/UI requests, before checkpointing. The two synthetic regressions failed on 0.7.2 and are retained.
- A failed script restores the document set, active document, selection/history/redo and journal; successful scripts keep one undo step. This changes the prior partial-script behavior deliberately. External file writes performed by earlier steps are not rolled back.
- LaTeX parsing and DTD content-model groups have depth limits; parameter-entity expansion is size-checked before replacement allocation. IDML XML bounds Start and Empty elements consistently. Word's existing, stronger depth/node/event budgets remain.
- IDML now indexes names and decompresses only referenced XML parts. Archive preflight bounds the entry count (100,000) and central directory (64 MiB) before ZIP indexing; actual inflated data is capped at 512 MiB per part and 2 GiB total, independently of declared sizes. Read/CRC failures in referenced parts propagate; unused corrupt parts are not decompressed. Native-format guards were already stronger and remain.
- Explicit table header/footer counts are capped at 100 before growth/arithmetic, matching Insert Table. Story Editor preferences accept only finite sizes and clamp to 8–36 pt without replacing untouched values.
- Native save checks that its JSON can deserialize before writing an archive, retaining Pagenor's archive preflight and part/total limits. Non-finite coordinates that serialize as null are refused.
- RTF preserves a literal hyphen after a control word and ends Unicode fallback skipping at group boundaries. DOCX honors explicit off toggles and keeps inline content-control text, including exact Telugu and joiner characters.
- IDML package links can resolve safe trailing subfolders beside the package. PDF page/crop identities stay distinct; a linked file is read once and its bytes shared across those placements. Existing PDF crop/box metadata and rendering are retained.
- ViewBox-only SVG placements retain their viewBox dimensions; parsing requires exactly four finite values and bounds the number-field work.
- Missing image data stays missing when packaging; no empty replacement file makes it appear present. Existing output-directory/symlink safeguards remain; the synthetic Mac test canonicalizes its temporary directory.
- ASE leaves out non-finite color components; page insertion clamps a maximal after index safely; math right-prefixed commands do not terminate a left fence early.
- Linux RPM requirements use library sonames for the existing 64-bit targets; Windows and Mac packaging are preserved. Native RPM installation is not claimed on this Mac.

## Explicitly excluded EPS change

[#1030](https://github.com/storytold/designcraft/pull/1030) proposes reading DOS EPS
TIFF offsets at bytes 12/16 and using a 22-byte header. Adobe's published EPSF 3.0
specification §5.2, Table 1 places WMF fields there, TIFF fields at 20/24 and the
checksum at 28–29. Pagenor already uses that 30-byte layout and has a synthetic
TIFF-preview regression. The proposed change is not applied.
[Primary specification](https://printtechnologies.org/standards/files/epsf_spec_v3_0.pdf).

## New-proposal inventory

“Further evaluation” is a metadata triage disposition, not a claim of detailed
source audit or completed implementation. Larger dependent feature branches need
separate integration and UI validation; closed status alone is not an adoption criterion.

| PR | State | Proposal | Disposition |
|---|---|---|---|
| [#751](https://github.com/storytold/designcraft/pull/751) | closed | Text Wrap panel: four offsets with the link button, and Invert | Further evaluation; not adopted in this batch |
| [#753](https://github.com/storytold/designcraft/pull/753) | open | Text Frame Options laid out as InDesign's: columns type and width, inset link, Ignore Text Wrap, Baseline Options | Further evaluation; not adopted in this batch |
| [#754](https://github.com/storytold/designcraft/pull/754) | open | Inner Glow mode and technique, Basic Feather choke and corners; inner glow blend modes mix with the object (blocked by #643) | Further evaluation; not adopted in this batch |
| [#757](https://github.com/storytold/designcraft/pull/757) | closed | Ctrl/Cmd + the + key zooms the canvas, not the whole window | Further evaluation; not adopted in this batch |
| [#758](https://github.com/storytold/designcraft/pull/758) | open | Save: delegate atomic writes to craft-atomic | Further evaluation; not adopted in this batch |
| [#759](https://github.com/storytold/designcraft/pull/759) | closed | Dialogs: drag the title to move the window | Further evaluation; not adopted in this batch |
| [#760](https://github.com/storytold/designcraft/pull/760) | closed | Scripts: GREP replacements like `$1` stay literal | Further evaluation; not adopted in this batch |
| [#761](https://github.com/storytold/designcraft/pull/761) | open | Colour management on craft-cms | Further evaluation; not adopted in this batch |
| [#763](https://github.com/storytold/designcraft/pull/763) | open | New Swatch names a gradient "New Gradient Swatch"; gradient swatches from colour swatches | Further evaluation; not adopted in this batch |
| [#764](https://github.com/storytold/designcraft/pull/764) | closed | Swatches: dropping an .ase file on the window loads its swatches | Further evaluation; not adopted in this batch |
| [#765](https://github.com/storytold/designcraft/pull/765) | closed | Radial gradients stretch with the object's box (an ellipse), as InDesign draws them | Further evaluation; not adopted in this batch |
| [#779](https://github.com/storytold/designcraft/pull/779) | closed | EPUB/HTML: anchored objects are exported where they are anchored | Further evaluation; not adopted in this batch |
| [#780](https://github.com/storytold/designcraft/pull/780) | closed | EPUB/HTML: list paragraphs share one ul/ol, and pre lines one block | Further evaluation; not adopted in this batch |
| [#782](https://github.com/storytold/designcraft/pull/782) | closed | EPUB: the navigation lists the headings, with a title in the book's language | Further evaluation; not adopted in this batch |
| [#783](https://github.com/storytold/designcraft/pull/783) | closed | EPUB: real modified time, a stable unique identifier, and a cover image file | Further evaluation; not adopted in this batch |
| [#785](https://github.com/storytold/designcraft/pull/785) | closed | EPUB/HTML: no file name as alt text, readable image names in the package | Further evaluation; not adopted in this batch |
| [#787](https://github.com/storytold/designcraft/pull/787) | closed | EPUB/HTML: generic font fallback from the font's class, and optional font embedding | Further evaluation; not adopted in this batch |
| [#789](https://github.com/storytold/designcraft/pull/789) | closed | Styles: name the attributes in the params docs, and a wrong one in the command's error | Further evaluation; not adopted in this batch |
| [#791](https://github.com/storytold/designcraft/pull/791) | closed | EPUB: one content file per chapter, split before each h1 | Further evaluation; not adopted in this batch |
| [#792](https://github.com/storytold/designcraft/pull/792) | open | Saving: an error never leaves an empty file or a document marked saved | Further evaluation; not adopted in this batch |
| [#793](https://github.com/storytold/designcraft/pull/793) | closed | Recovery: an explicit recovery folder never overwrites another instance's checkpoints | Further evaluation; not adopted in this batch |
| [#794](https://github.com/storytold/designcraft/pull/794) | open | Story Editor stays with its document; Undo restores the selection from before the command | Further evaluation; not adopted in this batch |
| [#795](https://github.com/storytold/designcraft/pull/795) | closed | Path edits respect locked layers; Properties links open panels when docking is on | Further evaluation; not adopted in this batch |
| [#796](https://github.com/storytold/designcraft/pull/796) | closed | Text recomposes when the baseline grid, a wrap layer's visibility or section numbering changes | Further evaluation; not adopted in this batch |
| [#797](https://github.com/storytold/designcraft/pull/797) | closed | Word import: table cells skip deleted text and field codes; Normal style formatting is kept | Further evaluation; not adopted in this batch |
| [#798](https://github.com/storytold/designcraft/pull/798) | closed | EPUB export includes tables, footnotes and endnotes | Further evaluation; not adopted in this batch |
| [#799](https://github.com/storytold/designcraft/pull/799) | closed | IDML export keeps overridden parent items | Further evaluation; not adopted in this batch |
| [#800](https://github.com/storytold/designcraft/pull/800) | open | PDF export leaves out form fields, buttons and media that aren't shown | Further evaluation; not adopted in this batch |
| [#801](https://github.com/storytold/designcraft/pull/801) | closed | macOS: menu-bar commands run at once | Further evaluation; not adopted in this batch |
| [#802](https://github.com/storytold/designcraft/pull/802) | open | File › Open Recent, and recent files with previews on the start screen | Already covered — Recent documents/previews, File Open Recent and shutdown prompts are existing Pagenor work |
| [#803](https://github.com/storytold/designcraft/pull/803) | open | Refuse command params the description doesn't name (#776) | Further evaluation; not adopted in this batch |
| [#804](https://github.com/storytold/designcraft/pull/804) | closed | fix: show Hangul in the UI with the system's Korean font | Deferred — CJK scope excluded by owner |
| [#807](https://github.com/storytold/designcraft/pull/807) | open | Preferences: OK without vertical units keeps them | Further evaluation; not adopted in this batch |
| [#808](https://github.com/storytold/designcraft/pull/808) | open | Open .indd and .indt files through indd-utils | Further evaluation; not adopted in this batch |
| [#810](https://github.com/storytold/designcraft/pull/810) | closed | Images: reject EPS bounding boxes with nonfinite corners or size (#740) | Further evaluation; not adopted in this batch |
| [#811](https://github.com/storytold/designcraft/pull/811) | open | Compare the format version without narrowing it to u32 (#739) | Further evaluation; not adopted in this batch |
| [#812](https://github.com/storytold/designcraft/pull/812) | closed | Fix overset preflight page for frames inside translated groups (#738) | Further evaluation; not adopted in this batch |
| [#813](https://github.com/storytold/designcraft/pull/813) | closed | MCP: render_page and screenshot are not read-only because they can write to path (#733) | Further evaluation; not adopted in this batch |
| [#814](https://github.com/storytold/designcraft/pull/814) | closed | fix(render): cap the failed-image cache at 64 entries (#728) | Further evaluation; not adopted in this batch |
| [#815](https://github.com/storytold/designcraft/pull/815) | closed | fix(pdf): leave form fields and media out of PDF/A-2b exports (#743) | Further evaluation; not adopted in this batch |
| [#816](https://github.com/storytold/designcraft/pull/816) | closed | fix: eyedropper ignores inactive states of a multi-state object (#737) | Further evaluation; not adopted in this batch |
| [#817](https://github.com/storytold/designcraft/pull/817) | closed | fix(cli): reject --pdf-options that is not a JSON object instead of panicking (#731) | Further evaluation; not adopted in this batch |
| [#818](https://github.com/storytold/designcraft/pull/818) | open | fix: drop the background render when a display option changes (#730) | Further evaluation; not adopted in this batch |
| [#819](https://github.com/storytold/designcraft/pull/819) | closed | Keep a missing graphic missing when it is pasted into another document (#720) | Further evaluation; not adopted in this batch |
| [#820](https://github.com/storytold/designcraft/pull/820) | closed | fix(geom): decompose of a flattened transform reports 0 shear instead of NaN | Further evaluation; not adopted in this batch |
| [#823](https://github.com/storytold/designcraft/pull/823) | closed | Gradient strokes: on screen, in PDF, in IDML and the Gradient panel | Further evaluation; not adopted in this batch |
| [#824](https://github.com/storytold/designcraft/pull/824) | closed | #21: opening or dropping an InDesign file says how to bring it over | Further evaluation; not adopted in this batch |
| [#825](https://github.com/storytold/designcraft/pull/825) | closed | Gradient swatches on text, laid across the text frame as in InDesign | Further evaluation; not adopted in this batch |
| [#826](https://github.com/storytold/designcraft/pull/826) | open | Gradient panel in InDesign's layout; drop swatches from the Swatches panel onto its stops | Further evaluation; not adopted in this batch |
| [#827](https://github.com/storytold/designcraft/pull/827) | closed | Tools: the app starts with the Hand tool, listed first in the Tools panel | Further evaluation; not adopted in this batch |
| [#828](https://github.com/storytold/designcraft/pull/828) | closed | Compose: spaces at the end of a paragraph take no room on the line | Further evaluation; not adopted in this batch |
| [#829](https://github.com/storytold/designcraft/pull/829) | open | i18n: add Polish interface language | Deferred — translation-only change |
| [#830](https://github.com/storytold/designcraft/pull/830) | closed | Images: CMYK TIFFs with alpha decode for display | Further evaluation; not adopted in this batch |
| [#831](https://github.com/storytold/designcraft/pull/831) | closed | macOS: no AccessKit, so launch no longer crashes on Touch Bar Macs | Further evaluation; not adopted in this batch |
| [#832](https://github.com/storytold/designcraft/pull/832) | closed | Window menu in InDesign's order; Window › Workspace with Essentials Classic | Further evaluation; not adopted in this batch |
| [#833](https://github.com/storytold/designcraft/pull/833) | closed | IDML: anchored settings from object styles, whole paragraph rules, style names with trailing spaces, Western leading | Further evaluation; not adopted in this batch |
| [#834](https://github.com/storytold/designcraft/pull/834) | closed | Images: EPS shows through its proxy at display time; simple PostScript (QR codes) is drawn | Further evaluation; not adopted in this batch |
| [#835](https://github.com/storytold/designcraft/pull/835) | closed | Several updates to TextFrame, Toolbar Stroke/Fill, Panels and Typography panel | Further evaluation; not adopted in this batch |
| [#836](https://github.com/storytold/designcraft/pull/836) | closed | Gradients between process colours blend as inks (C 50 M 50 halfway), as InDesign shows them | Further evaluation; not adopted in this batch |
| [#837](https://github.com/storytold/designcraft/pull/837) | closed | Preferences: Story Editor size stays finite and in range | Adapted in local 0.7.3 |
| [#838](https://github.com/storytold/designcraft/pull/838) | closed | Swatches panel header like InDesign's (Fill/Stroke proxy, container/text, Tint); swatches go to the fill or stroke in front (blocked by #592) | Further evaluation; not adopted in this batch |
| [#839](https://github.com/storytold/designcraft/pull/839) | closed | INDD: graphics whose linked file is missing show InDesign's preview | Further evaluation; not adopted in this batch |
| [#840](https://github.com/storytold/designcraft/pull/840) | closed | Object › Fitting with InDesign's shortcuts (Fill Frame Proportionally Ctrl+Alt+Shift+C, …) | Further evaluation; not adopted in this batch |
| [#843](https://github.com/storytold/designcraft/pull/843) | closed | Placed PDFs: render with pdfcraft's patched hayro crates | Further evaluation; not adopted in this batch |
| [#845](https://github.com/storytold/designcraft/pull/845) | closed | Respect Find/Change scope and first-match limits | Further evaluation; not adopted in this batch |
| [#846](https://github.com/storytold/designcraft/pull/846) | closed | Keep merged-cell selection highlighting and formatting consistent | Further evaluation; not adopted in this batch |
| [#849](https://github.com/storytold/designcraft/pull/849) | closed | Canvas: baseline grid draws only the lines on the page | Further evaluation; not adopted in this batch |
| [#850](https://github.com/storytold/designcraft/pull/850) | closed | Layout: Table of Contents dialog, entry options and IDML TOC styles | Further evaluation; not adopted in this batch |
| [#853](https://github.com/storytold/designcraft/pull/853) | closed | Compose: cross-reference index no longer rebuilds itself | Further evaluation; not adopted in this batch |
| [#854](https://github.com/storytold/designcraft/pull/854) | closed | perf(assets): index duplicate candidates by content (#727) | Further evaluation; not adopted in this batch |
| [#855](https://github.com/storytold/designcraft/pull/855) | closed | fix: repaint canvas when entering and leaving parent editing | Further evaluation; not adopted in this batch |
| [#857](https://github.com/storytold/designcraft/pull/857) | closed | Keep endnote and editorial markers out of child-story paste | Further evaluation; not adopted in this batch |
| [#858](https://github.com/storytold/designcraft/pull/858) | closed | macOS DMG: hdiutil create -srcfolder, so the installed app passes codesign --strict | Further evaluation; not adopted in this batch |
| [#859](https://github.com/storytold/designcraft/pull/859) | closed | Control: bound click and drag input before queuing events | Further evaluation; not adopted in this batch |
| [#860](https://github.com/storytold/designcraft/pull/860) | closed | fix(compose): odd/even page starts keep their parity at the top of a frame | Further evaluation; not adopted in this batch |
| [#862](https://github.com/storytold/designcraft/pull/862) | closed | Page tool: click selects a page; its handles preview the liquid rule, Alt-drag resizes, Shift keeps proportions | Further evaluation; not adopted in this batch |
| [#863](https://github.com/storytold/designcraft/pull/863) | closed | Tables: table.options limits header and footer rows | Adapted in local 0.7.3 |
| [#864](https://github.com/storytold/designcraft/pull/864) | closed | Control panel for the Page tool (W/H, preset, orientation, Liquid Page Rule); Controlled by Parent; Show Parent Page Overlay (blocked by #862) | Further evaluation; not adopted in this batch |
| [#865](https://github.com/storytold/designcraft/pull/865) | open | Unify DesignCraft dark chrome with ArtCraft palette | Excluded — upstream family branding/palette is not Pagenor branding |
| [#866](https://github.com/storytold/designcraft/pull/866) | closed | Page tool: liquid pins and resize locks on the canvas for object-based pages (blocked by #862, #864) | Further evaluation; not adopted in this batch |
| [#867](https://github.com/storytold/designcraft/pull/867) | closed | Liquid Layout panel in InDesign's layout (Auto-Fit, Object Constraints); Create Alternate Layout in the Layout menu (blocked by #864) | Further evaluation; not adopted in this batch |
| [#868](https://github.com/storytold/designcraft/pull/868) | closed | Page tool draws liquid guides from the rulers (dashed), as in InDesign | Further evaluation; not adopted in this batch |
| [#869](https://github.com/storytold/designcraft/pull/869) | closed | Pages panel: Alt-drag copies a page, drop a page on Create new page, Edit page size; duplicates keep their size | Further evaluation; not adopted in this batch |
| [#870](https://github.com/storytold/designcraft/pull/870) | closed | Pages panel menu in InDesign's order (blocked by #869) | Further evaluation; not adopted in this batch |
| [#875](https://github.com/storytold/designcraft/pull/875) | closed | refactor(pdf): use guarded shared writing | Further evaluation; not adopted in this batch |
| [#876](https://github.com/storytold/designcraft/pull/876) | open | Use shared PDF reading and report incomplete layered exports | Further evaluation; not adopted in this batch |
| [#878](https://github.com/storytold/designcraft/pull/878) | open | Open and save layouts in cloud storage | Deferred — external hub/cloud/AI integration and dependency review required |
| [#881](https://github.com/storytold/designcraft/pull/881) | closed | Typography panel | Further evaluation; not adopted in this batch |
| [#885](https://github.com/storytold/designcraft/pull/885) | closed |  Character panel: live variable font sliders, aligned rows, axis-only style names | Further evaluation; not adopted in this batch |
| [#887](https://github.com/storytold/designcraft/pull/887) | closed | Type tool: dead keys and input-method composition on the canvas | Do not replace Pagenor inline Indic/IME input; further delta evaluation |
| [#888](https://github.com/storytold/designcraft/pull/888) | open | Support dragging across checkbox controls | Further evaluation; not adopted in this batch |
| [#889](https://github.com/storytold/designcraft/pull/889) | closed | Facing pages take their side from their section's page numbers (odd on the right), as InDesign shuffles them | Further evaluation; not adopted in this batch |
| [#892](https://github.com/storytold/designcraft/pull/892) | open | Pages panel: spread labels ("2–3", "2, 1") and section-start triangles as InDesign's; it follows the page in view | Further evaluation; not adopted in this batch |
| [#893](https://github.com/storytold/designcraft/pull/893) | open | Numbering & Section Options opens as New Section on a page without one, as InDesign's (blocked by #342) | Further evaluation; not adopted in this batch |
| [#894](https://github.com/storytold/designcraft/pull/894) | closed | Type menu in InDesign's order; Character ⌘T, Paragraph ⌥⌘T, Current Page Number ⌥⇧⌘N | Further evaluation; not adopted in this batch |
| [#895](https://github.com/storytold/designcraft/pull/895) | closed | ui_shot: render every script line so long scripts don't time out on the GPU | Further evaluation; not adopted in this batch |
| [#901](https://github.com/storytold/designcraft/pull/901) | closed | Text: kerning takes a typed value, and the Character panel has it | Further evaluation; not adopted in this batch |
| [#902](https://github.com/storytold/designcraft/pull/902) | open | Direct Selection: pressing on a corner of an unselected object pulls just that corner, as in InDesign | Further evaluation; not adopted in this batch |
| [#903](https://github.com/storytold/designcraft/pull/903) | closed | Linux: build an Arch package (.pkg.tar.zst) | Further evaluation; not adopted in this batch |
| [#912](https://github.com/storytold/designcraft/pull/912) | closed | Render: wide effect blurs are drawn at a lower resolution | Further evaluation; not adopted in this batch |
| [#913](https://github.com/storytold/designcraft/pull/913) | closed | Color parsing: bare CMYK is always 0..100, so C 1 is a 1 % tint | Further evaluation; not adopted in this batch |
| [#914](https://github.com/storytold/designcraft/pull/914) | closed | Package: a missing link stays missing, with no empty file in Links/ | Adapted in local 0.7.3 |
| [#917](https://github.com/storytold/designcraft/pull/917) | closed | PDF: a placed PDF 1.7 with /Version /1.7 in its catalog goes into PDF/X-4 exports | Further evaluation; not adopted in this batch |
| [#918](https://github.com/storytold/designcraft/pull/918) | closed | Footnotes: Span Footnotes Across Columns sets notes across the frame | Further evaluation; not adopted in this batch |
| [#921](https://github.com/storytold/designcraft/pull/921) | closed | idml: cap IDML package part size on import instead of reading unbounded | Covered by the bounded lazy IDML reader adapted from #971 in this batch |
| [#922](https://github.com/storytold/designcraft/pull/922) | closed | fix Input Method Editor | Do not replace Pagenor inline Indic/IME input; further delta evaluation |
| [#925](https://github.com/storytold/designcraft/pull/925) | closed | Tools panel as InDesign's: Gradient Feather its own tool, two-column layout, View Options, Apply Color (,) and Apply Gradient (.) | Further evaluation; not adopted in this batch |
| [#926](https://github.com/storytold/designcraft/pull/926) | open | Free Transform tool: move, scale with contents, reflect, rotate and shear, as InDesign's (it was a placeholder) | Further evaluation; not adopted in this batch |
| [#928](https://github.com/storytold/designcraft/pull/928) | closed | packaging: RPM requires library sonames, so openSUSE can install it | Adapted in local 0.7.3 |
| [#929](https://github.com/storytold/designcraft/pull/929) | closed | Canvas & Tools panel: Selection is filled arrow, Direct Selection hollow arrow (fixes #437, #438) | Further evaluation; not adopted in this batch |
| [#930](https://github.com/storytold/designcraft/pull/930) | closed | Images: indexed-colour TIFFs decode through their palette | Already covered — Indexed TIFF decoding already supports palette images and PageMaker assets |
| [#933](https://github.com/storytold/designcraft/pull/933) | closed | Type tool: a press over a locked empty frame draws a new text frame instead of converting it | Adapted in local 0.7.3 |
| [#935](https://github.com/storytold/designcraft/pull/935) | closed | fix: find optical-size fonts named by their WWS family and style (#911) | Further evaluation; not adopted in this batch |
| [#936](https://github.com/storytold/designcraft/pull/936) | closed | Layers panel: the selection shows in it, images are listed by file name, double-click renames an object | Further evaluation; not adopted in this batch |
| [#937](https://github.com/storytold/designcraft/pull/937) | closed | Margins and Columns and Create Guides laid out as InDesign's: link button, Preview, arithmetic on leaving a field (blocked by #468) | Further evaluation; not adopted in this batch |
| [#939](https://github.com/storytold/designcraft/pull/939) | open | Rulers: a document zero point, a ruler origin, and guides from the ruler corner | Further evaluation; not adopted in this batch |
| [#940](https://github.com/storytold/designcraft/pull/940) | closed | Show 100% black as rich black on screen by default, as InDesign does | Further evaluation; not adopted in this batch |
| [#948](https://github.com/storytold/designcraft/pull/948) | closed | Keep XLSX formula text results as text on import | Already covered — Cached XLSX string handling already adapted in #673 / P11.16 |
| [#949](https://github.com/storytold/designcraft/pull/949) | closed | Type: let a blank line's hidden paragraph mark take leading immediately | Further evaluation; not adopted in this batch |
| [#951](https://github.com/storytold/designcraft/pull/951) | closed | Fix overflow in layout.pages.insert when `after` is u64::MAX | Adapted in local 0.7.3 |
| [#952](https://github.com/storytold/designcraft/pull/952) | closed | Keep viewBox-only placed SVGs at viewBox size on IDML import | Adapted in local 0.7.3 |
| [#954](https://github.com/storytold/designcraft/pull/954) | closed | math: cap LaTeX expression nesting depth so hostile input can't overflow the stack | Adapted in local 0.7.3 |
| [#957](https://github.com/storytold/designcraft/pull/957) | closed | Fix the AppStream screenshot URL and caption | Further evaluation; not adopted in this batch |
| [#958](https://github.com/storytold/designcraft/pull/958) | closed | Pathfinder keeps the frontmost object, as InDesign does (a photo intersected with shapes shows through them) | Further evaluation; not adopted in this batch |
| [#959](https://github.com/storytold/designcraft/pull/959) | closed | Fill with Placeholder Text in paragraphs, as InDesign does | Further evaluation; not adopted in this batch |
| [#960](https://github.com/storytold/designcraft/pull/960) | closed | Pathfinder panel laid out as InDesign's: Paths, Pathfinder, Convert Shape, Convert Point (with Join Path and Convert Point commands) | Further evaluation; not adopted in this batch |
| [#963](https://github.com/storytold/designcraft/pull/963) | closed | idml: read PDFAttribute PageNumber so placed PDFs show their page | Adapted in local 0.7.3 |
| [#964](https://github.com/storytold/designcraft/pull/964) | closed | layout.section: keep the settings the command does not name | Already covered — Pagenor preserves unspecified section properties before replacing the section |
| [#965](https://github.com/storytold/designcraft/pull/965) | closed | dtd: cap parameter-entity expansion instead of trusting a pass count | Adapted in local 0.7.3 |
| [#966](https://github.com/storytold/designcraft/pull/966) | closed | script.run: roll a failed script back instead of stranding its edits | Adapted in local 0.7.3 |
| [#967](https://github.com/storytold/designcraft/pull/967) | closed | guard: restore the whole command transaction on a panic | Adapted in local 0.7.3 |
| [#968](https://github.com/storytold/designcraft/pull/968) | closed | Color panel: the Tint field keeps typed text and applies it on Enter or leaving the field | Adapted in local 0.7.3 |
| [#969](https://github.com/storytold/designcraft/pull/969) | closed | Record one undo step for the four arrange shortcuts | Further evaluation; not adopted in this batch |
| [#970](https://github.com/storytold/designcraft/pull/970) | open | Report corrupt archive entries instead of treating them as missing | Further evaluation; not adopted in this batch |
| [#971](https://github.com/storytold/designcraft/pull/971) | closed | Bound decompressed size and entry count of native and IDML ZIP imports | IDML portion adapted; native limits retained |
| [#972](https://github.com/storytold/designcraft/pull/972) | closed | Preserve stored IDML page liquid rules | Further evaluation; not adopted in this batch |
| [#973](https://github.com/storytold/designcraft/pull/973) | closed | Flex Layout (InDesign 2026): containers that lay out their objects, the Flex Layout panel and Object › Flex Layout | Further evaluation; not adopted in this batch |
| [#974](https://github.com/storytold/designcraft/pull/974) | closed | IDML: import and export ruler guides | Further evaluation; not adopted in this batch |
| [#975](https://github.com/storytold/designcraft/pull/975) | closed | IDML: InDesign's default CJK leading model only with a Japanese composer (Western text spaced as InDesign) | Deferred — CJK scope excluded by owner |
| [#976](https://github.com/storytold/designcraft/pull/976) | closed | A line of text is set when its baseline fits in the frame, as in InDesign (descenders may hang below) | Further evaluation; not adopted in this batch |
| [#977](https://github.com/storytold/designcraft/pull/977) | closed | An empty last paragraph isn't overset text, as in InDesign | Further evaluation; not adopted in this batch |
| [#978](https://github.com/storytold/designcraft/pull/978) | closed | Find Font: a style that differs only in spacing isn't reported missing | Already covered — ScopedFonts.has_style already normalizes style names and validates actual aliases |
| [#979](https://github.com/storytold/designcraft/pull/979) | closed | i18n: Georgian (ka) interface language | Deferred — translation-only change |
| [#980](https://github.com/storytold/designcraft/pull/980) | open | Add designcraft-hub: the CraftHub client path (the v3 adapter, spec §3.6 phase C / §18.4 row 4) | Deferred — external hub/cloud/AI integration and dependency review required |
| [#982](https://github.com/storytold/designcraft/pull/982) | closed | Skip .ase swatches with NaN or infinite colour components | Adapted in local 0.7.3 |
| [#983](https://github.com/storytold/designcraft/pull/983) | open | Eyedropper: with text selected, pick up the clicked text's formatting, as InDesign does | Further evaluation; not adopted in this batch |
| [#984](https://github.com/storytold/designcraft/pull/984) | open | Add Craft AI: the family's built-in local assistant (Gemma 4, optional, shared, private) | Deferred — external hub/cloud/AI integration and dependency review required |
| [#985](https://github.com/storytold/designcraft/pull/985) | open | End the RTF \uN fallback skip at the end of its group | Adapted in local 0.7.3 |
| [#986](https://github.com/storytold/designcraft/pull/986) | open | Flatten deleted text variables in table cells and notes | Further evaluation; not adopted in this batch |
| [#987](https://github.com/storytold/designcraft/pull/987) | open | Fonts: register every explicitly supplied family for documents | Further evaluation; not adopted in this batch |
| [#988](https://github.com/storytold/designcraft/pull/988) | open | Swatches: Load Swatches numbers repeated .ase names and names unnamed colors by value | Further evaluation; not adopted in this batch |
| [#989](https://github.com/storytold/designcraft/pull/989) | open | Type menu: Define Lists path matches the menu | Further evaluation; not adopted in this batch |
| [#990](https://github.com/storytold/designcraft/pull/990) | open | Layers: Enter in the rename field keeps an unnamed object's automatic label | Further evaluation; not adopted in this batch |
| [#991](https://github.com/storytold/designcraft/pull/991) | open | Toolbar: Apply Gradient on the stroke is one undo step | Further evaluation; not adopted in this batch |
| [#992](https://github.com/storytold/designcraft/pull/992) | open | Sections: consecutive right pages get separate spreads | Further evaluation; not adopted in this batch |
| [#993](https://github.com/storytold/designcraft/pull/993) | open | Type: formatting set on a blank line applies to the text typed there | Further evaluation; not adopted in this batch |
| [#994](https://github.com/storytold/designcraft/pull/994) | open | Effects: wide feathers and inner effects stay sharp at high zoom | Further evaluation; not adopted in this batch |
| [#995](https://github.com/storytold/designcraft/pull/995) | open | Typography panel: every OpenType toggle, exported figure styles, working Standard Ligatures | Further evaluation; not adopted in this batch |
| [#996](https://github.com/storytold/designcraft/pull/996) | open | Paths: Join and Convert Point respect locked layers and groups | Further evaluation — Join/Convert commands need the larger Pathfinder prerequisite; existing Pagenor anchor guards retained |
| [#997](https://github.com/storytold/designcraft/pull/997) | open | Kerning: a value at the caret kerns the pair at the caret | Further evaluation; not adopted in this batch |
| [#998](https://github.com/storytold/designcraft/pull/998) | open | docs: add public font integration guidance to README | Further evaluation; not adopted in this batch |
| [#999](https://github.com/storytold/designcraft/pull/999) | open | EPUB/HTML: report anchored graphics without alt text too | Further evaluation; not adopted in this batch |
| [#1000](https://github.com/storytold/designcraft/pull/1000) | open | Type tool: a drag after several clicks extends the selection by words, lines or paragraphs (#981) | Further evaluation; not adopted in this batch |
| [#1001](https://github.com/storytold/designcraft/pull/1001) | open | Importers: XML element nesting and DTD group nesting are bounded (#712) | Adapted in local 0.7.3 |
| [#1002](https://github.com/storytold/designcraft/pull/1002) | open | Tools: Cmd held with another tool borrows the Selection tool for one gesture (#919) | Further evaluation; not adopted in this batch |
| [#1006](https://github.com/storytold/designcraft/pull/1006) | open | Type tool: mark composed text like a highlighter, in any script | Further evaluation; not adopted in this batch |
| [#1007](https://github.com/storytold/designcraft/pull/1007) | open | Scaling a text frame scales its text, as InDesign's When Scaling: Apply to Content | Further evaluation; not adopted in this batch |
| [#1009](https://github.com/storytold/designcraft/pull/1009) | open | Selection tool: with a frame's content selected, a drag on it moves the content (blocked by #415) | Further evaluation; not adopted in this batch |
| [#1010](https://github.com/storytold/designcraft/pull/1010) | open | IDML: preserve liquid guides on document and parent pages | Further evaluation; not adopted in this batch |
| [#1011](https://github.com/storytold/designcraft/pull/1011) | open | fix(text): Character panel: VF sliders preview live implementation | Further evaluation; not adopted in this batch |
| [#1012](https://github.com/storytold/designcraft/pull/1012) | open | Text: Digits setting in the Character panel and character styles; page numbers and list numbers follow it | Further evaluation; not adopted in this batch |
| [#1013](https://github.com/storytold/designcraft/pull/1013) | open | i18n(de): clearer terms, missing strings, six hard-coded dialog labels | Deferred — translation-only change |
| [#1014](https://github.com/storytold/designcraft/pull/1014) | open | IDML fidelity: fixes measured against a real Arabic and Dutch InDesign book | Further evaluation; not adopted in this batch |
| [#1015](https://github.com/storytold/designcraft/pull/1015) | open | fix: IDML links in a subfolder beside the package load on another machine | Adapted in local 0.7.3 |
| [#1017](https://github.com/storytold/designcraft/pull/1017) | open | feat(text): TextFrame: double-clicking handles fits to content | Further evaluation; not adopted in this batch |
| [#1018](https://github.com/storytold/designcraft/pull/1018) | open | i18n: Swedish (sv) interface translation | Deferred — translation-only change |
| [#1020](https://github.com/storytold/designcraft/pull/1020) | open | Support Fit Frame to Content for text frames (Cmd+Option+C) | Further evaluation; not adopted in this batch |
| [#1021](https://github.com/storytold/designcraft/pull/1021) | open | Panel tabs: InDesign's Close, Close Tab Group, Minimize and Collapse to Icons | Further evaluation; not adopted in this batch |
| [#1022](https://github.com/storytold/designcraft/pull/1022) | open | Window > Paragraph Styles opens the floating Character Styles / Paragraph Styles group, as InDesign does | Further evaluation; not adopted in this batch |
| [#1023](https://github.com/storytold/designcraft/pull/1023) | closed | Closed | Further evaluation; not adopted in this batch |
| [#1024](https://github.com/storytold/designcraft/pull/1024) | open | New Workspace: Capture Panel Locations and Menu Customization, as InDesign's | Further evaluation; not adopted in this batch |
| [#1026](https://github.com/storytold/designcraft/pull/1026) | open | Tables: Fit to Frame Width option, columns follow the text frame's width | Further evaluation; not adopted in this batch |
| [#1027](https://github.com/storytold/designcraft/pull/1027) | open | idml: one unusable CJK metric no longer refuses the whole file | Deferred — CJK scope excluded by owner |
| [#1028](https://github.com/storytold/designcraft/pull/1028) | open | format: refuse to save a document that can't be read back | Adapted in local 0.7.3 |
| [#1029](https://github.com/storytold/designcraft/pull/1029) | open | docx: an explicit off imports as off | Adapted in local 0.7.3 |
| [#1030](https://github.com/storytold/designcraft/pull/1030) | open | images: the DOS EPS TIFF preview is read where the spec puts it | Rejected as proposed — contradicts EPSF 3.0 header layout |
| [#1033](https://github.com/storytold/designcraft/pull/1033) | open | Direct Selection moves a placed graphic inside its frame | Further evaluation; not adopted in this batch |
| [#1034](https://github.com/storytold/designcraft/pull/1034) | open | Objects snap to the bleed guide | Further evaluation; not adopted in this batch |
| [#1035](https://github.com/storytold/designcraft/pull/1035) | open | The status bar's error count opens the Preflight panel | Already covered — Clickable Preflight error indicator is existing P11.8 behavior |
| [#1036](https://github.com/storytold/designcraft/pull/1036) | open | Overset marks for text frames inside groups (ports when selected; the red mark always) | Further evaluation; not adopted in this batch |
| [#1037](https://github.com/storytold/designcraft/pull/1037) | open | Export PDF: compression per image class, own bleed and slug, PSD fixes, colour conversion to CMYK, CMYK JPEG, installed destination profiles | Further evaluation; not adopted in this batch |
| [#1038](https://github.com/storytold/designcraft/pull/1038) | open | Fix \rightarrow closing a \left fence early in math expressions | Adapted in local 0.7.3 |
| [#1039](https://github.com/storytold/designcraft/pull/1039) | open | Compose: a balancing trial's shortened columns don't move a kept heading on | Further evaluation; not adopted in this batch |
| [#1040](https://github.com/storytold/designcraft/pull/1040) | open | Reject threading that would create linked-story ancestor cycles | Adapted in local 0.7.3 |
| [#1041](https://github.com/storytold/designcraft/pull/1041) | open | Anchored objects wrap their own story as their anchor lines are set (blocked by #563) | Further evaluation; not adopted in this batch |
| [#1042](https://github.com/storytold/designcraft/pull/1042) | open | Keep '-' after an RTF control word when no digit follows | Adapted in local 0.7.3 |
| [#1043](https://github.com/storytold/designcraft/pull/1043) | open | Layout: First/Previous/Next/Last Page and Next/Previous Spread, with InDesign's shortcuts | Adapted in local 0.7.3 |
| [#1044](https://github.com/storytold/designcraft/pull/1044) | open | View: Presentation shows one spread fitted to the screen; arrow keys and clicks go through the spreads | Further evaluation; not adopted in this batch |
| [#1045](https://github.com/storytold/designcraft/pull/1045) | open | UI: Tab, Shift+Tab and Presentation hide the flyout and floating panels too | Adapted in local 0.7.3 |
| [#1046](https://github.com/storytold/designcraft/pull/1046) | open | UI: double-clicking the status bar's preflight count shows the Preflight panel (builds on #1035) | Already covered — Existing single-click indicator already opens Preflight |
| [#1052](https://github.com/storytold/designcraft/pull/1052) | open | feat: EPS support through a clean-room PostScript interpreter (designcraft-postscript) | Further evaluation — substantial new PostScript interpreter, rendering/font/limit review required |
| [#1054](https://github.com/storytold/designcraft/pull/1054) | open | Keep text of inline content controls in DOCX import | Adapted in local 0.7.3 |

## Recently updated older proposals

These 30 older PRs also appeared in the latest-updated inventory. Head comparison
is not itself an incremental patch audit; changed heads often include rebases.
Pagenor's previously adopted code is not replaced wholesale. Unreviewed deltas
remain for later focused evaluation, especially the EOF, resize and font-picker work.

| PR | Title | Head comparison |
|---|---|---|
| [#400](https://github.com/storytold/designcraft/pull/400) | revision of PR #278: Adds ability to drag-drop files on to app to open them | not in prior snapshot; needs detailed delta review |
| [#460](https://github.com/storytold/designcraft/pull/460) | Preserve resize anchors and accurate dimension feedback | head changed since prior live review |
| [#542](https://github.com/storytold/designcraft/pull/542) | Preserve typing formats in trailing blank paragraphs | head changed since prior live review |
| [#467](https://github.com/storytold/designcraft/pull/467) | Preserve Text Frame Options targets and mixed values | head changed since prior live review |
| [#525](https://github.com/storytold/designcraft/pull/525) | Tint and Apply To in the Properties Fill and Stroke menus | head changed since prior live review |
| [#388](https://github.com/storytold/designcraft/pull/388) | fix(ui): Show the Pencil stroke while drawing | not in prior snapshot; needs detailed delta review |
| [#342](https://github.com/storytold/designcraft/pull/342) | feat(ui): Lay out Numbering & Section Options like InDesign's | not in prior snapshot; needs detailed delta review |
| [#466](https://github.com/storytold/designcraft/pull/466) | feat(ui): Lay out the font menu like InDesign's | head changed since prior live review |
| [#699](https://github.com/storytold/designcraft/pull/699) | Resize a selected placed graphic by its own handles (blocked by #415, #550) | head changed since prior live review |
| [#689](https://github.com/storytold/designcraft/pull/689) | Ungroup leaves frames with pasted-in content whole; name them by their shape | head changed since prior live review |
| [#748](https://github.com/storytold/designcraft/pull/748) | Effects panel: Object/Stroke/Fill/Text levels, fx badge, Clear All and Remove Effects (blocked by #643) | head changed since prior live review |
| [#643](https://github.com/storytold/designcraft/pull/643) | The Effects dialog: Object › Effects and the panel's fx button, with Preview | head changed since prior live review |
| [#618](https://github.com/storytold/designcraft/pull/618) | Paste vector art from Illustrator, VectorCraft or Figma as editable paths | head changed since prior live review |
| [#550](https://github.com/storytold/designcraft/pull/550) | Transform values act on a selected placed graphic, not its frame | head changed since prior live review |
| [#469](https://github.com/storytold/designcraft/pull/469) | feat(ui): Lay out Document Setup like InDesign's | head changed since prior live review |
| [#341](https://github.com/storytold/designcraft/pull/341) | feat(engine): Document chapter numbering and section prefix rules | not in prior snapshot; needs detailed delta review |
| [#325](https://github.com/storytold/designcraft/pull/325) | feat(ui): Lay out the New Document window like InDesign's | not in prior snapshot; needs detailed delta review |
| [#255](https://github.com/storytold/designcraft/pull/255) | fix(ui): Compact the Advanced object Control panel into aligned rows | not in prior snapshot; needs detailed delta review |
| [#253](https://github.com/storytold/designcraft/pull/253) | fix(ui): Integrate Windows window controls into the menu bar | not in prior snapshot; needs detailed delta review |
| [#580](https://github.com/storytold/designcraft/pull/580) | feat(desktop): portable mode, portable.txt next to the executable (#574) | head changed since prior live review |
| [#591](https://github.com/storytold/designcraft/pull/591) | Double-click a frame handle to fit the frame to its content; Fit Frame to Content fits text frames | head changed since prior live review |
| [#465](https://github.com/storytold/designcraft/pull/465) | Add Russian hyphenation with source-grapheme-safe wrapping | head changed since prior live review |
| [#303](https://github.com/storytold/designcraft/pull/303) | Distinguish unknown MCP commands from registered UI commands | not in prior snapshot; needs detailed delta review |
| [#468](https://github.com/storytold/designcraft/pull/468) | Layout: parent-page margins and guides, Create Guides on the current page, ruler-unit-aware Margins and Columns | unchanged head since prior live review |
| [#501](https://github.com/storytold/designcraft/pull/501) | Layout: parent pages pass margins and guides to their pages | head changed since prior live review |
| [#362](https://github.com/storytold/designcraft/pull/362) | IDML: `$ID/` font names, placed PDF page and crop, package IDML without embedded images | not in prior snapshot; needs detailed delta review |
| [#701](https://github.com/storytold/designcraft/pull/701) | docs: fix the README font integration link | unchanged head since prior live review |
| [#366](https://github.com/storytold/designcraft/pull/366) | Text: input method (IME) composition in the Type tool | not in prior snapshot; needs detailed delta review |
| [#319](https://github.com/storytold/designcraft/pull/319) | DTD: bound parameter-entity expansion, nesting and sequence matching | not in prior snapshot; needs detailed delta review |
| [#330](https://github.com/storytold/designcraft/pull/330) | Compose: the CJK leading model and character alignment apply only under the Japanese composers | not in prior snapshot; needs detailed delta review |

## Validation and local build

All seven `CARGO_INCREMENTAL=0 cargo xtask ci` steps pass: fmt, workspace/all-target
check, Clippy with warnings denied, tests, assets, layers and WASM. The complete
run reports **2,684 passed, zero failed, seven existing ignores**; twenty WASM
packages pass. No dependency versions were updated; IDML now uses the existing
workspace archive guard.

Offscreen UI images were inspected: last-page navigation, visible/hidden/restored
floating palettes and the Color panel with its separate Tint field. Synthetic
widget tests also verify typed tint persistence, Enter/focus-loss application,
slider release, page/spread shortcuts, panel hiding and preference round-trips.
Linux packaging YAML parses; RPM native installation and native Windows/Linux
execution remain unverified in this local Mac pass. No remote publication.

The separate optimized local app includes matching 0.7.3 GUI/CLI executables,
Pagenor owner artwork, separate .pgnr/legacy document icons and current bundle
metadata. Final source commit, binary/icon hashes, ad-hoc strict/deep signature
verification and a bundled-CLI sample PDF export are recorded in the artifact's
`BUILD_STATUS.json`. Running app bundles and document sessions are not replaced.

The separate Anu6/Anu7 Unicode-conversion work still requires verified matching
maps; the older Anupama tables are not applied by these changes.
