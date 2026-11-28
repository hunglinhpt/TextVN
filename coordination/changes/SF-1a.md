<!-- TASK: SF-1a · commit: <SHA_CODE> · agent: zcode-glm-sf1a0a01 (tác giả) + claude-opus-7e41b2d9 (tiếp nhận sau hết lease) -->
## VI
- **SF-1a — Khung crate `textvn-session` (P0-6 §3)**: crate mới `session/` (`textvn-session`) vào
  workspace member, `publish = false`, thuần logic — KHÔNG I/O (`std::fs`/mạng), không thread, không
  đồng hồ (`#![forbid(unsafe_code)]`, thời gian đến qua `now_ms` của sự kiện). Định nghĩa đủ 4 nhóm
  kiểu P0-6 §3: `Caps` (6 năng lực adapter), `SessionEvent` (10 variant: KeyDown/KeyUp/FocusIn/
  FocusOut/CaretMoved/Surrounding/Reset{Toolkit,Focus,App}/Timer/Ipc/AssistAction), `SessionCommand`
  (13 variant: SetPreedit/Commit/DeleteBefore/ForwardKey/EatKey/Ghost/GhostAfterKeyUp/AssistShow/
  AssistHide/TimerStart/TimerCancel/Ipc/ModeIndicator), `Session` (khung rỗng). Kiểu phụ khung:
  `ResetKind`, `TimerId`, `AssistAction`, `AssistItem` — tên chốt ở SF-1a theo spec; SF-5 chốt lại
  bộ `AssistAction` khi chuyển block assist.
- **Session rỗng trả `ForwardKey`**: hợp đồng khung — `KeyDown` ⇒ đúng một `ForwardKey`; mọi event
  khác ⇒ chuỗi rỗng. Chưa có logic nghiệp vụ thật (SF-2+ chuyển toggle/batch/compose/ghost/assist).
- **Luật đầu ra thắt chặt ở cấp cấu trúc**: `Session::event` trả `Commands` — trình dựng chặn bằng
  `debug_assert` ngay lúc `push`: tối đa 1 `DeleteBefore` + tối đa 1 `Commit` mỗi event (R2-165),
  `Commit` đứng TRƯỚC ẩn preedit `SetPreedit{""}` (R2-167), đúng MỘT trong `ForwardKey`/`EatKey`
  cho mỗi `KeyDown`. Mọi block SF-2+ không thể phá luật một cách vô ý.
- **Dependency (lý do theo P0-7 §4.3)**: KHÔNG crate crates.io mới. Duy nhất path-dep nội bộ
  `textvn-ipc` vì P0-6 §3 định kiểu `Ipc(textvn_ipc::Message)` cho cả `SessionEvent` lẫn
  `SessionCommand`; serde/serde_json (và `windows` trên target Windows) là phụ thuộc bậc hai đã có
  sẵn trong đồ thị của MỌI adapter tiêu thụ session ⇒ `cargo deny check` sạch, không giấy phép mới.
- **R12**: đã notify `*` trước khi push (thêm crate member mới, không đổi ABI/đường gọi cũ); SF-1b
  sẽ thêm C ABI `ime_session_*` vào `ffi/`.
