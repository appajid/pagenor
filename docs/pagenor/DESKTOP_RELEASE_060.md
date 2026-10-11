# Pagenor 0.6.0 — Windows and Linux preview

This milestone carries the shared 0.6.0 editor, publishing and import improvements
to Windows and Linux. See [implementation notes](UPSTREAM_ADOPTION_20261012.md)
for the reviewed upstream proposals, Pagenor adaptations and remaining limits.

Windows packages: x64, x86 and ARM64 MSI installers and portable ZIPs.
Linux packages: x86_64 and ARM64 AppImage, Debian, RPM, tar and Flatpak bundles.
Linux binaries target glibc 2.35 or newer. Flatpak includes its runtime reference.

These are development preview builds. Windows installers are unsigned; Windows
may show a publisher warning. Proprietary PageMaker recovery requires the optional
helper described in the PageMaker import notes. INDD recovery remains experimental.

The desktop workflow builds every package from one source commit, checks native
Windows/Linux quality gates, verifies Windows binary architecture and MSI install,
document associations and separate document icons, runs installed CLI PDF exports,
and checks Linux packages and Flatpak execution before publishing. SHA256SUMS.txt
accompanies the downloads. Automated checks do not replace interactive testing on
each desktop, printer and proprietary font configuration.

## Packaging corrections (P11.12)

- Windows legacy `.pagenor` files share the `.pgnr` document ProgId without
  duplicate WiX MIME/open-verb registry entries. Both use the document artwork.
- The HTTPS data-merge fixture explicitly supplies its trust root to curl in
  unit-test binaries, supporting OpenSSL, GnuTLS and Schannel. An additional
  untrusted-certificate check preserves production certificate validation.
- The EPS relink fixture constructs nested paths portably on Windows.
- Windows MSI and portable packages include matching software, provenance,
  owner-branding and bundled/optional font notices.
- Existing full-platform signing and release workflows remain available; the
  desktop milestone workflow uses pinned actions and requests no signing secrets.

Pagenor branding rights belong to Appaji Ambarisha. Application code retains its
MIT OR Apache-2.0 licence; package notices preserve upstream and font provenance.
