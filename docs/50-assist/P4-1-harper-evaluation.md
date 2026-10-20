# P4-1 — Đánh giá Harper (Automattic/harper) + chọn engine ngữ pháp cho TextVN

- **Ngày:** 2026-10-20 (khảo sát bằng API GitHub/crates.io + raw master cùng ngày) · **Trạng thái:** Vòng 1 review xong
- **Phần:** 4 — Trợ lý ngữ pháp (P4) · **Owner:** agent · **Review:** xem `P4-0-MASTER-PLAN.md` (kế hoạch thi công)
- **Cross-ref:** `PLAN.md` §2.3 (S1 chính tả offline + gạch đỏ · S2 gợi ý từ) · `docs/specs/antivirus-false-positive.md` §8 (chính sách kiến trúc hành vi) · `docs/10-shared/P0-2-engine-ffi-contract.md` · `docs/10-shared/P0-3-config-preset-strategy.md` · `docs/01-AGENT-HANDBOOK.md` (DoD 7 mục)
- **Yêu cầu gốc (user 2026-10-20):** "2 chế độ hỗ trợ chức năng giống Harper/Grammarly — không chỉ correct từ, mà **highlight lỗi ngữ pháp** và **correct cả câu** sau khi bấm dấu cách hoặc `,` + `.`"

---

## 0. Quyết định (TL;DR)

| # | Quyết định | Lý do ngắn |
|---|---|---|
| **D1** | **Không vendor / không phụ thuộc `harper-core`** | Harper **chỉ hỗ trợ tiếng Anh** và crates.io đã đứng yên ~27 tháng; master API đổi rất nhanh (repo push 2026-10-20) |
| **D2** | **Mượn kiến trúc Harper, tự viết luật tiếng Việt** (`Document`/`Token`/`Span`/`Lint`/`LintGroup` + DSL pattern) | Kiến trúc đúng cho "grammar checker tối giản, offline, µs-ms"; phần ngôn ngữ là 90% chi phí và Harper không cho mượn được |
| **D3** | **Tự xây từ điển tiếng Việt từ nguồn mở, kiểm tra license từng nguồn trước khi dùng** | Không có từ điển VN mở nào "đóng gói sẵn + giấy phép sạch" mà ta tin được; đây là việc làm được |
| **D4** | **Highlight = overlay cạnh con trỏ**, không gạch chân thật trong field của app | Gạch chân thật phải sửa character format của app → hỏng style tài liệu, chỉ làm được ở RichEdit/Word, không đồng nhất 3 OS |
| **D5** | **Thứ tự: Mode 1 (gợi ý/highlight) → Mode 2 (tự sửa cả câu)** | Auto-sửa text người dùng là hành động phá huỷ niềm tin nếu sai; dựng "gợi ý" trước để có corpus + feedback thật |

> D1–D5 đã được user chốt ngày 2026-10-20 (4 câu hỏi phạm vi, trả lời theo khuyến nghị).

---

## 1. Harper là gì — facts (đã xác minh, không suy đoán)

| Mục | Giá trị | Nguồn |
|---|---|---|
| Repo | `Automattic/harper` (Rust, default branch `master`) | GitHub API 2026-10-20 |
| Giấy phép repo | **Apache-2.0** | GitHub API `license.spdx_id` |
| Độ nổi tiếng | 16.879 ★ · 876 fork · 1.015 issue đang mở | GitHub API |
| Lịch sử | tạo 2023-10-22 · push gần nhất 2026-10-20 (rất active) | GitHub API |
| Ngôn ngữ hỗ trợ | **chỉ tiếng Anh** — README: *"Harper currently only supports English, but the core is extensible to support other languages"* | `README.md` (master) |
| Triết lý | offline, privacy-first; "milliseconds to lint"; memory < 1/50 LanguageTool; chạy được qua WebAssembly | `README.md` |
| crates.io | `harper-core` **0.8.7**, publish 2024-07-06 → **stale ~27 tháng**; bản 0.7.0+ khai báo `Apache-2.0` (0.6.x trở về trước: `MIT OR Apache-2.0`) | crates.io API |
| Topic | `grammar-checker`, `english-language`, `wasm`, `lsp` | GitHub API |

