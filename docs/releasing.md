# Building Pagenor release packages

The source version is the workspace Cargo.toml value; see
[pagenor/VERSIONING.md](pagenor/VERSIONING.md). The owner authorized native
Windows/Linux milestone builds and releases on 2026-10-12. Version 0.6.0 is a
desktop development preview; [distribution notes](pagenor/DESKTOP_RELEASE_060.md)
record the native evidence and signing/interactive limits. Keep routine work
local and publish versioned artifacts at milestones.

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

The focused desktop-release.yml runs on build/pagenor-*-desktop branches.
It builds unsigned Windows x64/x86/ARM64 and native Linux x86_64/ARM64 packages,
runs the full Windows/Linux quality gates, verifies MSI install/associations/
icons/export/uninstall and Linux/Flatpak execution, then publishes a development
prerelease only after all checks pass. Actions and the optional font input are
pinned. Publishing alone has contents-write permission; signing secrets and
paid Mac runners are not required. A duplicate version skips artifact rebuilding
and keeps source quality checks active. Published artifacts are never replaced.
Matching universal Mac packages can be appended under unique filenames with
their own checksum manifest after verification.

The broader release.yml retains Mac/Windows signing, FreeBSD, RISC-V and web
jobs and its restricted release environment. Those platforms and stable signed
distribution are not implied by the focused desktop preview. Source review is
[PR #3](https://github.com/appajid/pagenor/pull/3), separate from release
publication. Published v0.3.0 remains unchanged.

## Branded macOS installer

The DMG stages the committed Pagenor HiDPI background and Finder layout from
`packaging/macos/dmg/`, placing Pagenor.app beside an Applications link. The
mounted volume is named Pagenor to resolve the background alias; output files
retain their version/architecture. Generation tools are optional development
inputs, not release-job dependencies. See that directory’s README for portable
regeneration and attribution. Signing/notarization remains the existing flow.
