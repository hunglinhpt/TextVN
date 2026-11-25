#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# test-linux-el8-package.sh <tarball> <e2e_ibus_el8_bin>
# Kiểm thử đầu-cuối gói phát hành đa distro NGAY BÊN TRONG RHEL 8.10 / AlmaLinux 8
# (glibc 2.28, Bash 4.4, IBus 1.5.19, GTK 3.22, KHÔNG có GTK4, KHÔNG có Fcitx5 — E67).

set -euo pipefail

TARBALL="$(realpath "${1:?cần đường dẫn tarball}")"
E2E_IBUS="$(realpath "${2:?cần đường dẫn e2e_ibus (EL8)}")"
[[ -x "$E2E_IBUS" ]] || { echo "FAIL: thiếu $E2E_IBUS" >&2; exit 1; }

fail() { echo "FAIL (EL8): $*" >&2; exit 1; }
export -f fail
export TARBALL E2E_IBUS

new_el8_home() {
    local t
    t="$(mktemp -d)"
    mkdir -p "$t/home" "$t/run" "$t/fakebin"
    chmod 0700 "$t/run"
    cat > "$t/fakebin/gsettings" <<'G'
#!/usr/bin/env bash
store="$HOME/.fake-gsettings"
case "$1" in
    list-schemas) echo org.gnome.desktop.input-sources ;;
    get) cat "$store" 2>/dev/null || echo "[('xkb', 'us')]" ;;
    set) echo "$4" > "$store" ;;
esac
G
    chmod +x "$t/fakebin/gsettings"
    echo "$t"
}
export -f new_el8_home

echo "=== E67 EL8 package E2E ($(cat /etc/redhat-release 2>/dev/null || uname -a); bash $BASH_VERSION) ==="

# 1. Kiểm binary + launcher GTK3 tự động trên máy KHÔNG có GTK4.
t_abi="$(mktemp -d)"
tar -C "$t_abi" -xzf "$TARBALL"
pkg_abi="$(echo "$t_abi"/TextVN-*)"
"$pkg_abi/bin/textvn" --help >/dev/null || fail "bin/textvn --help lỗi trên EL8"
doc_out="$("$pkg_abi/bin/textvn" doctor)" || fail "textvn doctor lỗi trên EL8"
grep -q "TextVN doctor (Linux)" <<<"$doc_out" || fail "textvn doctor không nhận diện Linux: $doc_out"
"$pkg_abi/lib/textvn/textvn-ibus-engine" --check-abi || fail "textvn-ibus-engine --check-abi lỗi trên EL8"
"$pkg_abi/bin/textvn-settings" --check-abi || fail "textvn-settings --check-abi (GTK3 fallback) lỗi trên EL8"
ver_out="$("$pkg_abi/bin/textvn-settings" --version)" || fail "textvn-settings --version lỗi trên EL8"
grep -q "GTK3" <<<"$ver_out" || fail "textvn-settings không tự chọn GTK3 trên EL8: $ver_out"
# Kiểm Bash 4.4 escape XML & source-guard
out_src="$(bash -c "source '$pkg_abi/install.sh' 2>&1; echo ALIVE" || true)"
grep -q 'ALIVE' <<<"$out_src" || fail "Bash 4.4 source-guard bị đóng shell"
# shellcheck source=packaging/linux/textvn-common.sh
. "$pkg_abi/textvn-common.sh"
xml_out="$(tv_ibus_component "$pkg_abi" "/tmp/A & B/it's <x>/lib/textvn/textvn-ibus-engine" "/tmp/A & B/it's <x>/bin/textvn-settings")"
grep -Fq "&amp;" <<<"$xml_out" && ! grep -Fq '"&amp;"' <<<"$xml_out" || fail "Bash 4.4 XML escape sai: $xml_out"
rm -rf "$t_abi"
echo "PASS EL8-1: ABI glibc 2.28 + GTK3 auto-select + textvn doctor + Bash 4.4 XML escape"

