# Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store

> Trạng thái (cập nhật 2026-10-09): GPG + Sigstore **đã triển khai tự động** cho release 3
> platform (vòng 15) và **bắt buộc** trong CI (R2-74). Còn thiếu: Authenticode cho Windows
> (chờ SignPath Foundation duyệt — §4) và Developer ID/notarization cho macOS
> (`signing-status-mac.md`).
> Liên quan: `store-policy-10-2-9.md` (yêu cầu Authenticode của Microsoft Store),
> `msix-submission.md`, `code-signing-plan.md`, `tools/release/sign-artifacts.sh`.

## 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào

| Công nghệ | Chứng minh được gì | Tự động trên GitHub Actions | Đáp ứng Store 10.2.9? | Quyết định |
|---|---|---|---|---|
| **GPG/PGP** (detached signature) | Artifact do chủ khoá `3921595A…` ký, không bị sửa sau khi ký | ✓ — private key trong GH secret `RELEASE_GPG_PRIVATE_KEY` | ✗ (không phải Authenticode) | **DÙNG** — lớp 1, chuẩn cộng đồng Linux/Foss |
| **Sigstore (cosign keyless)** | Artifact được build bởi workflow `release.yml` của repo này (chứng thư OIDC ngắn hạn, provenance kiểu SLSA) | ✓ — không cần secret, dùng OIDC token của workflow | ✗ | **DÙNG** — lớp 2, provenance tự động |
| **OpenSSL** (`dgst -sign`) | Tương tự GPG nhưng tự chế định dạng, không có web-of-trust hay hạ tầng verify | ✓ | ✗ | **KHÔNG dùng cho artifact** — dư thừa khi đã có GPG; chỉ còn ý nghĩa cho cert sideload local (Windows dùng `New-SelfSignedCertificate` sẵn) |
| **Authenticode / Trusted Signing / SignPath** | Đơn vị ký đã được xác thực (chained to Microsoft Trusted Root) | ✓ (SignPath REST API — `tools/win/sign-signpath.ps1`) | ✓ **BẮT BUỘC** cho gói EXE | **Đường cho Store EXE + SmartScreen** — chờ SignPath Foundation (chương trình OSS) duyệt; Trusted Signing không khả dụng cho cá nhân ngoài Mỹ/Canada (`code-signing-plan.md` §1) |
| **MSIX không ký** | Store **tự ký miễn phí** khi publish | ✓ (upload .msix unsigned) | ✓ (bypass 10.2.9) | **ĐƯỜNG NỘP STORE CHÍNH** — chỉ còn thiếu `Package/Identity/Name` từ Partner Center (`msix-submission.md`) |

Kết luận: GPG + Sigstore là **lớp integrity/provenance miễn phí, tự động, đầy đủ
3 platform**; 10.2.9 của Store chỉ chấp nhận Authenticode (Trusted Signing /
SignPath / CA) hoặc **MSIX do Store tự ký**.

## 2. Vòng 15 — những gì đã triển khai trong `release.yml`

- Publish job thêm `id-token: write` + bước `sigstore/cosign-installer@v3`.
- Sau khi tạo `SHA256SUMS.txt`, gọi `tools/release/sign-artifacts.sh`:
  1. **GPG**: import key từ secret `RELEASE_GPG_PRIVATE_KEY` → `gpg --detach-sign
     --armor` cho **mỗi asset** (exe/zip/msix/tar.gz/pkg) → **`*.asc`** (armored
     detached); clearsign `SHA256SUMS.txt` (nội dung giữ nguyên — dùng
     `gpg --decrypt` rồi `sha256sum -c`).
  2. **Sigstore keyless**: `cosign sign-blob` từng asset → `*.cosign.sig` +
     `*.cosign.cert` (chứng thư gắn identity workflow).
- Public key commit tại `docs/release/signing/gpg-release-key.asc` và được
  upload kèm mỗi release.
- **Fail-closed (R2-74, 2026-10-08)**: trong GitHub Actions việc ký là bắt buộc —
  secret `RELEASE_GPG_PRIVATE_KEY` trống, import lỗi, khoá không đúng FPR phát hành,
  hoặc thiếu `cosign` đều làm job publish **dừng** thay vì phát hành bản không chữ ký.
  Script ký bằng đúng khoá (`--local-user 3921595A…`) và `gpg --verify` từng `.asc`
  ngay sau khi tạo. Chạy tay ngoài CI vẫn cho bỏ qua từng lớp.

## 3. Khoá ký GPG release (TextVN Release Signing)

