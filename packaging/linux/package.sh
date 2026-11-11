#!/usr/bin/env bash
# Build and package Pagenor for Linux (<arch> is x86_64, aarch64 or riscv64):
#
#   $DIST/pagenor-<version>-linux-<arch>.AppImage  any distro with glibc >= the build host's
#   $DIST/pagenor-<version>-linux-<arch>.AppImage.zsync  delta updates (needs zsyncmake)
#   $DIST/pagenor-<version>-linux-<arch>.deb       Debian, Ubuntu, Mint, Pop!_OS, ...
#   $DIST/pagenor-<version>-linux-<arch>.rpm       Fedora, openSUSE, RHEL, ...
#   $DIST/pagenor-<version>-linux-<arch>.tar.gz    plain FHS-style tree (bin/, share/)
#
# riscv64 currently supports tar only (Ubuntu 24.04 CI baseline: glibc 2.39).
# Cross builds: set CROSS_ARCH, CROSS_TARGET and CROSS_COMPILE; EMULATOR is optional.
#
# Usage: packaging/linux/package.sh [--skip-build] [--formats "appimage deb rpm tar"]
#
# Needs: cargo; nfpm for deb/rpm (https://nfpm.goreleaser.com); appimagetool for the AppImage
# (downloaded into $CARGO_TARGET_DIR if missing). Build on an old distro (CI: Ubuntu 22.04,
# glibc 2.35) so the binaries run on newer ones. Optional: desktop-file-validate, appstreamcli,
# zsyncmake (the zsync package) for the AppImage's .zsync.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/linux"
APP_ID=org.pagenor.app

SKIP_BUILD=0
FORMATS="appimage deb rpm tar"
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --formats)
      [ $# -ge 2 ] || { echo "--formats needs a value" >&2; exit 2; }
      FORMATS="$2"; shift 2 ;;
    -h | --help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

HOST_ARCH="$(uname -m)"
ARCH="${CROSS_ARCH:-$HOST_ARCH}"
case "$ARCH" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64 | arm64) ARCH=aarch64; DEB_ARCH=arm64 ;;
  riscv64) DEB_ARCH=riscv64 ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac
# Reject inconsistent labels before building or writing a package. Target paths are not freeform.
case "${CROSS_TARGET:-}" in
  "")
    if [ "$ARCH" != "$HOST_ARCH" ] && ! { [ "$ARCH" = aarch64 ] && [ "$HOST_ARCH" = arm64 ]; }; then
      echo "CROSS_ARCH requires a matching CROSS_TARGET when the host architecture differs" >&2; exit 2
    fi ;;
  x86_64-unknown-linux-gnu) TARGET_ARCH=x86_64 ;;
  aarch64-unknown-linux-gnu) TARGET_ARCH=aarch64 ;;
  riscv64gc-unknown-linux-gnu) TARGET_ARCH=riscv64 ;;
  *) echo "unsupported CROSS_TARGET ${CROSS_TARGET}" >&2; exit 2 ;;
esac
if [ -n "${CROSS_TARGET:-}" ] && [ "$ARCH" != "$TARGET_ARCH" ]; then
  echo "CROSS_ARCH does not match CROSS_TARGET" >&2; exit 2
fi
[ -n "$FORMATS" ] || { echo "--formats must not be empty" >&2; exit 2; }
for format in $FORMATS; do
  case "$format" in
    appimage | deb | rpm | tar) ;;
    *) echo "unsupported package format $format" >&2; exit 2 ;;
  esac
  if [ "$ARCH" = riscv64 ] && [ "$format" != tar ]; then
    echo "riscv64 currently supports only --formats tar" >&2; exit 2
  fi
done

export PAGENOR_MAINTAINER="${PAGENOR_MAINTAINER:-Appaji Ambarisha}"
BASENAME="pagenor-$VERSION-linux-$ARCH"

echo "==> Pagenor $VERSION for Linux $ARCH ($FORMATS)"

if [ "$SKIP_BUILD" = 0 ]; then
  BUILD_ARGS=(--release --locked -p pagenor -p pagenor-cli)
  if [ -n "${CROSS_TARGET:-}" ]; then BUILD_ARGS+=(--target "$CROSS_TARGET"); fi
  (cd "$ROOT" && cargo build "${BUILD_ARGS[@]}")
fi
BIN="$CARGO_TARGET_DIR/${CROSS_TARGET:+$CROSS_TARGET/}release"
WORK="$CARGO_TARGET_DIR/linux-package"
STAGE="$WORK/root"
rm -rf "$WORK"

# ---- stage an FHS tree (shared by every format) -------------------------------------------------
install -Dm755 "$BIN/pagenor" "$STAGE/usr/bin/pagenor"
install -Dm755 "$BIN/pagenor-cli" "$STAGE/usr/bin/pagenor-cli"
"${CROSS_COMPILE:-}strip" "$STAGE/usr/bin/pagenor" "$STAGE/usr/bin/pagenor-cli" 2>/dev/null || true
install -Dm644 "$HERE/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
install -Dm644 "$HERE/$APP_ID.mime.xml" "$STAGE/usr/share/mime/packages/$APP_ID.xml"
mkdir -p "$STAGE/usr/share/metainfo"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@DATE@/$PAGENOR_BUILD_DATE/g" \
  "$HERE/$APP_ID.metainfo.xml.in" >"$STAGE/usr/share/metainfo/$APP_ID.metainfo.xml"
