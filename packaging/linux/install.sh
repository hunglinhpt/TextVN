#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN — cài bản dựng sẵn (tarball) cho Linux, KHÔNG cần biên dịch.
#
#   ./install.sh                # cài cho người dùng hiện tại vào ~/.local (không cần root)
#   sudo ./install.sh --system  # cài cho mọi người dùng vào /usr
#
# Tuỳ chọn: --prefix DIR, --no-restart (không khởi động lại IBus/Fcitx5),
#           --no-activate (không tự thêm TextVN vào danh sách bộ gõ),
#           --install-deps (tự động cài gói IBus/Fcitx5/GTK còn thiếu),
#           --no-install-deps (chỉ kiểm tra, không gọi trình quản lý gói).
# Mọi file đã cài được ghi vào <prefix>/share/textvn/install-manifest.txt để
# uninstall.sh gỡ đúng và đủ.

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
    echo "✗ Không dùng 'source' hay '.' với install.sh (sẽ đóng terminal). Hãy chạy trực tiếp: ./install.sh" >&2
    return 1
fi

set -euo pipefail

PKG_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=packaging/linux/textvn-common.sh
. "$PKG_DIR/textvn-common.sh"

MODE=user
PREFIX=""
RESTART=1
ACTIVATE=1
DEP_MODE=ask

usage() {
    sed -n '3,14p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --user) MODE=user ;;
        --system) MODE=system ;;
        --prefix) PREFIX="${2:?--prefix cần thư mục}"; shift ;;
        --no-restart) RESTART=0 ;;
        --no-activate) ACTIVATE=0 ;;
        --install-deps) DEP_MODE=auto ;;
        --no-install-deps) DEP_MODE=check ;;
        -h|--help) usage 0 ;;
        *) tv_err "Tuỳ chọn không hợp lệ: $1"; usage 1 ;;
    esac
    shift
done

if [[ -z "$PREFIX" ]]; then
    if [[ "$MODE" == system ]]; then PREFIX=/usr; else PREFIX="$HOME/.local"; fi
fi
[[ "$PREFIX" == / ]] || PREFIX="${PREFIX%/}"
if [[ "$MODE" == system && $EUID -ne 0 ]]; then
    tv_err "--system cần quyền root: sudo $0 --system"
    exit 1
fi

tv_diag_log INFO INSTALL_START "mode=$MODE dep_mode=$DEP_MODE"
tv_ensure_linux_deps "$PKG_DIR" "$DEP_MODE" || exit 1

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
# Luôn kèm thư mục addon GỐC của Fcitx5 (hoặc mọi ứng viên khi chưa thấy Fcitx5):
# FCITX_ADDON_DIRS thay thế thư mục mặc định, thiếu nó Fcitx5 hỏng hẳn (R2-45).
FCITX_DIRS="$(tv_fcitx5_env_addon_dirs "$FCITX_LIB" "$SYS_FCITX_ADDONS")"
[[ "$MODE" == system ]] && FCITX_DIRS=""

# gsettings, profile Fcitx5 và khởi động lại IBus/Fcitx5 là việc của TỪNG người dùng
# trong phiên của họ. Dưới root (sudo --system) chúng chạm vào root — HOME=/root,
# không có session bus, gsettings vẫn thoát 0 — rồi in "✓" sai (R2-54); với `sudo -E`
# còn ghi file của root vào HOME người dùng. Bỏ qua, in lệnh để người dùng tự chạy.
PER_USER_STEPS=1
[[ $EUID -eq 0 ]] && PER_USER_STEPS=0

tv_say "Cài TextVN $(tv_version "$PKG_DIR") ($MODE) vào $PREFIX"

mkdir -p "$SHARE/textvn"
MANIFEST_NEW="$MANIFEST.new"
OLD_MANIFEST=""
XML_TMP=""
DESK_TMP=""
if [[ -f "$MANIFEST" ]]; then
    OLD_MANIFEST="$(mktemp "$SHARE/textvn/.install-manifest.old.XXXXXX")"
    cp -- "$MANIFEST" "$OLD_MANIFEST"
fi
trap 'rm -f -- "$MANIFEST_NEW" "${OLD_MANIFEST:-}" "${XML_TMP:-}" "${DESK_TMP:-}"' EXIT
: > "$MANIFEST_NEW"
put() { # put <mode> <src> <dst>
    install -D -m "$1" "$2" "$3"
    echo "$3" >> "$MANIFEST_NEW"
}

put 0755 "$PKG_DIR/bin/textvn" "$BIN/textvn"
[[ -f "$PKG_DIR/bin/textvn-settings" ]] && put 0755 "$PKG_DIR/bin/textvn-settings" "$BIN/textvn-settings"
[[ -f "$PKG_DIR/lib/textvn/textvn-settings-gtk4" ]] &&
    put 0755 "$PKG_DIR/lib/textvn/textvn-settings-gtk4" "$LIBDIR/textvn-settings-gtk4"
