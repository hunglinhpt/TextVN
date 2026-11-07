#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# notarize-macos.sh — notarize + staple artifact macOS (MAC-055, P2-4 §5, gate P2-5 §6).
#
# Dùng:
#   scripts/notarize-macos.sh dist/macos/TextVN-mac-v<version>.pkg   # pkg đã ký Developer ID Installer
#   scripts/notarize-macos.sh dist/macos/stage/TextVN.app            # app đã ký Developer ID Application
#   scripts/notarize-macos.sh --available                             # exit 0 nếu đã cấu hình xác thực
#
# Notary service KHÔNG nhận thư mục .app: bản .app được nén tạm bằng
# `ditto -c -k --keepParent`, gửi file zip, rồi staple vào chính .app (staple
# trước khi build-macos.sh tạo zip/tar.gz phát hành). `.zip` không staple được.
#
# Xác thực notarytool — chọn 1 (không dùng mật khẩu Apple ID thật):
#   (a) keychain profile : APPLE_NOTARY_PROFILE (xcrun notarytool store-credentials …)
#   (b) App Store Connect API key (khuyên dùng cho CI):
#                          APPLE_NOTARY_KEY_ID + APPLE_NOTARY_ISSUER_ID +
#                          APPLE_NOTARY_KEY (nội dung .p8) hoặc APPLE_NOTARY_KEY_PATH
#   (c) Apple ID         : APPLE_NOTARY_APPLE_ID + APPLE_NOTARY_TEAM_ID +
#                          APPLE_NOTARY_PASSWORD (app-specific password)
# Tên cũ vẫn nhận: NOTARY_PROFILE; APPLE_ID + APPLE_TEAM_ID + APPLE_APP_PASSWORD.
# Trạng thái ký/notarize thật của bản phát hành: docs/release/signing-status-mac.md.
set -euo pipefail

NOTARY_PROFILE_VALUE="${APPLE_NOTARY_PROFILE:-${NOTARY_PROFILE:-}}"
NOTARY_APPLE_ID="${APPLE_NOTARY_APPLE_ID:-${APPLE_ID:-}}"
NOTARY_TEAM_ID="${APPLE_NOTARY_TEAM_ID:-${APPLE_TEAM_ID:-}}"
NOTARY_PASSWORD="${APPLE_NOTARY_PASSWORD:-${APPLE_APP_PASSWORD:-}}"

# 0 = có đủ một bộ xác thực notarytool.
notary_configured() {
    [ -n "$NOTARY_PROFILE_VALUE" ] && return 0
    if [ -n "${APPLE_NOTARY_KEY_ID:-}" ] && [ -n "${APPLE_NOTARY_ISSUER_ID:-}" ] &&
        { [ -n "${APPLE_NOTARY_KEY:-}" ] || [ -n "${APPLE_NOTARY_KEY_PATH:-}" ]; }; then
        return 0
    fi
    [ -n "$NOTARY_APPLE_ID" ] && [ -n "$NOTARY_TEAM_ID" ] && [ -n "$NOTARY_PASSWORD" ]
}

ARTIFACT="${1:-}"
if [ "$ARTIFACT" = "--available" ]; then
    if notary_configured; then exit 0; else exit 1; fi
fi
if [ -z "$ARTIFACT" ]; then
    echo "usage: notarize-macos.sh <file.pkg|file.dmg|dir.app> | --available" >&2
    exit 2
fi
if [ ! -e "$ARTIFACT" ]; then
    echo "❌ không thấy artifact: $ARTIFACT" >&2
    exit 1
fi
ARTIFACT="${ARTIFACT%/}"
case "$ARTIFACT" in
    *.zip) echo "❌ .zip không staple được — notarize từng .app (script tự nén) rồi mới zip lại" >&2; exit 2 ;;
    *.app|*.pkg|*.dmg) ;;
    *) echo "❌ chỉ hỗ trợ .app, .pkg hoặc .dmg" >&2; exit 2 ;;
esac
command -v xcrun >/dev/null 2>&1 || { echo "❌ cần Xcode CLT (xcrun)" >&2; exit 1; }

