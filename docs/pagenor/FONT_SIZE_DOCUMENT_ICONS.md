# Font-size menus and separate document icons — P09.11

Version remains 0.3.0. Changes and installation are local; no GitHub update.

The top font-size control, right-side Character panel, Properties typography
controls and floating text bar share these presets (points):
6, 7, 8, 9, 10, 11, 12, 14, 18, 21, 24, 30, 36, 48, 60, 72.
Custom sizes remain editable through the existing numeric widget and text command.

## Artwork and associations

Appaji Ambarisha's supplied `Pagenor-Document-Icon.svg` is preserved exactly in
`assets/brand/`. Branding rights remain reserved. Its generated SVG outlines the
header text so runtime icons do not depend on Arial. Document artwork is packaged
separately for macOS (ICNS), Windows (installed ICO) and Linux (MIME hicolor icons).
Both `.pgnr` and the legacy `.pagenor` association use document artwork.
The application, Dock, launcher and installer retain app artwork.

The Mac bundle's `CFBundleIconFile` points to `Pagenor.icns`; its document and
exported UTI declarations point to `PagenorDocument.icns`. The native format now
conforms to `public.data` and `public.content` instead of the inherited image
conformance. These declarations follow [Apple's document-icon reference](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)
and [UTI declaration reference](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/understanding_utis/understand_utis_declare/understand_utis_declare.html).
The Windows nonadvertised ProgId references the separately installed icon File ID,
as specified in the [WiX ProgId reference](https://docs.firegiant.com/wix/schema/wxs/progid/).
Linux uses the MIME icon declaration and bundled hicolor files from the
[shared MIME specification](https://specifications.freedesktop.org/shared-mime-info/latest-single/).

## macOS findings and native verification

Apple's native ICNS reader corrupted the previous small app-icon entries even
though extracting their embedded PNG bytes showed correct source colours.
The portable Rust packer now writes classic RGB plus separate alpha masks for
16/32 pixels and PNG Retina/large entries. A native regression failed before the
fix and passes after it: 16/32/64 pixels with transparent, translucent and opaque
pixels. Both final bundle ICNS files also decode correctly with `iconutil`.

Finder initially kept the old app icon for documents. Two earlier local test
bundles (0.2.1 and 0.3.0) still exported the same UTI with app artwork; macOS even
tracked them after renaming their directories. They are preserved as verified
ZIP archives in their original output folders; redundant unpacked copies are
removed. The final verified bundle is installed at `/Applications/Pagenor.app`.
Finder and the current user's icon/association services were refreshed without
resetting their databases. The precise contribution of each refresh is not
isolated, but final native observations establish the result:

- Top size dropdown and right-side Character dropdown show the complete list;
  the existing Udayam 36-point document remains unchanged during inspection.
- A fresh `.pgnr` verification copy and the original `Untitled-1.pgnr` show the
  supplied page artwork in both file-info headers and large previews.
- Login Items still shows the blue Pagenor app logo after the document check.
- Installed bundle signature verifies and its version is 0.3.0.

No per-document custom icon was added. The originals remain saved; verification
copies contain the same saved bytes. LaunchServices CLI registration was
unavailable in this tool environment (`-10822`); verification used native UI.
Native Windows/Linux install and visual display remain milestone-host checks.

## Automated verification

`cargo xtask ci`: all seven gates pass (format, check, Clippy, tests, assets,
layers, WASM); 1,474 tests pass, 3 ignored, 20 WASM packages checked.
Eight packaging contracts pass, including document MIME resources in synthetic
FreeBSD and AppImage packages. Modified Bash scripts pass syntax checks and
`git diff --check` passes. No unsafe code or new production dependency is added.

Regenerate all artwork with `packaging/icons.sh`, or only document assets with
`packaging/icons.sh --documents-only`.
