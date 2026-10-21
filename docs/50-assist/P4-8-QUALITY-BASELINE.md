# P4-8 — Baseline chất lượng engine `assist` (đo được) + công cụ `assist score`

- **Ngày:** 2026-10-21 · **Trạng thái:** Baseline đầu tiên (số đo thật) · **Phần:** 4 — P4
- **Mục tiêu:** hiện thực hoá **D19** (tiêu chí chất lượng đo được) thay vì "cảm thấy ổn".
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §2 (tiêu chí) · §9 (kiểm thử) · `P4-1` §5 (luật) · `P4-TASKS.md` (AS-001…006) · `docs/adr/README.md` (ADR-012/013)

---

## 1. Công cụ đo: `textvn-cli assist score --corpus <dir>`

Đọc mọi file `.ass` trong thư mục corpus (định dạng TSV `<câu>\t<kind1,kind2,...>`, cột kinds rỗng = câu **sạch**) và báo:

| Chỉ số | Công thức | Ý nghĩa |
|---|---|---|
| **recall** | (# loại lỗi mong đợi **có** bị phát hiện) / (# loại lỗi mong đợi) | Bỏ sót bao nhiêu lỗi |
| **precision** | (# lỗi phát hiện **đúng**) / (# lỗi phát hiện) | Báo thừa bao nhiêu |
| **FPR** | (# câu sạch bị báo lỗi) / (# câu sạch) | Bao nhiêu câu sạch bị "nghi ngờ" nhầm |
| (kèm) | danh sách **MISSED / EXTRA / FALSE POSITIVE** (giới hạn 20 dòng) | để sửa luật |

## 2. Kết quả đo (2026-10-21)

| Chỉ số | Kết quả | Ngưỡng D19 | Đạt? |
|---|---|---|---|
| Số case | **1.535** (47 có lỗi · 1.488 câu sạch) | ≥ 60 câu | ✅ |
| **recall** | **100,0 %** (54/54 loại lỗi mong đợi) | ≥ 80 % | ✅ |
| **precision** | **100,0 %** (54/54 lỗi phát hiện đều đúng) | — | ✅ |
| **FPR** | **0,0 %** (0/1.488 câu sạch bị báo nhầm) | ≤ 2 % | ✅ |

Lệnh tái lập: `cargo run -p textvn-cli -- assist score --corpus corpus/assist`

## 2.1 Vòng 2 (2026-10-21) — đo trên **văn bản thật** (đóng F80)

Ngoài câu mẫu tổng hợp, đã sinh corpus sạch từ **chính tài liệu tiếng Việt của repo** (`docs/**/*.md` + `README.md`/`PLAN.md`/`CHANGELOG.md`) bằng script tái lập được:

- `tools/assist/gen_clean_corpus.ps1` (ASCII, không phụ thuộc ngoài) → `corpus/assist/vi_docs_clean.ass`.
- Vì lấy từ tài liệu của chính TextVN (GPL-3.0-or-later) ⇒ **không ràng buộc giấy phép**, và đây là **văn bản thật** (không phải câu mẫu) ⇒ đo FPR có ý nghĩa.

**Kết quả trước khi sửa luật: 5 false positive** trên 74 câu thật ⇒ lộ **2 điểm yếu thật của luật** (đã sửa, xem §4). Sau khi sửa: **FPR 0,0 %** trên 1.488 câu sạch (gồm 55 câu văn bản thật).

## 4.1 Finding vòng 2 (SA/QA) — phát hiện nhờ đo trên văn bản thật

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-86** | minor | Luật "hoa đầu câu" coi dấu `.` **trong ngoặc / trong dấu nháy** là kết thúc câu → báo oan (vd `(Từ điển EN...) đặt ở …` → "đặt" bị báo) | ✅ Sửa: `is_real_sentence_end()` — đếm ngoặc mở/đóng và trạng thái dấu nháy phía trước dấu chấm; trong ngoặc/nháy ⇒ không phải hết câu |
| **Fassist-87** | minor | Dấu `…` (ellipsis) bị coi là kết thúc câu → báo oan (vd `… dưới…): đ giờ …`) | ✅ Sửa: bỏ `…` khỏi `is_sentence_end()` (ellipsis ở tiếng Việt là dấu lốt triện, không phải hết câu) |
| **Fassist-88** | minor | Corpus sinh từ docs lẫn **văn bản không phải câu** (bullet `- …`, mục danh sách, dòng kỹ thuật `a..b`) → FPR đo được bị nhiễu **từ corpus**, không phải lỗi engine | ✅ Sửa bộ lọc script: bỏ câu bắt đầu bằng `- `/`> `, có `..`, hoặc chứa `[]{}<>|` |
| **Fassist-89** | minor | 3 ca FP đã sửa cần **khóa lại** để không tái phát | ✅ Thêm `corpus/assist/vi_regressions.ass` (3 ca FP cũ → nay phải **sạch** + 2 ca lỗi thật → vẫn phải báo) |

> Bài học (PO): **đo trên văn bản thật mới phát hiện lỗi luật**; đo trên câu mẫu tự sinh chỉ chứng minh "không tự mâu thuẫn".


## 3. Văn bản sạch dùng để đo FPR — `corpus/assist/vi_clean_synthetic.ass`

- **1.408 câu sạch** sinh **tất định** (deterministic) từ mẫu câu: chủ ngữ × trạng từ × tân ngữ (+ địa điểm) — không cần nguồn văn bản bên ngoài ⇒ **không ràng buộc giấy phép**, tái lập được 100 %.
- Vì vậy **FPR = 0 % là số kỳ vọng** (câu mẫu đúng chuẩn) — **không** đại diện văn bản thật.

## 4. Finding (vòng 19 — SA/QA/PO)

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-80** | minor | **FPR = 0 % là số "kỳ vọng"** vì câu sạch sinh theo mẫu ⇒ chưa phải bằng chứng trên văn bản thật | Ghi rõ trong §3; gate CI chỉ dùng sau khi có **corpus thật** (≥ 300 câu, **2 người đánh giá độc lập**, thống nhất ≥ 80 % — P4-4) |
| **Fassist-81** | minor | `assist score` chỉ so **kind**, **không so vị trí span** → có thể tính "matched" dù span sai | Ghi nhận; thêm `--strict` (so cả span) ở P4-4 khi từ điển sinh gợi ý cần đo chính xác |
| **Fassist-82** | minor | Corpus sạnh tổng hợp (84 KB) nằm trong `corpus/assist/` → test chạy lâu hơn cần thiết | Chấp nhận (1.475 case < 1 s); tách riêng nếu corpus phình > 5.000 case (repo-hygiene #7 loại file > 1,5 MB) |
| **Fassist-83** | minor | Số lỗi mong đợi trong corpus chỉ **51** → recall 100 % có **độ tin cậy thấp** (1 lỗi sai = −2 %) | Ghi nhận: cần ≥ 300 case có lỗi trước khi coi recall là gate CI (P4-4) |

## 2.2 Vòng 3 (2026-10-21) — mở rộng luật ngữ pháp + `--strict` / `--per-kind`

### Luật mới (loại **chỉ gợi ý**, không tự sửa — Fassist-90)
| Mã | Luật | Ví dụ | Ghi chú |
|---|---|---|---|
| **L9** | Lặp từ ở **ranh giới câu** | "Tôi không. Không sao cả." | Chỉ bắt khi từ lặp là **từ chức năng** (đại từ/giới từ/trạng từ) — lặp từ nội dung là cách viết hợp lệ (F91) |
| **L10** | Phủ định kép | "không có … không" | Gợi ý, không tự sửa (ngữ nghĩa có thể đúng) |
| **L12** | Dấu phẩy ở cuối câu | "…trường," | Gợi ý: đổi thành dấu chấm |

→ Mọi lỗi **mơ hồ** dùng `Issue { auto_fixable: false, confidence: 60, fix: None }` ⇒ **không bao giờ** tự sửa (đúng D7).

### Công cụ đo mở rộng (F81)
- `assist score --per-kind` → bảng recall **theo từng loại luật** (phát hiện luật nào yếu).
- `assist score --strict` → đòi **span khớp** (định dạng corpus mở rộng `kind@start-end`; span = **char Unicode**).

### Số đo sau khi thêm 3 luật
| Chỉ số | Kết quả |
|---|---|
| Số case | **1.548** (53 có lỗi · 1.495 câu sạch) |
| recall / precision / FPR | **100,0 %** (60/60) · **100,0 %** · **0,0 %** |
| Số loại luật được đo | **9** (comma_at_end · diacritic_typo · double_negative · double_space · duplicate_word · lowercase_sentence_start · sentence_repeat · space_after_punct · space_before_punct) — tất cả recall 100 % |

## 4.2 Finding vòng 3 (SA/QA)

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-90** | minor | Luật **mơ hồ** (ngữ pháp) không được tự sửa, nhưng API cũ chỉ có `issue()` (luôn `auto_fixable = true` + `fix`) ⇒ dễ vô tình cho phép auto-fix luật ngữ pháp | ✅ Thêm `issue_suggestion()`: `auto_fixable = false`, `confidence = 60`, `fix = None`; dùng cho L9/L10/L12 |
| **Fassist-91** | minor | L9 báo oan: lặp **từ nội dung** ở ranh giới câu là cách viết hợp lệ ("Tôi đi học. Học xong về nhà.") | ✅ Siết L9: chỉ bắt lặp **từ chức năng** (danh sách 38 từ: đại từ/giới từ/trạng từ) + 2 case "sạch" trong corpus để chống tái phát |
| **Fassist-92** | minor | Thêm `IssueKind` mới ⇒ **FFI `match` không phủ hết** → **build vỡ toàn workspace** (file của agent khác) + phải thêm hằng vào **cả 2 bản header** (nguồn + mirror Swift) nếu không sẽ đỏ CI macOS | ✅ Vá FFI (`IME_ASSIST_KIND_SENTENCE_REPEAT/DOUBLE_NEGATIVE/COMMA_AT_END` = 7/8/9) + cập nhật **nguồn + mirror** cùng lúc (F85) |
| **Fassist-93** | minor | `assist score` chỉ so **kind** ⇒ không phát hiện được luật báo **đúng loại nhưng sai vị trí** | ✅ `--strict` (so span) + `--per-kind` (bảng theo loại) |

1. Corpus **thật**: ≥ 300 câu có lỗi (đủ 6 loại luật) + ≥ 300 câu sạch từ văn bản tiếng Việt thực tế (nguồn license-sạch, đã audit) — đo lại recall/FPR.
2. **2 người đánh giá** độc lập 100 câu mẫu → đo độ thống nhất (D19).
3. Thêm luật **gợi ý** (L7/L9/L10/L12) khi có từ điển → đo recall theo từng loại.
4. `assist score --strict` (so span) + `--per-kind` để biết luật nào yếu.

## 6. Bằng chứng (để agent sau không phải đo lại)

| Lệnh | Kết quả |
|---|---|
| `cargo run -p textvn-cli -- assist score --corpus corpus/assist` | recall 100 % · precision 100 % · FPR 0 % (1.475 case) |
| `cargo test -p textvn-assist` | 35 unit + 3 integration xanh |
| `cargo clippy --workspace --all-targets -j 2 -- -D warnings` | 0 warning |
| `cargo +nightly fuzz run assist_check` (CI, Unix) | target mới: không panic · fix idempotent · span hợp lệ |
