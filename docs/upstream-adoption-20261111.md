# Pagenor 0.13.1 — reviewed fixes, 11 November 2026

Follow-up: eleven queued source scopes from this review are adapted locally in
**0.14.0 / P11.25**. See the [completion ledger](queued-adoption-20261111.md)
for source pins, compatibility decisions and final checks.

Status: implemented and validated locally as P11.24 / 0.13.1.
Base: `39ef376f0cf5888122367608887d642e76f62edd` (0.13.0).

The GitHub API scan covers **587 updated open/closed PR records since 31 October**,
including **261 new proposal numbers**, through **#2578**. A selected set of 65
candidate file/patch inventories was inspected against the local architecture;
this is not a claim that every changed line of every upstream PR was audited.
Thirty-three compatible bugfix scopes are adapted below. Source authors and
immutable heads identify provenance under the retained MIT OR Apache-2.0 licences.
Pagenor branding and owner asset rights remain unchanged. Vectnor is untouched.

## Adapted scopes

| PR | Author | Inspected head | Pagenor adaptation |
|---|---|---|---|
| [#2505](https://github.com/storytold/designcraft/pull/2505) | @Silvertree2010 | `55729a5128c5f1519b0c60ded8d5f6e44205e2c5` | Skip left/right/first-page RTF header/footer destinations. |
| [#2480](https://github.com/storytold/designcraft/pull/2480) | @Silvertree2010 | `5da2393e28c4504bc7416c94c3afd2dafccf6e14` | Reject signed/malformed XLSX row references. |
| [#2479](https://github.com/storytold/designcraft/pull/2479) | @Silvertree2010 | `8e341179226fb42c28da1aef982c9d3cee4f7a50` | Clamp negative new-document and saved-preset column gutters. |
| [#2477](https://github.com/storytold/designcraft/pull/2477) | @Silvertree2010 | `8ca6149325ebd28d9999e49cb8ca4ab6fd7ce06d` | Clamp expanded gradient midpoints; retain Pagenor PDF gradient path. |
| [#2464](https://github.com/storytold/designcraft/pull/2464) | @Silvertree2010 | `88a45f5d586ad38d79f9cee23b350d2a6455ef1d` | Apply typographic quotes even when a cross-reference tag is unterminated. |
| [#2442](https://github.com/storytold/designcraft/pull/2442) | @Silvertree2010 | `6365f98abebdff3615abf692f79f475fdcdeea11` | Keep rounded Lab hue below 360 degrees. |
| [#2437](https://github.com/storytold/designcraft/pull/2437) | @Silvertree2010 | `2fb2689efedda11909a8aa0dbb5c1869a0082a25` | Normalize reversed marquees and preserve flat-path/flat-marquee intersections. |
| [#2433](https://github.com/storytold/designcraft/pull/2433) | @Silvertree2010 | `f897077e331a52cd525332dfaa1fc46cb4d1fc23` | Respect degenerate base shapes in Subtract, Minus Back and Intersect. |
| [#2431](https://github.com/storytold/designcraft/pull/2431) | @Silvertree2010 | `42a288dec089fb807741f400f64655fbe344fab2` | Reject exponent notation in pica/cicero measure components. |
| [#2334](https://github.com/storytold/designcraft/pull/2334) | @y-az | `c739606e1a791854954914515de85d0752be4a8c` | Use bidi-neutral display identity for inline anchored objects. |
| [#2331](https://github.com/storytold/designcraft/pull/2331) | @y-az | `6978f3ff00444a9bf88326d298ac77694af4ef5a` | Thread an orphan IDML anchored text frame through its synthesized story key. |
| [#2330](https://github.com/storytold/designcraft/pull/2330) | @y-az | `3b7ecd8fac4e1edbd638e89820c6dafaba3a1a58` | Invalidate anchored text-story edits, including spread/anchor transitions. |
| [#2316](https://github.com/storytold/designcraft/pull/2316) | @DorianoMantegoso | `dcdd2df8fd6ee1982de955fb51e740733a9d15eb` | Write native document JSON compactly, retaining existing save/read budgets. |
| [#2572](https://github.com/storytold/designcraft/pull/2572) | @autocraft-bot[bot] | `78fb1bb3a40ecffcae595711992bc1164209c86e` | Refuse numeric perPage/arrange rather than silently ignoring the merge options. |
| [#2258](https://github.com/storytold/designcraft/pull/2258) | @dylanmtaylor | `fcca4096f932d6f49683d5e44eebbe9ba2203471` | Cap imported DOCX and Tagged Text style names at Unicode-safe character boundaries. |
| [#2254](https://github.com/storytold/designcraft/pull/2254) | @dylanmtaylor | `5724eac253a7d0a9dfdcc50d88a0334459b1aba1` | Bound nested SVG/SVGZ data within one parse; add bounded XML preflight and block external image file URLs while retaining scoped fonts. |
| [#2253](https://github.com/storytold/designcraft/pull/2253) | @dylanmtaylor | `8a2e3f1e49081fa2417b26c2a8044f1b0c92d1c5` | Charge decompressed entry bytes even when the read/CRC fails. |
| [#2220](https://github.com/storytold/designcraft/pull/2220) | @dylanmtaylor | `193d8e85f821f86eb303d712607ecf645b0984e0` | Escape logging control characters and indent continuations within the existing record-size cap. |
| [#2215](https://github.com/storytold/designcraft/pull/2215) | @dylanmtaylor | `0ebe712eee38a346d78e577df427eee1374f8286` | Bound named-instance enumeration per font file and index valid variable axes. |
| [#2179](https://github.com/storytold/designcraft/pull/2179) | @dylanmtaylor | `97bb82816e8036d53cd033bed3e5d5ff5101d863` | Refuse recursive/subroutine GREP using the pinned public AST; bound the nonempty-match helper without upgrading fancy-regex. |
| [#2153](https://github.com/storytold/designcraft/pull/2153) | @Silvertree2010 | `c065cec5171d253d8b0c626a2f4c811c79ce553c` | Include all single-anchor subpaths in path bounds. |
| [#2142](https://github.com/storytold/designcraft/pull/2142) | @TrevorBrown-dev | `5de0a0b806246fe5b783d4e5ec8ac1dfc60a08b6` | Custom text variables preserve numeric and boolean scalar values; reject object/array replacement atomically. |
| [#2141](https://github.com/storytold/designcraft/pull/2141) | @Silvertree2010 | `633bdb66bf2449b79055c7e3ab77ae531ee40bf9` | Reject signed data-merge record numbers. |
| [#2414](https://github.com/storytold/designcraft/pull/2414) | @aiongg | `81fc3d2c91c14f0a2211ccf2b1f715637fe404a7` | Read/write current Text*Inset cell attributes with legacy inset aliases. |
| [#2440](https://github.com/storytold/designcraft/pull/2440) | @aiongg | `578697b3ac89f2ae8dd164671ea24295f4c690b4` | Document Setup only sends edited width/height; bleed/margin changes preserve resized pages and masters. |
| [#2160](https://github.com/storytold/designcraft/pull/2160) | @4waymedia | `5f86f33c4274bd14f52a6f8f2a50a43e34e4ff11` | Open Color Picker with room for its full contents. |
| [#2162](https://github.com/storytold/designcraft/pull/2162) | @4waymedia | `885793f763527ef47d696b998b3638e7e8dbb683` | Remember the Color Picker component across openings/preferences; explicit opener choices win. |
| [#2273](https://github.com/storytold/designcraft/pull/2273) | @dylanmtaylor | `bc311542345dbe23477e844a8e406624854f8b08` | Refuse splitting more than 1024 cell owners before mutations. |
| [#2270](https://github.com/storytold/designcraft/pull/2270) | @dotCooCoo | `5b5732c118c7dab907f86bea92881529271d6c25` | Dim Convert Cell to Graphic Cell unless a table target exists. |
| [#2289](https://github.com/storytold/designcraft/pull/2289) | @sungkhum | `26d0f3d005b9f4e838b904ec6f3f9f6a87b49531` | Adapt source-cluster glyph selection for inverted canvas text so zero-length marks remain painted; retain the existing highlight-band geometry. |
| [#2546](https://github.com/storytold/designcraft/pull/2546) | @autocraft-bot[bot] | `25000ceb894e64f3db95d9ad8dc20806943b3fed` | Draw text threads between the actual port-square centers at the current zoom. |
| [#2547](https://github.com/storytold/designcraft/pull/2547) | @autocraft-bot[bot] | `2f1062c3d0881d94b00688764b9ffaa60bb14f11` | Show the loaded-text cursor when hovering text ports. |
| [#2527](https://github.com/storytold/designcraft/pull/2527) | @makesomethingshit | `1ed658a2945de6ffa7d6426f8841a5798f3dcd56` | Layers objects and footer follow the active master/document spread, preserving grouped rows and independent left/right navigation. |

## Compatibility decisions

Pagenor already supports Text Frame Options while typing through explicit actual-frame
UI targets. #2403's whole-thread backend fallback was not adopted; retain the existing
real-caret toolbar/taskbar and undo regressions.

- Bugfix version **0.13.1**. Native capability remains **7**, with `.pgnr` and legacy
  `.pagenor` reading. No new persisted document-model fields, language packs,
  CJK-only behavior, supplied fonts or icon changes are introduced.
- Preserve lazy/native/live font discovery, document scopes, dot-family filtering,
  inline Indic input, EOF formatting, left/right masters, welcome/recents, vertical
  grouped panels, save-all-on-quit and separate owner app/document icons.
- Existing external package identities/versions stay pinned. `quick-xml` and
  `flate2` become direct image-crate production dependencies; both were already in
  Cargo.lock. `unsafe_code = "forbid"` remains. Public GREP AST traversal replaces
  an inaccessible newer private field; no blanket dependency upgrade.
- Retain Pagenor's bounded native save/read policy for #2168 instead of upstream's
  uncapped stored assets. Compact JSON reduces inflated metadata without removing
  archive, entry, expanded-byte or document limits.
- #2534's embedded-ZIP preview API is absent here: Pagenor uses bounded asynchronous
  private import/render sessions, including INDD, and supports legacy JSON documents.
  Do not replace that workflow with an incompatible ZIP-only menu preview reader.
- #2470's signed numeric XML-reference guard already exists; retain it and add the
  upstream regression. #2311's Graphic.fill model field does not exist locally,
  so its newer image-coloring-only references are not copied into this model.
- #2289 contributes cluster glyph selection. Its newer RTL/warichu highlight-band
  rewrite is outside Pagenor's current canvas API and is not claimed here.
- Larger reviewed proposals need distinct model/platform validation: #2418 empty
  frame fitting, #2382 Keep with Previous, #2395 damaged-thread/legacy-vertical
  repair, #2333 feature-run shaping, #2412 white-overprint traversal, #2567 expanded
  Find Next workflow, #2124 shortcut zoom placement, #2507 PDF viewer-title options,
  #2544 per-placement relinking and the new PDF/XML/foreign-format feature families.
  They remain queued rather than importing whole refactored subsystems or changing
  native format fields inside this patch release. #2244's master-base graph API is
  not present in this older local master model. Other broad optimization patches
  (#2252, #2211, #2227, #2233 and related proposals) need per-path integration review.
- Do not adopt translation/CJK-only proposals, automatic update/distribution flows,
  release-branch merges, upstream branding, contributor identity edits or broad
  dependency changes as part of this local bugfix pass.

## Validation and local scope

All seven `cargo xtask ci` gates pass: **3089 passed, zero failures, seven existing
skips; twenty WASM packages; no compiler warnings**. The Mac workspace is tested
with generated fixtures and existing bundled fonts, without optional craft-fonts.

Three offscreen views were inspected: a master heading and its Layers row/footer,
the complete Color Picker, and Document Setup on a mixed-size document. Functional
UI tests cover component persistence/explicit overrides, master-view changes,
thread-line port-square geometry, caret-scoped Text Frame Options, and bleed-only
or equivalent-unit setup edits. The first QA script used an invalid spread shape;
it was corrected to `{kind: parent, index: 0}` and rerun before recording evidence.
No product API change was made for that script error.

Additional compile checks pass for composition, format, image and rendering
libraries on Windows GNU and Linux GNU. These are not complete platform app builds
or native interactive checks. Native Windows/Linux execution remains pending.

Compact logs, the corrected QA script, image hashes and validation metadata are in
`outputs/pagenor-pr-review-20261111` under the local Codex work folder. Temporary
builds and rendered images are removed after verification, respecting the owner's
cleanup request; no new app/test-release bundle, installation, native GUI launch,
GitHub upload or publication. Original user documents/fonts and Vectnor are untouched.