[[ -f "$PKG_DIR/lib/textvn/textvn-settings-gtk3" ]] &&
    put 0755 "$PKG_DIR/lib/textvn/textvn-settings-gtk3" "$LIBDIR/textvn-settings-gtk3"

HAS_IBUS=0
HAS_FCITX5=0
IBUS_ABI_OK=0
FCITX5_ABI_OK=0
tv_ibus_abi_ok "$PKG_DIR" && IBUS_ABI_OK=1
tv_fcitx5_abi_ok "$PKG_DIR" && FCITX5_ABI_OK=1

if [[ -f "$PKG_DIR/lib/textvn/textvn-ibus-engine" ]]; then
    put 0755 "$PKG_DIR/lib/textvn/textvn-ibus-engine" "$LIBDIR/textvn-ibus-engine"
    XML_TMP="$(mktemp)"
    tv_ibus_component "$PKG_DIR" "$LIBDIR/textvn-ibus-engine" "$BIN/textvn-settings" > "$XML_TMP"
    put 0644 "$XML_TMP" "$SHARE/ibus/component/textvn.xml"
    rm -f -- "$XML_TMP"
    XML_TMP=""
    HAS_IBUS=1
    if [[ "$IBUS_ABI_OK" == 1 ]]; then
        if [[ "$MODE" == user ]]; then
            tv_ibus_refresh_cache "$SHARE/ibus/component:/usr/share/ibus/component"
        elif [[ "$PREFIX" == /usr ]] && command -v ibus >/dev/null 2>&1; then
            env -u IBUS_COMPONENT_PATH ibus write-cache --system >/dev/null 2>&1 || true
        elif [[ "$MODE" == system ]]; then
            tv_say "Cài --system vào $PREFIX (khác /usr): IBus mặc định chỉ quét /usr/share/ibus/component — cần IBUS_COMPONENT_PATH=$SHARE/ibus/component:/usr/share/ibus/component."
        fi
    elif tv_running ibus-daemon || command -v ibus-daemon >/dev/null 2>&1; then
        tv_say "Lưu ý: thiếu thư viện runtime IBus (libibus) — đã chép file IBus nhưng bỏ qua khởi động lại IBus."
    fi
