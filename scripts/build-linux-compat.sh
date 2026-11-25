#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-linux-compat.sh [out_dir] — dựng textvn-settings-gtk4 + libtextvn-fcitx5.so
# chuẩn ABI tương thích rộng (Ubuntu 22.04 LTS: glibc 2.35, GLIBCXX 3.4.30, GTK 4.6,
# Fcitx5 5.0.14) để chạy mượt trên Ubuntu 22.04/Mint 21/Pop!_OS 22.04, Debian 12,
# Ubuntu 24.04, Fedora 40-42, openSUSE Leap 15.6 và Arch (E68).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/target/linux-compat}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-compat}"
BUILD="$CARGO_TARGET_DIR/adapters"

echo "=== E68: Build Linux compat baseline ($(. /etc/os-release 2>/dev/null && echo "${PRETTY_NAME:-Linux}" || uname -a)) ==="
echo "  glibc : $(ldd --version | sed -n '1p')"
echo "  gcc   : $(gcc --version | sed -n '1p')"
echo "  g++   : $(g++ --version | sed -n '1p')"
echo "  gtk4  : $(pkg-config --modversion gtk4)"
echo "  fcitx5: $(pkg-config --modversion Fcitx5Core)"

cargo build --release --locked -p textvn-ffi --manifest-path "$ROOT/Cargo.toml"

FFI_DIR="$CARGO_TARGET_DIR/release"
echo "== compat build + ctest linux-settings (GTK4)"
cmake -S "$ROOT/adapters/linux-settings" -B "$BUILD/settings-gtk4" \
    -DCMAKE_BUILD_TYPE=Release \
    -DTEXTVN_FFI_DIR="$FFI_DIR" \
    -DTEXTVN_GTK_MAJOR=4 >/dev/null
cmake --build "$BUILD/settings-gtk4" -j"$(nproc)"
(cd "$BUILD/settings-gtk4" && ctest --output-on-failure)

echo "== compat build + ctest linux-fcitx5"
cmake -S "$ROOT/adapters/linux-fcitx5" -B "$BUILD/fcitx5" \
    -DCMAKE_BUILD_TYPE=Release \
    -DTEXTVN_FFI_DIR="$FFI_DIR" >/dev/null
cmake --build "$BUILD/fcitx5" -j"$(nproc)"
(cd "$BUILD/fcitx5" && ctest --output-on-failure)

echo "== compat ABI self-check"
"$BUILD/settings-gtk4/textvn-settings" --check-abi
"$BUILD/settings-gtk4/textvn-settings" --version

mkdir -p "$OUT/settings-gtk4" "$OUT/fcitx5"
install -m 0755 "$BUILD/settings-gtk4/textvn-settings" "$OUT/settings-gtk4/textvn-settings"
install -m 0755 "$BUILD/fcitx5/libtextvn-fcitx5.so" "$OUT/fcitx5/libtextvn-fcitx5.so"
if [[ -x "$BUILD/fcitx5/e2e_fcitx5" ]]; then
    install -m 0755 "$BUILD/fcitx5/e2e_fcitx5" "$OUT/fcitx5/e2e_fcitx5"
fi
echo "=== Linux compat baseline OK -> $OUT ==="
