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

# Gói hợp lệ: có CLI, chạy được trên glibc máy hiện tại và có ít nhất một adapter.
tv_check_package() {
    local d="$1" err=""
    if [[ ! -x "$d/bin/textvn" ]]; then
        tv_err "Không phải thư mục gói TextVN (thiếu bin/textvn): $d"
        return 1
    fi
    if [[ ! -x "$d/lib/textvn/textvn-ibus-engine" && ! -f "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
        tv_err "Gói không có adapter IBus lẫn Fcitx5."
        return 1
    fi
    # E64/E67: kiểm tra tương thích glibc/loader TRƯỚC khi cài hoặc restart IBus — tránh
    # ghi đè bản binary không chạy được trên máy cũ làm hỏng phiên IBus.
    if ! err="$("$d/bin/textvn" --help 2>&1 >/dev/null)"; then
        tv_err "Binary textvn không tương thích thư viện hệ thống trên máy này:"
        [[ -n "$err" ]] && tv_err "  $err"
        tv_err "Hãy dựng trực tiếp từ mã nguồn cho máy bạn: ./scripts/install_linux.sh (xem docs/user-guide.md)."
        return 1
    fi
    if [[ -x "$d/lib/textvn/textvn-ibus-engine" ]]; then
        if ! err="$("$d/lib/textvn/textvn-ibus-engine" --check-abi 2>&1 >/dev/null)"; then
            tv_err "Binary textvn-ibus-engine thiếu thư viện hệ thống (ví dụ ibus / glib):"
            [[ -n "$err" ]] && tv_err "  $err"
            tv_err "  • RHEL / Rocky / Alma / Fedora : sudo dnf install -y ibus ibus-libs gtk3"
            tv_err "  • Ubuntu / Debian              : sudo apt install -y ibus libibus-1.0-5"
            tv_err "Hoặc dựng từ mã nguồn: ./scripts/install_linux.sh (xem docs/user-guide.md)."
            return 1
        fi
    fi
}

# Các thư mục addon Fcitx5 có thể có trên máy này, mỗi dòng một thư mục, theo thứ tự
# ưu tiên: libdir của Fcitx5Core (pkg-config, nếu có gói -dev), multiarch đúng kiến
# trúc máy (Debian/Ubuntu), lib64 (Fedora/openSUSE), lib (Arch), /usr/local.
# TV_FCITX5_ADDON_CANDIDATES (danh sách cách nhau bởi ':') thay TOÀN BỘ danh sách —
# cho Fcitx5 tự build ở /opt, và cho kiểm thử giả lập máy chưa cài Fcitx5.
tv_fcitx5_addon_candidates() {
    if [[ -n "${TV_FCITX5_ADDON_CANDIDATES:-}" ]]; then
        tr ':' '\n' <<< "$TV_FCITX5_ADDON_CANDIDATES" | sed '/^$/d'
        return 0
    fi
    local libdir="" triplet=""
    if command -v pkg-config >/dev/null 2>&1; then
        libdir="$(pkg-config --variable=libdir Fcitx5Core 2>/dev/null || true)"
    fi
    case "$(uname -m 2>/dev/null || true)" in
        x86_64|amd64) triplet=x86_64-linux-gnu ;;
        aarch64|arm64) triplet=aarch64-linux-gnu ;;
        armv7*|armv8l) triplet=arm-linux-gnueabihf ;;
        i?86) triplet=i386-linux-gnu ;;
        riscv64) triplet=riscv64-linux-gnu ;;
        ppc64le) triplet=powerpc64le-linux-gnu ;;
        s390x) triplet=s390x-linux-gnu ;;
        loongarch64) triplet=loongarch64-linux-gnu ;;
    esac
    {
        [[ -n "$libdir" ]] && echo "$libdir/fcitx5"
        [[ -n "$triplet" ]] && echo "/usr/lib/$triplet/fcitx5"
        printf '%s\n' /usr/lib64/fcitx5 /usr/lib/fcitx5 /usr/local/lib/fcitx5
    } | awk '!seen[$0]++'
}

# Thư mục addon hệ thống của Fcitx5 đang cài (rỗng nếu chưa cài Fcitx5).
tv_fcitx5_system_addon_dir() {
    local d
    while IFS= read -r d; do
        if compgen -G "$d/lib*.so" >/dev/null 2>&1; then
            echo "$d"
            return 0
        fi
    done < <(tv_fcitx5_addon_candidates)
    echo ""
}

