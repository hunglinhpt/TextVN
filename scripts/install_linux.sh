#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# install_linux.sh — biên dịch TextVN từ mã nguồn rồi cài bằng CHÍNH install.sh của gói
# (một đường cài duy nhất cho mã nguồn và tarball).
#
#   scripts/install_linux.sh                 # per-user (~/.local), không cần root
#   scripts/install_linux.sh --no-restart    # build với user thường, rồi cài hệ thống:
#   sudo target/linux-stage/install.sh --system
#
# Mọi tuỳ chọn được chuyển nguyên cho install.sh (--no-restart, --no-activate, --prefix).
# Cần: cargo, cmake, pkg-config, gcc/g++ và header IBus (libibus-1.0-dev) và/hoặc Fcitx5
# (libfcitx5core-dev fcitx5-modules-dev), GTK4 (libgtk-4-dev) cho bảng điều khiển.

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
    echo "✗ Không dùng 'source' hay '.' với install_linux.sh (sẽ đóng terminal). Hãy chạy trực tiếp: ./scripts/install_linux.sh" >&2
    return 1
fi

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ADAPTERS="$ROOT/target/linux-adapters"
# shellcheck source=packaging/linux/textvn-common.sh
. "$ROOT/packaging/linux/textvn-common.sh"

BUILD_DEP_MODE=ask
PASS_ARGS=()
for arg in "$@"; do
    case "$arg" in
        --install-deps) BUILD_DEP_MODE=auto; PASS_ARGS+=("$arg") ;;
        --no-install-deps) BUILD_DEP_MODE=check; PASS_ARGS+=("$arg") ;;
        *) PASS_ARGS+=("$arg") ;;
    esac
done

