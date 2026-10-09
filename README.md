# Pagenor

**Professional Publishing & Page Layout**

Pagenor is an independent open-source project. Its upstream origin and pinned
revision are documented in [PAGENOR_UPSTREAM.md](PAGENOR_UPSTREAM.md).

The project retains the upstream MIT/Apache-2.0 licensing and attribution.
ArtCraft trademarks and brand assets are not used by Pagenor.

Pagenor's principal development direction is a professional **Indic Publishing Engine**
covering major Indic scripts and multilingual publishing.

Source and issue tracking: [appajid/pagenor](https://github.com/appajid/pagenor).

## The sample magazine

Every page below was laid out by Pagenor from code (`crates/engine/src/sample.rs`) and exported
by its own renderer. Try it yourself with `File → New → Sample Document`, or start the app with
`--sample`.

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/page-cover.png" alt="Magazine cover for The Spring Issue, No. 01: a dusk landscape of layered purple hills under a large pale sun, with the white serif headline The Quiet Art of Layout and an italic deck below" width="100%"><p align="center"><sub><b>Cover.</b> A full-bleed graphic frame, display type and an italic deck.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/page-2.png" alt="Feature opener titled Notes on the Grid, with an orange FEATURE · DESIGN kicker over a rule, an italic deck, a wide landscape picture with a small caption, and two columns of justified body text above a QUARTERLY folio" width="100%"><p align="center"><sub><b>Styles.</b> Kicker rule, headline, deck, caption and justified two-column body.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/page-3.png" alt="Three-column page of justified, hyphenated body text that wraps around a shaded pull quote reading The best layouts disappear. What remains is the story., with a gradient rule and SPRING ISSUE folio at the foot" width="100%"><p align="center"><sub><b>Threading, columns and text wrap.</b> One story flows through three columns and around the pull quote.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/page-4.png" alt="Coming Next page titled Color, Ink and Paper on a plum background, with four swatch circles labeled Ink Plum, Sunset, C=100 M=0 Y=0 K=0 and Paper Warm, above a short paragraph of body text" width="100%"><p align="center"><sub><b>Swatches.</b> Named colors and a CMYK process swatch, laid out as a palette.</sub></p></td>
</tr>
</table>

## Why Pagenor

- **Familiar.** InDesign's layout, tools, menus, panels and shortcuts: spreads and parent pages,
  frames and threaded stories, the Control panel, paragraph and character styles, swatches, text
  wrap and more. If you know InDesign, you already know how to use it.
- **Beautiful type.** A Knuth–Plass paragraph composer (plus single-line), dictionary hyphenation
  (the public-domain Moby word list plus our own trained patterns), word, letter and glyph-scaling
  justification, keeps, optical margin alignment, columns, baseline grid, tabs, rules and shading.
  Line breaks are identical on screen and in PDF.
- **Fast.** Multithreaded SIMD rendering (vello_cpu), copy-on-write documents with O(1) undo
  snapshots, and cached composition.
- **Open.** A documented native format, IDML import and export, PNG and editable PowerPoint export, and PDF on the roadmap.
  No subscription, no licence server, no telemetry.
- **Agent-native.** Every menu item, tool gesture, panel control and dialog can be driven over a
  JSON control channel and an **MCP server**, so Claude and other agents can lay out and edit
  documents like a designer. See [`docs/control-protocol.md`](docs/control-protocol.md) and
  [`docs/mcp.md`](docs/mcp.md).
- **Everywhere.** One Rust codebase for desktop and the web.

## Quick start

Pagenor saves new native documents as **`.pgnr`**. Existing `.pagenor` documents
remain readable. Both extensions use the same native ZIP format, with embedded
assets and document data; no conversion is needed to open an older document.

On macOS, build and launch a local app bundle so Finder and the Dock use the
current Pagenor icon:

```sh
packaging/macos/dev.sh                              # current debug build → target/macos-dev/Pagenor.app
packaging/macos/dev.sh -- --sample                  # pass arguments to the app
packaging/macos/dev.sh --release --no-launch        # build a local release bundle without opening it
```

Quit this local app before rebuilding. The launcher uses the same icon and bundle
metadata as release packaging; it signs locally without making a DMG or contacting
GitHub. Windows and Linux keep their existing executable/desktop icon packaging.

```sh
cargo run --release -p pagenor                         # desktop app (start screen)
cargo run --release -p pagenor -- --sample             # open the sample magazine
cargo run --release -p pagenor -- --sample --control 7979   # + JSON control channel
cargo run --release -p pagenor-cli -- run --sample --all-pages out/       # headless: render every page to PNG
cargo run --release -p pagenor-cli -- commands         # list every command
cargo xtask ci                                             # fmt, clippy, tests, assets, layering, wasm
```

Japanese text (UI and documents) uses fonts from
[storytold/craft-fonts](https://github.com/storytold/craft-fonts), an optional build input (font
files are never committed here; see craftrules
[`standards/fonts.md`](https://github.com/storytold/craftrules/blob/main/standards/fonts.md)):

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$PWD/../craft-fonts" cargo run --release -p pagenor   # absolute path
```

Without it, Japanese falls back to the system's fonts (none on the web). Release builds always
include it.

The Windows desktop host selects DirectX 12 by default. An explicit
`WGPU_BACKEND` overrides that choice for graphics troubleshooting, for example:

```powershell
$env:WGPU_BACKEND = "vulkan"
& .\pagenor.exe
Remove-Item Env:WGPU_BACKEND
```

For CLI rendering, put `--page`, `--scale` and `--pdf-options` before the
`--export` or `--all-pages` that should use them. A trailing export option is an
error rather than silently ignored.

To drive a running app, explicitly start its control channel and send authenticated
JSON lines to `127.0.0.1:7979`. The app and clients use the same random
`PAGENOR_CONTROL_TOKEN`; the app generates one if unset. The protocol is described in
[`docs/control-protocol.md`](docs/control-protocol.md).

### Desktop logs

Desktop diagnostics go to stderr and `logs/pagenor.log` beside the per-user settings.
Each launch keeps two previous logs; each file is capped at 16 MiB. `RUST_LOG` controls
levels (default `warn,pagenor*=info`). `--version` leaves logs untouched. See
[locations, limits and behavior](docs/pagenor/DESKTOP_LOGGING.md).

### Web

```sh
cd apps/pagenor-web && trunk build --release          # → dist/web (serve it with any static server)
cd apps/pagenor-web && trunk serve --release          # http://127.0.0.1:8767
```

You need [trunk](https://trunkrs.dev) and the `wasm32-unknown-unknown` target. The same app runs
through eframe's web runner on WebGPU, falling back to WebGL2 (`?webgl` forces it; `?sample` opens
the sample magazine). Open and Place use the browser's file picker, and dropping files works too.
Save and Export download the file. The web build has no control channel.

## Architecture

Pagenor is an engine-first Cargo workspace with enforced layering (`cargo xtask layers`). The
egui frontend is a separate crate, so the UI can be swapped without touching the engine.

| Layer | Crates |
|---|---|
| L0 | `geom` (paths, units & measurement parsing, corner options) · `color` (CMYK/RGB/Lab, swatches, tints, gradients) |
| L1 | `doc` (spreads, pages, parents, layers, frames, stories, styles) · `fonts` (font DB, shaping, outlines) |
| L2 | `compose` (the text engine) |
| L3 | `render` (vello_cpu) |
| L4 | `tools` (pointer events → commands + overlays) |
| L5 | `engine` (session, history, command registry) |
| L6 | `ui-egui` (InDesign-style UI, control channel) |
| L7 | `apps/pagenor`, `apps/pagenor-cli`, `apps/pagenor-web` |

- **Status and milestones:** [ROADMAP.md](ROADMAP.md)
- **Development phases and audit findings:** [PHASE_STATUS.md](PHASE_STATUS.md) and [issue register](docs/pagenor/ISSUES.md)
- **Contributor and agent rules** (clean-room, asset policy, quality gates): [`AGENTS.md`](AGENTS.md)
- **Bundled assets:** every one is listed with its licence in [`ASSETS.md`](ASSETS.md)
- **App icon and logo:** Appaji Ambarisha’s Pagenor page-fold symbol and blue wordmark; see [`assets/app-icon/`](assets/app-icon/README.md)

## License and credits

Pagenor is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Original upstream copyright and required notices are preserved in [NOTICE](NOTICE)
and [PAGENOR_UPSTREAM.md](PAGENOR_UPSTREAM.md).

Bundled fonts, icons, images and other assets keep their own licences; each one is listed
with its author, source and license in [ASSETS.md](ASSETS.md).

The bundled fonts, and the craft-fonts fonts embedded by release builds, are under the SIL Open
Font License; all UI icons are drawn in code and are original.

Pagenor branding belongs to Appaji Ambarisha. Branding rights are reserved under
[assets/brand/LICENSE.txt](assets/brand/LICENSE.txt); the software remains open source.

<sub>Adobe, Photoshop, Illustrator, Premiere Pro, Lightroom, Acrobat, After Effects and InDesign are trademarks or registered trademarks of Adobe Inc. in the United States and/or other countries. Pagenor is an independent, open-source project and is not affiliated with, sponsored by or endorsed by Adobe Inc.; these names are used only to describe the workflows it is compatible with.</sub>
