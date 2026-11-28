<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# SF-6 (Phần 1): Luật nhận diện ô nhập Linux/macOS & Client IPC dùng chung

> Ghi chú cho người gộp phát hành (`G21`): đây là **Phần 1** của `SF-6` (mảng crate Rust dùng chung `field-detect/` + `ipc/`). **Phần 2** (nối `ffi/include/textvn_ffi.h`, `adapters/linux-common/`, `adapters/macos-imk/`, `adapters/windows-tsf/` vào lõi mới và xoá luật trùng lặp phía C/Swift) đang chờ `SF-3` giải phóng giữ chỗ `ffi/`.

## Tiếng Việt

### Tóm tắt thay đổi
1. **`textvn-field-detect` (`field-detect/src/{rules_linux.rs,rules_mac.rs,lib.rs}`)**
   - `rules_linux.rs`: chuyển 1:1 từ `adapters/linux-common/src/field_detect.c` gồm danh sách 20 ứng dụng terminal (`TERMINAL_APPS`), so khớp ranh giới từ không phân biệt hoa/thường (`has_word_icase`, `is_terminal_app`), cấu trúc `AtspiSnapshot` và hàm thuần `classify(Option<&AtspiSnapshot>) -> LinuxProbeResult` theo đúng thứ tự ưu tiên R1 → R6 → R2 → R3 → Chat (Bug B2) → R4 → R5 → R8 → R7 → R9 → Fallback (`strategy_hint = -1`), hỗ trợ chuẩn hoá Unicode cho nhãn tiếng Việt (`"Địa chỉ"`, `"Tìm kiếm"`, `"Soạn tin"`).
   - `rules_mac.rs`: chuyển 1:1 từ `adapters/macos-imk/Sources/IMKLib/FieldDetect.swift` và `TextVNInputController.isTerminalApp` gồm 10 bundle ID terminal macOS (`TERMINAL_BUNDLES`), `is_terminal_app`, `normalize_mac_app_id`, cấu trúc `AxSnapshot` và `classify(Option<&AxSnapshot>) -> MacProbeResult` cho R1–R10.
   - `lib.rs`: xuất `rules_linux` và `rules_mac`, thêm `FieldContext::apply_linux_atspi` và `FieldContext::apply_macos_ax` bảo đảm generation-guard và S3 security gate.
2. **`textvn-ipc` (`ipc/src/{client.rs,lib.rs}`)**
   - `client.rs`: gom logic client IPC dùng chung gồm `GLOBAL_APP_ID` (`"*"`), `ASSIST_APP_ID` (`"assist"`), bộ lọc sandbox `allowed_in_sandbox` & `encode_outgoing` (E80/E81/R2-26: chặn `StyleUpdate`, `AssistReport`, `AssistNotification`, `Shutdown`…), bộ đệm frame `FrameBuffer` cho socket/pipe non-blocking, máy trạng thái `IpcClientState` (`Snapshot` / `StateUpdate` / `ConfigReload` / đổi `app_id`), `ReconnectBackoff` luỹ tiến có trần (100 ms → 5 000 ms), `parse_state_json` (Phần 1.5: phân tích `state.json` dùng chung), và resolver đường dẫn socket `resolve_linux_socket_path` (giữ guard đường dẫn tuyệt đối C9) + `resolve_macos_socket_path`.
3. **Phần 1.5 — Nối Windows TSF (`adapters/windows-tsf/src/ipc_client.rs`) vào `textvn_ipc::client`**
   - Dùng trực tiếp `GLOBAL_APP_ID`, `ASSIST_APP_ID`, `allowed_in_sandbox` (trên `notify_assist_async` và `send_command_async`, vẫn giữ nguyên `send_style_update_async` dùng `style_drain_rsi_only` để không mất bộ đếm Dream RSI G19 qua sandbox), `parse_state_json` (xoá bản sao cục bộ), và `ReconnectBackoff::new(2s, 10s)` trong `run_client_loop`.

### Kiểm chứng
- `cargo fmt -p textvn-field-detect -p textvn-ipc -- --check`: PASS
- `cargo clippy -p textvn-field-detect -p textvn-ipc --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo clippy --target x86_64-unknown-linux-gnu -p textvn-field-detect -p textvn-ipc -- -D warnings`: PASS (0 warnings)
- `cargo test -p textvn-field-detect`: 25/25 PASS
- `cargo test -p textvn-ipc`: 21/21 PASS
- Phần 1.5 (`textvn-ipc` + `textvn-win-tsf`): `cargo fmt -p textvn-ipc -p textvn-win-tsf -- --check`, `cargo clippy` Windows + `--target x86_64-unknown-linux-gnu` (`-D warnings`), `cargo test -p textvn-ipc` (21/21 PASS), `cargo test -p textvn-win-tsf ipc_client` (7/7 PASS)
- `cargo run -q -p xtask -- regression-index --todo-max 0`: PASS (241 mã, 0 TODO)
- `cargo run -q -p xtask -- hygiene-docs`, `hygiene-secrets`, `hygiene-apis`: PASS

### Phần còn lại (Phần 2 — mở khi `SF-3` nhả `ffi/`)
- Xuất C ABI (`textvn-ffi`) + `corebridge` cho `field-detect` và `ipc` client (`FrameBuffer`, `IpcClientState`, socket path, sandbox gate).
- Nối `adapters/linux-common/src/{field_detect.c,ipc_client.c}`, `adapters/macos-imk/Sources/IMKLib/{FieldDetect.swift,TextVNInputController.swift,IpcClient.swift}`, `adapters/windows-tsf/src/ipc_client.rs` gọi 1 nguồn duy nhất và chạy test 3 OS.

