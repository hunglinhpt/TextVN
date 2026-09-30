#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# tools/mac/soak.sh — soak test TextVN-IM.app + TextVN.app trên macOS (MAC-063, P2-5 §5·§6).
#
# Dùng:
#   tools/mac/soak.sh -Hours 24                 # local Mac thật (weekly → Issue)
#   tools/mac/soak.sh -Hours 2 -Interval 60     # nightly CI (P2-5 §8)
#   tools/mac/soak.sh -Seconds 60 -Interval 20  # thử nhanh chính script này
#
# Gate (P2-5 §6): 0 crash · 0 lần IMK restart ngoài ý muốn · ΔRSS < 10 MB/process.
# Chỉ đọc **số liệu process** (pid/RSS) và đếm crash report — KHÔNG đọc nội dung gõ (S2).
set -euo pipefail

HOURS=24
INTERVAL=300
TOTAL=""
OUT=""

while [ $# -gt 0 ]; do
    case "$1" in
        -Hours)      HOURS="${2:?}"; shift 2 ;;
        -Hours=*)    HOURS="${1#*=}"; shift ;;
        -Interval)   INTERVAL="${2:?}"; shift 2 ;;
        -Interval=*) INTERVAL="${1#*=}"; shift ;;
        -Seconds)    TOTAL="${2:?}"; shift 2 ;;
        -Seconds=*)  TOTAL="${1#*=}"; shift ;;
        -Out)        OUT="${2:?}"; shift 2 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

[ -n "$TOTAL" ] || TOTAL=$((HOURS * 3600))
case "$INTERVAL" in ''|*[!0-9]*) echo "-Interval phải là số giây ≥ 1" >&2; exit 2 ;; esac
[ "$INTERVAL" -ge 1 ] || { echo "-Interval phải ≥ 1" >&2; exit 2; }

ROUNDS=$((TOTAL / INTERVAL))
if [ "$ROUNDS" -lt 2 ]; then
    echo "cần ≥ 2 vòng đo (TOTAL/INTERVAL = $ROUNDS) — giảm -Interval" >&2
    exit 2
fi

RSS_LIMIT_MB=10
STAMP="$(mktemp -t textvn-soak)"
DIAG_DIR="$HOME/Library/Logs/DiagnosticReports"
[ -n "$OUT" ] || OUT="dist/mac-soak-$(date +%Y%m%d-%H%M%S).json"
mkdir -p "$(dirname "$OUT")"

rss_kb() {   # $1 = pid (rỗng = không chạy) → RSS KB
    if [ -z "${1:-}" ]; then echo 0; return 0; fi
    ps -o rss= -p "$1" 2>/dev/null | tr -d ' ' || echo 0
}
pid_of() { pgrep -x "$1" 2>/dev/null | head -1 || true; }

echo "=== TextVN soak: ${TOTAL}s · mẫu mỗi ${INTERVAL}s · $ROUNDS vòng ==="
echo "report → $OUT"

FIRST_IM_RSS=0
FIRST_APP_RSS=0
LAST_IM_RSS=0
LAST_APP_RSS=0
IM_RESTARTS=0
IM_MISSING=0
IM_PID_LAST=""
i=0

while [ "$i" -lt "$ROUNDS" ]; do
    IM_PID="$(pid_of TextVN-IM)"
    APP_PID="$(pid_of TextVN)"
    IM_RSS="$(rss_kb "$IM_PID")"
    APP_RSS="$(rss_kb "$APP_PID")"

    if [ -z "$IM_PID" ]; then
        IM_MISSING=$((IM_MISSING + 1))
    else
        if [ -n "$IM_PID_LAST" ] && [ "$IM_PID" != "$IM_PID_LAST" ]; then
            IM_RESTARTS=$((IM_RESTARTS + 1))
            echo "  ! vòng $i: IMK restart (pid $IM_PID_LAST → $IM_PID)"
        fi
        IM_PID_LAST="$IM_PID"
        [ "$FIRST_IM_RSS" -eq 0 ] && FIRST_IM_RSS="$IM_RSS"
        LAST_IM_RSS="$IM_RSS"
    fi
    if [ -n "$APP_PID" ]; then
        [ "$FIRST_APP_RSS" -eq 0 ] && FIRST_APP_RSS="$APP_RSS"
        LAST_APP_RSS="$APP_RSS"
    fi

    i=$((i + 1))
    [ "$i" -lt "$ROUNDS" ] && sleep "$INTERVAL"
