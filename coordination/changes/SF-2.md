<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-2 — toggle + batch vào `textvn-session` (antigravity-gemini-37c7c6bc → claude-opus-c4c5a567)

> Bắt đầu bởi **antigravity-gemini-37c7c6bc** (WIP chưa commit, bị limit giữa chừng). Chủ repo yêu cầu
> **claude-opus-c4c5a567 tiếp quản** (board note `14e5c32c`). Board từ chối claim lại vì lease chưa
> hết hạn, và không ai giả danh agent khác để handoff (G21 §2).

## VI

### Pha 1 + shadow TSF — XONG trong commit này
- **`session/src/toggle.rs`** (`ToggleState`, `ToggleOutcome`): đưa `ModifierToggle` của TSF về **một
  nguồn**, theo P0-6 §5:
  - Ctrl+Shift nhấn-thả không kèm phím khác (2 thứ tự nhấn đều được, phím giữ lặp không đổi kết quả);
  - Alt/Win hoặc phím thứ ba huỷ lần nhấn đang chờ;
  - `Ctrl+Shift+Space` bỏ bit CapsLock (ADR-011);
  - Ctrl tap ≤ 400 ms (R2-143), debounce 250 ms;
  - pha thứ hai OnKeyUp không toggle lại;
  - thời gian chỉ đến qua `now_ms`, session không có đồng hồ.
- **`session/src/batch.rs`** (`EditBatch`): bản chuyển của `lc_edit_batch` (R2-165/R2-167):
  - lệnh xoá ăn vào phần đang chờ commit trước, phần dư mới xoá document thật;
  - lồng `begin`/`end`;
  - `flush_into` ra ≤ 1 `DeleteBefore` rồi ≤ 1 `Commit`;
  - ngưỡng `EDIT_BATCH_MAX_BYTES` = **8192**, khớp `LC_EDIT_BATCH_MAX`. WIP ghi nhầm 1024, đã sửa.
- **`Session::event`**:
  - `KeyDown`/`KeyUp` chạy máy toggle. Toggle hợp lệ ⇒ `ModeIndicator{vi_on}`. Chord `Ctrl+Shift+Space`
    ⇒ `EatKey`, phím khác ⇒ `ForwardKey`.
  - `FocusOut`/`Reset` huỷ lần nhấn đang chờ.
  - IPC `ToggleViEn`/`StateUpdate` toàn cục (`app_id "*"`) đặt `vi_on`.
- **G22**: hằng VK/MOD của session lấy giá trị từ `textvn_core::keymap`. Đã thêm `LSHIFT..RMENU`
  (`0xA0..0xA5`) vào `core::keymap::vk`. `session` giờ phụ thuộc `textvn-core`, cũng là phụ thuộc SF-3
  sẽ cần.
- **Shadow TSF** (`TEXTVN_SESSION_SHADOW=1`, chỉ CI/dev): session chạy song song với `ModifierToggle`, chỉ
  áp mã cũ, đếm số lệch và ghi trace. **G8**: log chỉ in mã VK khi là phím modifier, phím khác ghi
  `non-modifier` (bản WIP in `vk` của mọi phím nhả ⇒ rủi ro keylog, đã sửa).
- **Test**:
  - `session/tests/toggle_and_batch.rs`: 14 test cho từng luật toggle/batch;
  - `compose.rs::sf2_shadow_parity_tsf_modifier_toggle_vs_session`: 0 lệch giữa mã cũ và session trên
    các kịch bản Ctrl→Shift, Shift→Ctrl, phím giữ lặp, phím thứ ba, Alt/Win, Ctrl tap nhanh/chậm,
    FocusOut.

  WIP có 2 lỗi biên dịch (thiếu `vk::LCONTROL`; 2 closure cùng mượn `&mut`), đã viết lại test bằng
  struct `Pair`.
- Kết quả: TSF 130/130 · session 14 + 8 · clippy `-D warnings` (Windows + target Linux) · rustfmt sạch.

