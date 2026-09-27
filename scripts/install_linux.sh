#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN - Vietnamese Input Method for Linux (Fcitx5 & IBus)
# Installer script inspired by BambooMintKey & Freedesktop Standards.
# Supports both Rootless Per-User (~/.local) and System (/usr) installations.

set -euo pipefail

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

INSTALL_MODE="user"
AUTO_RESTART=true
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

usage() {
    cat <<EOF
TextVN Linux Installer

Sử dụng:
  $0 [TÙY CHỌN]

Tùy chọn:
  --user        Cài đặt cục bộ cho người dùng hiện tại vào ~/.local (MẶC ĐỊNH, KHÔNG CẦN ROOT)
  --system      Cài đặt toàn hệ thống vào /usr (Yêu cầu sudo / root)
  --no-restart  Không tự động khởi động lại Fcitx5 hoặc IBus sau khi cài
  -h, --help    Hiển thị hướng dẫn này

Ví dụ:
  ./scripts/install_linux.sh              # Cài đặt rootless nhanh cho user hiện tại
  sudo ./scripts/install_linux.sh --system # Cài đặt cho toàn bộ người dùng máy
EOF
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --user)
            INSTALL_MODE="user"
            shift
            ;;
        --system)
            INSTALL_MODE="system"
            shift
            ;;
        --no-restart)
            AUTO_RESTART=false
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

echo -e "${CYAN}====================================================${NC}"
echo -e "${PURPLE}  TextVN - Bộ gõ Tiếng Việt hiện đại cho Linux${NC}"
echo -e "${CYAN}====================================================${NC}"
echo -e "Chế độ cài đặt: ${GREEN}${INSTALL_MODE^^}${NC}"

# Detect Linux Distribution
DISTRO="unknown"
if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    source /etc/os-release
    DISTRO="${ID:-unknown}"
fi
echo -e "Hệ điều hành phát hiện: ${BLUE}${DISTRO^}${NC} (${PRETTY_NAME:-Linux})"

# Validate root privileges for system install
if [[ "$INSTALL_MODE" == "system" ]] && [[ $EUID -ne 0 ]]; then
    echo -e "${RED}[LỖI] Chế độ --system yêu cầu quyền root. Vui lòng chạy lại với 'sudo $0 --system'${NC}"
    exit 1
fi

# Define Target Directories
if [[ "$INSTALL_MODE" == "user" ]]; then
    TARGET_BIN="$HOME/.local/bin"
    TARGET_LIB="$HOME/.local/lib/textvn"
    TARGET_FCITX5_ADDON="$HOME/.local/share/fcitx5/addon"
    TARGET_FCITX5_LIB="$HOME/.local/lib/fcitx5"
    TARGET_IBUS_COMPONENT="$HOME/.local/share/ibus/component"
    TARGET_ICONS="$HOME/.local/share/icons/hicolor/scalable/apps"
    TARGET_APPS="$HOME/.local/share/applications"
    TARGET_AUTOSTART="$HOME/.config/autostart"
else
    TARGET_BIN="/usr/bin"
    TARGET_LIB="/usr/lib/textvn"
    TARGET_FCITX5_ADDON="/usr/share/fcitx5/addon"
    if [[ -d "/usr/lib/x86_64-linux-gnu" ]]; then
        TARGET_FCITX5_LIB="/usr/lib/x86_64-linux-gnu/fcitx5"
    else
        TARGET_FCITX5_LIB="/usr/lib/fcitx5"
    fi
    TARGET_IBUS_COMPONENT="/usr/share/ibus/component"
    TARGET_ICONS="/usr/share/icons/hicolor/scalable/apps"
    TARGET_APPS="/usr/share/applications"
    TARGET_AUTOSTART="/etc/xdg/autostart"
fi

echo -e "\n${YELLOW}--> 1. Kiểm tra công cụ biên dịch và dependencies...${NC}"
MISSING_TOOLS=()
for tool in cargo cmake pkg-config; do
    if ! command -v "$tool" &>/dev/null; then
        MISSING_TOOLS+=("$tool")
    fi
