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

## 5. Quy trình publish lên Microsoft Store — từng bước

1. **Tài khoản**: đăng ký [Partner Center](https://partner.microsoft.com/dashboard)
   (tài khoản cá nhân ~$19 hoặc công ty ~$99, thuế/ID xác minh một lần).
2. **Reserve app name**: Apps and Games → Overview → New product → Name.
   Đặt đúng tên hiển thị: `TextVN - Bộ gõ tiếng Việt` (hoặc giữ tên đã reserve).
3. **Chuẩn bị gói**:
   - Tải `TextVN-setup-<ver>-windows-x64.exe` từ GitHub Release (khuyên dùng
     bản có cờ CI xanh; `RELEASE_REPORT.json` bên trong ZIP portable cho biết
     commit + checksum).
   - Ghi lại SHA-256 từ `SHA256SUMS.txt`.
4. **Tạo submission**: Start submission →
   - **Product name / description**: chép từ `README.md` mục mô tả + ghi rõ
     điểm khác biệt (xem README "Điểm vượt trội so với các bộ gõ khác").
   - **Installer parameters** (quan trọng nhất):
     `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER`
   - **Privacy policy URL**: link GitHub blob của `PRIVACY_POLICY.txt`
     (`https://github.com/hunglinhpt/TextVN/blob/main/PRIVACY_POLICY.txt`).
   - **Support URL**: `https://github.com/hunglinhpt/TextVN/issues`.
   - **Website**: `https://linhbh.com` (nếu đã có site; không thì dùng repo).
   - **Age rating**: điền survey — kết quả thường là E (Everyone) vì không có
     nội dung nhạy cảm; IME không thu thập dữ liệu nên khai báo không có quyền.
   - **Screenshots**: chụp Bảng điều khiển (màn 1), gõ Telex trong Notepad
     (màn 2), menu khay hệ thống (màn 3) — 1366x768 trở lên, PNG/JPG.
5. **Certification**: đợi review (thường 1–3 ngày cho bản .exe silent). Nếu
   bị từ chối, tra §4 bên dưới trước khi nộp lại.
6. **Sau khi pass**: chọn Publish; listing hiện lên Store trong vài giờ.
7. **Bản cập nhật**: mỗi lần phát hành 0.2.x, lặp lại bước 3–6 với installer
   mới; tham số giữ nguyên. Label submission với tên phiên bản để dễ tra.

## 6. Loại gói và chứng thư số

> Cập nhật 2026-10-02 (điều chỉnh theo thực tế Partner Center + quyết định
> chủ repo): **Partner Center giờ chỉ nhận gói `.exe` hoặc `.msi`** (không còn
> luồng MSIX cho loại submission này). Bộ cài Inno Setup `.exe` của TextVN
> khớp loại này; luồng silent ở §1 là bắt buộc.

**Về chứng thư số:**

- **Luồng Store**: sau khi submission được **chứng nhận (certified)**, luồng
  phân phối của Store đảm bảo phần app được ký theo cơ chế của Microsoft —
  chủ repo sẽ có cert/ký từ luồng này, KHÔNG cần mua chứng thư riêng để phân
  phối qua Store. Điền mục chi phí/certificate theo hướng dẫn của Partner
  Center tại thời điểm submit.
- **Phân phối trực tiếp (GitHub Releases)**: bản tải trực tiếp vẫn là
  "unsigned" → SmartScreen hiện cảnh báo "Unknown publisher" — chấp nhận cho
  bản OSS hiện tại. Khi chủ repo đã có cert từ luồng Store (hoặc muốn bật
  sớm), pipeline đã có sẵn 2 nhánh opt-in không cần sửa code:
  1. **SignPath Foundation** (miễn phí cho OSS): thêm 4 secret
     `SIGNPATH_API_TOKEN` / `SIGNPATH_ORGANIZATION_ID` / `SIGNPATH_PROJECT_KEY` /
     `SIGNPATH_POLICY` → workflow tự ký 3 binary + bộ cài trước khi đóng gói.
  2. **Chứng thư riêng** (nếu dùng): đặt secret
     `SIGNTOOL_CERTIFICATE_THUMBPRINT`; `build-release.ps1 -SignCertificateThumbprint`
     ký bằng signtool (đã hỗ trợ sẵn).

### Lưu ý kiểm tra sau khi có cert/ký (bắt buộc trước khi release)
- `Get-AuthenticodeSignature <exe>` → Status = Valid.
- Bộ cài đã ký không được đổi sau khi upload lên Partner Center (hash khớp
  `SHA256SUMS.txt`).
