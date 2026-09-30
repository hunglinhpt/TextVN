#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN — cài bản dựng sẵn (tarball) cho Linux, KHÔNG cần biên dịch.
#
#   ./install.sh                # cài cho người dùng hiện tại vào ~/.local (không cần root)
#   sudo ./install.sh --system  # cài cho mọi người dùng vào /usr
#
# Tuỳ chọn: --prefix DIR, --no-restart (không khởi động lại IBus/Fcitx5),
#           --no-activate (không tự thêm TextVN vào danh sách bộ gõ).
# Mọi file đã cài được ghi vào <prefix>/share/textvn/install-manifest.txt để
# uninstall.sh gỡ đúng và đủ.

set -euo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=packaging/linux/textvn-common.sh
. "$PKG_DIR/textvn-common.sh"

MODE=user
PREFIX=""
RESTART=1
ACTIVATE=1

usage() {
    sed -n '3,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --user) MODE=user ;;
        --system) MODE=system ;;
        --prefix) PREFIX="${2:?--prefix cần thư mục}"; shift ;;
        --no-restart) RESTART=0 ;;
        --no-activate) ACTIVATE=0 ;;
        -h|--help) usage 0 ;;
        *) tv_err "Tuỳ chọn không hợp lệ: $1"; usage 1 ;;
    esac
    shift
done

if [[ -z "$PREFIX" ]]; then
    if [[ "$MODE" == system ]]; then PREFIX=/usr; else PREFIX="$HOME/.local"; fi
fi
if [[ "$MODE" == system && $EUID -ne 0 ]]; then
    tv_err "--system cần quyền root: sudo $0 --system"
    exit 1
fi

tv_check_package "$PKG_DIR" || exit 1

BIN="$PREFIX/bin"
LIBDIR="$PREFIX/lib/textvn"
SHARE="$PREFIX/share"
MANIFEST="$SHARE/textvn/install-manifest.txt"
SYS_FCITX_ADDONS="$(tv_fcitx5_system_addon_dir)"

# Addon Fcitx5: cài hệ thống → vào thư mục addon của Fcitx5 (nạp không cần biến môi
# trường); cài per-user → cạnh engine + FCITX_ADDON_DIRS.
if [[ "$MODE" == system && -n "$SYS_FCITX_ADDONS" ]]; then
    FCITX_LIB="$SYS_FCITX_ADDONS"
else
    FCITX_LIB="$LIBDIR/fcitx5"
fi
FCITX_DIRS="$FCITX_LIB${SYS_FCITX_ADDONS:+:$SYS_FCITX_ADDONS}"
[[ "$MODE" == system ]] && FCITX_DIRS=""

tv_say "Cài TextVN $(tv_version "$PKG_DIR") ($MODE) vào $PREFIX"

mkdir -p "$SHARE/textvn"
MANIFEST_NEW="$MANIFEST.new"
OLD_MANIFEST=""
if [[ -f "$MANIFEST" ]]; then
    OLD_MANIFEST="$(mktemp "$SHARE/textvn/.install-manifest.old.XXXXXX")"
    cp -- "$MANIFEST" "$OLD_MANIFEST"
fi
trap 'rm -f -- "$MANIFEST_NEW" "${OLD_MANIFEST:-}"' EXIT
: > "$MANIFEST_NEW"
put() { # put <mode> <src> <dst>
    install -D -m "$1" "$2" "$3"
    echo "$3" >> "$MANIFEST_NEW"
}

put 0755 "$PKG_DIR/bin/textvn" "$BIN/textvn"
[[ -f "$PKG_DIR/bin/textvn-settings" ]] && put 0755 "$PKG_DIR/bin/textvn-settings" "$BIN/textvn-settings"

HAS_IBUS=0
HAS_FCITX5=0
if [[ -f "$PKG_DIR/lib/textvn/textvn-ibus-engine" ]]; then
    put 0755 "$PKG_DIR/lib/textvn/textvn-ibus-engine" "$LIBDIR/textvn-ibus-engine"
    xml_tmp="$(mktemp)"
    tv_ibus_component "$PKG_DIR" "$LIBDIR/textvn-ibus-engine" "$BIN/textvn-settings" > "$xml_tmp"
    put 0644 "$xml_tmp" "$SHARE/ibus/component/textvn.xml"
    rm -f "$xml_tmp"
    HAS_IBUS=1
    [[ "$MODE" == user ]] && tv_ibus_clear_cache
