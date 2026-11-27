# P0-5 — Kế hoạch hỗ trợ chip ARM: Apple M · Windows on ARM (Snapdragon, NVIDIA) · Linux aarch64

> Trạng thái: **kế hoạch** (2026-11-27, sau 0.2.93). Chưa có mã. Mọi mục ARM-n dưới đây là việc
> cần làm; mục nào xong phải chuyển sang `build-release-report.md` + CHANGELOG như mọi tính năng.
> Nguyên tắc: **một engine Rust (`core/`) cho mọi kiến trúc** — ARM là việc build, đóng gói,
> nạp DLL và kiểm thử, không phải engine thứ hai.

## 1. Hiện trạng (0.2.93)

| Nền tảng | Kiến trúc đang phát hành | Thiếu |
|---|---|---|
| macOS (IMK) | **universal arm64 + x86_64** (`lipo`, CI `swift test --arch arm64` trên runner Apple Silicon) | notarization; e2e gõ thật trên arm64 mới ở mức test gói |
| Windows (TSF) | x64 + x86 (DLL 32-bit cho app WOW64) | **ARM64** (Snapdragon X Elite/Plus, NVIDIA RTX Spark/N1X): không có DLL ARM64 ⇒ app ARM64 native (Edge, Chrome, Notepad, Office ARM64EC…) **không nạp được TIP** |
| Linux (IBus/Fcitx5) | x86_64 tarball + deb + AUR | **aarch64** (NVIDIA Jetson/DGX Spark, Raspberry Pi 5, Snapdragon X Linux, Ampere/Graviton desktop): `debian/control` đã ghi `amd64 arm64` và `scripts/build-linux.sh` dùng `uname -m`, nhưng CI/release chỉ build x86_64; `PKGBUILD arch=('x86_64')` |

## 2. Bộ gõ tiên phong — học gì, tránh gì

