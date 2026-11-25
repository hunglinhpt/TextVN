#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# build-linux-el8.sh [out_dir] — dựng các binary Linux chuẩn ABI EL8 (glibc 2.28,
# GLib 2.56, IBus 1.5.19, GTK 3.22) bên trong môi trường RHEL 8.10 / AlmaLinux 8 /
# Rocky Linux 8 để gói tarball chạy được từ RHEL 8.10 tới Ubuntu 24.04 / Arch (E67).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/target/linux-el8}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target-el8}"
BUILD="$CARGO_TARGET_DIR/adapters"

echo "=== E67: Build EL8 baseline ($(cat /etc/redhat-release 2>/dev/null || uname -a)) ==="
echo "  glibc: $(ldd --version | sed -n '1p')"
echo "  gcc  : $(gcc --version | sed -n '1p')"
echo "  ibus : $(pkg-config --modversion ibus-1.0)"
echo "  gtk3 : $(pkg-config --modversion gtk+-3.0)"

cargo build --release --locked -p textvn-ffi -p textvn-cli --manifest-path "$ROOT/Cargo.toml"

FFI_DIR="$CARGO_TARGET_DIR/release"
for a in common ibus; do
    echo "== EL8 build + ctest linux-$a"
    cmake -S "$ROOT/adapters/linux-$a" -B "$BUILD/$a" \
        -DCMAKE_BUILD_TYPE=Release \
        -DTEXTVN_FFI_DIR="$FFI_DIR" >/dev/null
    cmake --build "$BUILD/$a" -j"$(nproc)"
    (cd "$BUILD/$a" && ctest --output-on-failure)
done

echo "== EL8 build + ctest linux-settings (GTK3)"
cmake -S "$ROOT/adapters/linux-settings" -B "$BUILD/settings-gtk3" \
    -DCMAKE_BUILD_TYPE=Release \
    -DTEXTVN_FFI_DIR="$FFI_DIR" \
    -DTEXTVN_GTK_MAJOR=3 >/dev/null
cmake --build "$BUILD/settings-gtk3" -j"$(nproc)"
(cd "$BUILD/settings-gtk3" && ctest --output-on-failure)

echo "== EL8 ABI self-check"
"$FFI_DIR/textvn-cli" --help >/dev/null
"$BUILD/ibus/textvn-ibus-engine" --check-abi
"$BUILD/ibus/textvn-ibus-engine" --version
"$BUILD/settings-gtk3/textvn-settings" --check-abi
"$BUILD/settings-gtk3/textvn-settings" --version

run_el8_ibus_e2e() {
    local log t daemon engine rc=0
    log="$(mktemp)"
    t="$(mktemp -d)"
    export HOME="$t" XDG_CONFIG_HOME="$t/config" XDG_CACHE_HOME="$t/cache"
    ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$log" 2>&1 &
    daemon=$!
    sleep 2
    "$BUILD/ibus/textvn-ibus-engine" >>"$log" 2>&1 &
    engine=$!
    sleep 1
    "$BUILD/ibus/e2e_ibus" || rc=$?
    kill "$engine" "$daemon" 2>/dev/null || true
    wait "$engine" "$daemon" 2>/dev/null || true
    [[ $rc -eq 0 ]] || cat "$log"
    rm -rf "$t" "$log"
    return $rc
}
export -f run_el8_ibus_e2e
export BUILD
echo "== EL8 e2e ibus-daemon (IBus $(pkg-config --modversion ibus-1.0))"
dbus-run-session -- bash -c run_el8_ibus_e2e

mkdir -p "$OUT/bin" "$OUT/ibus" "$OUT/settings-gtk3"
install -m 0755 "$FFI_DIR/textvn-cli" "$OUT/bin/textvn-cli"
install -m 0755 "$BUILD/ibus/textvn-ibus-engine" "$OUT/ibus/textvn-ibus-engine"
install -m 0755 "$BUILD/ibus/e2e_ibus" "$OUT/ibus/e2e_ibus"
install -m 0755 "$BUILD/settings-gtk3/textvn-settings" "$OUT/settings-gtk3/textvn-settings"
echo "=== EL8 baseline OK -> $OUT ==="
