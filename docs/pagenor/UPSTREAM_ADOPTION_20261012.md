# P11.11 — Pagenor 0.6.0 local adoption, 12 October 2026

The twenty proposals in the owner’s UI-first list through #491 are adapted locally: #466, #469,
#467, #486, #487, #488, #491, #477, #478, #460, #474, #468, #459, #473, #472, #485, #475,
#462, #482 and #463. Additional compatible changes from the [11 October review](UPSTREAM_20261011_RECHECK.md)
are recorded below. Workspace version is 0.6.0, updated with `cargo xtask version set`; dependency
versions were not upgraded. This is a local development milestone, not a published cross-platform
release or completion of the whole development roadmap. Owner application bundles/sessions and
GitHub have not been replaced or changed.

## Implemented in source

| PR | Author | Pinned source head | Local adaptation |
|---|---|---|---|
| [#504](https://github.com/storytold/designcraft/pull/504) | @aiongg | `2077d9f088dc41bde6b201b098a596ded61fafd7` | Installed interface fallbacks for Indic and other scripts, with marks checked, shared face data, background work and refresh generations; retain existing CJK policy. |
| [#498](https://github.com/storytold/designcraft/pull/498) | @aiongg | `fe8a724da44ecd9cc997118340c55390f2e20825` | Malformed note/marker payload extraction repair, checked footnote/endnote/editorial insertion, wholesale text replacement cleanup; retain rich clipboard IDs. |
| [#506](https://github.com/storytold/designcraft/pull/506) | @noah1400 | `257fb08dbdc1140223e8db1f897845bff6913d9c` | Locked per-process recovery namespaces and coordinated takeover of released runs. Preserve legacy flat entries for explicit recovery; refuse shared-folder fallback and removed/replaced locks. |
| [#485](https://github.com/storytold/designcraft/pull/485) | @aiongg | `1a8df25fddb6e555bbbc11578a262585fcc0f1b2` | Use the stronger existing native writer for export, snippet, book, library, ASE, XML, MCP and CLI output. Preserve file metadata; add directory-flush diagnostics and bounded Windows replacement retries. |
| [#509](https://github.com/storytold/designcraft/pull/509) | @mjohnson7cd | `a02fc011bf5af1300a7e59ecb146487138fe38d1` | Paragraph panel optical margin alignment checkbox through type.para with undo. |
| [#510](https://github.com/storytold/designcraft/pull/510) | @japbcoelho | `5a1e9de92ed789abc176ab668fee68378a224ee3` | One RGB/CMYK component scale per colour, preserving Gray and finite/range checks. |
| [#511](https://github.com/storytold/designcraft/pull/511) | @japbcoelho | `72bdc0a9197985b000105b6291daf89d6cc9fd06` | Fractional PDF/SVG natural dimensions through direct Place, place cursor, fitting, tables and data merge; retain EPS validation. |
| [#473](https://github.com/storytold/designcraft/pull/473) | @MarekSpodBiedry | `30cbd67e3de32bf5d810c6569880b54a2bfffdc1` | SameParaStyleSpacing import, composition, IDML export and explicit-null inheritance/serialization. |
| [#474](https://github.com/storytold/designcraft/pull/474) | @MarekSpodBiedry | `5ab84500fed60deb67a3ef451d20fb765ad86a18` | Single-sided pages use the first listed master; preserve geometric left/right selection for RTL binding and shared snapping. |
| [#497](https://github.com/storytold/designcraft/pull/497) | @aiongg | `cde814c0dff8aaa5321b5fe4cccf9fd9f53d9d3d` | Alt shrinking/flipping, centered Alt+Shift aspect resize and Shift side-handle centering. |
| [#488](https://github.com/storytold/designcraft/pull/488) | @MarekSpodBiedry | `233aef8986b1113b16a50841a5dc7b519489e94b` | Fractional-scale app-bar width slack, retaining genuine narrow-window overflow. |
| [#486](https://github.com/storytold/designcraft/pull/486) | @aiongg | `63375bc84d3ce976ca7b1321f09ab142f85d25f0` | Filled Selection and hollow Direct Selection vector arrow icons. |
| [#503](https://github.com/storytold/designcraft/pull/503) | @aiongg | `7b903cf6b16b5baf5e43699b61875961900dcc11` | Shift+W Presentation toggle, remembered return mode and consumed Escape preserving tool/selection. |
| [#491](https://github.com/storytold/designcraft/pull/491) | @noah1400 | `0a9bfa57352e4872562487415ab1e3e4bf688f50` | Alt/Option wheel pointer zoom with modifier latching through smoothed/trackpad motion; retain existing command/pinch and plain wheel routes. |
| [#499](https://github.com/storytold/designcraft/pull/499) | @aiongg | `a73d106ab45158386ea2bb493412ec04b80de9a2` | Interactive command search with localized/English/id matches, keyboard navigation, enabled states, shortcut hints and restored focus; cap query and result count. |
| [#472](https://github.com/storytold/designcraft/pull/472) | @moppeeer | `3943e4c8b74956f659fab5c0731a209033752fd9` | Transformed text and group bounds for incremental repaint; depth cap falls back to full repaint. |
| [#505](https://github.com/storytold/designcraft/pull/505) | @KnottyDyes | `9b7b5d0e9d976559007441bb1ee8ea61e06c1f95` | Bounded raw ICC profile registration, profile IDs/classes/channel counts; profile-picker and PDF/X integration remain outside these foundations. |
| [#475](https://github.com/storytold/designcraft/pull/475) | @noah1400 | `40e8dbc6fb358a12cdd5099acef10d8552107d87` | Case-insensitive CLI export dispatch with native save-a-copy, PDF options validation and early unsupported-format errors. |
| [#500](https://github.com/storytold/designcraft/pull/500) | @aiongg | `47d17ca6f50883eaf1861a10fabb78b0816d9a12` | Custom text-wrap paths in composition, resize/bake and IDML with bounded finite geometry; retain existing contour modes. |
| [#487](https://github.com/storytold/designcraft/pull/487) | @aiongg | `56c8ed96673efaf553d44ac0f9f0ad6cc72182c3` | Code-drawn tool cursors, including pen add/delete/close/convert; preserve linking cursors and WASM/system fallbacks. |
| [#463](https://github.com/storytold/designcraft/pull/463) | @KnottyDyes | `97b753619a51f31e5786434bed5d6499e055a62b` | Document paragraph hyphenation defaults and object fitting parameters. |
| [#469](https://github.com/storytold/designcraft/pull/469) | @moppeeer | `90407a892aa6feb2223a3c4968f400b4daa63c6e` | Page presets, orientation, linked margins, bleed/slug and Adjust Layout in Document Setup; preserve untouched margins. |
| [#467](https://github.com/storytold/designcraft/pull/467) | @sanyamsingh001 | `004668eba87ab0686e95dde93e1147e52995deb9` | Explicitly edited dialog fields, captured text-frame targets, mixed-selection preservation, partial inset updates, table style creation and Shift+Tab. |
| [#468](https://github.com/storytold/designcraft/pull/468) | @soro-82 | `2b55f2ed0d9e76c8f0327194e850fbc21fe10a29` | Remaining Create Guides and Margins/Columns UI with document ruler units and page/master-side targeting; retain stronger existing explicit master inheritance. |
| [#477](https://github.com/storytold/designcraft/pull/477) | @luniiya | `fe21f3c16cd38b67f84f34b30ac9a85b529e353e` | Favorite theme stars and header appearance shortcuts, adapted to Pagenor’s existing System/light/dark brightness settings. |
| [#466](https://github.com/storytold/designcraft/pull/466) | @moppeeer | `26e75c8608f3fe6fa33c6ae2c655edc747af9cb9` | Font picker filters, sample sizes, recent fonts and expandable styles. Use small table metadata for filters; visible rows alone load preview faces. Expanded styles, classification filters, keyboard focus and all font fields/workspaces now pass interaction tests. |
| [#462](https://github.com/storytold/designcraft/pull/462) | @sanyamsingh001 | `1e526c66bdaf54b2e45ce64a69326e81c026dad4` | Bounded styled Markdown import and style mapping, retaining unsupported constructs as literal text with warnings. |
| [#459](https://github.com/storytold/designcraft/pull/459) | @sanyamsingh001 | `caa6421a3e9079a557daf4816cdc15e50b4d3932` | Respect primary-chain capacity removal during cleanup and avoid deleting groups containing empty tail frames; retain existing reflow decisions. |
| [#482](https://github.com/storytold/designcraft/pull/482) | @xristosmoug | `af9bddf34d0c2d978524c5b759e530e2c230c2ef` | Strip build-time extended attributes before signing, preserve signed DMG contents and verify the finished image plus installed copy. Seven package-helper success/failure scenarios pass; finished local artifact verification is recorded below. |
| [#478](https://github.com/storytold/designcraft/pull/478) | @aiongg | `3d7638295a04ffda7c23bff445111b578df78a02` | Dispatch canvas pointer, key, clipboard and IME input in arrival order, including click/drag promotion and final release coordinates. Retain Pagenor multi-click selection, inline dead-key input, Alt-hand gestures, selected-text Alt-copy and platform shortcut distinctions. |
| [#460](https://github.com/storytold/designcraft/pull/460) | @sanyamsingh001 | `ab80e7f5b03b26787338727e6703ef362c8a2caf` | Resize/snapping preview agreement for fixed/scaled strokes, reflections and reachable dimension matches; retain centered resize and baseline/guide competition. Pointer/snapping and engine regression suites pass. |
| [#362](https://github.com/storytold/designcraft/pull/362) update | @aiongg | `0437ab27f5d3d6eb443d671d8194f93b7a741b9c` | Interpret PDF content bounds without rasterizing a 2048px preview; arbitrary clip/mask bounds remain conservative. |
| [#421](https://github.com/storytold/designcraft/pull/421) update | @aiongg | `8f30ba1c55342caff2bdd4af76c23b3614eeb602` | Avoid nested whole-line RTL ActualText; keep per-run cluster text and decoration. Regression and Poppler checks are recorded below; exact viewer spacing/mark order remains a follow-up. |
| [#427](https://github.com/storytold/designcraft/pull/427) update | @MaridWSH (earlier @skylartaylor) | `d0120bf6a642beb92e31e026fc8a9bc64778bc13` | Preserve dash spacing in optical kerning, retaining ordinary Latin kerning. |

## Validation

- All seven `cargo xtask ci` gates passed on the implemented batch: formatting, workspace check,
  strict Clippy, tests, asset attribution, crate layers and WASM. 2,306 tests passed, seven ignored,
  no failures; twenty WASM packages checked. The final 0.6.0 run also passes all seven gates with the same totals.
- Font menu interaction tests cover all font fields/workspaces, filters, focus and expanded styles.
  Metadata filters retain lazy installed faces; only visible rows need preview glyph data.
- Ordered-input tests cover multiple pointer/key events, modifiers, selected-text copying, drag
  thresholds, final pointer-up geometry and native inline IME. Existing Telugu dead-key input passes.
- Resize tests cover fixed/scaled strokes, flips, side/corner handles, unreachable size matching,
  center and baseline/guide behavior. Deliberately removed pages/frames remain removed through
  unrelated edits and undo; real primary-chain text/layout edits can resume automatic growth.
- Cross-target `cargo check --locked` passes for fonts, document, composition and tools on Windows
  GNU and Linux GNU. These are compile checks, not native Windows/Linux GUI or printer validation.
- Native macOS computer-use checks ran in a separate app bundle/profile. Document Setup fits at
  1440×900; font previews and expanded Mandali styles are visible; Mandali Regular renders the
  synthetic Telugu text and clears the missing-glyph error. Theme favorite stars work; closing the
  disposable document returns to Welcome with editor tools hidden. No owner documents were used.
- The offscreen UI-shot helper built but could not obtain a graphics adapter; native visual checks
  above replaced that attempt. No offscreen screenshots are claimed.
- The exact DMG verification helpers passed seven mocked scenarios: success, mounted signature
  failure, copied signature failure, copy failure, exhausted attach retries, exhausted detach retries
  and notarized Gatekeeper rejection. Actual local package results are recorded below.
- Real subprocess recovery tests passed without taking over the owner’s active sessions.
- PDF regression tests validate glyph Unicode, positions and absence of nested ActualText. A native
  Arial Arabic/Hebrew probe exports tagged and untagged PDFs without missing glyphs; Poppler retains
  every non-space source codepoint. Poppler’s inferred spacing and combining-mark order do not
  exactly match the source, so exact external-viewer copy semantics remain a compatibility follow-up
  for the additional #421 update. This probe is not claimed as perfect round-trip text copying.

## Recovery compatibility

A run holds its lock until process exit. Launch creation/takeover uses a root coordination lock
so a partially created folder cannot be mistaken for a crashed run. Failed locking disables
automatic recovery for that launch and shows a startup status error; it never writes to a
shared fallback folder. Deleted/replaced locks refuse checkpoint writes rather than reclaiming
someone else's namespace. Unknown files and stale lock directories remain untouched.

Older flat recovery entries carry no liveness lock and may belong to the owner's still-running
older app. They remain intact. Explicit `file.recovery.open {"dir":".../Recovery"}` can recover
those files after the older app is closed; automatic takeover is limited to released, locked
run namespaces. Recovery retains the existing per-document checkpoint ownership and reopening
semantics. Metadata reads are capped at 64 KiB and primary reads at the existing native archive
budget of 512 MiB, with actual-read checks.

## Colour parameter compatibility

If every component is within 0–1, RGB arrays and CMYK objects retain fractional interpretation.
If any RGB component is above 1, all channels use 0–255. If any CMYK component is above 1, all
channels use percentages. Thus C=0.7, Y=12.2 means C=0.7%, Y=12.2%. An all-small percentage colour
remains ambiguous under the existing API; callers can supply fractions. Gray support remains.

## Broader queue and distribution follow-ups

- #124 anchored text is already covered by the existing implementation: body/footnote composition,
  vector painting, PDF export, IDML round trips, bounded anchored traversal and overset preflight
  all have passing regressions. Do not rewrite this subsystem.
- #501 retains Pagenor’s stronger explicit master inheritance and guide/snapping integration;
  remaining direct Properties-panel master settings integration belongs to the broader queue.
- #442’s newer macOS movable-titlebar integration and #291’s bounded Affinity backend/UI remain
  in the broader approved queue; neither is claimed as part of these twenty completed proposals.
- Native Windows/Linux execution, printing, universal macOS artifacts and signed/notarized public
  distribution remain release validation work. The local arm64 package does not replace those gates.
- #514 (MCP replay), #513 (Linux object clipboard route) and #464 (XDG font roots) already have
  local counterparts. Excluded/deferred proposals remain as recorded in the review, including new
  CJK features, Italian and upstream branding/workflow swaps.

## Local artifact

- Release-profile native arm64 app and CLI built with `cargo build --release --locked -p pagenor
  -p pagenor-cli`; both executables report 0.6.0. No dependency upgrades.
- `pagenor-0.6.0-macos-aarch64-local.zip` and its extracted `Pagenor.app` are available in the chat’s
  `outputs/pagenor-0.6.0-macos-arm64/`. The app is ad-hoc signed for local testing, not notarized.
- Strict deep codesign validation passes on the staged app, delivered app and ZIP-extracted app.
  Plist version/identifier and owner app/document ICNS bytes match the expected assets; `.pgnr`
  and legacy `.pagenor` UTI declarations retain `PagenorDocument.icns`.
- The optimized CLI successfully exported uppercase `.PDF`, `.PGNR` and `.IDML` smoke fixtures.
- Real DMG creation failed all five bounded attempts with `Device not configured`. No successful
  finished-DMG, notarization or Gatekeeper result is claimed. The verified ZIP is the local fallback.
  The new finished-image verification code remains covered by the seven helper scenarios above.
- `git diff --check` and packaging shell syntax pass. Windows/Linux core compile checks were
  repeated successfully at 0.6.0. No installed owner bundle was replaced and no GitHub release,
  push, tagging or remote mutation occurred.

