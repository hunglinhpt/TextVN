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

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
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
RECOVERY="$RUNTIME/recovery"

runtime_put() { # <filename> <one-line value>
    printf '%s\n' "$2" > "$RUNTIME/$1"
}

runtime_get() { # <filename>
    sed -n '1p' "$RUNTIME/$1" 2>/dev/null
}

snapshot_manager_environment() { # <VARIABLE>
    local key="$1" all value
    if ! tv_systemd_user_ok; then
        runtime_put "manager-$key" unavailable
        return 0
    fi
    all="$(systemctl --user show-environment)"
    if printf '%s\n' "$all" | grep -q "^${key}="; then
        value="$(printf '%s\n' "$all" | sed -n "s/^${key}=//p" | head -n1)"
        {
            printf 'set\n'
            printf '%s\n' "$value"
        } > "$RUNTIME/manager-$key"
    else
        runtime_put "manager-$key" unset
    fi
}

restore_manager_environment() { # <VARIABLE>
    local key="$1" state value
    [[ -f "$RUNTIME/manager-$key" ]] || return 0
    state="$(runtime_get "manager-$key")"
    case "$state" in
        unavailable|"") return 0 ;;
        set)
            value="$(sed -n '2p' "$RUNTIME/manager-$key")"
            systemctl --user set-environment "$key=$value"
            ;;
        unset) systemctl --user unset-environment "$key" ;;
        *) tv_err "Snapshot môi trường portable không hợp lệ cho $key."; return 1 ;;
    esac
}

snapshot_portable_state() { # <ibus|fcitx5>
    local fw="$1" process key value was_running=0
    case "$fw" in
        ibus) process=ibus-daemon; key=IBUS_COMPONENT_PATH ;;
        fcitx5) process=fcitx5; key=FCITX_ADDON_DIRS ;;
        *) return 1 ;;
    esac
    tv_running "$process" && was_running=1
    runtime_put framework "$fw"
    runtime_put package "$PKG_DIR"
    runtime_put was-running "$was_running"
    snapshot_manager_environment "$key"
    value="$(tv_process_environment "$process" "$key" || true)"
    runtime_put "original-$fw-environment" "$value"
    # Nếu shell bị ngắt giữa takeover và ghi `active`, lần sau vẫn có đường
    # khôi phục thay vì coi thư mục runtime là rác và xóa mất snapshot.
    runtime_put recovery "$fw"
}

session_framework() {
    local fw
    if [[ -f "$MARK" ]]; then
        fw="$(cat "$MARK")"
    else
        fw="$(cat "$RECOVERY")"
    fi
    case "$fw" in ibus|fcitx5) printf '%s\n' "$fw" ;;
        *) tv_err "Marker portable không hợp lệ."; return 1 ;;
    esac
}

restore_ibus() {
    local was original
    was="$(runtime_get was-running)"
    original="$(runtime_get original-ibus-environment)"
    case "$was" in
        0)
            tv_stop_ibus
            restore_manager_environment IBUS_COMPONENT_PATH
            ;;
        1)
            # Khôi phục biến manager trước khi restart unit; fallback không dùng
            # manager nhận đúng biến môi trường của daemon trước portable.
            restore_manager_environment IBUS_COMPONENT_PATH
            TV_SKIP_MANAGER_ENV=1 tv_restart_ibus "$original"
            ;;
        *) tv_err "Không đọc được trạng thái IBus trước portable."; return 1 ;;
    esac
}

restore_fcitx5() {
    local was original profile_action=""
    was="$(runtime_get was-running)"
    original="$(runtime_get original-fcitx5-environment)"
    [[ -f "$RUNTIME/fcitx-profile-added" ]] && profile_action=remove
    case "$was" in
        0)
            tv_stop_fcitx5
            [[ -z "$profile_action" ]] || tv_deactivate_fcitx5
            restore_manager_environment FCITX_ADDON_DIRS
            ;;
        1)
            restore_manager_environment FCITX_ADDON_DIRS
            TV_SKIP_MANAGER_ENV=1 tv_restart_fcitx5 "$original" "" "$profile_action"
            ;;
        *) tv_err "Không đọc được trạng thái Fcitx5 trước portable."; return 1 ;;
    esac
}