mkdir -p "$STAGE/usr/share/icons"
cp -R "$ROOT/assets/app-icon/hicolor" "$STAGE/usr/share/icons/"
mkdir -p "$STAGE/usr/share/doc/pagenor"
copy_docs "$STAGE/usr/share/doc/pagenor"

if command -v desktop-file-validate >/dev/null; then
  desktop-file-validate "$STAGE/usr/share/applications/$APP_ID.desktop"
fi
if command -v appstreamcli >/dev/null; then
  appstreamcli validate --no-net --explain "$STAGE/usr/share/metainfo/$APP_ID.metainfo.xml"
fi

has() { case " $FORMATS " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }

# ---- .tar.gz ------------------------------------------------------------------------------------
if has tar; then
  mkdir -p "$WORK/tar"
  cp -R "$STAGE/usr" "$WORK/tar/$BASENAME"
  tar -C "$WORK/tar" -czf "$DIST/$BASENAME.tar.gz" "$BASENAME"
  echo "wrote $DIST/$BASENAME.tar.gz"
fi

# ---- .deb / .rpm --------------------------------------------------------------------------------
if has deb || has rpm; then
  command -v nfpm >/dev/null || { echo "error: nfpm not found (https://nfpm.goreleaser.com/install/)" >&2; exit 1; }
  export VERSION
  export NFPM_ARCH="$DEB_ARCH"
  # nfpm expands env vars in fields like `version` and `arch`, but not in `contents[].src`.
  sed "s|\${STAGE}|$STAGE|g" "$HERE/nfpm.yaml" >"$WORK/nfpm.yaml"
  for fmt in deb rpm; do
    if has "$fmt"; then (cd "$ROOT" && nfpm package -f "$WORK/nfpm.yaml" -p "$fmt" -t "$DIST/$BASENAME.$fmt"); fi
  done
fi

# ---- AppImage -----------------------------------------------------------------------------------
if has appimage; then
  APPDIR="$WORK/Pagenor.AppDir"
  cp -R "$STAGE" "$APPDIR"
  ln -s usr/bin/pagenor "$APPDIR/AppRun"
  cp "$HERE/$APP_ID.desktop" "$APPDIR/$APP_ID.desktop"
  cp "$ROOT/assets/app-icon/hicolor/256x256/apps/$APP_ID.png" "$APPDIR/$APP_ID.png"
  ln -s "$APP_ID.png" "$APPDIR/.DirIcon"

  TOOL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
  if [ -z "$TOOL" ]; then
    TOOL="$CARGO_TARGET_DIR/appimagetool-$ARCH.AppImage"
    if [ ! -x "$TOOL" ]; then
      curl -fsSL -o "$TOOL" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage"
      chmod +x "$TOOL"
    fi
  fi
  # Absolute, because appimagetool runs in $DIST below (CARGO_TARGET_DIR or APPIMAGETOOL may be
  # relative, e.g. target/agent-<name>).
  OUT="$(cd "$DIST" && pwd)/$BASENAME.AppImage"
  APPDIR="$(cd "$APPDIR" && pwd)"
  TOOL="$(cd "$(dirname "$TOOL")" && pwd)/$(basename "$TOOL")"
  # A .zsync left by an earlier run would hide a missing zsyncmake and describe another file.
  rm -f "$OUT.zsync"
  # Update information: AppImageUpdate, AppImageLauncher and the like read it from the file and
  # fetch only the blocks that changed in a newer release, through the .zsync published next to
  # each AppImage on GitHub Releases. `latest` is the newest published release that is not a
  # pre-release. A fork's builds point at its own releases through GITHUB_REPOSITORY.
  REPO="${GITHUB_REPOSITORY:-appajid/pagenor}"
  if ! [[ "$REPO" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    echo "error: GITHUB_REPOSITORY must be owner/repository" >&2
    exit 2
  fi
  UPDATE_INFO="gh-releases-zsync|${REPO%%/*}|${REPO#*/}|latest|pagenor-*-linux-$ARCH.AppImage.zsync"
  # Extract-and-run: works without FUSE (containers, CI). The output embeds the static runtime,
  # so users don't need libfuse2 either. With zsyncmake on the host (CI installs the zsync
  # package) appimagetool also writes the .zsync, into its working directory, hence the cd.
  (cd "$DIST" && ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$TOOL" --no-appstream -u "$UPDATE_INFO" "$APPDIR" "$OUT")
  echo "wrote $OUT"
  if [ -s "$OUT.zsync" ]; then
    echo "wrote $OUT.zsync"
  else
    warn "zsyncmake not found, so $OUT.zsync was not written; AppImage delta updates need it"
  fi
fi

if [ -z "${CROSS_TARGET:-}" ]; then
  "$STAGE/usr/bin/pagenor-cli" --version
elif [ -n "${EMULATOR:-}" ]; then
  "$EMULATOR" "$STAGE/usr/bin/pagenor-cli" --version
else
  echo "cross build: CLI execution skipped (set EMULATOR to smoke-test)"
fi
echo "==> done"
ls -lh "$DIST"