---

# English

> Note for release coordinator (`G21`): this is **Part 1** of `SF-6` (shared Rust crates `field-detect/` + `ipc/`). **Part 2** (wiring `ffi/include/textvn_ffi.h`, `adapters/linux-common/`, `adapters/macos-imk/`, `adapters/windows-tsf/` to the shared crates and removing C/Swift duplicates) waits for `SF-3` to release its `ffi/` reservation.

## Summary of Changes
1. **`textvn-field-detect` (`field-detect/src/{rules_linux.rs,rules_mac.rs,lib.rs}`)**
   - `rules_linux.rs`: 1:1 port of `adapters/linux-common/src/field_detect.c` including the 20-entry Linux terminal list (`TERMINAL_APPS`), ASCII word-boundary case-insensitive matching (`has_word_icase`, `is_terminal_app`), `AtspiSnapshot`, and pure `classify(Option<&AtspiSnapshot>) -> LinuxProbeResult` preserving exact priority R1 → R6 → R2 → R3 → Chat (Bug B2) → R4 → R5 → R8 → R7 → R9 → Fallback (`strategy_hint = -1`), with full Unicode lowercase handling for Vietnamese labels (`"Địa chỉ"`, `"Tìm kiếm"`, `"Soạn tin"`).
   - `rules_mac.rs`: 1:1 port of `adapters/macos-imk/Sources/IMKLib/FieldDetect.swift` and `TextVNInputController.isTerminalApp` including the 10 macOS terminal bundle IDs (`TERMINAL_BUNDLES`), `is_terminal_app`, `normalize_mac_app_id`, `AxSnapshot`, and `classify(Option<&AxSnapshot>) -> MacProbeResult` for R1–R10.
   - `lib.rs`: exports `rules_linux` and `rules_mac`, adds `FieldContext::apply_linux_atspi` and `FieldContext::apply_macos_ax` preserving generation-guard and S3 security gate semantics.
2. **`textvn-ipc` (`ipc/src/{client.rs,lib.rs}`)**
   - `client.rs`: shared IPC client building blocks including `GLOBAL_APP_ID` (`"*"`), `ASSIST_APP_ID` (`"assist"`), sandbox filter `allowed_in_sandbox` & `encode_outgoing` (E80/E81/R2-26 blocking `StyleUpdate`, `AssistReport`, `AssistNotification`, `Shutdown`, etc.), non-blocking `FrameBuffer` accumulator, `IpcClientState` state machine (`Snapshot` / `StateUpdate` / `ConfigReload` / app-switch recalculation), capped exponential `ReconnectBackoff` (100 ms → 5,000 ms), `parse_state_json` (Part 1.5 shared `state.json` parser), and socket path resolvers `resolve_linux_socket_path` (with C9 absolute-path guards) + `resolve_macos_socket_path`.
3. **Part 1.5 — Windows TSF wiring (`adapters/windows-tsf/src/ipc_client.rs`) to `textvn_ipc::client`**
   - Wired directly to `GLOBAL_APP_ID`, `ASSIST_APP_ID`, `allowed_in_sandbox` (on `notify_assist_async` and `send_command_async`, while keeping `send_style_update_async` intact with `style_drain_rsi_only` so G19 Dream RSI counters still reach tray over sandbox pipes), `parse_state_json` (removed local duplicate), and `ReconnectBackoff::new(2s, 10s)` in `run_client_loop`.

## Verification
- `cargo fmt -p textvn-field-detect -p textvn-ipc -- --check`: PASS
- `cargo clippy -p textvn-field-detect -p textvn-ipc --all-targets -- -D warnings`: PASS (0 warnings)
- `cargo clippy --target x86_64-unknown-linux-gnu -p textvn-field-detect -p textvn-ipc -- -D warnings`: PASS (0 warnings)
- `cargo test -p textvn-field-detect`: 25/25 PASS
- `cargo test -p textvn-ipc`: 21/21 PASS
- Part 1.5 (`textvn-ipc` + `textvn-win-tsf`): `cargo fmt -p textvn-ipc -p textvn-win-tsf -- --check`, `cargo clippy` Windows + `--target x86_64-unknown-linux-gnu` (`-D warnings`), `cargo test -p textvn-ipc` (21/21 PASS), `cargo test -p textvn-win-tsf ipc_client` (7/7 PASS)
- `cargo run -q -p xtask -- regression-index --todo-max 0`: PASS (241 codes, 0 TODOs)
- `cargo run -q -p xtask -- hygiene-docs`, `hygiene-secrets`, `hygiene-apis`: PASS

## Remaining Scope (Part 2 — once `SF-3` releases `ffi/`)
- Expose C ABI (`textvn-ffi`) + `corebridge` bindings for `field-detect` and `ipc` client (`FrameBuffer`, `IpcClientState`, socket path resolution, sandbox gate).
- Wire `adapters/linux-common/src/{field_detect.c,ipc_client.c}`, `adapters/macos-imk/Sources/IMKLib/{FieldDetect.swift,TextVNInputController.swift,IpcClient.swift}`, and `adapters/windows-tsf/src/ipc_client.rs` to this single source of truth and run 3-OS verification.
