#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# test-linux-package.sh <tarball> — kiểm thử gói Linux theo HAI kịch bản người dùng, mỗi
# kịch bản trên IBus và Fcitx5 THẬT, trong HOME tạm (không đụng máy người chạy):
#
#   A. Cài đặt:   giải nén → install.sh (per-user) → đăng nhập lại (nạp environment.d)
#                 → gõ tiếng Việt → uninstall.sh → không còn file nào.
#   B. Chạy ngay: giải nén vào thư mục CHỈ ĐỌC → textvn-portable.sh → gõ tiếng Việt
#                 → textvn-portable.sh stop → không để lại gì trong HOME.
#
# Gõ bằng chính client e2e (tests/e2e_ibus, tests/e2e_fcitx5.py) — nên cần build adapter
# trước (scripts/e2e-linux.sh hoặc scripts/build-linux.sh).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARBALL="$(realpath "${1:?cần đường dẫn tarball}")"
BUILD="${2:-$ROOT/target/linux-adapters}"
E2E_IBUS="$BUILD/ibus/e2e_ibus"
[[ -x "$E2E_IBUS" ]] || { echo "thiếu $E2E_IBUS — chạy scripts/e2e-linux.sh trước" >&2; exit 1; }
PY=""
for cand in python3 /usr/bin/python3 /usr/bin/python3.*; do
    if command -v "$cand" >/dev/null 2>&1 &&
        "$cand" -c 'import dbus, gi' >/dev/null 2>&1; then
        PY="$(command -v "$cand")"
        break
    fi
done
[[ -n "$PY" ]] || { echo "cần python3 có dbus + gi cho e2e Fcitx5" >&2; exit 1; }
export ROOT TARBALL BUILD E2E_IBUS PY

# Môi trường HOME tạm như một người dùng mới; gsettings giả để kiểm tra bước "thêm bộ gõ".
new_home() {
    local t
    t="$(mktemp -d)"
    mkdir -p "$t/home" "$t/run" "$t/fakebin"
    chmod 0700 "$t/run"
    cat > "$t/fakebin/gsettings" <<'G'
#!/usr/bin/env bash
# gsettings giả: một schema IBus, lưu giá trị vào file.
store="$HOME/.fake-gsettings"
case "$1" in
    list-schemas) echo org.freedesktop.ibus.general ;;
    get) cat "$store" 2>/dev/null || echo "['xkb:us::eng']" ;;
    set) echo "$4" > "$store" ;;
esac
G
    chmod +x "$t/fakebin/gsettings"
    echo "$t"
}
export -f new_home

fcitx_profile() {
    mkdir -p "$XDG_CONFIG_HOME/fcitx5"
    cat > "$XDG_CONFIG_HOME/fcitx5/profile" <<'P'
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
}
export -f fcitx_profile

fail() { echo "FAIL: $*" >&2; exit 1; }
export -f fail