restore_portable_state() {
    case "$1" in
        ibus) restore_ibus ;;
        fcitx5) restore_fcitx5 ;;
        *) return 1 ;;
    esac
}

rollback_start() { # <framework> <message>
    local fw="$1" message="$2"
    printf '%s\n' "$fw" > "$RECOVERY"
    if restore_portable_state "$fw"; then
        rm -rf -- "$RUNTIME"
    else
        tv_err "Khôi phục phiên trước portable chưa xong; chạy '$0 stop' để thử lại."
    fi
    tv_err "$message"
    exit 1
}

prepare_runtime() {
    if [[ -e "$RUNTIME" || -L "$RUNTIME" ]]; then
        if [[ -L "$RUNTIME" || ! -d "$RUNTIME" || ! -O "$RUNTIME" ]]; then
            tv_err "$RUNTIME không thuộc về bạn hoặc không phải thư mục — dừng lại vì an toàn."
            exit 1
        fi
        # Không tự xóa nội dung không có marker: có thể là dữ liệu của phiên
        # portable cũ/corrupt hoặc file người dùng. Chỉ rmdir nếu trống.
        if ! rmdir -- "$RUNTIME" 2>/dev/null; then
            tv_err "$RUNTIME có dữ liệu chưa rõ nguồn gốc; kiểm tra thủ công trước khi bật portable."
            exit 1
        fi
    fi
    mkdir -p -m 0700 "$RUNTIME"
    if [[ -L "$RUNTIME" || ! -d "$RUNTIME" || ! -O "$RUNTIME" ]]; then
        tv_err "$RUNTIME không thuộc về bạn — dừng lại vì an toàn."
        exit 1
    fi
    chmod 0700 "$RUNTIME"
}

# Marker liên kết phiên với đúng thư mục portable đã tạo nó. Bản giải nén thứ
# hai chỉ được xem trạng thái, không được dừng/restart IME của bản thứ nhất.
if [[ -f "$MARK" || -f "$RECOVERY" ]]; then
    MARK_FW="$(session_framework)" || exit 1
    MARK_PACKAGE="$(runtime_get package)"
    [[ -n "$MARK_PACKAGE" ]] || { tv_err "Thiếu nguồn gốc của phiên portable; không tự ý sửa IME."; exit 1; }
    if [[ "$ACTION" == status ]]; then
        if [[ -f "$RECOVERY" ]]; then
            tv_err "TextVN portable cần khôi phục phiên $MARK_FW từ $MARK_PACKAGE."
        else
            tv_ok "TextVN portable đang bật với $MARK_FW (từ $MARK_PACKAGE)."
        fi
        exit 0
    fi
    [[ "$MARK_PACKAGE" == "$PKG_DIR" ]] || {
        tv_err "Phiên portable đang thuộc $MARK_PACKAGE; dùng chính thư mục đó để dừng an toàn."
        exit 1
    }
    if [[ -n "$FW" && "$FW" != "$MARK_FW" ]]; then
        tv_err "Marker portable đang dùng $MARK_FW, không phải $FW."
        exit 1
    fi
    if [[ "$ACTION" == stop ]]; then
        restore_portable_state "$MARK_FW"
        rm -rf -- "$RUNTIME"
        tv_ok "Đã tắt TextVN portable và khôi phục phiên trước đó."
        exit 0
    fi
    [[ -f "$RECOVERY" ]] && {
        tv_err "Phiên trước chưa khôi phục xong; chạy '$0 stop' trước khi bật lại."
        exit 1
    }
    tv_ok "TextVN portable đã đang bật ($MARK_FW)."
    exit 0
fi

# `status`/`stop` không được đoán framework: một thư mục portable chưa từng bật
# không có quyền khởi động lại IBus/Fcitx hoặc sửa profile của bản cài thường.
if [[ "$ACTION" == status ]]; then
    echo "TextVN portable đang tắt."
    exit 0
