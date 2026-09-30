#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# notarize-macos.sh — notarize + staple artifact macOS (MAC-055, P2-4 §5, gate P2-5 §6).
#
# Dùng:
#   scripts/notarize-macos.sh dist/macos/TextVN-mac-v0.1.0.pkg
#   scripts/notarize-macos.sh dist/macos/stage/TextVN.app     # staple app rồi mới zip
#
# Xác thực notarytool — chọn 1 trong 2 (không dùng mật khẩu Apple ID):
#   (a) keychain profile:  xcrun notarytool store-credentials textvn-notary \
#                            --apple-id <id> --team-id <team> --password <app-specific>
#       → NOTARY_PROFILE=textvn-notary scripts/notarize-macos.sh <artifact>
#   (b) biến môi trường:   APPLE_ID + APPLE_TEAM_ID + APPLE_APP_PASSWORD
#
# Lưu ý: `.zip` KHÔNG staple được — staple từng `.app` rồi mới zip lại
# (`ditto -c -k --sequesterRsrc --keepParent`).
set -euo pipefail

ARTIFACT="${1:-}"
if [ -z "$ARTIFACT" ]; then
    echo "usage: notarize-macos.sh <file.pkg|dir.app>" >&2
    exit 2
fi
if [ ! -e "$ARTIFACT" ]; then
    echo "❌ không thấy artifact: $ARTIFACT" >&2
    exit 1
fi
case "$ARTIFACT" in
    *.zip) echo "❌ .zip không staple được — staple từng .app rồi zip lại (ditto -c -k --keepParent)" >&2; exit 2 ;;
esac
command -v xcrun >/dev/null 2>&1 || { echo "❌ cần Xcode CLT (xcrun)" >&2; exit 1; }

if [ -n "${NOTARY_PROFILE:-}" ]; then
    AUTH=(--keychain-profile "$NOTARY_PROFILE")
elif [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ] && [ -n "${APPLE_APP_PASSWORD:-}" ]; then
    AUTH=(--apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD")
else
    echo "❌ thiếu xác thực: đặt NOTARY_PROFILE, hoặc APPLE_ID + APPLE_TEAM_ID + APPLE_APP_PASSWORD" >&2
    echo "   (app-specific password tạo tại appleid.apple.com — KHÔNG dùng mật khẩu Apple ID)" >&2
    exit 2
fi

echo "=== notarytool submit --wait ($ARTIFACT) ==="
xcrun notarytool submit "$ARTIFACT" "${AUTH[@]}" --wait

echo "=== stapler staple + validate ==="
xcrun stapler staple "$ARTIFACT"
xcrun stapler validate "$ARTIFACT"

echo "=== Gatekeeper (spctl) ==="
# spctl chỉ "accept" khi artifact đã ký Developer ID **và** notarized; dùng làm gate release.
case "$ARTIFACT" in
    *.pkg) spctl -a -vv -t install "$ARTIFACT" ;;
    *.app) spctl -a -vv -t exec "$ARTIFACT" ;;
    *) echo "❌ chỉ hỗ trợ .pkg hoặc .app" >&2; exit 2 ;;
esac

echo "✅ notarized + stapled: $ARTIFACT"
