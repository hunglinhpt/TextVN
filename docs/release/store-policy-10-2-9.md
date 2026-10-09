# Microsoft Store Policy 10.2.9 — Ký số bắt buộc cho gói EXE/MSI

> Trạng thái: **Microsoft đã trả lời chính thức** (2026-10-06) sau khi nộp
> `approved/v0.2.24/TextVN-setup-0.2.24-windows-x64-machine.exe`. Tài liệu này
> ghi nhận yêu cầu, đối chiếu với hiện trạng, và chốt kế hoạch hành động.
> Liên quan: `store-submission.md`, `msix-submission.md`, `win-test-common-errors.md` (B13/B19).
>
> **Cập nhật 2026-10-09:** §3–§6 đã đổi theo hiện trạng — đường ký EXE là **SignPath
> Foundation** (thay Trusted Signing: không khả dụng cho cá nhân ngoài Mỹ/Canada) và đã nối
> vào pipeline, chờ Foundation duyệt; đường **MSIX là đường nộp chính**, chỉ còn thiếu
> `Package/Identity/Name` (`msix-submission.md`). §1–§2 giữ nguyên làm hồ sơ.

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
| 1 | **SignPath Foundation** (chương trình ký miễn phí cho OSS) | Miễn phí | Chờ Foundation duyệt hồ sơ (đã nộp) | Thấp — đã nối: `release.yml` truyền secret `SIGNPATH_*` → `build-release.ps1` → `tools/win/sign-signpath.ps1`; ký 4 PE + cả hai bộ cài (kể cả `-machine.exe`) | Giữ nguyên luồng EXE — ký xong phát hành bản mới rồi nộp lại. Trusted Signing (Azure) không khả dụng cho cá nhân ngoài Mỹ/Canada — xem `code-signing-plan.md` §1 |
| 2 | **MSIX** (Store tự ký) — **đường chính** | Miễn phí | Nhanh nhất — còn thiếu `Package/Identity/Name` (gói `.msix` của 0.2.27 không nộp được, phải build lại — `msix-submission.md` §0) | Thấp về ký; phải tạo sản phẩm MSIX mới (dùng lại tên `TextVN` thì xoá tên khỏi Win32 product trước) | Stage-out ra ngoài gói + guard dọn sau khi gỡ, kiểm bằng sideload thật trong CI |
| 3 | Mua chứng thư CA (trong danh sách Trusted Root) | ~$100–400/năm + token | Vài ngày | Trung bình — chọn CA, xác thực doanh nghiệp/cá nhân | Cũng fix SmartScreen ngoài Store |

## 4. Kế hoạch chốt (theo thứ tự ưu tiên — cập nhật 2026-10-09)

1. **MSIX (đường 2) — tooling xong, chờ chủ tài khoản**: chọn cách đặt tên sản phẩm
   MSIX (A: xoá tên `TextVN` khỏi sản phẩm Win32 theo link
   [delete your app name](https://go.microsoft.com/fwlink/?linkid=2189821) rồi reserve lại;
   B: sản phẩm MSIX tên khác) → copy **Package/Identity/Name** vào repo variable
   `MSIX_IDENTITY_NAME` → workflow build gói nộp được → upload trực tiếp. Chi tiết:
   `msix-submission.md` §2–§4. Windows publisher ID đã có:
   `CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545`.
2. ⏳ **Authenticode — SignPath Foundation (đường 1)**: đã nộp, chờ duyệt (thay kế hoạch
   Trusted Signing ngày 2026-10-06). Khi duyệt: đặt secret `SIGNPATH_*`
   (`code-signing-plan.md`) → phát hành bản mới (CI ký 4 PE + cả hai bộ cài) → nộp lại URL
   `approved/vX.Y.Z/TextVN-setup-X.Y.Z-windows-x64-machine.exe` (hash mới → Partner Center
   revalidate).
3. **Không mua CA riêng** trừ khi SignPath Foundation từ chối.

## 5. Hiện trạng kỹ thuật đã sẵn sàng (2026-10-09)

- **EXE**: dual-DLL (x64 + x86 WOW64), CRT tĩnh (không cần VC++ redist), uninstall sạch,
  silent install chuẩn (chỉ chép file, exit 0) — **chờ duy nhất chữ ký Authenticode**;
  khi có secret SignPath, `-machine.exe` (file nộp) cũng được ký. `unins000.exe` vẫn chưa
  ký (giới hạn đã biết).
- **MSIX**: exe trong gói chỉ chép payload ra `%USERPROFILE%\.textvn\msix-staging` →
  `%LOCALAPPDATA%\Programs\TextVN-Store\<V>`, đăng ký HKCU, guard ở mỗi lần đăng nhập
  dọn sạch sau khi gói bị gỡ — CI cài thử thật (`installer/windows/tests/test-msix-sideload.ps1`).
  Manifest kiểm bằng `tools/win/verify-msix.py`; Version map `A.B.C` → `(A+1).B.C.0`
  (0.2.27 → 1.2.27.0). Gói trên Release là placeholder cho tới khi có
  `MSIX_IDENTITY_NAME` (`msix-submission.md` §3).
- Script ký: `build-release.ps1` (gốc repo) — `-SigningCertificateThumbprint` (signtool,
  cert local) hoặc tự gọi `tools/win/sign-signpath.ps1` khi có `SIGNPATH_API_TOKEN`.
- VirusTotal: `tools/win/virustotal-scan.ps1` — `release.yml` quét (best-effort) cả hai
  bộ cài, ZIP portable và `.msix` sau khi build; sau khi có chữ ký, quét lại trước khi nộp.

## 6. Việc cần làm — checklist

- [ ] **Chủ repo**: chọn cách đặt tên sản phẩm MSIX (`msix-submission.md` §2), đặt repo
      variable `MSIX_IDENTITY_NAME` (và `MSIX_DISPLAY_NAME` nếu chọn tên khác).
- [x] **Chủ repo**: nộp hồ sơ SignPath Foundation — ⏳ chờ duyệt.
- [ ] **Chủ repo/Agent**: khi có Identity Name → chạy workflow build gói → đưa `.msix` có
      Identity thật lên `approved/vX.Y.Z/` → upload trực tiếp (Store tự ký MSIX — không cần
      ký thêm).
- [x] **Agent**: tích hợp SignPath vào `build-release.ps1`/`release.yml` (REST API, ký cả
      `-machine.exe`) — xong 2026-10-08.
- [ ] **Chủ repo**: khi SignPath duyệt → đặt secret `SIGNPATH_*` → phát hành bản mới → nộp
      lại URL bản máy.
- [ ] Sau khi được duyệt: bật auto-update channel qua Store.