### Pha 2 — shadow Linux (IBus + Fcitx5) — XONG
- **`lc_session_shadow`** (`lc_compose.h`/`compose.c`): chạy máy toggle của session qua C ABI
  `ime_session_*` (SF-1b) song song `lc_modifier_toggle`. Bật bằng `TEXTVN_SESSION_SHADOW=1`; mặc định
  tắt, mọi hàm là no-op. Shadow:
  - nhận đúng các cờ mã cũ dùng (`ctrl_down`/`shift_down`/`alt_or_super`), nên lệch chỉ có thể đến
    từ logic;
  - so "có bật/tắt Việt–Anh hay không" (`IME_SCMD_MODE_INDICATOR`) ở key-down (chord ADR-011) và
    key-up (Ctrl+Shift nhấn-thả);
  - lỗi ABI ⇒ shadow tự tắt, không bao giờ ảnh hưởng mã cũ.
- **G8**: log stderr chỉ ghi loại phím (`ctrl`/`shift`/`other-modifier`/`space`/`non-modifier`) và số
  đếm; shadow không nhận ký tự (`ch` luôn 0).
- **G22**: mã VK dùng khối `LC_VK_*` sẵn có (thêm `LC_VK_SHIFT/CONTROL/MENU`), không định nghĩa riêng.
- **Hook** IBus (`engine.c`) và Fcitx5 (`engine.cpp`): tạo/huỷ cùng engine/input context; reset ở mất
  focus; key-down ngay sau `lc_modifier_toggle_down`; key-up tách `ctrl_tap`/`toggled` rồi báo shadow
  (hành vi cũ giữ nguyên).
- **Test** `test_compose.c::test_session_shadow_parity` (chạy cùng engine Rust thật, ASan/UBSan trên
  ci-linux):
  - 9 kịch bản (Ctrl→Shift, Shift→Ctrl, phím thứ ba, Alt/Super, Alt chen giữa, phím giữ lặp,
    Ctrl+Shift+Space, mất focus, Ctrl tap) ⇒ **4 lần bật/tắt, 0 lệch**;
  - shadow tắt mặc định là no-op.
- **Khác biệt ĐÃ BIẾT** (shadow bắt được, test khẳng định; quyết định ở pha bật mặc định):
  - (a) Linux cũ bật/tắt cả `Ctrl+Shift+Alt+Space`, còn session (= TSF preserved key
    `TF_MOD_CONTROL|TF_MOD_SHIFT`) thì không;
  - (b) Linux cũ không có debounce 250 ms, nên 2 lần bật/tắt cách nhau < 250 ms bị session chặn lần
    thứ hai.

### Còn lại (pha 3, task tiếp tục)
- **Shadow macOS** (`TextVNInputController.swift`, `otherKeyPressed`).
- **Đưa batch vào đường áp lệnh** của 3 adapter, kiểm SF-G1/G2 cho corpus toggle, đo shadow 0 lệch trên
  e2e, rồi bật mặc định ở bản kế tiếp.
- **Nợ G22**: adapter TSF còn module `vk` riêng trong `compose.rs` (không phụ thuộc `textvn-core`); gom
  về core là một bước dọn riêng.

## EN

### Phase 1 + TSF shadow — DONE in this commit
- **`session/src/toggle.rs`** (`ToggleState`, `ToggleOutcome`): brings the TSF `ModifierToggle` into a
  **single source**, per P0-6 §5:
  - Ctrl+Shift press-release with no other key (both press orders work; key repeat does not change the
    result);
  - Alt/Win or a third key cancels the pending press;
  - `Ctrl+Shift+Space` ignores the CapsLock bit (ADR-011);
  - Ctrl tap ≤ 400 ms (R2-143), 250 ms debounce;
  - the second OnKeyUp phase does not toggle again;
  - time only arrives through `now_ms`; the session has no clock.
- **`session/src/batch.rs`** (`EditBatch`): port of `lc_edit_batch` (R2-165/R2-167):
  - deletes consume the pending commit text first, and only the remainder deletes the real document;
  - nested `begin`/`end`;
  - `flush_into` emits ≤ 1 `DeleteBefore` then ≤ 1 `Commit`;
  - `EDIT_BATCH_MAX_BYTES` = **8192**, matching `LC_EDIT_BATCH_MAX`. The WIP said 1024 by mistake; fixed.
- **`Session::event`**:
  - `KeyDown`/`KeyUp` drive the toggle machine. A valid toggle ⇒ `ModeIndicator{vi_on}`. The
    `Ctrl+Shift+Space` chord ⇒ `EatKey`, other keys ⇒ `ForwardKey`.
  - `FocusOut`/`Reset` cancel the pending press.
  - Global IPC `ToggleViEn`/`StateUpdate` (`app_id "*"`) set `vi_on`.
