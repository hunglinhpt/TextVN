#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Build + unit test + kiểm thử đầu-cuối adapter Linux với ibus-daemon / fcitx5 THẬT.
#
# Cần: cargo, cmake, pkg-config, libibus-1.0-dev, libfcitx5core-dev, fcitx5-modules-dev,
#      ibus, fcitx5, dbus (dbus-run-session), python3-dbus, python3-gi.
# Dùng: scripts/e2e-linux.sh [build-dir]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD="${1:-$ROOT/target/linux-adapters}"
# python3 của distro (python3-dbus chỉ build cho interpreter hệ thống).
PY="${PYTHON:-}"
if [[ -z "$PY" ]]; then
    for c in /usr/bin/python3 /usr/bin/python3.*; do
        if [[ -x "$c" ]] && "$c" -c 'import dbus, gi' 2>/dev/null; then PY="$c"; break; fi
    done
fi

echo "== build engine (staticlib)"
cargo build --release -p textvn-ffi --manifest-path "$ROOT/Cargo.toml"

PARTS="common ibus fcitx5"
if pkg-config --exists gtk4 2>/dev/null; then PARTS="$PARTS settings"; fi
for a in $PARTS; do
    echo "== build + ctest linux-$a"
    cmake -S "$ROOT/adapters/linux-$a" -B "$BUILD/$a" -DCMAKE_BUILD_TYPE=Release >/dev/null
    cmake --build "$BUILD/$a" -j"$(nproc)"
    (cd "$BUILD/$a" && ctest --output-on-failure)
done

run_ibus() {
    local log; log="$(mktemp)"
    ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$log" 2>&1 &
    local daemon=$!
    sleep 2
    "$BUILD/ibus/textvn-ibus-engine" >>"$log" 2>&1 &
    local engine=$!
    sleep 1
    local rc=0
    "$BUILD/ibus/e2e_ibus" || rc=$?
    kill "$engine" "$daemon" 2>/dev/null || true
    [[ $rc -eq 0 ]] || cat "$log"
    return $rc
}

run_fcitx5() {
    local t; t="$(mktemp -d)"
    export HOME="$t" XDG_CONFIG_HOME="$t/config" XDG_DATA_HOME="$t/data"
    mkdir -p "$t/data/fcitx5/addon" "$t/data/fcitx5/inputmethod" "$t/config/fcitx5"
    cp "$ROOT/packaging/linux/fcitx5/addon/textvn.conf" "$t/data/fcitx5/addon/"
    cp "$ROOT/packaging/linux/fcitx5/inputmethod/textvn.conf" "$t/data/fcitx5/inputmethod/"
    cat >"$t/config/fcitx5/profile" <<'P'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=textvn

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=textvn
Layout=

[GroupOrder]
0=Default
P
    local sys_addons
    sys_addons="$(pkg-config --variable=libdir Fcitx5Core)/fcitx5"
    export FCITX_ADDON_DIRS="$BUILD/fcitx5:$sys_addons"
    fcitx5 --disable=all --enable=dbus,dbusfrontend,keyboard,textvn >"$t/fcitx5.log" 2>&1 &
    local daemon=$!
    local rc=0
    "$PY" "$ROOT/adapters/linux-fcitx5/tests/e2e_fcitx5.py" || rc=$?
    kill "$daemon" 2>/dev/null || true
    [[ $rc -eq 0 ]] || cat "$t/fcitx5.log"
    return $rc
}

export -f run_ibus run_fcitx5
export BUILD ROOT PY
echo "== e2e ibus-daemon"
dbus-run-session -- bash -c run_ibus
if [[ -n "$PY" ]]; then
    echo "== e2e fcitx5"
    dbus-run-session -- bash -c run_fcitx5
else
    echo "!! bỏ qua e2e fcitx5: không có python3 kèm python3-dbus/python3-gi" >&2
    exit 1
fi
echo "== Linux adapters: OK"
