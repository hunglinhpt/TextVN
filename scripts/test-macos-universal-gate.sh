#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# test-macos-universal-gate.sh — tự kiểm + đột biến (D3) gate ARM-M1 KHÔNG cần
# macOS thật: gọi đúng hàm gate dùng chung (sourced từ scripts/build-macos.sh)
# với fixture stub mang magic Mach-O + "lipo" giả mô phỏng chuỗi arch. Chạy
# được trên Git Bash Windows, macOS bash 3.2, Linux bash (không phụ thuộc lipo).
#   ĐỎ  : stub 1 kiến trúc (chỉ x86_64 / chỉ arm64) ⇒ scan exit 1 + liệt kê tệp.
#   XANH: stub universal (fat magic); file thường (ELF) phải bị bỏ qua.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD="$SCRIPT_DIR/build-macos.sh"

pass=0
fail=0
ok() { pass=$((pass + 1)); echo "  PASS  $*"; }
ng() { fail=$((fail + 1)); echo "  FAIL  $*"; }

# Cú pháp sạch cho cả 3 tệp thuộc gate (bash -n dùng đúng bash đang chạy).
for f in "$BUILD" "$SCRIPT_DIR/package-macos-pkg.sh" "$SCRIPT_DIR/test-macos-universal-gate.sh"; do
    if bash -n "$f" 2>/dev/null; then ok "bash -n sạch: $(basename "$f")"; else ng "bash -n lệch cú pháp: $f"; fi
done

# Nạp hàm gate — guard trong build-macos.sh phải chặn body build khi source.
# shellcheck source=scripts/build-macos.sh
. "$BUILD"
if [ -z "${DIST_DIR:-}" ] && [ -z "${STAGE_DIR:-}" ]; then
    ok "source build-macos.sh không chạy body build (guard source hoạt động)"
else
    ng "source build-macos.sh lại chạy body build (guard hỏng)"
fi

# Gate phải còn được gọi trên đường phát hành thật (chống xoá lặng lẽ lời gọi).
grep -q "textvn_uni_scan_dir" "$BUILD" \
    && ok "build-macos.sh còn lời gọi gate ARM-M1" || ng "build-macos.sh MẤT lời gọi gate ARM-M1"
grep -q "textvn_uni_scan_dir" "$SCRIPT_DIR/package-macos-pkg.sh" \
    && ok "package-macos-pkg.sh còn lời gọi gate ARM-M1" || ng "package-macos-pkg.sh MẤT lời gọi gate ARM-M1"

# ---- 1. Hàm thuần: parse chuỗi arch mô phỏng (đúng dạng lipo -archs / file -b) ----
echo "== Hàm thuần textvn_uni_have_both_archs — chuỗi mô phỏng =="
if textvn_uni_have_both_archs "arm64 x86_64"; then
    ok "'arm64 x86_64' (lipo universal) ⇒ đủ"
else
    ng "'arm64 x86_64' bị từ chối oan"
fi
if textvn_uni_have_both_archs "x86_64 arm64"; then
    ok "'x86_64 arm64' (thứ tự ngược) ⇒ đủ"
else
    ng "'x86_64 arm64' bị từ chối oan"
fi
if textvn_uni_have_both_archs "x86_64"; then
    ng "'x86_64' (đột biến: chỉ 1 arch) lại ĐẠT — lọt lỗi OpenKey #189"
else
    ok "'x86_64' ⇒ từ chối (đột biến ĐỎ)"
fi
if textvn_uni_have_both_archs "arm64"; then
    ng "'arm64' (đột biến: chỉ 1 arch) lại ĐẠT"
else
    ok "'arm64' ⇒ từ chối (đột biến ĐỎ)"
fi
if textvn_uni_have_both_archs "i386 ppc7400"; then
    ng "'i386 ppc7400' (2 arch cũ) lại ĐẠT"
else
    ok "'i386 ppc7400' ⇒ từ chối (arch cũ không được tính thay arm64/x86_64)"
fi
# Chuỗi `file -b` thật trên macOS (universal + đơn arch) qua đúng đường parse sed.
file_uni="$(printf '%s ' "Mach-O universal binary with 2 architectures: [x86_64:Mach-O 64-bit executable x86_64] [arm64:Mach-O 64-bit executable arm64]" | sed 's/[][():]/ /g')"
if textvn_uni_have_both_archs "$file_uni"; then
    ok "chuỗi file -b universal ⇒ đủ"
else
    ng "chuỗi file -b universal bị từ chối oan"
fi
file_one="$(printf '%s ' "Mach-O 64-bit executable x86_64" | sed 's/[][():]/ /g')"
if textvn_uni_have_both_archs "$file_one"; then
    ng "chuỗi file -b đơn arch lại ĐẠT"
else
    ok "chuỗi file -b đơn arch ⇒ từ chối (đột biến ĐỎ)"
fi

# ---- 2. Scan thư mục: fixture stub + lipo giả (TEXTVN_UNI_LIPO) ----
echo "== Scan thư mục: stub binary + lipo giả =="
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

printf '\xcf\xfa\xed\xfe\x00\x00\x00\x00STUB-X86-ONLY' > "$work/stub-x86only.bin"          # MH_MAGIC_64 LE
printf '\xce\xfa\xed\xfe\x00\x00\x00\x00STUB-ARM-ONLY' > "$work/stub-armonly.bin"          # MH_CIGAM_64
printf '\xca\xfe\xba\xbe\x00\x00\x00\x01STUB-FAT-UNIVERSAL' > "$work/stub-universal.bin"   # FAT_MAGIC
printf '\x7f\x45\x4c\x46\x02\x01\x01\x00ELF-NOT-MACHO' > "$work/notmacho.so"               # ELF — phải bỏ qua

