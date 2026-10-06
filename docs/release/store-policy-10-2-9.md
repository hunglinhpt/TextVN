# Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

> Trạng thái: **Microsoft đã trả lời chính thức** (2026-10-06) sau khi nộp
> `approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe`. Tài liệu này
> ghi nhận yêu cầu, đối chiếu với hiện trạng, và chốt kế hoạch hành động.
> Liên quan: `store-submission.md`, `msix-submission.md`, `win-test-common-errors.md` (B13/B19).

## 1. Microsoft nói chính xác điều gì

Trích phản hồi (Package validation — Technical requirement policies):

> **10.2.9 Security — Package Submissions**: "The binary and all of its Portable
> Executable (PE) files has been signed with a certificate that has been observed
> being abused to sign malicious content **or must be digitally signed with a code
> sign certificate that chains up to a certificate issued by a Certificate Authority
> (CA) that is part of the Microsoft Trusted Root Program**. To code sign your app,
> you can use **Trusted Signing**… If your EXE or MSI cannot comply with Microsoft
> Store policy 10.2.9, you can consider **repackaging your existing EXE or MSI to
> MSIX format**. Microsoft Store offers many complimentary benefits for MSIX format
> such as **code signing, hosting** etc… Note that you have to **delete your app
> name from existing Win32 app** in Partner Center in case you want to use the
> same for MSIX packaged app."
>
> Gói bị ảnh hưởng: `approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe`
> — Code signing type: **Unsigned** — "Package should be signed with SHA256 or
> higher algorithm".

## 2. Điều này thay đổi nhận định gì

- **Các vòng validation trước (3 mục đỏ: Silent/ARP/Bundleware)** chỉ là lớp kiểm
  đầu. Lớp chốt là **10.2.9: exe KHÔNG KÝ không được phép vào Store, không ngoại lệ**.
- Nghĩa là: mọi nỗ lực tinh chỉnh installer (per-user/machine, ARP exact,
  publisher từng chữ…) đều **đúng nhưng chưa đủ** — thiếu chữ ký.
- MS **khuyên chính hãng**: (a) ký bằng Trusted Signing, hoặc (b) chuyển sang
  **MSIX — Store tự ký miễn phí**, không cần mua chứng thư.
- Lưu ý gài: muốn dùng tên "TextVN" cho sản phẩm MSIX thì phải **xoá tên khỏi
  sản phẩm Win32 hiện có** trước.

## 3. Ba con đường (đối chiếu chi phí / rủi ro / thời gian)

| # | Con đường | Chi phí | Thời gian | Rủi ro | Ghi chú |
|---|---|---|---|---|---|
| 1 | **Trusted Signing** (Azure, dịch vụ ký của Microsoft) | Free tier mới cho individual; tài khoản Azure + xác thực danh tính | Xác thực có thể mất vài ngày–vài tuần | Thấp — script ký có sẵn (`tools/win/sign-signpath.ps1` là mẫu; thay bằng azuresigntool) | Giữ nguyên luồng EXE hiện tại (0.2.26: dual-DLL, uninstall sạch) — ký xong nộp lại |
| 2 | **MSIX** (Store tự ký) | Miễn phí | Nhanh nhất — **chỉ còn thiếu `Package/Identity/Name`** | Thấp về ký; phải tạo sản phẩm MSIX mới + xoá tên khỏi Win32 product | MSIX 0.2.26 đã sửa xong 2 lỗi kiến trúc (bootstrap stage-out + payload đủ) |
| 3 | Mua chứng thư CA (trong danh sách Trusted Root) | ~$100–400/năm + token | Vài ngày | Trung bình — chọn CA, xác thực doanh nghiệp/cá nhân | Cũngfix SmartScreen ngoài Store |

## 4. Kế hoạch chốt (theo thứ tự ưu tiên)

1. **Ngay lập tức — chuẩn bị MSIX (đường 2)**: vào Partner Center → sản phẩm
   hiện tại → **xoá/reserve lại tên "TextVN" cho MSIX** (làm theo link
   [delete your app name](https://go.microsoft.com/fwlink/?linkid=2189821)) →
   tạo product MSIX → copy **Package/Identity/Name** + **Package/Identity/Publisher**
   → gửi 2 chuỗi đó.
2. **Song song — đăng ký Trusted Signing (đường 1)**: tài khoản Azure →
   Trusted Signing → tạo account + identity validation. Khi được duyệt: ký
   `TextVN-setup-0.2.26-windows-x64-machine.exe` (SHA256, timestamp) → nộp lại
   đúng URL đã có (bump hash → Partner Center revalidate).
3. **Không mua CA riêng** trừ khi Trusted Signing bị từ chối khu vực.

## 5. Hiện trạng kỹ thuật đã sẵn sàng

- **EXE 0.2.26**: dual-DLL (x64 + x86 WOW64), uninstall sạch 100% key, silent
  install chuẩn — **chờ duy nhất chữ ký**.
- **MSIX 0.2.26**: đã sửa 2 lỗi kiến trúc (bootstrap stage-out khỏi WindowsApps +
  payload đầy đủ resources/data); identity hiện là placeholder — build cuối chỉ
  cần `-Publisher "CN=1A703CAB-…" -IdentityName "<Name từ Partner Center>"`.
- Script ký: `tools/win/build-release.ps1 -SigningCertificateThumbprint` (signtool)
  + `tools/win/sign-signpath.ps1` (mẫu tích hợp dịch vụ ký).
- VirusTotal: `tools/win/virustotal-scan.ps1` — sau khi ký, quét lại để xác nhận
  0/false-positive trước khi nộp.

## 6. Việc cần làm — checklist

- [ ] **Chủ repo**: quyết định đường 2 (MSIX) trước vì miễn phí + nhanh; copy
      `Package/Identity/Name` gửi lại.
- [ ] **Chủ repo**: đăng ký Trusted Signing (Azure) cho đường 1 (dự phòng).
- [ ] **Agent**: khi có Identity Name → build MSIX đúng identity → publish
      `approved/v0.2.26/` → hướng dẫn upload trực tiếp.
- [ ] **Agent**: khi có Trusted Signing → tích hợp azuresigntool vào
      `build-release.ps1` (bước ký sau build, trước smoke) → ký cả setup exe
      lẫn MSIX → nộp.
- [ ] Sau khi được duyệt: bật auto-update channel qua Store.
