# SPDX-License-Identifier: GPL-3.0-or-later
# textvn-common.sh — hàm dùng chung cho install.sh / uninstall.sh / textvn-portable.sh.
# Được `source`, không chạy trực tiếp.

if [[ -t 1 ]]; then
    TV_C_OK=$'\033[0;32m' TV_C_ERR=$'\033[0;31m' TV_C_INFO=$'\033[0;36m' TV_C_OFF=$'\033[0m'
else
    TV_C_OK="" TV_C_ERR="" TV_C_INFO="" TV_C_OFF=""
fi
tv_say() { echo "${TV_C_INFO}==>${TV_C_OFF} $*"; }
tv_ok() { echo "${TV_C_OK}✓${TV_C_OFF} $*"; }
tv_err() { echo "${TV_C_ERR}✗ $*${TV_C_OFF}" >&2; }

tv_version() { cat "$1/VERSION" 2>/dev/null || echo "?"; }

# Gói hợp lệ: có CLI và ít nhất một adapter.
tv_check_package() {
    local d="$1"
    if [[ ! -x "$d/bin/textvn" ]]; then
        tv_err "Không phải thư mục gói TextVN (thiếu bin/textvn): $d"
        return 1
    fi
    if [[ ! -x "$d/lib/textvn/textvn-ibus-engine" && ! -f "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
        tv_err "Gói không có adapter IBus lẫn Fcitx5."
        return 1
    fi
}

# Thư mục addon hệ thống của Fcitx5 (Debian/Ubuntu multiarch, Fedora lib64, Arch lib).
tv_fcitx5_system_addon_dir() {
    local d
    for d in /usr/lib/x86_64-linux-gnu/fcitx5 /usr/lib/aarch64-linux-gnu/fcitx5 \
             /usr/lib64/fcitx5 /usr/lib/fcitx5 /usr/local/lib/fcitx5; do
        if compgen -G "$d/lib*.so" >/dev/null 2>&1; then
            echo "$d"
            return 0
        fi
    done
    echo ""
}

# Component XML của IBus trỏ tới engine/bảng cài đặt ở đường dẫn thật.
tv_ibus_component() { # <pkg_dir> <engine_path> <settings_path>
    sed -e "s|/usr/lib/textvn/textvn-ibus-engine|$2|g" \
        -e "s|/usr/bin/textvn-settings|$3|g" \
        "$1/share/ibus/component/textvn.xml"
}

tv_running() { pgrep -x -u "$(id -u)" "$1" >/dev/null 2>&1; }

# ibus-daemon giữ cache registry (~/.cache/ibus/bus/registry) và chỉ kiểm tra thay đổi ở
# các thư mục component ĐÃ biết — thêm thư mục qua IBUS_COMPONENT_PATH không làm cache mất
# hiệu lực, TextVN không hiện dù đã cài. Xoá cache (ibus tự dựng lại) mỗi khi đổi component.
tv_ibus_clear_cache() {
    rm -f "${XDG_CACHE_HOME:-$HOME/.cache}/ibus/bus/registry" 2>/dev/null || true
}

tv_systemd_user_ok() {
    command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1
}

# Khởi động lại ibus-daemon với IBUS_COMPONENT_PATH — giữ nguyên tham số của daemon đang
# chạy (GNOME: `--panel disable`). Phiên systemd (GNOME ≥ 43): đổi môi trường của user
# manager rồi restart đúng unit; còn lại: chạy lại daemon với cùng tham số.
tv_restart_ibus() { # <component_path | "">
    local comp="$1" unit pid args=()
    tv_ibus_clear_cache
    if tv_systemd_user_ok; then
        if [[ -n "$comp" ]]; then
            systemctl --user set-environment "IBUS_COMPONENT_PATH=$comp" 2>/dev/null || true
        else
            systemctl --user unset-environment IBUS_COMPONENT_PATH 2>/dev/null || true
        fi
        unit="$(systemctl --user list-units --state=active --no-legend 'org.freedesktop.IBus.session.*' 2>/dev/null | awk '{print $1; exit}')"
        if [[ -n "$unit" ]]; then
            systemctl --user restart "$unit" && tv_ok "Đã khởi động lại IBus ($unit)." && return 0
        fi
    fi
    pid="$(pgrep -x -u "$(id -u)" ibus-daemon | head -n1)"
    if [[ -n "$pid" && -r "/proc/$pid/cmdline" ]]; then
        mapfile -d '' args < "/proc/$pid/cmdline"
        args=("${args[@]:1}")
    fi
    [[ ${#args[@]} -eq 0 ]] && args=(--xim)
    local keep=()
    for a in "${args[@]}"; do
        case "$a" in -d|--daemonize|-r|--replace|-dr|-rd|-drx|-dxr|-rdx|-rxd|-xdr|-xrd) ;;
            *) keep+=("$a") ;; esac
    done
    if [[ -n "$comp" ]]; then
        IBUS_COMPONENT_PATH="$comp" setsid ibus-daemon "${keep[@]}" --replace --daemonize
    else
        env -u IBUS_COMPONENT_PATH setsid ibus-daemon "${keep[@]}" --replace --daemonize
    fi
    sleep 1
    tv_ok "Đã khởi động lại IBus."
}

# Dừng fcitx5 (nếu chạy), tuỳ chọn sửa profile, rồi chạy lại với FCITX_ADDON_DIRS và thư
# mục dữ liệu thêm. Profile phải sửa khi fcitx5 đã dừng: fcitx5 ghi đè profile khi thoát.
tv_restart_fcitx5() { # <addon_dirs | ""> <extra_xdg_data_dir | ""> <profile_action: add|remove|"">
    local addons="$1" extra="$2" action="${3:-}" data="${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
    local was_running=0 i pid args=() keep=() a
    [[ -n "$extra" ]] && data="$extra:$data"
    if tv_running fcitx5; then
        was_running=1
        # Giữ tham số của tiến trình đang chạy (ví dụ --disable/--enable của phiên).
        pid="$(pgrep -x -u "$(id -u)" fcitx5 | head -n1)"
        if [[ -n "$pid" && -r "/proc/$pid/cmdline" ]]; then
            mapfile -d '' args < "/proc/$pid/cmdline"
            for a in "${args[@]:1}"; do
                case "$a" in -d|-r|-rd|-dr|--replace) ;; *) keep+=("$a") ;; esac
            done
        fi
        command -v fcitx5-remote >/dev/null 2>&1 && fcitx5-remote -e >/dev/null 2>&1 || true
        for i in $(seq 1 50); do tv_running fcitx5 || break; sleep 0.1; done
        tv_running fcitx5 && pkill -x -u "$(id -u)" fcitx5 || true
    fi
    case "$action" in
        add) tv_activate_fcitx5 ;;
        remove) tv_deactivate_fcitx5 ;;
    esac
    if tv_systemd_user_ok; then
        if [[ -n "$addons" ]]; then
            systemctl --user set-environment "FCITX_ADDON_DIRS=$addons" 2>/dev/null || true
        else
            systemctl --user unset-environment FCITX_ADDON_DIRS 2>/dev/null || true
        fi
    fi
    [[ "$was_running" == 1 ]] || return 0
    if [[ -n "$addons" ]]; then
        FCITX_ADDON_DIRS="$addons" XDG_DATA_DIRS="$data" setsid fcitx5 -d "${keep[@]}" >/dev/null 2>&1 || true
    else
        env -u FCITX_ADDON_DIRS XDG_DATA_DIRS="$data" setsid fcitx5 -d "${keep[@]}" >/dev/null 2>&1 || true
    fi
    sleep 1
    tv_ok "Đã khởi động lại Fcitx5."
}