fi
if [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
    put 0755 "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" "$FCITX_LIB/libtextvn-fcitx5.so"
    put 0644 "$PKG_DIR/share/fcitx5/addon/textvn.conf" "$SHARE/fcitx5/addon/textvn.conf"
    put 0644 "$PKG_DIR/share/fcitx5/inputmethod/textvn.conf" "$SHARE/fcitx5/inputmethod/textvn.conf"
    HAS_FCITX5=1
fi
for f in icons/hicolor/scalable/apps/textvn_v.svg icons/hicolor/scalable/apps/textvn_e.svg \
         applications/textvn-settings.desktop metainfo/io.github.hunglinhpt.textvn.metainfo.xml; do
    [[ -f "$PKG_DIR/share/$f" ]] && put 0644 "$PKG_DIR/share/$f" "$SHARE/$f"
done
for f in "$PKG_DIR"/share/doc/textvn/*; do
    [[ -f "$f" ]] && put 0644 "$f" "$SHARE/doc/textvn/$(basename "$f")"
done
put 0755 "$PKG_DIR/uninstall.sh" "$SHARE/textvn/uninstall.sh"
put 0644 "$PKG_DIR/textvn-common.sh" "$SHARE/textvn/textvn-common.sh"

# Cài per-user: ibus-daemon chỉ quét /usr/share/ibus/component, fcitx5 chỉ nạp addon ở
# libdir hệ thống — trừ khi có IBUS_COMPONENT_PATH / FCITX_ADDON_DIRS. Khai báo cho các
# phiên đăng nhập sau qua environment.d (GNOME, KDE, mọi phiên systemd).
ENV_FILE=""
if [[ "$MODE" == user ]]; then
    ENV_FILE="$HOME/.config/environment.d/60-textvn.conf"
    mkdir -p "$(dirname "$ENV_FILE")"
    {
        echo "# TextVN (cài per-user) — tạo bởi install.sh, gỡ bởi uninstall.sh"
        if [[ "$HAS_IBUS" == 1 ]]; then
            echo "IBUS_COMPONENT_PATH=$SHARE/ibus/component:/usr/share/ibus/component"
        fi
        if [[ "$HAS_FCITX5" == 1 ]]; then
            # Không được phụ thuộc vào việc tìm được libdir hệ thống: trên distro
            # không nằm trong danh sách hard-code, addon per-user vẫn phải được
            # Fcitx nạp được ở phiên đăng nhập sau. Giữ cả các addon directory
            # người dùng đã khai báo trước TextVN.
            printf 'FCITX_ADDON_DIRS=%s${FCITX_ADDON_DIRS:+:$FCITX_ADDON_DIRS}\n' "$FCITX_DIRS"
        fi
    } > "$ENV_FILE"
    echo "$ENV_FILE" >> "$MANIFEST_NEW"
fi

# Publish manifest của bản mới trước, rồi mới dọn residue của bản cũ. Nếu lệnh
# bị ngắt, bản uninstall vẫn biết chính xác những file bản mới đang sở hữu.
mv -- "$MANIFEST_NEW" "$MANIFEST"
echo "$MANIFEST" >> "$MANIFEST"

# Nâng cấp có thể bỏ một adapter hay UI phụ. Manifest cũ là dữ liệu có thể bị
# hỏng/sửa tay nên chỉ xóa path nằm trong allowlist TextVN, không theo symlink và
# không xóa thư mục đệ quy. Điều này đặc biệt quan trọng với `sudo --system`.
if [[ -n "$OLD_MANIFEST" ]]; then
    while IFS= read -r old; do
        [[ -n "$old" && "$old" != "$MANIFEST" ]] || continue
        grep -Fqx -- "$old" "$MANIFEST" && continue
        if tv_textvn_install_path "$PREFIX" "$old" "$ENV_FILE"; then
            rm -f -- "$old"
        else
            tv_err "Bỏ qua path không thuộc TextVN trong manifest cũ: $old"
        fi
    done < "$OLD_MANIFEST"
    rm -f -- "$OLD_MANIFEST"
    OLD_MANIFEST=""
fi
trap - EXIT

command -v gtk-update-icon-cache >/dev/null 2>&1 &&
    gtk-update-icon-cache -q -t -f "$SHARE/icons/hicolor" 2>/dev/null || true

# Áp dụng ngay cho phiên hiện tại (không cần đăng xuất): khởi động lại framework đang
# chạy với đúng biến môi trường; thêm TextVN vào danh sách bộ gõ (trừ --no-activate).
if [[ "$HAS_IBUS" == 1 ]]; then
    if [[ "$RESTART" == 1 ]] && tv_running ibus-daemon; then
        if [[ "$MODE" == user ]]; then
            tv_restart_ibus "$SHARE/ibus/component:/usr/share/ibus/component"
        else
            tv_restart_ibus ""
        fi
    fi
    [[ "$ACTIVATE" == 1 ]] && tv_activate_ibus
fi
if [[ "$HAS_FCITX5" == 1 ]]; then
    action=""
    [[ "$ACTIVATE" == 1 ]] && action=add
    if [[ "$RESTART" == 1 ]]; then
        tv_restart_fcitx5 "$FCITX_DIRS" "" "$action"
    elif [[ "$action" == add ]] && ! tv_running fcitx5; then
        tv_activate_fcitx5
    fi
fi

tv_ok "Đã cài TextVN."
echo
echo "  • Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (hoặc Ctrl + Shift + Space)."
echo "  • Bảng điều khiển: chạy 'textvn-settings' hoặc mở 'TextVN' trong menu ứng dụng."
echo "  • Chưa thấy TextVN trong danh sách bộ gõ? Thêm 'TextVN' trong Cài đặt → Bàn phím"
echo "    (GNOME/IBus) hoặc fcitx5-configtool, rồi đăng xuất và đăng nhập lại."
[[ -n "$ENV_FILE" ]] && echo "  • Biến môi trường per-user: $ENV_FILE (áp dụng từ lần đăng nhập sau)."
echo "  • Gỡ cài đặt: $SHARE/textvn/uninstall.sh"
