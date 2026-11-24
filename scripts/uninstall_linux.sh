#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# uninstall_linux.sh — gỡ TextVN đã cài (per-user mặc định; --system cho /usr).
# Gọi uninstall.sh được cài kèm (gỡ theo install-manifest.txt).

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
    echo "✗ Không dùng 'source' hay '.' với uninstall_linux.sh (sẽ đóng terminal). Hãy chạy trực tiếp: ./scripts/uninstall_linux.sh" >&2
    return 1
fi

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
"$UN" --prefix "$PREFIX" "${args[@]}"
