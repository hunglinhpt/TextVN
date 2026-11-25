#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# install_linux.sh — biên dịch TextVN từ mã nguồn rồi cài bằng CHÍNH install.sh của gói
# (một đường cài duy nhất cho mã nguồn và tarball).
#
#   scripts/install_linux.sh                 # per-user (~/.local), không cần root
#   sudo -E scripts/install_linux.sh --system
#
# Mọi tuỳ chọn được chuyển nguyên cho install.sh (--no-restart, --no-activate, --prefix).
# Cần: cargo, cmake, pkg-config, gcc/g++ và header IBus (libibus-1.0-dev) và/hoặc Fcitx5
# (libfcitx5core-dev fcitx5-modules-dev), GTK4 (libgtk-4-dev) cho bảng điều khiển.

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then
    echo "✗ Không dùng 'source' hay '.' với install_linux.sh (sẽ đóng terminal). Hãy chạy trực tiếp: ./scripts/install_linux.sh" >&2
    return 1
fi

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ADAPTERS="$ROOT/target/linux-adapters"

for tool in cargo cmake pkg-config; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "Thiếu '$tool'." >&2
        echo "  • Ubuntu / Debian              : sudo apt install -y cargo cmake pkg-config g++ libibus-1.0-dev libgtk-4-dev" >&2
        echo "  • RHEL 8/9 / Rocky / Alma / Fed: sudo dnf install -y cmake pkgconf-pkg-config gcc gcc-c++ ibus-devel gtk3-devel (RHEL 8 cần bật CodeReady Builder / Alma-Rocky 8 thêm --enablerepo=powertools; Rust qua rustup.rs)" >&2
        exit 1
    }
done

cargo build --release -p textvn-ffi -p textvn-cli --manifest-path "$ROOT/Cargo.toml"
built=0
if pkg-config --exists ibus-1.0; then
    cmake -S "$ROOT/adapters/linux-ibus" -B "$ADAPTERS/ibus" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$ADAPTERS/ibus" -j"$(nproc)"
    built=1
fi
if pkg-config --exists Fcitx5Core; then
    cmake -S "$ROOT/adapters/linux-fcitx5" -B "$ADAPTERS/fcitx5" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$ADAPTERS/fcitx5" -j"$(nproc)"
    built=1
fi
if pkg-config --exists gtk4 || pkg-config --exists gtk+-3.0; then
    cmake -S "$ROOT/adapters/linux-settings" -B "$ADAPTERS/settings" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$ADAPTERS/settings" -j"$(nproc)"
fi
if [[ "$built" == 0 ]]; then
    echo "Không có header IBus (libibus-1.0-dev / ibus-devel) hay Fcitx5 (libfcitx5core-dev fcitx5-modules-dev) → không có adapter nào để cài." >&2
    exit 1
fi

STAGE="$ROOT/target/linux-stage"
"$ROOT/scripts/stage-linux.sh" "$STAGE" "$ADAPTERS"
"$STAGE/install.sh" "$@"
