---
title: Install Pagenor
description: Install the desktop development preview on macOS, Windows or Linux.
order: 20
verified: 0.7.0
---

Download the package for your system from [Pagenor releases](https://github.com/appajid/pagenor/releases).
Use the version shown on that release. Packages include a SHA256SUMS.txt checksum manifest.

## macOS

Open `pagenor-<version>-macos-universal.dmg` and drag Pagenor to Applications.
The universal bundle contains Apple silicon and Intel executables. Development
previews use ad-hoc signing and are not notarized.

## Windows

Choose the MSI installer for your computer: x64 for Intel/AMD 64-bit PCs, arm64
for Windows on ARM, or x86 for a 32-bit installation. Run it and follow the installer.
Portable ZIP packages are available for all three architectures; extract the archive
before launching Pagenor. Development preview installers are currently unsigned.

## Linux

Choose x86_64 for Intel/AMD or aarch64 for ARM. Available packages are AppImage,
Flatpak, DEB, RPM and a tar.gz archive. Use DEB on Debian/Ubuntu, RPM on compatible
RPM-based systems, or extract the archive and launch its executable. The AppImage
may need executable permission. Flatpak installs with `flatpak install --user <file>`.

## First run

Pagenor opens the Welcome screen. Choose a new-document preset, open a file,
or select a recent-document thumbnail. Panels and tools appear when a document opens.
See [Getting started](index.md) or [startup troubleshooting](../troubleshooting/app-closes-at-start.md).
