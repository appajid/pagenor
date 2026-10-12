# Pagenor 0.7.0 — desktop development preview

This milestone completes 26 selected upstream proposals and six incremental
updates, preserving Pagenor branding, architecture, lazy scoped fonts, inline
Indic input and .pgnr documents. [Per-change implementation and provenance](UPSTREAM_ADOPTION_070.md).

- UI: Properties Pathfinder, tint and Frame/Text targets, grouped View commands,
  document grid, Justification, content transforms, explicit page/spread targets
  and chronological split-view input.
- Publishing: curved corners, one-time image fitting, retained/inverted clipping,
  anchored text wrap, section markers, parent-page caches, continuous decorations,
  uppercase hyphenation and deletion-aware reflow.
- Text and import: saved blank final-paragraph formatting, scoped PostScript font
  lookup, preserved IDML family/style spaces, anchored inheritance, rotated spreads,
  CR/CRLF normalization and spacing-off round trips.
- Printing: Fit oversized sheets and explicit Original size, preserving printer
  duplex defaults; Pagenor user guides now describe the shipped application.

Native document format **5** reads older native documents, including .pagenor.
Older Pagenor versions reject new format-5 saves rather than discard their new
capabilities. Preserve older copies when sharing with older app versions.

Windows: x64, x86 and ARM64 MSI installers and portable ZIPs.
Linux: x86_64 and ARM64 AppImage, Debian, RPM, tar and Flatpak bundles;
archive/AppImage packages target glibc 2.35 or newer.
macOS: universal Intel/Apple-silicon DMG and CLI ZIP.

These are development previews. Windows installers are unsigned; Mac packages
are ad-hoc signed and not notarized. Interactive Windows/Linux editor and physical
printer validation remain separate work. INDD recovery is experimental; PageMaker
6.5/7 recovery requires the optional helper in [the import notes](PAGEMAKER_IMPORT.md).
CJK candidates, Warichu and additional interface translations remain deferred.

## Validation and distribution

All seven local Mac quality gates pass: 2,574 tests, zero failures, seven existing
ignores; twenty WASM packages. The native Mac UI was inspected using independent
QA profiles and synthetic documents, including Save/Open typing and print dry-runs.
The owner's running app and documents were left open.

The desktop matrix verifies native Windows/Linux quality, binary architectures,
MSI installation, separate document icons, .pgnr/legacy associations, installed CLI
PDF export and uninstall; Linux packages and Flatpak execution are checked.
Hosted results and matching Mac checksums are recorded after the builds complete.
Published 0.6.0 artifacts remain immutable. Source integration into the default
branch is tracked separately through a pull request; releases use their tagged
build-branch commit.

Pagenor branding rights belong to Appaji Ambarisha. Source retains MIT OR Apache-2.0;
package notices preserve upstream, asset and font provenance.
