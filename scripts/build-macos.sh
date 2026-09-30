#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-macos.sh — Automated build and packaging for macOS (Universal arm64 + x86_64)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Version = **nguồn duy nhất** `[workspace.package]` trong Cargo.toml (không hardcode 2 chỗ).
VERSION="$(grep -m1 '^version = "' "$ROOT/Cargo.toml" | cut -d'"' -f2)"
VERSION="${VERSION:-0.1.0}"
DIST_DIR="$ROOT/dist/macos"
STAGE_DIR="$DIST_DIR/stage"

if [ "$(uname -s)" != "Darwin" ]; then
    echo "❌ build-macos.sh chỉ chạy trên macOS" >&2
    exit 1
fi
echo "=== TextVN macOS Build v$VERSION ==="

# 1. Check prerequisites
for tool in cargo swift lipo tar zip shasum plutil codesign; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "❌ Missing required tool: $tool" >&2
        exit 1
    fi
done

# 2. Prepare directories
mkdir -p "$DIST_DIR"
rm -rf "$STAGE_DIR/TextVN.app" "$STAGE_DIR/TextVN-IM.app" "$STAGE_DIR/TextVN.iconset"
mkdir -p "$DIST_DIR" "$STAGE_DIR"
mkdir -p "$ROOT/adapters/macos-imk/lib"
mkdir -p "$ROOT/adapters/macos-imk/Sources/CTextVNFFI/include"

# Copy C-ABI header
cp "$ROOT/ffi/include/textvn_ffi.h" "$ROOT/adapters/macos-imk/Sources/CTextVNFFI/include/"

# 3. Build Rust FFI static library (Universal or Host)
echo "Building textvn-ffi Rust static library..."
HAS_ARM=0
HAS_X86=0

if rustup target list | grep -q "aarch64-apple-darwin (installed)"; then
    HAS_ARM=1
fi
if rustup target list | grep -q "x86_64-apple-darwin (installed)"; then
    HAS_X86=1
fi

if [ "$HAS_ARM" -eq 1 ] && [ "$HAS_X86" -eq 1 ]; then
    echo "Compiling for both aarch64 and x86_64 architectures..."
    cargo build --release -p textvn-ffi --target aarch64-apple-darwin --manifest-path "$ROOT/Cargo.toml"
    cargo build --release -p textvn-ffi --target x86_64-apple-darwin --manifest-path "$ROOT/Cargo.toml"

    lipo -create \
        "$ROOT/target/aarch64-apple-darwin/release/libtextvn_ffi.a" \
        "$ROOT/target/x86_64-apple-darwin/release/libtextvn_ffi.a" \
        -output "$ROOT/adapters/macos-imk/lib/libtextvn_ffi.a"
else
    echo "Compiling for host architecture..."
    cargo build --release -p textvn-ffi --manifest-path "$ROOT/Cargo.toml"
    cp "$ROOT/target/release/libtextvn_ffi.a" "$ROOT/adapters/macos-imk/lib/libtextvn_ffi.a"
fi

# 4. Build Swift Packages — universal nếu có đủ 2 Rust target (`--arch` × 2)
SWIFT_ARCH_FLAGS=""
if [ "$HAS_ARM" -eq 1 ] && [ "$HAS_X86" -eq 1 ]; then
    SWIFT_ARCH_FLAGS="--arch arm64 --arch x86_64"
fi

build_swift() {  # $1 = thư mục package
    # shellcheck disable=SC2086 — SWIFT_ARCH_FLAGS cố ý tách word (rỗng hoặc 2 flag)
    if [ -n "$SWIFT_ARCH_FLAGS" ]; then
        swift build -c release $SWIFT_ARCH_FLAGS --package-path "$1"
    else
        swift build -c release --package-path "$1"
    fi
}

echo "Building TextVN-IM input method..."
build_swift "$ROOT/adapters/macos-imk"

echo "Building TextVN menu bar & settings app..."
build_swift "$ROOT/adapters/macos-app"

# 5. Assemble Application Bundles
# `--show-bin-path` phải dùng CÙNG arch flags: universal → `.build/apple/Products/Release`,
# host-only → `.build/release`.
# shellcheck disable=SC2086
IM_BIN="$(swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-imk" --show-bin-path)/TextVN-IM"
# shellcheck disable=SC2086
APP_BIN="$(swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-app" --show-bin-path)/TextVN"

ARCH_LABEL="$(uname -m)"
if [ -n "$SWIFT_ARCH_FLAGS" ]; then
    for binary in "$IM_BIN" "$APP_BIN"; do
        archs="$(lipo -archs "$binary")"
        case " $archs " in
            *" arm64 "*) : ;; *) echo "❌ thiếu arm64: $binary ($archs)" >&2; exit 1 ;;
        esac
        case " $archs " in
            *" x86_64 "*) : ;; *) echo "❌ thiếu x86_64: $binary ($archs)" >&2; exit 1 ;;
        esac
    done
    ARCH_LABEL="universal"
fi

IM_BUNDLE="$STAGE_DIR/TextVN-IM.app"
APP_BUNDLE="$STAGE_DIR/TextVN.app"

mkdir -p "$IM_BUNDLE/Contents/MacOS" "$IM_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"

