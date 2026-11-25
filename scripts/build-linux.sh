#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-linux.sh — dựng gói Linux dist/TextVN-<version>-linux-<arch>.tar.gz
#
# Gói dùng được theo HAI cách (xem README trong gói):
#   1. Cài:           ./install.sh            (per-user, không cần root; --system cho /usr)
#   2. Chạy ngay:     ./textvn-portable.sh    (không cài gì, tắt bằng `stop`)
#
#   scripts/build-linux.sh [--skip-tests]
#
# Kiểm thử cả hai kịch bản với IBus/Fcitx5 thật: scripts/test-linux-package.sh <tarball>.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKIP_TESTS=0
EL8_DIR=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-tests) SKIP_TESTS=1 ;;
        --el8-dir) EL8_DIR="${2:?--el8-dir cần thư mục đầu ra của build-linux-el8.sh}"; shift ;;
        *) echo "Tuỳ chọn không hợp lệ: $1" >&2; exit 1 ;;
    esac
    shift
done
VERSION="$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -n1)"
ARCH="$(uname -m)"
DIST="$ROOT/dist"
NAME="TextVN-$VERSION-linux-$ARCH"
ADAPTERS="$ROOT/target/linux-adapters"

echo "=== TextVN $VERSION — gói Linux ($ARCH) ==="
if [[ "$SKIP_TESTS" == 0 ]]; then
    cargo test --workspace --manifest-path "$ROOT/Cargo.toml"
    # Build adapter (Release) + ctest + e2e với ibus-daemon/fcitx5 thật.
    "$ROOT/scripts/e2e-linux.sh" "$ADAPTERS"
else
    cargo build --release --locked -p textvn-ffi --manifest-path "$ROOT/Cargo.toml"
    for a in ibus fcitx5 settings; do
        cmake -S "$ROOT/adapters/linux-$a" -B "$ADAPTERS/$a" -DCMAKE_BUILD_TYPE=Release >/dev/null
        cmake --build "$ADAPTERS/$a" -j"$(nproc)"
    done
fi
cargo build --release --locked -p textvn-cli --manifest-path "$ROOT/Cargo.toml"

CLI_BIN="$ROOT/target/release/textvn-cli"
STAGE_ADAPTERS="$ADAPTERS"
TMP_ADAPTERS=""
trap '[[ -n "$TMP_ADAPTERS" ]] && rm -rf -- "$TMP_ADAPTERS"' EXIT
if [[ -n "$EL8_DIR" ]]; then
    echo "=== E67: ghép binary EL8 (glibc 2.28 + GTK3) vào gói đa distro ==="
    [[ -x "$EL8_DIR/bin/textvn-cli" ]] || { echo "thiếu $EL8_DIR/bin/textvn-cli" >&2; exit 1; }
    [[ -x "$EL8_DIR/ibus/textvn-ibus-engine" ]] || { echo "thiếu $EL8_DIR/ibus/textvn-ibus-engine" >&2; exit 1; }
    [[ -x "$EL8_DIR/settings-gtk3/textvn-settings" ]] || { echo "thiếu $EL8_DIR/settings-gtk3/textvn-settings" >&2; exit 1; }
    cargo run -q -p xtask --manifest-path "$ROOT/Cargo.toml" -- check-elf-abi --max-glibc 2.28 \
        "$EL8_DIR/bin/textvn-cli" \
        "$EL8_DIR/ibus/textvn-ibus-engine" \
        "$EL8_DIR/settings-gtk3/textvn-settings"
    CLI_BIN="$EL8_DIR/bin/textvn-cli"
    TMP_ADAPTERS="$(mktemp -d)"
    STAGE_ADAPTERS="$TMP_ADAPTERS"
    mkdir -p "$STAGE_ADAPTERS/ibus" "$STAGE_ADAPTERS/settings-gtk3"
    cp -f "$EL8_DIR/ibus/textvn-ibus-engine" "$STAGE_ADAPTERS/ibus/textvn-ibus-engine"
    cp -f "$EL8_DIR/settings-gtk3/textvn-settings" "$STAGE_ADAPTERS/settings-gtk3/textvn-settings"
    if [[ -f "$ADAPTERS/fcitx5/libtextvn-fcitx5.so" ]]; then
        mkdir -p "$STAGE_ADAPTERS/fcitx5"
        cp -f "$ADAPTERS/fcitx5/libtextvn-fcitx5.so" "$STAGE_ADAPTERS/fcitx5/libtextvn-fcitx5.so"
    fi
    if [[ -x "$ADAPTERS/settings/textvn-settings" ]]; then
        host_set_ver="$("$ADAPTERS/settings/textvn-settings" --version 2>/dev/null || true)"
        if grep -q "GTK4" <<<"$host_set_ver"; then
            mkdir -p "$STAGE_ADAPTERS/settings-gtk4"
            cp -f "$ADAPTERS/settings/textvn-settings" "$STAGE_ADAPTERS/settings-gtk4/textvn-settings"
        fi
    fi
fi

mkdir -p "$DIST"
TEXTVN_CLI_BIN="$CLI_BIN" "$ROOT/scripts/stage-linux.sh" "$DIST/$NAME" "$STAGE_ADAPTERS"
tar -C "$DIST" -czf "$DIST/$NAME.tar.gz" "$NAME"
rm -rf "${DIST:?}/$NAME"
(cd "$DIST" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256")
echo "=== Xong: $DIST/$NAME.tar.gz ==="
cat "$DIST/$NAME.tar.gz.sha256"
