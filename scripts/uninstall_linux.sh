#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall_linux.sh — gỡ TextVN đã cài (per-user mặc định; --system cho /usr).
# Gọi uninstall.sh được cài kèm (gỡ theo install-manifest.txt).

set -euo pipefail
PREFIX="$HOME/.local"
args=()
for a in "$@"; do
    case "$a" in
        --system) PREFIX=/usr ;;
        --user) PREFIX="$HOME/.local" ;;
        *) args+=("$a") ;;
    esac
done
UN="$PREFIX/share/textvn/uninstall.sh"
[[ -x "$UN" ]] || { echo "Không thấy $UN — TextVN chưa được cài ở $PREFIX." >&2; exit 1; }
exec "$UN" --prefix "$PREFIX" "${args[@]}"
