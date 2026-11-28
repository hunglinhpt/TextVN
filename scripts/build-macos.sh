#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-macos.sh — Automated build and packaging for macOS (Universal arm64 + x86_64)

# ============================================================================
# ARM-M1 — gate universal: MỌI Mach-O trong .app/.pkg phải có CẢ 2 slice
# arm64 + x86_64 (lặp lỗi OpenKey #189 — binary 1 kiến trúc trôi vào bản phát
# hành ⇒ đỏ). Hàm gate đặt MỘT nguồn ở đây (G22); `scripts/package-macos-pkg.sh`
# và `scripts/test-macos-universal-gate.sh` `source` tệp này — guard dưới chỉ
# cho chạy body build khi script được gọi trực tiếp.
# Logic quyết định tách thuần để test được cả trên Windows (không có lipo):
#   - textvn_uni_have_both_archs: parse chuỗi arch (đầu ra `lipo -archs`/`file`)
#   - textvn_uni_is_macho:        nhận diện Mach-O qua magic 4 byte (`od` — có
#                                 sẵn trên macOS lẫn Git Bash Windows)
# Lấy arch: ưu tiên `lipo -archs` (đè được bằng TEXTVN_UNI_LIPO khi test),
# không có lipo thì thử `file`. Không probe được ⇒ coi là KHÔNG đạt.
# ============================================================================
textvn_uni_have_both_archs() {
    # $1 = chuỗi kiến trúc (vd "arm64 x86_64"). exit 0 = đủ cả hai slice.
    local archs=" $1 "
    case "$archs" in
        *" arm64 "*) ;;
        *) return 1 ;;
    esac
    case "$archs" in
        *" x86_64 "*) ;;
        *) return 1 ;;
    esac
    return 0
}

textvn_uni_is_macho() {
    # exit 0 nếu 4 byte đầu là magic Mach-O / universal (fat) binary.
    local f="$1" magic
    magic="$(od -An -tx1 -N 4 "$f" 2>/dev/null | tr -d ' \n')"
    case "$magic" in
        cffaedfe|feedfacf|cefaedfe|feedface|cafebabe|bebafeca|cafebabf|bfbafeca) return 0 ;;
        *) return 1 ;;
    esac
}

textvn_uni_archs_of() {
    # In chuỗi arch của 1 Mach-O ra stdout; exit >0 nếu không probe được.
    local f="$1" lipo_bin out
    lipo_bin="${TEXTVN_UNI_LIPO:-lipo}"
    if command -v "$lipo_bin" >/dev/null 2>&1; then
        "$lipo_bin" -archs "$f" 2>/dev/null
        return $?
    fi
    if command -v file >/dev/null 2>&1; then
        out="$(file -b "$f" 2>/dev/null)" || return 2
        case "$out" in
            # vd "Mach-O universal binary with 2 architectures: [x86_64:…] [arm64:…]"
            # → đổi ':[]()' thành khoảng trắng rồi đối chiếu từng từ.
            *Mach-O*)
                out="$(printf '%s' "$out" | sed 's/[][():]/ /g')"
                printf '%s ' "$out"
                return 0 ;;
            *) return 2 ;;
        esac
    fi
    return 2
}