**Nhận xét Audit:** Apache-2.0 **tương thích một chiều với GPL-3** → dùng làm dependency hay vendor đều hợp pháp, nhưng bắt buộc giữ attribution (NOTICE/REUSE) nếu lấy code. Vì ta **không lấy code** (chỉ mượn ý tưởng kiến trúc) → **không phát sinh nghĩa vụ license**.

---

## 2. Kiến trúc `harper-core` (master, 2026-10-20) — bản đồ module

| Module | Vai trò | Ánh xạ sang TextVN (`assist/`) |
|---|---|---|
| `document.rs` (45 KB) | `Document`: nguồn text → token + `Span` char-offset | **Giữ nguyên ý tưởng**: `Sentence { text, tokens, spans }` |
| `token.rs`, `token_kind.rs` | `Token`/`TokenKind` (word, punctuation, number, …) | `Token { kind, range }` + `TokenKind` cho tiếng Việt (âm tiết, từ) |
| `span.rs` | `Span` char offset (start/end) | Giữ (cần map offset → vị trí thật trong field) |
| `lexing/` | Tách text → token (kèm rule tiếng Anh: contraction, hyphen, quote) | Tách theo **âm tiết tiếng Việt** (không cần ngữ pháp) |
| `linting/` | **hàng trăm file, 1 file = 1 luật** (`an_a.rs`, `amounts_for.rs`, `arrive_to.rs`, …) | Mỗi luật VN = 1 file trong `assist/src/rules/` (cùng nguyên tắc) |
| `expr/` | DSL pattern: `SequenceExpr`, `AnchorStart`… (khai báo lỗi theo pattern token) | **Mượn nguyên tắc** (không copy code): rule = pattern → lint |
| `parsers/` | Noun phrase, v.v. (POS-ish heuristics) | V1 không cần (tiếng Việt POS khó) |
| `spell/` + `weir/` + `weirpack/` | `FstDictionary` (từ điển FST + curated set) | Từ điển âm tiết VN → FST/bloom tương tự |
| `dict_word_metadata*.rs` | Metadata từ (noun/verb/adverb, degree, dialect) | V1: bỏ; V2: metadata hạn từ |
| `mask/` | `Masker`/`RegexMasker`: che URL, code, email | **Rất giá trị cho IME** — không kiểm tra URL/email/code |
| `ignored_lints/` | `IgnoredLints` (danh sách lint bị bỏ qua) | Tương đương "danh sách bỏ qua" của TextVN (`P1-3` R6/R8) |
| `language_detection.rs` | Phát hiện "có phải tiếng Anh không" để bỏ qua | Tương đương: chọn `assist.language = vi|en|auto` |
| `title_case.rs`, `case.rs` | Case title/…) | Bỏ (tiếng Việt không phân biệt hoa thường như tiếng Anh) |
| `indefinite_article.rs`, `irregular_*.rs`, `regular_nouns.rs` | a/an, động từ bất quy tắc, số nhiều | **Không dùng được** (hình thái tiếng Anh) |

**Kết luận kiến trúc:** Harper chia 3 tầng rõ ràng — *tokenize → rule/pattern → lint+fix*. Tầng 1 và 3 **chuyển ngữ được**; tầng 2 (luật) và từ điển là **100% tiếng Anh**.

---

## 3. Khoảng cách tiếng Việt — bảng "mượn được gì"

| Tầng | Harper (EN) | Tiếng Việt cần làm | Khó |
|---|---|---|---|
| Tokenize | tách từ theo khoảng trắng + contraction | tách **âm tiết** (tự động từ chuỗi ký tự có dấu) | 🟢 Trung bình |
| Dấu câu/câu | `Punctuation`, sentence split | tương đương (dấu câu VN giống EN) | 🟢 Dễ |
| Từ điển | `FstDictionary` hunspell-EN (~100k từ) | từ điển âm tiết VN (~60–100k âm tiết) + tần suất | 🟡 Trung bình (cần nguồn + license) |
| Luật ngữ pháp | hàng trăm luật EN (a/an, there/their, tense…) | ~12–20 luật VN đầu tiên: lặp từ, khoảng trắng, hoa đầu câu, trùng giới từ, phủ định kép… | 🟡 Trung bình (ngôn ngữ học) |
| Ngữ pháp nâng cao | POS + noun phrase | **không làm v1** (cần POS tiếng Việt) | 🔴 Cao |
| Gợi ý từ | edit distance + frequency | edit distance trên **âm tiết** + tần suất | 🟡 Trung bình |
| Masking | URL/code/email | giữ nguyên cách làm | 🟢 Dễ |

