#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# e2e-desktop.sh <de> <tarball> — COMPAT-3: gõ tiếng Việt THẬT trong một phiên desktop.
#
# Khác e2e_ibus/e2e_fcitx5 (điều khiển input context qua D-Bus, không có app GUI): ở đây
# phím đi đúng đường người dùng — X server/compositor → toolkit GTK của app → IM module →
# IBus/Fcitx5 → TextVN → chữ cam kết vào ô soạn thảo của một app thật, rồi đọc lại.
#
#   de = xfce-x11      Xvfb + xfwm4 (WM của Xfce) + IBus + Mousepad (GTK3, editor của Xfce)
#   de = sway-wayland  Sway headless (wlroots) + Fcitx5 qua input-method-v2 + Mousepad Wayland gốc
#
# Gói được cài bằng CHÍNH install.sh của tarball (per-user, HOME cách ly) — kiểm luôn đường cài.
# Bộ case: tools/e2e/typing-cases.txt (MỘT nguồn với harness macOS smoke-imk.sh, G22).
# Cần (Ubuntu/Debian): xvfb xfwm4 mousepad xdotool xclip ibus ibus-gtk3 dbus-x11.
# G8: chỉ in các case cố định của corpus, không đọc gì ngoài tài liệu test riêng.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DE="${1:?cần môi trường: xfce-x11 | sway-wayland}"
TARBALL="$(realpath "${2:?cần đường dẫn tarball TextVN-*-linux-*.tar.gz}")"
CASES_FILE="$ROOT/tools/e2e/typing-cases.txt"

say() { echo "[e2e-desktop:$DE] $*"; }
fail() { echo "FAIL [e2e-desktop:$DE] $*" >&2; exit 1; }

for tool in dbus-run-session; do
    command -v "$tool" >/dev/null 2>&1 || fail "thiếu $tool"
done

T="$(mktemp -d)"
export T TARBALL CASES_FILE DE
export HOME="$T/home" XDG_CONFIG_HOME="$T/home/.config" XDG_DATA_HOME="$T/home/.local/share"
export XDG_CACHE_HOME="$T/home/.cache" XDG_RUNTIME_DIR="$T/run"
mkdir -p "$HOME" "$XDG_RUNTIME_DIR"
chmod 0700 "$XDG_RUNTIME_DIR"

XVFB_PID=""
cleanup() {
    [[ -n "$XVFB_PID" ]] && kill "$XVFB_PID" 2>/dev/null || true
    rm -rf "$T" 2>/dev/null || true
}
trap cleanup EXIT

# Gõ một chuỗi phím vào cửa sổ đang focus rồi đọc lại toàn bộ văn bản qua clipboard.
# X11: xdotool (XTest) + xclip. Wayland: wtype (virtual-keyboard-v1) + wl-paste.
# Dấu cách cuối cam kết từ đang soạn (preedit không có trong buffer của app).
type_and_read() {
    local win="$1" keys="$2"
    if [[ -n "${WAYLAND_DISPLAY:-}" && -z "${DISPLAY:-}" ]]; then
        wtype -M ctrl a -m ctrl -k Delete
        sleep 0.3
        wtype -d 90 -- "$keys"
        wtype -k space
        sleep 0.6
        wtype -M ctrl a -m ctrl -M ctrl c -m ctrl
        sleep 0.4
        wl-paste -n 2>/dev/null | sed 's/[[:space:]]*$//'
    else
        xdotool windowactivate --sync "$win" >/dev/null 2>&1 || true
        xdotool key --clearmodifiers ctrl+a Delete
        sleep 0.3
        xdotool type --delay 90 -- "$keys"
        xdotool key space
        sleep 0.6
        xdotool key --clearmodifiers ctrl+a ctrl+c
        sleep 0.4
        xclip -selection clipboard -o 2>/dev/null | sed 's/[[:space:]]*$//'
    fi
}
export -f type_and_read

