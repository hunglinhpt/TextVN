#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# package-macos-pkg.sh — đóng gói `.pkg` per-user trên macOS (MAC-054, P2-4 §4).
#
# Cách dùng:
#   scripts/package-macos-pkg.sh                # đóng gói bundle đã build trong dist/macos/stage
#   scripts/package-macos-pkg.sh --rebuild      # chạy build-macos.sh trước
#   scripts/package-macos-pkg.sh --notarize     # ký + notarize + staple (cần DEVELOPER_ID*)
#
# Cài đặt (KHÔNG cần sudo — PLAN §3.7):
#   installer -pkg dist/macos/TextVN-mac-v<ver>.pkg -target CurrentUserHomeDirectory
# Cài toàn máy (cần admin — chỉ dùng cho VM test):
#   sudo installer -pkg dist/macos/TextVN-mac-v<ver>.pkg -target /
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(grep -m1 '^version = "' "$ROOT/Cargo.toml" | cut -d'"' -f2)"
VERSION="${VERSION:-0.1.0}"
DIST_DIR="$ROOT/dist/macos"
STAGE_DIR="$DIST_DIR/stage"
PKG_ROOT="$DIST_DIR/pkgroot"
COMPONENT_PKG="$DIST_DIR/textvn-component.pkg"
OUT_PKG="$DIST_DIR/TextVN-mac-v$VERSION.pkg"
PKG_ID="vn.textvn.pkg"

REBUILD=0
NOTARIZE=0
for arg in "$@"; do
    case "$arg" in
        --rebuild) REBUILD=1 ;;
        --notarize) NOTARIZE=1 ;;
        *) echo "unknown flag: $arg" >&2; exit 2 ;;
    esac
done

for tool in pkgbuild productbuild installer shasum codesign; do
    command -v "$tool" >/dev/null 2>&1 || { echo "❌ thiếu công cụ: $tool" >&2; exit 1; }
done

[ "$REBUILD" -eq 1 ] && "$ROOT/scripts/build-macos.sh"

for bundle in "$STAGE_DIR/TextVN.app" "$STAGE_DIR/TextVN-IM.app"; do
    if [ ! -d "$bundle" ]; then
        echo "❌ thiếu $bundle — chạy scripts/build-macos.sh (hoặc --rebuild) trước" >&2
        exit 1
    fi
done

echo "=== TextVN .pkg v$VERSION ==="
if [ -e "$OUT_PKG" ]; then
    echo "❌ .pkg v$VERSION đã tồn tại — không ghi đè bản phát hành cũ" >&2
    exit 1
fi

# 1. Payload root: đường dẫn TƯƠNG ĐỐI so với đích cài (xem packaging/macos/distribution.xml).
rm -rf "$PKG_ROOT"
mkdir -p "$PKG_ROOT/Applications" "$PKG_ROOT/Library/Input Methods"
cp -R "$STAGE_DIR/TextVN.app" "$PKG_ROOT/Applications/"
cp -R "$STAGE_DIR/TextVN-IM.app" "$PKG_ROOT/Library/Input Methods/"

# 2. Bundle phải ký hợp lệ trước khi đóng gói (build-macos.sh đã ký) — pkg chứa bundle
#    chưa ký sẽ bị Gatekeeper/notarization từ chối.
for bundle in "$PKG_ROOT/Applications/TextVN.app" "$PKG_ROOT/Library/Input Methods/TextVN-IM.app"; do
    codesign --verify --strict "$bundle"
done

# 3. Cảnh báo trung thực nếu bản build không universal (build-macos.sh đã in lý do).
IM_BIN="$STAGE_DIR/TextVN-IM.app/Contents/MacOS/TextVN-IM"
if [ -x "$IM_BIN" ] && ! lipo -archs "$IM_BIN" | grep -qE "arm64.*x86_64|x86_64.*arm64"; then
    echo "⚠️  TextVN-IM KHÔNG universal ($(lipo -archs "$IM_BIN")) — .pkg chỉ chạy trên $(uname -m)" >&2
fi

# 4. component pkg → distribution pkg (P2-4 §4: component-plist relocatable=false)
rm -f "$COMPONENT_PKG"
pkgbuild --root "$PKG_ROOT" \
    --identifier "$PKG_ID" \
    --version "$VERSION" \
    --install-location "/" \
    --component-plist "$ROOT/packaging/macos/component.plist" \
    --scripts "$ROOT/packaging/macos/pkg-scripts" \
    "$COMPONENT_PKG"

productbuild --distribution "$ROOT/packaging/macos/distribution.xml" \
    --package-path "$DIST_DIR" \
    "$OUT_PKG"
rm -f "$COMPONENT_PKG"

# 5. Ký .pkg — cần cert **Developer ID Installer** (KHÁC cert "Developer ID Application"
#    dùng cho bundle). Không có cert → giữ bản chưa ký (chỉ phát cho nội bộ).
if [ -n "${DEVELOPER_ID_INSTALLER:-}" ]; then
    SIGNED="$DIST_DIR/TextVN-mac-v$VERSION-signed.pkg"
    productsign --sign "$DEVELOPER_ID_INSTALLER" "$OUT_PKG" "$SIGNED"
    mv "$SIGNED" "$OUT_PKG"
    pkgutil --check-signature "$OUT_PKG"
else
    echo "ℹ️  Không có DEVELOPER_ID_INSTALLER — .pkg để CHƯA ký."
    echo "    Trạng thái phát hành: docs/release/signing-status-mac.md (MAC-055/RM3)"
fi

# 6. Notarize + staple (tuỳ chọn, cần Apple ID/team + app-specific password)
if [ "$NOTARIZE" -eq 1 ]; then
    "$ROOT/scripts/notarize-macos.sh" "$OUT_PKG"
fi

# 7. Checksum
(cd "$DIST_DIR" && shasum -a 256 "$(basename "$OUT_PKG")" > SHA256SUMS-pkg.txt)

echo ""
echo "✅ Đã tạo: $OUT_PKG"
echo "   cài per-user (không sudo): installer -pkg \"$OUT_PKG\" -target CurrentUserHomeDirectory"
echo "   kiểm gỡ 0 residue        : packaging/macos/uninstall-check.sh [--purge]"
