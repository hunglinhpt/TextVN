<!-- TASK: BOARD-1 · commit: 2ec75492 · agent: zcode-glm-9f2c4a17 -->
## VI
- **BOARD-1 — Điều phối độc lập với build sản phẩm (E83)**: tách `board` khỏi `xtask` thành crate
  mới `board/` (`textvn-board`) chỉ phụ thuộc `std` — KHÔNG `textvn-core`, KHÔNG serde. WIP hỏng
  của một agent không còn chặn điều phối của mọi agent nữa. Ba đường gọi tương đương: `cargo board`
  (sau `cargo install --path board` — binary `cargo-board` vào `~/.cargo/bin`), `cargo run -p
  textvn-board -- <lệnh>` (build riêng, chạy được khi `core/` hỏng), và `cargo xtask board`
  (mượn cùng lib — một nguồn sự thật, G22).
- 10/10 unit test của lib board chuyển nguyên sang crate mới (không đổi logic); help text ghi rõ 3
  đường gọi; `coordination/README.md` §1 (E83) + entry E83 trong `project-common-errors.md` cập
  nhật với bằng chứng đột biến.
## EN
- **BOARD-1 — Coordination independent of product build (E83)**: split `board` out of `xtask` into
  a new crate `board/` (`textvn-board`) depending only on `std` — no `textvn-core`, no serde. A
  broken WIP from one agent no longer blocks every agent's coordination. Three equivalent entry
  points: `cargo board` (after one-time `cargo install --path board` — `cargo-board` binary into
  `~/.cargo/bin`), `cargo run -p textvn-board -- <cmd>` (builds only this crate, works even when
  `core/` is broken), and `cargo xtask board` (borrows the same lib — single source of truth, G22).
- All 10 board unit tests moved unchanged into the new crate (no logic change); help text lists the
  three entry points; `coordination/README.md` §1 (E83) + the E83 entry in
  `project-common-errors.md` updated with mutation evidence.
## Tài liệu / Docs
- `coordination/README.md` §1 — E83: 3 đường gọi mới thay lời khuyên "dùng bản đã build".
- `docs/specs/project-common-errors.md` — E83 thêm "đã sửa tận gốc @BOARD-1" + đột biến.
## Kiểm thử / Testing
- `cargo build -p textvn-board` + `cargo test -p textvn-board` = 10 passed.
- Đột biến: thêm lỗi cú pháp vào `core/src/lib.rs` ⇒ `cargo xtask board status` đỏ (kỳ vọng —
  xtask phụ thuộc core) nhưng `cargo run -p textvn-board -- status --agent …` vẫn in đủ trạng thái
  board; đã khôi phục `core/src/lib.rs` nguyên trạng sau thử nghiệm.
- `cargo install --path board --offline` ⇒ `cargo-board.exe` cài thành công (alias `cargo board`).