**Ước lượng SA:** luật tiếng Việt v1 (không từ điển) ≈ 1.5–2.5k LOC Rust; thêm từ điển + gợi ý từ ≈ +3–4k LOC + pipeline build từ điển. So với "vendor Harper + port" thì **rẻ hơn và không kéo theo giấy phép/dependency**.

---

## 4. Đánh giá theo vai trò chuyên trách

### 4.1 PO (Product Owner)
- **Nhu cầu thật:** người dùng VN gõ văn bản tiếng Việt hằng ngày; lỗi hay gặp là **lặp từ, sai dấu, thừa/thiếu khoảng trắng, quên dấu chấm cuối câu** — không cần "ngữ pháp học thuật" như Grammarly ngay.
- **Giá trị tối thiểu (MVP):** bắt lỗi trong **câu vừa gõ xong** (space/`,`/`.`) + gợi ý sửa 1 phím. Không cần đọc lại toàn tài liệu.
- **Rủi ro sản phẩm:** "auto-sửa" sai → mất niềm tin → bắt buộc **bật mặc định = tắt**, có **Undo**, có **thống kê số lỗi đã sửa** trong toast.
- **Kết luận:** định vị "**trợ lý ngữ pháp offline cho bộ gõ**", không cạnh tranh Grammarly về độ phủ; cạnh tranh về **offline + không rò rỉ + tiếng Việt thật**.

### 4.2 BA (Business Analyst)
- **Luồng người dùng:** bật "Trợ lý ngữ pháp" ở tray → gõ → gõ space → nếu có lỗi: overlay hiện lỗi + gợi ý → `Enter` áp dụng / `Esc` bỏ.
- **Luồng lỗi cần xử lý:** (a) app không phải ô nhập (code/terminal) → **tắt tự động**; (b) text gõ bằng tiếng Anh → bỏ qua (auto-detect); (c) app paste sẵn text → shadow buffer không khớp → **bỏ qua, không đoán**; (d) app elevated/UIPI → không can thiệp.
- **Quy tắc nghiệp vụ:** không bao giờ tự sửa khi người dùng đang gõ dở (đang compose) — chỉ sau trigger.
- **Kết luận:** có 1 luồng chính + 4 nhánh loại trừ → đủ để đặc tả và test.

### 4.3 SA (Solution Architect)
- **Điểm mấu chốt:** TSF TIP nằm trong **mọi process** → không được làm nặng trong đó. Vì vậy engine check phải **rẻ** (<1–3 ms) và **không I/O**; từ điển phải **lazy-load** và có thể nằm ở tiến trình riêng nếu nặng — nhưng tiến trình riêng = rủi ro "AV nhìn thấy tiến trình ẩn giữ text" → **ưu tiên in-process, không sinh tiến trình mới** (xem `docs/specs/antivirus-false-positive.md` §8).
- **Tích hợp:** `assist/` là crate Rust thuần (không I/O, không network) → gọi trực tiếp từ TSF/IMK/IBus adapter qua FFI hiện có (`docs/10-shared/P0-2-engine-ffi-contract.md`) → **không cần mở rộng IPC** ở v1.
- **Áp sửa:** dùng lại `strategy` crate — `ActionKind::Replace` + chiến lược `SelectionReplace` (đã có vì fix bug B1 "gợi ý trong ComboBox/SearchField") → **không phát minh cơ chế mới**.
- **Kết luận:** kiến trúc khả thi, 3 OS dùng chung 1 crate.

