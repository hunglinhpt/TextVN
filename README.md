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

Bản mới nhất: **[GitHub Release v0.2.29](https://github.com/hunglinhpt/TextVN/releases/tag/v0.2.29)**
(release candidate do CI build và kiểm thử). **Mọi file phát hành đều có chữ ký GPG
(`.asc`) và Sigstore (`.cosign.sig` + `.cosign.cert`)**, `SHA256SUMS.txt` được clearsign
— cách kiểm: [signing.md](docs/release/signing.md). Chưa có: chữ ký Authenticode cho
Windows (đang chờ SignPath Foundation duyệt — SmartScreen có thể cảnh báo) và Apple
Developer ID/notarization cho macOS (Gatekeeper cảnh báo lần mở đầu). Thay đổi của từng
bản (0.2.28 đổi một số cách gõ — Backspace, viết hoa đầu câu, Telex `w`): [CHANGELOG.md](CHANGELOG.md).

| Nền tảng | Cài đặt | Giải nén dùng ngay |
|---|---|---|
| Windows 10/11 x64 | `TextVN-setup-<ver>-windows-x64.exe` (cho tài khoản của bạn, không cần admin) · `TextVN-setup-<ver>-windows-x64-machine.exe` (cho mọi người dùng, Program Files, UAC một lần) | `TextVN-portable-<ver>-windows-x64-*.zip` → chạy `TextVN.exe` |
| Windows (Microsoft Store) | Đang nộp gói MSIX ([msix-submission.md](docs/release/msix-submission.md)). File `TextVN-<ver>-windows-x64.msix` trên Release chưa ký, chỉ dùng để nộp Store — **không cài trực tiếp được** | — |
| macOS 13+ (Apple Silicon / Intel) | `TextVN-mac-v<ver>.pkg` → mở để cài, hoặc `installer -pkg … -target CurrentUserHomeDirectory` (không sudo) | `TextVN-macos-universal-v<ver>.zip` (hoặc `.tar.gz`) chứa `TextVN-IM.app` + `TextVN.app` — xem [Hướng dẫn sử dụng, mục macOS](docs/user-guide.md) |
| Linux (IBus / Fcitx5) | `tar xzf TextVN-<ver>-linux-x86_64.tar.gz` → `./install.sh` (per-user, không cần root; `sudo ./install.sh --system` cho mọi người dùng) | cùng tarball → `./textvn-portable.sh` |

Chi tiết cài, gỡ, sử dụng và xử lý sự cố: **[Hướng dẫn sử dụng](docs/user-guide.md)**.

Lưu ý Windows: `TextVN-setup-<ver>-windows-x64.exe` cài **cho tài khoản hiện tại**
(`%LOCALAPPDATA%\Programs\TextVN`, không UAC); `TextVN-setup-<ver>-windows-x64-machine.exe`
cài **cho mọi người dùng** (Program Files, UAC một lần). Mỗi tài khoản Windows tự được
đăng ký bộ gõ ở lần đầu TextVN chạy; nâng cấp **giữ nguyên** kiểu gõ, tuỳ chọn và thiết
lập theo từng ứng dụng. Trên **Windows 11 24H2+** một số máy từ chối kích hoạt bộ gõ chỉ
đăng ký cho tài khoản (lỗi B7 trong `docs/specs/win-test-common-errors.md`): vì an toàn,
bản cài cho tài khoản, bản portable và bản Store **không bao giờ xin UAC** — TextVN báo
một lần và hướng dẫn dùng bộ cài `-machine.exe` (chỉ bản nằm trong Program Files mới
được đăng ký phạm vi máy).

## Điểm vượt trội so với các bộ gõ khác

Những gì TextVN làm mà UniKey / EVKey / OpenKey hiện **không có** (hoặc làm
khác — so sánh theo tính năng công khai tại thời điểm 2026-10):

| Tính năng | TextVN | Bộ gõ truyền thống |
|---|---|---|
| **Tự xác định EN/VI khi gõ chung đoạn văn** | Từ điển EN thông dụng + lưới bảo vệ âm tiết Việt thông dụng: gõ xen kẽ 2 ngôn ngữ không cần đổi mode, từ nào là tiếng Anh được trả lại nguyên vẹn khi gõ dấu cách | Telex áp lên mọi từ — `text` thành `tẽt`, `is` thành `í`; muốn gõ tiếng Anh phải bấm tắt mode từng từ hoặc tự gỡ dấu |
| **Ưu tiên tiếng Việt theo mode** (không phá tiếng Việt) | Cặp mơ hồ `cow`/`cơ`, `sex`/`sẽ`, `queen`/`quên` được giải theo mode đang bật — đúng chuẩn UniKey cho tiếng Việt, đồng thời trả tự do cho tiếng Anh | Phải chọn: hoặc phá tiếng Anh hoặc tắt transform |
| **Tab gợi ý hoàn tất từ tiếng Anh** | Gõ dở `tes` → Tab thành `test` theo từ điển dựng sẵn + từ điển bổ sung cá nhân | Không có |
| **Từ điển EN cá nhân** | Thêm/xoá từ bằng UI (Từ điển EN…); quyết định của người dùng thắng mọi phỏng đoán engine | Sửa file config thủ công (nếu có) |
| **Escape khôi phục nguyên phím** | Gõ sai ở mọi thời điểm → Escape trả lại đúng chuỗi đã gõ | Nhiều bộ gõ chỉ có undo ở mức câu |
| **Ctrl+Shift đổi mode ở mọi ứng dụng** | Bộ dò tap chỉ quan sát (không ăn phím) + debounce chống lật đôi khi cả tray lẫn engine cùng nhận | Hook toàn cục có thể gây lật đôi / xung đột AV |
| **Không gạch chân preedit, không cài admin** | TSF per-user HKCU, portable giải nén chạy ngay, không UAC | UniKey/EVKey cần cài đặt + một số phiên bản yêu cầu admin |
| **Tự sửa ghost registration** | Đăng ký trỏ vào thư mục đã xoá → chạy lại `TextVN.exe` từ thư mục mới là tự sửa | Phải gỡ/cài lại thủ công |
| **Dialog DPI-aware đa màn hình** | Kéo giữa màn khác DPI tự co giãn + re-layout + font theo DPI | Thường render sai/khểnh khi đa màn hình |
| **Cài im lặng chuẩn Store** | `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` — CI kiểm cài/gỡ im lặng, ARP và giả lập validator của Store ở mỗi PR và mỗi release | Không kiểm chứng luồng Store |
| **Riêng tư mặc định** | Xử lý phím 100% cục bộ, không telemetry, không mạng; tự tắt ở ô mật khẩu; chính sách riêng tư công khai trong repo | Thường không công bố chính sách riêng tư |
| **Số lượng tiến trình** | 1 tiến trình tray + TIP in-process; global keyboard hook chỉ quan sát modifier (không ăn phím, không inject) | UniKey dùng hook toàn cục ăn phím |

Engine bằng Rust, test bằng corpus replay (hơn 230 kịch bản `*.keys`) qua 4 mô phỏng
adapter (headless, tsf, mac, linux) trên 3 hệ điều hành ở mỗi commit; quy tắc xung đột
ngôn ngữ công khai tại
[docs/specs/language-detection.md](docs/specs/language-detection.md).

## Trạng thái

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | ✅ Release candidate — gõ thật qua TSF được CI kiểm tra trên Windows (bộ cài, portable, MSIX cài thử). Còn thiếu duy nhất chữ ký Authenticode (chờ SignPath Foundation duyệt). Microsoft Store: đang nộp gói MSIX. |
| macOS 13+ Apple Silicon / Intel | ✅ Release candidate — bộ gõ IMK + app menu bar + `.pkg` per-user (không sudo); CI build 2 kiến trúc (arm64 + x86_64), chạy `swift test` trên arm64. Chưa ký Developer ID/notarize (bản phát hành ký ad-hoc, `.pkg` chưa ký) → Gatekeeper cảnh báo lần mở đầu ([signing-status-mac.md](docs/release/signing-status-mac.md)). Module CGEventTap (`adapters/macos-tap`) có mã + test nhưng **không** có trong bản phát hành. |
| Linux IBus / Fcitx5 | ✅ Release candidate — tarball `install.sh` (per-user hoặc `--system`) + chạy ngay, Settings GTK4; CI kiểm tra với ibus-daemon và fcitx5 thật. Chưa có `.deb`/`.rpm`/AUR. |

Kết quả kiểm thử của phiên bản hiện tại: [build-release-report.md](docs/release/build-release-report.md) ·
thay đổi: [CHANGELOG.md](CHANGELOG.md).

## Dựng từ mã nguồn

```powershell
# Windows: Rust stable + VS 2022 Build Tools (C++)
powershell -NoProfile -ExecutionPolicy Bypass -File .\build-release.ps1 -BuildInstaller -MachineInstaller   # cần Inno Setup 6.5+
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
scripts/package-macos-pkg.sh --rebuild --notarize # + notarytool + staple; ký Developer ID chỉ khi có
                                                  #   secret Apple — docs/release/signing-status-mac.md
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
corpus/            kịch bản gõ *.keys (shared · win · mac) chạy qua mô phỏng của mọi adapter
installer/  packaging/  scripts/    đóng gói và kiểm thử gói
docs/              đặc tả (10-shared, 20-windows, 30-macos, 40-linux), specs/, release/
```

## Đóng góp

Xem [CONTRIBUTING.md](CONTRIBUTING.md) và [SECURITY.md](SECURITY.md). Báo lỗi gõ: ghi
chính xác chuỗi phím, kết quả nhận được, ứng dụng và hệ điều hành.

## Giấy phép

[GPL-3.0-or-later](LICENSE) © 2026 LinhBH.CoM