textvn_uni_scan_dir() {
    # Quét MỌI tệp Mach-O dưới $1; thiếu slice arm64 hoặc x86_64 ⇒ in danh sách
    # tệp lỗi ra stderr và return 1 (caller quyết định exit 1).
    local root="$1" f archs bad=0 count=0
    if [ ! -d "$root" ]; then
        echo "❌ ARM-M1 gate: không tìm thấy thư mục quét: $root" >&2
        return 1
    fi
    while IFS= read -r -d '' f; do
        textvn_uni_is_macho "$f" || continue
        count=$((count + 1))
        if ! archs="$(textvn_uni_archs_of "$f")"; then
            echo "❌ ARM-M1 gate: không xác định được kiến trúc của Mach-O: $f" >&2
            bad=1
            continue
        fi
        if ! textvn_uni_have_both_archs "$archs"; then
            echo "❌ ARM-M1 gate: Mach-O KHÔNG universal (cần arm64+x86_64, thực tế: $archs): $f" >&2
            bad=1
        fi
    done < <(find "$root" -type f -print0)
    # E94: 0 Mach-O = KHÔNG kiểm được gì (sai thư mục, bộ nhận diện magic hỏng…) — fail-closed,
    # không được in "đủ universal" cho một lần quét rỗng.
    if [ "$count" -eq 0 ]; then
        echo "❌ ARM-M1 gate: $root — 0 Mach-O (không có gì để kiểm — sai thư mục hoặc nhận diện magic hỏng)" >&2
        return 1
    fi
    if [ "$bad" -eq 1 ]; then
        echo "❌ ARM-M1 gate: $root — $count Mach-O, có tệp không universal" >&2
        return 1
    fi
    echo "✅ ARM-M1 gate: $root — $count Mach-O, đủ universal (arm64 + x86_64)"
    return 0
}

# Khi được `source` (package-macos-pkg.sh / test fixture) — chỉ nạp hàm gate,
# KHÔNG chạy body build dưới đây.
[ "${BASH_SOURCE[0]}" = "$0" ] || return 0

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Version = **nguồn duy nhất** `[workspace.package]` trong Cargo.toml (không hardcode 2 chỗ).
VERSION="$(grep -m1 '^version = "' "$ROOT/Cargo.toml" | cut -d'"' -f2)"
[ -n "$VERSION" ] || { echo "❌ không đọc được version từ Cargo.toml" >&2; exit 1; }
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
    cargo build --release --locked -p textvn-ffi --target aarch64-apple-darwin --manifest-path "$ROOT/Cargo.toml"
    cargo build --release --locked -p textvn-ffi --target x86_64-apple-darwin --manifest-path "$ROOT/Cargo.toml"

    lipo -create \
        "$ROOT/target/aarch64-apple-darwin/release/libtextvn_ffi.a" \
        "$ROOT/target/x86_64-apple-darwin/release/libtextvn_ffi.a" \
        -output "$ROOT/adapters/macos-imk/lib/libtextvn_ffi.a"
else
    echo "Compiling for host architecture..."
    cargo build --release --locked -p textvn-ffi --manifest-path "$ROOT/Cargo.toml"
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

# Icon .icns — menu Input Sources + Finder/Spotlight hiện icon cho input source
# và 2 bundle. PNG nguồn được commit (sinh bằng `cargo xtask gen-app-icons` —
# E59 thay generate_app_icons.py; từ resources/icons/textvn_v.svg); ưu tiên 1024 để mức 512@2x không phải phóng to.
ICON_SRC="$ROOT/packaging/macos/icons/TextVN-1024.png"
[ -f "$ICON_SRC" ] || ICON_SRC="$ROOT/packaging/macos/icons/TextVN-512.png"
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

# 5b. ARM-M1 gate — chặn trước khi ký: MỌI Mach-O trong 2 bundle phải universal.
#     Build host-only (thiếu 1 rust target / swift 1 arch) ĐỎ ở đây — đúng ý:
#     bản phát hành không được chứa binary 1 kiến trúc (OpenKey #189).
echo "=== ARM-M1 gate: universal arm64+x86_64 cho mọi Mach-O trong .app ==="
_uni_ok=1
textvn_uni_scan_dir "$IM_BUNDLE" || _uni_ok=0
textvn_uni_scan_dir "$APP_BUNDLE" || _uni_ok=0
if [ "$_uni_ok" -ne 1 ]; then
    echo "❌ ARM-M1: bundle chứa Mach-O thiếu slice arm64/x86_64 — KHÔNG ký, KHÔNG đóng gói." >&2
    echo "   Cần build đủ 2 arch: rustup target add aarch64-apple-darwin x86_64-apple-darwin" >&2
    echo "   + swift build --arch arm64 --arch x86_64 (build_swift tự thêm khi đủ 2 target)." >&2
    exit 1
