# Pagenor app icons and branding

Pagenor branding rights belong to Appaji Ambarisha. The supplied artwork is preserved
in `assets/brand/` from `Pagenor_SVG_Assets.zip` and
`Pagenor_Splash_About_Editable_SVG.zip` (2026-10-08). Branding rights are
reserved; see `LICENSE.txt`. Existing software and upstream notices remain applicable.

The supplied splash design appears briefly on every desktop launch, on wide
start screens and during web loading. The desktop launch splash lasts about
1.2 seconds from its first UI frame, including launches that restore or open
documents. It then gives way to the normal workspace; it has no sample progress bar.
The start-screen wordmark adapts to light/dark mode; About uses the supplied page
artwork with the actual build version, plus Contributors and Models tabs. The original
wordmarks contain text; runtime derivatives outline that text to preserve its
appearance on macOS, Windows, Linux and WASM without needing Arial installed.

`pagenor.svg` uses the supplied light app tile; `pagenor-small.svg` uses the small
variant. Dark and monochrome alternatives are retained in `assets/brand/`.
The tiles have a 192 × 192 viewBox. macOS adds a transparent margin to fit its icon grid.

Generated files:

- `pagenor-1024.png`: full-size tile.
- `pagenor-macos-512.png`: runtime macOS Dock icon.
- `pagenor.icns`: macOS bundle icon.
- `pagenor.ico`: Windows executable and installer icon (16–256 px).
- `hicolor/<size>/apps/org.pagenor.app.png`: Linux icons, including the shared
  Windows/Linux runtime icon at 256 px.
- `hicolor/scalable/apps/org.pagenor.app.svg`: Linux scalable icon.

Run `packaging/icons.sh` to regenerate with Cargo, the workspace resvg renderer,
and portable ICO/ICNS packers on any host.
Generated assets are checked into source control, so building Pagenor on another platform does not require these generation tools.

For local macOS development, run `packaging/macos/dev.sh` from the repository.
It builds the current executable and packages `pagenor.icns` into
`target/macos-dev/Pagenor.app`, then opens that bundle. Finder gets its icon from
the bundle; the running app also supplies `pagenor-macos-512.png` to eframe.
Rebuild and relaunch after changing icon assets. `--no-launch` only builds the
bundle; `--release` selects the release profile. App arguments follow `--`.

## Editing the artwork

The nine `Pagenor-*.svg` files are the exact supplied originals. The two
`pagenor-*-editable.svg` screens are editable runtime adaptations: decorative window
buttons and the sample loading bar are removed, branding attribution is updated,
and About's placeholder version is omitted so Rust draws the current Cargo version.
Edit these adapted screens when changing the running splash/About artwork.
`runtime-*.svg` files are generated derivatives with text outlined for consistent
rendering across platforms. Their names are deliberately different from the
originals on both case-sensitive and case-insensitive filesystems.
