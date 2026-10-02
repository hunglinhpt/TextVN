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

## 6. Tích hợp chứng thư số (khi xin được)

Kể từ 0.2.6, pipeline đã có sẵn nhánh ký **SignPath** opt-in. Khi có chứng thư:

### Phương án A — SignPath Foundation (OSS, miễn phí, khuyên dùng)
1. Nộp đơn SignPath Foundation cho repo (`docs/release/code-signing-plan.md`
   đã ghi link + thông tin điền mẫu).
2. Sau khi được duyệt: thêm 4 secret trong repo Settings → Secrets → Actions:
   `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_KEY`,
   `SIGNPATH_POLICY`.
3. Kể từ tag kế tiếp, workflow `release-candidate` TỰ ĐỘNG ký 3 binary
   (TextVN.exe, textvn-cli.exe, textvn-tsf.dll) + bộ cài `.exe` trước khi
   đóng gói; `RELEASE_REPORT.json` sẽ ghi `"signing": "signpath"`.
4. Đối chiếu bằng chứng: mục "authenticode" trong build-release-report chuyển
   từ `not-signed` sang `passed`.

### Phương án B — Chứng thư mua riêng (SSL.com eSigner / Certum / DigiCert)
1. Mua/đăng ký chứng thư **OV/EV hoặc Cloud signing** (giữ khóa trên token/
   cloud — Windows 9+ yêu cầu bảo mật khóa; chứng thư file .pfx không được
   SmartScreen công nhận như EV).
2. Đặt secret `SIGNTOOL_THUMBPRINT` + `SIGNTOOL_SHA1` (hash chứng thư) trong
   Actions (đã hỗ trợ sẵn qua `build-release.ps1 -SignCertificateThumbprint`
   — xem `build-release.ps1` phần SigningRequested; nếu dùng HSM/cloud cần
   provider/signing  tool tương ứng chạy trên runner).
3. Sign test trên local trước khi đưa vào pipeline:
   `signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 <file>`.

### Lưu ý kiểm tra sau khi ký (bắt buộc trước khi release)
- `Get-AuthenticodeSignature <exe>` → Status = Valid.
- Tải bản ký về chạy: SmartScreen hiển thị thông tin publisher
  **LinhBH.CoM** (không còn "Unknown publisher").
- Bộ cài đã ký không được đổi sau khi upload lên Partner Center (hash khớp
  `SHA256SUMS.txt`).
