# SPDX-License-Identifier: GPL-3.0-or-later
# textvn-common.sh — hàm dùng chung cho install.sh / uninstall.sh / textvn-portable.sh.
# Được `source`, không chạy trực tiếp.

if [[ -t 1 ]]; then
    TV_C_OK=$'\033[0;32m' TV_C_WARN=$'\033[0;33m' TV_C_ERR=$'\033[0;31m' TV_C_INFO=$'\033[0;36m' TV_C_OFF=$'\033[0m'
else
    TV_C_OK="" TV_C_WARN="" TV_C_ERR="" TV_C_INFO="" TV_C_OFF=""
fi
tv_say() { echo "${TV_C_INFO}==>${TV_C_OFF} $*"; }
tv_ok() { echo "${TV_C_OK}✓${TV_C_OFF} $*"; }
tv_warn() { echo "${TV_C_WARN}!${TV_C_OFF} $*" >&2; }
tv_err() { echo "${TV_C_ERR}✗ $*${TV_C_OFF}" >&2; }

# E69: Ghi nhật ký chẩn đoán bền vững (G8/S2: KHÔNG BAO GIỜ ghi nội dung gõ; ẩn tên
# người dùng và thư mục HOME). Dùng bởi install.sh, textvn-portable.sh, textvn-settings
# và được `textvn doctor --export` đóng gói tự động khi người dùng báo lỗi.
tv_diag_log_path() {
    local base=""
    if [[ -n "${XDG_STATE_HOME:-}" && "${XDG_STATE_HOME}" == /* ]]; then
        base="$XDG_STATE_HOME"
    elif [[ -n "${HOME:-}" && "${HOME}" == /* ]]; then
        base="$HOME/.local/state"
    else
        return 1
    fi
    # Khi chạy qua sudo (EUID=0), không tạo file thuộc quyền root trong thư mục của user thường
    # (tránh làm user thường không ghi được log sau đó).
    if [[ ${EUID:-$(id -u)} -eq 0 ]]; then
        [[ -n "${SUDO_USER:-}" ]] && return 1
        if [[ -e "$base" && ! -O "$base" ]]; then
            return 1
        fi
        if [[ -n "${HOME:-}" && -e "$HOME" && ! -O "$HOME" ]]; then
            return 1
        fi
    fi
    printf '%s/TextVN/log/install-diag.log\n' "$base"
}

tv_diag_sanitize() {
    local s="$1" u
    if [[ -n "${HOME:-}" && "${#HOME}" -ge 3 ]]; then
        s="${s//"$HOME"/~}"
    fi
    for u in "${USER:-}" "${LOGNAME:-}"; do
        if [[ -n "$u" && "${#u}" -ge 2 ]]; then
            s="${s//"$u"/<REDACTED_USER>}"
        fi
    done
    printf '%s' "$s"
}

tv_diag_log() { # <LEVEL> <CODE> <MESSAGE>
    local lvl="$1" code="$2" msg="$3" logf dir sz ts clean
    logf="$(tv_diag_log_path 2>/dev/null || true)"
    [[ -n "$logf" ]] || return 0
    dir="$(dirname "$logf")"
    mkdir -p -m 0700 "$dir" 2>/dev/null || return 0
    # Bảo vệ chống symlink attack trên thư mục lẫn file log/rotation.
    [[ ! -L "$dir" && -d "$dir" && -O "$dir" ]] || return 0
    [[ ! -L "$logf" && ! -L "${logf}.1" ]] || return 0
    if [[ -f "$logf" ]]; then
        sz="$(wc -c < "$logf" 2>/dev/null || echo 0)"
        if [[ "$sz" -gt 262144 ]]; then
            mv -f -- "$logf" "${logf}.1" 2>/dev/null || true
        fi
    fi
    ts="$(date '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo 'unknown-time')"
    clean="$(tv_diag_sanitize "$msg")"
    (umask 077 && printf '[%s] [%s] [%s] %s\n' "$ts" "$lvl" "$code" "$clean" >> "$logf") 2>/dev/null || true
}

tv_print_report_hint() { # [textvn_bin]
    local bin="${1:-textvn}"
    tv_err "Để báo lỗi kèm đầy đủ thông tin hệ thống và log (an toàn, không chứa chữ đã gõ):"
    tv_err "  $bin doctor --export ~/textvn-diag.zip"
    tv_err "và gửi tệp ~/textvn-diag.zip tại: https://github.com/hunglinhpt/TextVN/issues"
}

tv_version() { cat "$1/VERSION" 2>/dev/null || echo "?"; }

# E68: kiểm tra ABI độc lập cho từng adapter để máy chỉ dùng Fcitx5 (Fedora KDE,
# Arch KDE, openSUSE KDE, Kubuntu, Debian KDE — không cài libibus) vẫn cài/chạy được,
# đồng thời phát hiện ngay libtextvn-fcitx5.so lệch ABI/SONAME trước khi nạp.
tv_ibus_abi_ok() { # <pkg_dir>
    local d="$1"
    [[ -x "$d/lib/textvn/textvn-ibus-engine" ]] &&
        "$d/lib/textvn/textvn-ibus-engine" --check-abi >/dev/null 2>&1
}

tv_fcitx5_abi_ok() { # <pkg_dir>
    local d="$1"
    [[ -f "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" && -x "$d/bin/textvn" ]] &&
        "$d/bin/textvn" check-dlopen "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" >/dev/null 2>&1
}

tv_gtk_abi_ok() { # <pkg_dir>
    local d="$1"
    if [[ -x "$d/lib/textvn/textvn-settings-gtk4" ]] &&
       "$d/lib/textvn/textvn-settings-gtk4" --check-abi >/dev/null 2>&1; then
        return 0
    fi
    if [[ -x "$d/lib/textvn/textvn-settings-gtk3" ]] &&
       "$d/lib/textvn/textvn-settings-gtk3" --check-abi >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

tv_detect_pkg_mgr() {
    if [[ -n "${TV_PKG_MGR_OVERRIDE:-}" ]]; then
        printf '%s\n' "$TV_PKG_MGR_OVERRIDE"
        return 0
    fi
    if command -v apt-get >/dev/null 2>&1; then echo apt
    elif command -v dnf >/dev/null 2>&1; then echo dnf
    elif command -v yum >/dev/null 2>&1; then echo yum
    elif command -v zypper >/dev/null 2>&1; then echo zypper
    elif command -v pacman >/dev/null 2>&1; then echo pacman
    else echo none
    fi
}

tv_distro_family() {
    if [[ -n "${TV_DISTRO_FAMILY_OVERRIDE:-}" ]]; then
        printf '%s\n' "$TV_DISTRO_FAMILY_OVERRIDE"
        return 0
    fi
    local id="" id_like="" ver=""
    if [[ -r /etc/os-release ]]; then
        id="$(sed -n 's/^ID=//p' /etc/os-release | head -n1 | tr -d "\"'")"
        id_like="$(sed -n 's/^ID_LIKE=//p' /etc/os-release | head -n1 | tr -d "\"'")"
        ver="$(sed -n 's/^VERSION_ID=//p' /etc/os-release | head -n1 | tr -d "\"'")"
    fi
    case "$id" in
        fedora) echo fedora; return 0 ;;
        rhel|almalinux|rocky|ol|centos|eurolinux)
            case "$ver" in
                8|8.*) echo el8 ;;
                9|9.*) echo el9 ;;
                *) echo el9 ;;
            esac
            return 0
            ;;
        sles|sled|opensuse*) echo suse; return 0 ;;
        ubuntu|debian|linuxmint|pop|elementary|zorin|kali) echo debian; return 0 ;;
        arch|manjaro|endeavouros|cachyos|garuda) echo arch; return 0 ;;
    esac
    case " $id_like " in
        *" rhel "*|*" centos "*)
            case "$ver" in 8|8.*) echo el8 ;; *) echo el9 ;; esac ;;
        *" fedora "*) echo fedora ;;
        *" suse "*|*" opensuse "*) echo suse ;;
        *" debian "*|*" ubuntu "*) echo debian ;;
        *" arch "*) echo arch ;;
        *) echo other ;;
    esac
}

tv_preferred_framework() { # [requested_fw]
    local req="${1:-}"
    if [[ "$req" == ibus || "$req" == fcitx5 ]]; then
        echo "$req"
        return 0
    fi
    if tv_running fcitx5 && ! tv_running ibus-daemon; then
        echo fcitx5
        return 0
    fi
    if [[ "${XDG_CURRENT_DESKTOP:-}" == *KDE* ]] &&
       (command -v fcitx5 >/dev/null 2>&1 || ! command -v ibus-daemon >/dev/null 2>&1); then
        echo fcitx5
        return 0
    fi
    echo ibus
}

tv_runtime_dep_packages() { # <mgr> <family> <need_ibus> <need_fcitx5> <need_gtk>
    local mgr="$1" fam="$2" need_ibus="$3" need_fcitx5="$4" need_gtk="$5"
    local pkgs=()
    case "$mgr" in
        apt)
            [[ "$need_ibus" == 1 ]] && pkgs+=(ibus libibus-1.0-5)
            [[ "$need_fcitx5" == 1 ]] && pkgs+=(fcitx5 fcitx5-modules)
            if [[ "$need_gtk" == 1 ]]; then
                if command -v apt-cache >/dev/null 2>&1 && apt-cache show libgtk-4-1 >/dev/null 2>&1; then
                    pkgs+=(libgtk-4-1)
                elif command -v apt-cache >/dev/null 2>&1 && apt-cache show libgtk-3-0t64 >/dev/null 2>&1; then
                    pkgs+=(libgtk-3-0t64)
                else
                    pkgs+=(libgtk-3-0)
                fi
            fi
            ;;
        dnf|yum)
            [[ "$need_ibus" == 1 ]] && pkgs+=(ibus ibus-libs)
            [[ "$need_fcitx5" == 1 ]] && pkgs+=(fcitx5)
            if [[ "$need_gtk" == 1 ]]; then
                if [[ "$fam" == fedora ]]; then
                    pkgs+=(gtk4)
                else
                    pkgs+=(gtk3)
                fi
            fi
            ;;
        zypper)
            [[ "$need_ibus" == 1 ]] && pkgs+=(ibus libibus-1_0-5)
            [[ "$need_fcitx5" == 1 ]] && pkgs+=(fcitx5)
            [[ "$need_gtk" == 1 ]] && pkgs+=(libgtk-3-0)
            ;;
        pacman)
            [[ "$need_ibus" == 1 ]] && pkgs+=(ibus)
            [[ "$need_fcitx5" == 1 ]] && pkgs+=(fcitx5)
            [[ "$need_gtk" == 1 ]] && pkgs+=(gtk4)
            ;;
    esac
    [[ ${#pkgs[@]} -gt 0 ]] && printf '%s\n' "${pkgs[@]}" || true
}

tv_format_pkg_install_cmd() { # <mgr> <pkgs...>
    local mgr="$1"; shift
    case "$mgr" in
        apt) echo "sudo apt-get update && sudo apt-get install -y --no-install-recommends $*" ;;
        dnf) echo "sudo dnf install -y $*" ;;
        yum) echo "sudo yum install -y $*" ;;
        zypper) echo "sudo zypper --non-interactive install --no-recommends $*" ;;
        pacman) echo "sudo pacman -Sy --noconfirm --needed $*" ;;
        *) echo "" ;;
    esac
}

tv_run_pkg_install() { # <mgr> <pkgs...>
    local mgr="$1"; shift
    [[ $# -gt 0 ]] || return 0
    local -a elev=()
    if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
        if command -v sudo >/dev/null 2>&1; then
            elev=(sudo)
        elif command -v pkexec >/dev/null 2>&1; then
            elev=(pkexec)
        else
            tv_err "Không có quyền root và không tìm thấy 'sudo' hoặc 'pkexec' để tự động cài gói phụ thuộc."
            return 1
        fi
    fi
    case "$mgr" in
        apt)
            ${elev[@]+"${elev[@]}"} env DEBIAN_FRONTEND=noninteractive apt-get update -qq &&
                ${elev[@]+"${elev[@]}"} env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "$@"
            ;;
        dnf)
            ${elev[@]+"${elev[@]}"} dnf install -y "$@"
            ;;
        yum)
            ${elev[@]+"${elev[@]}"} yum install -y "$@"
            ;;
        zypper)
            ${elev[@]+"${elev[@]}"} zypper --non-interactive install --no-recommends "$@"
            ;;
        pacman)
            ${elev[@]+"${elev[@]}"} pacman -Sy --noconfirm --needed "$@"
            ;;
        *)
            return 1
            ;;
    esac
}

# E69: Kiểm tra đầy đủ phụ thuộc runtime (glibc, IBus/Fcitx5, GTK4/GTK3) và tự động kéo về
# cài đặt khi thiếu (tuỳ chế độ: auto = tự cài, ask = tự cài nếu root hoặc hỏi [Y/n] nếu có
# TTY, check = chỉ kiểm tra không gọi trình quản lý gói).
tv_ensure_linux_deps() { # <pkg_dir> [mode: ask|auto|check] [requested_fw]
    local d="$1" mode="${2:-ask}" req_fw="${3:-}"
    local err="" ibus_ok=0 fcitx_ok=0 gtk_ok=0 has_gtk_pkg=0
    local need_ibus=0 need_fcitx5=0 need_gtk=0
    local missing_desc=() pkgs=() mgr fam cmd ans=""

    if [[ ! -x "$d/bin/textvn" ]]; then
        tv_err "[E_PKG_INVALID] Không phải thư mục gói TextVN (thiếu bin/textvn): $d"
        tv_diag_log ERROR E_PKG_INVALID "Thieu bin/textvn trong thu muc goi"
        return 1
    fi
    if [[ ! -x "$d/lib/textvn/textvn-ibus-engine" && ! -f "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
        tv_err "[E_PKG_INVALID] Gói không có adapter IBus lẫn Fcitx5."
        tv_diag_log ERROR E_PKG_INVALID "Goi khong co adapter IBus lan Fcitx5"
        return 1
    fi
    # E64/E67: kiểm tra tương thích glibc/loader TRƯỚC khi cài hoặc restart IME.
    if ! err="$("$d/bin/textvn" --help 2>&1 >/dev/null)"; then
        tv_err "[E_GLIBC_INCOMPAT] Binary textvn không tương thích thư viện hệ thống trên máy này:"
        [[ -n "$err" ]] && tv_err "  $err"
        tv_err "Hãy dựng trực tiếp từ mã nguồn cho máy bạn: ./scripts/install_linux.sh (xem docs/user-guide.md)."
        tv_diag_log ERROR E_GLIBC_INCOMPAT "bin/textvn failed: ${err:-unknown}"
        return 1
    fi

    # Kiểm tra cả daemon lẫn thư viện ABI của từng framework.
    if tv_ibus_abi_ok "$d" && (command -v ibus-daemon >/dev/null 2>&1 || tv_running ibus-daemon); then
        ibus_ok=1
    fi
    if tv_fcitx5_abi_ok "$d" && (command -v fcitx5 >/dev/null 2>&1 || tv_running fcitx5); then
        fcitx_ok=1
    fi
    if [[ -x "$d/lib/textvn/textvn-settings-gtk4" || -x "$d/lib/textvn/textvn-settings-gtk3" ]]; then
        has_gtk_pkg=1
        tv_gtk_abi_ok "$d" && gtk_ok=1
    else
        gtk_ok=1
    fi

    if [[ "$req_fw" == ibus ]]; then
        [[ "$ibus_ok" == 0 ]] && need_ibus=1
    elif [[ "$req_fw" == fcitx5 ]]; then
        [[ "$fcitx_ok" == 0 ]] && need_fcitx5=1
    elif [[ "$ibus_ok" == 0 && "$fcitx_ok" == 0 ]]; then
        if [[ "$(tv_preferred_framework "$req_fw")" == fcitx5 ]]; then
            need_fcitx5=1
        else
            need_ibus=1
        fi
    fi
    if [[ "$has_gtk_pkg" == 1 && "$gtk_ok" == 0 ]]; then
        need_gtk=1
    fi

    if [[ "$need_ibus" == 0 && "$need_fcitx5" == 0 && "$need_gtk" == 0 ]]; then
        tv_diag_log INFO DEP_OK "Runtime deps OK (ibus=$ibus_ok fcitx5=$fcitx_ok gtk=$gtk_ok)"
        return 0
    fi

    if [[ "$need_ibus" == 1 ]]; then
        err=""
        if ! command -v ibus-daemon >/dev/null 2>&1; then
            err="chưa cài ibus-daemon"
        elif [[ -x "$d/lib/textvn/textvn-ibus-engine" ]]; then
            err="$("$d/lib/textvn/textvn-ibus-engine" --check-abi 2>&1 >/dev/null || true)"
        fi
        missing_desc+=("IBus (${err:-thiếu ibus / libibus})")
    fi
    if [[ "$need_fcitx5" == 1 ]]; then
        err=""
        if ! command -v fcitx5 >/dev/null 2>&1; then
            err="chưa cài fcitx5"
        elif [[ -f "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
            err="$("$d/bin/textvn" check-dlopen "$d/lib/textvn/fcitx5/libtextvn-fcitx5.so" 2>&1 || true)"
        fi
        missing_desc+=("Fcitx5 (${err:-thiếu fcitx5 / libFcitx5Core})")
    fi
    if [[ "$need_gtk" == 1 ]]; then
        missing_desc+=("GTK3/GTK4 (thiếu thư viện giao diện cho bảng cài đặt textvn-settings)")
    fi

    mgr="$(tv_detect_pkg_mgr)"
    fam="$(tv_distro_family)"
    if [[ "$mgr" != none ]]; then
        mapfile -t pkgs < <(tv_runtime_dep_packages "$mgr" "$fam" "$need_ibus" "$need_fcitx5" "$need_gtk")
    fi
    cmd=""
    [[ ${#pkgs[@]} -gt 0 ]] && cmd="$(tv_format_pkg_install_cmd "$mgr" "${pkgs[@]}")"

    tv_warn "[E_DEP_MISSING] Thiếu thành phần phụ thuộc của bộ gõ trên máy này:"
    local item
    for item in "${missing_desc[@]}"; do
        tv_warn "  • $item"
    done
    tv_diag_log WARN E_DEP_MISSING "mgr=$mgr fam=$fam missing=${missing_desc[*]:-} pkgs=${pkgs[*]:-}"

    local do_install=0
    if [[ "$mode" != check && "$mgr" != none && ${#pkgs[@]} -gt 0 ]]; then
        if [[ "$mode" == auto ]]; then
            do_install=1
        elif [[ -t 0 ]] && ([[ ${EUID:-$(id -u)} -eq 0 ]] || command -v sudo >/dev/null 2>&1 || command -v pkexec >/dev/null 2>&1); then
            echo "  Lệnh cài đề xuất: $cmd" >&2
            printf '  Tự động tải và cài các gói phụ thuộc còn thiếu ngay bây giờ? [Y/n] ' >&2
            read -r ans || ans="n"
            case "$ans" in
                ""|y|Y|yes|YES) do_install=1 ;;
            esac
        fi
    fi

    if [[ "$do_install" == 1 ]]; then
        tv_say "Đang tự động cài gói phụ thuộc ($mgr): ${pkgs[*]}"
        if tv_run_pkg_install "$mgr" "${pkgs[@]}"; then
            tv_diag_log INFO DEP_INSTALL_OK "Installed via $mgr: ${pkgs[*]}"
            # Kiểm tra lại ABI sau khi cài gói.
            ibus_ok=0; fcitx_ok=0; gtk_ok=0
            tv_ibus_abi_ok "$d" && (command -v ibus-daemon >/dev/null 2>&1 || tv_running ibus-daemon) && ibus_ok=1
            tv_fcitx5_abi_ok "$d" && (command -v fcitx5 >/dev/null 2>&1 || tv_running fcitx5) && fcitx_ok=1
            [[ "$has_gtk_pkg" == 0 ]] && gtk_ok=1
            [[ "$has_gtk_pkg" == 1 ]] && tv_gtk_abi_ok "$d" && gtk_ok=1
            if [[ "$gtk_ok" == 1 ]] && {
                { [[ "$req_fw" == ibus && "$ibus_ok" == 1 ]]; } ||
                { [[ "$req_fw" == fcitx5 && "$fcitx_ok" == 1 ]]; } ||
                { [[ -z "$req_fw" && ( "$ibus_ok" == 1 || "$fcitx_ok" == 1 ) ]]; }
            }; then
                tv_ok "Đã cài xong và xác minh đầy đủ thư viện phụ thuộc (IBus/Fcitx5/GTK)."
                return 0
            fi
        else
            tv_err "[E_DEP_INSTALL_FAIL] Tự động cài gói phụ thuộc thất bại ($mgr: ${pkgs[*]})."
            tv_diag_log ERROR E_DEP_INSTALL_FAIL "Failed via $mgr: ${pkgs[*]}"
        fi
    fi

    # Nếu framework bộ gõ ĐÃ sẵn sàng và chỉ thiếu GTK (ví dụ môi trường headless/CLI),
    # cho phép tiếp tục cài bộ gõ kèm cảnh báo rõ ràng cho bảng cài đặt GUI.
    if [[ ( "$req_fw" == ibus && "$ibus_ok" == 1 ) ||
          ( "$req_fw" == fcitx5 && "$fcitx_ok" == 1 ) ||
          ( -z "$req_fw" && ( "$ibus_ok" == 1 || "$fcitx_ok" == 1 ) ) ]]; then
        if [[ "$has_gtk_pkg" == 1 && "$gtk_ok" == 0 ]]; then
            tv_warn "[W_GTK_MISSING] Bộ gõ hoạt động được, nhưng bảng cài đặt GUI (textvn-settings) thiếu GTK3/GTK4."
            [[ -n "$cmd" ]] && tv_warn "  Cài thêm giao diện cài đặt: $cmd"
            tv_diag_log WARN W_GTK_MISSING "IM OK but GTK missing (suggested: $cmd)"
        fi
        return 0
    fi

    tv_err "[E_DEP_MISSING] Không thể tiếp tục vì thiếu thư viện/daemon bộ gõ bắt buộc."
    if [[ -n "$cmd" ]]; then
        tv_err "Hãy chạy lệnh sau để cài phụ thuộc rồi chạy lại:"
        tv_err "  $cmd"
    else
        tv_err "  • RHEL 8/9 / Rocky / Alma / Fedora : sudo dnf install -y ibus ibus-libs gtk3"
        tv_err "  • SLES 15 / openSUSE Leap          : sudo zypper install -y ibus libibus-1_0-5 libgtk-3-0"
        tv_err "  • Ubuntu / Debian / Mint           : sudo apt-get install -y ibus libibus-1.0-5 libgtk-4-1"
        tv_err "  • Arch / Manjaro                   : sudo pacman -S ibus gtk4 (hoặc fcitx5)"
    fi
    tv_err "Hoặc chạy kèm '--install-deps' (hoặc dựng từ mã nguồn: ./scripts/install_linux.sh)."
    tv_print_report_hint "$d/bin/textvn"
    return 1
}

# Gói hợp lệ: tương thích ngược — kiểm tra không tự cài gói (dùng trong đường dẫn cũ / kiểm thử).
tv_check_package() {
    tv_ensure_linux_deps "$1" check "${2:-}"
}

# E69: Kiểm tra tiến trình theo tên ĐẦY ĐỦ (vượt giới hạn 15 ký tự của /proc/<pid>/comm như
# `ibus-engine-unikey` -> `ibus-engine-uni`) và tôn trọng cùng phiên DBus khi DBUS_SESSION_BUS_ADDRESS
# được đặt (tránh báo nhầm tiến trình ở phiên VNC/desktop khác hoặc trong dbus-run-session).
tv_session_proc_running() { # <process-name>
    local want="$1" p comm a0 a1 b0 b1 pdbus
    local want15="${want:0:15}"
    local cur_dbus="${DBUS_SESSION_BUS_ADDRESS:-}"
    # Nếu caller mock `tv_running` (unit test) và tên <= 15 ký tự → kiểm nhanh qua tv_running khi không có DBus riêng.
    if [[ ${#want} -le 15 && -z "$cur_dbus" ]]; then
        tv_running "$want"
        return $?
    fi
    # Khi tv_running đã bị override thành hàm trong unit test, vẫn ưu tiên gọi nó để unit test hoạt động.
    if [[ "$(type -t tv_running 2>/dev/null || true)" == function ]] &&
       ! declare -f tv_running 2>/dev/null | grep -Fq 'pgrep -x -u'; then
        tv_running "$want"
        return $?
    fi
    for p in /proc/[0-9]*; do
        [[ -O "$p" && -r "$p/comm" && -r "$p/cmdline" ]] || continue
        comm="$(cat "$p/comm" 2>/dev/null || true)"
        if [[ ${#want} -le 15 ]]; then
            [[ "$comm" == "$want" ]] || continue
        else
            [[ "$comm" == "$want15" ]] || {
                # Engine viết bằng script (vd. python3 .../ibus-engine-bogo): kiểm argv[0]/argv[1].
                a0=""; a1=""
                { IFS= read -r -d '' a0 || true; IFS= read -r -d '' a1 || true; } < "$p/cmdline" 2>/dev/null || true
                b0="${a0##*/}"; b1="${a1##*/}"
                [[ "$b0" == "$want" || "$b1" == "$want" ]] || continue
            }
            # Nếu khớp comm 15 ký tự, xác nhận tiếp argv[0]/argv[1] khớp đúng tên đầy đủ.
            if [[ "$comm" == "$want15" ]]; then
                a0=""; a1=""
                { IFS= read -r -d '' a0 || true; IFS= read -r -d '' a1 || true; } < "$p/cmdline" 2>/dev/null || true
                b0="${a0##*/}"; b1="${a1##*/}"
                [[ "$b0" == "$want" || "$b1" == "$want" ]] || continue
            fi
        fi
        if [[ -n "$cur_dbus" && -r "$p/environ" ]]; then
            pdbus="$(tr '\0' '\n' < "$p/environ" 2>/dev/null | sed -n 's/^DBUS_SESSION_BUS_ADDRESS=//p' | head -n1 || true)"
            if [[ -n "$pdbus" && "$pdbus" != "$cur_dbus" ]]; then
                continue
            fi
        fi
        return 0
    done
    return 1
}

# E69: Phát hiện các bộ gõ / IME đang chạy hoặc được cấu hình gây xung đột với TextVN,
# giải thích lỗi cụ thể và gợi ý chính xác cách thoát/tắt bộ gõ cũ cho người dùng.
# Trả 0 = không có xung đột cứng (có thể đã in cảnh báo mềm);
# Trả 2 = phát hiện xung đột ĐANG CHẠY nghiêm trọng (hai daemon IME hoặc xvnkb/fcitx v4/scim/uim).
tv_check_ime_conflicts() { # [target_fw: ibus|fcitx5|""]
    local target_fw="${1:-}" hard=0 found_any=0
    local ibus_up=0 fcitx5_up=0 legacy_daemons=() vi_engines=()
    tv_session_proc_running ibus-daemon && ibus_up=1
    tv_session_proc_running fcitx5 && fcitx5_up=1

    # 1. Hai framework IME chạy song song (IBus + Fcitx5 cùng lúc).
    if [[ "$ibus_up" == 1 && "$fcitx5_up" == 1 ]]; then
        hard=1; found_any=1
        tv_err "[CONFLICT_DUAL_IM_DAEMON] Phát hiện CẢ ibus-daemon VÀ fcitx5 đang chạy đồng thời!"
        tv_err "  Nguyên nhân: hai tiến trình bộ gõ tranh chấp phím nóng (Ctrl+Shift/Ctrl+Space) và gây mất chữ, lặp chữ hoặc không bật được tiếng Việt."
        if [[ "$target_fw" == fcitx5 ]]; then
            tv_err "  Cách xử lý: thoát IBus trước khi dùng TextVN trên Fcitx5 → chạy: ibus exit"
        else
            tv_err "  Cách xử lý: thoát Fcitx5 trước khi dùng TextVN trên IBus → chạy: fcitx5-remote -e"
            tv_err "  (Hoặc ngược lại, nếu bạn muốn dùng Fcitx5: chạy 'ibus exit' rồi chọn Fcitx5.)"
        fi
        tv_diag_log ERROR CONFLICT_DUAL_IM_DAEMON "Both ibus-daemon and fcitx5 are running (target=$target_fw)"
    elif [[ "$target_fw" == ibus && "$fcitx5_up" == 1 ]]; then
        hard=1; found_any=1
        tv_err "[CONFLICT_DUAL_IM_DAEMON] Bạn chọn IBus nhưng fcitx5 đang chạy trên phiên này."
        tv_err "  Cách xử lý: thoát fcitx5 ('fcitx5-remote -e') trước, hoặc chạy TextVN trực tiếp với Fcitx5 ('--fcitx5')."
        tv_diag_log ERROR CONFLICT_DUAL_IM_DAEMON "Requested ibus while fcitx5 is running"
    elif [[ "$target_fw" == fcitx5 && "$ibus_up" == 1 ]]; then
        hard=1; found_any=1
        tv_err "[CONFLICT_DUAL_IM_DAEMON] Bạn chọn Fcitx5 nhưng ibus-daemon đang chạy trên phiên này."
        tv_err "  Cách xử lý: thoát IBus ('ibus exit') trước, hoặc chạy TextVN trực tiếp với IBus ('--ibus')."
        tv_diag_log ERROR CONFLICT_DUAL_IM_DAEMON "Requested fcitx5 while ibus-daemon is running"
    fi

    # 2. Framework IME cũ/khác (fcitx v4, scim, uim) hoặc bộ gõ X11 hook độc lập (xvnkb).
    local proc
    for proc in fcitx scim scim-launcher uim-xim uim-toolbar; do
        tv_session_proc_running "$proc" && legacy_daemons+=("$proc")
    done
    if [[ ${#legacy_daemons[@]} -gt 0 ]]; then
        hard=1; found_any=1
        tv_err "[CONFLICT_LEGACY_IME] Phát hiện bộ khung bộ gõ cũ đang chạy: ${legacy_daemons[*]}"
        tv_err "  Nguyên nhân: fcitx v4 / scim / uim chiếm XIM và xung đột trực tiếp với IBus/Fcitx5 của TextVN."
        tv_err "  Cách xử lý: thoát bộ gõ cũ (vd. 'fcitx-remote -e' hoặc đổi mặc định trong 'im-config' sang IBus/Fcitx5) rồi đăng nhập lại."
        tv_diag_log ERROR CONFLICT_LEGACY_IME "Legacy IM daemons running: ${legacy_daemons[*]}"
    fi

    for proc in xvnkb xvnhook; do
        if tv_session_proc_running "$proc"; then
            hard=1; found_any=1
            tv_err "[CONFLICT_STANDALONE_VI_IME] Phát hiện bộ gõ tiếng Việt cũ '$proc' đang chạy!"
            tv_err "  Nguyên nhân: '$proc' hook trực tiếp X11 nên sẽ ăn phím và làm hỏng dấu của TextVN."
            tv_err "  Cách xử lý: thoát '$proc' khỏi khay hệ thống (hoặc chạy 'pkill -x $proc') rồi khởi động lại TextVN."
            tv_diag_log ERROR CONFLICT_STANDALONE_VI_IME "Standalone VI IME running: $proc"
        fi
    done

    # 3. Bộ gõ tiếng Việt khác trong IBus (ibus-unikey, ibus-bamboo, ibus-bogo, ibus-teni).
    for proc in ibus-engine-unikey ibus-engine-bamboo ibus-engine-bogo ibus-engine-teni; do
        tv_session_proc_running "$proc" && vi_engines+=("$proc")
    done
    if [[ ${#vi_engines[@]} -gt 0 ]]; then
        found_any=1
        tv_warn "[CONFLICT_IBUS_VI_IME] Phát hiện bộ gõ tiếng Việt IBus khác đang hoạt động: ${vi_engines[*]}"
        tv_warn "  Gợi ý: nhấn Super+Space để chuyển sang 'Vietnamese (TextVN)', hoặc gỡ Unikey/Bamboo khỏi"
        tv_warn "  Cài đặt → Bàn phím → Nguồn nhập (hoặc ibus-setup) để tránh tranh chấp phím nóng Ctrl+Shift."
        tv_diag_log WARN CONFLICT_IBUS_VI_IME "Running IBus VI engines: ${vi_engines[*]}"
    fi

    # 4. Bộ gõ tiếng Việt khác trong profile Fcitx5 (fcitx5-unikey, fcitx5-bamboo).
    local fprof
    fprof="$(tv_fcitx5_profile)"
    if [[ -f "$fprof" ]] && grep -qE '^Name=(unikey|bamboo|m17n_vi_)' "$fprof" 2>/dev/null; then
        local fc_old
        fc_old="$(grep -oE '^Name=(unikey|bamboo|m17n_vi_[^[:space:]]*)' "$fprof" 2>/dev/null | sed 's/^Name=//' | tr '\n' ' ')"
        found_any=1
        tv_warn "[CONFLICT_FCITX5_VI_IME] Phát hiện bộ gõ tiếng Việt Fcitx5 khác trong cấu hình: ${fc_old}"
        tv_warn "  Gợi ý: mở 'fcitx5-configtool' và gỡ ${fc_old}(hoặc đặt 'TextVN' làm bộ gõ tiếng Việt duy nhất)"
        tv_warn "  để tránh nhầm lẫn khi nhấn Ctrl+Space / Ctrl+Shift."
        tv_diag_log WARN CONFLICT_FCITX5_VI_IME "Fcitx5 profile has conflicting VI IMs: $fc_old"
    fi

    # 5. Lệch biến môi trường GTK_IM_MODULE / QT_IM_MODULE / XMODIFIERS so với framework mục tiêu.
    local eff_fw="$target_fw"
    if [[ -z "$eff_fw" ]]; then
        if [[ "$fcitx5_up" == 1 && "$ibus_up" == 0 ]]; then eff_fw=fcitx5
        elif [[ "$ibus_up" == 1 && "$fcitx5_up" == 0 ]]; then eff_fw=ibus
        fi
    fi
    if [[ "$eff_fw" == ibus ]]; then
        if [[ "${GTK_IM_MODULE:-}" == fcitx* || "${QT_IM_MODULE:-}" == fcitx* || "${XMODIFIERS:-}" == *"@im=fcitx"* ]]; then
            found_any=1
            tv_warn "[CONFLICT_IM_ENV_MISMATCH] Phiên đang dùng IBus nhưng biến môi trường lại trỏ tới Fcitx:"
            tv_warn "  GTK_IM_MODULE=${GTK_IM_MODULE:-<trống>}  QT_IM_MODULE=${QT_IM_MODULE:-<trống>}  XMODIFIERS=${XMODIFIERS:-<trống>}"
            tv_warn "  Gợi ý: chạy 'im-config -n ibus' (Debian/Ubuntu) hoặc sửa ~/.xprofile / ~/.config/environment.d rồi đăng xuất & đăng nhập lại."
            tv_diag_log WARN CONFLICT_IM_ENV_MISMATCH "eff=ibus GTK=${GTK_IM_MODULE:-} QT=${QT_IM_MODULE:-} XMOD=${XMODIFIERS:-}"
        fi
    elif [[ "$eff_fw" == fcitx5 ]]; then
        if [[ "${GTK_IM_MODULE:-}" == ibus || "${QT_IM_MODULE:-}" == ibus || "${XMODIFIERS:-}" == *"@im=ibus"* ]]; then
            found_any=1
            tv_warn "[CONFLICT_IM_ENV_MISMATCH] Phiên đang dùng Fcitx5 nhưng biến môi trường lại trỏ tới IBus:"
            tv_warn "  GTK_IM_MODULE=${GTK_IM_MODULE:-<trống>}  QT_IM_MODULE=${QT_IM_MODULE:-<trống>}  XMODIFIERS=${XMODIFIERS:-<trống>}"
            tv_warn "  Gợi ý: chạy 'im-config -n fcitx5' (hoặc đặt GTK_IM_MODULE=fcitx, QT_IM_MODULE=fcitx, XMODIFIERS=@im=fcitx) rồi đăng xuất & đăng nhập lại."
            tv_diag_log WARN CONFLICT_IM_ENV_MISMATCH "eff=fcitx5 GTK=${GTK_IM_MODULE:-} QT=${QT_IM_MODULE:-} XMOD=${XMODIFIERS:-}"
        fi
    fi

    [[ "$hard" == 1 ]] && return 2
    return 0
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
        printf '%s\n' /usr/lib64/fcitx5 /usr/lib/fcitx5 /usr/local/lib64/fcitx5 /usr/local/lib/fcitx5
    } | awk '!seen[$0]++'
}

# Thư mục addon hệ thống của Fcitx5 đang cài (rỗng nếu chưa cài Fcitx5).
tv_fcitx5_system_addon_dir() {
    local d
    while IFS= read -r d; do
        if compgen -G "$d/*.so" >/dev/null 2>&1; then
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
# hai lớp unescape — GKeyFile giải mã \\ trước, sau đó Exec= giải mã trích dẫn "..." và %%).
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
    # Lớp chuỗi GKeyFile: nhân đôi mọi dấu \ để GKeyFile giữ nguyên chuỗi escape cho Exec=.
    s=${s//\\/\\\\}
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
    if [[ -n "$env_file" ]]; then
        [[ "$path" == "$env_file" ]] && return 0
        local autostart_file="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/textvn-ibus-autostart.desktop"
        [[ "$path" == "$autostart_file" ]] && return 0
    fi
    # E68: đối chiếu MỌI ứng viên libdir Fcitx5 (tránh lệch /usr/lib64 <-> /usr/lib trên Arch khi đổi pkgconf).
    while IFS= read -r addon_dir; do
        [[ -n "$addon_dir" && "$path" == "$addon_dir/libtextvn-fcitx5.so" ]] && return 0
    done < <(tv_fcitx5_addon_candidates)
    return 1
}

# Dừng chính xác tiến trình `textvn-ibus-engine --wait-bus` của đúng đường dẫn cài đặt
# và đúng người dùng hiện tại (tránh dùng `pkill -f` giết nhầm shell cha hoặc prefix khác).
tv_stop_wait_bus_engine() { # <engine_path>
    local target="$1" p comm pid i a has_wait
    local -a cmd_args
    for p in /proc/[0-9]*; do
        [[ -O "$p" && -r "$p/comm" && -r "$p/cmdline" ]] || continue
        comm="$(cat "$p/comm" 2>/dev/null || true)"
        [[ "$comm" == textvn-ibus-eng* ]] || continue
        cmd_args=()
        {
            while IFS= read -r -d '' a; do
                cmd_args+=("$a")
            done < "$p/cmdline"
        } 2>/dev/null || true
        [[ "${cmd_args[0]:-}" == "$target" ]] || continue
        has_wait=0
        for a in "${cmd_args[@]:1}"; do
            [[ "$a" == "--wait-bus" ]] && { has_wait=1; break; }
        done
        [[ "$has_wait" == 1 ]] || continue
        pid="${p#/proc/}"
        kill -TERM "$pid" 2>/dev/null || continue
        for i in $(seq 1 30); do
            kill -0 "$pid" 2>/dev/null || break
            grep -q '^State:[[:space:]]*Z' "/proc/$pid/status" 2>/dev/null && break
            sleep 0.1
        done
        if kill -0 "$pid" 2>/dev/null && ! grep -q '^State:[[:space:]]*Z' "/proc/$pid/status" 2>/dev/null; then
            kill -KILL "$pid" 2>/dev/null || true
        fi
    done
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
        mapfile -d '' args < "/proc/$pid/cmdline" 2>/dev/null || true
        args=("${args[@]:1}")
    fi
    [[ ${#args[@]} -eq 0 ]] && args=(--xim)
    local keep=() had_xim=0 skip_next=0 a
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

# E68: Trên KDE Plasma Wayland (Fedora KDE, Arch/Manjaro KDE, openSUSE KDE, Debian 12 KDE),
# KWin trực tiếp khởi chạy fcitx5 với WAYLAND_SOCKET riêng. Giết rồi chạy `setsid fcitx5 -d`
# sẽ cắt đứt WAYLAND_SOCKET khiến ứng dụng Wayland mất bộ gõ.
tv_fcitx5_is_kwin_wayland() {
    [[ "${TV_FORCE_FCITX5_RESTART:-0}" != 1 ]] || return 1
    tv_running fcitx5 || return 1
    if [[ "${XDG_SESSION_TYPE:-}" == wayland && "${XDG_CURRENT_DESKTOP:-}" == *KDE* ]]; then
        return 0
    fi
    [[ -n "$(tv_process_environment fcitx5 WAYLAND_SOCKET || true)" ]]
}

# Dừng fcitx5 (nếu chạy), tuỳ chọn sửa profile, rồi chạy lại với FCITX_ADDON_DIRS và thư
# mục dữ liệu thêm. Profile phải sửa khi fcitx5 đã dừng: fcitx5 ghi đè profile khi thoát.
tv_restart_fcitx5() { # <addon_dirs | ""> <extra_xdg_data_dir | ""> <profile_action: add|remove|"">
    local addons="$1" extra="$2" action="${3:-}" data="${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
    local was_running=0 i pid args=() keep=() a
    [[ -n "$extra" ]] && data="$extra:$data"
    if tv_fcitx5_is_kwin_wayland; then
        if [[ "${TV_SKIP_MANAGER_ENV:-0}" != 1 ]] && tv_systemd_user_ok; then
            if [[ -n "$addons" ]]; then
                systemctl --user set-environment "FCITX_ADDON_DIRS=$addons" 2>/dev/null || true
            else
                systemctl --user unset-environment FCITX_ADDON_DIRS 2>/dev/null || true
            fi
        fi
        tv_say "KDE Plasma Wayland đang quản lý Fcitx5 (WAYLAND_SOCKET): giữ nguyên fcitx5 để không ngắt bàn phím ảo."
        tv_say "Hãy mở System Settings → Keyboard → Virtual Keyboard → áp dụng lại Fcitx 5 (hoặc đăng xuất/đăng nhập lại 1 lần)."
        return 0
    fi
    if tv_running fcitx5; then
        was_running=1
        # Giữ tham số của tiến trình đang chạy (ví dụ --disable/--enable của phiên).
        pid="$(pgrep -x -u "$(id -u)" fcitx5 2>/dev/null | sed -n '1p' || true)"
        if [[ -n "$pid" && -r "/proc/$pid/cmdline" ]]; then
            mapfile -d '' args < "/proc/$pid/cmdline" 2>/dev/null || true
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
    local cur new use_gnome=0
    # E68: gsettings-desktop-schemas thường cài sẵn cả trên Cinnamon/Xfce/MATE/KDE —
    # chỉ dùng org.gnome.desktop.input-sources khi đang ở GNOME (hoặc phiên headless chưa đặt desktop).
    if [[ -z "${XDG_CURRENT_DESKTOP:-}" || "${XDG_CURRENT_DESKTOP:-}" == *GNOME* ]]; then
        gsettings list-schemas 2>/dev/null | grep -qx org.gnome.desktop.input-sources && use_gnome=1
    fi
    if [[ "$use_gnome" == 1 ]]; then
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
        if cur="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null)" &&
           [[ "$cur" == *"('ibus', 'textvn')"* ]]; then
            new="${cur//, (\'ibus\', \'textvn\')/}"
            new="${new//(\'ibus\', \'textvn\'), /}"
            new="${new//(\'ibus\', \'textvn\')/}"
            [[ "$new" == "[]" ]] && new="@a(ss) []"
            gsettings set org.gnome.desktop.input-sources sources "$new" || true
        fi
    fi
    if gsettings list-schemas 2>/dev/null | grep -qx org.freedesktop.ibus.general; then
        if cur="$(gsettings get org.freedesktop.ibus.general preload-engines 2>/dev/null)" &&
           [[ "$cur" == *"'textvn'"* ]]; then
            new="${cur//, \'textvn\'/}"
            new="${new//\'textvn\', /}"
            new="${new//\'textvn\'/}"
            [[ "$new" == "[]" ]] && new="@as []"
            gsettings set org.freedesktop.ibus.general preload-engines "$new" || true
        fi
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
    if [[ ! -f "$p" ]]; then
        tv_say "Chưa có file profile Fcitx5 ($p) — mở fcitx5-configtool sau khi khởi động Fcitx5 để thêm 'TextVN'."
        return 0
    fi
    grep -qx 'Name=textvn' "$p" && return 0
    gid="$(tv_fcitx5_active_group "$p")"
    n="$(grep -c "^\[Groups/${gid}/Items/[0-9]*\]$" "$p" || true)"
    printf '\n[Groups/%s/Items/%s]\nName=textvn\nLayout=\n' "$gid" "$n" >> "$p"
    tv_ok "Đã thêm TextVN vào nhóm bộ gõ Fcitx5 (Ctrl+Space để chuyển)."
}

# E68: Phiên X11 cổ điển (Linux Mint Cinnamon/LightDM, Pop!_OS X11, Debian Xfce/MATE,
# openSUSE/SLES X11) không đọc ~/.config/environment.d mà đọc ~/.xprofile. Ghi khối
# đánh dấu có quản lý và gỡ sạch (kể cả xóa file nếu trống) trong uninstall.sh.
tv_xprofile_install() { # <ibus_comp | ""> <fcitx_dirs | "">
    local ibus_comp="$1" fcitx_dirs="$2" xp="$HOME/.xprofile"
    tv_xprofile_remove
    [[ -n "$ibus_comp" || -n "$fcitx_dirs" ]] || return 0
    if [[ -s "$xp" && -n "$(tail -c 1 "$xp" 2>/dev/null || true)" ]]; then
        printf '\n' >> "$xp"
    fi
    {
        echo "# BEGIN TextVN (per-user X11 env — managed by install.sh / uninstall.sh)"
        if [[ -n "$ibus_comp" ]]; then
            printf 'export IBUS_COMPONENT_PATH=%s"${IBUS_COMPONENT_PATH:+:$IBUS_COMPONENT_PATH}"\n' "$(tv_shell_quote "$ibus_comp")"
        fi
        if [[ -n "$fcitx_dirs" ]]; then
            printf 'export FCITX_ADDON_DIRS=%s"${FCITX_ADDON_DIRS:+:$FCITX_ADDON_DIRS}"\n' "$(tv_shell_quote "$fcitx_dirs")"
        fi
        echo "# END TextVN"
    } >> "$xp"
}

tv_xprofile_remove() {
    local xp="$HOME/.xprofile" tmp
    [[ -f "$xp" ]] || return 0
    grep -Fq '# BEGIN TextVN' "$xp" && grep -Fqx '# END TextVN' "$xp" || return 0
    tmp="$(mktemp)"
    if awk '
        /^# BEGIN TextVN/ { skip = 1; next }
        /^# END TextVN$/  { skip = 0; next }
        !skip { print }
    ' "$xp" > "$tmp"; then
        if [[ ! -s "$tmp" && ! -L "$xp" ]]; then
            rm -f -- "$xp"
        else
            cat "$tmp" > "$xp"
        fi
    fi
    rm -f -- "$tmp"
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