ensure_build_deps() {
    local missing=() mgr fam pkgs=() cmd="" ans="" do_inst=0
    local tool
    for tool in cmake pkg-config cc make; do
        command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
    done
    if command -v pkg-config >/dev/null 2>&1; then
        if ! pkg-config --exists ibus-1.0 && ! pkg-config --exists Fcitx5Core; then
            missing+=("ibus-devel/libibus-1.0-dev")
        fi
        if ! pkg-config --exists gtk4 && ! pkg-config --exists gtk+-3.0; then
            missing+=("gtk4-devel/gtk3-devel")
        fi
    fi
    if [[ ${#missing[@]} -gt 0 ]]; then
        mgr="$(tv_detect_pkg_mgr)"
        fam="$(tv_distro_family)"
        case "$mgr" in
            apt)
                if command -v apt-cache >/dev/null 2>&1 && apt-cache show libgtk-4-dev >/dev/null 2>&1; then
                    pkgs=(cmake pkg-config build-essential libibus-1.0-dev libgtk-4-dev ibus dbus libdbus-1-dev)
                else
                    pkgs=(cmake pkg-config build-essential libibus-1.0-dev libgtk-3-dev ibus dbus libdbus-1-dev)
                fi
                ;;
            dnf|yum)
                if [[ "$fam" == fedora ]]; then
                    pkgs=(gcc gcc-c++ make cmake pkgconf-pkg-config ibus-devel gtk4-devel ibus dbus-daemon)
                else
                    pkgs=(gcc gcc-c++ make cmake pkgconf-pkg-config ibus-devel gtk3-devel ibus dbus-daemon)
                fi
                ;;
            zypper) pkgs=(gcc gcc-c++ make cmake pkg-config ibus-devel gtk3-devel ibus) ;;
            pacman) pkgs=(base-devel cmake pkgconf ibus gtk4) ;;
        esac
        tv_warn "[E_BUILD_DEP_MISSING] Thiếu công cụ/header biên dịch: ${missing[*]}"
        tv_diag_log WARN E_BUILD_DEP_MISSING "mgr=$mgr fam=$fam missing=${missing[*]}"
        if [[ "$BUILD_DEP_MODE" != check && "$mgr" != none && ${#pkgs[@]} -gt 0 ]]; then
            cmd="$(tv_format_pkg_install_cmd "$mgr" "${pkgs[@]}")"
            if [[ "$BUILD_DEP_MODE" == auto ]]; then
                do_inst=1
            elif [[ -t 0 ]] && (command -v sudo >/dev/null 2>&1 || command -v pkexec >/dev/null 2>&1); then
                echo "  Lệnh cài đề xuất: $cmd" >&2
                printf '  Tự động cài gói biên dịch còn thiếu ngay bây giờ? [Y/n] ' >&2
                read -r ans || ans="n"
                case "$ans" in ""|y|Y|yes|YES) do_inst=1 ;; esac
            fi
            if [[ "$do_inst" == 1 ]]; then
                tv_say "Đang tự động cài gói biên dịch ($mgr): ${pkgs[*]}"
                tv_run_pkg_install "$mgr" "${pkgs[@]}" || tv_err "[E_BUILD_DEP_INSTALL_FAIL] Không thể cài tự động gói biên dịch."
            fi
        fi
    fi
}

ensure_build_deps

for tool in cargo cmake pkg-config cc make; do
    command -v "$tool" >/dev/null 2>&1 || {
        tv_err "[E_BUILD_DEP_MISSING] Thiếu '$tool'."
        echo "  • Ubuntu / Debian / Mint           : sudo apt install -y cargo cmake pkg-config build-essential libibus-1.0-dev libgtk-4-dev" >&2
        echo "  • RHEL 8 / Rocky 8 / Alma 8        : sudo dnf --enablerepo=powertools install -y gcc make cmake pkgconf-pkg-config ibus-devel gtk3-devel (RHEL 8 dùng codeready-builder; Rust qua rustup.rs)" >&2
        echo "  • RHEL 9 / Rocky 9 / Alma 9 / Fed  : sudo dnf --enablerepo=crb install -y gcc gcc-c++ make cmake pkgconf-pkg-config ibus-devel gtk4-devel (Fedora không cần --enablerepo)" >&2
        echo "  • SLES 15 / openSUSE Leap 15.6     : sudo zypper install -y gcc gcc-c++ make cmake pkg-config ibus-devel gtk3-devel" >&2
        echo "  • Arch / Manjaro                   : sudo pacman -S base-devel rust cmake pkgconf ibus gtk4" >&2
        [[ $EUID -eq 0 && "$tool" == cargo ]] && echo "  (Lưu ý: không chạy scripts/install_linux.sh bằng sudo; build bằng user thường rồi chạy: sudo target/linux-stage/install.sh --system)" >&2
        tv_diag_log ERROR E_BUILD_DEP_MISSING "Missing build tool: $tool"
        exit 1
    }
done

rm -rf "$ADAPTERS"
cargo build --release -p textvn-ffi -p textvn-cli --manifest-path "$ROOT/Cargo.toml"
built=0
if pkg-config --exists ibus-1.0; then
    cmake -S "$ROOT/adapters/linux-ibus" -B "$ADAPTERS/ibus" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$ADAPTERS/ibus" -j"$(nproc)"
    built=1
fi
if pkg-config --exists Fcitx5Core && pkg-config --exists Fcitx5Utils && command -v g++ >/dev/null 2>&1; then
    if cmake -S "$ROOT/adapters/linux-fcitx5" -B "$ADAPTERS/fcitx5" -DCMAKE_BUILD_TYPE=Release >/dev/null &&
       cmake --build "$ADAPTERS/fcitx5" -j"$(nproc)"; then
        built=1
    elif [[ "$built" == 1 ]]; then
        echo "Cảnh báo: build adapter Fcitx5 thất bại (có thể do compiler C++ cũ) — tiếp tục cài IBus." >&2
        rm -rf "$ADAPTERS/fcitx5"
    else
        exit 1
    fi
fi
if pkg-config --exists gtk4 || pkg-config --exists gtk+-3.0; then
    cmake -S "$ROOT/adapters/linux-settings" -B "$ADAPTERS/settings" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$ADAPTERS/settings" -j"$(nproc)"
fi
if [[ "$built" == 0 ]]; then
    tv_err "[E_BUILD_DEP_MISSING] Không có header IBus (libibus-1.0-dev / ibus-devel) hay Fcitx5 (libfcitx5core-dev fcitx5-modules-dev) → không có adapter nào để cài."
    tv_diag_log ERROR E_BUILD_DEP_MISSING "No IBus or Fcitx5 dev headers found"
    exit 1
fi

STAGE="$ROOT/target/linux-stage"
"$ROOT/scripts/stage-linux.sh" "$STAGE" "$ADAPTERS"
"$STAGE/install.sh" ${PASS_ARGS[@]+"${PASS_ARGS[@]}"}
