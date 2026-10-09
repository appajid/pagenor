# Upstream v0.4.0 release audit — P00.8

Reviewed the 34 items in the upstream v0.2.1 → v0.4.0 release notes, published
2026-10-08. Release revision: `14e677b24216396e398823ff034d9882322362d1`.
Comparison baseline: Pagenor `acee8b66de015bd009801a1ba89a46eb87465ea0`.

The 32 application/workflow contributions below are already ancestors of the
fork's initial source. Their current implementation paths were checked; recent
Pagenor adaptations and stronger font/recovery/clipboard/control behavior remain.
Reapplying these branches would duplicate work and could remove those fixes.
The two missing listed changes, distribution/documentation #132 and #135,
are adapted in P10.2. All 34 listed items are now accounted for: 32 retained
inherited implementations and two selected adaptations. No blanket upstream merge
is needed. Native package execution remains separate from source completion.

| PR | Change | Current evidence / action |
|---|---|---|
| [#37](https://github.com/storytold/designcraft/pull/37) | Text: text.select documents its offsets as UTF-8 bytes | `engine/cmd/text.rs`: UTF-8 byte-offset command contract and boundary-aware selection. |
| [#38](https://github.com/storytold/designcraft/pull/38) | Text: carriage returns in typed or created text start paragraphs | `engine/cmd/mod.rs`: text parameter normalization; frame/story creation and insertion share it. |
| [#39](https://github.com/storytold/designcraft/pull/39) | Engine: commands given ids run without a selection | `engine/cmd/mod.rs`: explicit id/ids targets bypass selection-only enablement without changing selection. |
| [#41](https://github.com/storytold/designcraft/pull/41) | PDF: [Paper] prints as no ink in print exports | `pdf/export.rs` and `paper_prints_as_no_ink`: Paper is zero CMYK in print. |
| [#40](https://github.com/storytold/designcraft/pull/40) | PDF: placed PDF 1.7 files go into PDF/X-4 exports | `pdf/export.rs`: placed PDF version negotiation and PDF/X output paths retain PDF 1.7 support. |
| [#42](https://github.com/storytold/designcraft/pull/42) | PDF: transparency groups blend in CMYK in print exports | `pdf/export.rs`/`pdfx.rs`: document blend space and CMYK transparency groups. |
| [#43](https://github.com/storytold/designcraft/pull/43) | CLI: a closed stdout ends upstream CLI quietly | `pagenor-cli/main.rs` and `closed_stdout_ends_quietly`: BrokenPipe exits quietly. |
| [#44](https://github.com/storytold/designcraft/pull/44) | Preflight: placed PDF and SVG graphics have no resolution to check | `engine/cmd/preflight.rs`: vector PDF/SVG previews are excluded from raster-resolution warnings. |
| [#36](https://github.com/storytold/designcraft/pull/36) | Fix #22: installed fonts are found by name whatever ran before | `fonts/fontdb.rs`: installed lookup and lazy catalogue; P01 adds dynamic/native/scoped fixes. |
| [#33](https://github.com/storytold/designcraft/pull/33) | Snap: smart guides and snapping | `tools/snap.rs`, canvas overlays and preferences: guide/object/grid/dimension/spacing passes. |
| [#49](https://github.com/storytold/designcraft/pull/49) | Windows on ARM64: release builds, PE header check, install test on ARM64 hardware | `windows-arm64.yml`: cross-package, PE machine check, native CLI install/uninstall workflow retained. |
| [#51](https://github.com/storytold/designcraft/pull/51) | Fonts: use craft-fonts as an optional build input; move the Japanese fonts out | `fonts/build.rs` and pinned workflow inputs: optional craft-fonts; no new bundled Japanese font files. |
| [#52](https://github.com/storytold/designcraft/pull/52) | Add Simplified Chinese and Arabic localization with RTL support | `ui-egui/i18n.rs`: Simplified Chinese and Arabic; shared RTL labeling. |
| [#95](https://github.com/storytold/designcraft/pull/95) | Rename upstream sibling to PDF sibling | Pagenor ownership/branding retained; the existing PdfCraft reference is the external sibling name, not a product rename. |
| [#108](https://github.com/storytold/designcraft/pull/108) | About: contributor and model credits compiled in (grab bag + table, self-submitted names) | `ui-egui/build.rs`/`about.rs` and contributor registries: compiled credits; no contributor profiles copied or modified. |
| [#105](https://github.com/storytold/designcraft/pull/105) | IDML: import scalar InsetSpacing property values | `idml/import.rs`: scalar/list inset properties; scalar inset fixtures remain. |
| [#102](https://github.com/storytold/designcraft/pull/102) | IDML: honor explicit cell edges at table borders | `doc/table.rs`, IDML import/export and compose: explicit cell-border overrides retained. |
| [#104](https://github.com/storytold/designcraft/pull/104) | Render: keep outside-aligned strokes out of closed interiors | `render/lib.rs` and `render/tests/stroke_alignment.rs`: outside strokes preserve interiors/fill/transparency. |
| [#101](https://github.com/storytold/designcraft/pull/101) | IDML: resolve nested packaged image links after relocation | `engine/cmd/interchange.rs`: packaged nested image links resolve relative to relocated document. |
| [#97](https://github.com/storytold/designcraft/pull/97) | Fonts: read Microsoft Office's bundled fonts on macOS | `fonts/fontdb.rs::system_font_dirs`: Microsoft Word/Excel/PowerPoint/Outlook font directories on macOS. |
| [#79](https://github.com/storytold/designcraft/pull/79) | Transform: Rotate and Shear open their dialogs (#34) | `ui-egui/menus.rs`/`dialogs.rs`: Rotate/Shear dialog routing and typed command values. |
| [#53](https://github.com/storytold/designcraft/pull/53) | Canvas: middle-mouse drag pans the view | `ui-egui/canvas.rs`: middle-button pan and control pointer support. |
| [#56](https://github.com/storytold/designcraft/pull/56) | Guides: honor layer protection in move and delete commands | `engine/cmd/guides.rs::check_editable`: locked/hidden guide-layer protection on move/delete. |
| [#66](https://github.com/storytold/designcraft/pull/66) | Control: close the connection on input that isn't a request | `pagenor/control_server.rs`: invalid non-request input closes; stronger P00.5 bounds/auth remain. |
| [#75](https://github.com/storytold/designcraft/pull/75) | Preserve and compose Chinese and Arabic typesetting rules | `doc`, `compose`, IDML and font shaping: Chinese/Arabic typography attributes and composition. |
| [#58](https://github.com/storytold/designcraft/pull/58) | PDF/X-4: flag device RGB under the CMYK output intent | `pdf/pdfx.rs`: device-RGB checks against CMYK output intent, including groups. |
| [#85](https://github.com/storytold/designcraft/pull/85) | CJK: language-aware shaping and fallback, vertical metrics, document fonts, missing-glyph boxes (builds on #75) | `fonts`/`compose`: language-aware fallback, vertical metrics, document scopes and missing-glyph boxes; P04 fixes preserved. |
| [#82](https://github.com/storytold/designcraft/pull/82) | Images: PSD composites decode by colour mode | `images/psd.rs`: bitmap/gray/indexed/RGB/CMYK/Lab composites and channel/depth regressions. |
| [#103](https://github.com/storytold/designcraft/pull/103) | i18n: add Brazilian Portuguese (pt-br) UI translation | `ui-egui/i18n/pt_br.rs`: Brazilian Portuguese and cached translation lookup tests. |
| [#55](https://github.com/storytold/designcraft/pull/55) | PDF: placed CMYK TIFFs keep their inks | `images/lib.rs` and `cmyk_tiff_keeps_its_inks`: ink-preserving CMYK TIFF PDF path. |
| [#78](https://github.com/storytold/designcraft/pull/78) | Complete Arabic translations, refine Chinese terminology, and cache RTL label layouts | `ui-egui/i18n/ar.rs`, Chinese catalogue and `rtl.rs`: translations and RTL label-layout cache. |
| [#76](https://github.com/storytold/designcraft/pull/76) | Data merge: linked source, placeholders, preview, and tiled merge | `doc/datamerge.rs`, engine commands and panel: linked sources/placeholders/preview/tiled merge; P09.3 advanced additions retained. |
| [#132](https://github.com/storytold/designcraft/pull/132) | Release: FreeBSD, Flatpak x86_64/aarch64, AppImage zsync; README Downloads | `packaging/freebsd/package.sh`, `linux/flatpak-bundle.sh`, source/bundle manifests, `linux/package.sh` and `release.yml`: adapted FreeBSD tar, matching-architecture Flatpak bundles and owner AppImage zsync routing. Bounded extraction, X11 host permissions, notices and synthetic regression checks added. Existing nfpm description retained; maintainer corrected to the owner. |
| [#135](https://github.com/storytold/designcraft/pull/135) | README Downloads: where to get it, then builds (Windows first) | `README.md` Downloads/build-format table (Windows first) and `docs/releasing.md`: Pagenor’s own repository, local build status, package commands, signing and native-validation scope. |

## Version checkpoint

Pagenor 0.3.0 records the accumulated application features and verified inherited
release behavior since its retained 0.2.1 label. P10.2 distribution additions
also stay at **0.3.0**, as requested by the owner on 2026-10-10. This is a local
source version, not a published release or GitHub tag. Workspace packages inherit one version; native document
format version 3 and `.pgnr` compatibility do not change.

The full seven-step gate is required before each checkpoint commit. Actual
Windows/Linux/FreeBSD package creation, signing and platform execution remain
milestone-host checks. Existing craft-font tests may skip when the optional input
is absent; inheritance/source evidence does not imply new native-host testing.

Pinned PR merge/head revisions and per-item source paths are recorded in the
local release audit JSON. The original copyright/licences and Pagenor branding
remain. The owner cancelled Adobe/dongle validation; this audit does not revive it.

P00.8 validation: all seven local gates pass at 0.3.0: **1,308 passed, zero failed,
3 existing ignored; 20 WASM packages**. External Cargo.lock packages are byte-for-byte
unchanged as parsed records. [Pinned item revisions](upstream-040-audit.json).

P10.2 validation: all seven local gates pass at **0.3.0**, with **1,308 passed,
zero failed, 3 existing ignored; 20 WASM packages**. Eight additional synthetic
packaging tests pass, covering both architectures, archive rejection, licence
contents, update URLs/stale files and workflow routing/publication gates. All ten
packaging shell scripts pass Bash syntax checks and ShellCheck 0.11.0; all
workflows pass actionlint 1.7.12. The new platform packages have not been built
or installed on their native hosts during this local task.
