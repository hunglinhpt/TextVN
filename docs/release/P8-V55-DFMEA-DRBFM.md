<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# P8/V55 — DFMEA + DRBFM: 5 fix phản hồi test thật của chủ repo (2026-11-05)

> Phạm vi: (F1) "uong"→"ương" tự biến đổi; (F2) xung đột 2 bộ gõ trên
> TSF-bypass apps (MobaXterm — ô chữ nhật nhỏ/mũi tên xanh); (F3) username
> `truong.giap` bị bind; (F4) dead-key Windows không ra dấu; (F5) panel popup
> hiện liên tục che vùng gõ. Mỗi fix: RCA (5-Why) → thiết kế → DFMEA → DRBFM
> → gate test.

## F1 — "uong" tự thành "ương" (phá chuẩn UniKey 20 năm)

**5-Why:**
1. User gõ `uong` thấy `ương` — **không gõ key dấu nào**.
2. Tại sao? Rule Telex `uo→ươ` của TextVN áp **NGAY** khi `o` gõ liền sau `u`
   (không điều kiện) — `telex.rs` rule auto.
3. Tại sao rule có điều kiện? Thiết kế cũ (test `duong`→`dương` khẳng định
   hành vi này) — chọn "tiện gõ nhanh" thay vì parity UniKey.
4. Tại sao trái chuẩn? UniKey/EVKey 20 năm: `uo` giữ nguyên khi không có key
   dấu; ươ chỉ xuất hiện khi có `w` (`dduowng`) hoặc dấu áp vào vần
   (`dduocj`→`được` — dấu buộc).
5. Tại sao không phát hiện sớm? Corpus hồi quy đã **khóa hành vi sai**
   (`stroke_dd_and_d_vowel` assert `duong`→`dương`) — test phản ánh code, không
   phản ánh chuẩn user.

**Thiết kế (P8-a):** rule auto chỉ áp khi từ **đã có biến đổi VN** (`dd`→`đ`,
dấu); `w`-path khi cặp `uo` thường liền trước `w` → promote thật
(`thuowng`→`thương`); tone-key path promote ươ trước áp dấu
(`duongf`→`dường` — UniKey parity "dấu vào o sau u → ơ").

**DFMEA (rút gọn — S/O/D 1-10):**

| Item | Failure Mode | Effect | S | Cause | O | D | RPN | Action + Gate |
|---|---|---|---|---|---|---|---|---|
| rule auto | `dduocj` mất ươ (quá chặt) | `đuocj` — regression | 9 | transformed-gate sai | 2 | test cũ `dduocj`→`được` giữ xanh | 2 | 36 | Gate: test method cũ không đổi kỳ vọng |
| tone path | `duongf` ra `dườn`g sai target | lỗi chính tả | 8 | promote sau áp-tone | 2 | promote TRƯỚC apply_key | 2 | 32 | Gate: `duongf`→`dường` |
| w path | `nguoiwf` break (w sau cụm uoi) | regression R2 | 8 | promote cả khi có i giữa | 3 | promote chỉ first_w + cặp liền (has_uo_vowel_pair; `nguoiwf` vẫn horn chuẩn "uoi"→"ươi") | 2 | 48 | Gate: `horn_after_whole_cluster` giữ xanh |
| corpus cũ | tests khóa hành vi cũ quay lại | lần sau lặp bug | 7 | không cập nhật kỳ vọng | 2 | cập nhật test + comment P8 lý do | 2 | 28 | Gate: `duong`→`duong` assert mới |

**DRBFM:** giả định nền — UniKey parity là chuẩn mong muốn (chủ repo xác
nhận "user quen UniKey 20 năm coi là lỗi"); changed points = 3 path trong
telex.rs; verification = RCA thực nghiệm 4 đường (Space/Tab/mặc định/username)
đều giữ nguyên + test method cũ xanh.

## F2 — xung đột OS-IME trên TSF-bypass apps (MobaXterm)

**5-Why:**
1. Gõ chữ trong MobaXterm — chữ rơi vào "ô chữ nhật nhỏ" (composition của
   Windows VIE IME) không tới terminal.
2. Tại sao? Windows VIE IME (bàn phím tiếng Việt tích hợp) bắt phím VK gốc.
3. Tại sao IME bắt được? TextVN hook inject chữ **đã biến đổi** qua
   KEYEVENTF_UNICODE (bypass IME) nhưng phím **không biến đổi** (raw ==
   display → Action::Pass) được **forward VK gốc** → OS-IME can thiệp.
4. Tại sao còn forward VK gốc? Hook chỉ nuốt khi Transform — Pass luôn
   CallNextHookEx.
5. Tại sao thiết kế như vậy? Chưa account session có VIE IME bật cạnh
   TextVN hook — 2 bộ gõ giành phím (RCA chủ repo dẫn UniKey/EVKey hiện
   tượng tương tự).

**Thiết kế (P8-b):** hook-mode **swallow-and-unicode** — engine Pass trên
letter thuần (không Ctrl/Alt/Win) → vẫn nuốt + inject lại chính ký tự qua
KEYEVENTF_UNICODE (chữ gõ-ready bypass IME); modifier/hotkey giữ Pass.