| Bộ gõ | ARM | Bài học cho TextVN |
|---|---|---|
| **UniKey** (Windows, nhiều người dùng nhất) | bản **ARM64 native 4.6 RC2** (06/2025) cho Snapdragon, khuyên dùng thay x64 giả lập | người dùng Việt trên Snapdragon đã quen có bản native ⇒ TextVN phải có ARM64 native, không trông vào giả lập x64. UniKey dùng hook toàn cục — TextVN dùng TSF in-process nên phải giải bài **nạp DLL vào cả tiến trình ARM64 lẫn x64 giả lập** (UniKey không gặp) |
| **EVKey** | chưa có (issue #71 "Hỗ trợ native windows ARM" mở, không phản hồi) | khoảng trống thị trường |
| **OpenKey** (macOS) | universal; issue M1: helper chạy Intel (#189), đơ sau sleep/khoá màn hình (#197/#179), xung đột Bitwarden (#193) | lỗi thuộc **event tap** + helper không universal. TextVN đi IMK (không event tap mặc định) và `package-macos-pkg.sh` đã chặn binary không universal — giữ gate đó cho MỌI binary phụ |
| **ibus-bamboo** (Linux) | gói ARM thiếu (#120, bổ sung sau), cross-build hỏng do file theo kiến trúc (#37) | build **native trên runner ARM** (không cross từ x86) + test gói trên container aarch64 thật |
| **fcitx5-unikey / ibus-unikey** | đóng gói bởi distro (Debian/Ubuntu/Arch ARM build lại) | mã C/C++ trung lập kiến trúc ⇒ distro tự build được; TextVN cần nguồn build sạch trên aarch64 (PPA/AUR) |
| **Microsoft Vietnamese Telex** (Windows 11) | native ARM64 | chuẩn so sánh hành vi trên Snapdragon (cùng TSF) |

Lớp lỗi riêng ARM phải chặn trước (không đợi người dùng báo):

1. **`char` không dấu trên Linux aarch64** (x86: có dấu) — so sánh `c < 0`/`c >= 0x80` trên `char` đổi
   nghĩa. Mã C hiện tại đã ép `(unsigned char)` đúng chỗ (soát 2026-11-27: `assist_result.c`,
   `compose.c`, `utf.c`), cần **gate** để không tái phát (ARM-L4).
2. **Kích thước trang 16K/64K** (Asahi, một số kernel Jetson/Grace) — `dict/src/mmap.rs` map offset 0,
   không giả định 4096 ⇒ an toàn; giữ quy tắc.
3. **`off_t` 32-bit trên armhf** — `dict/src/mmap.rs` tự khai `mmap(…, offset: i64)`: đúng trên 64-bit,
   **sai ABI trên ARM 32-bit** ⇒ chặn armhf cho tới khi chuyển sang `mmap64`/kiểu đúng (ARM-L6).
4. **Một CLSID, một đường dẫn DLL** cho cả tiến trình ARM64 lẫn x64 giả lập trên Windows ARM (cùng
   registry 64-bit) — nếu trỏ vào DLL x64 thì app ARM64 native không gõ được, trỏ vào ARM64 thì app
   x64 mất bộ gõ (ARM-W2).
5. **Hiệu năng/độ trễ khác** — runner ARM chậm/nhanh khác x64 ⇒ baseline perf theo từng target, không
   dùng chung số x64 (ARM-C3).

## 3. Kiến trúc mục tiêu

### 3.1 Windows on ARM (Snapdragon X, NVIDIA RTX Spark/N1X)

Quy tắc nạp của Windows trên ARM: tiến trình **ARM64** chỉ nạp DLL ARM64; tiến trình **x64 giả lập /
ARM64EC** (Office trên ARM là ARM64EC) nạp DLL x64/ARM64EC; **ARM64X** nạp được ở cả hai. Tiến trình
**x86 32-bit** dùng registry WOW64 riêng ⇒ DLL x86 hiện có giữ nguyên.

| Thành phần | Cách làm |
|---|---|
| `textvn-tsf.dll` (đường dẫn đăng ký `InprocServer32` 64-bit) | **ARM64X pure forwarder** (khuyến nghị của Microsoft cho COM in-proc dùng chung bởi host x64 và ARM64): DLL rỗng `link /MACHINE:ARM64X` từ 2 `.def` — phía ARM64 chuyển tiếp sang `textvn-tsf-arm64.dll`, phía EC/x64 sang `textvn-tsf-x64.dll` (DLL x64 hiện tại) |
| `textvn-tsf-arm64.dll` | `cargo build --target aarch64-pc-windows-msvc -p textvn-win-tsf` (Rust Tier 2) |
| DLL x86 | giữ nguyên (WOW64) |
| Tray / CLI / settings / hook | build `aarch64-pc-windows-msvc` native (x64 giả lập vẫn chạy nhưng tốn pin) |
| `cli register` | ghi `InprocServer32` = forwarder trên máy ARM64; ACL AppContainer (`S-1-15-2-1/2`) cho **cả 3** DLL (bài R2-26: ô tìm kiếm Start trên Snapdragon là tiến trình ARM64) |
| Inno Setup | `ArchitecturesAllowed=x64compatible arm64`, chọn bộ file theo `IsArm64` |
| MSIX | gói `arm64` (hoặc bundle x64+arm64) — Store yêu cầu khai đúng kiến trúc |
| Portable zip | thêm `TextVN-portable-*-windows-arm64.zip` |

Không chọn: (a) chỉ DLL ARM64EC — app ARM64 native không nạp được; (b) ARM64X "thật" trộn mã Rust
ARM64 + ARM64EC trong một PE — Rust chưa xuất cdylib ARM64X trực tiếp, forwarder đạt cùng kết quả bằng
công cụ MSVC chuẩn và cho phép kiểm từng nửa riêng.

### 3.2 Linux aarch64 (NVIDIA Jetson / DGX Spark, Raspberry Pi 5, Snapdragon X Linux)

| Mục tiêu | Môi trường thật | Cách làm |
|---|---|---|
| Tarball `TextVN-*-linux-aarch64.tar.gz` | GitHub `ubuntu-24.04-arm` / `ubuntu-22.04-arm` (miễn phí cho repo public) | chạy NGUYÊN `scripts/build-linux.sh` + `e2e-linux.sh` native (bài ibus-bamboo #37: không cross-build) |
| Baseline glibc cũ | container aarch64 AlmaLinux 8 / Debian 11 trên runner ARM | như bản compat x86_64 hiện có (`build-linux-el8.sh`, `build-linux-compat.sh`) |
| deb arm64 | `debian/control` đã có `arm64` | job `ci-deb` thêm ma trận arm64; PPA Launchpad tự build arm64 từ source |
| AUR | Arch Linux ARM | `arch=('x86_64' 'aarch64')` + test `makepkg` trong container `archlinuxarm` |
| NVIDIA | Jetson (JetPack 6 = Ubuntu 22.04), DGX Spark (DGX OS = Ubuntu 24.04) | bộ E2E container hiện có (6 distro) chạy lại trên runner ARM; checklist tay trên Jetson/DGX khi có máy |
| Raspberry Pi OS (Debian 12/13 arm64, Wayland labwc) | — | e2e Fcitx5/IBus như Debian 12; armhf 32-bit **không** làm ở giai đoạn này (mục 2 lỗi 3) |

### 3.3 macOS Apple Silicon

Đã universal. Việc còn lại: notarization (đợi Apple secrets), e2e gõ thật IMK trên runner arm64 (hiện
chỉ test gói + unit), và gate "mọi binary trong .pkg/.app đều universal" (đã có cho `TextVN-IM` —
mở rộng cho helper/settings) để không lặp OpenKey #189.

## 4. Bổ sung cho core / framework dùng chung

| ID | Việc | Vì sao |
|---|---|---|
| ARM-C1 | `verify-abi` + `sizes` (20/532, offset) chạy trên **mọi** target build (aarch64-pc-windows-msvc, aarch64-unknown-linux-gnu, aarch64-apple-darwin, i686) trong CI | hợp đồng FFI P0-2 phải giữ nguyên trên mọi kiến trúc |
| ARM-C2 | **Kiểm tất định đa kiến trúc**: replay toàn bộ corpus + Dream RSI corpus trên mỗi target, so hash đầu ra với x86_64 — lệch = đỏ | engine phải cho cùng kết quả; G19c (savings-rate) đo trên mọi kiến trúc |
| ARM-C3 | Baseline perf theo target triple (`perf/baseline-<triple>.json`), cổng `perf` so đúng baseline của máy đang chạy | số x64 không áp được cho ARM |
| ARM-C4 | `doctor`/gói chẩn đoán ghi kiến trúc tiến trình + kiến trúc DLL đã nạp + chế độ giả lập (Windows: `IsWow64Process2`) | báo lỗi trên Snapdragon phải biết app native hay giả lập |
| ARM-C5 | `xtask check-pe-imports` kiểm thêm machine type (ARM64 / ARM64X / x64 / x86) đúng tên tệp; `check-elf-abi` cho aarch64 | chặn đóng gói nhầm kiến trúc |
| ARM-C6 | Trong `core/` không thêm mã theo kiến trúc (`target_arch`, SIMD); nếu cần tối ưu thì có bản thuần + test tương đương | giữ một engine |

## 5. Lộ trình (mỗi giai đoạn đủ DoD 8 mục + Dream RSI, phát hành theo G20)

| Giai đoạn | Việc | DoD / bằng chứng |
|---|---|---|
| **A — Linux aarch64** (rủi ro thấp nhất) | ARM-L1 job `ci-linux` ma trận `ubuntu-24.04-arm`; ARM-L2 release tarball + deb arm64; ARM-L3 E2E 6 container trên ARM; ARM-L4 gate `-fsigned-char`/`-funsigned-char` + `-Wsign-compare` cho C test trên x86 (bắt lỗi `char` không cần máy ARM); ARM-L5 AUR `aarch64`; ARM-L6 sửa `mmap` (kiểu `off_t` đúng) trước khi xét armhf | tarball aarch64 qua `test-linux-package.sh` + e2e IBus/Fcitx5 thật; ARM-C1/C2 xanh trên aarch64 |
| **B — Windows ARM64** | ARM-W1 build DLL/exe `aarch64-pc-windows-msvc`; ARM-W2 forwarder ARM64X + test nạp trong tiến trình ARM64 và x64 giả lập; ARM-W3 `register` + ACL 3 DLL; ARM-W4 Inno Setup/MSIX/portable arm64; ARM-W5 CI `windows-11-arm`: test workspace, replay tsf, e2e gõ thật Notepad (ARM64) + app x64 giả lập + app x86 + `sandbox_e2e` (AppContainer ARM64) | ma trận "app ARM64 / x64 giả lập / ARM64EC (Office) / x86 / AppContainer" đều gõ được; 34 asset → thêm asset arm64 có GPG + Sigstore |
| **C — Apple Silicon hoàn thiện** | ARM-M1 gate universal cho mọi binary; ARM-M2 e2e IMK trên runner arm64; ARM-M3 notarization khi có secrets | `lipo -archs` mọi binary; e2e xanh |
| **D — Tối ưu & kiểm tay** | ARM-C3/C4; kiểm tay trên Snapdragon X, Jetson/DGX Spark, Raspberry Pi 5, MacBook M | báo cáo theo máy trong `build-release-report.md` |

Thứ tự đề xuất: A → B → C → D. A tái dùng gần hết hạ tầng hiện có; B là việc lớn nhất (forwarder +
đóng gói + CI mới) nhưng là khoảng trống so với EVKey và ngang UniKey.

## 6. Quyết định cần chủ repo

1. Có làm **armhf (32-bit)** cho Raspberry Pi OS 32-bit không? (Đề xuất: không — số máy giảm, cần sửa
   `mmap` trước.)
2. Windows ARM64: build **hook tương thích** (`textvn-win-hook`) cho ARM64 hay chỉ TSF? (Đề xuất: chỉ
   TSF như mặc định hiện tại; hook giữ opt-in x64.)
3. MSIX: một gói **bundle** (x64 + arm64) hay hai gói riêng trên Store?
4. Máy thật để kiểm tay: Snapdragon X, Jetson/DGX Spark — có sẵn máy nào?

Nguồn tham chiếu: Microsoft Learn "Build Arm64X binaries" / "Arm64X PE files" (pure forwarder cho COM
in-proc), Rust platform support `aarch64-pc-windows-msvc` · `arm64ec-pc-windows-msvc`, GitHub changelog
2025-08-07 (runner arm64 cho repo public), UniKey ARM64 (unikey.org), EVKey #71, OpenKey #189/#193/#197,
ibus-bamboo #37/#120.