fi
if [[ -f "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
    put 0755 "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" "$FCITX_LIB/libtextvn-fcitx5.so"
    put 0644 "$PKG_DIR/share/fcitx5/addon/textvn.conf" "$SHARE/fcitx5/addon/textvn.conf"
    put 0644 "$PKG_DIR/share/fcitx5/inputmethod/textvn.conf" "$SHARE/fcitx5/inputmethod/textvn.conf"
    HAS_FCITX5=1
    if [[ "$FCITX5_ABI_OK" == 0 ]] && (tv_running fcitx5 || command -v fcitx5 >/dev/null 2>&1); then
        fc_err="$("$PKG_DIR/bin/textvn" check-dlopen "$PKG_DIR/lib/textvn/fcitx5/libtextvn-fcitx5.so" 2>&1 || true)"
        tv_err "Cảnh báo: addon Fcitx5 không nạp được trên máy này (bỏ qua restart Fcitx5): $fc_err"
        tv_err "  Hãy dùng IBus hoặc dựng Fcitx5 từ mã nguồn cho distro này: ./scripts/install_linux.sh"
    fi
    if [[ "$MODE" == system && -z "$SYS_FCITX_ADDONS" ]]; then
        # Distro ngoài danh sách multiarch: addon nằm ở $LIBDIR/fcitx5 mà bản hệ
        # thống không khai báo FCITX_ADDON_DIRS → Fcitx5 sẽ không nạp (review R3 minor 11).
        tv_err "Không tìm thấy thư mục addon Fcitx5 hệ thống — Fcitx5 sẽ KHÔNG nạp TextVN."
        tv_err "Cài per-user (bỏ --system) hoặc thêm vào môi trường phiên (giữ cả thư mục addon của Fcitx5):"
        tv_err "  FCITX_ADDON_DIRS=$(tv_fcitx5_env_addon_dirs "$FCITX_LIB" "")"
    fi
fi
for f in icons/hicolor/scalable/apps/textvn_v.svg icons/hicolor/scalable/apps/textvn_e.svg \
         icons/hicolor/128x128/apps/textvn_v.png icons/hicolor/128x128/apps/textvn_e.png \
         metainfo/io.github.hunglinhpt.textvn.metainfo.xml; do
    [[ -f "$PKG_DIR/share/$f" ]] && put 0644 "$PKG_DIR/share/$f" "$SHARE/$f"
done
if [[ -f "$PKG_DIR/share/applications/textvn-settings.desktop" ]]; then
    # E68: ghi đường dẫn tuyệt đối vào Exec= để menu ứng dụng chạy được cả khi ~/.local/bin
    # không nằm trong PATH của phiên đồ họa (Arch, SUSE, Debian GDM/SDDM, EL8/EL9, --prefix).
    DESK_TMP="$(mktemp)"
    desk_exec="Exec=$(tv_desktop_quote "$BIN/textvn-settings")"
    while IFS= read -r line || [[ -n "$line" ]]; do
        line="${line%$'\r'}"
        if [[ "$line" == "Exec=textvn-settings" ]]; then
            printf '%s\n' "$desk_exec"
        else
            printf '%s\n' "$line"
        fi
    done < "$PKG_DIR/share/applications/textvn-settings.desktop" > "$DESK_TMP"
    put 0644 "$DESK_TMP" "$SHARE/applications/textvn-settings.desktop"
    rm -f -- "$DESK_TMP"
    DESK_TMP=""
fi
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
    # Cùng công thức với uninstall.sh / tv_textvn_install_path — lệch XDG_CONFIG_HOME
    # làm allowlist chặn file này lúc gỡ (review R3 major 2).
    ENV_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/environment.d/60-textvn.conf"
    mkdir -p "$(dirname "$ENV_FILE")"
    {
        echo "# TextVN (cài per-user) — tạo bởi install.sh, gỡ bởi uninstall.sh"
        if [[ "$HAS_IBUS" == 1 ]]; then
            printf 'IBUS_COMPONENT_PATH=%s${IBUS_COMPONENT_PATH:+:$IBUS_COMPONENT_PATH}\n' "$SHARE/ibus/component:/usr/share/ibus/component"
        fi
        if [[ "$HAS_FCITX5" == 1 ]]; then
            # FCITX_DIRS luôn có thư mục addon gốc của Fcitx5 (hoặc mọi ứng viên
            # khi Fcitx5 chưa cài — người dùng cài Fcitx5 SAU TextVN vẫn có
            # keyboard/frontend, R2-45). Giữ cả các addon directory người dùng
            # đã khai báo trước TextVN.
            printf 'FCITX_ADDON_DIRS=%s${FCITX_ADDON_DIRS:+:$FCITX_ADDON_DIRS}\n' "$FCITX_DIRS"
        fi
    } > "$ENV_FILE"
    echo "$ENV_FILE" >> "$MANIFEST_NEW"
    if [[ "$PER_USER_STEPS" == 1 ]]; then
        xp_ibus=""
        xp_fcitx=""
        [[ "$HAS_IBUS" == 1 ]] && xp_ibus="$SHARE/ibus/component:/usr/share/ibus/component"
        [[ "$HAS_FCITX5" == 1 ]] && xp_fcitx="$FCITX_DIRS"
        tv_xprofile_install "$xp_ibus" "$xp_fcitx"
    fi
    if [[ "$HAS_IBUS" == 1 ]] && tv_ibus_has_system_cache_bug; then
        # E67: RHEL 8.10 (IBus 1.5.19) luôn nạp /var/cache/ibus/bus/registry trước
        # IBUS_COMPONENT_PATH khi chạy --cache=auto. Tạo XDG autostart đăng ký component
        # sống (không cần root), được ghi vào manifest để uninstall.sh gỡ sạch.
        AUTOSTART_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/autostart/textvn-ibus-autostart.desktop"
        mkdir -p "$(dirname "$AUTOSTART_FILE")"
        cat > "$AUTOSTART_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=TextVN IBus Engine (EL8 per-user)
Exec=$(tv_desktop_quote "$LIBDIR/textvn-ibus-engine") --wait-bus --setup $(tv_desktop_quote "$BIN/textvn-settings")
Hidden=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
EOF
        echo "$AUTOSTART_FILE" >> "$MANIFEST_NEW"
        tv_say "Phát hiện IBus < 1.5.20 có cache hệ thống (vd. RHEL 8.10): đã thêm autostart per-user $AUTOSTART_FILE."
        tv_say "  (Khuyến nghị trên máy có quyền quản trị: sudo ./install.sh --system để đăng ký trực tiếp vào /usr/share/ibus/component.)"
    fi
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
command -v update-desktop-database >/dev/null 2>&1 &&
    update-desktop-database -q "$SHARE/applications" 2>/dev/null || true
if [[ "$MODE" == system ]] && command -v restorecon >/dev/null 2>&1; then
    # E67/E68: RHEL 8/9 / Rocky / Alma / Fedora SELinux Enforcing — chuẩn hoá label đủ cả bin/lib/share.
    rc_paths=(
        "$BIN/textvn" "$BIN/textvn-settings" "$LIBDIR"
        "$SHARE/ibus/component/textvn.xml"
        "$SHARE/applications/textvn-settings.desktop"
        "$SHARE/icons/hicolor"
        "$SHARE/metainfo/io.github.hunglinhpt.textvn.metainfo.xml"
        "$SHARE/textvn"
    )
    [[ "$HAS_FCITX5" == 1 ]] && rc_paths+=("$FCITX_LIB/libtextvn-fcitx5.so" "$SHARE/fcitx5")
    restorecon -RF "${rc_paths[@]}" 2>/dev/null || true
fi

# Áp dụng ngay cho phiên hiện tại (không cần đăng xuất): khởi động lại framework đang
# chạy với đúng biến môi trường; thêm TextVN vào danh sách bộ gõ (trừ --no-activate).
if [[ "$PER_USER_STEPS" == 0 ]]; then
    tv_say "Chạy bằng root: KHÔNG sửa danh sách bộ gõ hay khởi động lại IBus/Fcitx5 (đó là thiết lập của từng người dùng)."
    echo "  Mỗi người dùng chạy trong phiên của mình (không dùng sudo):"
    [[ "$HAS_IBUS" == 1 ]] &&
        echo "    • IBus  : ibus restart — rồi thêm 'TextVN' trong Cài đặt → Bàn phím → Nguồn nhập."
    [[ "$HAS_FCITX5" == 1 ]] &&
        echo "    • Fcitx5: fcitx5 -r -d — rồi thêm 'TextVN' trong fcitx5-configtool."
elif [[ "$HAS_IBUS" == 1 && "$IBUS_ABI_OK" == 1 ]]; then
    if [[ "$RESTART" == 1 ]] && tv_running ibus-daemon; then
        if [[ "$MODE" == user ]]; then
            if tv_ibus_needs_live_register; then
                # E67/E68: GNOME Wayland không systemd IBus unit (RHEL 8.10 GNOME 3.32,
                # RHEL 9 GNOME 40, Ubuntu 22.04 GNOME 42) — không restart ibus-daemon;
                # dừng bản cũ (nếu đang nâng cấp) rồi đăng ký live ngay cho phiên hiện tại.
                tv_stop_wait_bus_engine "$LIBDIR/textvn-ibus-engine"
                nohup "$LIBDIR/textvn-ibus-engine" --wait-bus --setup "$BIN/textvn-settings" </dev/null >/dev/null 2>&1 &
                disown "$!" 2>/dev/null || true
                sleep 1
            else
                tv_restart_ibus "$SHARE/ibus/component:/usr/share/ibus/component"
            fi
        else
            tv_restart_ibus ""
        fi
    fi
    [[ "$ACTIVATE" == 1 ]] && tv_activate_ibus
fi
if [[ "$PER_USER_STEPS" == 1 && "$HAS_FCITX5" == 1 && "$FCITX5_ABI_OK" == 1 ]]; then
    action=""
    [[ "$ACTIVATE" == 1 ]] && action=add
    if [[ "$RESTART" == 1 ]]; then
        tv_restart_fcitx5 "$FCITX_DIRS" "" "$action"
    elif [[ "$action" == add ]] && ! tv_running fcitx5; then
        tv_activate_fcitx5
    fi
fi

if [[ "$PER_USER_STEPS" == 1 ]]; then
    CONFLICT_RC=0
    tv_check_ime_conflicts "" || CONFLICT_RC=$?
    if [[ "$CONFLICT_RC" == 2 ]]; then
        tv_warn "Vui lòng thoát bộ gõ đang xung đột ở trên rồi thử gõ lại với TextVN."
    fi
fi

tv_diag_log INFO INSTALL_OK "mode=$MODE ibus=$IBUS_ABI_OK fcitx5=$FCITX5_ABI_OK"
tv_ok "Đã cài TextVN."
echo
echo "  • Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (hoặc Ctrl + Shift + Space)."
echo "  • Bảng điều khiển: chạy 'textvn-settings' hoặc mở 'TextVN' trong menu ứng dụng."
echo "  • Chưa thấy TextVN trong danh sách bộ gõ? Thêm 'TextVN' trong Cài đặt → Bàn phím"
echo "    (GNOME/IBus) hoặc fcitx5-configtool, rồi đăng xuất và đăng nhập lại."
[[ -n "$ENV_FILE" ]] && echo "  • Biến môi trường per-user: $ENV_FILE (áp dụng từ lần đăng nhập sau)."
echo "  • Kiểm tra hệ thống / xuất log báo lỗi: $BIN/textvn doctor --export ~/textvn-diag.zip"
echo "  • Gỡ cài đặt: $SHARE/textvn/uninstall.sh"
