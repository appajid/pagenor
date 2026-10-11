#!/usr/bin/env bash
# Regenerate app and document icons from the owner-supplied artwork.
#
# Needs: Cargo (the workspace resvg renderer and portable ICO/ICNS packers). The outputs are committed, so builds and packaging never need these tools.
#
#   packaging/icons.sh [--documents-only]
set -euo pipefail
DOCUMENTS_ONLY=0
case "${1:-}" in
  --documents-only) DOCUMENTS_ONLY=1; shift ;;
  "") ;;
  *) echo "usage: packaging/icons.sh [--documents-only]" >&2; exit 2 ;;
esac
if [ $# -ne 0 ]; then echo "usage: packaging/icons.sh [--documents-only]" >&2; exit 2; fi
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/pagenor.svg"
ID="org.pagenor.app"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cargo build -q --manifest-path "$ROOT/Cargo.toml" -p xtask
RENDER="$ROOT/target/debug/xtask"
if [ -n "${CARGO_TARGET_DIR:-}" ]; then RENDER="$CARGO_TARGET_DIR/debug/xtask"; fi

render() { "$RENDER" svg "$1" "$3" "$2"; }

# Outline the supplied document header so generated icons need no installed font.
DOC="$DIR/pagenor-document.svg"
render "$ROOT/assets/brand/Pagenor-Document-Icon.svg" 512 "$DOC"
render "$DOC" 1024 "$DIR/pagenor-document-1024.png"
DOC_ICNS=()
DOC_ICO=()
for s in 16 20 24 32 40 48 64 128 256 512 1024; do
  render "$DOC" "$s" "$TMP/document-$s.png"
  if [ "$s" -le 256 ]; then DOC_ICO+=("$TMP/document-$s.png"); fi
  case "$s" in 16|32|64|128|256|512|1024) DOC_ICNS+=("$TMP/document-$s.png");; esac
done
"$RENDER" ico "$DIR/pagenor-document.ico" "${DOC_ICO[@]}"
"$RENDER" icns "$DIR/pagenor-document.icns" "${DOC_ICNS[@]}"
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/mimetypes"
  cp "$TMP/document-$s.png" "$DIR/hicolor/${s}x${s}/mimetypes/$ID.document.png"
done
mkdir -p "$DIR/hicolor/scalable/mimetypes"
cp "$DOC" "$DIR/hicolor/scalable/mimetypes/$ID.document.svg"
echo "document icons written to $DIR"
if [ "$DOCUMENTS_ONLY" = 1 ]; then exit 0; fi

# The artwork is a full-bleed 192 tile (rx=40). Windows and Linux use it as is. macOS icons sit on
# Apple's grid: an 824/1024 body with a transparent margin, so widen the viewBox to 192*1024/824.
MAC="$TMP/mac.svg"
sed 's/viewBox="0 0 192 192"/viewBox="-23.301 -23.301 238.602 238.602"/' "$SVG" >"$MAC"
grep -q 'viewBox="-23.301' "$MAC" || { echo "error: expected viewBox=\"0 0 192 192\" in $SVG" >&2; exit 1; }

render "$SVG" 1024 "$DIR/pagenor-1024.png"
# Runtime window/Dock icon on macOS (embedded by apps/pagenor); other platforms use hicolor 256.
render "$MAC" 512 "$DIR/pagenor-macos-512.png"

# Linux hicolor theme.
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  SOURCE="$SVG"
  if [ "$s" -le 48 ]; then SOURCE="$DIR/pagenor-small.svg"; fi
  render "$SOURCE" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$DIR/pagenor-small.svg" "$DIR/hicolor/scalable/apps/$ID.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  SOURCE="$SVG"
  if [ "$s" -le 48 ]; then SOURCE="$DIR/pagenor-small.svg"; fi
  render "$SOURCE" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
"$RENDER" ico "$DIR/pagenor.ico" "${ICO_PNGS[@]}"

# macOS .icns, packed on any host without iconutil.
ICNS_PNGS=()
for s in 16 32 64 128 256 512 1024; do
  render "$MAC" "$s" "$TMP/icns-$s.png"
  ICNS_PNGS+=("$TMP/icns-$s.png")
done
"$RENDER" icns "$DIR/pagenor.icns" "${ICNS_PNGS[@]}"
echo "icons written to $DIR"

# Outline logo text once so runtime branding needs no installed Arial/Helvetica font.
for variant in Primary Inverse; do
  lower=$(echo "$variant" | tr '[:upper:]' '[:lower:]')
  "$RENDER" svg "$ROOT/assets/brand/Pagenor-Logo-$variant.svg" "$ROOT/assets/brand/runtime-logo-$lower.svg" 650
done

for screen in about-page splash-screen; do
  "$RENDER" svg "$ROOT/assets/brand/pagenor-$screen-editable.svg" "$ROOT/assets/brand/runtime-$screen.svg" 980
done
