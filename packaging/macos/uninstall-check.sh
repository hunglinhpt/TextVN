#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall-check.sh — Verify 0 system residue after macOS uninstallation (Rule S9, LNX/MAC-054)

set -euo pipefail

PURGE_MODE=0
for arg in "$@"; do
    if [ "$arg" = "--purge" ]; then
        PURGE_MODE=1
    fi
done

RESIDUE_FOUND=0
USER_HOME="${HOME:?HOME chưa được đặt}"
PKG_ID="vn.textvn.pkg"

# `-e` bỏ sót symlink hỏng — kiểm cả `-L` (review R3 F3-7).
present() { [ -e "$1" ] || [ -L "$1" ]; }

echo "=== TextVN macOS Uninstall Residue Verification ==="

CHECK_TARGETS=(
    "$USER_HOME/Library/Input Methods/TextVN-IM.app"
    "/Library/Input Methods/TextVN-IM.app"
    "$USER_HOME/Applications/TextVN.app"
    "/Applications/TextVN.app"
    "$USER_HOME/Library/LaunchAgents/vn.textvn.app.plist"
    "$USER_HOME/Library/Application Support/TextVN/ipc.sock"
)

for target in "${CHECK_TARGETS[@]}"; do
    if present "$target"; then
        echo "❌ [RESIDUE] Found unexpected residual file/bundle: $target"
        RESIDUE_FOUND=1
    else
        echo "✅ [CLEAN] Not present: $target"
    fi
done

# Receipt pkg (user volume + system) — review R3 F3-7b.
if command -v pkgutil >/dev/null 2>&1; then
    if pkgutil --volume "$USER_HOME" --pkg-info "$PKG_ID" >/dev/null 2>&1 ||
       pkgutil --pkg-info "$PKG_ID" >/dev/null 2>&1; then
        echo "❌ [RESIDUE] pkg receipt still registered: $PKG_ID"
        RESIDUE_FOUND=1
    else
        echo "✅ [CLEAN] No pkg receipt: $PKG_ID"
    fi
fi

CONFIG_DIR="$USER_HOME/Library/Application Support/TextVN"
LOG_DIR="$USER_HOME/Library/Logs/TextVN"
if [ "$PURGE_MODE" -eq 1 ]; then
    # --purge xoá cả config lẫn log → verify cả hai (review R3 F3-7d).
    for d in "$CONFIG_DIR" "$LOG_DIR"; do
        if present "$d"; then
            echo "❌ [RESIDUE] Purge mode specified, but directory still exists: $d"
            RESIDUE_FOUND=1
        else
            echo "✅ [CLEAN] Purged as requested: $d"
        fi
    done
else
    if [ -d "$CONFIG_DIR" ]; then
        echo "ℹ️ [RULE S9 PRESERVED] User configuration directory safely preserved: $CONFIG_DIR"
    else
        echo "ℹ️ Config directory not present."
    fi
fi

if [ "$RESIDUE_FOUND" -ne 0 ]; then
    echo "❌ Uninstallation verification FAILED: Residual artifacts detected."
    exit 1
fi

echo "✅ SUCCESS: 0 system residue verified. TextVN uninstallation cleanly confirmed."
exit 0
