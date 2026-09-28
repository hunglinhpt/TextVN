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
    if [[ -d "/usr/lib/x86_64-linux-gnu" ]]; then
        TARGET_FCITX5_LIB="/usr/lib/x86_64-linux-gnu/fcitx5"
    else
        TARGET_FCITX5_LIB="/usr/lib/fcitx5"
    fi
    TARGET_IBUS_COMPONENT="/usr/share/ibus/component"
    TARGET_ICONS="/usr/share/icons/hicolor/scalable/apps"
    TARGET_APPS="/usr/share/applications"
    TARGET_AUTOSTART="/etc/xdg/autostart"
    TARGET_SYSTEMD="/usr/lib/systemd/user"
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
mkdir -p "$TARGET_BIN" "$TARGET_LIB" "$TARGET_FCITX5_ADDON" "$TARGET_FCITX5_IM" "$TARGET_FCITX5_LIB" \
         "$TARGET_IBUS_COMPONENT" "$TARGET_ICONS" "$TARGET_APPS"
echo -e "${GREEN}   ✓ Đã tạo các thư mục cài đặt.${NC}"

echo -e "\n${YELLOW}--> 3. Biên dịch TextVN Core & Adapters...${NC}"
cd "$ROOT_DIR"
# Chỉ engine (staticlib) + CLI: tray là thành phần Windows.
cargo build --release -p textvn-ffi -p textvn-cli 2>&1 | tail -n 5
echo -e "${GREEN}   ✓ Đã biên dịch engine + CLI.${NC}"

BUILD_DIR="$ROOT_DIR/target/linux-adapters"
BUILT_ANY=false
if pkg-config --exists ibus-1.0 2>/dev/null; then
    echo -e "   Biên dịch IBus engine..."
    cmake -S "$ROOT_DIR/adapters/linux-ibus" -B "$BUILD_DIR/ibus" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$BUILD_DIR/ibus" -j"$(nproc)"
    BUILT_ANY=true
fi
if pkg-config --exists Fcitx5Core 2>/dev/null; then
    echo -e "   Biên dịch Fcitx5 addon..."
    cmake -S "$ROOT_DIR/adapters/linux-fcitx5" -B "$BUILD_DIR/fcitx5" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$BUILD_DIR/fcitx5" -j"$(nproc)"
    BUILT_ANY=true
fi
if pkg-config --exists gtk4 2>/dev/null; then
    echo -e "   Biên dịch GTK4 Settings panel..."
    cmake -S "$ROOT_DIR/adapters/linux-settings" -B "$BUILD_DIR/settings" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$BUILD_DIR/settings" -j"$(nproc)"
fi
if [[ "$BUILT_ANY" != true ]]; then
    echo -e "${RED}[LỖI] Không tìm thấy dev headers của IBus (libibus-1.0-dev) hay Fcitx5 (libfcitx5core-dev + fcitx5-modules-dev).${NC}"
    echo -e "Không có adapter nào → TextVN không thể gõ. Cài header rồi chạy lại."
    exit 1
fi

echo -e "\n${YELLOW}--> 4. Cài đặt các file nhị phân & cấu hình...${NC}"
install -m 0755 "$ROOT_DIR/target/release/textvn-cli" "$TARGET_BIN/textvn"
echo -e "   + CLI: $TARGET_BIN/textvn"

if [[ -f "$BUILD_DIR/settings/textvn-settings" ]]; then
    install -m 0755 "$BUILD_DIR/settings/textvn-settings" "$TARGET_BIN/textvn-settings"
    echo -e "   + Settings UI: $TARGET_BIN/textvn-settings"
fi