fake_lipo="$work/fake-lipo.sh"
{
    printf '#!/bin/sh\n'
    printf '# lipo thật: lipo -archs <tệp> → đối chiếu toàn bộ tham số.\n'
    printf 'case "$*" in\n'
    printf '  *stub-x86only*)   echo "x86_64" ;;\n'
    printf '  *stub-armonly*)   echo "arm64" ;;\n'
    printf '  *stub-universal*) echo "arm64 x86_64" ;;\n'
    printf '  *) echo "x86_64" ;;\n'
    printf 'esac\n'
} > "$fake_lipo"
chmod +x "$fake_lipo"
export TEXTVN_UNI_LIPO="$fake_lipo"

# XANH: dir chứa universal + file thường (ELF bị bỏ qua; nếu gate quét nhầm ELF
# thì fake lipo trả "x86_64" ⇒ đỏ — tự phát hiện false-positive).
green="$work/green"; mkdir "$green"
cp "$work/stub-universal.bin" "$green/"
cp "$work/notmacho.so" "$green/"
rc=0; textvn_uni_scan_dir "$green" > "$work/green.log" 2>&1 || rc=$?
if [ "$rc" -eq 0 ]; then
    ok "scan dir universal + ELF ⇒ exit 0 (ELF bị bỏ qua)"
else
    ng "scan dir universal ⇒ exit $rc (phải 0). Log:"; cat "$work/green.log" >&2
fi
if grep -q "notmacho" "$work/green.log"; then
    ng "scan nhầm file ELF (notmacho.so) là Mach-O"
else
    ok "không quét nhầm file ELF"
fi

# ĐỎ #1: stub chỉ x86_64 — đột biến tái hiện OpenKey #189.
red1="$work/red1"; mkdir "$red1"
cp "$work/stub-x86only.bin" "$red1/"
rc=0; textvn_uni_scan_dir "$red1" > "$work/red1.log" 2>&1 || rc=$?
if [ "$rc" -eq 1 ] && grep -q "stub-x86only.bin" "$work/red1.log"; then
    ok "ĐỎ: stub chỉ x86_64 ⇒ exit 1 + liệt kê tệp lỗi"
else
    ng "stub chỉ x86_64 ⇒ exit $rc (phải 1). Log:"; cat "$work/red1.log" >&2
fi

# ĐỎ #2: stub chỉ arm64.
red2="$work/red2"; mkdir "$red2"
cp "$work/stub-armonly.bin" "$red2/"
rc=0; textvn_uni_scan_dir "$red2" > "$work/red2.log" 2>&1 || rc=$?
if [ "$rc" -eq 1 ] && grep -q "stub-armonly.bin" "$work/red2.log"; then
    ok "ĐỎ: stub chỉ arm64 ⇒ exit 1 + liệt kê tệp lỗi"
else
    ng "stub chỉ arm64 ⇒ exit $rc (phải 1). Log:"; cat "$work/red2.log" >&2
fi

# ĐỎ #3: trộn universal + 1 arch ⇒ vẫn đỏ (không nuốt tệp lỗi).
red3="$work/red3"; mkdir "$red3"
cp "$work/stub-universal.bin" "$work/stub-x86only.bin" "$red3/"
rc=0; textvn_uni_scan_dir "$red3" > "$work/red3.log" 2>&1 || rc=$?
if [ "$rc" -eq 1 ]; then
    ok "ĐỎ: trộn universal + 1 arch ⇒ exit 1"
else
    ng "trộn universal + 1 arch ⇒ exit $rc (phải 1). Log:"; cat "$work/red3.log" >&2
fi

# ĐỎ #4: thư mục quét không tồn tại.
rc=0; textvn_uni_scan_dir "$work/khong-ton-tai" > "$work/red4.log" 2>&1 || rc=$?
if [ "$rc" -eq 1 ]; then
    ok "ĐỎ: thư mục quét không tồn tại ⇒ exit 1"
else
    ng "thư mục quét không tồn tại ⇒ exit $rc (phải 1)"
fi

# ---- 3. Nhánh `file` (không có lipo) — RED portability ----
echo "== Nhánh \`file\` fallback (TEXTVN_UNI_LIPO trỏ tên không tồn tại) =="
filered="$work/filered"; mkdir "$filered"
cp "$work/stub-x86only.bin" "$filered/"
rc=0
TEXTVN_UNI_LIPO="$work/khong-co-lipo-nao" textvn_uni_scan_dir "$filered" > "$work/filered.log" 2>&1 || rc=$?
if [ "$rc" -eq 1 ]; then
    ok "ĐỎ: probe bằng file (không lipo) vẫn từ chối stub 1 arch ⇒ exit 1"
else
    ng "file-fallback: stub 1 arch ⇒ exit $rc (phải 1). Log:"; cat "$work/filered.log" >&2
fi

echo "== Kết quả: $pass PASS, $fail FAIL =="
if [ "$fail" -eq 0 ]; then
    echo "✅ ARM-M1 gate self-test ĐẠT (đủ ca đột biến ĐỎ + fixture XANH)"
    exit 0
fi
echo "❌ ARM-M1 gate self-test THẤT BẠI"
exit 1