# ---- A. Cài đặt ----------------------------------------------------------------------
scenario_install() {
    local fw="$1" t
    t="$(new_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
    export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
    unset IBUS_COMPONENT_PATH FCITX_ADDON_DIRS
    mkdir -p "$t/x"
    tar -C "$t/x" -xzf "$TARBALL"
    local pkg
    pkg="$(echo "$t"/x/TextVN-*)"
    [[ "$fw" == fcitx5 ]] && fcitx_profile

    "$pkg/install.sh" --no-restart >"$t/install.log" 2>&1 || { cat "$t/install.log"; fail "install.sh"; }
    local pre="$HOME/.local"
    for f in bin/textvn bin/textvn-settings lib/textvn/textvn-ibus-engine \
             lib/textvn/fcitx5/libtextvn-fcitx5.so share/ibus/component/textvn.xml \
             share/fcitx5/addon/textvn.conf share/fcitx5/inputmethod/textvn.conf \
             share/applications/textvn-settings.desktop share/textvn/uninstall.sh \
             share/textvn/install-manifest.txt; do
        [[ -e "$pre/$f" ]] || fail "thiếu $pre/$f sau khi cài"
    done
    grep -q "<exec>$pre/lib/textvn/textvn-ibus-engine --ibus</exec>" "$pre/share/ibus/component/textvn.xml" ||
        fail "component IBus không trỏ vào engine đã cài"
    local envf="$XDG_CONFIG_HOME/environment.d/60-textvn.conf"
    [[ -f "$envf" ]] || fail "thiếu $envf"
    if [[ "$fw" == ibus ]]; then
        grep -q "'textvn'" "$HOME/.fake-gsettings" || fail "install.sh không thêm textvn vào IBus"
    else
        grep -qx 'Name=textvn' "$XDG_CONFIG_HOME/fcitx5/profile" || fail "install.sh không thêm textvn vào Fcitx5"
    fi

    # "Đăng nhập lại": phiên mới nạp environment.d.
    set -a
    # shellcheck disable=SC1090
    . "$envf"
    set +a
    local rc=0
    if [[ "$fw" == ibus ]]; then
        ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
        sleep 2
        "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
        pgrep -af textvn-ibus-engine | grep -q "$pre/lib/textvn/textvn-ibus-engine" ||
            { rc=1; echo "engine không chạy từ bản đã cài"; }
        pkill -x ibus-daemon || true
    else
        fcitx5 --disable=all --enable=dbus,dbusfrontend,keyboard,textvn >"$t/d.log" 2>&1 &
        "$PY" "$ROOT/adapters/linux-fcitx5/tests/e2e_fcitx5.py" >"$t/e2e.log" 2>&1 || rc=$?
        pkill -x fcitx5 || true
    fi
    sleep 1
    [[ $rc -eq 0 ]] || { cat "$t/e2e.log" "$t/d.log"; fail "gõ sau khi cài ($fw)"; }
    tail -n1 "$t/e2e.log"

    "$pre/share/textvn/uninstall.sh" --no-restart >"$t/un.log" 2>&1 || { cat "$t/un.log"; fail "uninstall.sh"; }
    local left
    # Dữ liệu người dùng (cấu hình, nhật ký) được giữ lại khi gỡ — chỉ file cài đặt phải hết.
    left="$(find "$HOME" \( -path "$HOME/.config/TextVN" -o -path "$HOME/.local/state/TextVN" \) -prune \
        -o \( -iname '*textvn*' -print \) | grep -v fake-gsettings || true)"
    [[ -z "$left" ]] || fail "còn sót sau khi gỡ: $left"
    if grep -q "'textvn'" "$HOME/.fake-gsettings" 2>/dev/null; then
        fail "gỡ xong vẫn còn textvn trong danh sách engine IBus"
    fi
    if [[ "$fw" == fcitx5 ]]; then
        grep -qx 'Name=textvn' "$XDG_CONFIG_HOME/fcitx5/profile" && fail "gỡ xong vẫn còn textvn trong profile Fcitx5"
    fi
    echo "PASS kịch bản CÀI ĐẶT ($fw): cài → gõ → gỡ sạch"
}
export -f scenario_install

# ---- B. Chạy ngay từ thư mục giải nén -----------------------------------------------
scenario_portable() {
    local fw="$1" t
    t="$(new_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config" XDG_DATA_HOME="$t/home/.local/share"
    export XDG_CACHE_HOME="$t/home/.cache" XDG_RUNTIME_DIR="$t/run" PATH="$t/fakebin:$PATH"
    unset IBUS_COMPONENT_PATH FCITX_ADDON_DIRS
    mkdir -p "$t/x"
    tar -C "$t/x" -xzf "$TARBALL"
    local pkg
    pkg="$(echo "$t"/x/TextVN-*)"
    chmod -R a-w "$pkg"   # thư mục giải nén chỉ đọc: portable không được ghi vào đó

    local rc=0
    if [[ "$fw" == ibus ]]; then
        # Phiên đang chạy IBus bình thường (chưa biết TextVN).
        ibus-daemon --panel=disable --xim=false --config=default --replace --single >"$t/d.log" 2>&1 &
        sleep 2
        "$pkg/textvn-portable.sh" --ibus >"$t/p.log" 2>&1 || { cat "$t/p.log"; fail "textvn-portable.sh --ibus"; }
        sleep 1
        "$E2E_IBUS" >"$t/e2e.log" 2>&1 || rc=$?
        pgrep -af textvn-ibus-engine | grep -q "$pkg/lib/textvn/textvn-ibus-engine" ||
            { rc=1; echo "engine không chạy từ thư mục giải nén"; }
    else
        fcitx_profile
        fcitx5 --disable=all --enable=dbus,dbusfrontend,keyboard,textvn >"$t/d.log" 2>&1 &
        sleep 2
        "$pkg/textvn-portable.sh" --fcitx5 >"$t/p.log" 2>&1 || { cat "$t/p.log"; fail "textvn-portable.sh --fcitx5"; }
        grep -qx 'Name=textvn' "$XDG_CONFIG_HOME/fcitx5/profile" || fail "portable không thêm textvn vào nhóm Fcitx5"
        "$PY" "$ROOT/adapters/linux-fcitx5/tests/e2e_fcitx5.py" >"$t/e2e.log" 2>&1 || rc=$?
    fi
    [[ $rc -eq 0 ]] || { cat "$t/p.log" "$t/e2e.log" "$t/d.log"; fail "gõ ở chế độ portable ($fw)"; }
    tail -n1 "$t/e2e.log"
    "$pkg/textvn-portable.sh" status | grep -q "đang bật" || fail "status không báo đang bật"

    "$pkg/textvn-portable.sh" stop >"$t/s.log" 2>&1 || { cat "$t/s.log"; fail "textvn-portable.sh stop"; }
    [[ ! -e "$XDG_RUNTIME_DIR/textvn-portable-$(id -u)" ]] || fail "stop không dọn runtime dir"
    if [[ "$fw" == fcitx5 ]]; then
        grep -qx 'Name=textvn' "$XDG_CONFIG_HOME/fcitx5/profile" && fail "stop vẫn để textvn trong profile"
    fi
    local stray
    stray="$(find "$HOME/.local" -path "$HOME/.local/state/TextVN" -prune -o -iname '*textvn*' -print 2>/dev/null || true)"
    [[ -z "$stray" ]] || fail "portable đã ghi file cài đặt vào ~/.local: $stray"
    pkill -x ibus-daemon 2>/dev/null || true
    pkill -x fcitx5 2>/dev/null || true
    sleep 1
    chmod -R u+w "$pkg"
    echo "PASS kịch bản CHẠY NGAY ($fw): giải nén (chỉ đọc) → portable → gõ → stop sạch"
}
export -f scenario_portable

