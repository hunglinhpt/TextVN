#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# e2e-desktop.sh <de> <tarball> — COMPAT-3: gõ tiếng Việt THẬT trong một phiên desktop.
#
# Khác e2e_ibus/e2e_fcitx5 (điều khiển input context qua D-Bus, không có app GUI): ở đây
# phím đi đúng đường người dùng — X server/compositor → toolkit GTK của app → IM module →
# IBus/Fcitx5 → TextVN → chữ cam kết vào ô soạn thảo của một app thật, rồi đọc lại.
#
#   de = xfce-x11   Xvfb + xfwm4 (WM của Xfce) + IBus + Mousepad (GTK3, editor của Xfce)
#
# Gói được cài bằng CHÍNH install.sh của tarball (per-user, HOME cách ly) — kiểm luôn đường cài.
# Bộ case: tools/e2e/typing-cases.txt (MỘT nguồn với harness macOS smoke-imk.sh, G22).
# Cần (Ubuntu/Debian): xvfb xfwm4 mousepad xdotool xclip ibus ibus-gtk3 dbus-x11.
# G8: chỉ in các case cố định của corpus, không đọc gì ngoài tài liệu test riêng.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DE="${1:?cần môi trường: xfce-x11}"
TARBALL="$(realpath "${2:?cần đường dẫn tarball TextVN-*-linux-*.tar.gz}")"
CASES_FILE="$ROOT/tools/e2e/typing-cases.txt"

say() { echo "[e2e-desktop:$DE] $*"; }
fail() { echo "FAIL [e2e-desktop:$DE] $*" >&2; exit 1; }

for tool in xdotool xclip dbus-run-session; do
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
# Dùng chung cho mọi DE X11 (hàm được export vào phiên dbus-run-session).
type_and_read() {
    local win="$1" keys="$2"
    xdotool windowactivate --sync "$win" >/dev/null 2>&1 || true
    xdotool key --clearmodifiers ctrl+a Delete
    sleep 0.3
    xdotool type --delay 90 -- "$keys"
    # Dấu cách cam kết từ đang soạn (preedit không có trong buffer của app).
    xdotool key space
    sleep 0.6
    xdotool key --clearmodifiers ctrl+a ctrl+c
    sleep 0.4
    xclip -selection clipboard -o 2>/dev/null | sed 's/[[:space:]]*$//'
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
        for tool in Xvfb xfwm4 mousepad ibus-daemon; do
            command -v "$tool" >/dev/null 2>&1 || fail "thiếu $tool"
        done
        export DISPLAY=:99
        Xvfb :99 -screen 0 1280x800x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
        XVFB_PID=$!
        for _ in $(seq 1 50); do xdotool getdisplaygeometry >/dev/null 2>&1 && break; sleep 0.1; done
        xdotool getdisplaygeometry >/dev/null 2>&1 || { cat "$T/xvfb.log"; fail "Xvfb không lên"; }

        xfce_session() {
            set -euo pipefail
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
            ibus engine textvn
            echo "[e2e-desktop:$DE] engine hiện hành: $(ibus engine)"
            : > "$T/doc.txt"
            mousepad "$T/doc.txt" >"$T/editor.log" 2>&1 &
            local win
            win="$(timeout 30 xdotool search --sync --onlyvisible --class mousepad | head -n1 || true)"
            [[ -n "$win" ]] || { cat "$T/editor.log"; echo "FAIL không thấy cửa sổ Mousepad" >&2; return 1; }
            sleep 1
            ibus engine textvn
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
    *)
        fail "môi trường lạ: $DE (hỗ trợ: xfce-x11)"
        ;;
esac
