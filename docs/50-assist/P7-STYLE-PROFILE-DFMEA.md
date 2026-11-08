<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# P7 — User Style Profile: DFMEA + DRBFM (vòng 54)

> Tính năng: học từ/cặp sửa-lỗi **cá nhân** (từ 0.2.74 **bật mặc định**, tắt/xóa bằng nút trong
> Cài đặt; trước đó opt-in tắt mặc định) dùng cho
> Text Prediction (boost) và Trợ lý ngữ pháp (correction ưu tiên). Dữ liệu
> lưu trong `config.v1.json` có **cap chặt** (chủ repo: "style lưu không được
> phép quá lớn"), kết hợp từ điển cá nhân `english_words` hiện có.
> Luật nền: **G8** — không lưu/log nội dung phím/đoạn văn; chỉ từ đơn + số
> đếm + cặp sai→đúng tối thiểu. Tài liệu theo khung DFMEA/DRBFM (AIAG-VDA).

## 0. Thiết kế tóm tắt (Design Intent)

| Thiết kế con | Ý định | Giới hạn |
|---|---|---|
| `style_profile` trong `config.v1.json` | `enabled: false` (mặc định), `words: [{w, n}]`, `corrections: [{from, to}]` | `words ≤ 200`, `corrections ≤ 50`, mỗi từ ≤ 32 ký tự, tổng bytes của 2 mảng ≤ 16 KB (kiểm lúc load; vượt → prune giữ top `n` rồi vẫn ≤ cap, không fail-open tắt cả tính năng) |
| Học từ | đếm từ user **gõ xong + nhả từ** (FLAG_WORD_END) trong engine, chỉ khi `enabled` | chỉ từ hợp lệ (validator), ≥2 ký tự, không phải paste (paste-detector đã có), KHÔNG ghi nội dung câu (G8) |
| Học correction | khi user **chấp nhận** suggestion/fix tường minh (nút Sửa / Apply) — tín hiệu tường minh, không đoán | cặp `from→to` nguyên tắc 1-1, `from ≠ to`, ≤ 32 ký tự |
| Prediction boost | từ cá nhân đẩy weight trước nguồn chung | KHÔNG phá luật ext-rank vòng 52 (âm-đầu-khớp vẫn đứng đầu) |
| Assist correction | cặp cá nhân tra **trước** `OFFICE_BIGRAMS` | chỉ câu đúng ngôn ngữ của cặp (guard VI/ASCII hiện có) |
| Sync | 6 điểm config bắt buộc + engine đọc qua `EngineOptions` | reload giữa từ đã có luật (macOS B3) |

## 1. DFMEA (Design Failure Mode and Effects Analysis)

S/O/D thang 1-10; RPN = S×O×D. RPN ≥ 100 → action bắt buộc có gate test.

| # | Item | Failure Mode | Effect | S | Cause | O | Controls hiện có | D | RPN | Action + Gate test |
|---|---|---|---|---|---|---|---|---|---|---|
| F1 | `style_profile` bytes | Config phình (không cap) | IPC buffer/tệp config lớn, reload chậm | 7 | thiếu kiểm size lúc load/ghi | 4 | cap 200/50 + 16 KB kiểm lúc load (P7) | 3 | 84 | Load-time check + prune; test: config 1 MB words → load OK ≤16 KB giữ top |
| F2 | `words[].w` | Từ lạ/điều khiển/UTF-8 dở lưu vào | Prediction chèn rác vào văn bản | 9 | thiếu validator lúc load | 4 | validator từ (≤32, alphabetic+space, không điều khiển) | 3 | 108 | **Gate:** `style_profile_validator` — 40 input xấu → 0 mục sống |
| F3 | `corrections[].from` | Cặp trùng cặp dictionary chung | Double-apply / thứ tự sai | 8 | personal tra trước OFFICE_BIGRAMS mà không dedup | 3 | dedup khi load: cặp cá nhân trùng chung → cá nhân thắng, ghi đè 1 lần | 3 | 72 | **Gate:** test correction ưu tiên + không double-report |
| F4 | Migration | Config cũ không có `style_profile` | serde fail → config chết | 9 | thiếu `#[serde(default)]` | 2 | `#[serde(default)]` (mẫu `english_words`) + test default | 2 | 36 | **Gate:** `testDefaultConfig` Swift + `test_settings_model.c` assertion |
| F5 | Sync 3 OS | Lost update khi tray + IMK ghi đồng thời | Mất corrections user | 7 | ghi config không khoá | 2 | ghi config một nơi (tray/panel ghi; adapter chỉ reload-đọc) + merge theo trường | 4 | 56 | Ghi đúng một chủ (documented); DRBFM R6 |
| F6 | G8 | Lưu cụm/đoạn văn thay vì từ | Rò nội dung người dùng vào tệp | 10 | hook học nhầm ở câu, paste | 3 | chỉ từ đơn qua word-end; paste-detector chặn; cap 32 ký tự | 3 | 90 | **Gate:** test học không bao giờ lưu chuỗi có dấu cách/`;{}`; audit G8 rà |
| F7 | Prediction boost | Boost bậy làm bêu chữ sai (FPR) | Suggestion sai thứ tự phá UX | 7 | personal weight > luật ext-rank | 3 | personal đẩy weight 95 nhưng ext-rank vẫn ưu tiên; chỉ prefix khớp | 3 | 63 | **Gate:** "tiến" + personal "tiếng" → vẫn "tiến độ" đầu |
| F8 | Assist correction | Correction cá nhân bêu chữ đúng (FPR) | Tự sửa sai người dùng | 9 | user từng chấp nhận 1 lần, ngữ cảnh khác | 3 | chỉ áp đúng cặp nguyên tắc; guard ngôn ngữ cặp; suggestion-first (không auto) | 3 | 81 | **Gate:** correction cá nhân là suggestion-only, không auto-fix |
| F9 | Cap pruning | Prune xóa nhầm từ user vừa thêm | Tính năng "quên" | 5 | giữ top-n theo count, tie mới nhất | 4 | tie theo lần-dùng-gần (timestamp tương đối) | 3 | 60 | **Gate:** test prune giữ từ mới dùng > từ cũ cùng count |
| F10 | ConfigReload | Reload giữa từ đang gõ | Mất chữ (luật macOS B3) | 8 | cache style rebuild trong key path | 2 | style build OnceLock-like per reload, ngoài key path; reload giữ buffer câu | 3 | 48 | **Gate:** e2e reload giữa từ (đã có luật) + style reload không đổi hành vi từ đang gõ |
| F11 | Trie/lookup | Correction lookup O(n²) mỗi câu | Lag gõ (perf spec p99 10ms) | 7 | HashMap per-call | 3 | HashMap build 1 lần per load (OnceLock-style) | 3 | 63 | **Gate:** perf test correction lookup < 1µs/câu |
| F12 | Paste detector | Paste dài xả cap từ giả | Từ thật bị prune | 6 | paste đếm như gõ | 3 | paste-detector đã có (R5) — bỏ qua khi paste | 4 | 72 | **Gate:** test paste 10k từ không đổi style words |
| F13 | Encoding config | Config UTF-8 BOM/NFD từ cá nhân | Khớp không được / doubles | 5 | so khớp byte | 4 | chuẩn hóa NFC-equivalent qua fold hiện có (strip_diacritics cho key, giữ hiển thị) | 3 | 60 | **Gate:** test từ NFD vẫn khớp key deaccent |
| F14 | IME_MAX_TEXT | Boost gợi ý dài > 64 | Chèn fail-open mất gợi ý | 4 | cap từ ≤32 đã chặn | 1 | cap 32 | 2 | 8 | (đủ) |
| F15 | ABI | Thêm field vào ime_result/config struct C | Break ABI 532 bytes | 9 | đổi struct C | 1 | style không vào ime_result_v1 — chỉ vào config JSON string (ABI không đổi) | 2 | 18 | **Gate:** `abi_constants` verify vẫn 532/20/532 |
| F16 | Schema | schema.json thiếu style_profile | doctor/schema-check đỏ | 5 | quên sync schema | 3 | sync 6 điểm + preflight gate | 2 | 30 | preflight `check-version-sync`-style: thêm schema assertion |
| F17 | Serialization Swift/C | `Codable` Swift / struct C thiếu trường | Reload mất style trên 1 OS | 8 | sync 6 điểm sót | 3 | checklist 6 điểm + tests Swift/C | 3 | 72 | **Gate:** Swift test + C test decode style_profile |
| F18 | Kích thước từ | Từ 32 ký tự (ghép dài) hợp lệ nhưng khó dùng | Cap nhanh đầy | 3 | chấp nhận | 5 | cap 32 + tổng 16KB | 2 | 30 | (đủ) |

## 2. DRBFM (Design Review Based on Failure Mode)

| Thiết kế con | Giả định nền | Điểm đổi mới | Kết quả review | Xác minh bắt buộc |
|---|---|---|---|---|
| Schema config | serde `default` fieldwise đã dùng cho `english_words` | mảng object mới `{w, n}` / `{from, to}` | OK — an toàn migration | test default + config cũ load được |
| Cap & pruning | list top-N theo count | tie-break theo lần dùng gần | OK | test prune giữ mới; bytes ≤ 16 KB sau prune |
| Học từ word-end | engine đã có FLAG_WORD_END; paste-detector có sẵn | đếm tăng khi word end + enabled | Concern: paste-detector phủ đủ các OS? | e2e: paste 10k → words không đổi (3 OS replay) |
| Học correction | panel/apply có tín hiệu tường minh | API learn-correction qua tray | Concern: luồng accept phải một đường (tránh double-count) | test accept 1 lần = +1 count |
| Inject prediction | predict_prefix_candidates pure + danh sách static | thêm nguồn personal (weight 95) | OK với ext-rank | test thứ tự: ext > personal > compound |
| Inject assist | collocation_typo tra bảng tĩnh | personal trước bảng chung | OK suggestion-only | test suggestion-only + no double-report |
| Sync 6 điểm | các vòng trước đã luân phiên đủ 6 | thêm 6 điểm cho style | OK theo checklist | Swift/C test decode |
| Export style (persist) | tray là chủ ghi config | engine đếm in-memory → export khi tray hỏi | Concern: tần suất export (không spam IPC) | test export gộp mỗi 60s/khi idle |

## 3. Quyết định mở cho chủ repo (câu hỏi)

1. Tần suất export style từ engine → tray: mỗi 60 s khi idle (đề xuất) hay chỉ khi thoát app?
2. Corrections cá nhân: suggestion-only (đề xuất — F8) hay cho phép auto-fix khi count ≥ 5?
3. Có cần UI quản lý (xem/xóa từ cá nhân) ngay trong 0.2.64 hay để file config thủ công trước?

## 4. Đề xuất tối ưu thu được khi audit (thêm theo yêu cầu chủ repo)

- `predict_prefix_candidates` bucket byte-đầu đã áp vòng 53 — personal words chỉ ≤ 200 mục: quét tuyến tính personal đủ nhanh, KHÔNG cần bucket riêng (tránh over-engineering).
- Cặp correction cá nhân build HashMap 1 lần per-load — trùng mô hình `static_set_lower` đã có trong restore_en.rs (tái dùng pattern, 0 phụ thuộc mới).
- Tránh đụng `ime_result_v1` (ABI 532) — style chỉ chảy qua config JSON: zero ABI risk.
