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

### Pha 3a — shadow macOS (IMK) — XONG
- **`MacModifierTap`** (`IMKLib/ModifierTap.swift`): tách nguyên văn logic `flagsChanged` cũ
  (Ctrl+Shift nhấn-thả, Ctrl tap ≤ 400 ms, debounce 250 ms, `otherKeyPressed`) khỏi
  `TextVNInputController.handle` thành struct thuần, có test; hành vi không đổi.
- **`SessionShadow`** (`CoreBridge/SessionShadow.swift`): bọc C ABI `ime_session_*`, bật bằng
  `TEXTVN_SESSION_SHADOW=1`. IMK chỉ báo bộ cờ modifier, nên shadow dựng lại key-down/key-up từ chênh
  lệch cờ (Option/Cmd ⇒ `VK_MENU`), rồi so "có bật/tắt hay không" ở `flagsChanged` và ở keyDown
  (chord `Ctrl+Shift+Space`). VK lấy từ bảng sinh mã `KeyMapMacGenerated` (G22). G8: log chỉ ghi pha
  và số đếm.
- **Test** `SessionShadowTests`: 8 kịch bản (gồm debounce, Option chen giữa, chord, mất focus) ⇒ 4 lần
  bật/tắt, **0 lệch**; shadow bắt được lệch giả lập; tắt mặc định.

### Pha 3b — corpus toggle SF-G1/G2 + e2e shadow — XONG
- **`corpus/toggle/toggle.cases`**: 16 ca, MỘT nguồn kịch bản cho mọi harness (G22). Cú pháp ghi ở đầu
  tệp; cột `<os>=` chỉ dành cho khác biệt đã biết, lý do ghi ở `corpus/traces/ALLOWED_DIFF.md` mục SF-2.
- **`session/src/toggle_corpus.rs`**: parse và dựng sự kiện (VK + modifier đang giữ + `now_ms`), không
  I/O. **SF-G1** `session/tests/toggle_corpus.rs`: session khớp vết vàng 16/16; có test đột biến vết
  vàng.
- **SF-G2**, mỗi OS đọc cùng tệp:
  - TSF `sf2_toggle_corpus_tsf_legacy_vs_session`: `ModifierToggle` + debounce + preserved key;
  - Linux `test_compose.c::test_toggle_corpus`: `lc_modifier_toggle` + shadow C ABI; gốc repo
    suy từ `__FILE__`;
  - macOS `SessionShadowTests.testToggleCorpus`: `MacModifierTap` + `SessionShadow`, chuyển `+X/-X`
    thành bộ cờ `flagsChanged`.

  Mã cũ từng OS và session phải ra đúng số ghi.
- **Debounce TSF**: tách lõi thuần `compose::claim_hotkey_toggle_at` +
  `HOTKEY_TOGGLE_DEBOUNCE_MS`; `ipc_client::try_claim_hotkey_toggle` gọi lại nó, hành vi không đổi.
- **Khác biệt đã biết** (3): `alt_tap_between` `mac=0`, `chord_with_alt` `linux=1`,
  `double_toggle_fast` `linux=2`.
- **Đột biến đã thử (D3)**:
  - debounce session = 0 ⇒ `double_toggle_fast` đỏ;
  - bỏ điều kiện Alt khi Shift xuống ⇒ `alt_held_first` đỏ ở harness TSF.
- **e2e IBus (Xfce/X11)** `scripts/e2e-desktop.sh`:
  - chạy với `TEXTVN_SESSION_SHADOW=1`;
  - ca mới `toggle_case`: Ctrl+Shift tắt ⇒ gõ `vieetj` ra thô ⇒ Ctrl+Shift bật ⇒ `việt`;
  - `shadow_check` đỏ khi log có `SF2 shadow mismatch`; engine in `SF2 shadow on` để chứng minh
    shadow có chạy.

### Còn lại (task tiếp tục / bản kế tiếp)
- **Đưa `EditBatch` vào đường áp lệnh**: đi cùng SF-3. Lệnh xoá/commit chỉ phát sinh khi compose chạy
  trong session; trước đó Linux vẫn dùng `lc_edit_batch` (C) cùng luật.
