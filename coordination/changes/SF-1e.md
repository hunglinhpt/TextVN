<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# SF-1e — Ghi vết vàng adapter macOS (claude-opus-c4c5a567)

## VI

- **`adapters/macos-imk/Tests/IMKLibTests/TraceTests.swift` (mới)**: chạy **engine thật** (CoreBridge)
  cùng `ApplyReplace`/`MarkedState` của IMKLib trên luồng phím do Rust xuất
  (`textvn-cli replay … --adapter mac --emit-keys <dir>`, SF-1c). **Không** viết lại bộ đọc `.keys`
  bằng Swift; bộ đọc duy nhất là `cli/src/replay.rs` (G22).
- `TraceRecordingTarget` (một `TextTarget`) chuẩn hoá thao tác thành đúng 5 lệnh của SF-1c/SF-1d:
  - `setMarked` → `set_preedit` (`n` = số scalar; rỗng = ẩn);
  - `insert` → `commit`, rồi ẩn preedit nếu đang marked (commit đứng TRƯỚC, P0-6 §3);
  - `unmark` → `commit` chữ marked;
  - `deleteBackward`/`deleteSurroundingText` → `delete_before`;
  - quyết định của controller → `eat_key`/`forward_key`.

  Mỗi dòng `{"case","step","cmd","text","n"}` ⇒ `corpus/traces/mac/<rel>.jsonl`. Mỗi ca chạy 2 lần
  phải giống hệt (tất định).
- **Phạm vi** (P0-6 §6.1 “ApplyReplace/Marked trên corpus”): lớp áp outcome. Cổng phím của
  `handle(NSEvent)` (phím bơm, modifier, ô secure, trợ lý) cần `NSEvent` thật, sẽ chuyển vào session ở
  SF-2. Nhánh `.pass` và ranh giới Enter/Tab phản chiếu `TextVNInputController.handle`; đây là
  test parity **tạm** theo G22, xoá khi SF-2 dời logic vào session.
- Hàm dùng thật (không chép): `engineConfig`, `configNonPreedit`, `outputStrategy`,
  `withoutNativeBoundary`, `StrategyResolver`, `imkCaps`, `ImeEngine.reloadConfig`. `:config` giữa ca
  dùng `reloadConfig`, giống `ime_reload_config` của replay Rust.
- **CI** (`ci-macos.yml`, job `swift` arm64): `--emit-keys` → `swift test --filter TraceTests` →
  ≥ 50 vết; artifact `sf1e-traces-mac`. Khi `corpus/traces/mac` đã được commit, bước này chặn lệch
  (`git status --porcelain`).
- Test riêng `testRecordingTargetOrdersCommitBeforeHidingPreedit`: thứ tự commit trước ẩn preedit,
  và escape JSON.

## EN

- **`adapters/macos-imk/Tests/IMKLibTests/TraceTests.swift` (new)**: runs the **real engine**
  (CoreBridge) together with IMKLib's `ApplyReplace`/`MarkedState` on the key stream exported by Rust
  (`textvn-cli replay … --adapter mac --emit-keys <dir>`, SF-1c). It does **not** reimplement the
  `.keys` parser in Swift; the only parser is `cli/src/replay.rs` (G22).
- `TraceRecordingTarget` (a `TextTarget`) normalizes operations into the same 5 commands as
  SF-1c/SF-1d:
  - `setMarked` → `set_preedit` (`n` = scalar count; empty = hide);
  - `insert` → `commit`, then hide the preedit if marked (commit comes FIRST, P0-6 §3);
  - `unmark` → `commit` of the marked text;
  - `deleteBackward`/`deleteSurroundingText` → `delete_before`;
  - the controller's decision → `eat_key`/`forward_key`.

  Each line `{"case","step","cmd","text","n"}` ⇒ `corpus/traces/mac/<rel>.jsonl`. Each case runs twice
  and both runs must match (deterministic).
- **Scope** (P0-6 §6.1 “ApplyReplace/Marked on the corpus”): the outcome-applying layer. The key gate in
  `handle(NSEvent)` (injected keys, modifiers, secure fields, assistant) needs a real `NSEvent` and moves
  into the session in SF-2. The `.pass` branch and the Enter/Tab boundary mirror
  `TextVNInputController.handle`; this is a **temporary** parity harness under G22, to be removed once
  SF-2 moves that logic into the session.
- Real functions reused (not copied): `engineConfig`, `configNonPreedit`, `outputStrategy`,
  `withoutNativeBoundary`, `StrategyResolver`, `imkCaps`, `ImeEngine.reloadConfig`. A mid-case `:config`
  uses `reloadConfig`, like the Rust replay's `ime_reload_config`.
- **CI** (`ci-macos.yml`, `swift` job, arm64): `--emit-keys` → `swift test --filter TraceTests` →
  ≥ 50 traces; artifact `sf1e-traces-mac`. Once `corpus/traces/mac` is committed, the step blocks drift
  (`git status --porcelain`).
- Dedicated test `testRecordingTargetOrdersCommitBeforeHidingPreedit`: commit-before-hide ordering and
  JSON escaping.