run_cases() {
    local win="$1" pass=0 failn=0 keys expect got
    while IFS='=' read -r keys expect; do
        [[ -n "$keys" && "$keys" != \#* ]] || continue
        got="$(type_and_read "$win" "$keys")"
        if [[ "$got" == "$expect" ]]; then
            pass=$((pass + 1))
            echo "PASS  $keys → $got"
        else
            failn=$((failn + 1))
            echo "FAIL  $keys → '$got' (mong đợi '$expect')"
        fi
    done < "$CASES_FILE"
    echo "--- $pass pass · $failn fail ---"
    [[ "$pass" -gt 0 && "$failn" -eq 0 ]]
}
export -f run_cases

case "$DE" in
    xfce-x11)
        for tool in Xvfb xfwm4 mousepad ibus-daemon xdotool xclip; do
            command -v "$tool" >/dev/null 2>&1 || fail "thiếu $tool"
        done
        export DISPLAY=:99
        Xvfb :99 -screen 0 1280x800x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
        XVFB_PID=$!
        for _ in $(seq 1 50); do xdotool getdisplaygeometry >/dev/null 2>&1 && break; sleep 0.1; done
        xdotool getdisplaygeometry >/dev/null 2>&1 || { cat "$T/xvfb.log"; fail "Xvfb không lên"; }

        xfce_session() {
            set -euo pipefail
            trap 'echo "FAIL [e2e-desktop:$DE] dòng $LINENO: $BASH_COMMAND" >&2; for f in install ibus fcitx5 sway editor wm; do [[ -s "$T/$f.log" ]] && { echo "--- $f.log"; tail -n 25 "$T/$f.log"; }; done; command -v ibus >/dev/null && { echo "--- ibus list-engine:"; ibus list-engine 2>&1 | grep -i -C1 textvn; echo "IBUS_COMPONENT_PATH=${IBUS_COMPONENT_PATH:-}"; }' ERR
            xfwm4 --compositor=off >"$T/wm.log" 2>&1 &
            sleep 1
            mkdir -p "$T/x"
            tar -C "$T/x" -xzf "$TARBALL"
            local pkg
            pkg="$(echo "$T"/x/TextVN-*)"
            "$pkg/install.sh" --no-restart >"$T/install.log" 2>&1 || { cat "$T/install.log"; echo "FAIL install.sh" >&2; return 1; }
            set -a
            # shellcheck disable=SC1091
            . "$XDG_CONFIG_HOME/environment.d/60-textvn.conf"
            set +a
            # Như im-config/phiên Xfce chọn IBus.
            export GTK_IM_MODULE=ibus XMODIFIERS=@im=ibus QT_IM_MODULE=ibus
            ibus-daemon --xim --panel=disable --config=default --cache=refresh --replace --single >"$T/ibus.log" 2>&1 &
            for _ in $(seq 1 50); do ibus list-engine >/dev/null 2>&1 && break; sleep 0.2; done
            ibus list-engine 2>/dev/null | grep -qw textvn || {
                cat "$T/ibus.log"; echo "IBUS_COMPONENT_PATH=$IBUS_COMPONENT_PATH"; echo "FAIL engine textvn không có trong ibus list-engine" >&2; return 1; }
            : > "$T/doc.txt"
            mousepad "$T/doc.txt" >"$T/editor.log" 2>&1 &
            local win
            win="$(timeout 30 xdotool search --sync --onlyvisible --class mousepad | head -n1 || true)"
            [[ -n "$win" ]] || { cat "$T/editor.log"; echo "FAIL không thấy cửa sổ Mousepad" >&2; return 1; }
            sleep 1
            # Đặt engine SAU khi Mousepad có input context đang focus (IBus từ chối đặt engine toàn cục
            # khi chưa có IC nào — run 46122048149). Thử lại vài lần, in chẩn đoán nếu vẫn hỏng.
            xdotool windowactivate --sync "$win" >/dev/null 2>&1 || true
            local ok=0
            for _ in $(seq 1 10); do
                if ibus engine textvn >"$T/engine.err" 2>&1; then ok=1; break; fi
                sleep 0.5
            done
            if [[ $ok != 1 ]]; then
                # Chẩn đoán (run 46123336889: từ chối, stderr rỗng) — rồi VẪN gõ để thấy chữ đi đâu.
                echo "--- ibus engine textvn: $(cat "$T/engine.err")"
                echo "--- ibus engine (hiện hành): $(ibus engine 2>&1)"
                echo "--- preload-engines: $(gsettings get org.freedesktop.ibus.general preload-engines 2>&1)"
                local mp; mp="$(pgrep -n mousepad || true)"
                echo "--- mousepad pid=$mp nạp im-ibus: $(grep -c im-ibus "/proc/$mp/maps" 2>/dev/null || echo 0) · GTK_IM_MODULE=$GTK_IM_MODULE"
                echo "--- immodules: $(ls /usr/lib/x86_64-linux-gnu/gtk-3.0/3.0.0/immodules 2>&1 | tr '\n' ' ')"
            fi
            echo "[e2e-desktop:$DE] engine hiện hành: $(ibus engine)"
            local rc=0
            run_cases "$win" || rc=$?
            if [[ $rc -ne 0 ]]; then
                echo "--- ibus engine: $(ibus engine 2>&1) ---"
                tail -n 20 "$T/ibus.log" || true
            fi
            ibus exit >/dev/null 2>&1 || true
            return $rc
        }
        export -f xfce_session
        dbus-run-session -- bash -c xfce_session || fail "gõ thật trong Xfce/X11 không đạt"
        say "✅ PASS (Xvfb + xfwm4 + IBus + Mousepad)"
        ;;
    sway-wayland)
        for tool in sway wtype wl-paste fcitx5 fcitx5-remote mousepad; do
            command -v "$tool" >/dev/null 2>&1 || fail "thiếu $tool"
        done
        # Compositor wlroots headless (không GPU, không thiết bị vào) — chuẩn bị WL-1.
        export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman
        unset DISPLAY
        printf 'output HEADLESS-1 resolution 1280x800\n' > "$T/sway.conf"

        sway_session() {
            set -euo pipefail
            trap 'echo "FAIL [e2e-desktop:$DE] dòng $LINENO: $BASH_COMMAND" >&2; for f in install ibus fcitx5 sway editor wm; do [[ -s "$T/$f.log" ]] && { echo "--- $f.log"; tail -n 25 "$T/$f.log"; }; done; command -v ibus >/dev/null && { echo "--- ibus list-engine:"; ibus list-engine 2>&1 | grep -i -C1 textvn; echo "IBUS_COMPONENT_PATH=${IBUS_COMPONENT_PATH:-}"; }' ERR
            . "$ROOT/scripts/lib/fcitx5-test-env.sh"
            sway -c "$T/sway.conf" >"$T/sway.log" 2>&1 &
            local sock=""
            for _ in $(seq 1 50); do
                sock="$(cd "$XDG_RUNTIME_DIR" && ls wayland-* 2>/dev/null | grep -v '\.lock$' | head -n1 || true)"
                [[ -n "$sock" ]] && break
                sleep 0.2
            done
            [[ -n "$sock" ]] || { cat "$T/sway.log"; echo "FAIL sway headless không tạo socket Wayland" >&2; return 1; }
            export WAYLAND_DISPLAY="$sock"
            mkdir -p "$T/x"
            tar -C "$T/x" -xzf "$TARBALL"
            local pkg
            pkg="$(echo "$T"/x/TextVN-*)"
            "$pkg/install.sh" --no-restart >"$T/install.log" 2>&1 || { cat "$T/install.log"; echo "FAIL install.sh" >&2; return 1; }
            set -a
            # shellcheck disable=SC1091
            . "$XDG_CONFIG_HOME/environment.d/60-textvn.conf"
            set +a
            # Profile phiên test: nhóm Default có textvn; phím đổi IM mặc định tắt (E68/R2-175).
            textvn_fcitx5_test_hotkeys "$XDG_CONFIG_HOME/fcitx5"
            cat >"$XDG_CONFIG_HOME/fcitx5/profile" <<'P'
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
            # Wayland gốc: app dùng text-input-v3, Fcitx5 nối compositor qua input-method-v2.
            unset GTK_IM_MODULE QT_IM_MODULE XMODIFIERS
            export GDK_BACKEND=wayland
            # Run 46123336889: fcitx5-remote kích hoạt qua D-Bus một Fcitx5 THỨ HAI không có
            # WAYLAND_DISPLAY ("Failed to open wayland connection") tranh tên với bản của ta ⇒ đưa
            # môi trường phiên vào môi trường kích hoạt D-Bus, và chờ tên bus mà KHÔNG kích hoạt.
            dbus-update-activation-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR FCITX_ADDON_DIRS \
                XDG_CONFIG_HOME XDG_DATA_HOME HOME GDK_BACKEND
            fcitx5 --replace >"$T/fcitx5.log" 2>&1 &
            for _ in $(seq 1 50); do
                dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus \
                    org.freedesktop.DBus.NameHasOwner string:org.fcitx.Fcitx5 2>/dev/null |
                    grep -q 'boolean true' && break
                sleep 0.2
            done
            : > "$T/doc.txt"
            mousepad "$T/doc.txt" >"$T/editor.log" 2>&1 &
            sleep 3
            fcitx5-remote -s textvn || true
            echo "[e2e-desktop:$DE] fcitx5 IM hiện hành: $(fcitx5-remote -n 2>&1)"
            local rc=0
            run_cases "-" || rc=$?
            if [[ $rc -ne 0 ]]; then
                echo "--- fcitx5 IM: $(fcitx5-remote -n 2>&1) · sway:"; tail -n 15 "$T/sway.log" || true
                echo "--- sway app_ids: $(swaymsg -t get_tree 2>&1 | grep -o '"app_id": "[^"]*"' | tr '\n' ' ')"
                echo "--- sway focused: $(swaymsg -t get_tree 2>&1 | tr -d ' \n' | grep -o '"focused":true' | wc -l) nút"
                echo "--- wtype: $(wtype -k space 2>&1; echo rc=$?) · wl-paste: $(wl-paste -n 2>&1 | head -c 80; echo " rc=$?")"
                echo "--- fcitx5.log:"; tail -n 25 "$T/fcitx5.log" || true
            fi
            return $rc
        }
        export -f sway_session
        export ROOT
        dbus-run-session -- bash -c sway_session || fail "gõ thật trong Sway/Wayland không đạt"
        say "✅ PASS (Sway headless + Fcitx5 input-method-v2 + Mousepad Wayland)"
        ;;
    *)
        fail "môi trường lạ: $DE (hỗ trợ: xfce-x11, sway-wayland)"
        ;;
esac