# Thêm TextVN vào danh sách bộ gõ: GNOME input-sources, hoặc preload-engines của IBus.
# Chỉ sửa khi định dạng giá trị đúng như mong đợi; không có gsettings/schema thì bỏ qua.
tv_activate_ibus() {
    command -v gsettings >/dev/null 2>&1 || return 0
    local cur new
    if gsettings list-schemas 2>/dev/null | grep -qx org.gnome.desktop.input-sources; then
        cur="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null)" || return 0
        [[ "$cur" == *"'textvn'"* ]] && return 0
        case "$cur" in
            "@a(ss) []"|"[]") new="[('ibus', 'textvn')]" ;;
            \[*\]) new="${cur%]}, ('ibus', 'textvn')]" ;;
            *) return 0 ;;
        esac
        gsettings set org.gnome.desktop.input-sources sources "$new" &&
            tv_ok "Đã thêm TextVN vào nguồn nhập của GNOME (Super+Space để chuyển)."
    elif gsettings list-schemas 2>/dev/null | grep -qx org.freedesktop.ibus.general; then
        cur="$(gsettings get org.freedesktop.ibus.general preload-engines 2>/dev/null)" || return 0
        [[ "$cur" == *"'textvn'"* ]] && return 0
        case "$cur" in
            "@as []"|"[]") new="['textvn']" ;;
            \[*\]) new="${cur%]}, 'textvn']" ;;
            *) return 0 ;;
        esac
        gsettings set org.freedesktop.ibus.general preload-engines "$new" &&
            tv_ok "Đã thêm TextVN vào danh sách engine của IBus."
    fi
}

