# Nộp TextVN lên Microsoft Store (bản cài .exe — silent install)

> Trạng thái: hướng dẫn cho chủ repo, áp từ 0.2.8. Bộ cài Inno Setup đã hỗ trợ
> đủ cờ im lặng chuẩn và CI kiểm chứng cả hai kịch bản silent mỗi lần release
> (`installer/windows/tests/test-installer.ps1`: per-user `/CURRENTUSER` +
> machine `/ALLUSERS`, đều chạy `/VERYSILENT`).

## 1. Tham số điền vào Partner Center (ô "Installer parameters")

```
/VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

Khuyến nghị thêm `/CURRENTUSER` để luồng cài im lặng **không cần UAC**:

```
/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER
```

Ý nghĩa:

| Tham số | Tác dụng |
|---|---|
| `/VERYSILENT` | Không hiện bất kỳ cửa sổ wizard nào (Inno Setup chuẩn). |
| `/SUPPRESSMSGBOXES` | Chặn mọi hộp thoại phụ (cảnh báo, ghi đè…). |
| `/NORESTART` | Không bao giờ khởi động lại máy sau cài. |
| `/CURRENTUSER` | Cài cho tài khoản hiện tại vào `%LOCALAPPDATA%\Programs\TextVN`, đăng ký TSF per-user — không cần quyền admin (cần `PrivilegesRequiredOverridesAllowed`, có từ 0.2.8). |

## 2. Hành vi của bộ cài trong chế độ silent (đã kiểm chứng)

- Không có trang wizard chặn: `DisableDirPage=yes`, trang Welcome/Ready/Finish
  bị bỏ qua tự nhiên trong `/VERYSILENT`.
- Hai `[Run]` entry `runhidden runasoriginaluser` (`textvn-cli config init` và
  `TextVN.exe --free-ctrl-shift`) là lệnh CLI **tự thoát**, không mở UI.
- Entry tự mở TextVN sau cài có cờ `skipifsilent` → silent không khởi chạy app.
- Lỗi đăng ký TSF KHÔNG hiện hộp thoại: `WizardSilent` guard; báo lỗi qua
  **exit code 10** (`GetCustomSetupExitCode`), thành công = 0.
- Gỡ cài đặt cũng im lặng được: `unins000.exe /VERYSILENT /SUPPRESSMSGBOXES
  /NORESTART`.
- Cấu hình người dùng (`%APPDATA%\TextVN`) được giữ lại sau gỡ.

## 3. Checklist trước khi submit

1. [ ] Tải `TextVN-setup-<ver>-windows-x64.exe` từ GitHub Release (có
       `SHA256SUMS.txt` — ghi lại hash để điền vào Partner Center nếu được hỏi).
2. [ ] Test local trên máy sạch: chạy với đúng bộ tham số ở §1, kiểm exit
       code 0 và gõ được tiếng Việt trong Notepad sau khi mở TextVN từ Start
       Menu (silent KHÔNG tự mở app).
3. [ ] Điền Privacy policy URL (xem `PRIVACY_POLICY.txt` ở gốc repo — dùng
       link GitHub blob/raw của file này; cập nhật nội dung ngay trong repo
       khi có thay đổi).
4. [ ] Điền Description/Notes theo `README.md` mục "Tính năng".
5. [ ] Giấy phép: GPL-3.0-or-later (khai báo đúng ở mục Legal).

## 4. Nếu Store từ chối

- **"Installer requires elevation"**: thêm `/CURRENTUSER` vào tham số.
- **"Installer shows UI"**: kiểm tra lại đã dùng `/VERYSILENT` (không phải
  `/SILENT` — `/SILENT` vẫn hiện thanh tiến trình).
- **"App does not launch after install"**: đúng hành vi — silent không tự mở
  app; người dùng mở từ Start Menu. Không thêm auto-launch cho luồng Store.
