# TextVN — Bộ gõ tiếng Việt cho Windows, macOS và Linux

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![CI macOS](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-macos.yml/badge.svg)](https://github.com/hunglinhpt/TextVN/actions/workflows/ci-macos.yml)

TextVN là bộ gõ tiếng Việt mã nguồn mở. Một engine Rust dùng chung cho **Windows**
(Text Services Framework), **macOS** (Input Method Kit) và **Linux** (IBus, Fcitx5),
một bảng điều khiển với cùng tuỳ chọn và cùng file cấu hình trên cả ba nền tảng.

- **Gõ như UniKey/OpenKey**: Telex, VNI, VIQR, Telex đơn giản; đặt dấu theo chính tả
  (`hoà`/`hòa`), `uow` → ươ, `z` xoá dấu, gõ dấu cuối từ hay giữa từ đều được, gõ được khi
  bật Caps Lock.
- **Không mất chữ, không lặp chữ, không gạch chân**: cả từ nằm trong vùng soạn (composition/preedit)
  với định dạng sạch không đường kẻ (TF_LS_NONE trên Windows TSF, tắt gạch chân trên Linux/macOS)
  và chốt ở ranh giới từ — không gửi Backspace giả, nên không có lỗi kinh điển của bộ gõ kiểu hook
  trên Chrome, Electron, Excel, thanh địa chỉ.
- **Tránh bị phần mềm diệt virus nhận nhầm**: gói mặc định không hook bàn phím toàn cục,
  không tiêm mã vào process khác, không giả lập phím bằng `SendInput`
  ([lý do](docs/specs/antivirus-false-positive.md)).
- **Tiện ích**: bảng mã TCVN3/VNI Windows/Unicode tổ hợp, gõ tắt, Quick Telex, khôi phục từ
  tiếng Anh, tự viết hoa đầu câu, Ctrl+Shift để chuyển V/E tức thì và cập nhật biểu tượng trạng thái.

## Tải và cài

Bản mới nhất: **[GitHub Release v0.2.6](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.6)**
(0.2.6 — chốt đăng ký TSF: VI + EN layouts (thiết kế), tự sửa ghost
registration khi chạy lại từ thư mục portable còn tồn tại; giữ các fix
0.2.4/0.2.5: không cần admin, không gạch chân, Ctrl+Shift ổn định; build bởi
CI, kèm `SHA256SUMS.txt`; chưa ký số).

| Nền tảng | Cài đặt | Giải nén dùng ngay |
|---|---|---|
| Windows 10/11 x64 | `TextVN-setup-<ver>-windows-x64.exe` | `TextVN-portable-<ver>-windows-x64-*.zip` → chạy `TextVN.exe` |
| macOS 13+ (Apple Silicon / Intel) | `TextVN-mac-v<ver>.pkg` → `installer -pkg … -target CurrentUserHomeDirectory` (không sudo); zip `TextVN-macos-<universal\|arm64\|x86_64>-v<ver>.zip` → kéo `TextVN-IM.app` vào `~/Library/Input Methods/` | xem [Hướng dẫn sử dụng macOS](docs/30-macos/P2-4-ui-packaging-release.md) |
| Linux (IBus / Fcitx5) | `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → `./install.sh` (per-user, không cần root) | cùng tarball → `./textvn-portable.sh` |

Chi tiết cài, gỡ, sử dụng và xử lý sự cố: **[Hướng dẫn sử dụng](docs/user-guide.md)**.

Lưu ý Windows: v0.2.4 hỗ trợ đăng ký TSF ở cả cấp độ User (`HKCU`, không cần quyền Admin)
lẫn Machine (`HKLM` qua installer). Bản portable chỉ cần giải nén và chạy `TextVN.exe` là có thể
đăng ký và sử dụng ngay lập tức mà không yêu cầu UAC elevation.

## Trạng thái

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | ✅ Release candidate — gõ thật qua TSF được CI kiểm tra trên Windows. |
| macOS 13+ Apple Silicon / Intel | 🔧 Beta — IMK adapter + CGEventTap opt-in; corpus headless 114 case; CI build + test Swift (arm64 + x86_64) và đóng gói `.pkg`. GUI typing, cài/gỡ `.pkg` và notarization còn cần xác thực trên máy Mac thật; không coi là production. |
| Linux IBus / Fcitx5 | ✅ Release candidate — CI kiểm tra với ibus-daemon và fcitx5 thật. |

Kết quả kiểm thử của phiên bản hiện tại: [build-release-report.md](docs/release/build-release-report.md) ·
thay đổi: [CHANGELOG.md](CHANGELOG.md).

## Dựng từ mã nguồn

```powershell
# Windows: Rust stable + VS 2022 Build Tools (C++)
powershell -File .\build-release.ps1 -BuildInstaller   # cần Inno Setup 6.5+
```

```bash
# macOS 13+: Xcode CLT 15+, rustup với targets aarch64-apple-darwin + x86_64-apple-darwin
cd adapters/macos-imk
./build-rust.sh                   # build engine + swift build + assemble TextVN-IM.app
./build-rust.sh --lib-only        # chỉ build libtextvn_ffi.a (cho swift test)
./build-rust.sh --release         # build bản release
```

```bash
# macOS — đóng gói phát hành (từ thư mục gốc repo; script tự kiểm tool khi chạy)
scripts/build-macos.sh                            # universal nếu đủ 2 arch; nếu không, tên zip ghi arch thực tế
scripts/package-macos-pkg.sh --rebuild            # .pkg per-user (pkgbuild + productbuild, không sudo)
scripts/package-macos-pkg.sh --rebuild --notarize # thêm productsign (Developer ID Installer, env
                                                  #   DEVELOPER_ID_INSTALLER) + notarytool + staple
scripts/notarize-macos.sh dist/macos/TextVN-mac-v<ver>.pkg   # notarize riêng lẻ (tuỳ chọn)
```

```bash
# Linux (Ubuntu/Debian)
scripts/install_linux.sh        # dựng + cài per-user
scripts/build-linux.sh          # tạo tarball phát hành
```

Kiến trúc, kiểm thử, quy ước và quy trình phát hành: **[Hướng dẫn phát triển](docs/developer-guide.md)**.

## Cấu trúc

```
core/              engine (Telex/VNI/VIQR, đặt dấu, bảng mã, gõ tắt) — không phụ thuộc OS
config/  ffi/      cấu hình config.v1 · C ABI cho adapter
adapters/          macos-imk · macos-tap · macos-app
                   windows-tsf · windows-hook (gói Compatibility)
                   linux-ibus · linux-fcitx5 · linux-common · linux-settings (GTK4)
tray/  cli/        khay + bảng điều khiển Windows · textvn-cli (register, doctor, replay)
corpus/            kịch bản gõ chạy qua mô phỏng của mọi adapter (mac: 114, win: 78, shared: 35)
installer/  packaging/  scripts/    đóng gói và kiểm thử gói
docs/              đặc tả (10-shared, 20-windows, 30-macos, 40-linux), specs/, release/
```

## Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) và [SECURITY.md](SECURITY.md). Báo lỗi gõ: ghi
chính xác chuỗi phím, kết quả nhận được, ứng dụng và hệ điều hành.

## Giấy phép

[GPL-3.0-or-later](LICENSE) © 2026 hunglinhpt
