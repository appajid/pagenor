# P11.19 — upstream review and local Pagenor 0.9.0

Review snapshot: 16 October 2026 (Asia/Kolkata). Base: `64f41ef561ba8bdcfaf947f17ff2fbd32238b7a4` (0.8.0).

Reviewed 300 latest-created and 100 latest-updated open/closed PR records through #1081. The snapshot contains 25 new proposals after #1054 and two older updated heads. Twenty-two new proposals plus the incremental #325 update supply compatible changes; #1071 is already covered; #1070 and #1077 remain queued for missing prerequisites. #400 remains excluded.

Pagenor retains its MIT OR Apache-2.0 source provenance and Appaji Ambarisha’s reserved branding rights, Rust layers and unsafe_code=forbid, .pgnr/legacy loading and separate icons, scoped/lazy fonts, inline Indic input and explicit book masters. No private font, document or encoding map is included.

## Source pins and outcomes

| PR | Author | Inspected head | Outcome |
|---|---|---|---|
| [#325](https://github.com/storytold/designcraft/pull/325) | @moppeeer | `c96d3e1faba00e119d8f74b14c9a2f614542f09e` | Incremental adaptation from changed head: Mobile presets enable primary text frames. Retain all Pagenor measurement units and its existing Saved-tab semantics. |
| [#400](https://github.com/storytold/designcraft/pull/400) | @seanth | `25dc9ff57d06c8ef5ea00905640e830548d90431` | Not adopted: current head still introduces workspace unsafe Objective-C calls. Same head as the final P11.18 recheck; existing file/open support retained. |
| [#1056](https://github.com/storytold/designcraft/pull/1056) | @moppeeer | `29f02d54581af5657c4e841381c20885eb1e4020` | Adapted: page-paper underpaint while a render is pending. |
| [#1057](https://github.com/storytold/designcraft/pull/1057) | @sanyamsingh001 | `61740eadbddddca7f7573543b979aabea20ac42c` | Adapted: rebase extreme top-level story revisions on native/legacy load, retaining linked-story freshness and archive limits. |
| [#1058](https://github.com/storytold/designcraft/pull/1058) | @moppeeer | `5ab0abaf6f6b29829bf22e9a8ba86e9acc32f4c9` | Adapted: page/spread deletion warnings, session opt-out and source-document identity guard. |
| [#1059](https://github.com/storytold/designcraft/pull/1059) | @moppeeer | `09f80899431366d6338aa02629f67fa625bda956` | Adapted: Edit Page opens page-specific Properties via scoped UI state; existing page-size commands and Page tool retained. |
| [#1060](https://github.com/storytold/designcraft/pull/1060) | @moppeeer | `7313bdfc93241b3fc3a98b40cc2aa7c304e349c8` | Adapted: effective style override indicators and Clear Overrides row action; existing raw query counters remain stable. |
| [#1061](https://github.com/storytold/designcraft/pull/1061) | @moppeeer | `4f3a55a190e1191f5625b302d86825bf3161ec2a` | Adapted: captured visible text follows later style edits; preserve Pagenor’s caret/final-LF/default-marker contracts. |
| [#1062](https://github.com/storytold/designcraft/pull/1062) | @moppeeer | `7d4ddc3679e24ef71760095c7c1ba743f77d4e50` | Adapted: style previews use transactional apply/cancel and source-document binding; original history/dirty-field behavior retained. |
| [#1063](https://github.com/storytold/designcraft/pull/1063) | @moppeeer | `4ea23d4ccd8bc3452a51e6a4598845780f2d6370` | Adapted: Typography panel groups integrated with Pagenor’s existing grouped and dual-column panel system. |
| [#1064](https://github.com/storytold/designcraft/pull/1064) | @moppeeer | `dea0352299f8f721553ad41c30456900b8bcda03` | Adapted: redefine paragraph/character styles from local formatting, with one engine history step. |
| [#1065](https://github.com/storytold/designcraft/pull/1065) | @moppeeer | `70df1191495108c675821e73825461abd41fd01f` | Adapted: paragraph Style Settings summarizes color, spacing and pending attribute changes relative to its parent. |
| [#1066](https://github.com/storytold/designcraft/pull/1066) | @moppeeer | `b50eb265af6e5851cf2d8aca8c62d9f8b80a7089` | Adapted: master/page sections scroll independently with a draggable divider and fixed footer; live left/right masters retained. |
| [#1067](https://github.com/storytold/designcraft/pull/1067) | @moppeeer | `ff71790737bbe1a2a6220bcad43b3868416b9f16` | Adapted: document/master thumbnail sizes, layout order and resize modes, with live book-master controls and master tiling. |
| [#1068](https://github.com/storytold/designcraft/pull/1068) | @marcoarnulfo | `f65eb0fd4202f34b5bf5e3a42f613ef259a0ed88` | Adapted: convert selected rows to header/body/footer with edge/merge validation. |
| [#1070](https://github.com/storytold/designcraft/pull/1070) | @switch87 | `c71ab5d12bb4501c0d5329ffbceee6e372dea52c` | Queued: requires earlier configurable TOC-entry model and its settings/import/export UI, absent in this fork. Existing TOC retained; no partial changes applied. |
| [#1071](https://github.com/storytold/designcraft/pull/1071) | @moppeeer | `02a7b28d5b41d1d5fa1111b923ab84f9a628d494` | Already covered: retained Document Setup fields fit full bleed/slug values; source regression added. |
| [#1072](https://github.com/storytold/designcraft/pull/1072) | @danielmigueltejedor | `4d1833d2e60c559558df05ef3d15960c7fb7f797` | Adapted: Linux-only winit Adwaita window controls; dependency asset/licence notices and packaging of NOTICE. Feature graph verified; native execution pending. |
| [#1073](https://github.com/storytold/designcraft/pull/1073) | @moppeeer | `a8388df9e8a5341c667cfc796612cda68ee64105` | Adapted: inches display up to four decimals, other units retain three; trailing zero behavior unchanged. |
| [#1074](https://github.com/storytold/designcraft/pull/1074) | @moppeeer | `0e5a6bd9d1b2011cb51c22b03a75d2db0d37af5c` | Adapted: ruler-corner units apply to both rulers, with individual menus, Hide Rulers and guide action. |
| [#1075](https://github.com/storytold/designcraft/pull/1075) | @Hamza0101 | `39ef7799c0c89ac2ede378f661ec3fd4222fa4fb` | Adapted: loaded text cursor finds grouped frames through depth-bounded descendant hit testing. |
| [#1076](https://github.com/storytold/designcraft/pull/1076) | @moppeeer | `42be0c57601fc0d120c11f2674495894e107dad9` | Adapted: tool flyouts close on Escape/outside click, with accessible flyout entries. |
| [#1077](https://github.com/storytold/designcraft/pull/1077) | @Hamza0101 | `76d0560ff839c22310bf7f0cb51956e39fae2184` | Queued: assumes a multilingual hypher facade; Pagenor currently uses its English dictionary/pattern implementation. Greek pattern MIT option verified; no replacement composer/data added. |
| [#1078](https://github.com/storytold/designcraft/pull/1078) | @marcoarnulfo | `ed218b9817e79cfafa3277a123de5ef03859f8e9` | Adapted: text parameters normalize vertical tab to a forced line break. |
| [#1079](https://github.com/storytold/designcraft/pull/1079) | @moppeeer | `c4772c37450cd75b8a6da15ce73672ff1716e1d5` | Adapted: deterministic local raster Color Theme extraction (bounded at 16 million pixels) and undoable palette swatch groups. |
| [#1080](https://github.com/storytold/designcraft/pull/1080) | @moppeeer | `7cf1e30acb7beed0127899549902191c41e0601d` | Adapted: Color Picker component field/slider, HSB/Lab/RGB/CMYK, old color and Add CMYK Swatch; existing target routing retained. |
| [#1081](https://github.com/storytold/designcraft/pull/1081) | @moppeeer | `528c7ea4891ff121fd63bdecd6a9d3bd5a8ae594` | Adapted: per-document new-shape fill/stroke colors, native persistence, history, validation and swatch renaming; Pagenor’s existing proxy actions retained. |

Color Picker #1080 advanced from `2a7f4d95059006d20f559b841ba4fae8cd276dcb` during the review; the additional layout/numeric-entry diff at `7cf1e30acb7beed0127899549902191c41e0601d` was inspected and adapted. Pagenor’s deferred hex typing and modal keyboard behavior are retained. Hex uses a monospace code field so its glyphs and partial input remain visible; component text fields have stable edit identities.

## Compatibility choices

- Formatting queries retain raw `paraOverrides` and `charOverrides`. The new `effectiveParaOverrides`, `effectiveCharOverrides` and `characterStyleOverrides` drive the visible indicators. EOF staging, explicit genuine defaults and final-LF formatting keep their existing semantics.
- Style previews use the existing interaction engine and commit one final edit. Cancel, replacement, quit, document switches and stale confirmation are checked. Effects shares the same modal cleanup. Deletion warnings capture their originating document.
- Pages options keep live left/right master thumbnails, default/application actions, header/footer/page-number controls and master settings. Thumbnail keys include their size and document/revision scope. Jumbo thumbnails in short panels have bounded divider ranges. Master cards retain enough space for their existing controls, and unselected page labels follow the system theme.
- Native format **6** retains future-shape document colors. Prior formats and the legacy .pagenor extension remain readable. Older readers reject the new capability instead of silently removing it. The format-5 EOF representation itself is unchanged.
- Text and graphics frames retain their current object-style defaults; the new color defaults apply to newly drawn shapes/lines. Current Tools proxy actions remain.
- Color Theme samples the source raster on a bounded grid, ignores transparent padding and previews the palette before an undoable swatch-group command. Palette cuts occur only between distinct component values, retaining the exact palette of solid-color images with up to five colors. Vector-image palette extraction is unsupported. No network service is involved.
- Linux adds six locked decoration/font dependencies without updating unrelated dependency versions. `NOTICE` carries the Adwaita MIT and Cantarell OFL text; the font remains embedded by the Linux dependency, not copied into Pagenor assets. The Linux dependency graph is checked separately from native execution.
- TOC #1070 needs earlier entry-style/page-number/separator settings throughout the model, IDML and UI. Greek #1077 expects a multilingual hyphenation facade that is absent here. Both remain explicitly queued; their incomplete patches are excluded. The Greek source offers an MIT licence: [primary pattern header](https://raw.githubusercontent.com/hyphenation/tex-hyphen/master/hyph-utf8/tex/generic/hyph-utf8/patterns/tex/hyph-el-monoton.tex).

## Validation

All seven `CARGO_INCREMENTAL=0 cargo xtask ci` gates pass: fmt, all-target workspace check, Clippy with warnings denied, tests, asset audit, layer audit and twenty-package WASM check. **2,815 tests pass, zero failures, 7 existing ignores.** The original EOF/default-marker/IME, import, recovery, master-page and color-target regressions remain enabled. New tests exercise style capture/redefinition/preview cancellation, effective override counts, loaded revision freshness, deletion warnings, table row conversion, grouped threading, bounded palette sampling and shape-default persistence/history. UI regressions cover the Pages divider/short viewport/options, Typography stack, page Properties, ruler menus, flyout dismissal and color numeric/hex typing.

Offscreen visual inspection uses original synthetic content only. Six renders cover live Pages/master controls, Pages Panel Options, Typography, paragraph-style Preview/summary, Color Picker and Color Theme. No owner app is launched, closed or replaced. Source heads for all 27 reviewed records were rechecked unchanged after the additional #1080 adaptation.

The optimized local macOS arm64 GUI and matching CLI are built from the committed source; artifact-side `BUILD_STATUS.json` records source/hash/signature/version/icon evidence. CLI smoke checks export a four-page synthetic PDF, write/reopen native format 6, and verify that the older 0.8.0 CLI rejects that capability without creating an output. Build verification is recorded alongside the artifact rather than changing this committed note afterward.

Native Windows/Linux execution and remote publication remain pending; shared workspace/WASM and Linux dependency-graph checks do not establish native execution. Owner app sessions stay open. Anu6/Anu7 Unicode conversion remains separate and awaits matching verified maps.
