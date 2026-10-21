# P4-7 — Review độc lập AS-021/AS-022 (code của agent song song) — vòng 15

- **Ngày:** 2026-10-21 · **Đối tượng:** commit `dba598c` "feat(ffi): AS-021 + AS-022" (agent song song) · **Reviewer:** Agent Core (độc lập, theo handoff `P4-6`)
- **Cross-ref:** `P4-6-AS-021-022-HANDOFF.md` (recipe) · `P4-5-P4-2A-REVIEW.md` (P4-2a) · `P4-0-MASTER-PLAN.md` §3.1 (D22/D23/D27/D29/D31/D36)

## 0. Kết quả đối chiếu với handoff

| Yêu cầu (handoff) | Thực hiện | Kết quả |
|---|---|---|
| FFI C-ABI `ime_assist_*` | 5 hàm: `ime_assist_trigger_kind` · `ime_assist_check` · `ime_assist_check_instance` · `ime_assist_sentence_tail` · `ime_assist_glyph_len` + 2 struct + 12 hằng | ✅ khớp |
| Header C khớp Rust (drift 0) | `verify` đồng bộ **55 hằng · 6 struct · 16 hàm** | ✅ |
| Fail-open, không panic | `catch_unwind` quanh mọi entry (`ffi/src/lib.rs` dòng 269/273/325/379/426/491/722) | ✅ |
| Không log nội dung câu (S2) | không thấy `log::`/`println!` trong phần FFI mới | ✅ |
| Offset trả về = char Unicode; số ký tự xoá do core tính (D27/D29) | `ime_assist_glyph_len` + `ime_assist_sentence_tail` | ✅ |
| 0-alloc view cho adapter (Fassist-76) | `core`: `sentence_tail_chars(&self, max) -> &[char]` | ✅ (F76 đóng) |
| Test | ffi **24/24** · cli **46/46** · core **168/168** · `preflight` **23/23 gates PASS** | ✅ |

## 1. Vòng 16 — SA/QA

| ID | Mức | Finding | Xử lý |
|---|---|---|---|
| **Fassist-78** | minor | `ime_assist_result_v1` (1736 B) — **cap đã rõ** (`count: 0..IME_MAX_ASSIST_ISSUES`) nhưng header **chưa nói rõ ownership** (struct là **giá trị**, caller giữ, **không** `free`) → dễ bị `free` nhầm (use-after-free) | ⚠️ **Không sửa tay `ffi/include/textvn_ffi.h`** — file này **do `cbindgen` sinh** (drift check ở CI, job macOS "header C-ABI") ⇒ sửa tay làm **CI đỏ** (đã tái hiện 2026-10-21). Cách đúng: thêm **doc comment** vào struct trong `ffi/src/lib.rs` (cbindgen lấy vào header) rồi **regenerate header** bằng cbindgen của CI. Việc này thuộc lane FFI (Agent Platform/FFI) |
| **Fassist-79** | minor | `ime_assist_trigger_kind` phân loại phím — nhưng **điểm hiển thị** (D11: chỉ hiện khi hết câu / idle ≥ 700 ms) **chưa có** trong diff | Đúng phạm vi: điểm hiển thị thuộc **UI (AS-023/AS-030)**; `trigger_kind` chỉ là *điểm kiểm* ✓ |

## 2. Vòng 17 — Audit (license/privacy/AV)

- Không dep mới ngoài crate nội bộ · không I/O trong FFI · không log · `catch_unwind` bọc mọi entry ⇒ S2/P0-2 §5 giữ được.
- **Đã verify**: `ime_assist_check_instance` có **null check** (`inst.is_null() || out.is_null()` → `IME_ERR_INVALID_ARG`) **trước** khi deref ⇒ an toàn con trỏ rỗng ✓ (không cần finding).
- Header `ime_assist_issue_v1` có `confidence` (D28 ✓) và `start`/`end` = **offset ký tự Unicode** (D27/D29 ✓).
- Không phát sinh vấn đề REUSE/deny (không crate mới).

## 3. Vòng 18 — PO/UX

- Cần **bằng chứng chạy thật**: gõ "tôi tôi đi học." trong Notepad → phải ra **đúng 1** kết quả (không spam mỗi space) — DoD của AS-022 yêu cầu screenshot/CLI output.
- `ime_assist_check` (theo instance) vs `ime_assist_check` (tự tạo) ⇒ chọn đường dẫn **không I/O** cho engine mới (tạo instance mỗi lần = tốn hơn) — cần benchmark p95 (D36: ≤ 50 ms, budget §5.5).

## 4. Kết luận

- **Chấp nhận** code `dba598c` (khớp handoff, fail-open, không log, test đầy đủ).
- **2 điểm cần làm tiếp** (task nhỏ, không đổi ABI): (1) header nêu rõ **cap + ownership** của `ime_assist_result_v1` (F78); (2) **null check** con trỏ engine trước khi deref (Audit vòng 17).
- Điểm hiển thị (D11) + UI thuộc lane Agent Tray/Windows-TSF (AS-023/AS-030) — không chặn.
