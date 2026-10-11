# Building Pagenor release packages

The source version is the workspace Cargo.toml value; see
[pagenor/VERSIONING.md](pagenor/VERSIONING.md). Version 0.3.0 is a development preview, published to GitHub at the owner’s
2026-10-10 request. Its release notes distinguish the verified Apple-silicon Mac
development bundle from pending Windows/Linux/FreeBSD packages. Keep routine
development local and run the native matrix at the completed milestone.

## Local prerequisites and commands

All build scripts share `packaging/env.sh`, use the Pagenor IDs/owner assets and
read the workspace version. `PAGENOR_VERSION` is a package override; when changing
binary versions use `cargo xtask version set` first so executable metadata matches.
Releases embed the pinned optional craft-fonts input with its font licences.

| Host | Build |
|---|---|
| macOS | `packaging/macos/package.sh --arch universal` (both Rust targets; signing/notarization credentials optional) |
| Windows | `packaging/windows/package.ps1 -Arch x64` (or `arm64`/`x86`; WiX 5.0.2 and matching Rust target) |
| Linux | `packaging/linux/package.sh` (nfpm, appimagetool and zsyncmake; glibc baseline set by the builder) |
| Linux Flatpak | `packaging/linux/flatpak-bundle.sh <release-tarball>` (python3, flatpak, flatpak-builder, SVG pixbuf loader) |
| FreeBSD | `bash packaging/freebsd/package.sh` (bash, Rust and the VM build dependencies listed in release.yml) |
| Web | `packaging/web/package.sh` (trunk and wasm32 target) |

Linux packages are built natively for x86_64/aarch64 on Ubuntu 22.04 runners. The
Flatpak job repackages each corresponding tarball on its matching Ubuntu 24.04
runner, using the same binaries as the AppImage/deb/rpm. The archive's release
root and both ELF architectures are checked before extraction. Paths, file types,
entry counts and expanded sizes are bounded; links are rejected. Limits are
20,000 entries, 2 GiB/file and 4 GiB expanded total. `--stage-only` checks/stages
without installing/running Flatpak; `--no-test` skips the final user installation
smoke check. Normal bundle builds fetch the runtime/SDK from Flathub.

The RISC-V job cross-compiles `riscv64gc-unknown-linux-gnu` on Ubuntu 24.04 with
matching riscv64 multiarch libraries, GNU C/binutils and explicit target compiler
settings for the AI backend's `ring` dependency. It emits **tar.gz only**, with a
2.39 glibc baseline, and calls the CLI under QEMU. This is not native RISC-V GUI
validation. Flatpak/AppImage/deb/rpm remain the existing x86_64/aarch64 paths.

For a configured cross toolchain:

```sh
CROSS_ARCH=riscv64 CROSS_TARGET=riscv64gc-unknown-linux-gnu \
CROSS_COMPILE=riscv64-linux-gnu- EMULATOR=qemu-riscv64 \
packaging/linux/package.sh --formats tar
```

Set the Cargo linker, target C/archiver and pkg-config environment as in
`release.yml`. `--skip-build` selects the matching target's release directory.
Without `EMULATOR`, a cross package skips CLI execution rather than running a
foreign binary directly. Unsupported format or architecture/target combinations
fail before staging. The local packaging contracts use synthetic tools/binaries;
the real cross job and native GUI remain milestone checks.

The source and bundle Flatpak manifests share runtime, app ID and permissions.
Both grant X11, including XWayland, because the desktop host currently selects
X11 for drag delivery. Documents/Pictures and chooser portals cover common file
access; network and home/host access are not granted. Optional online AI/control
use requires an explicit user network override; it is not enabled by packaging.
The source manifest also installs pagenor-cli. A Flathub submission remains separate.

AppImage builds embed `gh-releases-zsync` update information for `appajid/pagenor`
by default or the builder's `GITHUB_REPOSITORY`. The `.zsync` describes its adjacent
AppImage and is uploaded with it. Old `.zsync` files are removed before rebuilding;
without zsyncmake, local packaging warns and leaves no stale update file. Release
CI requires a nonempty file and verifies the embedded update information.

FreeBSD ships a /usr/local-style tree, retaining desktop/MIME metadata, supplied
icons, software/provenance/branding licences and available embedded-font notices.
For an intended install, unpack the tarball into /usr/local after inspection.
`--dry-run` uses labelled synthetic binaries in the work directory and never
publishes those stubs into dist/release. It works on macOS/Linux for layout checks.

## Validation and publishing

Run `cargo xtask ci` and the packaging checks before a local commit:

```sh
python3 packaging/tests/release_packages.py   # PyYAML required for manifest checks
bash packaging/freebsd/package.sh --dry-run
shellcheck packaging/*.sh packaging/*/*.sh
actionlint
```

Packaging contracts use synthetic binaries and tool shims. They test archive
validation, architecture routing, owner update URLs, licence/layout contents and
manifest permissions. They do not create genuine Linux AppImages/Flatpaks or run
FreeBSD/Windows applications on macOS. Test the real artifacts on their native
hosts before describing them as working installers or publishing them.

At the milestone, the release workflow builds macOS, Windows, Linux, Flatpak,
FreeBSD and web artifacts, then creates a draft with SHA256SUMS.txt. Unsigned jobs
(version, Linux, RISC-V, Flatpak, FreeBSD and web) need no release secrets/environment;
macOS, Windows and draft publishing keep the restricted release environment.
A published version is never overwritten. The owner authorized the v0.3.0 development prerelease and current source upload
on 2026-10-10. Its known validation scope is recorded in the release notes; it
does not complete the native installer/signing/font/GUI phase exit gates. Do not
push the release branch again for an already published version: that pipeline
refuses to replace published artifacts. The owner lifted the version hold on 2026-10-11; local source is now 0.4.0.
This version has not been published and does not overwrite v0.3.0.

## Branded macOS installer

The DMG stages the committed Pagenor HiDPI background and Finder layout from
`packaging/macos/dmg/`, placing Pagenor.app beside an Applications link. The
mounted volume is named Pagenor to resolve the background alias; output files
retain their version/architecture. Generation tools are optional development
inputs, not release-job dependencies. See that directory’s README for portable
regeneration and attribution. Signing/notarization remains the existing flow.
