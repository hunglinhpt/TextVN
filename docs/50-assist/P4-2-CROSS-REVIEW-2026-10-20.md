# P4-2 — CROSS-REVIEW 2026-10-20: Rà soát chéo "Trợ lý ngữ pháp" (Phần 4) + phản hồi của agent A

- **Ngày:** 2026-10-20 · **Trạng thái:** Cross-review đã xử lý (Fassist-32..45) · **Phần:** 4 — P4
- **Người rà soát:** agent B (cross-review, 6 finding `Fassist-32..37`) · **Phản hồi + xử lý:** agent A
- **Đối tượng:** `P4-0-MASTER-PLAN.md` v2 (D1–D20) · `P4-1-harper-evaluation.md` (Q1–Q5, §9.1) · crate `assist/` skeleton (`2216053`) · ADR-012
- **Nguyên tắc (G6):** finding **không bị xóa** — sai tiền đề thì đánh dấu **sai + superseded**, ghi rõ bằng chứng mới.
- **Cross-ref:** `P4-0-MASTER-PLAN.md` §3.1 (D1–D29) · §11.3 (audit vòng 6) · `docs/adr/README.md` (ADR-012, ADR-013) · `docs/specs/verified-ops.md` (B9–B10)

---

## 0. Kết luận của agent A sau khi verify lại từng fact

**Agent B đúng ở 6/6 fact** — và **tiền đề sai nằm trong chính docs của agent A** (`ADR-012`, `P4-1` §1/§0/§8, `P4-0` D1, log `00-INDEX`): tôi từng khẳng định *"crates.io `harper-core` stale ~27 tháng (0.8.7 / 2024-07-06)"*. **Sai.** Số thật:

| Mục | Số thật (verify 2026-10-20) |
|---|---|
| `harper-core` trên crates.io | **2.13.0**, publish **2026-10-16** (4 ngày trước), **123 versions**, 126.501 lượt tải, `Apache-2.0` |
| Nhịp phát hành | 2.7.0 (2026-07-28) → 2.8.0 (08-13) → 2.10.0 (09-10) → 2.11.0 (09-16) → 2.13.0 (10-16) → **phát hành ~hàng tháng** |
| Edition / MSRV | **edition 2024** → cần Rust ≥ 1.85 |
| Dependency | `hashbrown`, `ordered-float`, `smallvec`, `levenshtein_automata` (FST), `harper-brill`, `harper-thesaurus` (optional), `bitflags`, **`zip`**, **`boxcar`** |
| Từ điển | `dictionary.dict` = **789.666 B** FST, **embed** trong crate (không I/O runtime) |

> **Nguyên nhân gốc lỗi của tôi:** lần tra crates.io đầu, output JSON là **một dòng khổng lồ** bị tool cắt; tôi grep theo mẫu rồi đọc **đầu mảng `versions`** — mảng trả về **mới nhất trước** nên phần bị cắt đúng là các bản mới nhất, còn `0.8.7` là bản **cũ nhất trong phần còn lại**. Đã sửa quy trình: verify bằng **parse JSON có cấu trúc** (`max_version`/`newest_version`), không đọc mảng cắt.

**Kết luận D1 giữ nguyên (không dùng `harper-core` ở v1) nhưng tiền đề phải viết lại đúng** → **D21**.

---

## 1. Disposition 6 finding của agent B

