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
[[ "${1:-}" == "--skip-tests" ]] && SKIP_TESTS=1
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

mkdir -p "$DIST"
"$ROOT/scripts/stage-linux.sh" "$DIST/$NAME" "$ADAPTERS"
tar -C "$DIST" -czf "$DIST/$NAME.tar.gz" "$NAME"
rm -rf "${DIST:?}/$NAME"
(cd "$DIST" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256")
echo "=== Xong: $DIST/$NAME.tar.gz ==="
cat "$DIST/$NAME.tar.gz.sha256"