cp "$IM_BIN" "$IM_BUNDLE/Contents/MacOS/TextVN-IM"
cp "$APP_BIN" "$APP_BUNDLE/Contents/MacOS/TextVN"

cp "$ROOT/packaging/macos/Info-IM.plist" "$IM_BUNDLE/Contents/Info.plist"
cp "$ROOT/packaging/macos/Info-App.plist" "$APP_BUNDLE/Contents/Info.plist"
cp "$ROOT/data/appdb.default.json" "$IM_BUNDLE/Contents/Resources/appdb.default.json"

# Resources: uninstaller + bộ kiểm residue (uninstall_macos.sh dùng lại 2 file này)
cp "$ROOT/scripts/uninstall_macos.sh" "$APP_BUNDLE/Contents/Resources/"
chmod +x "$APP_BUNDLE/Contents/Resources/uninstall_macos.sh"
if [ -f "$ROOT/packaging/macos/uninstall-check.sh" ]; then
    cp "$ROOT/packaging/macos/uninstall-check.sh" "$APP_BUNDLE/Contents/Resources/"
    chmod +x "$APP_BUNDLE/Contents/Resources/uninstall-check.sh"
fi

# Icon .icns (tuỳ chọn) — menu Input Sources hiện icon cho input source.
# Đặt PNG nguồn (≥512×512) ở `packaging/macos/icons/TextVN-512.png` để bật.
ICON_SRC="$ROOT/packaging/macos/icons/TextVN-512.png"
if [ -f "$ICON_SRC" ]; then
    ICONSET="$STAGE_DIR/TextVN.iconset"
    rm -rf "$ICONSET"; mkdir -p "$ICONSET"
    for size in 16 32 128 256 512; do
        sips -z "$size" "$size" "$ICON_SRC" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
        sips -z "$((size * 2))" "$((size * 2))" "$ICON_SRC" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$ICONSET" -o "$STAGE_DIR/TextVN.icns"
    rm -rf "$ICONSET"
    cp "$STAGE_DIR/TextVN.icns" "$IM_BUNDLE/Contents/Resources/TextVN.icns"
    cp "$STAGE_DIR/TextVN.icns" "$APP_BUNDLE/Contents/Resources/TextVN.icns"
else
    echo "⚠️  Chưa có $ICON_SRC — bundle không kèm .icns: input source/app dùng icon mặc định"
fi

# Info.plist phải hợp lệ trước khi ký/phát hành (plutil = gate rẻ, bắt lỗi XML)
plutil -lint "$IM_BUNDLE/Contents/Info.plist" "$APP_BUNDLE/Contents/Info.plist"

# 6. Codesign (ad-hoc `-` khi dev · Developer ID khi release)
# KHÔNG dùng `--deep` (Apple deprecated): ký từng bundle với entitlements thật trong
# `packaging/macos/TextVN.entitlements` rồi `codesign --verify` để chắc chắn.
SIGN_IDENTITY="${DEVELOPER_ID:--}"
echo "Signing bundles with identity: $SIGN_IDENTITY"

for bundle in "$IM_BUNDLE" "$APP_BUNDLE"; do
    codesign --force -s "$SIGN_IDENTITY" \
        --entitlements "$ROOT/packaging/macos/TextVN.entitlements" \
        --options runtime "$bundle"
    codesign --verify --strict --verbose=2 "$bundle"
done

# Gatekeeper chỉ chấp nhận Developer ID + notarized → chỉ kiểm khi ký bằng cert thật.
if [ "$SIGN_IDENTITY" != "-" ]; then
    spctl -a -vv "$IM_BUNDLE" || echo "⚠️  spctl từ chối TextVN-IM.app — cần notarize (MAC-055)"
    spctl -a -vv "$APP_BUNDLE" || echo "⚠️  spctl từ chối TextVN.app — cần notarize (MAC-055)"
fi

# 7. Create Distribution Archives
TARBALL="$DIST_DIR/TextVN-macos-$ARCH_LABEL-v$VERSION.tar.gz"
ZIPFILE="$DIST_DIR/TextVN-macos-$ARCH_LABEL-v$VERSION.zip"
if [ -e "$TARBALL" ] || [ -e "$ZIPFILE" ]; then
    echo "❌ artifact v$VERSION/$ARCH_LABEL đã tồn tại — tăng version hoặc lưu bản cũ trước khi build" >&2
    exit 1
fi

echo "Creating release archives..."
(cd "$STAGE_DIR" && tar -czf "$TARBALL" TextVN.app TextVN-IM.app)
(cd "$STAGE_DIR" && zip -qry "$ZIPFILE" TextVN.app TextVN-IM.app)

# 8. Compute Checksums
(cd "$DIST_DIR" && shasum -a 256 "TextVN-macos-$ARCH_LABEL-v$VERSION.tar.gz" "TextVN-macos-$ARCH_LABEL-v$VERSION.zip" > SHA256SUMS.txt)

echo "✅ Build completed successfully:"
echo "   - $TARBALL"
echo "   - $ZIPFILE"
echo "   - $DIST_DIR/SHA256SUMS.txt"
echo ""
echo "Bước tiếp theo:"
echo "  scripts/package-macos-pkg.sh   # đóng gói .pkg per-user + kiểm gỡ sạch (MAC-054)"
echo "  scripts/install_macos.sh       # cài per-user ngay trên máy này (Rule S5)"
