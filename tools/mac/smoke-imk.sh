#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# tools/mac/smoke-imk.sh — smoke IMK **thật** trên macOS: TextEdit + 6 case (P2-5 §4.1).
#
# Dùng: tools/mac/smoke-imk.sh
#
# Yêu cầu: input source TextVN đã bật + Accessibility cho terminal/runner
# (System Events `keystroke`). Thiếu GUI/quyền → in lý do rồi exit 3 (P2-5 §4 RM5:
# "AX harness = local/nightly, ghi rõ lý do" — không được fail ngầm).
#
# Bộ case: lấy từ `corpus/mac` (đã verify bằng engine, replay headless ở CI) nên
# giá trị mong đợi ở đây KHÔNG bịa: chính là output engine ở `--adapter mac`.
set -euo pipefail

say() { echo "[smoke-imk] $*"; }

if [ "$(uname -s)" != "Darwin" ]; then
    say "SKIP: không phải macOS (uname=$(uname -s))"
    exit 3
fi
command -v osascript >/dev/null 2>&1 || { say "SKIP: thiếu osascript"; exit 3; }

if ! pgrep -x TextVN-IM >/dev/null 2>&1; then
    say "SKIP: TextVN-IM chưa chạy — bật input source TextVN (System Settings → Keyboard → Input Sources)"
    exit 3
fi

if ! osascript -e 'tell application "System Events" to return UI elements enabled' 2>/dev/null | grep -qi true; then
    say "SKIP: chưa cấp Accessibility cho process chạy script (System Events keystroke sẽ im lặng)"
    exit 3
fi

# keys=expect — mọi dòng đều đã được engine xác nhận (corpus/mac + corpus/shared).
CASES='dduocj=được
chaof=chào
hoaf=hoà
tieengs=tiếng
vietj=việt
nguyen=nguyen'

SMOKE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/TextVN-smoke-XXXXXXXX")"
SMOKE_NAME="$(basename "$SMOKE_DIR").txt"
SMOKE_FILE="$SMOKE_DIR/$SMOKE_NAME"
: > "$SMOKE_FILE"

cleanup() {
    osascript -e "tell application \"TextEdit\" to if (count of documents whose name is \"$SMOKE_NAME\") is 1 then close document \"$SMOKE_NAME\" saving no" >/dev/null 2>&1 || true
    rm -f -- "$SMOKE_FILE"
    rmdir -- "$SMOKE_DIR" 2>/dev/null || true
}
trap cleanup EXIT
open -a TextEdit "$SMOKE_FILE"
if ! osascript -e "tell application \"TextEdit\" to return (count of documents whose name is \"$SMOKE_NAME\")" 2>/dev/null | grep -qx 1; then
    say "SKIP: TextEdit không mở được tài liệu smoke riêng; không chạm vào tài liệu khác"
    exit 3
fi
sleep 1

PASS=0
FAIL=0

type_keys() {  # $1 = chuỗi telex, gõ từng ký tự qua System Events (đúng đường phím → IMK)
    local i=0 n=${#1} ch
    while [ "$i" -lt "$n" ]; do
        ch="${1:$i:1}"
        case "$ch" in
            '"'|'\') osascript -e 'return 1' >/dev/null ;;   # ký tự đặc biệt: bỏ qua (case không dùng)
            *) osascript -e "tell application \"System Events\" to keystroke \"$ch\"" >/dev/null ;;
        esac
        i=$((i + 1))
    done
}

clear_doc() { osascript -e "tell application \"TextEdit\" to set text of document \"$SMOKE_NAME\" to \"\"" >/dev/null; sleep 0.3; }
read_doc() { osascript -e "tell application \"TextEdit\" to return text of document \"$SMOKE_NAME\"" 2>/dev/null | tr -d '\r'; }

while IFS='=' read -r KEYS EXPECT; do
    [ -n "$KEYS" ] || continue
    clear_doc
    type_keys "$KEYS"
    sleep 1
    GOT="$(read_doc)"
    if [ "$GOT" = "$EXPECT" ]; then
        PASS=$((PASS + 1))
        echo "PASS  $KEYS → $GOT"
    else
        FAIL=$((FAIL + 1))
        echo "FAIL  $KEYS → '$GOT' (mong đợi '$EXPECT')"
    fi
done <<EOF
$CASES
EOF

# Case 6 — undo: gõ xong rồi ⌘Z, mong đợi TextEdit xoá cả lần chèn (document trở về rỗng).
clear_doc
type_keys "dduocj"
sleep 1
AFTER_TYPE="$(read_doc)"
osascript -e 'tell application "System Events" to keystroke "z" using command down' >/dev/null
sleep 1
AFTER_UNDO="$(read_doc)"
if [ "$AFTER_TYPE" = "được" ] && [ -z "$AFTER_UNDO" ]; then
    PASS=$((PASS + 1))
    echo "PASS  undo: '$AFTER_TYPE' → '$AFTER_UNDO'"
else
    FAIL=$((FAIL + 1))
    echo "FAIL  undo: sau khi gõ '$AFTER_TYPE', sau ⌘Z '$AFTER_UNDO' (mong đợi '' )"
fi

# Case 7 — toggle EN/VN: Ctrl+Shift+Space (MAC-018) → gõ phải ra ký tự thô.
osascript -e 'tell application "System Events" to keystroke " " using {control down, shift down}' >/dev/null
sleep 0.5
clear_doc
type_keys "dduocj"
sleep 1
GOT_TOGGLE="$(read_doc)"
# toggle lại về VN để không để máy ở trạng thái khác lúc đầu
osascript -e 'tell application "System Events" to keystroke " " using {control down, shift down}' >/dev/null
if [ "$GOT_TOGGLE" = "dduocj" ]; then
    PASS=$((PASS + 1))
    echo "PASS  toggle EN: 'dduocj' (không biến đổi)"
else
    FAIL=$((FAIL + 1))
    echo "FAIL  toggle EN: '$GOT_TOGGLE' (mong đợi 'dduocj' — có thể hotkey bị hệ thống chiếm: xem MAC-018)"
fi

echo "--- $PASS pass · $FAIL fail ---"
[ "$FAIL" -eq 0 ] || exit 1
say "✅ smoke IMK pass ($PASS case)"
