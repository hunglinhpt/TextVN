#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# install_macos.sh — One-line per-user installation for TextVN on macOS (Rule S5)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
USER_HOME="${HOME:-~}"

echo "=== TextVN macOS Installation ==="

# 1. Build if not already built
STAGE_DIR="$ROOT/dist/macos/stage"
if [ ! -d "$STAGE_DIR/TextVN.app" ] || [ ! -d "$STAGE_DIR/TextVN-IM.app" ]; then
    echo "Compiled bundles not found in stage. Building from source..."
    "$ROOT/scripts/build-macos.sh"
fi

# 2. Determine target destinations
IM_DIR="$USER_HOME/Library/Input Methods"
APP_DIR="/Applications"

if [ ! -w "$APP_DIR" ]; then
    APP_DIR="$USER_HOME/Applications"
fi

mkdir -p "$IM_DIR"
mkdir -p "$APP_DIR"

# 3. Terminate running instances
killall TextVN 2>/dev/null || true
killall TextVN-IM 2>/dev/null || true
sleep 0.5

# 4. Install App Bundles
echo "Installing TextVN-IM.app to $IM_DIR..."
rm -rf "$IM_DIR/TextVN-IM.app"
cp -R "$STAGE_DIR/TextVN-IM.app" "$IM_DIR/"

echo "Installing TextVN.app to $APP_DIR..."
rm -rf "$APP_DIR/TextVN.app"
cp -R "$STAGE_DIR/TextVN.app" "$APP_DIR/"

# 5. Initialize user configuration directory (Rule S5, S9)
CONFIG_DIR="$USER_HOME/Library/Application Support/TextVN"
mkdir -p "$CONFIG_DIR"
chmod 700 "$CONFIG_DIR"

CONFIG_FILE="$CONFIG_DIR/config.json"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default configuration at $CONFIG_FILE..."
    cat <<'EOF' > "$CONFIG_FILE"
{
  "config_version": 1,
  "enabled": true,
  "method": "telex",
  "diacritic_style": "new",
  "free_marking": true,
  "auto_restore_english": true,
  "auto_capitalize": true,
  "macro_trigger": "tab",
  "allow_macro_when_vi_off": false,
  "output_charset": "unicode_precomposed",
  "show_dialog_on_startup": true,
  "autostart": true,
  "non_preedit": true,
  "run_in_tray": true,
  "switch_key": "ctrl_shift",
  "macros": []
}
EOF
    chmod 600 "$CONFIG_FILE"
fi

# 6. Register input source with LaunchServices
if [ -x "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" ]; then
    echo "Registering Input Method with LaunchServices..."
    /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$IM_DIR/TextVN-IM.app"
fi

# 7. Launch Menu Bar App
echo "Starting TextVN Menu Bar App..."
open "$APP_DIR/TextVN.app"

echo ""
echo "✅ Installation complete!"
echo "👉 Để kích hoạt bộ gõ tiếng Việt trên macOS:"
echo "   1. Mở System Settings (Cài đặt hệ thống) → Keyboard (Bàn phím) → Input Sources (Nguồn đầu vào) → Edit..."
echo "   2. Bấm nút dấu (+) → Chọn Vietnamese (Tiếng Việt) → Chọn TextVN"
echo "   3. Chọn TextVN làm nguồn gõ mặc định."