fi
if [[ "$ACTION" == stop ]]; then
    tv_ok "TextVN portable đã tắt (không có phiên portable đang hoạt động)."
    exit 0
fi

if [[ -z "$FW" ]]; then
    if tv_running fcitx5 && [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
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

prepare_runtime
SETTINGS="$PKG_DIR/bin/textvn-settings"

if [[ "$FW" == ibus ]]; then
    [[ -x "$PKG_DIR/lib/textvn/textvn-ibus-engine" ]] || { tv_err "Gói không có engine IBus."; exit 1; }
    snapshot_portable_state ibus
    mkdir -p "$RUNTIME/ibus/component"
    tv_ibus_component "$PKG_DIR" "$PKG_DIR/lib/textvn/textvn-ibus-engine" "$SETTINGS" \
        > "$RUNTIME/ibus/component/textvn.xml"
    tv_restart_ibus "$RUNTIME/ibus/component:/usr/share/ibus/component" ||
        rollback_start ibus "Không thể khởi động lại IBus cho TextVN portable."
    # Chọn TextVN cho phiên này (không sửa danh sách bộ gõ lâu dài).
    active=0
    for _ in 1 2 3 4 5; do
        if ibus engine textvn >/dev/null 2>&1 && [[ "$(ibus engine 2>/dev/null)" == textvn ]]; then
            active=1
            break
        fi
        sleep 1
    done
    if [[ "$active" != 1 ]]; then
        rollback_start ibus "IBus không kích hoạt được TextVN; không đánh dấu phiên portable là thành công."
    fi
else
    [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]] || { tv_err "Gói không có addon Fcitx5."; exit 1; }
    snapshot_portable_state fcitx5
    mkdir -p "$RUNTIME/data/fcitx5/addon" "$RUNTIME/data/fcitx5/inputmethod"
    cp "$PKG_DIR/share/fcitx5/addon/textvn.conf" "$RUNTIME/data/fcitx5/addon/"
    cp "$PKG_DIR/share/fcitx5/inputmethod/textvn.conf" "$RUNTIME/data/fcitx5/inputmethod/"
    sys="$(tv_fcitx5_system_addon_dir)"
    was_running="$(runtime_get was-running)"
    profile_action=add
    if grep -qx 'Name=textvn' "$(tv_fcitx5_profile)" 2>/dev/null; then
        profile_action=""
    else
        : > "$RUNTIME/fcitx-profile-added"
    fi
    tv_restart_fcitx5 "$PKG_DIR/lib/textvn/fcitx5${sys:+:$sys}" "$RUNTIME/data" "$profile_action" ||
        rollback_start fcitx5 "Không thể khởi động lại Fcitx5 cho TextVN portable."
    if [[ "$was_running" == 0 ]]; then
        FCITX_ADDON_DIRS="$PKG_DIR/lib/textvn/fcitx5${sys:+:$sys}" \
            XDG_DATA_DIRS="$RUNTIME/data:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" \
            setsid fcitx5 -d >/dev/null 2>&1 || true
        sleep 1
    fi
    active=0
    if command -v fcitx5-remote >/dev/null 2>&1; then
        for _ in 1 2 3 4 5; do
            if fcitx5-remote -s textvn >/dev/null 2>&1 &&
                fcitx5-remote -n 2>/dev/null | grep -qi '^textvn$'; then
                active=1
                break
            fi
            sleep 1
        done
    fi
    if [[ "$active" != 1 ]]; then
        rollback_start fcitx5 "Fcitx5 không kích hoạt được TextVN; không đánh dấu phiên portable là thành công."
    fi
fi
echo "$FW" > "$MARK"
rm -f -- "$RECOVERY"

tv_ok "TextVN đang chạy từ $PKG_DIR ($FW) — không cài gì vào hệ thống."
echo "  • Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (hoặc Ctrl + Shift + Space)."
[[ -x "$SETTINGS" ]] && echo "  • Bảng điều khiển: $SETTINGS"
echo "  • Tắt bản portable: $0 stop   (đăng xuất cũng tự tắt)."
