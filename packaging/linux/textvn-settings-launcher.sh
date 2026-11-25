#!/usr/bin/env sh
# SPDX-License-Identifier: GPL-3.0-or-later
# textvn-settings — trình khởi chạy bảng điều khiển TextVN cho Linux (E67).
# Tự chọn GTK4 (Ubuntu 24.04 / Fedora / Arch / Debian 12+) hoặc GTK3 (RHEL 8.10 /
# Rocky 8 / AlmaLinux 8 / Ubuntu 20.04-22.04) theo thư viện sẵn có trên máy.

SELF="$(readlink -f -- "$0" 2>/dev/null || printf '%s' "$0")"
SELF_DIR="$(CDPATH='' cd -- "$(dirname -- "$SELF")" && pwd -P)" || exit 1
LIB_DIR="$SELF_DIR/../lib/textvn"

if [ -x "$LIB_DIR/textvn-settings-gtk4" ] && "$LIB_DIR/textvn-settings-gtk4" --check-abi >/dev/null 2>&1; then
    exec "$LIB_DIR/textvn-settings-gtk4" "$@"
fi
if [ -x "$LIB_DIR/textvn-settings-gtk3" ] && "$LIB_DIR/textvn-settings-gtk3" --check-abi >/dev/null 2>&1; then
    exec "$LIB_DIR/textvn-settings-gtk3" "$@"
fi

echo "✗ Không mở được bảng điều khiển TextVN (thiếu thư viện GTK3/GTK4 trên hệ thống)." >&2
echo "  • RHEL / Rocky / Alma / Fedora : sudo dnf install -y gtk3" >&2
echo "  • Ubuntu / Debian              : sudo apt install -y libgtk-3-0" >&2
echo "  • Dùng CLI không cần GUI       : textvn doctor (cấu hình ở ~/.config/TextVN/config.json)" >&2
exit 1