tv_deactivate_ibus() {
    command -v gsettings >/dev/null 2>&1 || return 0
    local cur new
    if gsettings list-schemas 2>/dev/null | grep -qx org.gnome.desktop.input-sources; then
        cur="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null)" || return 0
        [[ "$cur" == *"('ibus', 'textvn')"* ]] || return 0
        new="${cur//, (\'ibus\', \'textvn\')/}"
        new="${new//(\'ibus\', \'textvn\'), /}"
        new="${new//(\'ibus\', \'textvn\')/}"
        [[ "$new" == "[]" ]] && new="@a(ss) []"
        gsettings set org.gnome.desktop.input-sources sources "$new" || true
    fi
    if gsettings list-schemas 2>/dev/null | grep -qx org.freedesktop.ibus.general; then
        cur="$(gsettings get org.freedesktop.ibus.general preload-engines 2>/dev/null)" || return 0
        [[ "$cur" == *"'textvn'"* ]] || return 0
        new="${cur//, \'textvn\'/}"
        new="${new//\'textvn\', /}"
        new="${new//\'textvn\'/}"
        [[ "$new" == "[]" ]] && new="@as []"
        gsettings set org.freedesktop.ibus.general preload-engines "$new" || true
    fi
}

# Profile Fcitx5 (~/.config/fcitx5/profile): thêm TextVN vào nhóm đầu tiên nếu chưa có.
# Chỉ sửa khi fcitx5 KHÔNG chạy (fcitx5 ghi đè profile khi thoát) — gọi trước khi restart.
tv_fcitx5_profile() { echo "${XDG_CONFIG_HOME:-$HOME/.config}/fcitx5/profile"; }

tv_activate_fcitx5() {
    local p n
    p="$(tv_fcitx5_profile)"
    [[ -f "$p" ]] || return 0
    grep -qx 'Name=textvn' "$p" && return 0
    n="$(grep -c '^\[Groups/0/Items/[0-9]*\]$' "$p" || true)"
    printf '\n[Groups/0/Items/%s]\nName=textvn\nLayout=\n' "$n" >> "$p"
    tv_ok "Đã thêm TextVN vào nhóm bộ gõ Fcitx5 (Ctrl+Space để chuyển)."
}

tv_deactivate_fcitx5() {
    local p tmp
    p="$(tv_fcitx5_profile)"
    [[ -f "$p" ]] && grep -qx 'Name=textvn' "$p" || return 0
    tmp="$(mktemp)"
    # Bỏ section item có Name=textvn, rồi đánh lại chỉ số liên tục cho item của nhóm 0.
    awk '
        function flush() { if (!skip) printf "%s", buf; buf = ""; skip = 0 }
        /^\[/ { flush(); sec = $0 }
        { buf = buf $0 "\n"
          if (sec ~ /^\[Groups\/0\/Items\/[0-9]+\]$/ && $0 == "Name=textvn") skip = 1 }
        END { flush() }
    ' "$p" | awk '
        /^\[Groups\/0\/Items\/[0-9]+\]$/ { printf "[Groups/0/Items/%d]\n", i++; next }
        { print }
    ' > "$tmp" && cat "$tmp" > "$p"
    rm -f "$tmp"
}