## EN
- **SF-1a — `textvn-session` crate skeleton (P0-6 §3)**: new crate `session/` (`textvn-session`)
  as workspace member, `publish = false`, pure logic — NO I/O (`std::fs`/network), no threads, no
  clock (`#![forbid(unsafe_code)]`, time arrives via the event's `now_ms`). Defines all 4 type
  groups of P0-6 §3: `Caps` (6 adapter capabilities), `SessionEvent` (10 variants: KeyDown/KeyUp/
  FocusIn/FocusOut/CaretMoved/Surrounding/Reset{Toolkit,Focus,App}/Timer/Ipc/AssistAction),
  `SessionCommand` (13 variants: SetPreedit/Commit/DeleteBefore/ForwardKey/EatKey/Ghost/
  GhostAfterKeyUp/AssistShow/AssistHide/TimerStart/TimerCancel/Ipc/ModeIndicator), `Session`
  (empty frame). Helper types: `ResetKind`, `TimerId`, `AssistAction`, `AssistItem` — names
  finalized at SF-1a per spec; SF-5 finalizes the `AssistAction` set when porting the assist block.
- **Empty session returns `ForwardKey`**: frame contract — `KeyDown` ⇒ exactly one `ForwardKey`;
  every other event ⇒ empty command list. No real business logic yet (SF-2+ moves
  toggle/batch/compose/ghost/assist in).
- **Output laws enforced structurally**: `Session::event` returns `Commands` — the builder rejects
  violations at `push` time via `debug_assert`: at most 1 `DeleteBefore` + 1 `Commit` per event
  (R2-165), `Commit` BEFORE hiding preedit `SetPreedit{""}` (R2-167), exactly ONE of
  `ForwardKey`/`EatKey` per `KeyDown`. Later SF-2+ blocks cannot silently break the laws.
- **Dependency (reason per P0-7 §4.3)**: NO new crates.io crates. Single in-workspace path dep
  `textvn-ipc` because P0-6 §3 types `Ipc(textvn_ipc::Message)` in both `SessionEvent` and
  `SessionCommand`; serde/serde_json (and `windows` on Windows targets) are second-order deps
  already present in every session consumer's graph ⇒ `cargo deny check` clean, no new licenses.
- **R12**: notified `*` before pushing (new member crate, no ABI/call-path change); SF-1b will add
  the C ABI `ime_session_*` to `ffi/`.
## Tiếp nhận / Takeover (claude-opus-7e41b2d9, 2026-11-28)
- VI: lease của zcode-glm-sf1a0a01 hết 09:47Z, WIP `session/` không đổi từ 01:55Z, không trả lời thông báo
  trên board ⇒ nhận lại theo R10. Mã nguồn `session/` là của zcode, GIỮ NGUYÊN; phần tôi làm: thêm
  `"session"` vào `members` của `Cargo.toml` gốc + `Cargo.lock` (bị chặn trước đó vì `Cargo.toml` đang được
  reserve), xoá `session/target` lạc, chạy lại toàn bộ kiểm chứng và audit chéo: 14/14 test; đột biến lần lượt
  4 `debug_assert` của `Commands::push` (2 Commit, Commit sau khi ẩn preedit, 2 DeleteBefore, 2 quyết định
  phím) ⇒ mỗi đột biến làm đỏ thêm đúng 1 test; clippy `-D warnings`, rustfmt, `cargo deny check` sạch;
  `cargo xtask preflight` 28 bước.
- EN: zcode-glm-sf1a0a01's lease expired at 09:47Z, the `session/` WIP had not changed since 01:55Z and the
  board notice got no reply ⇒ reclaimed under R10. The `session/` source is zcode's, KEPT AS IS; my part: add
  `"session"` to the root `Cargo.toml` `members` + `Cargo.lock` (previously blocked because `Cargo.toml` was
  reserved), delete a stray `session/target`, re-run every check plus a cross-audit: 14/14 tests; mutating each
  of the 4 `debug_assert`s in `Commands::push` (2 Commits, Commit after hiding preedit, 2 DeleteBefores, 2 key
  decisions) ⇒ each mutant turns exactly one more test red; clippy `-D warnings`, rustfmt, `cargo deny check`
  clean; `cargo xtask preflight` 28 steps.
## Tài liệu / Docs
- Không sửa CHANGELOG/README (tệp nóng R3 — release manager gộp ở bản phát hành kế).
- `docs/10-shared/P0-6-SESSION-FRAMEWORK-SPEC.md` §3 đã triển khai nguyên vẹn; SF-1b (C ABI) là
  task kế trong luồng SF.
- Crate doc (`session/src/lib.rs`) ghi đủ 5 bất biến kiến trúc P0-6 §2 và luật đầu ra §3.
## Kiểm thử / Testing
- `cargo test -p textvn-session` = **14 passed** (`session/tests/invariants.rs`, PRNG splitmix64
  tự viết — không dev-dep): 4 test thuộc tính trên stream 64×512 event phủ đủ 10 loại event
  (≤1 xoá + ≤1 commit; commit trước ẩn preedit; đúng một quyết định phím cho `KeyDown`, không có
  cho event khác; secure ⇒ không ghost/assist/IPC — S3), 4 test `should_panic` chặn luật ở
  `Commands::push`, 6 test hợp đồng khung rỗng/Caps/API/tất định 2 lượt chạy giống hệt.
- Đột biến D3: bỏ `debug_assert` R2-165 trong `Commands::push` (cho phép 2 Commit) ⇒ test
  `commands_tu_choi_2_commit` ĐỎ (`1 failed`); đã khôi phục mã nguyên trạng ⇒ 14/14 xanh lại.
- `cargo clippy -p textvn-session --all-targets -- -D warnings` = sạch; `cargo fmt --check` = sạch
  (rustfmt áp cho `session/src/lib.rs` + `session/tests/invariants.rs`).
- `cargo deny check` = sạch (không crate mới; ban/license/advisory/source không đổi).