done

if [[ ${#MISSING_TOOLS[@]} -gt 0 ]]; then
    echo -e "${RED}[LỖI] Thiếu các công cụ sau: ${MISSING_TOOLS[*]}${NC}"
    echo -e "Vui lòng cài đặt trước khi tiếp tục:"
    case "$DISTRO" in
        ubuntu|debian|linuxmint|pop)
            echo -e "  sudo apt update && sudo apt install -y cargo cmake pkg-config libfcitx5core-dev libibus-1.0-dev libgtk-4-dev"
            ;;
        fedora|rhel|centos)
            echo -e "  sudo dnf install -y cargo cmake pkg-config fcitx5-devel ibus-devel gtk4-devel"
            ;;
        arch|manjaro)
            echo -e "  sudo pacman -S --needed cargo cmake pkgconf fcitx5 ibus gtk4"
            ;;
        *)
            echo -e "  Hãy cài đặt Rust (cargo), CMake, pkg-config và dev headers cho Fcitx5/IBus/GTK4."
            ;;
    esac
    exit 1
fi
echo -e "${GREEN}   ✓ Công cụ build đã sẵn sàng.${NC}"

echo -e "\n${YELLOW}--> 2. Tạo thư mục đích...${NC}"
mkdir -p "$TARGET_BIN" "$TARGET_LIB" "$TARGET_FCITX5_ADDON" "$TARGET_FCITX5_LIB" \
         "$TARGET_IBUS_COMPONENT" "$TARGET_ICONS" "$TARGET_APPS" "$TARGET_AUTOSTART"
echo -e "${GREEN}   ✓ Đã tạo các thư mục cài đặt.${NC}"

echo -e "\n${YELLOW}--> 3. Biên dịch TextVN Core & Adapters...${NC}"
cd "$ROOT_DIR"
cargo build --release --bin textvn --bin textvn-tray 2>&1 | tail -n 10
echo -e "${GREEN}   ✓ Đã biên dịch xong các thành phần Rust.${NC}"

echo -e "\n${YELLOW}--> 4. Cài đặt các file nhị phân & cấu hình...${NC}"

# Install CLI and Tray
if [[ -f "$ROOT_DIR/target/release/textvn" ]]; then
    install -m 0755 "$ROOT_DIR/target/release/textvn" "$TARGET_BIN/textvn"
    echo -e "   + Đã cài: $TARGET_BIN/textvn"
fi

if [[ -f "$ROOT_DIR/target/release/textvn-tray" ]]; then
    install -m 0755 "$ROOT_DIR/target/release/textvn-tray" "$TARGET_BIN/textvn-tray"
    echo -e "   + Đã cài: $TARGET_BIN/textvn-tray"
fi

# Install Icons (Crimson V and Blue E)
if [[ -d "$ROOT_DIR/resources/icons" ]]; then
    if [[ -f "$ROOT_DIR/resources/icons/textvn_v.svg" ]]; then
        install -m 0644 "$ROOT_DIR/resources/icons/textvn_v.svg" "$TARGET_ICONS/textvn_v.svg"
        install -m 0644 "$ROOT_DIR/resources/icons/textvn_v.svg" "$TARGET_ICONS/textvn.svg"
        echo -e "   + Đã cài icon [V]: $TARGET_ICONS/textvn_v.svg"
    fi
    if [[ -f "$ROOT_DIR/resources/icons/textvn_e.svg" ]]; then
        install -m 0644 "$ROOT_DIR/resources/icons/textvn_e.svg" "$TARGET_ICONS/textvn_e.svg"
        echo -e "   + Đã cài icon [E]: $TARGET_ICONS/textvn_e.svg"
    fi
fi

# Install Fcitx5 Addon Conf
if [[ -f "$ROOT_DIR/packaging/linux/fcitx5/textvn.conf" ]]; then
    install -m 0644 "$ROOT_DIR/packaging/linux/fcitx5/textvn.conf" "$TARGET_FCITX5_ADDON/textvn.conf"
    echo -e "   + Đã cài Fcitx5 addon descriptor: $TARGET_FCITX5_ADDON/textvn.conf"
