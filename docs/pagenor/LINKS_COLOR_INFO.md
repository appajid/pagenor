# Links colour information — P11.9

Local Pagenor 0.5.2 addition, 2026-10-11.

The Links panel includes a Color Mode column. Select a placed graphic to see
its colour mode and ICC Profile description below the list. Long link names
are shortened in the list and remain available through their tooltip.

The shared engine exposes `links.list.colorMode` and the read-only
`links.info {asset}` query returning `colorMode`, `iccProfile` and
`metadataSource: storedAsset`. Queries preserve document and undo state.

Metadata comes from the stored graphic, not the screen's RGB conversion.
CMYK JPEG/TIFF/PSD, RGB, grayscale, indexed and PSD Lab/bitmap/duotone/
multichannel headers are recognised. JPEG ICC segments, PNG profiles, TIFF
ICC tags, PSD/PSB profile resources and WebP ICC chunks expose embedded
profile names. ICC v2 ASCII and v4 multilingual Unicode descriptions are
supported. Missing raster profiles read Not embedded when supported; unreadable
or unsupported metadata reads Unknown or Embedded (unnamed or unreadable).
BMP/GIF profile inspection is unavailable. PDF/SVG/EPS read Vector / mixed
and Unknown profile; their potentially multiple internal colour spaces are
not inferred from a preview.

The query reads headers and bounded profile data, without decoding pixels.
ICC copies are capped at 4 MiB and marker/resource/tag traversal is bounded.
The UI reads profile information only for the selected asset and caches it
until document revision or selection changes. No new dependency, file-format
field, image conversion or document colour setting is introduced. The Rust
implementation is shared by macOS, Windows, Linux and WASM.

For imported INDD cached previews, the values describe the stored preview;
its colour mode/profile may differ from the original linked image. This is
stated in the panel. Modified/missing links retain metadata for the placed
copy until Update/Relink changes that copy. Saving and reopening preserves
metadata through the existing stored image bytes.

## Validation

Synthetic regressions cover CMYK source modes, segmented ICC reassembly,
PNG profiles/grayscale, TIFF/PSD profiles, Unicode descriptions, hostile offsets,
truncated input and read-only query/save/reopen behavior. A full-window UI
regression selects a link and checks that its mode/profile details appear.
All 1,912 Rust tests pass (six existing ignores). Formatting, workspace check
and Clippy pass. All seven `cargo xtask ci` gates pass, including assets,
layers and 20 WASM package checks (`CARGO_INCREMENTAL=0` to limit disk usage). The locked offline macOS release build succeeds; the ad hoc
signed bundle verifies and reports 0.5.2 with build local-0.5.2-links-color.
The app and document icons are unchanged.

Native inspection in a distinct bundle/profile created a disposable sample,
opened Links, and selected dusk-cover.png. The column shows RGB and the details
show Color Mode: RGB / ICC Profile: Not embedded. All four list columns and
selected-image details fit the current sidebar width without clipping. Existing
owner sessions were not replaced or closed. A quit attempt on the disposable
instance was blocked by the computer-use concurrent-state guard; its final
window state was not subsequently changed. Native Windows/Linux runtime tests
remain separate from the shared Rust/WASM compile checks.

No publication, /Applications replacement or closure of the owner's session.
