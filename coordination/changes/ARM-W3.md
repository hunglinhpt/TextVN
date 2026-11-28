<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-W3 — Đăng ký TSF trên Windows ARM64 (IsWow64Process2, forwarder & ACL 4 DLL)

## Tiếng Việt

### Tóm tắt thay đổi
- Thêm nhận diện máy chủ Windows ARM64 trong [`cli/src/register.rs`](../../cli/src/register.rs):
  - Hằng `IMAGE_FILE_MACHINE_ARM64 = 0xAA64`, hàm thuần `is_arm64_native_machine(native_machine)`, `host_is_arm64()`, `select_inproc_dll_in_dir(dir, is_arm64_host)`, và `companion_dlls_for_acl(inproc_dll, is_arm64_host)`.
  - Trong `win_impl::is_arm64_host()`, nạp động `IsWow64Process2` từ `kernel32.dll` (`LoadLibraryExW(..., LOAD_LIBRARY_SEARCH_SYSTEM32)` + `GetProcAddress` + `FreeLibrary`, pseudo-handle tiến trình hiện tại) để giữ tương thích **E65** với Windows 10 1507/1607, fallback về `cfg!(target_arch = "aarch64")`.
  - Khi máy chủ là ARM64, `resolve_dll_path` → `select_inproc_dll_in_dir` ưu tiên đăng ký COM `InprocServer32` trỏ tới ARM64X pure forwarder `textvn-tsf.dll` (P0-5 §6.2), fallback `textvn-tsf-arm64.dll`.
  - `do_register` (cấp quyền AppContainer `S-1-15-2-1` / `S-1-15-2-2`) và `machine_scope_dll` (chuẩn hoá đường dẫn an toàn scope máy) lặp qua `companion_dlls_for_acl` để bao phủ đủ tối đa 4 DLL hiện diện cạnh nhau trên ARM64 (`textvn-tsf.dll`, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) không trùng lặp.
  - `do_status` in thêm `Host arch: ARM64 (IsWow64Process2)` và các `Companion DLL` hiện có.
  - Kiểm tra `do_unregister`: xoá key COM CLSID và CTF TIP theo GUID nên gỡ sạch bất kể tên DLL được trỏ trong `InprocServer32`.
- Thêm 3 unit test trong `cli/src/register.rs::tests`: `arm_w3_is_arm64_native_machine_chinh_xac`, `arm_w3_select_inproc_dll_uu_tien_forwarder_tren_arm64`, `arm_w3_companion_dlls_for_acl_du_4_dll_tren_arm64`.

### Kiểm chứng
- `rustfmt --edition 2021 --check cli/src/register.rs`: PASS.
- `cargo clippy -p textvn-cli --all-targets -- -D warnings`: PASS.
- `cargo test -p textvn-cli`: 70/70 PASS (bao gồm 3 test mới `arm_w3_*` và kiểm tra mutation kill bằng cách đảo ưu tiên `textvn-tsf-arm64.dll` lên trước `textvn-tsf.dll`).
- Chạy e2e trên `windows-11-arm` do CI workflow kiểm chứng.

---

## English

### Summary of Changes
- Added Windows ARM64 host detection and multi-DLL TSF registration support in [`cli/src/register.rs`](../../cli/src/register.rs):
  - Added constant `IMAGE_FILE_MACHINE_ARM64 = 0xAA64` and pure cross-platform helpers `is_arm64_native_machine(native_machine)`, `host_is_arm64()`, `select_inproc_dll_in_dir(dir, is_arm64_host)`, and `companion_dlls_for_acl(inproc_dll, is_arm64_host)`.
  - In `win_impl::is_arm64_host()`, dynamically resolves `IsWow64Process2` from `kernel32.dll` via `LoadLibraryExW(..., LOAD_LIBRARY_SEARCH_SYSTEM32)` + `GetProcAddress` + `FreeLibrary` (using the current-process pseudo-handle) to preserve **E65** compatibility on Windows 10 1507/1607, falling back to `cfg!(target_arch = "aarch64")`.
  - On ARM64 hosts, `resolve_dll_path` → `select_inproc_dll_in_dir` prioritizes the ARM64X pure forwarder `textvn-tsf.dll` (P0-5 §6.2) for COM `InprocServer32`, falling back to `textvn-tsf-arm64.dll`.
  - `do_register` (granting AppContainer `S-1-15-2-1` / `S-1-15-2-2` read ACLs) and `machine_scope_dll` (machine-scope directory validation) iterate over `companion_dlls_for_acl` to cover all present sibling DLLs on ARM64 (`textvn-tsf.dll`, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) without duplicates.
  - `do_status` additionally reports `Host arch: ARM64 (IsWow64Process2)` and present `Companion DLL` files.
  - Verified `do_unregister`: removes CLSID and CTF TIP registry trees by GUID regardless of which DLL filename `InprocServer32` points to.
- Added 3 unit tests in `cli/src/register.rs::tests`: `arm_w3_is_arm64_native_machine_chinh_xac`, `arm_w3_select_inproc_dll_uu_tien_forwarder_tren_arm64`, `arm_w3_companion_dlls_for_acl_du_4_dll_tren_arm64`.

### Verification
- `rustfmt --edition 2021 --check cli/src/register.rs`: PASS.
- `cargo clippy -p textvn-cli --all-targets -- -D warnings`: PASS.
- `cargo test -p textvn-cli`: 70/70 PASS (including 3 new `arm_w3_*` tests and mutation-kill verification by swapping `textvn-tsf-arm64.dll` ahead of `textvn-tsf.dll`).
- Real `windows-11-arm` end-to-end verification is covered by CI.
