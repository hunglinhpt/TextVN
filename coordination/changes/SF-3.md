<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-3 — compose vào `textvn-session` (claude-opus-c4c5a567)

> Nhận theo lệnh chủ repo ("Start SF-3 now", 2026-11-28) bằng `board claim --dor-override`.
> SF-2 đã xong phần việc (CI xanh), nhưng claim SF-2 vẫn do antigravity-gemini-37c7c6bc giữ.

## VI

### Pha 3a — planner compose MỘT nguồn (TSF + Linux) — XONG trong commit này
- **`session/src/compose.rs`**: hợp nhất `plan_key`/`text_after`/`CompositionPlan`/`KeyKind` của TSF và
  `lc_plan_key` của Linux thành MỘT planner thuần (`plan_key_with`). Khác biệt theo năng lực OS là
  tham số `ComposePolicy`, không phải mã riêng:
  - `TSF`: ranh giới in được commit CÙNG từ, phím bị ăn (CUAS, tránh `" được"`);
  - `LINUX`: commit từ rồi forward phím; tối đa 256 (`LC_COMP_MAX`, C5); Ctrl+Space ⇒ `reset_engine`
    (R2-121).

  Luật giữ nguyên: B2 (Enter/Tab là phím thật), E62 (`PASS && CONSUMED` ăn im lặng), action lạ ⇒
  fail-open. Hằng lấy từ `textvn_core`; thêm `vk::PACKET`/`PROCESSKEY` vào core.
- **TSF**: `adapters/windows-tsf/src/compose.rs` bỏ bản định nghĩa, `pub use` từ session; giải mã
  `ime_result_v1` qua `EngineStep::from_result`. Session định nghĩa trait `RawEngineResult`, crate
  `ffi` cài cho `ime_result_v1` (session không phụ thuộc FFI; `cli replay` không phải sửa). Hành vi
  không đổi:
  TSF 131/131; `trace-diff` giữ 136 vết TSF byte-to-byte (SF-G1 cho TSF).
- **C ABI `ime_compose_plan`** (`ffi/src/compose_abi.rs`, header + bản sao macOS, `textvn-cli verify`):
  - hàm thuần, struct `ime_compose_plan_v1` (abi_version + 8 trường u32 + `text[256]`);
  - policy/loại phím lạ, con trỏ NULL, surrogate ⇒ `IME_ERR_INVALID_ARG`; lệch ABI ⇒ `IME_ERR_ABI`;
  - text > 256 ⇒ `IME_ERR_BUFFER`; panic ⇒ `IME_ERR_INTERNAL`.
- **Linux**: `lc_plan_key` thành lớp mỏng gọi `ime_compose_plan(POLICY_LINUX)` (lỗi ⇒ fail-open). Bản
  C cũ chỉ còn trong `tests/test_compose.c` làm oracle cho
  `test_sf3_plan_parity_with_legacy`: hơn 50 nghìn tổ hợp (độ dài preedit × loại phím × action ×
  cờ × delete_count × insert × cursor_left) phải ra ĐÚNG kế hoạch cũ. Đây là parity tạm (G22), xoá
  cùng mã cũ.
- **Test**:
  - `session/tests/compose.rs`: 13 luật cho cả 2 policy;
  - `ffi` `sf3_*`: 4 test ABI;
  - 2 vết vàng TSF/Linux phải giữ nguyên (`trace-diff`, `ci-linux` `trace_compose`).

### Còn lại
- **3b — macOS** (`ApplyReplace.swift`/`Marked.swift`): IMK mở preedit ở lần biến đổi đầu
  (`MAC_IMK_PREEDIT_LIFECYCLE`). Cần thêm policy macOS rồi chuyển qua `ime_compose_plan`, vết
  `corpus/traces/mac` giữ nguyên.
- **3c — session tự chạy engine + compose** trong `Session::event` (adapter chỉ áp lệnh), có shadow 3 OS,
  luật *reset* R2-171/E78 chuyển vào session, `replay` chạy chính session cho mọi adapter.
- Xoá oracle C và `lc_key_classify` trùng khi session thành mặc định.

## EN

### Phase 3a — ONE compose planner (TSF + Linux) — DONE in this commit
- **`session/src/compose.rs`**: merges TSF's `plan_key`/`text_after`/`CompositionPlan`/`KeyKind` and
  Linux's `lc_plan_key` into ONE pure planner (`plan_key_with`). OS capability differences are a
  `ComposePolicy` parameter, not separate code:
  - `TSF`: a printable boundary is committed WITH the word and the key is eaten (CUAS, avoids
    `" được"`);
  - `LINUX`: commit the word, then forward the key; at most 256 (`LC_COMP_MAX`, C5); Ctrl+Space ⇒
    `reset_engine` (R2-121).

  Rules kept: B2 (Enter/Tab are real keys), E62 (`PASS && CONSUMED` eaten silently), unknown action ⇒
  fail-open. Constants come from `textvn_core`; `vk::PACKET`/`PROCESSKEY` added to core.
- **TSF**: `adapters/windows-tsf/src/compose.rs` drops its own definitions and `pub use`s the session;
  `ime_result_v1` is decoded with `EngineStep::from_result`. The session defines the
  `RawEngineResult` trait and the `ffi` crate implements it for `ime_result_v1` (the session does not
  depend on FFI; `cli replay` needs no change). Behaviour unchanged: TSF 131/131; `trace-diff` keeps the 136 TSF traces byte-for-byte (SF-G1 for
  TSF).
- **C ABI `ime_compose_plan`** (`ffi/src/compose_abi.rs`, header + macOS copy, `textvn-cli verify`):
  - pure function, struct `ime_compose_plan_v1` (abi_version + 8 u32 fields + `text[256]`);
  - unknown policy/key kind, NULL pointer, surrogate ⇒ `IME_ERR_INVALID_ARG`; ABI mismatch ⇒
    `IME_ERR_ABI`;
  - text > 256 ⇒ `IME_ERR_BUFFER`; panic ⇒ `IME_ERR_INTERNAL`.
- **Linux**: `lc_plan_key` is now a thin layer calling `ime_compose_plan(POLICY_LINUX)` (error ⇒
  fail-open). The old C version lives only in `tests/test_compose.c` as the oracle for
  `test_sf3_plan_parity_with_legacy`: over 50 thousand combinations (preedit length × key kind ×
  action × flags × delete_count × insert × cursor_left) must give EXACTLY the old plan. This is a
  temporary parity test (G22), removed together with the old code.
- **Tests**:
  - `session/tests/compose.rs`: 13 rules for both policies;
  - `ffi` `sf3_*`: 4 ABI tests;
  - both TSF/Linux golden traces must stay unchanged (`trace-diff`, `ci-linux` `trace_compose`).

### Remaining
- **3b — macOS** (`ApplyReplace.swift`/`Marked.swift`): IMK opens preedit on the first transform
  (`MAC_IMK_PREEDIT_LIFECYCLE`). Needs a macOS policy, then switching to `ime_compose_plan` with
  `corpus/traces/mac` unchanged.
- **3c — the session runs engine + compose itself** in `Session::event` (adapters only apply
  commands), with 3-OS shadow, the *reset* rules R2-171/E78 moved into the session, and `replay`
  running the session itself for every adapter.
- Remove the C oracle and the duplicate `lc_key_classify` once the session is the default.