| ID | Mức (B) | Nội dung | Disposition của A | Hành động |
|---|---|---|---|---|
| **Fassist-32** | major | ADR-012 + `P4-1` ghi "crates.io stale ~27 tháng" → **dữ liệu sai**, tiền đề quyết định sai (vi phạm G3) | ✅ **ACCEPT** (verify lại: `2.13.0` / 2026-10-16 / ed2024 / 123 bản) | Sửa **8 chỗ**; **D21** viết lại tiền đề; `Fassist-2` đánh dấu **sai tiền đề → superseded** (không xóa, G6) |
| **Fassist-33** | major | Mâu thuẫn nội bộ: Mode 1 còn nói "dùng `strategy::SelectionReplace`" trong khi D12 kết luận Backspace+Type | ✅ **ACCEPT (đã xử lý một phần ở v2)** — `P4-0` §4.2 v2 đã dùng A2/Backspace+Type; còn sót ở §3 (diagram) + §4.4 (nhắc SelectionReplace) | **D22**: v1 **bỏ hẳn** SelectionReplace khỏi kế hoạch (giữ "có sẵn trong `strategy`" như ghi chú, không dùng vì rủi ro map vị trí) + sweep wording |
| **Fassist-34** | minor | §II diagram nói chung chung "adapter gọi thẳng crate assist" — chỉ adapter **Rust** gọi trực tiếp được; Swift/C/C++ phải qua C-ABI `ime_assist_*` | ✅ **ACCEPT** | **D23** + sửa diagram: `windows-tsf` (Rust) gọi crate `assist` trực tiếp; macOS/Linux qua `textvn_ffi` C-ABI + header `textvn_ffi.h` + `textvn-cli verify` chống drift |
| **Fassist-35** | major | Shadow buffer đặt trong adapter (3 bản) → drift + lệch không gian offset; `recent` của core đang ở **glyph space** (`emit()` = `charset::encode`), 13 call site `recent_replace` đã điều phối đủ case biên | ✅ **ACCEPT** (verify: `recent: Vec<char>`, `RECENT_MAX=64`, `emit()` = `transform::charset::encode`) — nhưng **điều chỉnh**: `recent` (64 ký tự **glyph**) **không đủ** cho câu (≤500) → cần **2 buffer** | **D24**: `Engine.sentence` (**Unicode**, ≤ 256) là buffer *câu*; `recent` giữ nguyên vai trò *glyph* (64) → **refactor trước**: gom 13 call site `recent_replace` → 2 helper (`recent_push`/`recent_pop`) rồi mới thêm mirror → task **P4-2a** + **ADR-013** |
| **Fassist-36** | major | (a) IBus: 1 `ime_instance`/input context → tài nguyên nặng **per-instance**; (b) Windows TIP in-process → engine nhúng **trong mọi app process**; plan chưa chốt cơ chế share | ✅ **ACCEPT** (verify: `P3-1-ibus.md` dòng 15 "Mỗi `IBusEngine` (= 1 input context) có 1 `ime_instance` riêng"; TIP in-proc = kiến trúc TSF) | **D25**: `OnceLock<Arc<…>>` cho tài nguyên nặng (**immutable-after-init** → không vi phạm "no global **mutable** state") + **2 spike**: **S4a** đo per-process RSS/binary/p95 của tài nguyên assist **của ta** (gate: RSS ≤ +5 MB, binary ≤ +3 MB, p95 ≤ 2 ms) — **chặn P4-4**; **S4b** khảo sát `harper-core` cho tiếng Anh = **v2, không chặn v1** |
| **Fassist-37** | minor | `cargo clippy --workspace` full parallel → hết paging file trên máy dev; chưa ghi `verified-ops.md` | ✅ **ACCEPT** | Thêm **B9** (`-j 2` cho clippy/test workspace) + **B10** (bump page file khi build nặng) vào `docs/specs/verified-ops.md` |

**Không có finding nào của B phủ nhận D11–D20** — đồng thuận.

---

## 2. Finding mới của agent A (Fassist-38..45) — phát hiện khi review lại đề xuất của B

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-38** | major | Đề xuất F35 (đưa buffer câu vào `core`) **tạo mâu thuẫn mới** với F36: `core` nằm trong TIP in-process → buffer 500 char × mọi app = RSS/ngữ cảnh | **D26**: buffer câu chỉ tồn tại **khi `assist.mode != off`** (feature flag) + cap **256**; tách khỏi `recent` (không dùng chung) |
| **Fassist-39** | major | Hợp đồng giữa `assist` ↔ `core` còn mơ hồ: `assist` trả offset **char Unicode**, nhưng app nhận **glyph** (TCVN3/VNI-Windows: 1 chữ = 2 ký tự) → số ký tự cần xóa ≠ số ký tự Unicode | **D27**: `assist` trả issue theo **char offset Unicode**; lớp adapter đổi sang **glyph count** qua `Engine::glyph_len_of(unicode)` (core đã có `emit()` để tính) — `Fix` **không** mang số xóa, việc đếm do core |
| **Fassist-40** | major | Luật **L0** (bỏ qua câu không phải tiếng Việt) đã chốt ở D16 nhưng **chưa có trong skeleton** → P4-1 còn thiếu | Task **P4-1**: thêm `rules/language.rs` + test (heuristic tỷ lệ từ có dấu) |
| **Fassist-41** | minor | `Issue` chưa có **điểm tin cậy** nhưng D19 cần dùng nó để lọc luật mơ hồ về sau → thêm sau này = phá API | **D28**: thêm `confidence: u8` (0–100) vào `Issue` **ngay trong P4-1** (additive, rẻ hơn nhiều so với sửa API sau) |
| **Fassist-42** | minor | Quy ước offset xuyên biên chưa chốt: `assist` = char; TSF/AX = **UTF-16**; AT-SPI = char | **D29**: chuẩn hoá tại biên FFI sang **UTF-16 code unit** (khớp TSF/AX); crate `assist` giữ char + cung cấp helper đổi |
| **Fassist-43** | minor | Định dạng corpus `.ass` chưa được đặc tả (ai parse, format, số lượng) | Task **P4-1**: đặc tả TSV (`câu \t kind1,kind2`) trong `P4-TASKS.md` + `P0-4` |
| **Fassist-44** | minor | Mô tả "6 luật" vs thực tế **6 module / 7 mã luật** (L1+L2+L11 gộp 1 module) | Sửa wording ở commit message + `00-INDEX` + `P4-0` §5.1 |
| **Fassist-45** | minor | Timeline của B "15–19 tuần nếu EN pass" **mâu thuẫn D9** (v1 không EN) | **Giữ scope v1 = không EN.** S4b chỉ là khảo sát v2, **không** cộng vào timeline v1, **không** chặn P4-2..P4-6 |

---

## 3. Quyết định mới D21–D29 (bổ sung D1–D20)

