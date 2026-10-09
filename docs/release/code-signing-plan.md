# Kế hoạch ký số (code signing) — TextVN

> **Mục tiêu:** binary Windows (`TextVN.exe`, `textvn-cli.exe`, `textvn-tsf.dll`,
> `textvn-tsf-x86.dll`, `TextVN-setup-<ver>-windows-x64.exe` và `-machine.exe`) có chữ ký
> Authenticode chính thống, **chi phí 0 đồng**, để qua được SmartScreen/Smart App Control,
> policy 10.2.9 của Microsoft Store và giảm cảnh báo antivirus.
>
> **Trạng thái (2026-10-09):** lớp integrity/provenance ĐÃ chạy cho mọi asset từ v0.2.27
> (GPG `.asc` + Sigstore keyless + `SHA256SUMS.txt` clearsign, bắt buộc trong CI —
> `signing.md`). **Authenticode là hạng mục Windows DUY NHẤT còn thiếu**: đã nộp SignPath
> Foundation, **đang chờ duyệt**. Đường ký đã nối sẵn và tự chạy khi có secret (§2 Bước 3);
> chưa có secret thì bản phát hành vẫn là `release-candidate` chưa Authenticode
> (`RELEASE_REPORT.json` ghi `checks.authenticode = not-signed`).

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
   - Link download công khai: GitHub Releases (v0.2.0 → v0.2.27; từ v0.2.27 mọi asset có chữ ký GPG + Sigstore)
2. Chờ duyệt (họ xem repo public, lịch sử phát hành; project cần "đủ tuổi").
3. Được duyệt → tạo tổ chức trên <https://signpath.io> (gói Open Source),
   nhận **API token** + **Signing Policy** từ Foundation.

### Bước 2 — Tạo project + artifact configuration trên SignPath.io

- **Project** (slug → secret `SIGNPATH_PROJECT_SLUG`) gắn repo GitHub.
- **Artifact configuration** (slug → secret `SIGNPATH_ARTIFACT_CONFIGURATION`): loại **ZIP**,
  **deep sign** mọi `*.exe`/`*.dll` bên trong — `build-release.ps1` gửi 4 PE trong MỘT ZIP
  (một lần duyệt cho cả bộ binary); mỗi bộ cài được gửi riêng (một file `.exe`).
- **Signing policy** (slug → secret `SIGNPATH_POLICY`, vd. `release-signing`) do Foundation
  cấp; gói OSS có thể cần người duyệt tay mỗi yêu cầu (trạng thái `WaitingForApproval`).

### Bước 3 — Bật ký trong GitHub Actions (đã nối sẵn — chỉ cần đặt secret)

Không có bước nào phải bỏ comment. Job `windows` của `release.yml` luôn truyền env
`SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` (tên cũ
`SIGNPATH_PROJECT_KEY` vẫn nhận), `SIGNPATH_POLICY`, `SIGNPATH_ARTIFACT_CONFIGURATION`; khi
`SIGNPATH_API_TOKEN` có giá trị:

1. `build-release.ps1` gọi `tools/win/sign-signpath.ps1` cho `TextVN.exe`,
   `textvn-cli.exe`, `textvn_win_tsf.dll` (đóng gói thành `textvn-tsf.dll`),
   `textvn-tsf-x86.dll` (+ `textvn-hook.exe` ở gói Compatibility) — TRƯỚC khi đóng ZIP
   portable/bộ cài — rồi ký `TextVN-setup-<ver>-windows-x64.exe`.
2. Bước "Build + validate machine installer" của `release.yml` ký
   `TextVN-setup-<ver>-windows-x64-machine.exe` (file nộp Store EXE) trước khi validate.
3. `sign-signpath.ps1` dùng REST API công bố của SignPath: `POST
   https://app.signpath.io/API/v1/{org}/SigningRequests` (multipart: `ProjectSlug`,
   `SigningPolicySlug`, `ArtifactConfigurationSlug`, `Description`, `Artifact`) → poll
   `Location` tới `Completed` (chờ qua `WaitingForApproval`/`Processing`…, tối đa 60 phút;
   `Failed`/`Denied`/`Canceled` = lỗi) → tải `{request}/SignedArtifact` → kiểm
   `Get-AuthenticodeSignature` = `Valid`. Lỗi ở bất kỳ bước nào = job fail.
4. `RELEASE_REPORT.json` ghi `checks.authenticode = signpath`.

Giới hạn đã biết: `unins000.exe` (uninstaller do Inno sinh lúc cài) **không** được ký.
Tài liệu API: <https://about.signpath.io/documentation/build-system-integration#rest-api>.

### Bước 4 — Khi đã có cert riêng (Certum/SSL.com, tương lai)

`build-release.ps1 -SigningCertificateThumbprint <SHA1>` — signtool ký
`TextVN.exe`, `textvn-cli.exe`, `textvn_win_tsf.dll`, `textvn-tsf-x86.dll` (+ cả hai bộ
cài khi có `-BuildInstaller -MachineInstaller`) bằng cert trong cert store; script từ chối
publish nếu chữ ký không `Valid`.

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
| `build-release.ps1` | `-SigningCertificateThumbprint` (signtool local) hoặc SignPath khi có `SIGNPATH_API_TOKEN`: ký 4 PE + bộ cài per-user (+ `-machine.exe` khi `-MachineInstaller`), verify chữ ký `Valid` |
| `release.yml` | Job `windows` truyền env `SIGNPATH_*`; bước "Build + validate machine installer" ký `-machine.exe`. Không cần bỏ comment — chỉ cần đặt secret |
| `tools/win/sign-signpath.ps1` | Client REST API SignPath (ZIP deep sign, poll cả bước duyệt tay, kiểm `Valid`) — sửa theo API công bố 2026-10-08 (R2-72) |
| `tools/release/sign-artifacts.sh` | GPG `.asc` + Sigstore keyless cho mọi asset, clearsign `SHA256SUMS.txt` (từ v0.2.27; bắt buộc trong CI từ R2-74) |
| `tools/win/virustotal-scan.ps1` | Quét VirusTotal best-effort trong `release.yml` (secret `VIRUSTOTAL_API_KEY`) |
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