**DFMEA:**

| Item | Failure Mode | Effect | S | Cause | O | D | RPN | Action + Gate |
|---|---|---|---|---|---|---|---|---|
| PassUnicode shape | inject thiếu byte (UTF-16 surrogate) | chữ hỏng | 8 | insert_len chỉ 1 unit | 2 | surrogate pair chữ BMP-VN không có; gate contract: insert_len==1 | 2 | 32 | Gate: ime_conflict_1000 shape |
| phím hệ thống bị nuốt | Ctrl+C/S… vào hook app | mất hotkey | 9 | is_plain_letter sai | 2 | loại Ctrl/Alt/Win trong helper + matrix test | 2 | 36 | Gate: modifier matrix |
| vòng đệ quy | UNICODE inject tự kích hook | loop gõ vô tận | 9 | thiếu IN_INJECTION guard | 1 | guard WIN-041 có sẵn giữ nguyên | 2 | 18 | (đủ) |
| Space giữ VK | IME composition với Space? | box kẹt | 4 | Space không letter | 4 | Space qua Pass VK (IME không dựng box với space thuần — hành vi đã quan sát) | 3 | 48 | Ghi nhận e2e user |
| app không nhận UNICODE | terminal cũ chỉ nhận VK | mất chữ | 8 | KEYEVENTF_UNICODE unsupported | 2 | hiện trạng: đã dùng cho Transform từ 0.2.61 (MobaXterm nhận tốt — chứng minh runtime) | 2 | 32 | (đủ — runtime đã chứng minh) |

**DRBFM:** nền — Transform-UNICODE chạy production từ 0.2.61 không báo lỗi
chèn; changed = mở cơ chế cho Pass; verification = ime_conflict_1000 + e2e
user MobaXterm sau 0.2.65.

## F3 — username `truong.giap` bị bind

**5-Why:** `truong.giap` → `trương.giap` vì rule `uo→ươ` không điều kiện
(cùng gốc F1); `r` sau `t` không đổi nhưng `uo` liền → ươ. **Thiết kế:** fix
F1 xử lý gốc; gate riêng 1000-case username (dot/dash/underscore/số/email,
kể cả style_words cá nhân có "truong" cũng không bind). **DRBFM:** Giả định nền:
prediction ghost vẫn gợi ý nhưng commit không đổi; verification =
username_identity_1000.

## F4 — dead-key Windows (`Ctrl+'`, `Ctrl+` + letter…)

**5-Why:** gõ `Ctrl+'` + `e` không ra `é`. (2) TIP nuốt letter `e` vào
composition. (3) `plan_key` PASS+Char → eaten:true mặc định. (4) Không
account OS dead-key state. (5) TIP không có track dead-key.

**Thiết kế (P8-c):** session flag `dead_key_armed` — phím OEM-range Pass
(Ctrl-combo hoặc không modifier) → arm; letter kế khi armed → KHÔNG ăn +
commit composition hiện có + forward cho OS.

**DFMEA:** phím F-key/arrows nhầm arm (arm giới hạn VK 0xBA..0xE2); dead-key
nhưng letter bị engine biến đổi trước (letter có nghĩa Telex — dead-key +
letter có key: `Ctrl+'`+`ss`? — `s` biến sắc: ưu tiên Telex, dead-key hủy
— ghi nhận behavior: Telex thắng khi phím có nghĩa, dead-key chỉ còn với
letter không biến đổi; UniKey cùng behavior). DRBFM: verification = e2e user
Trên layout EN-US (Notepad: `Ctrl+'` `e` → `é`), gate regression edit_session
tests cũ xanh.

## F5 — panel popup hiện liên tục che vùng gõ

**5-Why:** panel hiện NGAY tại sentence-trigger; đứng đó trong lúc gõ tiếp.
(2) show không có idle gating. (3) live-popup refresh mỗi Word-trigger.
(4) Thiết kế D11 ưu tiên "hiện ngay"; user lại cần không-che. (5) Không có
cấu hình idle.

**Thiết kế (P9):** debounce 1s (`panel_idle.rs` — timer riêng, mỗi phím gõ
re-schedule + popup sống ẨN NGAY khi gõ tiếp; timer fire → on-demand check →
show). Auto-fix không qua debounce.

**DFMEA:** timer đụng ID khác (0x5EC3 — tách khỏi selection/deferred);
check-on-timer fail-open (no-view → im lặng); auto-fix regression — giữ path
cũ; G8: chỉ lý do + đếm trong skip-log. DRBFM: verification = panel_idle_1000
+ e2e user (gõ liên tục panel không nhấp nháy; dừng 1s mới hiện).

## Tổng hợp gate tests mới (vòng 55)

| Gate | File | Số case | Bảo vệ |
|---|---|---|---|
| username_identity_1000 | assist/tests (core engine) | 1000 | F3 |
| ime_conflict_1000 | adapters/windows-hook/tests | 1000 | F2 |
| panel_idle_1000 | adapters/windows-tsf/tests | 1000 | F5 |
| rca_uong (4 đường) | core/tests (đã bắc cầu → method tests) | 4 | F1 |
| method tests P8 | core/src/method/telex.rs | cập nhật | F1/F3 |
