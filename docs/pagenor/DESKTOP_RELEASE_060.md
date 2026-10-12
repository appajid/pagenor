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
  OpenSSL's synthetic HTTP server uses binary file reads to preserve the PNG's
  bytes on Windows, including its DOS EOF marker. This is a fixture correction,
  not an application transfer-policy change.
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

## Candidate status — 2026-10-12

Final source: `0ed0c53af1ae748443082e5d94fedd9fe9abacc4`.
[Native run](https://github.com/appajid/pagenor/actions/runs/38180716355) builds
and verifies Linux x86_64 AppImage/Debian/RPM/tar/zsync candidates and passes
all seven Linux gates (2,302 tests, no failures, seven ignored). All seven local
Mac gates pass (2,306 tests, no failures, seven ignored); Rust 1.95 checks pass.
The matching universal Mac DMG/CLI passes strict signatures in the finished
image and install-copy, architecture/icon/notice checks and native ARM PDF export.
It is ad-hoc signed and not notarized; native Intel GUI validation remains open.

GitHub refused final Windows x64/x86/ARM64 and Linux ARM64 jobs before startup,
reporting failed account payments or insufficient spending limit. Hosted Mac
quality was blocked too. Dependent Flatpak/publication jobs were skipped.
No v0.6.0 GitHub release has been published. Resolve account billing and rerun
failed jobs in the prepared native run before publishing.

Earlier Windows packages at `444f875` passed all three native installer,
association/icon, CLI export and uninstall checks but retain the subsequently
corrected IDML path issue. Linux ARM64/Flatpak packages at `c8c0d1b` passed native
package/CLI checks. These are intermediate snapshots, separate from the final
candidates. Source integration is in [PR #3](https://github.com/appajid/pagenor/pull/3).

The resume attempt again refused the Windows job before startup with the same
GitHub account billing message. The separate FreeBSD job's two font-discovery
failures exposed a fontless VM: Fontconfig was present but had no system faces.
The VM preparation now installs the FreeBSD DejaVu package, refreshes Fontconfig
and fails early if no host fonts are available. This CI-only change preserves
the lazy font catalogue and requires a fresh VM run before claiming BSD success.

The owner made the repository public on 2026-10-12; native desktop jobs now run.
The hosted Mac gate exposed an older OpenSSL rejecting the Windows-only
`-http_server_binmode` option. The test fixture now requests that option only on
Windows, retaining byte equality and untrusted-certificate checks on every host.
All seven local gates pass (2,306 tests, seven ignored; twenty WASM packages).
This test-only follow-up changes no production binaries; hosted revalidation
will follow the release run. The separate system LibreSSL was also checked and
does not support the fixture's loopback address syntax; this fixture requires
OpenSSL, as used by CI and the local build host.
