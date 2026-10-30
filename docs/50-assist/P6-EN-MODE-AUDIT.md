# P6-EN-MODE-AUDIT — Trợ lý & dự đoán tiếng Anh: audit + đề xuất (vòng 42)

> 2026-10-31 · Expert auditor + SA view · Bối cảnh: user báo "mode tiếng Anh
> prediction và auto sửa lỗi/gợi ý không chạy ổn định hoặc không chạy hẳn".

## 1. Bản đồ đường tiếng Anh (đã trace từng nhánh)

| Tính năng | Đường đi | Điều kiện chạy |
|---|---|---|
| Ghost text EN | adapter → `ime_peek_completion` → `peek_completion` → `probe.key(Tab)` | VI bật + `suggest.enabled` = true (opt-in) |
| Tab hoàn tất EN | `key(Tab)` → `try_english_complete` | VI bật + `suggest.enabled` (từ chưa fold) hoặc từ đang fold |
| Space hoàn tất EN | `try_english_complete(space)` | VI bật + `suggest.trigger = tab_and_space` |
| Trợ lý ngữ pháp EN (confusions) | trigger space/,/. → `ime_assist_check_v1` → `check_confusions` | **mọi ngôn ngữ** (vòng 42 — trước đây En-only) |
| Trợ lý SVA/giới từ EN | `check_en_agreement` | **mọi ngôn ngữ** (vòng 42 — trước đây En-only) |
| Correlative EN (Although…but) | `check_en` | En-dominant (giữ — cần ngữ cảnh mệnh đề EN trọn) |
| Đại từ "I" thường | `check_pronoun` | En-dominant (giữ — "i" là âm tiết Việt ASCII) |
| Chính tả EN (OS spell) | `en_spelling` + WinSpell/MacSpell | En-dominant + OS provider khoẻ (Windows: retry 30 s — vòng 42) |

## 2. Root cause đã fix (vòng 42)

- **RC-3** — câu mix Vi-dominant (≥2 từ dấu) bỏ qua TOÀN BỘ luật EN:
  "Anh should of done it sớm hơn", "He don't know nhé anh",
  "Em cần discuss about kế hoạch" đều SẠCH. Fix: `check_confusions` +
  `check_en_agreement` (cặp ASCII đóng, tự-guard) chạy mọi ngôn ngữ; giữ
  En-only cho pronoun "i" + correlative + en_spelling.
- **RC-4** — WinSpell probe quá hạn 150 ms MỘT lần = `OnceLock` cache `None`
  **cả phiên** (máy tải nặng là đủ): spell-assist tắt vĩnh viễn tới khi
  restart. Fix: retry mỗi 30 s (`SpellAttempt` + Mutex poison-safe).
- **PERF** — sau khi mở gate, vòng SVA O(n²) trên câu dài (10k từ = 10 s):
  hoist HashMap start→idx, về <3 s cả suite abnormal.
- **RC-1 (một phần)** — VI tắt = strategy Passthrough = **prediction/ghost/
  Tab EN chết hoàn toàn** (theo thiết kế: Tab là phím cấu trúc của app;
  buffer từ không được track trong Passthrough). Assist (auto-fix) VẪN chạy
  khi VI tắt ✓ (`check_sentence` không gate VI, `record()` vẫn feed câu).
  Phần prediction VI-off: xem đề xuất P1.

## 3. Đề xuất (cần quyết định PO — không tự ý đổi)

- **P1 — EN prediction khi VI tắt**: track buffer từ trong Passthrough
  (feed `word.raw` từ `note_pass`, không secure) + hoàn tất **chỉ qua
  Ctrl+Space** (Tab phải giữ nguyên cho indent/điều hướng app) + ghost preview
  theo `suggest.enabled`. Chi phí: config mới (`suggest.when_vi_off`?) —
  đồng bộ 15 site theo luật default. Khuyến nghị làm vòng 43 sau khi PO chốt.
- **P2 — modal + V2/V3** ("will sent" → "will send"): cần bảng động từ bất
  quy tắc đóng (~120 cặp) — recall thực tế cao trong email. Danh sách đóng,
  FPR thấp, làm được ngay khi có ngân sách test.
- **P3 — discoverability của `suggest.enabled`** (mặc định TẮT): phần lớn
  report "prediction không chạy" rơi vào đây. Đề xuất: bật ON cho cài mới
  (sync 15 site + test Swift/C) HOẶC hint trong tray settings lần đầu.
- **P4 — backlog Pass C vòng 41**: appStates cap parity Windows, pipe header
  deadline, SwiftUI duplicate-id.

## 4. Kiểm chứng

- 137 assist lib + 43 abnormal tests xanh; FPR corpus 0/2739 + tatoeba
  rule-touched 0 FP mới; recall 28/28 + 38/38 giữ nguyên.
- Probe CLI: "Anh should of done it sớm hơn." → word_confusion ✓;
  "He don't know nhé anh." → word_confusion ✓; "Tôi is học sinh." → SẠCH
  (đúng — "tôi" không thuộc cặp đại từ EN); "gửi cho i nhé" → không bêu.
