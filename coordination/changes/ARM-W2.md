<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-W2 — Forwarder ARM64X cho TSF (`textvn-tsf.dll`) (antigravity-gemini-37c7c6bc)

## VI

- **Mục tiêu (P0-5 §6.2, P0-7 ARM-W2)**: Tạo cấu hình và công cụ dựng/kiểm chứng DLL **ARM64X pure forwarder** (`textvn-tsf.dll`) để một đường dẫn `InprocServer32` duy nhất trên Windows 11 ARM64 chuyển tiếp export COM `DllGetClassObject` + `DllCanUnloadNow` đúng kiến trúc: ARM64 native → `textvn-tsf-arm64.dll`, ARM64EC/x64 giả lập → `textvn-tsf-x64.dll`.
- **Thay đổi**:
  - `installer/windows/arm64x/{dummy.c, tsf_arm64.def, tsf_x64.def}`: đơn vị biên dịch rỗng 0 mã / 0 CRT cùng 2 bảng `.def` chuyển tiếp `DllGetClassObject` và `DllCanUnloadNow` sang `textvn-tsf-arm64` và `textvn-tsf-x64`.
  - `xtask/src/win_tools.rs`:
    - Hợp nhất parser PE dùng chung `PeView` (G15/G22) cho `pe_imports_bytes`, `pe_is_arm64x`, `pe_forwarded_exports`.
    - `pe_is_arm64x`: đọc COFF `Machine == 0xAA64`, `DataDirectory[10]` (Load Config, size `>= 0xE6`) và bảng DVRT (`version == 1`) xác nhận có `Symbol == 6` (`IMAGE_DYNAMIC_RELOCATION_ARM64X`), phân biệt chính xác ARM64X với ARM64 thường và ARM64EC.
    - `pe_forwarded_exports`: đọc `DataDirectory[0]` xác nhận mọi export đều là pure forwarder (`func_rva` nằm trong bảng export, 0 code export).
    - Mở rộng `check_pe_imports_cli` hỗ trợ `--machine arm64x` (bắt lỗi nếu nạp nhầm DLL ARM64 thường hoặc forwarder chứa code export).
    - Thêm `build_arm64x_cli` (nối lệnh CLI `build-arm64x` vào `xtask/src/main.rs`): tìm VS ARM64 tools qua `vswhere`, chạy `cl /c` + `cl /c /arm64EC` + `link /lib` + `link /dll /noentry /machine:arm64x`, tự kiểm chứng đầu ra bằng `pe_is_arm64x`, `pe_forwarded_exports`, và `pe_imports_bytes`.

## EN

- **Goal (P0-5 §6.2, P0-7 ARM-W2)**: Provide source files, pure-Rust verifier, and MSVC builder for the **ARM64X pure forwarder** `textvn-tsf.dll` so a single `InprocServer32` entry on Windows 11 ARM64 forwards `DllGetClassObject` + `DllCanUnloadNow` to `textvn-tsf-arm64.dll` (ARM64 native) and `textvn-tsf-x64.dll` (ARM64EC/x64 emulated).
- **Changes**:
  - `installer/windows/arm64x/{dummy.c, tsf_arm64.def, tsf_x64.def}`: empty zero-code / zero-CRT translation unit plus ARM64 and x64 forwarder `.def` files.
  - `xtask/src/win_tools.rs`:
    - Unified `PeView` PE/section parser shared across `pe_imports_bytes`, `pe_is_arm64x`, and `pe_forwarded_exports` (G15/G22).
    - `pe_is_arm64x`: inspects COFF `Machine == 0xAA64`, `DataDirectory[10]` Load Config (`>= 0xE6`), and DVRT (`version == 1`) for `Symbol == 6` (`IMAGE_DYNAMIC_RELOCATION_ARM64X`), rejecting plain ARM64 and non-ARM64X images.
    - `pe_forwarded_exports`: validates `DataDirectory[0]` so every export points strictly into the export directory (pure forwarders only, zero code exports).
    - Extended `check_pe_imports_cli` with `--machine arm64x`.
    - Added `build_arm64x_cli` (wired as `build-arm64x` in `xtask/src/main.rs`): locates VS ARM64 build tools via `vswhere`, invokes `cl` (`ARM64` + `ARM64EC`) and `link /machine:arm64x`, and self-verifies the output DLL.

## Kiểm thử / Testing

- `rustfmt --edition 2021 --check xtask/src/win_tools.rs` → PASS.
- `cargo clippy -p xtask --all-targets -- -D warnings` → PASS.
- `cargo test -p xtask` trong worktree sạch → 101/101 PASS, gồm `arm_w2_pe_is_arm64x_phan_biet_arm64x_voi_arm64_thuong_va_x64` (kiểm chứng ARM64X thật pass, ARM64 thường fail `--machine arm64x` và pass `--machine arm64`, ARM64EC `symbol=3` fail, x64 fail, export trỏ tới code fail) và `arm_w2_def_files_va_args_hop_le`.
