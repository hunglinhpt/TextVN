#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall_macos.sh — Clean uninstallation for TextVN on macOS (Rule S9)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
USER_HOME="${HOME:-~}"

PURGE_MODE=0
for arg in "$@"; do
    if [ "$arg" = "--purge" ]; then
        PURGE_MODE=1
    fi
done

echo "=== TextVN macOS Uninstallation ==="

# 1. Stop running processes
echo "Stopping TextVN processes..."
killall TextVN 2>/dev/null || true
killall TextVN-IM 2>/dev/null || true
sleep 0.5

# 2. Disable and remove LaunchAgent autostart
PLIST_PATH="$USER_HOME/Library/LaunchAgents/vn.textvn.app.plist"
if [ -f "$PLIST_PATH" ]; then
    echo "Disabling autostart LaunchAgent..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    rm -f "$PLIST_PATH"
fi

# 3. Remove application bundles
echo "Removing application bundles..."
rm -rf "$USER_HOME/Library/Input Methods/TextVN-IM.app"
rm -rf "/Applications/TextVN.app"
rm -rf "$USER_HOME/Applications/TextVN.app"

# 4. Remove active IPC socket
rm -f "$USER_HOME/Library/Application Support/TextVN/ipc.sock"

# 5. Handle user configuration (Rule S9)
CONFIG_DIR="$USER_HOME/Library/Application Support/TextVN"
if [ "$PURGE_MODE" -eq 1 ]; then
    echo "Purging configuration and logs as requested by --purge..."
    rm -rf "$CONFIG_DIR"
    rm -rf "$USER_HOME/Library/Logs/TextVN"
else
    echo "ℹ️ [RULE S9] Preserving user configuration at $CONFIG_DIR."
    echo "   (Use --purge flag if you want to completely erase user settings and macros)."
fi

# 6. Verify 0 system residue
CHECK_SCRIPT="$ROOT/packaging/macos/uninstall-check.sh"
if [ ! -f "$CHECK_SCRIPT" ]; then
    CHECK_SCRIPT="$(dirname "${BASH_SOURCE[0]}")/uninstall-check.sh"
fi
if [ ! -f "$CHECK_SCRIPT" ]; then
    CHECK_SCRIPT="$(dirname "${BASH_SOURCE[0]}")/../Resources/uninstall-check.sh"
fi

if [ -f "$CHECK_SCRIPT" ]; then
    if [ "$PURGE_MODE" -eq 1 ]; then
        bash "$CHECK_SCRIPT" --purge
    else
        bash "$CHECK_SCRIPT"
    fi
fi

echo "✅ TextVN has been successfully uninstalled from your Mac."
