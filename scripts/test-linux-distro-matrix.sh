#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# test-linux-distro-matrix.sh [--system-only] <distro-tag> <tarball> <e2e_ibus_bin> [e2e_fcitx5_bin]
# Kiểm thử đầu-cuối ĐÚNG MỘT gói phát hành đa distro trên các distro Linux doanh nghiệp
# và phổ thông hàng đầu: RHEL 9 / AlmaLinux 9, SLES 15 / openSUSE Leap 15.6,
# Ubuntu 22.04 LTS, Debian 12 Bookworm, Fedora 41 (E68).

set -euo pipefail

SYSTEM_ONLY=0
if [[ "${1:-}" == --system-only ]]; then
    SYSTEM_ONLY=1
    shift
fi
DISTRO="${1:?cần tên distro (vd. almalinux-9, ubuntu-22-04, debian-12, opensuse-leap-15-6, fedora-41)}"
TARBALL="$(realpath "${2:?cần đường dẫn tarball}")"
E2E_IBUS="$(realpath "${3:?cần đường dẫn e2e_ibus}")"
E2E_FCITX5=""
if [[ -n "${4:-}" && -x "${4:-}" ]]; then
    E2E_FCITX5="$(realpath "$4")"
fi
[[ -x "$E2E_IBUS" ]] || { echo "FAIL: thiếu $E2E_IBUS" >&2; exit 1; }

fail() { echo "FAIL ($DISTRO): $*" >&2; exit 1; }
export -f fail
export DISTRO TARBALL E2E_IBUS E2E_FCITX5

new_test_home() {
    local t
    t="$(mktemp -d)"
    mkdir -p "$t/home" "$t/run" "$t/fakebin"
    chmod 0700 "$t/run"
    cat > "$t/fakebin/gsettings" <<'G'
#!/usr/bin/env bash
store_gnome="$HOME/.fake-gsettings-gnome"
store_ibus="$HOME/.fake-gsettings-ibus"
case "$1" in
    list-schemas)
        echo org.gnome.desktop.input-sources
        echo org.freedesktop.ibus.general
        ;;
    get)
        if [[ "$2" == org.gnome.desktop.input-sources ]]; then
            cat "$store_gnome" 2>/dev/null || echo "[('xkb', 'us')]"
        else
            cat "$store_ibus" 2>/dev/null || echo "['xkb:us::eng']"
        fi
        ;;
    set)
        if [[ "$2" == org.gnome.desktop.input-sources ]]; then
            echo "$4" > "$store_gnome"
        else
            echo "$4" > "$store_ibus"
        fi
        ;;
esac
G
    chmod +x "$t/fakebin/gsettings"
    echo "$t"
}
export -f new_test_home

OS_PRETTY="$(. /etc/os-release 2>/dev/null && echo "${PRETTY_NAME:-$DISTRO}" || echo "$DISTRO")"
echo "=== E68 Distro Matrix E2E: $DISTRO ($OS_PRETTY; $(ldd --version | sed -n '1p'); bash $BASH_VERSION) ==="

