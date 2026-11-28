<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-4 — ghost (chữ mờ gợi ý) vào `textvn-session` (claude-opus-c4c5a567)

## VI

### Pha 1 — chính sách chữ mờ MỘT nguồn — XONG trong commit này
- **`session/src/ghost.rs`**, hợp nhất TSF `update_ghost`/`ghost_waits_for_key_up`, IBus/Fcitx5
  `update_preview`/`lc_preview_label` và IMK `updatePredictionPreview`/`previewLabel`:
  - `ghost_timing(key, eaten)`:
    - phím gõ chữ/xoá mà bộ gõ KHÔNG ăn ⇒ vẽ ở key-up (E79/R2-173: app chưa chèn ký tự);
    - chữ/xoá bị ăn, Tab, Ctrl+Space ⇒ vẽ ngay;
    - phím khác giữ nguyên (R2-134);
  - `ghost_text(peek, gates)`: chữ mờ = `peek_completion` NGUYÊN VĂN (R2-173 — đúng phần Tab nối);
    bất kỳ cổng nào cũng ẩn: secure, terminal, trợ lý đang hiện, tiếng Việt và gợi ý cùng tắt, sau caret
    còn chữ (P0-6 §5);
  - `preview_label(ghost)`: `"⇥ "` + chữ mờ bỏ khoảng trắng hai đầu.
- **C ABI `ime_ghost_label(ghost, len, gates, out, cap)`** + `IME_GHOST_*` (header, bản sao macOS,
  `verify` 37 export).
- **Adapter** chỉ đo trạng thái, session quyết định:
  - **TSF**: `ghost_timing` thay hàm riêng; `update_ghost` dựng cổng (terminal, trợ lý sống, tắt gợi ý,
    sau caret còn chữ) rồi gọi `ghost_text`;
  - **IBus/Fcitx5**: `lc_preview_label_gated` (lớp mỏng trên `ime_ghost_label`);
  - **macOS**: `CoreBridge/GhostPolicy.swift`.
- **Khác biệt cũ được thống nhất theo spec** (thay đổi hành vi có chủ đích):
  - TSF trước đây vẫn vẽ chữ mờ khi trợ lý sống đang hiện; giờ ẩn như Linux/macOS;
  - macOS trước đây không xét ô secure; giờ có;
  - nhãn Linux trước chỉ bỏ khoảng trắng đầu; giờ bỏ hai đầu như macOS.
- **Test**:
  - `session/tests/ghost.rs`:
    - `r2_173_ghost_waits_for_key_up_only_for_passed_char_keys`, chuyển từ TSF và chạy qua session;
    - R2-134;
    - cổng;
    - nhãn;
    - bất biến **`r2_173_ghost_equals_what_tab_inserts_on_real_engine`**: trên engine thật, sau mỗi
      phím có chữ mờ, nhấn Tab (bản sao) nối ĐÚNG chữ đó vào văn bản;
  - `ffi` `sf4_*`: 2 test;
  - test C: `lc_preview_label_gated`.

### Còn lại
- Chữ mờ như một `SessionCommand::Ghost`/`GhostAfterKeyUp` do `Session::event` phát (cần session tự chạy
  engine — cùng hướng nối tiếp của SF-3); adapter TSF vẫn tự đo caret/font để vẽ (phần OS).

## EN

### Phase 1 — ONE ghost-text policy — DONE in this commit
- **`session/src/ghost.rs`**, merging TSF `update_ghost`/`ghost_waits_for_key_up`, IBus/Fcitx5
  `update_preview`/`lc_preview_label` and IMK `updatePredictionPreview`/`previewLabel`:
  - `ghost_timing(key, eaten)`:
    - a text/delete key the IME does NOT eat ⇒ draw on key-up (E79/R2-173: the app has not inserted
      the character yet);
    - an eaten text/delete key, Tab, Ctrl+Space ⇒ draw now;
    - other keys keep it (R2-134);
  - `ghost_text(peek, gates)`: the ghost is `peek_completion` VERBATIM (R2-173 — exactly what Tab
    appends); any gate hides it: secure, terminal, assistant visible, Vietnamese and suggestions both
    off, text after the caret (P0-6 §5);
  - `preview_label(ghost)`: `"⇥ "` + the ghost trimmed on both ends.
- **C ABI `ime_ghost_label(ghost, len, gates, out, cap)`** + `IME_GHOST_*` (header, macOS copy,
  `verify` 37 exports).
- **Adapters** only measure state; the session decides:
  - **TSF**: `ghost_timing` replaces its own function; `update_ghost` builds the gates (terminal, live
    assistant, suggestions off, text after the caret) then calls `ghost_text`;
  - **IBus/Fcitx5**: `lc_preview_label_gated` (thin layer over `ime_ghost_label`);
  - **macOS**: `CoreBridge/GhostPolicy.swift`.
- **Old differences aligned with the spec** (intentional behaviour changes):
  - TSF used to draw the ghost while the live assistant was showing; it now hides it like Linux/macOS;
  - macOS did not check secure fields; it now does;
  - the Linux label only trimmed leading spaces; it now trims both ends like macOS.
- **Tests**:
  - `session/tests/ghost.rs`:
    - `r2_173_ghost_waits_for_key_up_only_for_passed_char_keys`, moved from TSF and run through the
      session;
    - R2-134;
    - gates;
    - label;
    - invariant **`r2_173_ghost_equals_what_tab_inserts_on_real_engine`**: on the real engine, after
      every key with a ghost, pressing Tab (on a copy) appends EXACTLY that ghost to the text;
  - `ffi` `sf4_*`: 2 tests;
  - C test: `lc_preview_label_gated`.

### Remaining
- Ghost as a `SessionCommand::Ghost`/`GhostAfterKeyUp` emitted by `Session::event` (needs the session
  to run the engine itself — the same follow-up direction as SF-3); the TSF adapter keeps measuring
  caret/font to draw (OS part).
