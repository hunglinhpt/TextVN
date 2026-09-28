#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN - Vietnamese Input Method for Linux (Uninstaller)
# Safely removes TextVN files from Rootless User (~/.local) or System (/usr) directories.

set -euo pipefail

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

UNINSTALL_MODE="user"
PURGE_CONFIG=false

usage() {
    cat <<EOF
TextVN Linux Uninstaller

Sử dụng:
  $0 [TÙY CHỌN]

Tùy chọn:
  --user        Gỡ bỏ cài đặt cục bộ tại ~/.local (MẶC ĐỊNH)
  --system      Gỡ bỏ cài đặt toàn hệ thống tại /usr (Yêu cầu sudo / root)
  --purge       Xóa cả file cấu hình và dữ liệu người dùng (~/.config/TextVN)
  -h, --help    Hiển thị hướng dẫn này

Ví dụ:
  ./scripts/uninstall_linux.sh              # Gỡ bỏ cài đặt user thông thường
  sudo ./scripts/uninstall_linux.sh --system # Gỡ bỏ cài đặt system
EOF
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --user)
            UNINSTALL_MODE="user"
            shift
            ;;
        --system)
            UNINSTALL_MODE="system"
            shift
            ;;
        --purge)
            PURGE_CONFIG=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}[LỖI] Tùy chọn không hợp lệ: $1${NC}"
            usage
            ;;
    esac
done

echo -e "${PURPLE}--> Đang gỡ bỏ TextVN (Chế độ: ${UNINSTALL_MODE^^})...${NC}"

if [[ "$UNINSTALL_MODE" == "system" ]] && [[ $EUID -ne 0 ]]; then
    echo -e "${RED}[LỖI] Chế độ --system yêu cầu quyền root. Vui lòng chạy lại với 'sudo $0 --system'${NC}"
    exit 1
fi

if [[ "$UNINSTALL_MODE" == "user" ]]; then
    TARGET_BIN="$HOME/.local/bin"
    TARGET_LIB="$HOME/.local/lib/textvn"
    TARGET_FCITX5_ADDON="$HOME/.local/share/fcitx5/addon"
    TARGET_FCITX5_IM="$HOME/.local/share/fcitx5/inputmethod"
    TARGET_FCITX5_LIB="$HOME/.local/lib/fcitx5"
    TARGET_IBUS_COMPONENT="$HOME/.local/share/ibus/component"
    TARGET_ICONS="$HOME/.local/share/icons/hicolor/scalable/apps"
    TARGET_APPS="$HOME/.local/share/applications"
    TARGET_AUTOSTART="$HOME/.config/autostart"
    TARGET_SYSTEMD="$HOME/.config/systemd/user"
else
    TARGET_BIN="/usr/bin"
    TARGET_LIB="/usr/lib/textvn"
    TARGET_FCITX5_ADDON="/usr/share/fcitx5/addon"
    TARGET_FCITX5_IM="/usr/share/fcitx5/inputmethod"
    TARGET_FCITX5_LIB="/usr/lib/fcitx5"
    TARGET_IBUS_COMPONENT="/usr/share/ibus/component"
    TARGET_ICONS="/usr/share/icons/hicolor/scalable/apps"
    TARGET_APPS="/usr/share/applications"
    TARGET_AUTOSTART="/etc/xdg/autostart"
    TARGET_SYSTEMD="/usr/lib/systemd/user"
fi

FILES_TO_REMOVE=(
    "$TARGET_BIN/textvn"
    "$TARGET_BIN/textvn-tray"
    "$TARGET_BIN/textvn-settings"
    "$TARGET_LIB/textvn-ibus-engine"
    "$TARGET_FCITX5_LIB/libtextvn-fcitx5.so"
    "$TARGET_FCITX5_ADDON/textvn.conf"
    "$TARGET_FCITX5_IM/textvn.conf"
    "$TARGET_IBUS_COMPONENT/textvn.xml"
    "$TARGET_ICONS/textvn_v.svg"
    "$TARGET_ICONS/textvn_e.svg"
    "$TARGET_ICONS/textvn.svg"
    "$TARGET_APPS/textvn-settings.desktop"
    "$TARGET_AUTOSTART/textvn-tray.desktop"
    "$TARGET_AUTOSTART/textvn.desktop"
    "$TARGET_SYSTEMD/textvn-tray.service"
)
if [[ "$INSTALL_MODE" == "user" ]]; then
    FILES_TO_REMOVE+=("$HOME/.config/environment.d/60-textvn.conf")
fi

for file in "${FILES_TO_REMOVE[@]}"; do
    if [[ -f "$file" ]]; then
        rm -f "$file"
        echo -e "   - Đã xóa: $file"
    fi
done

if [[ -d "$TARGET_LIB" ]] && [[ -z "$(ls -A "$TARGET_LIB" 2>/dev/null)" ]]; then
    rmdir "$TARGET_LIB" || true
fi

if [[ "$PURGE_CONFIG" == true ]]; then
    echo -e "${YELLOW}--> Đang dọn dẹp cấu hình người dùng (--purge)...${NC}"
    rm -rf "$HOME/.config/TextVN" "$HOME/.local/state/TextVN"
    echo -e "   - Đã xóa: $HOME/.config/TextVN và $HOME/.local/state/TextVN"
fi

# Reload daemons
if pgrep -x "fcitx5" &>/dev/null; then
    echo -e "   Khởi động lại Fcitx5..."
    fcitx5 -r -d >/dev/null 2>&1 || true
fi

if pgrep -x "ibus-daemon" &>/dev/null; then
    echo -e "   Khởi động lại IBus..."
    ibus restart >/dev/null 2>&1 || true
fi

echo -e "${GREEN}✓ Đã gỡ bỏ TextVN thành công khỏi hệ thống!${NC}"
