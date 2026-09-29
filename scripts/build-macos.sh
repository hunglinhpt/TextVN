#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-macos.sh — Automated build and packaging for macOS (Universal arm64 + x86_64)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="0.1.0"
DIST_DIR="$ROOT/dist/macos"
STAGE_DIR="$DIST_DIR/stage"

echo "=== TextVN macOS Universal Build v$VERSION ==="

# 1. Check prerequisites
for tool in cargo swift lipo tar zip shasum; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "❌ Missing required tool: $tool" >&2
        exit 1
    fi
done

# 2. Prepare directories
rm -rf "$DIST_DIR"
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

# 4. Build Swift Packages
SWIFT_ARCH_FLAGS=""
if [ "$HAS_ARM" -eq 1 ] && [ "$HAS_X86" -eq 1 ]; then
    SWIFT_ARCH_FLAGS="--arch arm64 --arch x86_64"
fi

echo "Building TextVN-IM input method..."
swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-imk"

echo "Building TextVN menu bar & settings app..."
swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-app"

# 5. Assemble Application Bundles
IM_BIN="$(swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-imk" --show-bin-path)/TextVN-IM"
APP_BIN="$(swift build -c release $SWIFT_ARCH_FLAGS --package-path "$ROOT/adapters/macos-app" --show-bin-path)/TextVN"

IM_BUNDLE="$STAGE_DIR/TextVN-IM.app"
APP_BUNDLE="$STAGE_DIR/TextVN.app"

mkdir -p "$IM_BUNDLE/Contents/MacOS" "$IM_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"

cp "$IM_BIN" "$IM_BUNDLE/Contents/MacOS/TextVN-IM"
cp "$APP_BIN" "$APP_BUNDLE/Contents/MacOS/TextVN"

cp "$ROOT/packaging/macos/Info-IM.plist" "$IM_BUNDLE/Contents/Info.plist"
cp "$ROOT/packaging/macos/Info-App.plist" "$APP_BUNDLE/Contents/Info.plist"

# Include uninstaller & residue checker in TextVN.app bundle resources
cp "$ROOT/scripts/uninstall_macos.sh" "$APP_BUNDLE/Contents/Resources/"
chmod +x "$APP_BUNDLE/Contents/Resources/uninstall_macos.sh"
cp "$ROOT/packaging/macos/uninstall-check.sh" "$APP_BUNDLE/Contents/Resources/"
chmod +x "$APP_BUNDLE/Contents/Resources/uninstall-check.sh"

# 6. Codesign (Ad-hoc or Developer ID)
SIGN_IDENTITY="${DEVELOPER_ID:--}"
echo "Signing bundles with identity: $SIGN_IDENTITY"

codesign --force --deep -s "$SIGN_IDENTITY" \
    --entitlements "$ROOT/packaging/macos/TextVN.entitlements" \
    --options runtime "$IM_BUNDLE"

codesign --force --deep -s "$SIGN_IDENTITY" \
    --entitlements "$ROOT/packaging/macos/TextVN.entitlements" \
    --options runtime "$APP_BUNDLE"

# 7. Create Distribution Archives
TARBALL="$DIST_DIR/TextVN-macos-universal-v$VERSION.tar.gz"
ZIPFILE="$DIST_DIR/TextVN-macos-universal-v$VERSION.zip"

echo "Creating release archives..."
(cd "$STAGE_DIR" && tar -czf "$TARBALL" TextVN.app TextVN-IM.app)
(cd "$STAGE_DIR" && zip -qry "$ZIPFILE" TextVN.app TextVN-IM.app)

# 8. Compute Checksums
(cd "$DIST_DIR" && shasum -a 256 "TextVN-macos-universal-v$VERSION.tar.gz" "TextVN-macos-universal-v$VERSION.zip" > SHA256SUMS.txt)

echo "✅ Build completed successfully:"
echo "   - $TARBALL"
echo "   - $ZIPFILE"
echo "   - $DIST_DIR/SHA256SUMS.txt"