done

CRASHES="$(find "$DIAG_DIR" -name 'TextVN*' -newer "$STAMP" 2>/dev/null | wc -l | tr -d ' ')"
rm -f "$STAMP"

IM_DELTA=$(((LAST_IM_RSS - FIRST_IM_RSS) / 1024))
APP_DELTA=$(((LAST_APP_RSS - FIRST_APP_RSS) / 1024))
ABS_IM=${IM_DELTA#-}
ABS_APP=${APP_DELTA#-}

STATUS="pass"
REASONS=""
if [ "$FIRST_IM_RSS" -eq 0 ]; then
    STATUS="fail"; REASONS="$REASONS IMK chưa từng chạy (mở TextVN-IM.app trước khi soak);"
fi
[ "$IM_MISSING" -gt 0 ] && { STATUS="fail"; REASONS="$REASONS IMK vắng ở $IM_MISSING/$ROUNDS mẫu;"; }
[ "$IM_RESTARTS" -gt 0 ] && { STATUS="fail"; REASONS="$REASONS IMK restart $IM_RESTARTS lần;"; }
[ "$CRASHES" -gt 0 ] && { STATUS="fail"; REASONS="$REASONS $CRASHES crash report;"; }
if [ "$ABS_IM" -gt "$RSS_LIMIT_MB" ] || [ "$ABS_APP" -gt "$RSS_LIMIT_MB" ]; then
    STATUS="fail"; REASONS="$REASONS ΔRSS vượt ${RSS_LIMIT_MB}MB;"
fi

{
    printf '{\n'
    printf '  "suite": "soak",\n'
    printf '  "platform": "macos",\n'
    printf '  "seconds": %s,\n' "$TOTAL"
    printf '  "interval_s": %s,\n' "$INTERVAL"
    printf '  "rounds": %s,\n' "$ROUNDS"
    printf '  "im_rss_first_mb": %s,\n' "$((FIRST_IM_RSS / 1024))"
    printf '  "im_rss_last_mb": %s,\n' "$((LAST_IM_RSS / 1024))"
    printf '  "app_rss_first_mb": %s,\n' "$((FIRST_APP_RSS / 1024))"
    printf '  "app_rss_last_mb": %s,\n' "$((LAST_APP_RSS / 1024))"
    printf '  "rss_delta_limit_mb": %s,\n' "$RSS_LIMIT_MB"
    printf '  "im_restarts": %s,\n' "$IM_RESTARTS"
    printf '  "im_missing_samples": %s,\n' "$IM_MISSING"
    printf '  "crash_reports": %s,\n' "$CRASHES"
    printf '  "status": "%s",\n' "$STATUS"
    printf '  "notes": "%s"\n' "$REASONS"
    printf '}\n'
} > "$OUT"

echo "--- kết quả ---"
echo "IMK RSS  : $((FIRST_IM_RSS / 1024)) MB → $((LAST_IM_RSS / 1024)) MB (Δ ${IM_DELTA} MB)"
echo "App RSS  : $((FIRST_APP_RSS / 1024)) MB → $((LAST_APP_RSS / 1024)) MB (Δ ${APP_DELTA} MB)"
echo "status   : $STATUS$REASONS"

[ "$STATUS" = "pass" ] || exit 1
echo "✅ soak pass (0 crash · 0 restart · ΔRSS < ${RSS_LIMIT_MB}MB)"
