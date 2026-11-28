<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SIM-1 — Một bộ mô phỏng văn bản dùng chung (claude-opus-c4c5a567, nhận lại sau hết lease R10)

## VI

- **Mới**: `core/src/sim.rs` — `textvn_core::sim::{apply, apply_from}` (`#[doc(hidden)]`, thuần, không
  I/O). Áp `Action` mà engine trả về vào buffer như một app thật. Ngữ nghĩa là **hợp chặt nhất**
  của các bản sao cũ:
  - `Pass` + Backspace ⇒ app xoá 1 ký tự (kể cả văn bản có sẵn, vì đó là phím người dùng);
  - Ctrl+Space là hợp âm, không chèn;
  - ký tự điều khiển bị bỏ, trừ `\n \r \t`;
  - `Replace`/`Restore` xoá vượt phần gõ được (dưới `floor`) ⇒ `panic!`, vì đó là lỗi sổ engine
    (nhân đôi/mất chữ, xoá lẹm). Không che bằng `saturating_sub` như vài bản cũ.
- **Xoá 12 bản sao** (G22):
  - 9 `fn apply` trong `core/tests/*`. `invariants.rs` giữ hàm bọc `apply_checked`, chỉ còn các
    assert MAX_TEXT riêng, rồi gọi `sim::apply_from(doc, base, …)`.
  - bản trong `mod tests` của `core/src/lib.rs`, cùng logic Backspace dựng tay trong `backspaces()`.
  - `sim_apply` của `tools/bench` (bản thứ 13).
- **Acceptance**: `rg "fn apply\(" core/tests` = **0**.
- **Lộ ra 2 test dựa vào lỗ của bản cũ**: `backspace_after_sentence_end_cancels_auto_capitalize`
  và `auto_capitalize_needs_space_after_dot` tự `buf.pop()` sau Backspace, vì bản `apply` cũ trong
  `lib.rs` không mô phỏng app xoá. Đã bỏ pop thủ công; kết quả mong đợi giữ nguyên.
- **Bất biến số**:
  - `textvn-bench dream-rsi` JSON **trùng khít** `perf/baseline-win.json`;
  - `cargo test -p textvn-core` 31/31 binary (202 test lib);
  - `cargo test --workspace --exclude xtask` xanh; clippy `-D warnings` và rustfmt sạch.
- Test mới trong `sim.rs`: Pass/Backspace/Ctrl+Space/ký tự điều khiển; Replace/Commit; xoá lẹm
  `floor` ⇒ panic.

## EN

- **New**: `core/src/sim.rs` — `textvn_core::sim::{apply, apply_from}` (`#[doc(hidden)]`, pure, no
  I/O). It applies the `Action` returned by the engine to a buffer the way a real app does. The
  semantics are the **strictest union** of the old copies:
  - `Pass` + Backspace ⇒ the app deletes one char (pre-existing text too, since it is a user key);
  - Ctrl+Space is a chord and inserts nothing;
  - control chars are dropped except `\n \r \t`;
  - a `Replace`/`Restore` deleting past the typed part (below `floor`) ⇒ `panic!`, because it is an
    engine ledger bug (doubled/lost text, over-deletion). It is not masked with `saturating_sub`
    as some old copies did.
- **Removed 12 copies** (G22):
  - 9 `fn apply` in `core/tests/*`. `invariants.rs` keeps an `apply_checked` wrapper that only adds
    its own MAX_TEXT asserts, then calls `sim::apply_from(doc, base, …)`.
  - the copy in `core/src/lib.rs` `mod tests`, plus the hand-rolled Backspace logic in
    `backspaces()`.
  - `tools/bench` `sim_apply` (the 13th copy).
- **Acceptance**: `rg "fn apply\(" core/tests` = **0**.
- **2 tests relied on a gap in the old copy**: `backspace_after_sentence_end_cancels_auto_capitalize`
  and `auto_capitalize_needs_space_after_dot` called `buf.pop()` themselves after Backspace, because
  the old `lib.rs` `apply` did not model the app deleting. The manual pops are removed; the expected
  results are unchanged.
- **Unchanged numbers**:
  - `textvn-bench dream-rsi` JSON is **identical** to `perf/baseline-win.json`;
  - `cargo test -p textvn-core` 31/31 binaries (202 lib tests);
  - `cargo test --workspace --exclude xtask` is green; clippy `-D warnings` and rustfmt are clean.
- New tests in `sim.rs`: Pass/Backspace/Ctrl+Space/control chars; Replace/Commit; deleting below
  `floor` ⇒ panic.