- **G22**: the session's VK/MOD constants take their values from `textvn_core::keymap`. `LSHIFT..RMENU`
  (`0xA0..0xA5`) were added to `core::keymap::vk`. `session` now depends on `textvn-core`, which SF-3
  will need as well.
- **TSF shadow** (`TEXTVN_SESSION_SHADOW=1`, CI/dev only): the session runs alongside `ModifierToggle`;
  only the old code is applied, mismatches are counted and traced. **G8**: the log prints the VK only
  for modifier keys and writes `non-modifier` otherwise (the WIP printed the `vk` of every released key
  ⇒ keylog risk; fixed).
- **Tests**:
  - `session/tests/toggle_and_batch.rs`: 14 tests, one per toggle/batch rule;
  - `compose.rs::sf2_shadow_parity_tsf_modifier_toggle_vs_session`: 0 mismatches between the old code
    and the session on Ctrl→Shift, Shift→Ctrl, key repeat, a third key, Alt/Win, fast/slow Ctrl tap
    and FocusOut.

  The WIP had 2 compile errors (missing `vk::LCONTROL`; two closures both borrowing `&mut`); the test
  was rewritten around a `Pair` struct.
- Results: TSF 130/130 · session 14 + 8 · clippy `-D warnings` (Windows + Linux target) · rustfmt clean.

### Phase 2 — Linux shadow (IBus + Fcitx5) — DONE
- **`lc_session_shadow`** (`lc_compose.h`/`compose.c`): runs the session's toggle machine through the
  `ime_session_*` C ABI (SF-1b) alongside `lc_modifier_toggle`. Enabled with `TEXTVN_SESSION_SHADOW=1`;
  off by default, where every function is a no-op. The shadow:
  - receives exactly the flags the old code uses (`ctrl_down`/`shift_down`/`alt_or_super`), so a
    mismatch can only come from logic;
  - compares "did Vietnamese/English toggle or not" (`IME_SCMD_MODE_INDICATOR`) on key-down (ADR-011
    chord) and key-up (Ctrl+Shift press-release);
  - an ABI error ⇒ the shadow turns itself off and never affects the old code.
- **G8**: the stderr log only records the key kind (`ctrl`/`shift`/`other-modifier`/`space`/
  `non-modifier`) and counters; the shadow never receives characters (`ch` is always 0).
- **G22**: VK codes use the existing `LC_VK_*` block (`LC_VK_SHIFT/CONTROL/MENU` added), no private
  definitions.
- **Hooks** in IBus (`engine.c`) and Fcitx5 (`engine.cpp`): created/freed with the engine/input
  context; reset on focus loss; key-down right after `lc_modifier_toggle_down`; key-up splits
  `ctrl_tap`/`toggled` then reports to the shadow (old behaviour unchanged).
- **Test** `test_compose.c::test_session_shadow_parity` (runs with the real Rust engine, ASan/UBSan on
  ci-linux):
  - 9 scenarios (Ctrl→Shift, Shift→Ctrl, third key, Alt/Super, Alt in between, key repeat,
    Ctrl+Shift+Space, focus loss, Ctrl tap) ⇒ **4 toggles, 0 mismatches**;
  - the shadow is a no-op when off (default).
- **KNOWN differences** (caught by the shadow and asserted by the test; to be decided when the session
  becomes the default):
  - (a) old Linux also toggles on `Ctrl+Shift+Alt+Space`, while the session (= TSF preserved key
    `TF_MOD_CONTROL|TF_MOD_SHIFT`) does not;
  - (b) old Linux has no 250 ms debounce, so of two toggles < 250 ms apart the session blocks the
    second.

### Remaining (phase 3, task continues)
- **macOS shadow** (`TextVNInputController.swift`, `otherKeyPressed`).
- **Route the batch into the apply path** of all 3 adapters, check SF-G1/G2 on the toggle corpus,
  measure 0 shadow mismatches on e2e, then turn it on by default in the next release.
- **G22 debt**: the TSF adapter still has its own `vk` module in `compose.rs` (it does not depend on
  `textvn-core`); moving it into core is a separate cleanup.
