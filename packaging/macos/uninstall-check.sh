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
USER_HOME="${HOME:-~}"

echo "=== TextVN macOS Uninstall Residue Verification ==="

CHECK_TARGETS=(
    "$USER_HOME/Library/Input Methods/TextVN-IM.app"
    "$USER_HOME/Applications/TextVN.app"
    "/Applications/TextVN.app"
    "$USER_HOME/Library/LaunchAgents/vn.textvn.app.plist"
    "$USER_HOME/Library/Application Support/TextVN/ipc.sock"
)

for target in "${CHECK_TARGETS[@]}"; do
    if [ -e "$target" ]; then
        echo "❌ [RESIDUE] Found unexpected residual file/bundle: $target"
        RESIDUE_FOUND=1
    else
        echo "✅ [CLEAN] Not present: $target"
    fi
done

CONFIG_DIR="$USER_HOME/Library/Application Support/TextVN"
if [ "$PURGE_MODE" -eq 1 ]; then
    if [ -d "$CONFIG_DIR" ]; then
        echo "❌ [RESIDUE] Purge mode specified, but config directory still exists: $CONFIG_DIR"
        RESIDUE_FOUND=1
    else
        echo "✅ [CLEAN] Config directory purged as requested."
    fi
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
