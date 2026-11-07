<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Đề xuất tối ưu & tính năng — vòng 56 (2026-11-07)

> Thu thập từ RCA các vòng 51-55 + audit code hiện trạng. Mỗi đề xuất: giá
> trị với user, chi phí kỹ thuật, rủi ro, thứ tự ưu tiên. Đánh dấu ✅ = đã
> thực hiện trong phiên này; ◻ = đề xuất chờ chủ repo duyệt.

## ✅ Đã thực hiện (0.2.65 + vòng 56)

| # | Tính năng/Solution | Giá trị | Gate |
|---|---|---|---|
| 1 | **User Style Profile (P7)** — học từ cá nhân + cặp sửa lỗi user đã chấp nhận, cap 16KB | Gợi ý đúng từ người dùng hay dùng (tên riêng, thuật ngữ công ty), correction không bêu chữ đúng | username_identity_1000 + style_profile tests |
| 2 | **Swallow-and-unicode** (hook) — phím letter không biến đổi cũng inject qua KEYEVENTF_UNICODE | Hết xung đột 2 bộ gõ trên MobaXterm/Terminal (ô chữ nhật nhỏ/mũi tên xanh) | ime_conflict_1000 |
| 3 | **Chuẩn UniKey vần uo** — `luong`/`uong` giữ nguyên khi không dấu | 20 năm thói quen UniKey không bị phá; muốn `lương` gõ `luonwg` | rca_luong + corpus P8 |
| 4 | **Dead-key passthrough** — `Ctrl+'`+`e` → `é` | Ký hiệu Â/Ê/Ô và dấu thanh kiểu Windows dùng được cạnh TextVN | dead_key_armed logic + R2-120e corpus |
| 5 | **Panel debounce 1s** — gõ liên tục panel không nhấp nháy/che chữ | Popup chỉ hiện khi dừng tay, không che vùng đang gõ | panel_idle_1000 |
| 6 | **Gate username/id** — `truong.giap` không bị hoàn tất | Đăng nhập/gõ id an toàn | username_identity_1000 |
| 7 | **abnormal_1000** — 1000 kịch bản biên deterministic | Chặn hồi quy toàn bề mặt engine | abnormal_1000 |
| 8 | **gen-vocab Rust** thay Python | Đúng ngôn ngữ policy; validator làm nguồn sự thật | gen-vocab + 114 corpus |

## ◻ Đề xuất tính năng (ưu tiên theo giá trị user)

### A. Hoàn thiện User Style Profile (P7) — vòng tiếp theo
1. **UI quản lý dữ liệu học** (P7-DFMEA F.6): bảng xem từ đã học + xóa từng
   mục/nút "Xóa tất cả" trong bảng điều khiển 3 nền tảng. Giá trị: minh bạch
   G8, user kiểm soát. Chi phí: UI 3 OS + IPC message mới. Rủi ro thấp.
2. **Drain persist theo OS** (P7-DFMEA D2): tray Windows (IPC `StyleUpdate`),
   IMK macOS (đọc/ghi config.json trực tiếp), Linux (`lc_style_profile_merge`).
   Đang để v2 vì cần debounce + merge chống lost-update (F5).
3. **Học cụm (bigram contexts)** cho next-phrase cá nhân — "kính gửi" user
   hay gõ kèm cụm riêng của họ. Cần thêm `phrases` cap riêng trong schema.

### B. Prediction chất lượng cao hơn
4. **Tần suất thật thay weight tĩnh**: corpora hiện weight tay 1-100; chuyển
   sang frequency rank thật (từ corpus văn bản công khai, chuẩn hóa) giúp
   ứng viên đầu đúng hơn trên prefix dài. Chi phí: script import + re-gen,
   giữ API.
5. **Context-aware ranking**: 2-3 từ liền trước tăng điểm ứng viên khớp cụm
   (n-gram rank nội bộ, không mạng). Đã có nền tảng phrase-bucket; cần thêm
   bảng trigram hữu hạn.
6. **Ghost tắt theo app** (appdb field `ghost` riêng per-app): Excel/browser
   URL bar thường không cần ghost — giảm nhiễu.

### C. Ổn định hệ thống
7. **Windows VIE IME guard**: khi hook-owner app focus + Windows VIE keyboard
   layout active → toast hướng dẫn tắt VIE IME (swallow-and-unicode đã xử lý
   phần lớn; guard cho case unicode-not-supported apps). Chi phí: enumerate
   keyboard layout + toast tray.
8. **Dead-keyarmed reset khi focus đổi** — đã disarm tại `finish()`; thêm
   disarm khi app switch (WM_ACTIVATEAPP) cho chắc chắn (R2-120e).
9. **Perf regression baseline re-record từ CI** (R2-120a): baseline-win.json
   đo máy cũ → re-record trên runner GitHub để preflight perf không đớp
   nhiễu +16% vô hại.

### D. Trải nghiệm văn phòng (persona tiếp theo)
10. **Macro thích ứng dấu thanh kiểu mới/cũ** — macro `HD` hiện ra HỢP ĐỒNG
    theo dấu trong macro; thêm khả năng macro theo diacritic_style config.
11. **Tab hoàn tất cụm nhiều từ có dấu cuối** — "kính gữi" → Tab → "kính
    gửi quý khách hàng" (hiện chỉ phrase prediction trên từ đúng); mở rộng
    fuzzy-match 1 lỗi chính tả trong key (Levenshtein ≤1 trên từ cuối).
12. **Gõ tắt ngày/tháng tự động** — `=ngay` → `06/11/2026`, `=thang` →
    `11/2026` (từ giờ hệ thống) cho kế toán/hợp đồng. Chi phí thấp
    (number_vn mở rộng), giá trị cao persona kế toán.

### E. Chất lượng nội bộ
13. **Corpus generator một nguồn**: gen-vocab/gen-*-corpus chung trait output
    + kiểm checksum trong repo — chống lệch tay (R2-120e bài học).
14. **IME conflict E2E tự động**: Appium/WinAppDriver chạy MobaXterm thật
    gõ 50 từ → assert caret + text terminal đúng (hiện chỉ có replay logic).
15. **Fuzz parse config/style**: cargo-fuzz target cho `parse_config` +
    `StyleProfile::sanitize` (chặn JSON độc hại 16KB-cap).

## Thứ tự đề xuất (nếu duyệt)
1. A1 + A2 (UI + persist P7) — hoàn tất tính năng đã cam kết.
2. B4 (frequency rank) + D12 (macro ngày) — giá trị user cao, chi phí thấp.
3. E15 (fuzz) + C8 (dead-key disarm app-switch) — chặn tái diễn.
4. B5 (context-aware) — nâng cấp lớn, cần design doc riêng.