# ---- Kiểm thử đơn vị: sửa danh sách bộ gõ (GNOME input-sources, profile Fcitx5) ----------
unit_activation() {
    local t
    t="$(new_home)"
    export HOME="$t/home" XDG_CONFIG_HOME="$t/home/.config"
    cat > "$t/fakebin/gsettings" <<'G'
#!/usr/bin/env bash
store="$HOME/.fake-gsettings"
case "$1" in
    list-schemas) echo org.gnome.desktop.input-sources ;;
    get) cat "$store" ;;
    set) echo "$4" > "$store" ;;
esac
G
    export PATH="$t/fakebin:$PATH"
    # shellcheck source=packaging/linux/textvn-common.sh
    . "$ROOT/packaging/linux/textvn-common.sh"
    local store="$HOME/.fake-gsettings" in out
    for in in "[('xkb', 'us')]" "@a(ss) []" "[('xkb', 'us'), ('xkb', 'fr')]"; do
        echo "$in" > "$store"
        tv_activate_ibus >/dev/null
        tv_activate_ibus >/dev/null   # lần hai không thêm trùng
        out="$(cat "$store")"
        [[ "$(grep -o "'textvn'" <<<"$out" | wc -l)" == 1 ]] || fail "GNOME thêm sai: $in → $out"
        tv_deactivate_ibus
        out="$(cat "$store")"
        [[ "$out" == "$in" ]] || fail "GNOME gỡ sai: $in → $out"
    done
    mkdir -p "$XDG_CONFIG_HOME/fcitx5"
    printf '[Groups/0]\nName=Default\n\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n\n[Groups/0/Items/1]\nName=unikey\nLayout=\n\n[GroupOrder]\n0=Default\n' \
        > "$XDG_CONFIG_HOME/fcitx5/profile"
    cp "$XDG_CONFIG_HOME/fcitx5/profile" "$t/orig"
    tv_activate_fcitx5 >/dev/null
    tv_activate_fcitx5 >/dev/null
    grep -q '^\[Groups/0/Items/2\]$' "$XDG_CONFIG_HOME/fcitx5/profile" || fail "Fcitx5: chỉ số item mới sai"
    [[ "$(grep -cx 'Name=textvn' "$XDG_CONFIG_HOME/fcitx5/profile")" == 1 ]] || fail "Fcitx5 thêm trùng"
    tv_deactivate_fcitx5
    diff <(grep -v '^$' "$t/orig") <(grep -v '^$' "$XDG_CONFIG_HOME/fcitx5/profile") >/dev/null ||
        fail "Fcitx5 gỡ không trả profile về như cũ"
    echo "PASS đơn vị: thêm/bỏ TextVN trong GNOME input-sources và profile Fcitx5"
}
export -f unit_activation
bash -c unit_activation

for fw in ibus fcitx5; do
    dbus-run-session -- bash -c "scenario_install $fw" 2> >(grep -v 'fd limit' >&2)
    dbus-run-session -- bash -c "scenario_portable $fw" 2> >(grep -v 'fd limit' >&2)
done
echo "== Gói Linux: 4/4 kịch bản PASS (cài + chạy ngay × IBus + Fcitx5) + kiểm thử đơn vị kích hoạt"
