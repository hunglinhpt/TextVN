# Hướng dẫn sử dụng TextVN

TextVN là bộ gõ tiếng Việt mã nguồn mở cho **Windows** (Text Services Framework),
**macOS** (Input Method Kit) và **Linux** (IBus, Fcitx5). Ba nền tảng dùng chung một engine,
một bảng điều khiển với cùng tuỳ chọn và cùng file cấu hình — học một lần, dùng ở đâu cũng
giống nhau.

---

## 1. Cài đặt

Mỗi bản phát hành có hai cách dùng — chọn một.

### Windows

| Cách | Làm gì | Khi nào nên dùng |
|---|---|---|
| **Cài đặt** | Chạy `TextVN-setup-<phiên bản>-windows-x64.exe` → Tiếp → Cài. Bản đã phát hành v0.2.3 cài per-user; bản vá kế tiếp cần UAC một lần để đăng ký TSF toàn máy. | Máy của bạn, dùng lâu dài: tự khởi động cùng Windows, có trong Settings → Apps để gỡ. |
| **Giải nén dùng ngay** | Giải nén `TextVN-portable-<phiên bản>-windows-x64-*.zip` → nhấn đúp `TextVN.exe`. | Máy mượn, USB, dùng thử. Không ghi gì vào Program Files. |

Lần chạy đầu tiên TextVN tự đăng ký bộ gõ với Windows cho tài khoản của bạn. Nếu vẫn
chưa gõ được: mở Bảng điều khiển → **[Cài & bật TSF]**, rồi chọn *TextVN* trong danh sách
bộ gõ (**Win + Space**).

Nếu `register.log` ghi `Đăng ký qua API TSF → 0x80004005` trong khi
`COM server HKCU → OK`, lỗi không nằm ở quyền ghi HKCU. Bản portable không thể
tự nâng quyền an toàn từ thư mục giải nén; chờ installer đã qua kiểm thử máy thật
hoặc liên hệ quản trị viên. Không chạy `textvn-cli.exe` portable bằng quyền admin.

Gỡ: *Settings → Apps → TextVN → Uninstall* (bản cài) hoặc chuột phải `uninstall.ps1` →
*Run with PowerShell* rồi xoá thư mục (bản portable). Menu khay **Gỡ cài đặt** làm đúng việc
tương ứng cho cả hai loại.

### macOS (13 Ventura trở lên — Apple Silicon và Intel)

**Yêu cầu**: macOS 13+. Không cần cài thêm gì — `TextVN-IM.app` là một bundled executable
hoàn chỉnh (engine Rust universal đã nhúng trong app).

**Cài đặt:**

```bash
# 1. Tải và giải nén TextVN-<ver>-macos-universal.zip
# 2. Kéo TextVN-IM.app vào ~/Library/Input Methods/
cp -R TextVN-IM.app ~/Library/Input\ Methods/

# 3. Bật trong System Settings → Keyboard → Input Sources → Add (+)
#    Tìm "TextVN" trong danh sách → thêm vào
```

**Chuyển bộ gõ**: `Control + Space` (macOS default) hoặc menu Input Source trên menu bar.

**Gỡ cài đặt:**

```bash
# Tắt bộ gõ trong System Settings → Keyboard → Input Sources → xoá TextVN
# Sau đó:
rm -rf ~/Library/Input\ Methods/TextVN-IM.app
```

> **Lưu ý beta**: Bản phát hành chưa được ký với Apple Developer ID — macOS Gatekeeper có thể
> cảnh báo lần đầu mở. Để bypass: click phải → Open → Open anyway.

### Linux

Tải `TextVN-<phiên bản>-linux-x86_64.tar.gz`, giải nén:

```bash
tar xzf TextVN-*-linux-x86_64.tar.gz
cd TextVN-*-linux-x86_64
```

| Cách | Lệnh | Ghi chú |
|---|---|---|
| **Cài đặt** (khuyến nghị) | `./install.sh` | Cài vào `~/.local`, không cần root, dùng được ngay; tự thêm TextVN vào danh sách bộ gõ (GNOME, IBus, Fcitx5). Cho mọi người dùng: `sudo ./install.sh --system`. |
| **Chạy ngay** | `./textvn-portable.sh` | Không cài gì; tắt bằng `./textvn-portable.sh stop` (đăng xuất cũng tự tắt). Thư mục giải nén có thể chỉ đọc. |

Gỡ bản cài: `~/.local/share/textvn/uninstall.sh` (hoặc `sudo /usr/share/textvn/uninstall.sh`).
Thêm `--purge` nếu muốn xoá cả cấu hình.

