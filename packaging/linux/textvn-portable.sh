#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN — chạy ngay từ thư mục giải nén, KHÔNG cài gì vào hệ thống.
#
#   ./textvn-portable.sh            # bật TextVN cho phiên đăng nhập hiện tại
#   ./textvn-portable.sh stop       # trả IBus/Fcitx5 về như cũ
#   ./textvn-portable.sh status
#
# Tuỳ chọn: --ibus / --fcitx5 (mặc định: framework đang chạy; cả hai thì ưu tiên Fcitx5).
# Cách hoạt động: sinh component/addon trỏ vào thư mục này trong $XDG_RUNTIME_DIR, rồi khởi
# động lại IBus (IBUS_COMPONENT_PATH) hoặc Fcitx5 (FCITX_ADDON_DIRS + XDG_DATA_DIRS).
# Không có gì tồn tại qua lần đăng xuất; cấu hình ở ~/.config/TextVN dùng chung với bản cài.

set -euo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=packaging/linux/textvn-common.sh
. "$PKG_DIR/textvn-common.sh"

ACTION=start
FW=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        start|stop|status) ACTION="$1" ;;
        --ibus) FW=ibus ;;
        --fcitx5) FW=fcitx5 ;;
        -h|--help) sed -n '3,13p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) tv_err "Tuỳ chọn không hợp lệ: $1"; exit 1 ;;
    esac
    shift
done

tv_check_package "$PKG_DIR" || exit 1

RUNTIME="${XDG_RUNTIME_DIR:-/tmp}/textvn-portable-$(id -u)"
MARK="$RUNTIME/active"

if [[ -z "$FW" ]]; then
    if [[ -f "$MARK" ]]; then
        FW="$(cat "$MARK")"
    elif tv_running fcitx5 && [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
        FW=fcitx5
    elif tv_running ibus-daemon && [[ -x "$PKG_DIR/lib/textvn/textvn-ibus-engine" ]]; then
        FW=ibus
    elif command -v fcitx5 >/dev/null 2>&1 && [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
        FW=fcitx5
    elif command -v ibus-daemon >/dev/null 2>&1; then
        FW=ibus
    else
        tv_err "Không thấy IBus hay Fcitx5. Cài một trong hai (ví dụ: sudo apt install ibus) rồi chạy lại."
        exit 1
    fi
fi

case "$ACTION" in
status)
    if [[ -f "$MARK" ]]; then
        tv_ok "TextVN portable đang bật với $(cat "$MARK") (từ $PKG_DIR)."
    else
        echo "TextVN portable đang tắt."
    fi
    exit 0
    ;;
stop)
    if [[ "$FW" == ibus ]]; then
        tv_running ibus-daemon && tv_restart_ibus ""
    else
        tv_restart_fcitx5 "" "" remove
    fi
    rm -rf "$RUNTIME"
    tv_ok "Đã tắt TextVN portable."
    exit 0
    ;;
esac

mkdir -p "$RUNTIME"
chmod 0700 "$RUNTIME"
SETTINGS="$PKG_DIR/bin/textvn-settings"

if [[ "$FW" == ibus ]]; then
    [[ -x "$PKG_DIR/lib/textvn/textvn-ibus-engine" ]] || { tv_err "Gói không có engine IBus."; exit 1; }
    mkdir -p "$RUNTIME/ibus/component"
    tv_ibus_component "$PKG_DIR" "$PKG_DIR/lib/textvn/textvn-ibus-engine" "$SETTINGS" \
        > "$RUNTIME/ibus/component/textvn.xml"
    tv_restart_ibus "$RUNTIME/ibus/component:/usr/share/ibus/component"
    # Chọn TextVN cho phiên này (không sửa danh sách bộ gõ lâu dài).
    for _ in 1 2 3 4 5; do
        ibus engine textvn >/dev/null 2>&1 && break
        sleep 1
    done
else
    [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]] || { tv_err "Gói không có addon Fcitx5."; exit 1; }
    mkdir -p "$RUNTIME/data/fcitx5/addon" "$RUNTIME/data/fcitx5/inputmethod"
    cp "$PKG_DIR/share/fcitx5/addon/textvn.conf" "$RUNTIME/data/fcitx5/addon/"
    cp "$PKG_DIR/share/fcitx5/inputmethod/textvn.conf" "$RUNTIME/data/fcitx5/inputmethod/"
    sys="$(tv_fcitx5_system_addon_dir)"
    was_running=0
    tv_running fcitx5 && was_running=1
    tv_restart_fcitx5 "$PKG_DIR/lib/textvn/fcitx5${sys:+:$sys}" "$RUNTIME/data" add
    if [[ "$was_running" == 0 ]]; then
        FCITX_ADDON_DIRS="$PKG_DIR/lib/textvn/fcitx5${sys:+:$sys}" \
            XDG_DATA_DIRS="$RUNTIME/data:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" \
            setsid fcitx5 -d >/dev/null 2>&1 || true
        sleep 1
    fi
    command -v fcitx5-remote >/dev/null 2>&1 && fcitx5-remote -s textvn >/dev/null 2>&1 || true
fi
echo "$FW" > "$MARK"

tv_ok "TextVN đang chạy từ $PKG_DIR ($FW) — không cài gì vào hệ thống."
echo "  • Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (hoặc Ctrl + Shift + Space)."
[[ -x "$SETTINGS" ]] && echo "  • Bảng điều khiển: $SETTINGS"
echo "  • Tắt bản portable: $0 stop   (đăng xuất cũng tự tắt)."