if ! notary_configured; then
    echo "❌ thiếu xác thực notarytool: APPLE_NOTARY_PROFILE, hoặc APPLE_NOTARY_KEY_ID +" >&2
    echo "   APPLE_NOTARY_ISSUER_ID + APPLE_NOTARY_KEY(_PATH), hoặc APPLE_NOTARY_APPLE_ID +" >&2
    echo "   APPLE_NOTARY_TEAM_ID + APPLE_NOTARY_PASSWORD (app-specific password, KHÔNG phải mật khẩu Apple ID)" >&2
    exit 2
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/textvn-notary.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

if [ -n "$NOTARY_PROFILE_VALUE" ]; then
    AUTH=(--keychain-profile "$NOTARY_PROFILE_VALUE")
elif [ -n "${APPLE_NOTARY_KEY_ID:-}" ] && [ -n "${APPLE_NOTARY_ISSUER_ID:-}" ] &&
    { [ -n "${APPLE_NOTARY_KEY:-}" ] || [ -n "${APPLE_NOTARY_KEY_PATH:-}" ]; }; then
    KEY_FILE="${APPLE_NOTARY_KEY_PATH:-}"
    if [ -z "$KEY_FILE" ]; then
        KEY_FILE="$WORK/AuthKey.p8"
        (umask 077 && printf '%s\n' "$APPLE_NOTARY_KEY" > "$KEY_FILE")
    fi
    AUTH=(--key "$KEY_FILE" --key-id "$APPLE_NOTARY_KEY_ID" --issuer "$APPLE_NOTARY_ISSUER_ID")
else
    AUTH=(--apple-id "$NOTARY_APPLE_ID" --team-id "$NOTARY_TEAM_ID" --password "$NOTARY_PASSWORD")
fi

# Notary từ chối code ký ad-hoc / pkg chưa ký — báo sớm, rõ ràng.
SUBMIT="$ARTIFACT"
case "$ARTIFACT" in
    *.app)
        # Đọc hết rồi mới so (pipefail + grep -q có thể làm codesign dính SIGPIPE).
        SIGN_INFO="$(codesign -dv --verbose=2 "$ARTIFACT" 2>&1 || true)"
        case "$SIGN_INFO" in
            *"Authority=Developer ID Application"*) ;;
            *) echo "❌ $ARTIFACT chưa ký Developer ID Application (đặt APPLE_DEVELOPER_ID_APP rồi chạy build-macos.sh)" >&2
               exit 1 ;;
        esac
        SUBMIT="$WORK/$(basename "$ARTIFACT" .app).zip"
        ditto -c -k --sequesterRsrc --keepParent "$ARTIFACT" "$SUBMIT"
        ;;
    *.pkg)
        SIGN_INFO="$(pkgutil --check-signature "$ARTIFACT" 2>&1 || true)"
        case "$SIGN_INFO" in
            *"Developer ID Installer"*) ;;
            *) echo "❌ $ARTIFACT chưa ký Developer ID Installer (đặt APPLE_DEVELOPER_ID_INSTALLER)" >&2
               exit 1 ;;
        esac
        ;;
esac

echo "=== notarytool submit --wait ($(basename "$SUBMIT")) ==="
RESULT="$WORK/submit.json"
xcrun notarytool submit "$SUBMIT" "${AUTH[@]}" --wait --output-format json > "$RESULT" || true
STATUS="$(plutil -extract status raw -o - "$RESULT" 2>/dev/null || echo unknown)"
SUBMISSION_ID="$(plutil -extract id raw -o - "$RESULT" 2>/dev/null || echo "")"
echo "   id=${SUBMISSION_ID:-?} status=$STATUS"
if [ "$STATUS" != "Accepted" ]; then
    echo "❌ notarization không được chấp nhận ($STATUS)" >&2
    if [ -n "$SUBMISSION_ID" ]; then
        xcrun notarytool log "$SUBMISSION_ID" "${AUTH[@]}" >&2 || true
    fi
    exit 1
fi

echo "=== stapler staple + validate ==="
xcrun stapler staple "$ARTIFACT"
xcrun stapler validate "$ARTIFACT"

echo "=== Gatekeeper (spctl) ==="
# spctl chỉ "accept" khi artifact đã ký Developer ID **và** notarized; dùng làm gate release.
case "$ARTIFACT" in
    *.pkg) spctl -a -vv -t install "$ARTIFACT" ;;
    *.app) spctl -a -vv -t exec "$ARTIFACT" ;;
    *.dmg) spctl -a -vv -t open --context context:primary-signature "$ARTIFACT" ;;
esac

echo "✅ notarized + stapled: $ARTIFACT"