- **Fingerprint**: `3921595ABC961199F15303B6C45B84D0C7F4A822`
- **User ID**: `TextVN Release Signing <hunglinhpt@users.noreply.github.com>`
- **Thuật toán**: Ed25519 (sign-only, không hết hạn)
- **Public key**: `docs/release/signing/gpg-release-key.asc`
- **Private key**: GH secret `RELEASE_GPG_PRIVATE_KEY` — **BẮT BUỘC là bản
  `--armor` (ASCII)**: `gpg --batch --armor --export-secret-keys <FPR>`. Ví
  dụ 2026-10-07 (R5): export thiếu `--armor` → 298 byte nhị phân chứa 6 byte
  NUL → runner không materialize được env → **workflow `startup_failure` không
  log**; kiểm bằng `python -c "b=open(f,'rb').read(); print(b.count(b'\\x00'))"`
  (phải = 0) trước khi `gh secret set`. Không passphrase (bảo vệ bởi encryption
  của GitHub Actions secrets).
- **Chứng thư thu hồi**: do lệnh sinh key in ra tại
  `~/.gnupg/openpgp-revocs.d/3921595A….rev` — chủ repo LƯU Ý giữ file này ở
  nơi an toàn (dùng để thu hồi khi key lộ).

### Verify một release (dùng GPG)

```bash
gpg --import docs/release/signing/gpg-release-key.asc
gpg --verify TextVN-setup-0.2.27-windows-x64-machine.exe.asc \
            TextVN-setup-0.2.27-windows-x64-machine.exe
gpg --verify SHA256SUMS.txt            # clearsigned
sha256sum -c --ignore-missing <(gpg --decrypt SHA256SUMS.txt 2>/dev/null)   # chỉ kiểm các file đã tải
```

### Verify provenance Sigstore (dùng cosign)

```bash
cosign verify-blob \
  --signature TextVN-setup-0.2.27-windows-x64-machine.exe.cosign.sig \
  --certificate TextVN-setup-0.2.27-windows-x64-machine.exe.cosign.cert \
  --certificate-identity-regexp \
    '^https://github.com/hunglinhpt/TextVN/\.github/workflows/release\.yml@refs/tags/v.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  TextVN-setup-0.2.27-windows-x64-machine.exe
```

## 4. Authenticode cho Microsoft Store (chưa xong — đang chờ)

- EXE: chờ **SignPath Foundation** duyệt. Pipeline đã nối sẵn: job `windows` của
  `release.yml` truyền secret `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`,
  `SIGNPATH_PROJECT_SLUG` (tên cũ `SIGNPATH_PROJECT_KEY` vẫn nhận), `SIGNPATH_POLICY`,
  `SIGNPATH_ARTIFACT_CONFIGURATION` → `build-release.ps1` gọi
  `tools/win/sign-signpath.ps1` (REST API công bố của SignPath: `POST
  {org}/SigningRequests` multipart, **một** yêu cầu ZIP deep-sign cho cả 4 PE, mỗi bộ cài
  một yêu cầu riêng; poll tới `Completed` kể cả khi chờ duyệt tay `WaitingForApproval`;
  kiểm `Get-AuthenticodeSignature` = `Valid` sau khi ghi đè). Có secret thì cả
  `TextVN-setup-<ver>-windows-x64.exe` lẫn `-machine.exe` (file nộp Store) đều được ký;
  chưa có secret thì bản phát hành vẫn là release candidate chưa Authenticode.
  Chi tiết + checklist bật: `code-signing-plan.md`.
- MSIX: **không cần ký** — Store tự ký; build cuối chỉ cần `Package/Identity/Name` thật
  từ Partner Center (repo variable `MSIX_IDENTITY_NAME` → `build-msix.ps1
  -RequireStoreIdentity`) — xem `msix-submission.md` §3.
- Lưu ý MS: dùng **cùng tên** "TextVN" cho sản phẩm MSIX đòi hỏi xoá tên khỏi sản phẩm
  Win32 (EXE) hiện có — chủ repo quyết trên Partner Center (`msix-submission.md` §2).

## 4b. macOS — Developer ID + notarization

Chưa có: bản phát hành ký ad-hoc, `.pkg` chưa ký. Script đã sẵn và chỉ chạy khi có secret
Apple — trạng thái, biến môi trường và việc còn lại: `signing-status-mac.md`.

## 5. Quy tắc khóa-tay (đừng lặp)

- Private key KHÔNG commit vào repo — chỉ qua GH secret; file export local phải
  xoá ngay sau `gh secret set` (đã làm).
- Mọi asset trong release phải có cặp chữ ký trước khi `gh release upload` —
  script ký chạy TRƯỚC upload trong publish job.
- `SHA256SUMS.txt` phải là bản clearsign (nội dung giữ nguyên để
  `sha256sum -c` vẫn chạy được sau khi giải chữ ký).
