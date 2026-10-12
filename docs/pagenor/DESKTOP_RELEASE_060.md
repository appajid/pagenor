# Pagenor 0.6.0 — desktop development preview

This milestone carries the shared 0.6.0 editor, publishing and import improvements
to Windows and Linux. See [implementation notes](UPSTREAM_ADOPTION_20261012.md)
for the reviewed upstream proposals, Pagenor adaptations and remaining limits.

Windows packages: x64, x86 and ARM64 MSI installers and portable ZIPs.
Linux packages: x86_64 and ARM64 AppImage, Debian, RPM, tar and Flatpak bundles.
Linux binaries target glibc 2.35 or newer. Flatpak includes its runtime reference.
macOS: universal Intel/Apple-silicon DMG and CLI ZIP, ad-hoc signed and not notarized.

These are development preview builds. Windows installers are unsigned; Windows
may show a publisher warning. Proprietary PageMaker recovery requires the optional
helper described in the [PageMaker import notes](https://github.com/appajid/pagenor/blob/v0.6.0/docs/pagenor/PAGEMAKER_IMPORT.md). INDD recovery remains experimental.

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
  The synthetic OpenSSL server uses binary file reads to serve PNG bytes
  unchanged on Windows. Production trust and revocation policies remain unchanged.
- The EPS relink fixture constructs nested paths portably on Windows.
- Windows MSI and portable packages include matching software, provenance,
  owner-branding and bundled/optional font notices.
- Recovery-directory setup compiles without Unix-only mutable bindings on Windows;
  Unix recovery directories retain their private permission mode.
- IDML links normalize Windows drive, extended-length and UNC file paths into
  escaped URI paths. UNC import retains the server/share authority. The package
  round-trip test compares canonical file identities on each host.
- Synthetic Mac packaging tests mock `xattr` alongside signing/plist tools so
  Linux can check staging without macOS utilities.
- Once a version is published, further source checks preserve its existing
  artifacts and skip rebuilding/replacing that release.
- Existing full-platform signing and release workflows remain available; the
  desktop milestone workflow uses pinned actions and requests no signing secrets.

Pagenor branding rights belong to Appaji Ambarisha. Application code retains its
MIT OR Apache-2.0 licence; package notices preserve upstream and font provenance.

## Build and validation

Source: `0ed0c53af1ae748443082e5d94fedd9fe9abacc4`.

Windows/Linux native build and install evidence: [desktop release run](https://github.com/appajid/pagenor/actions/runs/38180716355).
Windows/Linux quality and Rust 1.95 checks: [CI run](https://github.com/appajid/pagenor/actions/runs/38180720096).

Windows/Linux downloads are covered by `SHA256SUMS.txt`; the matching Mac files are covered by `SHA256SUMS-macos.txt`.

Native checks cover compilation, automated tests, package contents, CLI execution and installer operations. Interactive Windows/Linux editor and printer testing, Windows publisher signing and Mac notarization remain separate work. The owner’s current Mac session was left open.

The hosted Mac fixture initially rejected a Windows-only OpenSSL binary-file flag.
A [test-only follow-up in PR #3](https://github.com/appajid/pagenor/pull/3) restricts that flag to Windows; Unix file reads are already
binary. This changes no production code or packaged executable. The released
source already passes all seven local Mac gates with the build host's OpenSSL.
The separate FreeBSD workflow now provisions DejaVu before host-font tests;
FreeBSD VM validation is outside this desktop preview's release checks.

Repository visibility is public at the owner’s choice; standard native jobs executed successfully after that change. Default-branch integration is tracked separately in PR #3.
