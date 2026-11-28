<!-- TASK: SF-1b · commit: f4a2e85d · agent: claude-opus-7e41b2d9 -->
## VI
- **SF-1b — C ABI `ime_session_*` (P0-6 §4)**: thêm vào CÙNG `libtextvn_ffi` (không thư viện thứ hai)
  4 hàm `ime_session_new` / `ime_session_free` / `ime_session_event` / `ime_session_reload_config` và
  4 struct `#[repr(C)]` `ime_session_config_v1` / `ime_session_event_v1` / `ime_session_cmd_v1` /
  `ime_session_out_v1`, đúng bố cục spec. Hằng: `IME_SESSION_ABI_VERSION = 1`, `IME_SESSION_MAX_CMDS = 16`,
  `IME_SESSION_ARENA = 2048`, mã lỗi mới `IME_ERR_BUFFER = -5`, bit năng lực `IME_SCAP_*` (6), loại sự kiện
  `IME_SEV_*` (10), lệnh `IME_SCMD_*` (13), `IME_SFOCUS_*`, `IME_SRESET_*`, `IME_SASSIST_*`.
- **Hợp đồng**: không callback, không cấp phát chéo — adapter cấp `ime_session_out_v1` trên stack, session
  ghi lệnh + chữ UTF-8 vào `arena[text_off..+text_len]`. Mọi struct có `abi_version` (lệch ⇒ `IME_ERR_ABI`).
  NULL/UTF-8 hỏng/`kind` lạ ⇒ `IME_ERR_INVALID_ARG`. Tràn `MAX_CMDS` hoặc `ARENA` ⇒ `IME_ERR_BUFFER`,
  `out.count = 0` và session được **khôi phục từ ảnh chụp** (không đổi trạng thái — adapter coi như
  `ForwardKey`). Panic bị `catch_unwind` chặn ⇒ `IME_ERR_INTERNAL`, cũng khôi phục ảnh chụp.
- **IPC qua codec chung**: `IME_SEV_IPC`/`IME_SCMD_IPC` mang JSON thân frame `textvn-ipc` (không prefix độ dài)
  — không có bộ mã hoá IPC thứ hai (G22).
- **Chỉ THÊM**: không đổi struct/hàm cũ; IME ABI 20/532 giữ nguyên (preflight `sizes`). R12: đã notify `*`
  trước khi push.
- **`textvn-cli verify`**: thêm 4 export (tổng 34, đúng thứ tự), 4 struct (tên + thứ tự trường), mọi hằng mới,
  và sizeof/offset trên MỌI target: 64-bit (config 24, event 56, now_ms@24, text@32, arg@48), i686 MSVC
  (16, 48, 24, 32, 40 — `u64` canh 8), i386 SysV (16, 40, 20, 28, 36 — `u64` canh 4); `out` = 8 + 16×20 + 2048.
- Header `ffi/include/textvn_ffi.h` và bản Swift `adapters/macos-imk/Sources/CTextVNFFI/include/textvn_ffi.h`
  giống hệt byte (gate `ci-macos.yml` đã có).

## EN
- **SF-1b — C ABI `ime_session_*` (P0-6 §4)**: added to the SAME `libtextvn_ffi` (no second library)
  4 functions `ime_session_new` / `ime_session_free` / `ime_session_event` / `ime_session_reload_config` and
  4 `#[repr(C)]` structs `ime_session_config_v1` / `ime_session_event_v1` / `ime_session_cmd_v1` /
  `ime_session_out_v1`, matching the spec layout. Constants: `IME_SESSION_ABI_VERSION = 1`,
  `IME_SESSION_MAX_CMDS = 16`, `IME_SESSION_ARENA = 2048`, new error code `IME_ERR_BUFFER = -5`, capability bits
  `IME_SCAP_*` (6), event kinds `IME_SEV_*` (10), commands `IME_SCMD_*` (13), `IME_SFOCUS_*`, `IME_SRESET_*`,
  `IME_SASSIST_*`.