fi

# 6. Codesign (ad-hoc `-` khi không có cert · Developer ID khi có secret — R2-48)
# KHÔNG dùng `--deep` (Apple deprecated): ký từng bundle với entitlements thật trong
# `packaging/macos/TextVN.entitlements` rồi `codesign --verify` để chắc chắn.
# APPLE_DEVELOPER_ID_APP = "Developer ID Application: <Tên> (<TEAMID>)" (tên cũ:
# DEVELOPER_ID); APPLE_KEYCHAIN (tuỳ chọn) = keychain chứa cert — xem
# scripts/macos-signing-keychain.sh. Trạng thái thật: docs/release/signing-status-mac.md.
SIGN_IDENTITY="${APPLE_DEVELOPER_ID_APP:-${DEVELOPER_ID:--}}"
SIGN_ARGS=(--force -s "$SIGN_IDENTITY" --entitlements "$ROOT/packaging/macos/TextVN.entitlements" --options runtime)
if [ "$SIGN_IDENTITY" != "-" ]; then
    # Notarization bắt buộc secure timestamp (Apple: "Resolving common notarization issues").
    SIGN_ARGS+=(--timestamp)
    if [ -n "${APPLE_KEYCHAIN:-}" ]; then
        SIGN_ARGS+=(--keychain "$APPLE_KEYCHAIN")
    fi
    echo "Signing bundles with Developer ID: $SIGN_IDENTITY"
else
    echo "ℹ️  Không có APPLE_DEVELOPER_ID_APP — bundle ký AD-HOC: Gatekeeper chặn bản tải về"
    echo "    (System Settings → Privacy & Security → Open Anyway). Xem docs/release/signing-status-mac.md."
fi

for bundle in "$IM_BUNDLE" "$APP_BUNDLE"; do
    codesign "${SIGN_ARGS[@]}" "$bundle"
    codesign --verify --strict --verbose=2 "$bundle"
done

# Notarize + staple từng .app TRƯỚC khi đóng zip/tar.gz (bước 7) — app trong archive
# phải mang sẵn ticket; zip không staple được. Chỉ khi có cert + xác thực notarytool.
NOTARIZED=0
if [ "$SIGN_IDENTITY" != "-" ]; then
    if "$ROOT/scripts/notarize-macos.sh" --available; then
        for bundle in "$IM_BUNDLE" "$APP_BUNDLE"; do
            "$ROOT/scripts/notarize-macos.sh" "$bundle"
        done
        NOTARIZED=1
    else
        echo "ℹ️  Đã ký Developer ID nhưng thiếu APPLE_NOTARY_* — bundle CHƯA notarize/staple."
        # Gatekeeper chỉ chấp nhận Developer ID + notarized → báo trước khi phát hành.
        spctl -a -vv "$IM_BUNDLE" || echo "⚠️  spctl từ chối TextVN-IM.app — cần notarize (MAC-055)"
        spctl -a -vv "$APP_BUNDLE" || echo "⚠️  spctl từ chối TextVN.app — cần notarize (MAC-055)"
    fi
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

if [ "$NOTARIZED" -eq 1 ]; then
    echo "✅ Bundle: Developer ID + notarized + stapled"
elif [ "$SIGN_IDENTITY" != "-" ]; then
    echo "⚠️  Bundle: Developer ID, CHƯA notarize"
else
    echo "⚠️  Bundle: ad-hoc (chưa Developer ID, chưa notarize)"
fi
echo "✅ Build completed successfully:"
echo "   - $TARBALL"
echo "   - $ZIPFILE"
echo "   - $DIST_DIR/SHA256SUMS.txt"
echo ""
echo "Bước tiếp theo:"
echo "  scripts/package-macos-pkg.sh   # đóng gói .pkg per-user + kiểm gỡ sạch (MAC-054)"
echo "  scripts/install_macos.sh       # cài per-user ngay trên máy này (Rule S5)"
