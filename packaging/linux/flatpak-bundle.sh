#!/usr/bin/env bash
# Repackage a Pagenor Linux tarball as a single-file Flatpak bundle:
#
#   $DIST/pagenor-<version>-linux-<arch>.flatpak
#
# Usage: packaging/linux/flatpak-bundle.sh [--no-test] [--stage-only] [TARBALL]
#
# TARBALL defaults to $DIST/pagenor-<version>-linux-<arch>.tar.gz from package.sh. The bundle
# is built for the host architecture (x86_64 or aarch64), which must match the tarball's.
# --stage-only validates/extracts the release without installing or running Flatpak.
# Needs python3, flatpak, flatpak-builder and the SVG pixbuf loader (librsvg2-common) for the host's
# `appstreamcli compose`; the freedesktop runtime and SDK named in the manifest are
# installed per-user from Flathub. Unless --no-test, the bundle is then installed per-user and
# `pagenor-cli --version` is run inside the sandbox as a smoke test.
# Manifest: packaging/linux/flatpak/org.pagenor.app.bundle.yml.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/linux"
APP_ID=org.pagenor.app
FLATHUB=https://dl.flathub.org/repo/flathub.flatpakrepo

TEST=1
STAGE_ONLY=0
TARBALL=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-test) TEST=0; shift ;;
    --stage-only) STAGE_ONLY=1; shift ;;
    -h | --help) sed -n '2,14p' "$0"; exit 0 ;;
    -*) echo "unknown argument: $1" >&2; exit 2 ;;
    *)
      [ -z "$TARBALL" ] || { echo "error: pass only one tarball" >&2; exit 2; }
      TARBALL="$1"; shift ;;
  esac
done

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ;;
  aarch64 | arm64) ARCH=aarch64 ;;
  *) echo "unsupported architecture $ARCH" >&2; exit 2 ;;
esac
BASENAME="pagenor-$VERSION-linux-$ARCH"
TARBALL="${TARBALL:-$DIST/$BASENAME.tar.gz}"
[ -f "$TARBALL" ] || { echo "error: $TARBALL not found (run packaging/linux/package.sh --formats tar first)" >&2; exit 1; }
command -v python3 >/dev/null || { echo "error: python3 is required to validate the release archive" >&2; exit 1; }

echo "==> Pagenor $VERSION Flatpak bundle for $ARCH from $(basename "$TARBALL")"

WORK="$CARGO_TARGET_DIR/flatpak-bundle"
rm -rf "$WORK"
mkdir -p "$WORK/stage"
# Validate the whole envelope and both ELF headers before writing any payload.
# Only the expected release root, directories and regular files are accepted.
python3 - "$TARBALL" "$WORK/stage" "$BASENAME" "$ARCH" <<'PYARCHIVE'
import os
from pathlib import Path, PurePosixPath
import shutil
import sys
import tarfile
archive, destination, expected_root, arch = sys.argv[1:]
try:
    with tarfile.open(archive, "r:gz") as tar:
        members = []
        paths = {}
        size = 0
        for member in tar:
            if len(members) >= 20000:
                raise ValueError("too many archive entries")
            raw = member.name
            parts = PurePosixPath(raw).parts
            if (not parts or parts[0] != expected_root or raw.startswith("/")
                    or ".." in parts or "\\" in raw or any(ord(c) < 32 for c in raw)):
                raise ValueError("unexpected release path")
            if not (member.isdir() or member.isfile()):
                raise ValueError("release links and special files are not accepted")
            relative = PurePosixPath(*parts[1:])
            if relative in paths:
                raise ValueError("duplicate release path")
            if member.size < 0 or member.size > 2 * 1024**3:
                raise ValueError("release file exceeds the size limit")
            size += member.size
            if size > 4 * 1024**3:
                raise ValueError("release exceeds the expanded size limit")
            paths[relative] = member
            members.append((relative, member))
        for relative, member in members:
            if any(paths.get(parent) and not paths[parent].isdir() for parent in relative.parents):
                raise ValueError("release file used as a directory")
        machine = {"x86_64": b"\x3e\x00", "aarch64": b"\xb7\x00"}[arch]
        for binary in ("pagenor", "pagenor-cli"):
            member = paths.get(PurePosixPath("bin", binary))
            if not member or not member.isfile() or not member.mode & 0o111:
                raise ValueError("release executable missing")
            with tar.extractfile(member) as payload:
                header = payload.read(20)
            if len(header) != 20 or header[:6] != b"\x7fELF\x02\x01" or header[18:20] != machine:
                raise ValueError("release executable architecture does not match the host")
        for relative, member in members:
            target = Path(destination).joinpath(*relative.parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with tar.extractfile(member) as src, target.open("xb") as dst:
                    shutil.copyfileobj(src, dst, length=1024 * 1024)
                os.chmod(target, member.mode & 0o777)
except (OSError, ValueError, tarfile.TarError) as error:
    raise SystemExit(f"error: invalid release archive: {error}")
PYARCHIVE
[ -x "$WORK/stage/bin/pagenor" ] || { echo "error: $TARBALL has no bin/pagenor" >&2; exit 1; }
for required in "share/applications/$APP_ID.desktop" "share/mime/packages/$APP_ID.xml" \
  "share/metainfo/$APP_ID.metainfo.xml" "share/icons/hicolor/256x256/apps/$APP_ID.png" \
  "share/icons/hicolor/scalable/apps/$APP_ID.svg" "share/icons/hicolor/256x256/mimetypes/$APP_ID.document.png" \
  "share/icons/hicolor/scalable/mimetypes/$APP_ID.document.svg" "share/doc/pagenor/LICENSE-MIT" "share/doc/pagenor/LICENSE-APACHE"; do
  [ -f "$WORK/stage/$required" ] || { echo "error: release resource missing: $required" >&2; exit 1; }
done
cp "$HERE/flatpak/$APP_ID.bundle.yml" "$WORK/$APP_ID.yml"
if [ "$STAGE_ONLY" = 1 ]; then
  echo "==> validated release tree: $WORK/stage"
  exit 0
fi
for tool in flatpak flatpak-builder; do
  command -v "$tool" >/dev/null || { echo "error: $tool not found (install flatpak flatpak-builder librsvg2-common)" >&2; exit 1; }
done

flatpak remote-add --user --if-not-exists flathub "$FLATHUB"
# --disable-rofiles-fuse: no FUSE in containers and CI runners.
flatpak-builder --user --install-deps-from=flathub --disable-rofiles-fuse --force-clean \
  --default-branch=stable --repo="$WORK/repo" --state-dir="$WORK/state" "$WORK/build" "$WORK/$APP_ID.yml"

OUT="$DIST/$BASENAME.flatpak"
# --runtime-repo lets `flatpak install` fetch the freedesktop runtime from Flathub if missing.
flatpak build-bundle --runtime-repo="$FLATHUB" "$WORK/repo" "$OUT" "$APP_ID" stable
echo "wrote $OUT"

if [ "$TEST" = 1 ]; then
  flatpak install --user -y --noninteractive --reinstall "$OUT"
  flatpak info --user "$APP_ID"
  flatpak run --command=pagenor-cli "$APP_ID" --version
  flatpak run --command=sh "$APP_ID" -c 'ls /app/share/applications /app/share/metainfo /app/share/mime/packages /app/share/icons/hicolor/scalable/apps'
fi
echo "==> done"
ls -lh "$OUT"
