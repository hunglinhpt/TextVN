<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ARM-W4 — Bộ cài Inno Setup + portable ZIP Windows ARM64 (ADR-015)

## Tiếng Việt

### Tóm tắt thay đổi
- Chốt quyết định đóng gói trong [`docs/adr/ADR-015-windows-arm64-packaging.md`](../../docs/adr/ADR-015-windows-arm64-packaging.md) và cập nhật [`docs/adr/README.md`](../../docs/adr/README.md): dùng một tệp `.iss` duy nhất với `/DTargetArch=x64|arm64|multi`, mặc định phát hành gói riêng theo kiến trúc `-windows-x64` và `-windows-arm64` (giữ chế độ `multi` sẵn sàng).
- Cập nhật [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss):
  - Hỗ trợ `#ifndef TargetArch` (`x64`/`arm64`/`multi`), thiết lập `OutputBaseFilename`, `ArchitecturesAllowed`, `ArchitecturesInstallIn64BitMode`, `TargetDir` và `TargetDirArm64`.
  - Bổ sung payload 4 DLL trên ARM64 (`textvn-tsf.dll` ARM64X forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) cùng dọn `.old-*` trong `[UninstallDelete]` và `RenameLockedTsfDll()`.
- Cập nhật [`build-release.ps1`](../../build-release.ps1):
  - Thêm `-Arch x64|arm64` (mặc định `x64`, cấm kết hợp `arm64` + `-IncludeCompatibilityHook`), dựng `aarch64-pc-windows-msvc` + `x86_64-pc-windows-msvc` (`textvn-tsf-x64.dll`) + `i686-pc-windows-msvc` (`textvn-tsf-x86.dll`) + `cargo xtask build-arm64x --out textvn-tsf.dll`.
  - Kiểm `check-pe-imports` đúng `--machine` cho từng trong 6 PE, đóng gói `TextVN-portable-<v>-windows-arm64-*.zip`, dựng bộ cài `/DTargetArch=arm64`, giữ G24 dọn `target/debug/build`.
- Cập nhật [`installer/windows/portable/uninstall.ps1`](../../installer/windows/portable/uninstall.ps1), [`installer/windows/tests/test-portable.ps1`](../../installer/windows/tests/test-portable.ps1), [`installer/windows/tests/test-installer.ps1`](../../installer/windows/tests/test-installer.ps1) để gỡ sạch và kiểm đủ cả 4 DLL trên ARM64.
- Cập nhật job `windows-arm64` trong [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml) chạy `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller` cùng `test-portable.ps1` và `test-installer.ps1` trên `windows-11-arm`.

### Kiểm chứng
- PowerShell AST parser parse sạch cả 4 tệp `.ps1`; `cargo xtask ascii-ps1` PASS.
- `cargo xtask iss-tabs`, `cargo xtask check-version-sync`, `cargo xtask no-python`, `cargo xtask hygiene-docs`, `cargo xtask hygiene-apis`, `cargo xtask hygiene-secrets`, `cargo xtask regression-index`, `cargo xtask eval-leak` PASS.
- `ISCC.exe` (Inno Setup 6.7.3) biên dịch thử sạch cả 3 chế độ `/DTargetArch=x64`, `/DTargetArch=arm64` và `/DTargetArch=multi` (kèm `[Code]` `IsArm64Arch()`).
- Đóng gói & cài/gỡ e2e trên `windows-11-arm` chạy trong CI job `windows-arm64`.

---

## English

### Summary of Changes
- Recorded the packaging architecture decision in [`docs/adr/ADR-015-windows-arm64-packaging.md`](../../docs/adr/ADR-015-windows-arm64-packaging.md) and indexed it in [`docs/adr/README.md`](../../docs/adr/README.md): one shared `.iss` parameterized by `/DTargetArch=x64|arm64|multi`, shipping separate `-windows-x64` and `-windows-arm64` artifacts by default with `multi` mode supported.
- Updated [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss):
  - Added `/DTargetArch` (`x64`/`arm64`/`multi`) controlling `OutputBaseFilename`, `ArchitecturesAllowed`, `ArchitecturesInstallIn64BitMode`, `TargetDir`, and `TargetDirArm64`.
  - Added the 4-DLL ARM64 TSF payload (`textvn-tsf.dll` ARM64X forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) plus `.old-*` cleanup in `[UninstallDelete]` and `RenameLockedTsfDll()`.
- Updated [`build-release.ps1`](../../build-release.ps1):
  - Added `-Arch x64|arm64` (default `x64`, rejects legacy hook on ARM64), building `aarch64-pc-windows-msvc` + `x86_64-pc-windows-msvc` (`textvn-tsf-x64.dll`) + `i686-pc-windows-msvc` (`textvn-tsf-x86.dll`) + `cargo xtask build-arm64x --out textvn-tsf.dll`.
  - Runs `check-pe-imports` with per-binary `--machine` flags across all 6 PEs, packages `TextVN-portable-<v>-windows-arm64-*.zip`, builds `/DTargetArch=arm64` installers, and preserves G24 `target/debug/build` cleanup.
- Updated [`installer/windows/portable/uninstall.ps1`](../../installer/windows/portable/uninstall.ps1), [`installer/windows/tests/test-portable.ps1`](../../installer/windows/tests/test-portable.ps1), and [`installer/windows/tests/test-installer.ps1`](../../installer/windows/tests/test-installer.ps1) to assert and clean all 4 ARM64 TSF DLLs.
- Updated the `windows-arm64` job in [`.github/workflows/ci-windows.yml`](../../.github/workflows/ci-windows.yml) to run `build-release.ps1 -Arch arm64 -SkipTests -BuildInstaller`, `test-portable.ps1`, and `test-installer.ps1` on `windows-11-arm`.

### Verification
- PowerShell AST parser checks passed on all 4 modified `.ps1` files; `cargo xtask ascii-ps1` PASSED.
- `cargo xtask iss-tabs`, `cargo xtask check-version-sync`, `cargo xtask no-python`, `cargo xtask hygiene-docs`, `cargo xtask hygiene-apis`, `cargo xtask hygiene-secrets`, `cargo xtask regression-index`, and `cargo xtask eval-leak` PASSED.
- `ISCC.exe` (Inno Setup 6.7.3) compiled all 3 modes (`/DTargetArch=x64`, `/DTargetArch=arm64`, `/DTargetArch=multi` including `[Code]` `IsArm64Arch()`) cleanly.
- Real Windows ARM64 packaging and install/uninstall E2E run in the `windows-arm64` CI job (`windows-11-arm`).