Yêu cầu: IBus ≥ 1.5 hoặc Fcitx5 ≥ 5.0, glibc tương đương Ubuntu 24.04 trở lên cho bản dựng
sẵn (bản cũ hơn: dựng từ mã nguồn bằng `scripts/install_linux.sh`, xem
[developer guide](developer-guide.md)). Bảng điều khiển cần GTK 4.



---

## 2. Bật/tắt tiếng Việt

- Nhấn rồi nhả **Ctrl + Shift** (không kèm phím khác) — như UniKey. Windows mặc định cũng
  dùng Ctrl + Shift để đổi bàn phím: bộ cài đã chọn sẵn **Dành Ctrl + Shift cho TextVN**; bản
  portable thì bật ô này trong Bảng điều khiển.
- Hoặc **Ctrl + Shift + Space**.
- Hoặc bấm biểu tượng: khay hệ thống (Windows), thanh IBus/Fcitx5 (Linux). **V** = tiếng
  Việt, **E** = tiếng Anh.

Trạng thái V/E được nhớ cho lần khởi động sau.

## 3. Kiểu gõ

**Telex** (mặc định)

| Gõ | Ra | Gõ | Ra |
|---|---|---|---|
| `aa` `ee` `oo` | â ê ô | `aw` `ow` `uw` | ă ơ ư |
| `dd` | đ | `uow` / `uwow` | ươ (`nguowif` → người) |
| `s` `f` `r` `x` `j` | sắc huyền hỏi ngã nặng | `z` | xoá dấu thanh |

Gõ dấu ở cuối từ hay ngay sau nguyên âm đều được (`tieengs` = `tieesng` = tiếng). Bấm lại
đúng phím dấu để gõ chữ đó (`ass` → as). Bật **Caps Lock** vẫn gõ dấu được (`VIEETJ` →
VIỆT); chữ viết tắt gõ bằng Shift như `USA`, `JSON` giữ nguyên.

**VNI**: `1`–`5` sắc huyền hỏi ngã nặng · `6` mũ (â ê ô) · `7` móc (ơ ư) · `8` trăng (ă) ·
`9` đ · `0` xoá dấu. **VIQR** và **Telex đơn giản** (w là chữ thường) cũng có sẵn.

**Quick Telex** (tuỳ chọn): `cc`→ch `gg`→gi `kk`→kh `nn`→ng `qq`→qu `pp`→ph `tt`→th.

## 4. Bảng điều khiển

Windows: nhấn biểu tượng khay. Linux: chạy `textvn-settings`, mở *TextVN* trong menu ứng
dụng, hoặc chọn **Cài đặt TextVN…** trong menu IBus/Fcitx5.

Bố cục chi tiết: [ui-spec.md](release/ui-spec.md).

| Mục | Ý nghĩa |
|---|---|
| Bảng mã | Unicode dựng sẵn (mặc định) · Unicode tổ hợp · TCVN3 (ABC) · VNI Windows — cho tài liệu/font cũ |
| Kiểu gõ | Telex · VNI · VIQR · Telex đơn giản |
| Bật gõ tiếng Việt | như Ctrl + Shift |
| Dấu mới / Dấu cũ | `hoà, thuỷ, khoẻ` hoặc `hòa, thủy, khỏe` (chỉ khác ở vần oa/oe/uy) |
| Khôi phục từ tiếng Anh khi gõ sai | `asdf ` không thành "àd" |
| Đặt dấu tự do | gõ phím dấu ở bất kỳ đâu trong từ |
| Tự viết hoa chữ đầu câu | sau `.` `!` `?` và Enter |
| Quick Telex | như trên |
| Gõ tắt cả khi tắt tiếng Việt | gõ tắt vẫn bung khi đang ở chế độ E |
| Khởi động cùng Windows / Bật hội thoại này khi khởi động | chỉ Windows |
| Dành Ctrl + Shift cho TextVN | chỉ Windows: tắt phím tắt Ctrl + Shift đổi bàn phím của Windows để Ctrl + Shift luôn chuyển V/E; bỏ chọn thì trả lại cho Windows |

Mọi thay đổi lưu ngay, không cần khởi động lại. **Mặc định** đưa mọi tuỳ chọn về ban đầu
nhưng giữ nguyên bảng gõ tắt.

## 5. Gõ tắt

Bảng điều khiển → **Gõ tắt...** → mỗi dòng một mục:

```
vn = Việt Nam
cty = Công ty TNHH
dc = Địa chỉ:\n123 Lê Lợi
# dòng bắt đầu bằng # là ghi chú
```

