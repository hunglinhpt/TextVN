# Ký số TextVN — Sigstore, GPG/PGP, OpenSSL và đường lên Microsoft Store

> Trạng thái: **đã triển khai tự động** cho release 3 platform (vòng 15).
> Liên quan: `store-policy-10-2-9.md` (yêu cầu Authenticode của Microsoft Store),
> `msix-submission.md`, `tools/release/sign-artifacts.sh`.

## 1. Đánh giá ba công nghệ — cái nào giải quyết vấn đề nào

| Công nghệ | Chứng minh được gì | Tự động trên GitHub Actions | Đáp ứng Store 10.2.9? | Quyết định |
|---|---|---|---|---|
| **GPG/PGP** (detached signature) | Artifact do chủ khoá `3921595A…` ký, không bị sửa sau khi ký | ✓ — private key trong GH secret `RELEASE_GPG_PRIVATE_KEY` | ✗ (không phải Authenticode) | **DÙNG** — lớp 1, chuẩn cộng đồng Linux/Foss |
| **Sigstore (cosign keyless)** | Artifact được build bởi workflow `release-candidate.yml` của repo này (chứng thư OIDC ngắn hạn, provenance kiểu SLSA) | ✓ — không cần secret, dùng OIDC token của workflow | ✗ | **DÙNG** — lớp 2, provenance tự động |
| **OpenSSL** (`dgst -sign`) | Tương tự GPG nhưng tự chế định dạng, không có web-of-trust hay hạ tầng verify | ✓ | ✗ | **KHÔNG dùng cho artifact** — dư thừa khi đã có GPG; chỉ còn ý nghĩa cho cert sideload local (Windows dùng `New-SelfSignedCertificate` sẵn) |
| **Authenticode / Trusted Signing / SignPath** | Đơn vị ký đã được xác thực (chained to Microsoft Trusted Root) | ✓ (SignPath/Trusted Signing qua API) | ✓ **BẮT BUỘC** | **ĐƯỜNG CHÍNH cho Store EXE** — chờ SignPath Foundation / Trusted Signing duyệt (xem `store-policy-10-2-9.md`) |
| **MSIX không ký** | Store **tự ký miễn phí** khi publish | ✓ (upload .msix unsigned) | ✓ (bypass 10.2.9) | **ĐƯỜNG CHÍNH cho Store MSIX** — chỉ còn thiếu Identity từ Partner Center |

Kết luận: GPG + Sigstore là **lớp integrity/provenance miễn phí, tự động, đầy đủ
3 platform**; 10.2.9 của Store chỉ chấp nhận Authenticode (Trusted Signing /
SignPath / CA) hoặc **MSIX do Store tự ký**.

## 2. Vòng 15 — những gì đã triển khai trong `release-candidate.yml`

- Publish job thêm `id-token: write` + bước `sigstore/cosign-installer@v3`.
- Sau khi tạo `SHA256SUMS.txt`, gọi `tools/release/sign-artifacts.sh`:
  1. **GPG**: import key từ secret `RELEASE_GPG_PRIVATE_KEY` → `gpg --detach-sign
     --armor` cho **mỗi asset** (exe/zip/msix/tar.gz/pkg) → `*.sig`; clearsign
     `SHA256SUMS.txt` (nội dung giữ nguyên, có chữ ký bao quanh).
  2. **Sigstore keyless**: `cosign sign-blob` từng asset → `*.cosign.sig` +
     `*.cosign.cert` (chứng thư gắn identity workflow).
- Public key commit tại `docs/release/signing/gpg-release-key.asc` và được
  upload kèm mỗi release.

## 3. Khoá ký GPG release (TextVN Release Signing)

- **Fingerprint**: `3921595ABC961199F15303B6C45B84D0C7F4A822`
- **User ID**: `TextVN Release Signing <hunglinhpt@users.noreply.github.com>`
- **Thuật toán**: Ed25519 (sign-only, không hết hạn)
- **Public key**: `docs/release/signing/gpg-release-key.asc`
- **Private key**: GH secret `RELEASE_GPG_PRIVATE_KEY` (armored, không
  passphrase — bảo vệ bởi encryption của GitHub Actions secrets)
- **Chứng thư thu hồi**: do lệnh sinh key in ra tại
  `~/.gnupg/openpgp-revocs.d/3921595A….rev` — chủ repo LƯU Ý giữ file này ở
  nơi an toàn (dùng để thu hồi khi key lộ).

### Verify một release (dùng GPG)

```bash
gpg --import docs/release/signing/gpg-release-key.asc
gpg --verify TextVN-setup-0.2.26-windows-x64-machine.exe.sig \
            TextVN-setup-0.2.26-windows-x64-machine.exe
gpg --verify SHA256SUMS.txt            # clearsigned
sha256sum -c <(gpg --decrypt SHA256SUMS.txt)   # kiểm toàn bộ checksum
```

### Verify provenance Sigstore (dùng cosign)

```bash
cosign verify-blob \
  --signature TextVN-setup-0.2.26-windows-x64-machine.exe.cosign.sig \
  --certificate TextVN-setup-0.2.26-windows-x64-machine.exe.cosign.cert \
  --certificate-identity-regexp \
    '^https://github.com/hunglinhpt/TextVN/\.github/workflows/release-candidate\.yml@refs/tags/v.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  TextVN-setup-0.2.26-windows-x64-machine.exe
```

## 4. Authenticode cho Microsoft Store (chưa xong — đang chờ)

- EXE: chờ **SignPath Foundation** (OSS program) hoặc **Trusted Signing**.
  Khi có: ký `TextVN-setup-<ver>-…-machine.exe` bằng SHA256 + timestamp
  (`signtool sign /fd SHA256 /tr …` hoặc azuresigntool) rồi nộp lại — xem
  `store-policy-10-2-9.md` mục 4.
- MSIX: **không cần ký** — Store tự ký; build cuối chỉ cần Identity thật từ
  Partner Center (`build-msix.ps1 -Publisher "CN=…" -IdentityName "…"`).
- Lưu ý MS: dùng tên "TextVN" cho sản phẩm MSIX đòi hỏi xoá tên khỏi sản phẩm
  Win32 hiện có — chủ repo thực hiện trên Partner Center.

## 5. Quy tắc khóa-tay (đừng lặp)

- Private key KHÔNG commit vào repo — chỉ qua GH secret; file export local phải
  xoá ngay sau `gh secret set` (đã làm).
- Mọi asset trong release phải có cặp chữ ký trước khi `gh release upload` —
  script ký chạy TRƯỚC upload trong publish job.
- `SHA256SUMS.txt` phải là bản clearsign (nội dung giữ nguyên để
  `sha256sum -c` vẫn chạy được sau khi giải chữ ký).