# 2. Kịch bản CÀI ĐẶT per-user trên EL8 + IBus 1.5.19 thật + gỡ sạch.
el8_install() {
    local t daemon_pid=""
    cleanup() {
        ibus exit >/dev/null 2>&1 || true
        [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
    }
    trap cleanup EXIT
    t="$(new_el8_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
    export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
    unset IBUS_COMPONENT_PATH
    mkdir -p "$t/x"
    tar -C "$t/x" -xzf "$TARBALL"
    local pkg pre="$HOME/.local"
    pkg="$(echo "$t"/x/TextVN-*)"

    "$pkg/install.sh" --no-restart >"$t/install.log" 2>&1 || { cat "$t/install.log"; fail "install.sh"; }
    [[ -x "$pre/bin/textvn" && -x "$pre/bin/textvn-settings" && -x "$pre/lib/textvn/textvn-ibus-engine" ]] ||
        fail "thiếu binary sau install.sh"
    "$pre/bin/textvn-settings" --check-abi || fail "textvn-settings sau cài lỗi ABI trên EL8"
    grep -q "'textvn'" "$HOME/.fake-gsettings" || fail "install.sh không thêm textvn vào GNOME input-sources"

    set -a
    # shellcheck disable=SC1090
    . "$XDG_CONFIG_HOME/environment.d/60-textvn.conf"
    set +a
    local rc=0
    ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
    daemon_pid=$!
    sleep 2
    "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
    if [[ $rc -ne 0 ]]; then
        echo "--- ibus list-engine (EL8 debug) ---"
        ibus list-engine 2>&1 || true
        ls -la "$XDG_CACHE_HOME/ibus/bus" 2>&1 || true
    fi
    cleanup
    sleep 1
    [[ $rc -eq 0 ]] || { cat "$t/e2e.log" "$t/d.log"; fail "EL8 install e2e_ibus"; }
    tail -n1 "$t/e2e.log"

    "$pre/share/textvn/uninstall.sh" --no-restart >"$t/un.log" 2>&1 || { cat "$t/un.log"; fail "uninstall.sh"; }
    local left
    left="$(find "$HOME" \( -path "$HOME/.config/TextVN" -o -path "$HOME/.local/state/TextVN" \) -prune \
        -o \( -iname '*textvn*' -print \) | grep -v fake-gsettings || true)"
    [[ -z "$left" ]] || fail "còn sót sau khi gỡ trên EL8: $left"
    rm -rf "$t"
    trap - EXIT
    echo "PASS EL8-2: cài đặt per-user → gõ tiếng Việt qua IBus 1.5.19 → gỡ sạch"
}
export -f el8_install
dbus-run-session -- bash -c el8_install

# 3. Kịch bản CHẠY NGAY (portable) trong thư mục chỉ đọc có khoảng trắng + dấu trên EL8.
el8_portable() {
    local t="" pkg="" daemon_pid=""
    cleanup() {
        ibus exit >/dev/null 2>&1 || true
        [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
        [[ -n "$pkg" && -d "$pkg" ]] && chmod -R u+w "$pkg" 2>/dev/null || true
        [[ -n "$t" && -d "$t" ]] && rm -rf "$t" 2>/dev/null || true
    }
    trap cleanup EXIT
    t="$(new_el8_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
    export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
    unset IBUS_COMPONENT_PATH
    local x="$t/Tải về"
    mkdir -p "$x"
    tar -C "$x" -xzf "$TARBALL"
    pkg="$(echo "$x"/TextVN-*)"
    chmod -R a-w "$pkg"

    ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
    daemon_pid=$!
    sleep 2
    "$pkg/textvn-portable.sh" --ibus >"$t/p.log" 2>&1 || { cat "$t/p.log"; fail "portable start (EL8)"; }
    sleep 1
    local rc=0
    "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
    [[ $rc -eq 0 ]] || { cat "$t/p.log" "$t/e2e.log" "$t/d.log"; fail "EL8 portable e2e_ibus"; }
    tail -n1 "$t/e2e.log"
    "$pkg/textvn-portable.sh" stop >"$t/s.log" 2>&1 || { cat "$t/s.log"; fail "portable stop (EL8)"; }
    [[ ! -e "$XDG_RUNTIME_DIR/textvn-portable-$(id -u)" ]] || fail "stop không dọn runtime dir (EL8)"
    chmod -R u+w "$pkg"
    cleanup
    rm -rf "$t"
    trap - EXIT
    echo "PASS EL8-3: portable (thư mục chỉ đọc) → gõ tiếng Việt qua IBus 1.5.19 → stop sạch"
}
export -f el8_portable
dbus-run-session -- bash -c el8_portable

# 4. Kịch bản GNOME Wayland RHEL 8.10 (GNOME 3.32 không systemd IBus unit):
#    portable live-register KHÔNG được restart ibus-daemon và gõ tiếng Việt ngay lập tức!
el8_wayland_live_portable() {
    local t daemon_pid=""
    cleanup() {
        ibus exit >/dev/null 2>&1 || true
        [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
    }
    trap cleanup EXIT
    t="$(new_el8_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
    export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
    export XDG_SESSION_TYPE=wayland XDG_CURRENT_DESKTOP=GNOME TV_SKIP_MANAGER_ENV=1
    unset IBUS_COMPONENT_PATH
    local x="$t/Tải về"
    mkdir -p "$x"
    tar -C "$x" -xzf "$TARBALL"
    local pkg
    pkg="$(echo "$x"/TextVN-*)"

    ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
    daemon_pid=$!
    sleep 2
    "$pkg/textvn-portable.sh" --ibus >"$t/p.log" 2>&1 || { cat "$t/p.log"; fail "Wayland live portable start"; }
    kill -0 "$daemon_pid" 2>/dev/null || fail "Wayland live portable đã giết/restart ibus-daemon gốc!"
    local rc=0
    "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
    [[ $rc -eq 0 ]] || { cat "$t/p.log" "$t/e2e.log" "$t/d.log"; fail "Wayland live portable e2e_ibus"; }
    tail -n1 "$t/e2e.log"
    "$pkg/textvn-portable.sh" stop >"$t/s.log" 2>&1 || { cat "$t/s.log"; fail "Wayland live portable stop"; }
    kill -0 "$daemon_pid" 2>/dev/null || fail "Wayland live portable stop đã giết ibus-daemon gốc!"
    ! pgrep -f -u "$(id -u)" 'textvn-ibus-engine' >/dev/null 2>&1 || fail "Wayland live portable stop để sót textvn-ibus-engine"
    cleanup
    rm -rf "$t"
    trap - EXIT
    echo "PASS EL8-4: GNOME Wayland RHEL 8.10 portable live-register (ibus-daemon nguyên vẹn + gõ ngay + stop sạch)"
}
export -f el8_wayland_live_portable
dbus-run-session -- bash -c el8_wayland_live_portable

echo "=== TOÀN BỘ 4/4 KIỂM THỬ RHEL 8.10 / ALMALINUX 8 PASS ==="