fi

# Install IBus Component XML
if [[ -f "$ROOT_DIR/packaging/linux/ibus/textvn.xml" ]]; then
    # Generate component xml with correct exec path for user or system
    TMP_XML=$(mktemp)
    if [[ "$INSTALL_MODE" == "user" ]]; then
        sed "s|/usr/lib/textvn/textvn-ibus-engine|$TARGET_LIB/textvn-ibus-engine|g" \
            "$ROOT_DIR/packaging/linux/ibus/textvn.xml" > "$TMP_XML"
    else
        cp "$ROOT_DIR/packaging/linux/ibus/textvn.xml" "$TMP_XML"
    fi
    install -m 0644 "$TMP_XML" "$TARGET_IBUS_COMPONENT/textvn.xml"
    rm -f "$TMP_XML"
    echo -e "   + Đã cài IBus component descriptor: $TARGET_IBUS_COMPONENT/textvn.xml"
fi

# Install Desktop files
if [[ -f "$ROOT_DIR/packaging/linux/desktop/textvn-settings.desktop" ]]; then
    install -m 0644 "$ROOT_DIR/packaging/linux/desktop/textvn-settings.desktop" "$TARGET_APPS/textvn-settings.desktop"
    echo -e "   + Đã cài Desktop Entry: $TARGET_APPS/textvn-settings.desktop"
fi

if [[ -f "$ROOT_DIR/packaging/linux/desktop/textvn-tray.desktop" ]]; then
    install -m 0644 "$ROOT_DIR/packaging/linux/desktop/textvn-tray.desktop" "$TARGET_AUTOSTART/textvn-tray.desktop"
    echo -e "   + Đã cài Autostart Entry: $TARGET_AUTOSTART/textvn-tray.desktop"
fi

# Update icon cache if tool available
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -q -t -f "$(dirname "$TARGET_ICONS")" || true
fi

echo -e "\n${YELLOW}--> 5. Khởi động lại dịch vụ Input Method...${NC}"
if [[ "$AUTO_RESTART" == true ]]; then
    # Restart Fcitx5 if running
    if pgrep -x "fcitx5" &>/dev/null; then
        echo -e "   Phát hiện Fcitx5 đang chạy, nạp lại cấu hình..."
        fcitx5 -r -d >/dev/null 2>&1 || true
        echo -e "${GREEN}   ✓ Đã khởi động lại Fcitx5 thành công.${NC}"
    fi

    # Restart IBus if running
    if pgrep -x "ibus-daemon" &>/dev/null; then
        echo -e "   Phát hiện IBus đang chạy, nạp lại cấu hình..."
        ibus restart >/dev/null 2>&1 || true
        echo -e "${GREEN}   ✓ Đã khởi động lại IBus thành công.${NC}"
    fi
else
    echo -e "   Bỏ qua khởi động lại theo yêu cầu (--no-restart)."
fi

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}  Cài đặt TextVN cho Linux thành công!${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Hướng dẫn sử dụng:"
echo -e "1. Khởi động giao diện điều khiển:"
echo -e "   ${CYAN}textvn-tray${NC} (hoặc mở 'TextVN Settings' từ Menu ứng dụng)"
echo -e "2. Thiết lập bộ gõ:"
echo -e "   • Với ${BLUE}Fcitx5${NC}: Mở Fcitx5 Configuration -> Thêm 'TextVN'"
echo -e "   • Với ${BLUE}IBus / GNOME${NC}: Settings -> Keyboard -> Input Sources -> Thêm 'TextVN'"
echo -e "3. Phím tắt chuyển đổi Tiếng Việt / Tiếng Anh: ${PURPLE}Ctrl + Shift${NC} (hoặc Alt + Z)"
echo -e "4. Để gỡ cài đặt sạch sẽ, chạy:"
echo -e "   ${YELLOW}./scripts/uninstall_linux.sh --${INSTALL_MODE}${NC}\n"
