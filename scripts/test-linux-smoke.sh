#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# test-linux-smoke.sh <tarball>
# E70 — kiểm thử khói (smoke) gói Linux phát hành trên distro cũ/LTS ngoài ma trận E2E
# đầy đủ (Ubuntu 20.04 Focal glibc 2.31, Debian 11 Bullseye glibc 2.31…): chạy với user
# thường (không root), chứng minh trên MÁY THẬT của distro đó:
#   1. 3 ELF baseline EL8 (glibc ≤ 2.28) — bin/textvn, textvn-ibus-engine,
#      textvn-settings-gtk3 — không còn thư viện "not found" (ldd).
#   2. CLI chẩn đoán: --help, --check-env --json (JSON hợp lệ), --diagnose (Markdown),
#      --export-log <zip> (ZIP PK hợp lệ, không lộ tên người dùng / $HOME).
#   3. textvn-ibus-engine --check-abi, bin/textvn-settings --check-abi (tự chọn GTK3 khi
#      không có GTK4), plugin Fcitx5 chỉ báo trạng thái (không có Fcitx5 ≥ 5.0.14 là hợp lệ).
#   4. install.sh --no-install-deps --no-restart --no-activate per-user → uninstall.sh sạch.

set -euo pipefail

TARBALL="$(realpath "${1:?cần đường dẫn tarball}")"
fail() { echo "FAIL (smoke): $*" >&2; exit 1; }

OS_PRETTY="$(. /etc/os-release 2>/dev/null && echo "${PRETTY_NAME:-unknown}" || echo unknown)"
echo "=== E70 Linux smoke: $OS_PRETTY; $(ldd --version | sed -n '1p'); user=$(id -un) ==="

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
tar -C "$work" -xzf "$TARBALL"
pkg="$(echo "$work"/TextVN-*)"
[[ -d "$pkg" ]] || fail "tarball không có thư mục TextVN-*"

# 1. ldd — baseline EL8 phải chạy được trên mọi distro glibc ≥ 2.28.
for elf in bin/textvn lib/textvn/textvn-ibus-engine lib/textvn/textvn-settings-gtk3; do
    [[ -f "$pkg/$elf" ]] || fail "thiếu $elf trong tarball"
    missing="$(ldd "$pkg/$elf" 2>&1 | grep -F 'not found' || true)"
    [[ -z "$missing" ]] || fail "$elf thiếu thư viện:"$'\n'"$missing"
    echo "  ldd OK: $elf"
done

# 2. CLI chẩn đoán (exit 0 = sạch, 1 = có phát hiện; 2+ = lỗi dùng/xuất → FAIL).
"$pkg/bin/textvn" --help >/dev/null || fail "bin/textvn --help"
rc=0; json="$("$pkg/bin/textvn" --check-env --json 2>/dev/null)" || rc=$?
(( rc <= 1 )) || fail "--check-env --json exit $rc"
grep -q '"diagnostics"' <<<"$json" || fail "--check-env --json thiếu diagnostics: $json"
grep -q '"glibc"' <<<"$json" || fail "--check-env --json thiếu glibc: $json"
rc=0; md="$("$pkg/bin/textvn" --diagnose 2>/dev/null)" || rc=$?
(( rc <= 1 )) || fail "--diagnose exit $rc"
grep -q "Báo cáo chẩn đoán TextVN" <<<"$md" || fail "--diagnose thiếu tiêu đề Markdown"
zip="$work/out/diag.zip"
"$pkg/bin/textvn" --export-log "$zip" >/dev/null || fail "--export-log exit $?"
[[ -s "$zip" ]] || fail "--export-log không tạo ZIP"
[[ "$(head -c 2 "$zip")" == PK ]] || fail "ZIP chẩn đoán sai định dạng"
for needle in "$HOME" "$(id -un)"; do
    if [[ ${#needle} -ge 3 ]] && grep -aqF "$needle" "$zip"; then
        fail "ZIP chẩn đoán lộ thông tin người dùng ($needle) — vi phạm G8/S2"
    fi
done
for entry in report_summary.md version.json system_info.json linux_env.json panic_tail.log; do
    grep -aqF "$entry" "$zip" || fail "ZIP chẩn đoán thiếu $entry"
done
echo "  CLI chẩn đoán OK (--check-env/--diagnose/--export-log, không lộ PII)"

# 3. ABI runtime.
if command -v ibus-daemon >/dev/null 2>&1; then
    "$pkg/lib/textvn/textvn-ibus-engine" --check-abi || fail "textvn-ibus-engine --check-abi"
    echo "  IBus engine ABI OK"
fi
"$pkg/bin/textvn-settings" --check-abi || fail "textvn-settings --check-abi"
echo "  settings backend: $("$pkg/bin/textvn-settings" --version 2>/dev/null || echo '?')"
if [[ -f "$pkg/lib/textvn/fcitx5/libtextvn-fcitx5.so" ]]; then
    if "$pkg/bin/textvn" check-dlopen "$pkg/lib/textvn/fcitx5/libtextvn-fcitx5.so" >/dev/null 2>&1; then
        echo "  Fcitx5 plugin: nạp được"
    else
        echo "  Fcitx5 plugin: không nạp (không có Fcitx5 ≥ 5.0.14 / glibc 2.35) — IBus là đường chính, đúng thiết kế"
    fi
fi

# 4. Cài per-user rồi gỡ sạch (không cần root, không gọi trình quản lý gói).
pre="$HOME/.local"
"$pkg/install.sh" --no-install-deps --no-restart --no-activate >"$work/in.log" 2>&1 ||
    { cat "$work/in.log"; fail "install.sh per-user"; }
[[ -x "$pre/bin/textvn" ]] || fail "install.sh không cài $pre/bin/textvn"
"$pre/bin/textvn" --check-env --json >/dev/null 2>&1 || [[ $? -le 1 ]] || fail "textvn đã cài lỗi"
"$pre/share/textvn/uninstall.sh" --no-restart >"$work/un.log" 2>&1 ||
    { cat "$work/un.log"; fail "uninstall.sh"; }
[[ ! -e "$pre/bin/textvn" ]] || fail "uninstall.sh để sót $pre/bin/textvn"
[[ ! -e "$pre/share/textvn/install-manifest.txt" ]] || fail "uninstall.sh để sót manifest"
echo "  install.sh/uninstall.sh per-user OK"

echo "=== SMOKE PASS: $OS_PRETTY ==="