if [[ -f "$BUILD_DIR/ibus/textvn-ibus-engine" ]]; then
    install -m 0755 "$BUILD_DIR/ibus/textvn-ibus-engine" "$TARGET_LIB/textvn-ibus-engine"
    sed -e "s|/usr/lib/textvn/textvn-ibus-engine|$TARGET_LIB/textvn-ibus-engine|g" \
        -e "s|/usr/bin/textvn-settings|$TARGET_BIN/textvn-settings|g" \
        "$ROOT_DIR/packaging/linux/ibus/textvn.xml" > "$TARGET_IBUS_COMPONENT/textvn.xml"
    chmod 0644 "$TARGET_IBUS_COMPONENT/textvn.xml"
    echo -e "   + IBus engine: $TARGET_LIB/textvn-ibus-engine"
fi

if [[ -f "$BUILD_DIR/fcitx5/libtextvn-fcitx5.so" ]]; then
    install -m 0755 "$BUILD_DIR/fcitx5/libtextvn-fcitx5.so" "$TARGET_FCITX5_LIB/libtextvn-fcitx5.so"
    install -m 0644 "$ROOT_DIR/packaging/linux/fcitx5/addon/textvn.conf" "$TARGET_FCITX5_ADDON/textvn.conf"
    install -m 0644 "$ROOT_DIR/packaging/linux/fcitx5/inputmethod/textvn.conf" "$TARGET_FCITX5_IM/textvn.conf"
    echo -e "   + Fcitx5 addon: $TARGET_FCITX5_LIB/libtextvn-fcitx5.so"
fi

for icon in textvn_v textvn_e; do
    install -m 0644 "$ROOT_DIR/resources/icons/$icon.svg" "$TARGET_ICONS/$icon.svg"
done
install -m 0644 "$ROOT_DIR/packaging/linux/desktop/textvn-settings.desktop" "$TARGET_APPS/textvn-settings.desktop"

# Cài per-user: ibus-daemon chỉ quét /usr/share/ibus/component và fcitx5 chỉ nạp addon
# trong libdir hệ thống, trừ khi có IBUS_COMPONENT_PATH / FCITX_ADDON_DIRS. Khai báo qua
# environment.d (systemd user session: GNOME, KDE…) — có hiệu lực từ lần đăng nhập sau.
if [[ "$INSTALL_MODE" == "user" ]]; then
    ENV_DIR="$HOME/.config/environment.d"
    mkdir -p "$ENV_DIR"
    SYS_FCITX_ADDONS="$(pkg-config --variable=libdir Fcitx5Core 2>/dev/null || echo /usr/lib)/fcitx5"
    cat > "$ENV_DIR/60-textvn.conf" <<ENVEOF
# TextVN (cài per-user) — tạo bởi install_linux.sh
IBUS_COMPONENT_PATH=$TARGET_IBUS_COMPONENT:/usr/share/ibus/component
FCITX_ADDON_DIRS=$TARGET_FCITX5_LIB:$SYS_FCITX_ADDONS
ENVEOF
    echo -e "   + Biến môi trường per-user: $ENV_DIR/60-textvn.conf ${YELLOW}(đăng xuất/đăng nhập lại để áp dụng)${NC}"
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
echo -e "1. Cài đặt kiểu gõ (Telex/VNI/VIQR…):"
echo -e "   ${CYAN}textvn-settings${NC} (hoặc 'TextVN' trong Menu ứng dụng)"
echo -e "2. Thiết lập bộ gõ:"
echo -e "   • Với ${BLUE}Fcitx5${NC}: Mở Fcitx5 Configuration -> Thêm 'TextVN'"
echo -e "   • Với ${BLUE}IBus / GNOME${NC}: Settings -> Keyboard -> Input Sources -> Thêm 'TextVN'"
echo -e "3. Phím chuyển Tiếng Việt / Tiếng Anh: ${PURPLE}Ctrl + Shift${NC} (nhấn rồi nhả) hoặc ${PURPLE}Ctrl + Shift + Space${NC}"
echo -e "4. Để gỡ cài đặt sạch sẽ, chạy:"
echo -e "   ${YELLOW}./scripts/uninstall_linux.sh --${INSTALL_MODE}${NC}\n"
