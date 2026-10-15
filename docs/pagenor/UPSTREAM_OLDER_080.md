# P11.18 — older upstream PR updates, local Pagenor 0.8.0

Review date: 15 October 2026. Base: `e3f03ca8d3627e9165f1357a429b1168ec6ee217` (0.7.3).
The owner requested older proposals before new ones. Reviewed the source diffs of
all 30 older proposals in the previous updated-PR queue, including open and closed
records. Sixteen source proposals supply compatible changes (including #643’s
prerequisite through #748); eleven have existing Pagenor equivalents; three are
excluded/deferred as noted below. This is local work; no GitHub write or release.

Pagenor retains MIT OR Apache-2.0 source provenance and Appaji Ambarisha’s branding
rights. Rust layering, unsafe_code=forbid, .pgnr/legacy documents, separate icons,
lazy/scoped fonts, live activation keys, inline Indic input and explicit masters
remain. No private owner documents, fonts or encoding tables are committed.

## Source pins and outcomes

| PR | Author | Inspected head | Outcome |
|---|---|---|---|
| [#253](https://github.com/storytold/designcraft/pull/253) | @moppeeer | `6f127f9a4f0e6e5a523b14ec05025dc9266f6348` | Adapted: narrow Windows caption/scroll layout and lower drag band; retain native Mac title bar |
| [#255](https://github.com/storytold/designcraft/pull/255) | @moppeeer | `da7a457c9522d5f0b078f5577661bb69d8b3d963` | Adapted shared widget response/accessibility API; existing compact object controls retained |
| [#303](https://github.com/storytold/designcraft/pull/303) | @sanyamsingh001 | `c8840b61fb77a37d0840e232693316559ba7759c` | Adapted: one exact Pagenor UI-command registry shared with headless MCP; unknown IDs remain unknown |
| [#319](https://github.com/storytold/designcraft/pull/319) | @haranishi | `eb82632591fd6df55d1ae1b1f86b4621f97b0bd9` | Adapted: bounded, linear parameter expansion and memoized/deduplicated DTD matching |
| [#325](https://github.com/storytold/designcraft/pull/325) | @moppeeer | `3d91e6460572071f2ff72ce9832dfa42709cc67d` | Adapted: switching preset tabs selects their first card; reselecting the current tab preserves its card/name |
| [#330](https://github.com/storytold/designcraft/pull/330) | @aiongg | `04420fd25c83d507c3d0a6d3ad5519cff97847be` | Excluded: CJK composer additions are outside the owner’s current scope |
| [#341](https://github.com/storytold/designcraft/pull/341) | @moppeeer | `a6814bc61acc1eb2279c6baa18c67db49eda0992` | Already covered: bounded chapter numbering, styles, book order and prefix behavior |
| [#342](https://github.com/storytold/designcraft/pull/342) | @moppeeer | `0d89d869926770f65cdc7e37c7c3786179b0b095` | Already covered by Pagenor’s atomic layout.numberingSection and existing section dialog; no parallel numbering API |
| [#362](https://github.com/storytold/designcraft/pull/362) | @aiongg | `ac524c4e7ac3710848ce9f4299a13a3c3ca7aee4` | Adapted PDFAttribute properties and bounded page parsing; existing scoped font, crop/cache/export/package implementations retained |
| [#366](https://github.com/storytold/designcraft/pull/366) | @aiongg | `06892bbc7e90a91289f4580c60d48d64b1a30e30` | Retain Pagenor’s tested inline Indic/VariType IME and scoped host bridge; no wholesale upstream IME replacement |
| [#388](https://github.com/storytold/designcraft/pull/388) | @moppeeer | `16e3469f7a4237a676a692fdce52702370e275f9` | Already covered: Pencil path overlays and their native/offscreen regression already exist |
| [#400](https://github.com/storytold/designcraft/pull/400) | @seanth | `2480f469dec5877f4e6a7ed3990afcb4893f1b3c` | Not adopted: proposed Cocoa bridge adds unsafe Rust to a workspace that forbids it; existing OS/file drop support retained; new running-app Apple events remain separate |
| [#460](https://github.com/storytold/designcraft/pull/460) | @sanyamsingh001 | `c8eb56402d034ac567975cb3cfc112bd48003394` | Current resize behavior retained; additional signed-span, stroke-floor and cancel regressions validate it; idle corner widgets are preserved |
| [#465](https://github.com/storytold/designcraft/pull/465) | @sanyamsingh001 | `456c8ba34c839e962f628a1d08bdea3571aad5cc` | Deferred optional language expansion: Russian patterns/normalization are not added in this core batch |
| [#466](https://github.com/storytold/designcraft/pull/466) | @moppeeer | `dba8774a139041f85fb67fba3bdee1fb80bd0fec` | Adapted: bound menu preview textures to an LRU of 256; retain lazy metadata and activation-generation/document scope keys |
| [#467](https://github.com/storytold/designcraft/pull/467) | @sanyamsingh001 | `7a0840285732b2eef84a2a4c3240395d3a0211dd` | Already covered: captured targets, explicit/mixed edits, rules and partial insets; retained Pagenor’s existing tests and dirty-field model |
| [#468](https://github.com/storytold/designcraft/pull/468) | @soro-82 | `5f535ef60f87b03dd69a65d9e64e6322e4816331` | Already adopted at this unchanged source head; preserve explicit left/right master settings and guide inheritance |
| [#469](https://github.com/storytold/designcraft/pull/469) | @moppeeer | `12a41f69dbbae6f303aed0ecbd76a1ecc6021e40` | Already covered: ruler-unit setup, linked margins, orientation and preserved values; primary-frame status remains read-only |
| [#501](https://github.com/storytold/designcraft/pull/501) | @aiongg | `e22205039c1615f71a76e1c88b4e9da811c3c29f` | Already covered by Pagenor’s explicit master inheritance/overrides, guide display and snapping; do not replace that model |
| [#525](https://github.com/storytold/designcraft/pull/525) | @moppeeer | `a14acea3a404202a7cbf11df8366b9324b683b0b` | Already covered: Properties tint and frame/text targeting retain existing command-backed behavior |
| [#542](https://github.com/storytold/designcraft/pull/542) | @sanyamsingh001 | `3e12b56a9655adfd790330b210ec8de59d37f0c7` | Adapt compatible fixes: EOF style capture, table-cell typing, separator leading and inherited swatch references; preserve Pagenor format-5/marker semantics |
| [#550](https://github.com/storytold/designcraft/pull/550) | @moppeeer | `e9d3effe65eb946f47e714551d07aabacaee1ee1` | Adapted incremental: Transform Info reports the selected graphic’s measurements; retain existing guarded transforms |
| [#580](https://github.com/storytold/designcraft/pull/580) | @marcoarnulfo | `eb4e811b522922a894e814e02bf2de3bfbbee1fd` | Adapted: portable.txt routes settings/logs/recovery to PagenorData, respects explicit profile override and warns on unwritable locations |
| [#591](https://github.com/storytold/designcraft/pull/591) | @moppeeer | `183feade3bc5e74541effd270dde175240302ab0` | Adapted text fitting and axis-aware non-text handle fitting; retain Pagenor’s established text-frame/Direct Selection multiple-click entry |
| [#618](https://github.com/storytold/designcraft/pull/618) | @moppeeer | `bfe7ba75b74584b93eafcad3dcd552b60161763f` | Adapted editable SVG clipboard paths/groups/paints with bounded parsing; preserve literal SVG when there is a text insertion point |
| [#643](https://github.com/storytold/designcraft/pull/643) | @moppeeer | `c8cbe0a0dc009dc42f3b875e3c5a45c967022571` | Effects-dialog prerequisite adapted via the pinned #748 superset; retain existing serialized effect defaults |
| [#689](https://github.com/storytold/designcraft/pull/689) | @moppeeer | `8ca022a11d5ec44e9a6b664766ab75813bab110b` | Adapted: Ungroup leaves clipping/pasted-into frames intact and selected; labels follow the frame shape |
| [#699](https://github.com/storytold/designcraft/pull/699) | @moppeeer | `b3fb92a97ff95a4a2996a2ce5bc3a1c784626cfb` | Adapted: graphic content handles resize the graphic within a fixed frame, with preview/commit/cancel and geometry guards |
| [#701](https://github.com/storytold/designcraft/pull/701) | @haranishi | `bd93ea92c348d01ca48eaba54956bafbbb946f8c` | Already covered by Pagenor’s own README font integration link and project naming |
| [#748](https://github.com/storytold/designcraft/pull/748) | @moppeeer | `5ccfc89311347a33d6d3bca6b49c535d4d7da81e` | Adapted: Effects levels, fx badges, Clear/Remove actions and the multi-page dialog with one-step preview/apply/cancel |

All 30 heads were rechecked on 15 October. Twenty-nine remained unchanged.
#400 moved to `25dc9ff57d06c8ef5ea00905640e830548d90431`; its new diff was
inspected and still introduces unsafe Objective-C calls, so its disposition stays
unchanged. The 16 adapted source heads all remained unchanged.

## Integration details

- Effects uses the existing engine interaction model, one final undo step, numeric
  sanitization and captured object IDs. A document UID guard and suspension before
  document switches prevent a preview or confirmation reaching another document.
  Editing applies at Object level; independent Stroke/Fill/Text effects remain
  outside the current document model and their controls stay disabled.
  Panel grouping/closing and Pagenor’s saved effect defaults remain. Locked items
  and layers are skipped without an empty history step.
- Fit Frame to Content now measures horizontal text with its actual columns,
  insets and justification, searches within 20,000 pt and validates every target
  before changing it. Non-text handle double-clicks fit their local axis; Pagenor’s
  existing text-frame and Direct Selection multiple-click input remains.
- SVG paste requires SVG markup (for example Copy as SVG). Paths become editable,
  groups and solid fill/stroke values are retained. Gradient approximation/pattern omission and
  unsupported content generate warnings; native AI/PDF clipboard formats are not
  newly supported. Existing text insertion keeps SVG markup as literal text.
- Graphics use their own content bounds for handles and Transform Info. Frames,
  clipping, original geometry guards, locked targets, history and Escape remain.
- Font previews retain document/activation-generation keys and are capped at 256
  textures. Family/style metadata and font discovery remain lazy.
- Portable settings use `portable.txt` beside the executable (or beside a Mac app)
  and a sibling `PagenorData` directory. Explicit `PAGENOR_CONFIG_DIR` wins; an empty
  override disables profile use. An exclusive temporary write probe avoids clobbering
  existing files. Unwritable portable folders fall back with a warning; per-run
  recovery isolation remains. Windows ZIP packaging includes the marker.
- Preset tabs select the first available card when changing tabs. Clicking the
  active tab keeps its chosen card. Pagenor’s empty Saved tab keeps the entered
  details instead of adding a synthetic saved preset.
- The #542 adaptation keeps Pagenor’s native-format-5 staging and tested selected-LF
  marker policy. Four incoming tests tied to different unmarked-default/marker
  semantics are not copied. Compatible source regressions are retained alongside
  all original EOF/transport/consumer tests. Joining stories writes a correctly
  styled separator; table cells and style capture read the persisted caret format;
  swatch renaming also visits the inherited EOF defaults. The genuine-default
  swatch fixture uses Pagenor’s existing supported paragraph-to-EOF workflow.
- DTD expansion rejects cycles/deep chains and checks its size budget before
  appending. Sequence/repetition matches memoize end positions instead of repeating
  exponential paths. Existing XML and archive budgets remain.

## Validation

All seven workspace gates pass: 2,768 tests, zero failures, 7 existing ignores;
twenty WASM packages. Offscreen inspection covers the Effects dialog/panel,
editable SVG paste and New Document presets. A separate optimized local macOS
arm64 app includes the matching CLI. Strict/deep ad-hoc signature verification,
version/icon checks and a four-page synthetic CLI PDF export pass. Native
Windows/Linux execution and publication remain pending. The current owner app
session is left open.
The separate Anu6/Anu7 Unicode work still requires matching verified maps; the
older Anupama mapping is not applied.