- **Contract**: no callbacks, no cross-allocation — the adapter provides `ime_session_out_v1` on its stack, the
  session writes commands + UTF-8 text into `arena[text_off..+text_len]`. Every struct carries `abi_version`
  (mismatch ⇒ `IME_ERR_ABI`). NULL / bad UTF-8 / unknown `kind` ⇒ `IME_ERR_INVALID_ARG`. Overflowing
  `MAX_CMDS` or `ARENA` ⇒ `IME_ERR_BUFFER`, `out.count = 0` and the session is **restored from a snapshot**
  (no state change — the adapter treats it as `ForwardKey`). Panics are caught by `catch_unwind` ⇒
  `IME_ERR_INTERNAL`, also restoring the snapshot.
- **IPC through the shared codec**: `IME_SEV_IPC`/`IME_SCMD_IPC` carry the JSON body of a `textvn-ipc` frame
  (no length prefix) — no second IPC encoder (G22).
- **Additive only**: no existing struct/function changed; IME ABI 20/532 unchanged (preflight `sizes`). R12:
  notified `*` before pushing.
- **`textvn-cli verify`**: 4 new exports (34 total, in order), 4 structs (names + field order), every new
  constant, and sizeof/offset on EVERY target: 64-bit (config 24, event 56, now_ms@24, text@32, arg@48), i686
  MSVC (16, 48, 24, 32, 40 — `u64` aligned to 8), i386 SysV (16, 40, 20, 28, 36 — `u64` aligned to 4);
  `out` = 8 + 16×20 + 2048.
- Header `ffi/include/textvn_ffi.h` and the Swift copy `adapters/macos-imk/Sources/CTextVNFFI/include/textvn_ffi.h`
  are byte-identical (existing `ci-macos.yml` gate).

## Tài liệu / Docs
- Không sửa CHANGELOG/README (tệp phát hành — release manager gộp, `coordination/README.md`).
- `docs/10-shared/P0-6-SESSION-FRAMEWORK-SPEC.md` §4 đã triển khai nguyên vẹn; module doc
  `ffi/src/session_abi.rs` ghi bảng ánh xạ trường theo từng `IME_SEV_*`.

## Kiểm thử / Testing
- `cargo test -p textvn-ffi` = 53 passed, gồm 6 test `sf_1b_*`: KeyDown qua ABI ⇒ `ForwardKey`; tham số sai
  (NULL, `abi_version` lệch, `kind` lạ, reset kind lạ, UTF-8 hỏng) ⇒ đúng mã lỗi; **tràn `MAX_CMDS` (17 lệnh)
  và `ARENA` (4096 byte) ⇒ `IME_ERR_BUFFER` + `count = 0`, vừa khít 2048 byte ⇒ đạt**; mã hoá đủ 13 loại lệnh
  (2 test) + arena UTF-8 tiếng Việt + JSON IPC đúng thân frame codec chung; sự kiện IPC qua codec chung (JSON
  hỏng ⇒ `IME_ERR_INVALID_ARG`).
- **Giới hạn (nói thật)**: đường khôi phục ảnh chụp khi tràn/panic trong `ime_session_event` chưa kích hoạt được
  qua ABI vì session khung rỗng (SF-1a) chỉ sinh ≤1 lệnh; tràn mới kiểm ở `encode_commands`. SF-2 (session có
  trạng thái thật) phải thêm test tràn qua `ime_session_event` và kiểm trạng thái giữ nguyên.
- `textvn-cli verify` PASS trên x86_64-pc-windows-msvc và i686-pc-windows-msvc; `cargo test -p textvn-cli verify`
  11 passed.
- Header biên dịch bằng C thật: MSVC `cl /W4 /WX /std:c11 /TC` + `_Static_assert` 8 kích thước/offset 64-bit ⇒ sạch.
- `cargo clippy -p textvn-ffi -p textvn-cli --all-targets -D warnings` sạch; `cargo xtask preflight` 28 bước.