| # | Quyết định | Lý do |
|---|---|---|
| **D21** | **D1 viết lại**: không dùng `harper-core` ở v1 vì (a) **English-only ở mọi tầng** → không có giá trị cho tiếng Việt; (b) **API churn ~hàng tháng** (123 bản, 2.13.0 mới 4 ngày) → vendor phải pin + nợ nâng cấp; (c) **chi phí resident** khi nhúng trong TIP in-process (`zip`/`boxcar`/FST 790 KB) → cần S4 mới biết; (d) edition 2024 → MSRV ≥ 1.85. **Không** dùng lý do "crates.io stale" (sai) | Fassist-32 |
| **D22** | v1 **không dùng `SelectionReplace`** để áp sửa; chỉ dùng **Backspace+Type** (A2/A3) theo D12 | Fassist-33 |
| **D23** | Biên gọi: `windows-tsf` (Rust) gọi crate `assist` trực tiếp; macOS/Linux (Swift/C/C++) qua **C-ABI `ime_assist_*`** trong crate `ffi` + header + `textvn-cli verify` | Fassist-34 |
| **D24** | Buffer câu nằm trong **core** (`Engine.sentence`, Unicode) — **refactor 13 call site `recent_replace` → 2 helper** trước khi thêm; tách khỏi `recent` (glyph, 64) | Fassist-35 |
| **D25** | Tài nguyên nặng dùng **`OnceLock<Arc<…>>`** (immutable-after-init); **S4a** đo per-process RSS/binary/p95 (gate ≤ +5 MB / ≤ +3 MB / ≤ 2 ms) **chặn P4-4**; **S4b** (harper-core/EN) = v2, không chặn | Fassist-36 |
| **D26** | Buffer câu chỉ bật khi `assist.mode != off`, cap 256 char | Fassist-38 |
| **D27** | `assist` trả offset **char Unicode**; **số ký tự cần xóa do core tính** (`Engine::glyph_len_of`) — `Fix` không mang số xóa | Fassist-39 |
| **D28** | `Issue` có `confidence: u8` ngay từ P4-1 | Fassist-41 |
| **D29** | Biên FFI chuẩn hoá **UTF-16 code unit**; crate `assist` giữ char + helper đổi | Fassist-42 |

---

## 4. Lộ trình cập nhật (chèn vào `P4-0` §8)

| Pha | Nội dung | Nguồn |
|---|---|---|
| **P4-1c** | Sửa 8 chỗ tiền đề sai (F32) + sweep wording (F33/F34) + `P4-TASKS.md` + `verified-ops` (F37) + ADR-012/013 | chính file này |
| **P4-1** | corpus `.ass` ≥ 60 câu + test tích hợp + `textvn-cli assist check` + **L0** (F40) + `confidence` (F41) | `P4-0` §8 |
| **P4-1b** | S1 TSF candidate list · S2 caret 3 OS · S3 từ điển VN · **S4a** per-process RSS (chặn P4-4) · **S4b** harper-core/EN (v2, không chặn) | D18 + D25 |
| **P4-2a** | **Refactor core**: gom 13 call site `recent_replace` → 2 helper + thêm `Engine.sentence` (Unicode, ≤256, chỉ bật khi assist bật) | D24/D26 |
| **P4-2** | FFI `ime_assist_*` + trigger (space/,/.) + toast + kill-switch | D23 + `P4-0` §8 |
| **P4-3** | Candidate list Windows + A2 (sửa 1 từ) + precondition | `P4-0` §8 |
| **P4-4** | Từ điển VN (FST mmap + `OnceLock` share) — **chỉ sau S4a** | D15/D25 |
| **P4-5** | Mode 2 / A3 (tự sửa cả câu) | `P4-0` §8 |
| **P4-6** | Parity macOS/Linux + settings 3 OS | `P4-0` §8 |

**Timeline v1 giữ nguyên ~13–16 tuần** (Fassist-45) — S4b không tính vào v1.

---

## 5. Trạng thái finding sau cross-review

| Nhóm | Trạng thái |
|---|---|
| Fassist-1, 3–12, 13–31 | Đã xử lý ở plan (D1–D20, task list) |
| **Fassist-2** | ⚠️ **Sai tiền đề** (crates.io không stale) → **superseded** bởi Fassist-32; giữ ID theo G6, không xóa |
| **Fassist-32..37** | Đã xử lý (D21–D25 + patch docs + verified-ops + ADR-013) |
| **Fassist-38..45** | Đã xử lý (D26–D29 + task P4-1/P4-2a) |
| Fassist-7, 11, 12 | Còn mở → P4-TASKS (`F7`), P4-2 (`F11`), P4-6 (`F12`) |

---

## 6. Liên kết

- `P4-0-MASTER-PLAN.md` · `P4-1-harper-evaluation.md` (cùng thư mục)
- `docs/adr/README.md` — ADR-012 (tiền đề đã sửa) · ADR-013 (resource-sharing + core sentence buffer)
- `docs/specs/verified-ops.md` — B9/B10
- `docs/00-WORKFLOW.md` G5/G6 · `docs/01-AGENT-HANDBOOK.md` (DoD 7 mục)
