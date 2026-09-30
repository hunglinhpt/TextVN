#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# tools/mac/mem-check.sh — kiểm RSS + CPU idle của 2 process TextVN (P2-5 §5).
#
# Dùng: tools/mac/mem-check.sh
# Ngưỡng (P2-5 §5): IMK < 60 MB · TextVN.app < 80 MB · không poll CPU nóng khi idle.
#
# Chỉ đọc số liệu process — KHÔNG đọc nội dung gõ (S2).
set -euo pipefail

IM_LIMIT_MB=60
APP_LIMIT_MB=80
CPU_WARN_PCT=5

if [ "$(uname -s)" != "Darwin" ]; then
    echo "SKIP: không phải macOS (uname=$(uname -s))"
    exit 3
fi

rss_mb() {   # $1 = tên process → RSS MB (rỗng → 0)
    PID="$(pgrep -x "$1" 2>/dev/null | head -1 || true)"
    if [ -z "$PID" ]; then echo "0"; return 0; fi
    KB="$(ps -o rss= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0)"
    echo "$((KB / 1024))"
}
cpu_pct() {  # $1 = tên process → %CPU (rỗng → 0)
    PID="$(pgrep -x "$1" 2>/dev/null | head -1 || true)"
    if [ -z "$PID" ]; then echo "0"; return 0; fi
    ps -o %cpu= -p "$PID" 2>/dev/null | tr -d ' ' || echo 0
}

IM_RSS="$(rss_mb TextVN-IM)"
APP_RSS="$(rss_mb TextVN)"
IM_CPU="$(cpu_pct TextVN-IM)"
APP_CPU="$(cpu_pct TextVN)"

if [ "$IM_RSS" -eq 0 ]; then
    echo "SKIP: TextVN-IM chưa chạy — bật input source rồi chạy lại"
    exit 3
fi

FAIL=0
echo "=== TextVN memory / CPU (P2-5 §5) ==="
printf 'IMK      : %s MB (ngưỡng < %s) · CPU %s%%\n' "$IM_RSS" "$IM_LIMIT_MB" "$IM_CPU"
if [ "$IM_RSS" -ge "$IM_LIMIT_MB" ]; then
    echo "❌ IMK vượt ngưỡng RSS"; FAIL=1
fi
if [ "$APP_RSS" -gt 0 ]; then
    printf 'TextVN   : %s MB (ngưỡng < %s) · CPU %s%%\n' "$APP_RSS" "$APP_LIMIT_MB" "$APP_CPU"
    if [ "$APP_RSS" -ge "$APP_LIMIT_MB" ]; then
        echo "❌ TextVN.app vượt ngưỡng RSS"; FAIL=1
    fi
else
    echo "⚠️  TextVN.app chưa chạy — bỏ qua ngưỡng 80 MB"
fi

# CPU idle: 3 mẫu cách nhau 5s, lấy mẫu lớn nhất (poll nóng = luôn cao)
i=0
CPU_MAX=0
while [ "$i" -lt 3 ]; do
    CPU_NOW="$(cpu_pct TextVN-IM)"
    # %cpu có thể là "0.0" → cắt phần thập phân
    CPU_INT="${CPU_NOW%%.*}"
    [ -n "$CPU_INT" ] || CPU_INT=0
    [ "$CPU_INT" -gt "$CPU_MAX" ] && CPU_MAX="$CPU_INT"
    i=$((i + 1))
    [ "$i" -lt 3 ] && sleep 5
done
printf 'CPU idle : max %s%% (cảnh báo > %s%%)\n' "$CPU_MAX" "$CPU_WARN_PCT"
if [ "$CPU_MAX" -gt "$CPU_WARN_PCT" ]; then
    echo "⚠️  CPU idle cao — kiểm lại vòng lặp poll (health/IPC), không phải lỗi cứng"
fi

[ "$FAIL" -eq 0 ] || exit 1
echo "✅ RSS trong ngưỡng"
