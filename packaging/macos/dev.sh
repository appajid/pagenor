#!/usr/bin/env bash
# Build a local Pagenor.app with the current icon, then open it.
# Usage: packaging/macos/dev.sh [--release] [--no-launch] [-- app arguments...]
# Output: $CARGO_TARGET_DIR/macos-dev/Pagenor.app (ad-hoc signed, for this Mac).
set -euo pipefail

PROFILE=debug
LAUNCH=1
while [ $# -gt 0 ]; do
  case "$1" in
    --release) PROFILE=release; shift ;;
    --no-launch) LAUNCH=0; shift ;;
    --) shift; break ;;
    -h | --help) sed -n '2,4p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1 (put app arguments after --)" >&2; exit 2 ;;
  esac
done
if [ "$(uname -s)" != Darwin ]; then
  echo "error: this launcher requires macOS" >&2
  exit 1
fi
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
cd "$ROOT"
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TARGET_DIR="$(cd "$CARGO_TARGET_DIR" && pwd)"
export CARGO_TARGET_DIR
WORK="$CARGO_TARGET_DIR/macos-dev"
APP="$WORK/Pagenor.app"
mkdir -p "$WORK"

# Do not replace an executable that is still serving an open document.
# Icon/preview readers may hold ordinary file descriptors after the app exits.
if [ -f "$APP/Contents/MacOS/Pagenor" ] && lsof -a -d txt -t "$APP/Contents/MacOS/Pagenor" >/dev/null 2>&1; then
  echo "error: quit the local Pagenor.app before rebuilding it" >&2
  exit 1
fi
args=()
[ "$PROFILE" = release ] && args+=(--release)
cargo build --locked -p pagenor ${args[@]+"${args[@]}"}

# Stage the complete bundle before replacing the previous local build. Use the
# release metadata and owner-supplied ICNS; no runtime icon cache reset is needed.
STAGE="$(mktemp -d "$WORK/stage.XXXXXX")"
trap 'rm -rf "$STAGE"' EXIT
BUNDLE="$STAGE/Pagenor.app"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"
cp "$CARGO_TARGET_DIR/$PROFILE/pagenor" "$BUNDLE/Contents/MacOS/Pagenor"
cp "$ROOT/assets/app-icon/pagenor.icns" "$BUNDLE/Contents/Resources/Pagenor.icns"
cp "$ROOT/assets/app-icon/pagenor-document.icns" "$BUNDLE/Contents/Resources/PagenorDocument.icns"
copy_font_licences "$BUNDLE/Contents/Resources"
SHORT_VERSION="${VERSION%%-*}"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" \
  -e "s/@BUILD_SHA@/${PAGENOR_BUILD_SHA:-unknown}/g" \
  "$ROOT/packaging/macos/Info.plist.in" >"$BUNDLE/Contents/Info.plist"
plutil -lint "$BUNDLE/Contents/Info.plist"
printf 'APPL????' >"$BUNDLE/Contents/PkgInfo"
codesign --force --sign - --timestamp=none "$BUNDLE/Contents/MacOS/Pagenor"
codesign --force --sign - --timestamp=none "$BUNDLE"
codesign --verify --strict --deep "$BUNDLE"
rm -rf "$APP"
mv "$BUNDLE" "$APP"
echo "Local app: $APP"
if [ "$LAUNCH" = 1 ]; then
  open "$APP" --args "$@"
fi
