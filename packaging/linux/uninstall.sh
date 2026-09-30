#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN — gỡ bản đã cài bằng install.sh (đọc install-manifest.txt, gỡ đúng file đã cài).
#
#   ~/.local/share/textvn/uninstall.sh          # bản cài per-user
#   sudo /usr/share/textvn/uninstall.sh         # bản cài hệ thống
#
# Tuỳ chọn: --prefix DIR (mặc định: thư mục cài chứa script này), --no-restart,
#           --purge (xoá luôn cấu hình ~/.config/TextVN và nhật ký — mặc định GIỮ LẠI).

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=packaging/linux/textvn-common.sh
. "$HERE/textvn-common.sh"

PREFIX=""
RESTART=1
PURGE=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix) PREFIX="${2:?--prefix cần thư mục}"; shift ;;
        --user) PREFIX="$HOME/.local" ;;
        --system) PREFIX=/usr ;;
        --no-restart) RESTART=0 ;;
        --purge) PURGE=1 ;;
        -h|--help) sed -n '3,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) tv_err "Tuỳ chọn không hợp lệ: $1"; exit 1 ;;
    esac
    shift
done
# Script nằm ở <prefix>/share/textvn/ khi đã cài.
[[ -z "$PREFIX" ]] && PREFIX="$(cd "$HERE/../.." && pwd)"
MANIFEST="$PREFIX/share/textvn/install-manifest.txt"
ENV_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/environment.d/60-textvn.conf"
if [[ ! -f "$MANIFEST" ]]; then
    tv_err "Không tìm thấy $MANIFEST — TextVN chưa được cài ở $PREFIX?"
    exit 1
fi
if [[ ! -w "$MANIFEST" ]]; then
    tv_err "Không có quyền gỡ ở $PREFIX (bản cài hệ thống cần sudo)."
    exit 1
fi

tv_say "Gỡ TextVN khỏi $PREFIX"
had_ibus=0
had_fcitx5=0
grep -q '/ibus/component/textvn.xml$' "$MANIFEST" && had_ibus=1
grep -q 'libtextvn-fcitx5.so$' "$MANIFEST" && had_fcitx5=1

# Bỏ khỏi danh sách bộ gõ trước. install.sh gọi tv_activate_ibus cho CẢ bản hệ thống
# (gsettings là per-user) → gỡ cũng phải đối xứng (review R3 major 3).
[[ "$had_ibus" == 1 ]] && tv_deactivate_ibus

# Manifest là dữ liệu trên đĩa, không phải danh sách lệnh xoá được tin cậy. Chỉ
# gỡ các path install.sh có thể đã tạo; một manifest hỏng không được phép xoá
# file bất kỳ của người dùng hay của hệ thống.
while IFS= read -r f; do
    [[ -n "$f" ]] || continue
    if tv_textvn_install_path "$PREFIX" "$f" "$ENV_FILE"; then
        rm -f -- "$f"
    else
        tv_err "Bỏ qua path không thuộc TextVN trong manifest: $f"
    fi
done < "$MANIFEST"
for d in "$PREFIX/lib/textvn/fcitx5" "$PREFIX/lib/textvn" "$PREFIX/share/textvn" \
         "$PREFIX/share/doc/textvn"; do
    rmdir "$d" 2>/dev/null || true
done

[[ "$had_ibus" == 1 && "$PREFIX" != /usr ]] && tv_ibus_clear_cache
if [[ "$RESTART" == 1 ]]; then
    if [[ "$had_ibus" == 1 ]] && tv_running ibus-daemon; then
        tv_restart_ibus ""
    fi
fi
if [[ "$had_fcitx5" == 1 ]]; then
    if [[ "$RESTART" == 1 ]]; then
        tv_restart_fcitx5 "" "" remove
    elif ! tv_running fcitx5; then
        tv_deactivate_fcitx5
    fi
fi

if [[ "$PURGE" == 1 ]]; then
    rm -rf "${XDG_CONFIG_HOME:-$HOME/.config}/TextVN" "${XDG_STATE_HOME:-$HOME/.local/state}/TextVN"
    tv_ok "Đã xoá cấu hình ~/.config/TextVN và nhật ký ~/.local/state/TextVN."
fi
tv_ok "Đã gỡ TextVN."
[[ "$PURGE" == 1 ]] || echo "  Cấu hình (gõ tắt, tuỳ chọn) được giữ ở ~/.config/TextVN — thêm --purge để xoá."
