<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-5 — assist (Trợ lý ngữ pháp) vào `textvn-session` (claude-opus-c4c5a567)

## VI

### Pha 1a — luật quyết định MỘT nguồn; TSF dùng lại — XONG trong commit này
- **`session/src/assist.rs`**: chuyển NGUYÊN VĂN các hàm thuần của TSF `assist.rs`/`panel_idle.rs`, vốn
  là bản có test đầy đủ nhất:
  - trigger D11/AS-022 (`assist_trigger`);
  - idle 1 s R2-154/155 (`idle_arms`);
  - ngữ nghĩa Mode R-5/R-6 (`mode2_auto_fix_enabled`, `mode1_display_enabled`,
    `should_auto_fix_sentence`, `should_show_overlay`);
  - phím trên bảng gợi ý R2-129/P9 (`overlay_action`, `overlay_action_for`, `is_typing_key`,
    `OverlayAction`, `PopupKind`).

  Hằng VK/MOD lấy từ `textvn_core`, `AssistMode` lấy từ `textvn-config` (phụ thuộc mới đã có trong
  P0-6 §2).
- **`live_action_vk_ch`**: biến thể theo `(vk, ch)` cho IBus/Fcitx5, nơi không có bảng VK OEM tin được.
  Test mã hoá ĐÚNG hành vi `lc_assist_live_key` hiện tại. `Ctrl+Shift+G` (gọi lại) chỉ TSF có UI;
  adapter khác sẽ coi như `Dismiss` (năng lực OS).
- **TSF** `pub use` lại cùng tên; 176 dòng trùng bị xoá. Bộ test TSF cũ (130) vẫn khoá hành vi.
- **Test**: `session/tests/assist.rs` có 5 nhóm (D11, R2-154, R-5/R-6, R2-129 live/on-demand/selection,
  biến thể vk/ch). Preflight 29/30 PASS (homebrew SKIP local).

### Còn lại (pha 1b+)
- Linux: `lc_assist_live_key(keysym,…)` và `lc_assist_idle_arms` gọi session qua C ABI. Cần đưa
  `(vk, ch)` từ keymap adapter, viết lại test C theo `(vk, ch)`.
- macOS: `assistIdleArms` qua C ABI. macOS hiện đếm cả ký tự C1 là chữ, lệch nhẹ với TSF/Linux.
- Trạng thái trợ lý trong session: undo chỉ sống tới phím kế (R-3), Ctrl+Space sửa câu (R2-131), Mode EN
  không sửa câu (R2-143), timer idle qua `TimerStart`/`Timer`. Kèm corpus `corpus/assist` và e2e trợ lý
  3 OS.

## EN

### Phase 1a — ONE source of decision rules; TSF reuses it — DONE in this commit
- **`session/src/assist.rs`**: the pure functions of TSF `assist.rs`/`panel_idle.rs`, the most
  thoroughly tested copy, moved VERBATIM:
  - D11/AS-022 triggers (`assist_trigger`);
  - R2-154/155 1-second idle (`idle_arms`);
  - R-5/R-6 Mode semantics (`mode2_auto_fix_enabled`, `mode1_display_enabled`,
    `should_auto_fix_sentence`, `should_show_overlay`);
  - R2-129/P9 keys on the suggestion panel (`overlay_action`, `overlay_action_for`, `is_typing_key`,
    `OverlayAction`, `PopupKind`).

  VK/MOD constants come from `textvn_core`, `AssistMode` from `textvn-config` (a new dependency
  already listed in P0-6 §2).
- **`live_action_vk_ch`**: a `(vk, ch)` variant for IBus/Fcitx5, which have no reliable OEM VK table.
  Its test encodes EXACTLY the current `lc_assist_live_key` behaviour. `Ctrl+Shift+G` (recall) has UI
  only on TSF; other adapters will treat it as `Dismiss` (OS capability).
- **TSF** `pub use`s the same names; 176 duplicated lines removed. The existing TSF tests (130) still
  lock the behaviour.
- **Tests**: `session/tests/assist.rs` has 5 groups (D11, R2-154, R-5/R-6, R2-129 live/on-demand/
  selection, vk/ch variant). Preflight 29/30 PASS (homebrew SKIP local).

### Remaining (phase 1b+)
- Linux: `lc_assist_live_key(keysym,…)` and `lc_assist_idle_arms` call the session through the C ABI.
  This needs `(vk, ch)` from the adapter keymap and the C tests rewritten around `(vk, ch)`.
- macOS: `assistIdleArms` through the C ABI. macOS currently counts C1 characters as text, a slight
  difference from TSF/Linux.
- Assistant state in the session: undo lives only until the next key (R-3), Ctrl+Space fixes the
  sentence (R2-131), Mode EN does not fix sentences (R2-143), idle timer via `TimerStart`/`Timer`.
  Plus the `corpus/assist` corpus and 3-OS assistant e2e.
