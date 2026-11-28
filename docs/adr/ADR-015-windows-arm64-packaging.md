<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# ADR-015 — Đóng gói Windows ARM64: một `.iss` cho `x64`, `arm64` và `multi`

- **Trạng thái / Status**: ✅ Accepted (2026-11-28)
- **Ngữ cảnh / Context**: P0-5 §6.2, P0-6 §7.4 & §11, P0-7 (`ARM-W2`, `ARM-W3`, `ARM-W4`).

---

## Tiếng Việt

### Bối cảnh
Trên Windows 11 ARM64, ứng dụng có thể chạy ở 3 ABI khác nhau trong cùng phiên desktop: tiến trình ARM64 native, tiến trình x64 giả lập và tiến trình x86 (WOW64 32-bit). Theo `ARM-W2` và `ARM-W3`, bộ cài TextVN trên ARM64 phải cài đủ 4 DLL TSF (`textvn-tsf.dll` ARM64X pure forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) cùng `TextVN.exe` và `textvn-cli.exe` (`aarch64-pc-windows-msvc`, `+crt-static`), trong khi máy x64 truyền thống chỉ cần 2 DLL (`textvn-tsf.dll` + `textvn-tsf-x86.dll`).

Câu hỏi kiến trúc (P0-6 §11, `ARM-W4`): phát hành một bộ cài Inno Setup đa kiến trúc hay tách gói theo kiến trúc?

### Quyết định
1. **Một nguồn Inno Setup duy nhất (G22)**: dùng chung [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss) qua cờ biên dịch `/DTargetArch=x64|arm64|multi` (mặc định `x64`), tránh nhân đôi logic cài/nâng cấp/gỡ cài đặt.
2. **Phát hành mặc định tách gói theo kiến trúc**:
   - Installer: `TextVN-setup-<v>-windows-x64.exe` (`ArchitecturesAllowed=x64compatible`) và `TextVN-setup-<v>-windows-arm64.exe` (`ArchitecturesAllowed=arm64`), cộng biến thể `-machine.exe`.
   - Portable ZIP: `TextVN-portable-<v>-windows-x64-<build>.zip` và `TextVN-portable-<v>-windows-arm64-<build>.zip` (built via `build-release.ps1 -Arch x64|arm64`).
3. **Chế độ gộp sẵn sàng (`multi`)**: cùng tệp `.iss` hỗ trợ `/DTargetArch=multi` (`ArchitecturesAllowed=x64compatible arm64`, `ArchitecturesInstallIn64BitMode=x64compatible arm64`, chọn tệp theo `IsArm64Arch()`) khi cần tạo một bộ cài đơn lẻ không đổi logic `[Code]`.

### Lý do chọn tách gói mặc định từ một nguồn `.iss`
- Kích thước gói cài nhỏ gọn hơn cho người dùng cuối (máy x64 không phải tải DLL ARM64 và nhị phân trùng lặp).
- Khớp trực tiếp quy trình nộp Win32 của Microsoft Store Partner Center (cho phép khai báo URL cài đặt riêng cho từng kiến trúc `x64` và `arm64`, tránh gói gộp vượt chuẩn dung lượng hoặc nhầm nhận diện).
- Phù hợp kiến trúc CI hai runner gốc độc lập (`windows-2022` và `windows-11-arm`) mà không cần bước ghép chéo hai chiều trước khi kiểm thử installer/portable.

### Điều kiện xem xét lại (Revisit trigger)
- Khi Microsoft Store hoặc người dùng cuối yêu cầu duy nhất một URL tải tự nhận diện cho cả x64 và ARM64, có thể bật `/DTargetArch=multi` trong `release.yml` mà không phải viết lại `.iss`.

---

## English

### Context
On Windows 11 ARM64, desktop applications execute across three ABIs within the same session: native ARM64, emulated x64, and WOW64 x86. Per `ARM-W2` and `ARM-W3`, an ARM64 TextVN installation requires all four TSF DLLs (`textvn-tsf.dll` ARM64X pure forwarder, `textvn-tsf-arm64.dll`, `textvn-tsf-x64.dll`, `textvn-tsf-x86.dll`) plus native `aarch64-pc-windows-msvc` (`+crt-static`) `TextVN.exe` and `textvn-cli.exe`, whereas x64 machines only need two DLLs (`textvn-tsf.dll` + `textvn-tsf-x86.dll`).

Architectural question (P0-6 §11, `ARM-W4`): ship a single multi-architecture Inno Setup installer or per-architecture packages?

### Decision
1. **Single `.iss` source of truth (G22)**: parameterize [`installer/windows/TextVN-setup.iss`](../../installer/windows/TextVN-setup.iss) with `/DTargetArch=x64|arm64|multi` (default `x64`) so install/upgrade/uninstall logic is never duplicated.
2. **Ship per-architecture packages by default**:
   - Installers: `TextVN-setup-<v>-windows-x64.exe` (`ArchitecturesAllowed=x64compatible`) and `TextVN-setup-<v>-windows-arm64.exe` (`ArchitecturesAllowed=arm64`), plus `-machine.exe` variants.
   - Portable ZIPs: `TextVN-portable-<v>-windows-x64-<build>.zip` and `TextVN-portable-<v>-windows-arm64-<build>.zip` (built via `build-release.ps1 -Arch x64|arm64`).
3. **Multi-arch mode retained (`multi`)**: the same `.iss` supports `/DTargetArch=multi` (`ArchitecturesAllowed=x64compatible arm64`, `ArchitecturesInstallIn64BitMode=x64compatible arm64`, per-arch `[Files]` guarded by `IsArm64Arch()`).

### Rationale
- Keeps downloads smaller for end users (x64 users do not carry ARM64 binaries and extra TSF DLLs).
- Matches Microsoft Store Partner Center Win32 submission URLs cleanly per architecture (`x64` vs `arm64`).
- Allows `windows-2022` and `windows-11-arm` CI runners to build and end-to-end verify their native portable ZIP and Inno installer independently without cross-runner staging.

### Revisit Trigger
- If a single combined download URL is preferred later for direct web distribution, switch `release.yml` to invoke `/DTargetArch=multi` on the existing `.iss`.