- **Bật session mặc định** ở bản kế tiếp, sau khi chốt 3 khác biệt đã biết; **xoá mã cũ** ở bản sau
  nữa (P0-7 SF-2 "Bước").
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

### Phase 3a — macOS shadow (IMK) — DONE
- **`MacModifierTap`** (`IMKLib/ModifierTap.swift`): the old `flagsChanged` logic (Ctrl+Shift
  press-release, Ctrl tap ≤ 400 ms, 250 ms debounce, `otherKeyPressed`) moved verbatim out of
  `TextVNInputController.handle` into a pure, tested struct; behaviour unchanged.
- **`SessionShadow`** (`CoreBridge/SessionShadow.swift`): wraps the `ime_session_*` C ABI, enabled with
  `TEXTVN_SESSION_SHADOW=1`. IMK only reports the modifier flag set, so the shadow rebuilds key-down/
  key-up from the flag difference (Option/Cmd ⇒ `VK_MENU`), then compares "toggled or not" on
  `flagsChanged` and on keyDown (`Ctrl+Shift+Space` chord). VKs come from the generated
  `KeyMapMacGenerated` table (G22). G8: the log only records the phase and counters.
- **Test** `SessionShadowTests`: 8 scenarios (including debounce, Option in between, chord, focus loss)
  ⇒ 4 toggles, **0 mismatches**; the shadow catches a simulated mismatch; off by default.

### Phase 3b — SF-G1/G2 toggle corpus + e2e shadow — DONE
- **`corpus/toggle/toggle.cases`**: 16 cases, ONE scenario source for every harness (G22). The syntax
  is described at the top of the file; the `<os>=` column is only for known differences, explained in
  `corpus/traces/ALLOWED_DIFF.md` section SF-2.
- **`session/src/toggle_corpus.rs`**: parses the file and builds events (VK + held modifiers +
  `now_ms`), no I/O. **SF-G1** `session/tests/toggle_corpus.rs`: the session matches the golden counts
  16/16; includes a golden-mutation test.
- **SF-G2**, each OS reads the same file:
  - TSF `sf2_toggle_corpus_tsf_legacy_vs_session`: `ModifierToggle` + debounce + preserved key;
  - Linux `test_compose.c::test_toggle_corpus`: `lc_modifier_toggle` + C ABI shadow; repo root
    derived from `__FILE__`;
  - macOS `SessionShadowTests.testToggleCorpus`: `MacModifierTap` + `SessionShadow`, turning `+X/-X`
    into `flagsChanged` flag sets.

  Each OS's old code and the session must produce the recorded counts.
- **TSF debounce**: pure core `compose::claim_hotkey_toggle_at` + `HOTKEY_TOGGLE_DEBOUNCE_MS` extracted;
  `ipc_client::try_claim_hotkey_toggle` calls it, behaviour unchanged.
- **Known differences** (3): `alt_tap_between` `mac=0`, `chord_with_alt` `linux=1`,
  `double_toggle_fast` `linux=2`.
- **Mutations tried (D3)**:
  - session debounce = 0 ⇒ `double_toggle_fast` red;
  - dropping the Alt condition on Shift down ⇒ `alt_held_first` red in the TSF harness.
- **IBus e2e (Xfce/X11)** `scripts/e2e-desktop.sh`:
  - runs with `TEXTVN_SESSION_SHADOW=1`;
  - new `toggle_case`: Ctrl+Shift off ⇒ typing `vieetj` stays raw ⇒ Ctrl+Shift on ⇒ `việt`;
  - `shadow_check` fails when the log has `SF2 shadow mismatch`; the engine prints `SF2 shadow on` to
    prove the shadow actually ran.

### Remaining (task continues / next release)
- **Route `EditBatch` into the apply path**: goes with SF-3. Delete/commit commands only appear once
  compose runs in the session; until then Linux keeps `lc_edit_batch` (C) with the same rules.
- **Turn the session on by default** in the next release, after settling the 3 known differences;
  **remove the old code** in the release after that (P0-7 SF-2 "Steps").
- **G22 debt**: the TSF adapter still has its own `vk` module in `compose.rs` (it does not depend on
  `textvn-core`); moving it into core is a separate cleanup.