# Giá trị FCITX_ADDON_DIRS cho Fcitx5 nạp addon TextVN ngoài thư mục hệ thống.
# FCITX_ADDON_DIRS THAY THẾ (không nối thêm) thư mục addon gốc của Fcitx5: thiếu thư
# mục đó thì Fcitx5 mất keyboard/dbus/wayland/xim/classicui — hỏng hoàn toàn, không
# app nào gõ được (R2-45). Chưa tìm thấy Fcitx5 (chưa cài, hoặc libdir lạ) → liệt kê
# MỌI thư mục ứng viên; thư mục không tồn tại bị Fcitx5 bỏ qua.
tv_fcitx5_env_addon_dirs() { # <textvn_addon_dir> <system_addon_dir | "">
    local out="$1" d
    if [[ -n "$2" ]]; then
        printf '%s:%s\n' "$1" "$2"
        return 0
    fi
    while IFS= read -r d; do
        out="$out:$d"
    done < <(tv_fcitx5_addon_candidates)
    printf '%s\n' "$out"
}

# Trích một đường dẫn cho g_shell_parse_argv — ibus-daemon tách <exec> bằng hàm này
# (bus/component.c) nên đường dẫn có khoảng trắng ("Tải về") thành 2 argv và engine
# không bao giờ chạy (R2-46). Đường dẫn chỉ gồm ký tự an toàn giữ nguyên (component
# của bản cài thường không đổi); còn lại bọc '...' và thay mỗi ' bằng '\''.
tv_shell_quote() {
    local s="$1"
    if [[ -n "$s" && "$s" != *[![:alnum:]_./+:@%,=-]* ]]; then
        printf '%s' "$s"
        return 0
    fi
    s=${s//\'/\'\\\'\'}
    printf "'%s'" "$s"
}

# Trích một đường dẫn cho trường Exec= trong file .desktop (FreeDesktop Desktop Entry Spec:
# dùng dấu ngoặc kép "...", escape \, ", `, $ và nhân đôi %).
tv_desktop_quote() {
    local s="$1"
    if [[ -n "$s" && "$s" != *[![:alnum:]_./+:@,=-]* ]]; then
        printf '%s' "$s"
        return 0
    fi
    s=${s//\\/\\\\}
    s=${s//\"/\\\"}
    s=${s//\`/\\\`}
    s=${s//\$/\\\$}
    s=${s//%/%%}
    printf '"%s"' "$s"
}

# Escape text cho nội dung phần tử XML. Phần thay thế PHẢI đặt trong ngoặc kép:
# bash 5.2 bật patsub_replacement, '&' không quote trong phần thay thế = chuỗi khớp.
tv_xml_escape() {
    local s="$1"
    s=${s//&/"&amp;"}
    s=${s//</"&lt;"}
    s=${s//>/"&gt;"}
    printf '%s' "$s"
}

# Component XML của IBus trỏ tới engine/bảng cài đặt ở đường dẫn thật. Không dùng sed
# với đường dẫn của người dùng: '&' và '|' trong đường dẫn bị sed hiểu là cú pháp.
tv_ibus_component() { # <pkg_dir> <engine_path> <settings_path>
    local engine_q setup_q line
    local pat_engine=/usr/lib/textvn/textvn-ibus-engine pat_setup=/usr/bin/textvn-settings
    engine_q="$(tv_xml_escape "$(tv_shell_quote "$2")")"
    setup_q="$(tv_xml_escape "$(tv_shell_quote "$3")")"
    while IFS= read -r line || [[ -n "$line" ]]; do
        line=${line//"$pat_engine"/"$engine_q"}
        line=${line//"$pat_setup"/"$setup_q"}
        printf '%s\n' "$line"
    done < "$1/share/ibus/component/textvn.xml"
}

tv_running() { pgrep -x -u "$(id -u)" "$1" >/dev/null 2>&1; }

# Giá trị môi trường của daemon đang chạy. Chỉ dùng để khôi phục phiên portable
# về đúng plugin/component mà người dùng đã có trước đó; biến môi trường không
# thể chứa NUL nên cách đọc /proc này không làm mất dữ liệu hợp lệ.
tv_process_environment() { # <process-name> <VARIABLE>
    local process="$1" key="$2" pid
    pid="$(pgrep -x -u "$(id -u)" "$process" | head -n1)"
    [[ -n "$pid" && -r "/proc/$pid/environ" ]] || return 1
    tr '\0' '\n' < "/proc/$pid/environ" | sed -n "s/^${key}=//p" | head -n1
}

# Manifest là dữ liệu ghi trên đĩa nên không được tin cậy để xoá một đường dẫn
# bất kỳ. Chỉ các file mà install.sh thực sự có thể cài mới được phép bị dọn.
# Mỗi nhánh dùng tên con trực tiếp để chặn cả `..` lẫn đường dẫn lồng không thuộc
# gói (ví dụ manifest bị sửa thành <prefix>/share/textvn/../../bin/other-app).
tv_textvn_install_path() { # <prefix> <path> [environment.d file]
    local prefix="$1" path="$2" env_file="${3:-}" leaf addon_dir base
    [[ "$prefix" == / ]] || prefix="${prefix%/}"
    base="$prefix"
    [[ "$base" == / ]] && base=""
    case "$path" in
        "$base/bin/textvn"|"$base/bin/textvn-settings"|\
        "$base/lib/textvn/textvn-ibus-engine"|\
        "$base/lib/textvn/textvn-settings-gtk4"|\
        "$base/lib/textvn/textvn-settings-gtk3"|\
        "$base/lib/textvn/fcitx5/libtextvn-fcitx5.so"|\
        "$base/share/ibus/component/textvn.xml"|\
        "$base/share/fcitx5/addon/textvn.conf"|\
        "$base/share/fcitx5/inputmethod/textvn.conf"|\
        "$base/share/icons/hicolor/scalable/apps/textvn_v.svg"|\
        "$base/share/icons/hicolor/scalable/apps/textvn_e.svg"|\
        "$base/share/icons/hicolor/128x128/apps/textvn_v.png"|\
        "$base/share/icons/hicolor/128x128/apps/textvn_e.png"|\
        "$base/share/applications/textvn-settings.desktop"|\
        "$base/share/metainfo/io.github.hunglinhpt.textvn.metainfo.xml")
            return 0
            ;;
        "$base/share/textvn/"*|"$base/share/doc/textvn/"*)
            leaf="${path##*/}"
            [[ "$leaf" != . && "$leaf" != .. && "$path" != *"/../"* && "$path" != *"//"* ]] || return 1
            # Sau khi cắt phần gốc, chỉ một filename được phép còn lại.
            case "$path" in
                "$base/share/textvn/$leaf"|"$base/share/doc/textvn/$leaf") return 0 ;;
            esac
            return 1
            ;;
    esac
    [[ -n "$env_file" && "$path" == "$env_file" ]] && return 0
    local autostart_file="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/textvn-ibus-autostart.desktop"
    [[ "$path" == "$autostart_file" ]] && return 0
    addon_dir="$(tv_fcitx5_system_addon_dir)"
    [[ -n "$addon_dir" && "$path" == "$addon_dir/libtextvn-fcitx5.so" ]]
}

# E67: IBus 1.5.19 (RHEL 8.10 / Rocky 8 / AlmaLinux 8) kiểm tra cache HỆ THỐNG
# /var/cache/ibus/bus/registry TRƯỚC ~/.cache/ibus/bus/registry và TRƯỚC IBUS_COMPONENT_PATH
# khi chạy --cache=auto (bus/ibusimpl.c). Trả 0 khi máy dính hành vi này (IBus < 1.5.20
# và tồn tại /var/cache/ibus/bus/registry).
tv_ibus_has_system_cache_bug() {
    [[ -f /var/cache/ibus/bus/registry ]] || return 1
    command -v ibus >/dev/null 2>&1 || return 1
    local ver maj min pat
    ver="$(ibus version 2>/dev/null | sed -n 's/.*IBus \([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | sed -n '1p')"
    [[ -n "$ver" ]] || return 1
    IFS=. read -r maj min pat <<<"$ver"
    [[ "$maj" -lt 1 ]] && return 0
    [[ "$maj" -eq 1 && "$min" -lt 5 ]] && return 0
    [[ "$maj" -eq 1 && "$min" -eq 5 && "$pat" -lt 20 ]] && return 0
    return 1
}

# ibus-daemon giữ cache registry (~/.cache/ibus/bus/registry). Xoá không thôi chưa đủ trên
# RHEL 8.10 (IBus 1.5.19): khi không có cache user, ibus-daemon 1.5.19 rơi về cache HỆ THỐNG
# /var/cache/ibus/bus/registry và bỏ qua hoàn toàn IBUS_COMPONENT_PATH (E67). Vì vậy cần
# dựng lại cache user bằng `IBUS_COMPONENT_PATH=... ibus write-cache`.
tv_ibus_clear_cache() {
    rm -f "${XDG_CACHE_HOME:-$HOME/.cache}/ibus/bus/registry" 2>/dev/null || true
}

tv_ibus_refresh_cache() { # <component_path | "">
    local comp="${1:-}"
    tv_ibus_clear_cache
    command -v ibus >/dev/null 2>&1 || return 0
    if [[ -n "$comp" ]]; then
        IBUS_COMPONENT_PATH="$comp" ibus write-cache >/dev/null 2>&1 || true
    else
        env -u IBUS_COMPONENT_PATH ibus write-cache >/dev/null 2>&1 || true
    fi
}

tv_systemd_user_ok() {
    command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1
}

# E67: GNOME Wayland cũ hơn GNOME 43 (RHEL 8.10 GNOME 3.32, Ubuntu 22.04 GNOME 42)
# không có systemd unit cho IBus — gnome-shell tự giữ socket Wayland IM với ibus-daemon.
# Trả 0 khi đang ở phiên đó (KHÔNG được `ibus-daemon --replace`; portable dùng live-register).
tv_ibus_needs_live_register() {
    [[ "${TV_FORCE_IBUS_REPLACE:-0}" != 1 ]] || return 1
    [[ "${XDG_SESSION_TYPE:-}" == wayland && "${XDG_CURRENT_DESKTOP:-}" == *GNOME* ]] || return 1
    tv_running ibus-daemon || return 1
    if [[ "${TV_SKIP_MANAGER_ENV:-0}" != 1 ]] && tv_systemd_user_ok; then
        local unit
        unit="$(systemctl --user list-units --state=active --no-legend 'org.freedesktop.IBus.session.*' 2>/dev/null | awk '{print $1; exit}')"
        [[ -z "$unit" ]] || return 1
    fi
    return 0
}

# Khởi động lại ibus-daemon với IBUS_COMPONENT_PATH — giữ nguyên tham số của daemon đang
# chạy (GNOME: `--panel disable`). Phiên systemd (GNOME ≥ 43): đổi môi trường của user
# manager rồi restart đúng unit; GNOME Wayland không có unit systemd (Ubuntu 22.04 GNOME 42):
# KHÔNG giết/replace thô ibus-daemon do gnome-shell quản lý (E64 — dễ đứt socket Wayland IM
# làm sập/đóng gnome-terminal); các phiên còn lại: chạy lại daemon tách khỏi PTY terminal.
tv_restart_ibus() { # <component_path | "">
    local comp="$1" unit pid args=()
    tv_ibus_refresh_cache "$comp"
    if tv_systemd_user_ok; then
        if [[ "${TV_SKIP_MANAGER_ENV:-0}" != 1 ]]; then
            if [[ -n "$comp" ]]; then
                systemctl --user set-environment "IBUS_COMPONENT_PATH=$comp" 2>/dev/null || true
                command -v dbus-update-activation-environment >/dev/null 2>&1 &&
                    IBUS_COMPONENT_PATH="$comp" dbus-update-activation-environment --systemd IBUS_COMPONENT_PATH 2>/dev/null || true
            else
                systemctl --user unset-environment IBUS_COMPONENT_PATH 2>/dev/null || true
            fi
        fi
        unit="$(systemctl --user list-units --state=active --no-legend 'org.freedesktop.IBus.session.*' 2>/dev/null | awk '{print $1; exit}')"
        if [[ -n "$unit" ]]; then
            systemctl --user restart "$unit" && tv_ok "Đã khởi động lại IBus ($unit)." && return 0
        fi
    fi
    # E64/E67: trên GNOME Wayland cũ hơn GNOME 43 (RHEL 8.10 GNOME 3.32, Ubuntu 22.04 GNOME 42)
    # không có systemd unit cho IBus — gnome-shell tự giữ socket Wayland IM với ibus-daemon.
    # Chỉ giữ nguyên khi ibus-daemon ĐANG chạy (cache user đã được làm mới ở trên).
    if [[ "${TV_FORCE_IBUS_REPLACE:-0}" != 1 && "${XDG_SESSION_TYPE:-}" == wayland &&
          "${XDG_CURRENT_DESKTOP:-}" == *GNOME* ]] && tv_running ibus-daemon; then
        tv_say "GNOME Wayland (không có systemd IBus unit, vd. RHEL 8.10 / Ubuntu 22.04): giữ nguyên ibus-daemon để không làm gián đoạn terminal."
        tv_say "Hãy đăng xuất và đăng nhập lại 1 lần để GNOME nạp TextVN an toàn."
        return 0
    fi
    pid="$(pgrep -x -u "$(id -u)" ibus-daemon 2>/dev/null | sed -n '1p' || true)"
    if [[ -n "$pid" && -r "/proc/$pid/cmdline" ]]; then
        mapfile -d '' args < "/proc/$pid/cmdline"
        args=("${args[@]:1}")
    fi
    [[ ${#args[@]} -eq 0 ]] && args=(--xim)
    local keep=() had_xim=0 skip_next=0
    for a in "${args[@]}"; do
        if [[ "$skip_next" == 1 ]]; then
            skip_next=0
            continue
        fi
        case "$a" in
            -d|--daemonize|-r|--replace|-dr|-rd) ;;
            -t|--cache) skip_next=1 ;;
            --cache=*|-t*) ;;
            -x|--xim) had_xim=1 ;;
            -drx|-dxr|-rdx|-rxd|-xdr|-xrd|-dx|-xd|-rx|-xr) had_xim=1 ;;
            *) keep+=("$a") ;;
        esac
    done
    [[ "$had_xim" == 1 ]] && keep+=(--xim)
    if [[ -n "$comp" ]]; then
        # E67: --cache=refresh ép IBus 1.5.19 (RHEL 8.10) đọc IBUS_COMPONENT_PATH thay vì
        # rơi về cache hệ thống /var/cache/ibus/bus/registry.
        IBUS_COMPONENT_PATH="$comp" setsid ibus-daemon "${keep[@]}" --cache=refresh --replace --daemonize </dev/null >/dev/null 2>&1
    else
        env -u IBUS_COMPONENT_PATH setsid ibus-daemon "${keep[@]}" --cache=refresh --replace --daemonize </dev/null >/dev/null 2>&1
    fi
    sleep 1
    tv_ok "Đã khởi động lại IBus."
}

# Không dùng pkill: IBus/Fcitx thường là IME dùng chung cho cả desktop session.
# Lệnh DBus chỉ tác động daemon gắn với DBUS_SESSION_BUS_ADDRESS hiện tại; nếu nó
# không dừng được, caller phải báo lỗi thay vì giết bừa tiến trình của người dùng.
tv_stop_ibus() {
    tv_running ibus-daemon || return 0
    command -v ibus >/dev/null 2>&1 || {
        tv_err "IBus đang chạy nhưng không có lệnh ibus để dừng an toàn."
        return 1
    }
    ibus exit >/dev/null 2>&1 || true
    local i
    for i in $(seq 1 50); do
        tv_running ibus-daemon || return 0
        sleep 0.1
    done
    tv_err "IBus không dừng qua DBus; không dùng pkill vì có thể làm hỏng phiên IME khác."
    return 1
}

tv_stop_fcitx5() {
    tv_running fcitx5 || return 0
    command -v fcitx5-remote >/dev/null 2>&1 || {
        tv_err "Fcitx5 đang chạy nhưng không có fcitx5-remote để dừng an toàn."
        return 1
    }
    fcitx5-remote -e >/dev/null 2>&1 || true
    local i
    for i in $(seq 1 50); do
        tv_running fcitx5 || return 0
        sleep 0.1
    done
    tv_err "Fcitx5 không dừng qua DBus; không dùng pkill vì có thể giết phiên IME khác."
    return 1
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
        pid="$(pgrep -x -u "$(id -u)" fcitx5 2>/dev/null | sed -n '1p' || true)"
        if [[ -n "$pid" && -r "/proc/$pid/cmdline" ]]; then
            mapfile -d '' args < "/proc/$pid/cmdline"
            for a in "${args[@]:1}"; do
                case "$a" in -d|-r|-rd|-dr|--replace) ;; *) keep+=("$a") ;; esac
            done
        fi
        tv_stop_fcitx5 || return 1
    fi
    case "$action" in
        add) tv_activate_fcitx5 ;;
        remove) tv_deactivate_fcitx5 ;;
    esac
    if [[ "${TV_SKIP_MANAGER_ENV:-0}" != 1 ]] && tv_systemd_user_ok; then
        if [[ -n "$addons" ]]; then
            systemctl --user set-environment "FCITX_ADDON_DIRS=$addons" 2>/dev/null || true
        else
            systemctl --user unset-environment FCITX_ADDON_DIRS 2>/dev/null || true
        fi
    fi
    [[ "$was_running" == 1 ]] || return 0
    if [[ -n "$addons" ]]; then
        FCITX_ADDON_DIRS="$addons" XDG_DATA_DIRS="$data" setsid fcitx5 -d "${keep[@]}" </dev/null >/dev/null 2>&1 || true
    else
        env -u FCITX_ADDON_DIRS XDG_DATA_DIRS="$data" setsid fcitx5 -d "${keep[@]}" </dev/null >/dev/null 2>&1 || true
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
        if [[ "$cur" == *"'Unikey'"* || "$cur" == *"'unikey'"* ]]; then
            tv_say "Phát hiện ibus-unikey đang có sẵn trong nguồn nhập GNOME."
            tv_say "Hãy nhấn Super+Space chọn 'Vietnamese (TextVN)' (hoặc bỏ 'Unikey' trong Cài đặt → Bàn phím → Nguồn nhập để tránh nhầm hai bộ gõ)."
        fi
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
        if [[ "$cur" == *"'Unikey'"* || "$cur" == *"'unikey'"* ]]; then
            tv_say "Phát hiện ibus-unikey đang có sẵn trong IBus — hãy chọn 'Vietnamese (TextVN)' hoặc gỡ Unikey khỏi danh sách engine để tránh nhầm lẫn."
        fi
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

# Profile Fcitx5 (~/.config/fcitx5/profile): thêm TextVN vào nhóm ĐẦU TIÊN theo
# GroupOrder (không cứng [Groups/0]) nếu chưa có. Chỉ sửa khi fcitx5 KHÔNG chạy
# (fcitx5 ghi đè profile khi thoát) — gọi trước khi restart.
tv_fcitx5_profile() { echo "${XDG_CONFIG_HOME:-$HOME/.config}/fcitx5/profile"; }

# C8 (audit) + R2-108: nhóm ĐANG DÙNG = mục đầu của [GroupOrder] ("0=<tên nhóm>"),
# tra ra [Groups/N] có Name=<tên nhóm> đó. Fcitx5 không ghi các [Groups/N] theo thứ
# tự GroupOrder, nên [Groups/0] chưa chắc là nhóm đang dùng. (Bản C8 đầu tiên đoán
# bằng DefaultIM — đó là bộ gõ mặc định TRONG mỗi nhóm, nên luôn ra nhóm đầu file.)
# Không tìm thấy → 0. In chỉ số N.
tv_fcitx5_active_group() { # <profile>
    awk '
        /^\[/ { sec = $0; next }
        sec == "[GroupOrder]" && /^0=/ { want = substr($0, 3) }
        sec ~ /^\[Groups\/[0-9]+\]$/ && /^Name=/ {
            id = sec
            gsub(/^\[Groups\/|\]$/, "", id)
            name[id] = substr($0, 6)
        }
        END {
            if (want != "") for (id in name) if (name[id] == want) { print id; exit }
            print 0
        }
    ' "$1"
}

tv_activate_fcitx5() {
    local p gid n
    p="$(tv_fcitx5_profile)"
    [[ -f "$p" ]] || return 0
    grep -qx 'Name=textvn' "$p" && return 0
    gid="$(tv_fcitx5_active_group "$p")"
    n="$(grep -c "^\[Groups/${gid}/Items/[0-9]*\]$" "$p" || true)"
    printf '\n[Groups/%s/Items/%s]\nName=textvn\nLayout=\n' "$gid" "$n" >> "$p"
    tv_ok "Đã thêm TextVN vào nhóm bộ gõ Fcitx5 (Ctrl+Space để chuyển)."
}

tv_deactivate_fcitx5() {
    local p tmp
    p="$(tv_fcitx5_profile)"
    [[ -f "$p" ]] && grep -qx 'Name=textvn' "$p" || return 0
    tmp="$(mktemp)"
    # Bỏ section item có Name=textvn, rồi đánh lại chỉ số liên tục cho MỌI nhóm.
    awk '
        function flush() { if (!skip) printf "%s", buf; buf = ""; skip = 0 }
        /^\[/ { flush(); sec = $0 }
        { buf = buf $0 "\n"
          if (sec ~ /^\[Groups\/[0-9]+\/Items\/[0-9]+\]$/ && $0 == "Name=textvn") skip = 1 }
        END { flush() }
    ' "$p" | awk '
        /^\[Groups\/[0-9]+\/Items\/[0-9]+\]$/ {
            key = $0; sub(/\/Items\/[0-9]+\]$/, "", key)   # -> [Groups/N
            printf "%s/Items/%d]\n", key, cnt[key]++
            next
        }
        { print }
    ' > "$tmp" && cat "$tmp" > "$p"
    rm -f "$tmp"
}
