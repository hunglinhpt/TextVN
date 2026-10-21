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
| **Fassist-84** | major | **Mirror header Swift bị stale** → job macOS "header C-ABI trong module Swift không được lệch `ffi/include/`" **ĐỎ từ `dba598c`**: repo có **2 bản** header — nguồn `ffi/include/textvn_ffi.h` + mirror `adapters/macos-imk/Sources/CTextVNFFI/include/textvn_ffi.h`; AS-021 thêm API assist vào **nguồn** nhưng **quên mirror** ⇒ chặn CI của cả team | ✅ Đã sửa (`dd1afeb`): copy nguồn → mirror (đúng như CI copy) ⇒ ci-macos xanh. **Bài học:** mọi thay đổi `ffi/include/textvn_ffi.h` phải **copy sang mirror** trong cùng commit (CI chỉ copy 1 chiều) |
| **Fassist-85** | minor | `ffi/include/textvn_ffi.h` **không** do cbindgen sinh trong xtask (chú thích `xtask/src/main.rs:13` nói "chưa có") ⇒ sửa tay header **được**, nhưng phải sửa **cả 2 bản** | Ghi rõ quy trình (F84) để lane FFI không mắc lại |

## 4.1 Bổ sung (2026-10-21, sau khi CI báo đỏ)

- `764ec4c` (của tôi) sửa **1 dòng comment** trong `ffi/include/textvn_ffi.h` → **làm lệch mirror** → ci-macos đỏ. Đã hoàn nguyên phần sửa đó (`e962fce`).
- Nguyên nhân gốc thực sự: mirror **đã stale sẵn từ `dba598c`** (thiếu khối assist API) ⇒ CI đỏ từ trước, tôi chỉ là người phát hiện sớm nhất.
- Đã đồng bộ mirror (`dd1afeb`) ⇒ **7/7 workflow xanh**.
- **F78 (ownership của `ime_assist_result_v1`)** vẫn **mở**: phải sửa ở **nguồn** `ffi/include/textvn_ffi.h` **và** mirror trong cùng commit (không sửa lệch 1 bản).
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
