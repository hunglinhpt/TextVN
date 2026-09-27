#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# TextVN - Linux Uninstall Verification Script (LNX-054 / P3-5 §5 / P3-6 §6)
# Verifies zero system/user residue after uninstallation, while checking config preservation (Rule S9).

set -euo pipefail

MODE="user"
PURGE=false

usage() {
    cat <<EOF
Usage: $0 [--user | --system] [--purge]
EOF
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --user) MODE="user"; shift ;;
        --system) MODE="system"; shift ;;
        --purge) PURGE=true; shift ;;
        *) usage ;;
    esac
done

if [[ "$MODE" == "user" ]]; then
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

FILES_TO_CHECK=(
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
    "$TARGET_APPS/textvn-settings.desktop"
    "$TARGET_AUTOSTART/textvn-tray.desktop"
    "$TARGET_AUTOSTART/textvn.desktop"
    "$TARGET_SYSTEMD/textvn-tray.service"
)

RESIDUE_COUNT=0

for file in "${FILES_TO_CHECK[@]}"; do
    if [[ -e "$file" ]]; then
        echo "[RESIDUE FOUND] $file"
        RESIDUE_COUNT=$((RESIDUE_COUNT + 1))
    fi
done

if [[ -d "$TARGET_LIB" ]] && [[ -z "$(ls -A "$TARGET_LIB" 2>/dev/null)" ]]; then
    echo "[EMPTY DIR RESIDUE] $TARGET_LIB"
    RESIDUE_COUNT=$((RESIDUE_COUNT + 1))
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/TextVN"

if [[ "$PURGE" == true ]]; then
    if [[ -d "$CONFIG_DIR" ]]; then
        echo "[PURGE FAILED] Config directory still exists: $CONFIG_DIR"
        RESIDUE_COUNT=$((RESIDUE_COUNT + 1))
    fi
else
    # Non-purge uninstall MUST NOT delete user config (Rule S9)
    if [[ -d "$CONFIG_DIR" ]]; then
        echo "[OK] User config preserved as required by Rule S9: $CONFIG_DIR"
    fi
fi

if [[ $RESIDUE_COUNT -eq 0 ]]; then
    echo "[PASS] 0 residue found. Clean uninstall verified."
    exit 0
else
    echo "[FAIL] $RESIDUE_COUNT residue items found."
    exit 1
fi
