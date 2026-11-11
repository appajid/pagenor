# INDD portfolio import — P11.7

Local Pagenor 0.5.2 correction, 2026-10-11. Appaji Ambarisha supplied
`Appaji_Portfolio2.indd` in the external drive's `Appaji_Portfolio Folder`,
then requested that conversion preserve the layer sequence.

## Cause and correction

- The binary model lists layers from back to front, just as IDML does. The
  writer reversed that list, causing backgrounds to cover the imported text
  and pictures. Keep the original order and each object's layer assignment.
- Retain empty user layers, including `Page Number`; exclude the identified
  private page-storage layer, which is absent from the companion IDML's UI
  layer list. Native document order is front to back, handled by the existing
  IDML importer. No change to the working IDML layer-order convention.
- Raster images use class `0x1702`, absent from the earlier PDF/EPS-only class
  list. Read their existing link, bounds, transform and proxy-blob chain using
  the existing graphic reader. Preserve image placement/crop geometry and
  local packaged-link rebinding. These recovered pixels are the INDD's cached
  previews; original linked full-resolution files are not automatically
  substituted for embedded previews during import.
- Multi-column object `0x263` preferences block `0x2D1` carries count at offset
  0 and gutter at offset 4. Preserve count/gutter instead of forcing all frames
  to one column. Validate count 1–40 and finite gutter 0–10,000 points; retain
  existing defaults for missing/corrupt preferences.

Two production INDD modules change; the shared IDML parser, composition engine
and renderer stay intact. No dependency or unsafe code is introduced. The
change is shared by macOS, Windows, Linux and WASM.

## Real-file verification

- INDD container version 21.6, save sequence 11; 22 document pages.
- All 57 image placements recovered, referencing 55 distinct nonempty assets.
- 18 text frames and 17 used stories; all 5,103 normalized text characters
  match the binary model exactly. No missing glyph IDs during composition.
- Six user layers preserved. Back to front: `Background`, `Image`,
  `Objects/Shape/Line`, `Text`, `Title`, `Page Number`. The Layers panel displays
  this list in the opposite direction, with the uppermost layer first.
- Ten document-font faces load from the owner's adjacent `Document fonts`
  folder with no font-file warnings. No fonts are redistributed.
- Native `.pgnr` save/reopen preserves text, thread frames, pages, assets and
  layer sequence. Selected pages render, and the diagnostic PDF exports.
- Source INDD, IDML, PDF, Links and fonts were read only. No owner files,
  images, font binaries or Adobe-produced data enter repository regression tests.

## Fidelity limits

The companion PDF confirms that cover text and photography are restored, but
source typography is not yet reproduced exactly: capitalization, tracking,
frame insets/baselines, master items, effects and other unsupported settings
remain subset limits. Three stories report overset after local reflow; the
filmography story fits after restoring its two-/three-column frames. All used
story text is retained even when overset. Master-layer definitions are retained
in the layer stack, but master-page artwork is not imported by this INDD reader.
Use the supplied IDML companion when source-layout fidelity is the priority.

The owner's running app session remains open. The corrected native document
and separately staged Mac test build are local deliverables, not a GitHub
release or an installation over the running app.

## Automated validation

Synthetic tests cover raster child/proxy geometry and links, layer order plus
empty-layer retention/internal-layer exclusion, multi-column preference bounds,
and native import/save with text above its background. All seven `cargo xtask ci` gates pass: 1,902 tests passed, 6 existing
ignores, no failures; 20 WASM packages. Native Windows/Linux execution remains
a separate validation step.

Whole-window `ui_shot` capture was unavailable in this sandbox: wgpu reported
`No adapter found`. The isolated native-engine CPU page renders were inspected;
no new desktop GUI run or interaction is claimed.

The INDD crate also cross-checks successfully for Windows x86_64 and Linux
x86_64. The Mac release build succeeds and the separately staged app reports
0.5.2 with build identity `local-0.5.2-indd-portfolio`; its ad-hoc signature
and separate application/document icon resources verify. No GitHub publication.
