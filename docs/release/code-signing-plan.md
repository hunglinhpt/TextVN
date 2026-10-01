# Kế hoạch ký số (code signing) — TextVN

> **Mục tiêu:** binary Windows (TextVN.exe, textvn-cli.exe, textvn-tsf.dll,
> TextVN-setup.exe) có chữ ký Authenticode chính thống, **chi phí 0 đồng**,
> để qua được SmartScreen/Smart App Control và giảm cảnh báo antivirus.
> Trạng thái hiện tại: `build-release.ps1` hỗ trợ sẵn cờ
> `-SigningCertificateThumbprint` (signtool + cert trong cert store máy build)
> nhưng chưa có chứng chỉ nào — mọi release là `release-candidate` chưa ký.

## 1. So sánh các đường chính thống (cập nhật 2026-10-02)

| Phương án | Chi phí | Khả dụng cho TextVN (maintainer ở Việt Nam, dự án OSS GPL-3.0) | Ghi chú |
|---|---|---|---|
| **SignPath Foundation** | **Miễn phí** cho dự án OSS | ✅ **Khuyến nghị — đường đi chính** | OV-level cert, private key nằm HSM của SignPath; không cần giấy tờ cá nhân — họ xác minh binary build từ repo công khai; ký qua API/GitHub Actions, không cần USB token |
| Azure Artifact Signing (Trusted Signing) | $9.99/tháng (Basic, 5.000 chữ ký) | ❌ Individual chỉ nhận **USA/Canada**; org nhận US/CA/EU/UK | Là dịch vụ managed của Microsoft (tên cũ "Trusted Signing"), SmartScreen nhận reputation nhanh nhất — nhưng región gate chặn maintainer VN; theo dõi mở rộng vùng sau |
| SSL.com eSigner / Open Developer | Có chương trình giảm giá cho OSS (~$25–129/năm) + 30 ngày dùng thử eSigner | ⚠️ trả phí | Cloud HSM FIPS 140-2 L3, ký qua CodeSignTool CLI; hợp khi muốn cert riêng tên cá nhân |
| Certum (Open Source) | Trả phí (~€25–69/năm) | ⚠️ trả phí | CA Ba Lan nổi tiếng với cert OSS; SimplySign cloud |
| Self-signed | Free | ❌ vô ích với SmartScreen | Chỉ dùng khi người dùng TỰ cài cert vào máy — không phù hợp phát hành công khai |
| SignServer (Keyfactor CE) | Free (tool) | ⚠️ chỉ là **server ký tự-host** | Vẫn PHẢI có certificate + HSM riêng — không giải quyết bài toán "chưa có cert"; hữu ích sau này khi đã có cert và muốn tự kiểm soát ký trong hạ tầng |

## 2. Lộ trình đề xuất — SignPath Foundation

### Bước 1 — Nộp đơn (làm một lần, maintainer tự làm)

1. Mở <https://signpath.org/open-source> → submit dự án:
   - Tên + link repo `https://github.com/hunglinhpt/TextVN`
   - License: `GPL-3.0-or-later` (OSI-approved ✓)
   - Link download công khai: GitHub Releases (đã có v0.2.0 → v0.2.4)
2. Chờ duyệt (họ xem repo public, lịch sử phát hành; project cần "đủ tuổi").
3. Được duyệt → tạo tổ chức trên <https://signpath.io> (gói Open Source),
   nhận **API token** + **Signing Policy** từ Foundation.

### Bước 2 — Tạo artifact configuration trên SignPath.io

- Input: `TextVN-setup-<ver>-windows-x64.exe` (file duy nhất cần ký per-release
  nếu ký installer; portable nên ký cả 3 binary TRƯỚC khi zip — xem bước CI).
- Signing policy: của Foundation (chờ duyệt policy từ SignPath Foundation).

### Bước 3 — Bật ký trong GitHub Actions (đã chờ sẵn trong `release.yml`)

`release.yml` có job-step mẫu gated theo secret `SIGNPATH_API_TOKEN` — khi
token được cấu hình trong repo Settings → Secrets, signing tự chạy; không có
secret → bỏ qua (giữ đúng trạng thái release-candidate chưa ký hiện tại).
Khung REST chính thức: `POST https://app.signpath.io/api/v1/{org}/signing-requests`
(với file cần ký + policy) → poll → tải artifact đã ký. Đối chiếu docs mới:
<https://signpath.io/docs> (trang "GitHub integration").

### Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai)

`build-release.ps1 -SigningCertificateThumbprint <SHA1>` — signtool ký
`TextVN.exe`, `textvn-cli.exe`, `textvn_win_tsf.dll` (+ setup exe) bằng cert
trong cert store; script từ chối publish nếu chữ ký không `Valid`.

## 3. Smart App Control / SmartScreen — kỳ vọng thực tế

- [Smart App Control](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control)
  ưu tiên app có chữ ký từ cert "well-known". **Chữ ký mới chưa có reputation
  không mở khoá ngay** — cần tích luỹ lượt tải/cài (telemetry của Microsoft) và
  không bị flags. Ký là điều kiện cần, không phải điều kiện đủ.
- SmartScreen tương tự: chữ ký OV + thời gian + lượt tải → reputation tăng dần.
  EV cert bỏ qua "reputation build" nhưng không free.
- Truyền thông repo nên ghi rõ trong release notes: hướng dẫn bấm
  "More info → Run anyway" cho bản chưa ký/ký mới.

## 4. Việc đã wire trong repo

| File | Nội dung |
|---|---|
| `build-release.ps1` | `-SigningCertificateThumbprint` (signtool local, có sẵn từ trước) + verify chữ ký `Valid` |
| `release.yml` | Step mẫu SignPath gated `SIGNPATH_API_TOKEN` (thêm 2026-10-02) — bỏ comment + cấu hình secrets khi được duyệt |
| Tài liệu này | So sánh + lộ trình + kỳ vọng SmartScreen |

## Nguồn

- SignPath Foundation: <https://signpath.org> · <https://signpath.io/solutions/open-source-community>
- signtool: <https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool>
- Smart App Control signing:
  <https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control>
- Azure Artifact Signing:
  <https://learn.microsoft.com/en-us/azure/artifact-signing/> ·
  [certificate management](https://learn.microsoft.com/en-us/azure/artifact-signing/concept-certificate-management) ·
  [pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing)
- SignServer (Keyfactor CE, self-host):
  <https://github.com/Keyfactor/signserver-ce>