if [[ "$SYSTEM_ONLY" == 0 ]]; then
    # 1. ABI + GTK4/GTK3 auto-select + textvn doctor (text & JSON) + Fcitx5 dlopen
    t1="$(mktemp -d)"
    tar -C "$t1" -xzf "$TARBALL"
    pkg1="$(echo "$t1"/TextVN-*)"
    "$pkg1/bin/textvn" --help >/dev/null || fail "bin/textvn --help lỗi"
    doc_txt="$("$pkg1/bin/textvn" doctor)" || fail "textvn doctor lỗi"
    grep -q "TextVN doctor (Linux)" <<<"$doc_txt" || fail "textvn doctor thiếu header Linux: $doc_txt"
    doc_json="$("$pkg1/bin/textvn" doctor --json)" || fail "textvn doctor --json lỗi"
    grep -q '"glibc"' <<<"$doc_json" || fail "textvn doctor --json thiếu trường Linux: $doc_json"

    "$pkg1/lib/textvn/textvn-ibus-engine" --check-abi || fail "textvn-ibus-engine --check-abi lỗi"
    "$pkg1/bin/textvn-settings" --check-abi || fail "textvn-settings --check-abi lỗi"
    set_ver="$("$pkg1/bin/textvn-settings" --version)" || fail "textvn-settings --version lỗi"
    echo "  settings backend: $set_ver"
    case "$DISTRO" in
        ubuntu-22-04|debian-12|fedora-41)
            grep -q "GTK4" <<<"$set_ver" || fail "mong đợi GTK4 native trên $DISTRO nhưng ra: $set_ver"
            ;;
        almalinux-9|opensuse-leap-15-6)
            grep -qE "GTK(3|4)" <<<"$set_ver" || fail "settings không nhận diện GTK3/GTK4: $set_ver"
            ;;
    esac
    if command -v fcitx5 >/dev/null 2>&1; then
        "$pkg1/bin/textvn" check-dlopen "$pkg1/lib/textvn/fcitx5/libtextvn-fcitx5.so" ||
            fail "libtextvn-fcitx5.so không dlopen được trên $DISTRO!"
        echo "  fcitx5 dlopen   : OK"
    fi
    rm -rf "$t1"
    echo "PASS [$DISTRO] 1: ABI + GTK auto-select + textvn doctor (text/json) + Fcitx5 dlopen"

    # 2. Cài per-user: kiểm tra non-GNOME gsettings (Cinnamon/Xfce), Exec= tuyệt đối trong .desktop,
    #    ~/.xprofile cho X11, gõ tiếng Việt thật qua IBus, và gỡ sạch (kể cả ~/.xprofile).
    distro_ibus_install() {
        local t="" daemon_pid=""
        cleanup() {
            ibus exit >/dev/null 2>&1 || true
            [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
            [[ -n "$t" && -d "$t" ]] && rm -rf "$t" 2>/dev/null || true
        }
        trap cleanup EXIT
        t="$(new_test_home)"
        export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
        export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
        export XDG_CURRENT_DESKTOP="X-Cinnamon"
        unset IBUS_COMPONENT_PATH FCITX_ADDON_DIRS
        mkdir -p "$t/x"
        tar -C "$t/x" -xzf "$TARBALL"
        local pkg pre="$HOME/.local"
        pkg="$(echo "$t"/x/TextVN-*)"

        "$pkg/install.sh" --no-restart >"$t/in.log" 2>&1 || { cat "$t/in.log"; fail "install.sh"; }
        # Kiểm non-GNOME gsettings (E68 #6): KHÔNG chạm GNOME input-sources, phải ghi ibus preload-engines!
        [[ ! -e "$HOME/.fake-gsettings-gnome" ]] || fail "Cinnamon bị ghi nhầm vào org.gnome.desktop.input-sources"
        grep -q "'textvn'" "$HOME/.fake-gsettings-ibus" || fail "không thêm textvn vào org.freedesktop.ibus.general"
        # Kiểm .desktop Exec= tuyệt đối (E68 #7)
        grep -Fq "Exec=$pre/bin/textvn-settings" "$pre/share/applications/textvn-settings.desktop" ||
            fail ".desktop thiếu Exec tuyệt đối: $(cat "$pre/share/applications/textvn-settings.desktop")"
        # Kiểm ~/.xprofile cho X11/LightDM (E68 #5)
        grep -Fq "# BEGIN TextVN" "$HOME/.xprofile" || fail "install.sh không tạo khối TextVN trong ~/.xprofile"

        set -a
        # shellcheck disable=SC1090
        . "$XDG_CONFIG_HOME/environment.d/60-textvn.conf"
        set +a
        local rc=0
        ibus-daemon --panel=disable --xim=false --config=default --cache=refresh --replace --single >"$t/d.log" 2>&1 &
        daemon_pid=$!
        sleep 2
        "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
        if [[ $rc -ne 0 ]]; then
            echo "--- ibus list-engine ($DISTRO debug) ---"
            ibus list-engine 2>&1 | grep -i -C 1 textvn || true
            echo "--- IBUS_COMPONENT_PATH=$IBUS_COMPONENT_PATH ---"
        fi
        ibus exit >/dev/null 2>&1 || true
        [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
        daemon_pid=""
        sleep 1
        [[ $rc -eq 0 ]] || { cat "$t/in.log" "$t/e2e.log" "$t/d.log"; fail "IBus install e2e_ibus"; }
        tail -n1 "$t/e2e.log"

        "$pre/share/textvn/uninstall.sh" --no-restart >"$t/un.log" 2>&1 || { cat "$t/un.log"; fail "uninstall.sh"; }
        ! grep -q "'textvn'" "$HOME/.fake-gsettings-ibus" || fail "uninstall.sh không gỡ textvn khỏi org.freedesktop.ibus.general"
        [[ ! -e "$HOME/.xprofile" ]] || fail "uninstall.sh để sót ~/.xprofile"
        local left
        left="$(find "$HOME" \( -path "$HOME/.config/TextVN" -o -path "$HOME/.local/state/TextVN" \) -prune \
            -o \( -iname '*textvn*' -print \) | grep -v fake-gsettings || true)"
        [[ -z "$left" ]] || fail "còn sót file sau gỡ: $left"
        rm -rf "$t"
        trap - EXIT
        echo "PASS [$DISTRO] 2: cài per-user (Cinnamon/X11 .xprofile + .desktop Exec) → gõ IBus → gỡ sạch"
    }
    export -f distro_ibus_install
    dbus-run-session -- bash -c distro_ibus_install

    # 3. Portable (thư mục chỉ đọc có khoảng trắng + dấu) trên cả X11 & GNOME Wayland live-register.
    distro_ibus_portable() {
        local t="" pkg="" daemon_pid=""
        cleanup() {
            ibus exit >/dev/null 2>&1 || true
            [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
            [[ -n "$pkg" && -d "$pkg" ]] && chmod -R u+w "$pkg" 2>/dev/null || true
            [[ -n "$t" && -d "$t" ]] && rm -rf "$t" 2>/dev/null || true
        }
        trap cleanup EXIT
        t="$(new_test_home)"
        export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
        export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
        export XDG_SESSION_TYPE=wayland XDG_CURRENT_DESKTOP=GNOME TV_SKIP_MANAGER_ENV=1
        unset IBUS_COMPONENT_PATH
        local x="$t/Tải về"
        mkdir -p "$x"
        tar -C "$x" -xzf "$TARBALL"
        pkg="$(echo "$x"/TextVN-*)"
        chmod -R a-w "$pkg"

        ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
        daemon_pid=$!
        sleep 2
        "$pkg/textvn-portable.sh" --ibus >"$t/p.log" 2>&1 || { cat "$t/p.log"; fail "portable Wayland start"; }
        kill -0 "$daemon_pid" 2>/dev/null || fail "portable Wayland đã giết ibus-daemon gốc"
        local rc=0
        "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
        [[ $rc -eq 0 ]] || { cat "$t/p.log" "$t/e2e.log" "$t/d.log"; fail "portable Wayland e2e_ibus"; }
        tail -n1 "$t/e2e.log"
        "$pkg/textvn-portable.sh" stop >"$t/s.log" 2>&1 || { cat "$t/s.log"; fail "portable stop"; }
        kill -0 "$daemon_pid" 2>/dev/null || fail "portable stop đã giết ibus-daemon gốc"
        ! pgrep -u "$(id -u)" '^textvn-ibus-eng' >/dev/null 2>&1 || fail "portable stop để sót engine"
        chmod -R u+w "$pkg"
        cleanup
        trap - EXIT
        echo "PASS [$DISTRO] 3: portable (chỉ đọc + GNOME Wayland live-register) → gõ IBus → stop sạch"
    }
    export -f distro_ibus_portable
    dbus-run-session -- bash -c distro_ibus_portable

    # 4. Fcitx5 E2E thật (khi distro có fcitx5 + e2e_fcitx5: Ubuntu 22.04, Debian 12, Fedora 41)
    #    + kiểm thử kịch bản KDE Fcitx5-only (không có libibus trong gói).
    if command -v fcitx5 >/dev/null 2>&1 && [[ -n "$E2E_FCITX5" ]]; then
        distro_fcitx5_e2e() {
            local t="" pkg=""
            cleanup() {
                fcitx5-remote -e >/dev/null 2>&1 || true
                [[ -n "$pkg" && -d "$pkg" ]] && chmod -R u+w "$pkg" 2>/dev/null || true
                [[ -n "$t" && -d "$t" ]] && rm -rf "$t" 2>/dev/null || true
            }
            trap cleanup EXIT
            t="$(new_test_home)"
            export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
            export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
            unset IBUS_COMPONENT_PATH FCITX_ADDON_DIRS
            mkdir -p "$t/x" "$XDG_CONFIG_HOME/fcitx5"
            # E68: Fcitx5 5.0.x (Ubuntu 22.04 / Debian 12) mặc định gắn Ctrl+Shift vào
            # EnumerateForwardKeys (đổi IM của Fcitx5 trước khi phím tới TextVN).
            # Tắt phím đổi IM mặc định trong phiên test để e2e_fcitx5 kiểm đúng Ctrl+Shift của TextVN.
            cat >"$XDG_CONFIG_HOME/fcitx5/config" <<'C'
[Hotkey/EnumerateForwardKeys]

[Hotkey/EnumerateBackwardKeys]

[Hotkey/EnumerateGroupForwardKeys]

[Hotkey/EnumerateGroupBackwardKeys]
C
            cat >"$XDG_CONFIG_HOME/fcitx5/profile" <<'P'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=keyboard-us

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[GroupOrder]
0=Default
P
            tar -C "$t/x" -xzf "$TARBALL"
            pkg="$(echo "$t"/x/TextVN-*)"
            # Giả lập máy KDE Fcitx5-only không có libibus (xoá engine ibus khỏi bản copy):
            # tv_check_package và portable --fcitx5 PHẢI thành công!
            rm -f "$pkg/lib/textvn/textvn-ibus-engine"
            chmod -R a-w "$pkg"

            # Khởi chạy fcitx5 nền tối giản (giống test-linux-package.sh) để phím chuyển nhóm mặc định
            # của Fcitx5 không chặn tổ hợp Ctrl+Shift trong e2e_fcitx5.
            fcitx5 --disable=all --enable=dbus,dbusfrontend,keyboard,textvn >"$t/d.log" 2>&1 &
            sleep 2
            "$pkg/textvn-portable.sh" --fcitx5 >"$t/p.log" 2>&1 || { cat "$t/p.log" "$t/d.log"; fail "Fcitx5-only portable start"; }
            sleep 1
            local rc=0
            "$E2E_FCITX5" >"$t/e2e.log" 2>&1 || rc=$?
            [[ $rc -eq 0 ]] || { cat "$t/p.log" "$t/e2e.log" "$t/d.log"; fail "Fcitx5 e2e_fcitx5"; }
            tail -n1 "$t/e2e.log"
            "$pkg/textvn-portable.sh" stop >"$t/s.log" 2>&1 || { cat "$t/s.log"; fail "Fcitx5 portable stop"; }
            chmod -R u+w "$pkg"
            cleanup
            trap - EXIT
            echo "PASS [$DISTRO] 4: Fcitx5-only portable (không cần libibus) → gõ tiếng Việt thật → stop sạch"
        }
        export -f distro_fcitx5_e2e
        dbus-run-session -- bash -c distro_fcitx5_e2e
    fi

    echo "=== TOÀN BỘ KIỂM THỬ NGƯỜI DÙNG [$DISTRO] PASS ==="
fi

# 5. Cài hệ thống (--system với quyền root) → ibus-daemon gốc không cần env → gõ tiếng Việt → gỡ sạch.
if [[ "$SYSTEM_ONLY" == 1 || "$(id -u)" -eq 0 ]]; then
    [[ "$(id -u)" -eq 0 ]] || fail "--system-only cần quyền root"
    distro_system_install() {
        local t="" daemon_pid=""
        cleanup() {
            ibus exit >/dev/null 2>&1 || true
            [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
            if [[ -x /usr/share/textvn/uninstall.sh ]]; then
                /usr/share/textvn/uninstall.sh --no-restart >/dev/null 2>&1 || true
            fi
            [[ -n "$t" && -d "$t" ]] && rm -rf "$t" 2>/dev/null || true
        }
        trap cleanup EXIT
        t="$(new_test_home)"
        export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
        export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
        export IBUS_COMPONENT_PATH="/tmp/must-not-leak"
        tar -C "$t" -xzf "$TARBALL"
        local pkg
        pkg="$(echo "$t"/TextVN-*)"

        "$pkg/install.sh" --system --no-restart >"$t/in.log" 2>&1 || { cat "$t/in.log"; fail "install.sh --system"; }
        [[ -f /usr/share/ibus/component/textvn.xml ]] || fail "--system thiếu /usr/share/ibus/component/textvn.xml"
        [[ -x /usr/bin/textvn && -x /usr/bin/textvn-settings ]] || fail "--system thiếu binary /usr/bin"
        unset IBUS_COMPONENT_PATH

        local rc=0
        ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
        daemon_pid=$!
        sleep 2
        "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
        ibus exit >/dev/null 2>&1 || true
        [[ -n "$daemon_pid" ]] && kill "$daemon_pid" 2>/dev/null || true
        daemon_pid=""
        sleep 1
        [[ $rc -eq 0 ]] || { cat "$t/in.log" "$t/e2e.log" "$t/d.log"; fail "--system e2e_ibus"; }
        tail -n1 "$t/e2e.log"

        /usr/share/textvn/uninstall.sh --prefix /usr/ --no-restart >"$t/un.log" 2>&1 || { cat "$t/un.log"; fail "uninstall.sh --system"; }
        [[ ! -e /usr/share/ibus/component/textvn.xml && ! -e /usr/bin/textvn ]] || fail "--system uninstall để sót file"
        rm -rf "$t"
        trap - EXIT
        echo "PASS [$DISTRO] 5: cài hệ thống (--system) → gõ tiếng Việt qua IBus gốc → gỡ sạch"
    }
    export -f distro_system_install
    dbus-run-session -- bash -c distro_system_install
fi
