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
| Số case | **1.475** (45 có lỗi · 1.430 câu sạch) | ≥ 60 câu | ✅ |
| **recall** | **100,0 %** (51/51 loại lỗi mong đợi) | ≥ 80 % | ✅ |
| **precision** | **100,0 %** (51/51 lỗi phát hiện đều đúng) | — | ✅ |
| **FPR** | **0,0 %** (0/1.430 câu sạch bị báo nhầm) | ≤ 2 % | ✅ |

Lệnh tái lập: `cargo run -p textvn-cli -- assist score --corpus corpus/assist`

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

## 5. Việc còn lại để "chất lượng tốt nhất" (P4-4)

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
