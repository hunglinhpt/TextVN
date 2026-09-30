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
# Link: SwiftPM đọc `-L lib -ltextvn_ffi` từ Package.swift (CoreBridge target).
#
# MAC-031: bash 3.2 (macOS system shell) dùng scalar flag, không dùng mảng rỗng
# vì `set -u` ném "unbound variable" khi expand mảng rỗng trong bash 3.2.
set -euo pipefail

# PKG_DIR = thư mục của script này (adapters/macos-imk/)
# ROOT_DIR = gốc workspace (2 cấp trên) — chứa Cargo.toml workspace và target/
PKG_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$PKG_DIR/../.." && pwd)"

# LIB_DIR tuyệt đối — nơi đặt libtextvn_ffi.a để swift build tìm thấy
LIB_DIR="$PKG_DIR/lib"
MODE_LIB_ONLY=0
PROFILE_FLAG=""   # "" = debug, "--release" = release — scalar an toàn với bash 3.2 set -u

for arg in "$@"; do
  case "$arg" in
    --lib-only) MODE_LIB_ONLY=1 ;;
    --release)  PROFILE_FLAG="--release" ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# Xác định subdir debug|release để copy staticlib đúng vị trí.
SUBDIR="debug"
if [ "$PROFILE_FLAG" = "--release" ]; then SUBDIR="release"; fi

echo "== [1/4] cargo build 2 arch (P2-1 §2) =="
echo "  workspace root : $ROOT_DIR"
echo "  package dir    : $PKG_DIR"
echo "  profile        : ${PROFILE_FLAG:-debug}"

# Chạy cargo từ workspace root để Cargo.toml workspace được tìm thấy đúng.
# shellcheck disable=SC2086 — PROFILE_FLAG intentionally unquoted (empty or single flag)
(cd "$ROOT_DIR" && cargo build $PROFILE_FLAG --target aarch64-apple-darwin -p textvn-ffi)
(cd "$ROOT_DIR" && cargo build $PROFILE_FLAG --target x86_64-apple-darwin  -p textvn-ffi)

echo "== [2/4] lipo universal libtextvn_ffi.a =="
mkdir -p "$LIB_DIR"
lipo -create \
  "$ROOT_DIR/target/aarch64-apple-darwin/$SUBDIR/libtextvn_ffi.a" \
  "$ROOT_DIR/target/x86_64-apple-darwin/$SUBDIR/libtextvn_ffi.a" \
  -output "$LIB_DIR/libtextvn_ffi.a"
lipo -info "$LIB_DIR/libtextvn_ffi.a"

if [ "$MODE_LIB_ONLY" -eq 1 ]; then
  echo "== done (lib-only) — swift build/test dùng được rồi =="
  exit 0
fi

echo "== [3/4] swift build (SwiftPM) =="
cd "$PKG_DIR"
# shellcheck disable=SC2086
swift build $PROFILE_FLAG
BIN="$PKG_DIR/.build/debug/TextVN-IM"
if [ "$PROFILE_FLAG" = "--release" ]; then BIN="$PKG_DIR/.build/release/TextVN-IM"; fi
test -x "$BIN" || { echo "swift build output missing: $BIN" >&2; exit 1; }

echo "== [4/4] assemble TextVN-IM.app (per-user — P2-1 §8) =="
APP="$PKG_DIR/build/TextVN-IM.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/TextVN-IM"
cp "$PKG_DIR/Resources/Info.plist" "$APP/Contents/Info.plist"
cp "$ROOT_DIR/data/appdb.default.json" "$APP/Contents/Resources/appdb.default.json"
cp "$PKG_DIR/TextVN-IM.entitlements" "$APP/Contents/TextVN-IM.entitlements"

echo "Universal engine + bundle assembled:"
ls -la "$LIB_DIR" "$APP/Contents/MacOS"
echo
echo "Cài per-user (P2-1 §8 — KHÔNG ghi /Library):"
echo "  mkdir -p ~/Library/Input\\ Methods && cp -R $APP ~/Library/Input\\ Methods/"
echo "Rồi bật: System Settings → Keyboard → Input Sources → Add TextVN"