Gõ chữ tắt rồi nhấn **Tab** (hoặc **Space** — chọn ở cuối cửa sổ) để bung. Chữ tắt không
phân biệt hoa/thường, chỉ bung khi đứng đầu một từ (`xvn` không bung), nội dung tối đa 64 ký
tự; dòng sai được báo và bôi đen khi bấm Lưu.

## 6. Cấu hình

| | Windows | Linux |
|---|---|---|
| Tuỳ chọn | `%APPDATA%\TextVN\config.json` | `~/.config/TextVN/config.json` |
| Trạng thái V/E | `%APPDATA%\TextVN\state.json` | `~/.config/TextVN/state.json` |

Cùng định dạng (schema `schemas/config.v1.schema.json`) — chép file giữa hai máy được.
Bảng điều khiển chỉ sửa đúng khoá bạn đổi và giữ nguyên phần còn lại; file hỏng được giữ lại
thành `config.json.bak`. Gỡ cài đặt không xoá cấu hình.

## 7. Xử lý sự cố

| Hiện tượng | Cách xử lý |
|---|---|
| Windows: không thấy TextVN khi nhấn Win + Space | Bảng điều khiển → **Cài & bật TSF**; chạy `textvn-cli doctor` và xem `%LOCALAPPDATA%\TextVN\logs\register.log`. Nếu COM HKCU OK nhưng API TSF báo `0x80004005`, xem lưu ý cài đặt ở trên. |
| Windows: gõ trong ô mật khẩu không ra dấu | Chủ ý — TextVN tắt trong ô mật khẩu. |
| Windows: Ctrl + Shift lúc được lúc không, hoặc đổi sang bàn phím khác | Windows mặc định cũng dùng Ctrl + Shift để đổi bố cục bàn phím. Bảng điều khiển → chọn **Dành Ctrl + Shift cho TextVN** (bộ cài chọn sẵn), hoặc tự tắt ở *Settings → Time & language → Typing → Advanced keyboard settings → Input language hot keys → Switch Keyboard Layout: (None)*. **Ctrl + Shift + Space** luôn dùng được. `textvn-cli doctor` cho biết Windows còn giữ phím này không. |
| Windows: phần mềm diệt virus cảnh báo | Gói mặc định không dùng hook bàn phím toàn cục hay tiêm mã; xem [antivirus-false-positive.md](specs/antivirus-false-positive.md). Kiểm tra `RELEASE_REPORT.json` trong gói để biết bản đó đã được ký số hay chưa. |
| Linux: vừa cài mà chưa thấy TextVN | Đăng xuất rồi đăng nhập lại (biến môi trường per-user có hiệu lực từ phiên mới), hoặc thêm *TextVN* trong Cài đặt → Bàn phím (GNOME) / `fcitx5-configtool`. |
| Linux: ứng dụng Electron/Chromium (VS Code, Chrome, Discord…) không gõ được | Trên Wayland chạy với `--enable-wayland-ime` (Chrome/Electron ≥ 120) hoặc chạy trên X11 (`--ozone-platform=x11`); đảm bảo `GTK_IM_MODULE=ibus` (hoặc `fcitx`). |
| Linux: Steam / game 32-bit không gõ được với Fcitx5 | Giới hạn của Fcitx5 với ứng dụng 32-bit: chạy Steam với `GTK_IM_MODULE=xim`. |
| Chữ gõ ra bị gạch chân cho tới hết từ | Bình thường: TextVN giữ từ đang gõ trong vùng soạn và chốt khi gõ dấu cách/dấu câu — cách an toàn nhất, không bao giờ xoá nhầm chữ của bạn. |

Báo lỗi: <https://github.com/hunglinhpt/TextVN/issues> — ghi **chính xác chuỗi phím đã gõ**
(ví dụ `dduocj` → ra `...`), ứng dụng, hệ điều hành, và đính kèm — Windows: file tạo bởi
`textvn-cli doctor --export diag.zip`; Linux: `~/.local/state/TextVN/log/textvn.log`. Cả hai
không chứa nội dung bạn gõ.

## Trạng thái từng nền tảng

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | Bản thử nghiệm phát hành (release candidate): gõ thật qua TSF được kiểm thử tự động trên Windows cho cả bản cài và bản portable. |
| Linux IBus / Fcitx5 | Bản thử nghiệm phát hành: kiểm thử tự động với ibus-daemon và fcitx5 thật, cả cài đặt lẫn chạy ngay. |
| macOS | Beta: đã có IMK adapter và gói `.pkg`, CI build/test; còn cần thử GUI/cài-gỡ trên Mac thật trước production. |

Chi tiết kết quả kiểm thử của phiên bản hiện tại: [build-release-report.md](release/build-release-report.md).
