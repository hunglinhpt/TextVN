#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# build-rust.sh — build engine Rust static cho macOS rồi assemble TextVN-IM.app
# (MAC-001/003, P2-1 §2: cargo 2 arch + lipo → libtextvn_ffi.a universal).
#
# Cách dùng:
#   ./build-rust.sh                 # build static lib + swift build + assemble .app
#   ./build-rust.sh --lib-only      # chỉ build libtextvn_ffi.a (cho `swift build`/`swift test`)
#   ./build-rust.sh --release       # chạy kèm cargo/swift --release
#
# Yêu cầu (MAC-001): macOS 13+, Xcode CLT 15+, rustup targets
#   aarch64-apple-darwin + x86_64-apple-darwin.
# Link: SwiftPM đọc `-L lib -ltextvn_ffi` từ Package.swift (IMKApp target).
set -euo pipefail

cd "$(dirname "$0")"

LIB_DIR="lib"
MODE_LIB_ONLY=0
PROFILE_FLAG=""

for arg in "$@"; do
  case "$arg" in
    --lib-only) MODE_LIB_ONLY=1 ;;
    --release) PROFILE_FLAG="--release" ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

ROOT_DIR="$(cd ../.. && pwd)"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"

mkdir -p Sources/CTextVNFFI/include
cp "$ROOT_DIR/ffi/include/textvn_ffi.h" Sources/CTextVNFFI/include/textvn_ffi.h

echo "== [1/4] cargo build 2 arch (P2-1 §2) =="
cargo build $PROFILE_FLAG --target aarch64-apple-darwin -p textvn-ffi --target-dir "$CARGO_TARGET_DIR"
cargo build $PROFILE_FLAG --target x86_64-apple-darwin -p textvn-ffi --target-dir "$CARGO_TARGET_DIR"

# staticlib output: target/<triple>/{debug|release}/libtextvn_ffi.a
SUBDIR="debug"
if [ "$PROFILE_FLAG" = "--release" ]; then SUBDIR="release"; fi

echo "== [2/4] lipo universal libtextvn_ffi.a =="
mkdir -p "$LIB_DIR"
lipo -create \
  "$CARGO_TARGET_DIR/aarch64-apple-darwin/$SUBDIR/libtextvn_ffi.a" \
  "$CARGO_TARGET_DIR/x86_64-apple-darwin/$SUBDIR/libtextvn_ffi.a" \
  -output "$LIB_DIR/libtextvn_ffi.a"
lipo -info "$LIB_DIR/libtextvn_ffi.a"

if [ "$MODE_LIB_ONLY" -eq 1 ]; then
  echo "== done (lib-only) — swift build/test dùng được rồi =="
  exit 0
fi

echo "== [3/4] swift build (SwiftPM) =="
BIN_DIR="$(swift build $PROFILE_FLAG --show-bin-path)"
BIN="$BIN_DIR/TextVN-IM"
test -x "$BIN" || { echo "swift build output missing: $BIN" >&2; exit 1; }

echo "== [4/4] assemble TextVN-IM.app (per-user — P2-1 §8) =="
APP="build/TextVN-IM.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/TextVN-IM"
cp Resources/Info.plist "$APP/Contents/Info.plist"
cp TextVN-IM.entitlements "$APP/Contents/TextVN-IM.entitlements"

echo "Universal engine + bundle assembled:"
ls -la "$LIB_DIR" "$APP/Contents/MacOS"
echo
echo "Cài per-user (P2-1 §8 — KHÔNG ghi /Library):"
echo "  mkdir -p ~/Library/Input\\ Methods && cp -R $APP ~/Library/Input\\ Methods/"
echo "Rồi bật: System Settings → Keyboard → Input Sources → Add TextVN"
