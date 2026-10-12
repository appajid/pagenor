# Pagenor macOS installer window

The installer uses a 660 × 400 pt background in a 660 × 532 pt Finder window,
allowing room for always-visible tab/path/status bars on newer macOS without
changing the user’s Finder preferences. It uses 128 pt icons, Pagenor.app at
(326, 205), and Applications at (574, 205). The light icon-label area works
with Finder’s black labels in either appearance. The arrow and artwork remain
outside the icon boxes, with at least 10 pt clearance.

`background.svg` is a self-contained vector source. The supplied Pagenor symbol
and logo belong to Appaji Ambarisha; branding rights are reserved under
`assets/brand/LICENSE.txt`. Its two captions are outlined from bundled Inter,
under `assets/fonts/OFL-Inter.txt`. It contains no text elements, external images
or fonts. `background.tiff` contains 1x/72 dpi and 2x/144 dpi representations.
`dmg-layout.DS_Store` contains the Finder view settings and background alias.

The mounted volume is named **Pagenor** so Finder can resolve that alias. The
DMG filename still includes the version and architecture. Only the committed
TIFF/layout are copied when packaging; no Python, Finder automation or mounted
disk is required in the signing job. The software/document icons and existing
signing/notarization steps are retained.

## Regeneration (macOS, Windows or Linux)

```sh
cargo build -p xtask
python3 -m venv /tmp/pagenor-dmg-tools
/tmp/pagenor-dmg-tools/bin/pip install pillow==12.0.0 ds_store==1.3.3 mac_alias==2.2.3
/tmp/pagenor-dmg-tools/bin/python packaging/macos/dmg/generate.py
```

On Windows, use the virtual environment’s Scripts/python.exe. Set
`PAGENOR_SVG_RENDERER` to the xtask executable when using a custom Cargo target
directory. Regeneration uses Pagenor’s existing resvg renderer; text is already
outlined, so installed fonts do not affect the output. Keep volume/window/icon
constants in generate.py and package.sh consistent. The generated files use a
fixed date and contain no developer account, local path or disk UUID. Verify
byte-for-byte reproduction when changing artwork or generator dependencies.

Generator and staging approach adapted from upstream PR #131 by @XusBadia
(head ec42e542e909ba0ff75a4efb9a95012be11124ee), under MIT OR Apache-2.0.
The upstream installer artwork and trademarks are not used. Provenance and
licence details are in PAGENOR_UPSTREAM.md, ASSETS.md and LICENSE.txt here.
