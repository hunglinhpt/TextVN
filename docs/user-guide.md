# Hướng dẫn sử dụng TextVN

TextVN là bộ gõ tiếng Việt mã nguồn mở cho **Windows** (Text Services Framework),
**macOS** (Input Method Kit) và **Linux** (IBus, Fcitx5). Ba nền tảng dùng chung một engine,
một bảng điều khiển với cùng tuỳ chọn và cùng file cấu hình — học một lần, dùng ở đâu cũng
giống nhau.

---

## Cài đặt im lặng (cho quản trị / Microsoft Store)

Bộ cài hỗ trợ cài không tương tác:

```
TextVN-setup-<bản>-windows-x64.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

- `TextVN-setup-<bản>-windows-x64.exe` mặc định cài **cho tài khoản hiện tại**
  (`%LOCALAPPDATA%\Programs\TextVN`, không cần admin). Cài cho mọi người dùng: dùng
  `TextVN-setup-<bản>-windows-x64-machine.exe` (Program Files, ARP ở HKLM, cần quyền
  admin) hoặc thêm `/ALLUSERS` khi chạy bản thường bằng quyền admin.
- Chế độ im lặng **chỉ chép file** và **luôn trả exit code 0** khi đã chép đủ; không tự
  mở app. Bộ gõ được đăng ký khi người dùng mở TextVN lần đầu (mỗi tài khoản Windows tự
  được đăng ký ở lần đầu TextVN chạy cho tài khoản đó); lỗi đăng ký ghi ở
  `%LOCALAPPDATA%\TextVN\logs\register.log`. Chỉ khi cài **có giao diện** mà đăng ký
  thất bại, bộ cài mới trả exit code 10.
- Microsoft Store: đường nộp hiện tại là gói **MSIX** (Store tự ký khi phát hành). File
  `.msix` trên GitHub Release chưa ký nên **không cài trực tiếp được**. Chi tiết:
  `docs/release/msix-submission.md`. Chính sách riêng tư: `PRIVACY_POLICY.txt` (ở gốc repo).

## 1. Cài đặt

Mỗi bản phát hành có hai cách dùng — chọn một.

### Windows

| Cách | Làm gì | Khi nào nên dùng |
|---|---|---|
| **Cài đặt** | Chạy `TextVN-setup-<phiên bản>-windows-x64.exe` → Tiếp → Cài. Bộ cài mặc định cài **cho tài khoản của bạn** (`%LOCALAPPDATA%\Programs\TextVN`, không cần admin). Muốn cài cho mọi tài khoản trên máy: dùng `TextVN-setup-<phiên bản>-windows-x64-machine.exe` (Program Files, UAC một lần). | Máy của bạn, dùng lâu dài: tự khởi động cùng Windows, có trong Settings → Apps để gỡ. |
| **Giải nén dùng ngay** | Giải nén `TextVN-portable-<phiên bản>-windows-x64-*.zip` → nhấn đúp `TextVN.exe`. | Máy mượn, USB, dùng thử. Không ghi gì vào Program Files. |
| **Microsoft Store** | Đang nộp (gói MSIX). Khi có trên Store: cài, mở TextVN một lần để app đăng ký bộ gõ cho tài khoản của bạn. | Muốn Store tự cập nhật. |

Lần đầu TextVN chạy cho một tài khoản Windows, app tự đăng ký bộ gõ cho tài khoản đó (kể cả
khi đã cài cho mọi người dùng). Nếu vẫn chưa gõ được: mở Bảng điều khiển → **[Cài & bật TSF]**,
rồi chọn *TextVN* trong danh sách bộ gõ (**Win + Space**). Nâng cấp lên bản mới **giữ nguyên**
kiểu gõ, tuỳ chọn và thiết lập theo từng ứng dụng của bạn.

**Windows 11 24H2+ (lỗi B7):** một số máy từ chối kích hoạt bộ gõ chỉ đăng ký cho tài khoản
(`register.log` ghi `Đăng ký qua API TSF → 0x80004005` trong khi `COM server HKCU → OK`).
Vì an toàn, TextVN **chỉ** đăng ký cho cả máy khi chính nó nằm trong Program Files (bộ cài
`-machine.exe`); bản cài cho tài khoản, bản portable và bản Store không bao giờ xin UAC mà
báo một lần và hướng dẫn cài `TextVN-setup-<phiên bản>-windows-x64-machine.exe`. Không chạy
`textvn-cli.exe` portable bằng quyền admin (CLI cũng từ chối đăng ký phạm vi máy cho DLL
ngoài Program Files).

Gỡ: *Settings → Apps → TextVN → Uninstall* (bản cài, bản Store) hoặc chuột phải
`uninstall.ps1` → *Run with PowerShell* (bản portable — script chỉ xoá đúng các file của
TextVN, thư mục chỉ bị xoá khi đã trống). Menu khay **Gỡ cài đặt** làm đúng việc tương ứng cho
từng loại.

### macOS (13 Ventura trở lên — Apple Silicon và Intel)

**Yêu cầu**: macOS 13+. Không cần cài thêm gì — engine Rust universal đã nhúng sẵn trong app.
Bản phát hành gồm hai thành phần: `TextVN-IM.app` (bộ gõ) và `TextVN.app` (menu bar + Cài đặt).

**Cách khuyến nghị — bộ cài `.pkg`:** tải `TextVN-mac-v<phiên bản>.pkg` rồi mở để cài, hoặc
cài riêng cho bạn, không cần sudo:

```bash
installer -pkg TextVN-mac-v<phiên bản>.pkg -target CurrentUserHomeDirectory
```

Gói cài `TextVN-IM.app` vào `~/Library/Input Methods/` và `TextVN.app` vào `~/Applications/`.
Sau đó vào System Settings → Keyboard → Input Sources → Edit… → (+) → Vietnamese → **TextVN**.

**Bản nén:** `TextVN-macos-universal-v<phiên bản>.zip` (hoặc `.tar.gz`) chứa cả hai app:

```bash
cp -R TextVN-IM.app ~/Library/Input\ Methods/
cp -R TextVN.app ~/Applications/        # hoặc /Applications
xattr -dr com.apple.quarantine ~/Library/Input\ Methods/TextVN-IM.app ~/Applications/TextVN.app
```

**Chuyển bộ gõ**: `Control + Space` (macOS default) hoặc menu Input Source trên menu bar.

**Gỡ cài đặt:** menu bar TextVN → **Gỡ cài đặt TextVN...** (chạy
`TextVN.app/Contents/Resources/uninstall_macos.sh`; bản cài cho mọi người dùng sẽ hỏi quyền
quản trị). Cấu hình ở `~/Library/Application Support/TextVN` được giữ lại. Sau đó bỏ TextVN
khỏi System Settings → Keyboard → Input Sources nếu còn.

> **Chữ ký:** mỗi file phát hành có chữ ký GPG (`.asc`) và Sigstore (`.cosign.sig`/`.cosign.cert`),
> kèm `SHA256SUMS.txt` ký clearsign (`docs/release/signing.md`). Gói **chưa** được ký Apple
> Developer ID/notarize (`docs/release/signing-status-mac.md`), nên Gatekeeper chặn lần đầu mở
> `.pkg`/`TextVN.app`: vào System Settings → Privacy & Security → **Open Anyway** (macOS 15
> không còn cách click phải → Open).

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

- Nhấn rồi nhả **Ctrl + Shift** (không kèm phím khác) — như UniKey trên cả 3 nền tảng:
  - **Windows**: TextVN giải phóng `Ctrl + Shift` khỏi phím tắt chuyển ngôn ngữ mặc định của Windows (bộ cài chọn sẵn; bản Store hỏi trước khi đổi); tray có bộ dò tap **chỉ quan sát** (không ăn phím, không gõ thay) nên tổ hợp được nhận ở **mọi ứng dụng** — kể cả khi bạn đang đứng ở bàn phím khác trong Win+Space — đổi mode tức thì và đồng bộ icon khay hệ thống **[V]** (Tím) ↔ **[E]** (Xanh). Hai lần bấm trong 0,25 giây tính là một (chống lật đôi khi cả tray lẫn engine cùng nhận một lần bấm).
  - **macOS**: TextVN nhận diện tổ hợp `Ctrl + Shift` tap trong `flagsChanged`, đồng bộ với menu bar app `TextVN.app` để chuyển đổi chế độ và hiển thị rõ chỉ báo **[V]** / **[E]** trên thanh menu bar.
  - **Linux**: IBus và Fcitx5 tự động cập nhật icon `textvn_v` ↔ `textvn_e` và nhãn `V` ↔ `E` trên thanh trạng thái / khay hệ thống.
- Hoặc nhấn tổ hợp **Ctrl + Shift + Space**.
- **Gõ nhầm sang tiếng Anh**: nếu kết quả biến đổi KHÔNG phải âm tiết Việt
  hợp lệ hoặc là từ tiếng Anh thông dụng (mà không đụng cách gõ Telex của từ
  Việt thông dụng), TextVN tự trả lại đúng chuỗi phím bạn gõ khi kết thúc từ
  (dấu cách/dấu câu). Nghiêm trọng hơn: **Escape** khôi phục nguyên chuỗi phím
  ngay giữa chừng; **Tab** hoàn tất từ tiếng Anh đang gõ dở thành từ đầy đủ
  theo từ điển dựng sẵn. Tắt toàn bộ bằng bỏ chọn "Khôi phục từ tiếng Anh khi
  gõ sai"; **Từ điển EN...** để thêm từ riêng — từ bạn thêm thắng mọi phỏng
  đoán của engine (kể cả khi kết quả trùng từ Việt thông dụng như
  `cow`/`cơ`). Vị trí từng nền tảng: **Windows** Bảng điều khiển → "Từ điển
  EN..."; **macOS** Settings → "Từ điển EN..."; **Linux** cửa sổ cài đặt →
  nút "Từ điển EN..." cạnh "Gõ tắt...". Quy tắc ưu tiên đầy đủ:
  [language-detection.md](specs/language-detection.md).
- Hoặc click chuột trái trực tiếp vào biểu tượng trên khay hệ thống (Windows) / menu bar (macOS) / status area (Linux) để chuyển đổi nhanh giữa tiếng Việt và tiếng Anh.
- Khi đang soạn thảo, chữ hiển thị tự nhiên, hoàn toàn không bị gạch chân (clean composition/preedit) trên cả Word, Notepad, Chrome, Safari và các ứng dụng Linux.

Trạng thái V/E được ghi nhớ xuyên suốt các lần khởi động.

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
`9` đ · `0` xoá dấu. **VIQR** và **Telex đơn giản** (`w` chỉ là dấu sừng sau nguyên âm:
`aw ow uw`) cũng có sẵn.

**Quick Telex** (tuỳ chọn): `cc`→ch `gg`→gi `kk`→kh `nn`→ng `qq`→qu `pp`→ph `tt`→th.

## 4. Bảng điều khiển

Windows: nhấn biểu tượng khay. macOS: menu bar TextVN → **Cài đặt...** (⌘,). Linux: chạy
`textvn-settings`, mở *TextVN* trong menu ứng dụng, hoặc chọn **Cài đặt TextVN…** trong menu
IBus/Fcitx5.

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
| Khởi động cùng Windows (macOS: Khởi động cùng OS) / Bật hội thoại này khi khởi động | Windows và macOS |
| Gõ không gạch chân (Non-preedit) · Chạy ngầm trong menu bar | chỉ macOS |
| Dành Ctrl + Shift cho TextVN | chỉ Windows: tắt phím tắt Ctrl + Shift đổi bàn phím của Windows để Ctrl + Shift luôn chuyển V/E; bỏ chọn thì trả lại cho Windows (đúng giá trị trước đó, gồm cả phím đổi ngôn ngữ) và TextVN không còn tự kéo bạn về TextVN khi Windows đổi bàn phím. Gỡ cài đặt cũng trả lại như vậy |

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

| | Windows | macOS | Linux |
|---|---|---|---|
| Tuỳ chọn | `%APPDATA%\TextVN\config.json` | `~/Library/Application Support/TextVN/config.json` | `~/.config/TextVN/config.json` |
| Trạng thái V/E | `%APPDATA%\TextVN\state.json` | khoá `enabled` trong `config.json` (không có `state.json`) | `~/.config/TextVN/state.json` |

Cùng định dạng (schema `schemas/config.v1.schema.json`) — chép file giữa các máy được.
Bảng điều khiển chỉ sửa đúng khoá bạn đổi và giữ nguyên phần còn lại; file hỏng được giữ lại
thành `config.json.bak`. Gỡ cài đặt không xoá cấu hình.

## 7. Xử lý sự cố

| Hiện tượng | Cách xử lý |
|---|---|
| Windows: không thấy TextVN khi nhấn Win + Space | Bảng điều khiển → **Cài & bật TSF**; chạy `textvn-cli doctor` và xem `%LOCALAPPDATA%\TextVN\logs\register.log`. Nếu COM HKCU OK nhưng API TSF báo `0x80004005`, xem lưu ý cài đặt ở trên. |
| Windows: gõ trong ô mật khẩu không ra dấu | Chủ ý — TextVN tắt trong ô mật khẩu. |
| Windows: Ctrl + Shift lúc được lúc không, hoặc đổi sang bàn phím khác | Windows mặc định cũng dùng Ctrl + Shift để đổi bố cục bàn phím — nếu Windows còn giữ phím này, một lần bấm vừa đổi mode TextVN vừa đổi bàn phím hệ thống. Bảng điều khiển → chọn **Dành Ctrl + Shift cho TextVN** (bộ cài chọn sẵn), hoặc tự tắt ở *Settings → Time & language → Typing → Advanced keyboard settings → Input language hot keys → Switch Keyboard Layout: (None)*. **Ctrl + Shift + Space** luôn dùng được. `textvn-cli doctor` cho biết Windows còn giữ phím này không. |

| Windows: chọn TextVN xong vẫn gõ ra tiếng Anh/không ra chữ, hoặc TextVN không có trong Win+Space | (1) Win11 24H2+ có thể từ chối kích hoạt bộ gõ chỉ đăng ký cho tài khoản (HKCU) — bản cài cho tài khoản, portable và Store không tự xin UAC; (2) TextVN từng bị mất khỏi danh sách Win+Space sau nâng cấp liên tục. Cách xử lý: chạy **bộ cài cho mọi người dùng** `TextVN-setup-<phiên bản>-windows-x64-machine.exe` (Program Files, UAC một lần) — luồng `/ALLUSERS` được CI kiểm chứng đầy đủ; sau cài, đăng xuất/đăng nhập nếu vẫn chưa thấy. Kiểm tra nhanh: PowerShell `Get-WinUserLanguageList` phải thấy dòng chứa `{6F2B9C31-8E47-4D2A-9C84-1D5A3E70F9B8}`. |
| Windows: phần mềm diệt virus / SmartScreen cảnh báo | Gói mặc định không dùng hook bàn phím toàn cục hay tiêm mã; xem [antivirus-false-positive.md](specs/antivirus-false-positive.md). Bản phát hành chưa có chữ ký Authenticode (đang chờ SignPath Foundation) — kiểm `checks.authenticode` trong `RELEASE_REPORT.json` của gói; tính toàn vẹn kiểm bằng chữ ký GPG/Sigstore (`release/signing.md`). |
| Linux: vừa cài mà chưa thấy TextVN | Đăng xuất rồi đăng nhập lại (biến môi trường per-user có hiệu lực từ phiên mới), hoặc thêm *TextVN* trong Cài đặt → Bàn phím (GNOME) / `fcitx5-configtool`. |
| Linux: ứng dụng Electron/Chromium (VS Code, Chrome, Discord…) không gõ được | Trên Wayland chạy với `--enable-wayland-ime` (Chrome/Electron ≥ 120) hoặc chạy trên X11 (`--ozone-platform=x11`); đảm bảo `GTK_IM_MODULE=ibus` (hoặc `fcitx`). |
| Linux: Steam / game 32-bit không gõ được với Fcitx5 | Giới hạn của Fcitx5 với ứng dụng 32-bit: chạy Steam với `GTK_IM_MODULE=xim`. |
| Chữ gõ ra bị gạch chân cho tới hết từ | Bình thường: TextVN giữ từ đang gõ trong vùng soạn và chốt khi gõ dấu cách/dấu câu — cách an toàn nhất, không bao giờ xoá nhầm chữ của bạn. |

Báo lỗi: <https://github.com/hunglinhpt/TextVN/issues> — ghi **chính xác chuỗi phím đã gõ**
(ví dụ `dduocj` → ra `...`), ứng dụng, hệ điều hành, và đính kèm — Windows: file tạo bởi
`textvn-cli doctor --export diag.zip` (xem nhanh trong cửa sổ: `textvn-cli doctor --pause` — lối
tắt *Kiem tra he thong (TextVN Doctor)* trong Start menu của bản cài); macOS: thư mục `~/Library/Logs/TextVN/` (chỉ có log khi
chạy với biến môi trường `TEXTVN_LOG=1`); Linux: `~/.local/state/TextVN/log/textvn.log`. Các
file này không chứa nội dung bạn gõ.

## Trạng thái từng nền tảng

| Nền tảng | Trạng thái |
|---|---|
| Windows 10/11 x64 (TSF) | Bản thử nghiệm phát hành (release candidate): gõ thật qua TSF được kiểm thử tự động trên Windows cho bộ cài, bản portable và gói MSIX (cài thử thật). Còn chờ chữ ký **Authenticode** qua SignPath Foundation (đang chờ duyệt) — trước đó SmartScreen/AV có thể cảnh báo. Microsoft Store: đang nộp gói MSIX. |
| Linux IBus / Fcitx5 | Bản thử nghiệm phát hành: tarball `install.sh` (per-user hoặc `--system`) + chạy ngay, Settings GTK4; kiểm thử tự động với ibus-daemon và fcitx5 thật, cả cài đặt lẫn chạy ngay. Chưa có `.deb`/`.rpm`. |
| macOS 13+ (IMK) | Bản thử nghiệm phát hành: bộ gõ IMK, app menu bar, gói `.pkg`; CI build 2 kiến trúc và chạy test. **Chưa** ký Apple Developer ID/notarize — xem lưu ý Gatekeeper ở §1. |

Mọi file phát hành (cả 3 nền tảng) đều có chữ ký GPG `.asc` + Sigstore `.cosign.sig`/`.cosign.cert`.
Kiểm tra nhanh (fingerprint `3921 595A BC96 1199 F153  03B6 C45B 84D0 C7F4 A822`):

```bash
gpg --import gpg-release-key.asc
gpg --verify SHA256SUMS.txt
gpg --decrypt SHA256SUMS.txt 2>/dev/null | sha256sum -c --ignore-missing
```

Chi tiết (kể cả kiểm bằng cosign): [signing.md](release/signing.md). Kết quả kiểm thử của
phiên bản hiện tại: [build-release-report.md](release/build-release-report.md).