### 4.4 QA
- **Kiểm thử được:** corpus câu tiếng Việt + expected lint (giống `corpus/` sẵn có của repo) → deterministic.
- **Cạm bẫy QA:** "đúng/sai" ngữ pháp tiếng Việt có tính chủ quan → **chỉ tự động hoá được luật objective** (lặp từ, khoảng trắng, hoa đầu câu, dấu chấm) — luật ngữ pháp mơ hồ phải ở dạng "gợi ý", và **không được auto-fix**.
- **Perf gate:** cần gate p95 theo `PLAN.md` §5.5 (ime_key <0.5 ms, adapter+strategy <3 ms, end-to-end <10 ms, RSS <30 MB) — check ngữ pháp **không được** phá budget này.
- **Cross-OS:** cùng corpus chạy 3 OS; overlay UI cần test thủ công theo app (Notepad/Word/Chrome/VSCode...).
- **Kết luận:** có thể đưa vào CI (corpus + perf), UI cần manual checklist.

### 4.5 Audit (license / security / privacy)
- **License:** Harper Apache-2.0 — dùng làm dependency hợp pháp với GPL-3, nhưng master đổi nhanh + crates.io stale ⇒ vendor = phải **pin commit + ghi NOTICE/REUSE + không theo dõi master**. Ta **không lấy code** ⇒ không có nghĩa vụ.
- **Privacy:** engine phải **không log text** (S2) và **không network** (P2). Harper thiết kế offline → cùng hướng ✓. Nhưng nếu ta tự viết: phải đảm bảo shadow buffer chỉ tồn tại trong RAM, **không ghi log, không crash-dump text**, xoá khi đổi focus.
- **AV posture:** không sinh tiến trình ẩn giữ text người dùng; không đọc text của process khác bằng API bị cấm (`repo-hygiene` check #8).
- **Kết luận:** đường "tự viết" ít rủi ro pháp lý + AV hơn hẳn đường "vendor".

### 4.6 Design (UX)
- **Highlight:** overlay cạnh con trỏ là lựa chọn duy nhất đồng nhất 3 OS mà không sửa tài liệu người dùng.
- **Overlay phải:** không chiếm focus (WS_EX_NOACTIVATE / NSPanel non-activating), tự tắt khi click ra ngoài, không che dòng đang gõ, tương phản cao, hỗ trợ dark mode, kích thước nhỏ (≤ 320 px).
- **Phím:** `↑/↓` lỗi, `Enter` áp dụng gợi ý đầu, `Alt+1..3` chọn gợi ý, `Esc` bỏ, `Ctrl+Shift+G` gọi lại.
- **Thông điệp lỗi tiếng Việt, ngắn:** "Lặp từ: 'của của' → 'của'".
- **Kết luận:** UX gợi ý kiểu IME (candidate list) quen thuộc hơn kiểu Grammarly (sóng đỏ trong editor) — người dùng VN quen UniKey/Windows Pinyin.

---

## 5. So sánh phương án (điểm số 0–5, cao = tốt)

| Phương án | Giấy phép | Offline | Hỗ trợ tiếng Việt | Nhẹ cho IME | Rủi ro AV/privacy | Effort | Tổng |
|---|---|---|---|---|---|---|---|
| **Tự viết `assist/` (mượn kiến trúc Harper)** | GPL-3 (của ta) | ✅ | ⭐ (tự làm) | ✅ (µs–ms) | ✅ thấp | ⭐⭐⭐ | **28/30** |
| Vendor `harper-core` (git pin) + luật VN | Apache-2.0 → GPL-3 | ✅ | ❌ (Harper không có VN) | ⚠️ build nặng, thêm dep | ⚠️ thêm dep lạ | ⭐⭐⭐ | 18/30 |
| LanguageTool (JVM + n-gram) | LGPL | ⚠️ cần data | ❌ không có VN | ❌ (JVM, 16 GB n-gram) | ❌ runtime mới (vi phạm G-rule) | ⭐⭐ | 8/30 |
| Vale (Go) | MIT | ✅ | ❌ không có VN | ⚠️ runtime Go mới | ⚠️ | ⭐⭐ | 12/30 |
| `write-good` / `proselint` (JS/Python) | MIT | ✅ | ❌ | ⚠️ runtime mới | ⚠️ | ⭐ | 10/30 |
| LLM cloud (Grammarly-style) | — | ❌ mạng (vi phạm P2) | ✅ (mạnh) | ❌ (round-trip) | ❌ rò rỉ text | ⭐⭐ | 9/30 |
| LLM local (Ollama) | MIT | ✅ | ✅ | ❌ nặng, cần model | ⚠️ | ⭐⭐⭐⭐ | 16/30 |

> "LLM local" để lại là **hướng v2.x** (PLAN §2.3 `C1`), không nằm trong đường chính.

---

## 6. Điều kiện nếu sau này muốn dùng Harper cho phần **tiếng Anh**

1. Vendor **một commit cụ thể** (không theo dõi `master`) vào `third_party/harper/` — **không** đưa vào `Cargo.toml` workspace mặc định.
2. Giữ `LICENSE` + `NOTICE` của Apache-2.0; cập nhật `REUSE.toml` với `LicenseRef-Apache-2.0` (và attribution trong `docs/compliance/`).
3. Chỉ dùng cho **đoạn tiếng Anh**; lỗi tiếng Anh **không bao giờ** auto-fix (chỉ gợi ý).
4. Bọc sau interface `assist::Checker` để tắt được không ảnh hưởng luật VN.
5. Không để nó nằm trong đường gõ chính (timeout 2 ms → bỏ qua, không chặn).

---

## 7. Rủi ro bảo trì (đã biết trước)

| Rủi ro | Mức | Giảm thiểu |
|---|---|---|
| Harper đổi API nhanh (master push 2026-10-20, 1.015 issue mở) | Cao nếu vendor | Không vendor; nếu vendor thì pin commit + CI test hợp đồng API |
| crates.io `harper-core` stale 27 tháng | Trung bình | Không dùng bản crates.io |
| Từ điển VN "đúng" là bài toán mở (nguồn + license) | Trung bình | Chỉ ship từ điển **tự sinh từ nguồn đã kiểm license**, ghi rõ nguồn trong doc + file sinh ra |
| Ngôn ngữ học tiếng Việt: "lỗi" có tính chủ quan | Cao nếu làm ngữ pháp học thuật | V1 chỉ luật **objective**; phần mơ hồ = gợi ý, không auto-fix |
| Bản quyền từ điển/ngữ liệu (Wikimedia CC BY-SA có nghĩa vụ chia sẻ) | Trung bình | Audit từng nguồn trước khi đóng gói; ưu tiên nguồn permissive/ODbL-compatible |

---

## 8. Findings — vòng 1 (5W1H: "Đúng & Đủ") — `Fassist-*`

| ID | Mức độ | Finding | Bằng chứng | Trạng thái |
|---|---|---|---|---|
| **Fassist-1** | major | `harper-core` **chỉ hỗ trợ tiếng Anh** ở mọi tầng (lexer, dictionary, luật, POS) — dùng làm dependency **không** tạo ra tính năng ngữ pháp tiếng Việt mà user cần | `README.md` + `lib.rs` (`Document::new_plain_english_curated`, `FstDictionary::curated()`, `Dialect::American`) + `linting/` hàng trăm luật EN | đã xử lý → **D1/D2** |
| **Fassist-2** | major | crates.io `harper-core` **stale ~27 tháng** (0.8.7 / 2024-07-06) trong khi master tiếp tục phát triển → vendor bản crates.io = dùng code cũ; vendor master = API đổi liên tục | crates.io API (versions + timestamps) + GitHub API (`pushed_at` 2026-10-20) | đã xử lý → **D1** + §6 (điều kiện nếu vendor sau) |
| **Fassist-3** | minor | Không có từ điển tiếng Việt mở nào đóng gói sẵn với giấy phép đã kiểm → bắt buộc tự xây + audit license từng nguồn | rà soát nguồn mở trong `docs/specs/reference-parity.md` (chỉ có spec WinVNKey/EVKey, không có từ điển) | đã xử lý → **D3** + P4-0 §5 |
| **Fassist-4** | minor | "Highlight lỗi" trong app là **giới hạn kỹ thuật**, không phải tuỳ chọn UX: TSF/IMK/IBus không vẽ gạch chân vào edit control của app mà không sửa format → phải dùng overlay | `adapters/windows-tsf/src/display_attr.rs` — TIP còn **chủ động** trả `TF_LS_NONE` để app *không* gạch chân chữ đang gõ (P1-1 §7) | đã xử lý → **D4** + P4-0 §4 |
| **Fassist-5** | minor | Áp sửa "cả câu" cần **map offset → vị trí thật trong field**; app khác nhau dùng cơ chế khác nhau (ITfRange / AXRange / AT-SPI) → rủi ro sửa nhầm vị trí | `strategy/src/action.rs` (`ActionKind::Replace`) + `rules_field.rs` (`parse_field_role`) — có sẵn chiến lược `SelectionReplace` để tái dùng | đã xử lý → P4-0 §3 (giới hạn v1: chỉ câu cuối, chỉ app hỗ trợ) |
| **Fassist-6** | minor | Chi phí bảo trì của "chính tả + gạch đỏ" đã nằm sẵn trong backlog `S1`/`S2` của PLAN → không được tạo phần trùng lặp, mà **nâng cấp** S1/S2 | `PLAN.md` §2.3 (S1, S2) | đã xử lý → P4-0 §1 (mapping) |

---

## 9. Câu hỏi mở — **đã trả lời (2026-10-20)**

| # | Câu hỏi | Quyết định | Người quyết |
|---|---|---|---|
| Q1 | Tên feature hiển thị cho người dùng | **"Trợ lý ngữ pháp"** (UI 3 OS; khóa config vẫn là `assist.*`) | **User** |
| Q2 | Ngưỡng auto-fix | **Chỉ luật objective** (L1/L2/L11, L3, L4, L5, L6, L8); luật mơ hồ chỉ gợi ý, không bao giờ tự sửa; `assist.auto_fix` mặc định `off` | Expert (D7) |
| Q3 | Có cho phép đọc text của ô nhập (UIA/AX/AT-SPI)? | **Không đọc nội dung.** Chỉ dùng shadow buffer do ta tự commit. Được phép đọc **vị trí con trỏ** (metadata) khi cần precondition cho cơ chế sửa (xem `P4-0` §4.5) | Expert (D8/D12) |
| Q4 | Ngưỡng báo động giả của từ điển | Chỉ báo khi từ **không có** trong từ điển **và** không khớp từ nào (edit distance ≤ 1 âm tiết) **và** không phải tên riêng/viết tắt; đo **FPR ≤ 2%** trên corpus ≥ 500 câu sạch, recall ≥ 80% | Expert (D19) |
| Q5 | V1 có hỗ trợ tiếng Anh không? | **Không** → giữ D1 tuyệt đối (không đụng `harper-core`); luật L0 bỏ qua câu không phải tiếng Việt; EN để backlog v2.x | Expert (D9/D16) |

## 9.1 Quy tắc clean-room khi "mượn kiến trúc" (D17 — bắt buộc)

1. Được **đọc** Harper để hiểu *ý tưởng* (mô hình token/span/lint/rule, kỹ thuật mask, FST dictionary) — ý tưởng/thuật toán không thuộc quyền tác giả.
2. **Không copy code** Harper vào repo. Mọi file trong `assist/` viết **từ đầu** của TextVN.
3. Nếu sau này cần lấy code (ví dụ đọc lại logic FST): phải giữ `LICENSE`/`NOTICE` Apache-2.0, ghi attribution trong `REUSE.toml` + `docs/compliance/`, và **không** để nó nằm trong đường gõ chính.
4. Tên module/kiểu dữ liệu của TextVN phải **khác** tên Harper (tránh nhầm là bản sao) — hiện đã dùng: `Span`, `Issue`, `IssueKind`, `Fix`, `Ctx`, `Token`, `TokenKind` (Harper: `Span`, `Lint`, `LintKind`, `Token`, `TokenKind` — trùng `Span`/`Token` là tên generic, chấp nhận được, nhưng **không** đặt tên file/module giống Harper).
5. Ràng buộc này ghi trong **ADR-012** (`docs/adr/README.md`) để agent sau không vô tình vi phạm.

---

## 10. Liên kết

- Kế hoạch thi công: `P4-0-MASTER-PLAN.md` (cùng thư mục)
- Chính sách kiến trúc hành vi (không giống malware): `docs/specs/antivirus-false-positive.md` §8
- Backlog nguồn: `PLAN.md` §2.3 (S1, S2, C1)
