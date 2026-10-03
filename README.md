# approved — bản phát hành đã được duyệt (nguồn nộp Microsoft Store)

Branch này KHÔNG chứa mã nguồn. Nó chỉ chứa **binary của các phiên bản đã được
chủ repo duyệt**, đặt sẵn để lấy **raw content URL** (không qua redirect của
trang Releases) khi khai báo gói cài trên Microsoft Partner Center.

## Cấu trúc

```
vX.Y.Z/
  TextVN-setup-vX.Y.Z-windows-x64.exe      <- bộ cài nộp Store (Inno Setup)
  TextVN-portable-vX.Y.Z-windows-x64-*.zip <- bản chạy ngay (đối chiếu)
  TextVN-vX.Y.Z-windows-x64.msix           <- DỰ PHÒNG khi gói exe bị Store từ chối
  SHA256SUMS.txt                            <- checksum của release gốc
```

Gói `.msix` là bản full-trust (`runFullTrust`): cài xong mở TextVN một lần để
tự đăng ký bộ gõ (như portable). Nộp Store **không cần ký** (Store ký lại);
nhưng **Publisher/Identity trong manifest phải khớp Partner Center** — build
lại bằng `tools/win/build-msix.ps1 -Publisher ... -IdentityName ...` với giá
trị trong *Product identity* (xem `docs/release/store-submission.md` §2c).

Binary tại đây **byte-identical** với asset của GitHub Release cùng tag
(tải trực tiếp từ release, không build lại).

## URL thô để nộp Store (không redirect)

Thay `<ver>` bằng phiên bản, ví dụ `v0.2.16`:

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/<ver>/TextVN-setup-<ver-without-v>-windows-x64.exe
```

Ví dụ v0.2.16:

```
https://raw.githubusercontent.com/hunglinhpt/TextVN/approved/v0.2.16/TextVN-setup-0.2.16-windows-x64.exe
```

- `raw.githubusercontent.com` phục vụ file trực tiếp (HTTP 200), không dùng
  redirect như `.../releases/download/...` — phù hợp trường URL trong Partner
  Center.
- Tham số cài đặt cho ô "Installer parameters" của Store:
  `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /CURRENTUSER`
  (xem `docs/release/store-submission.md` trong nhánh `main`).

## Xác minh checksum

```
sha256sum TextVN-setup-0.2.16-windows-x64.exe
# hoặc trên Windows:
certutil -hashfile TextVN-setup-0.2.16-windows-x64.exe SHA256
```

Đối chiếu dòng tương ứng trong `vX.Y.Z/SHA256SUMS.txt`.

## Quy trình cập nhật (thủ công — chỉ bản chủ repo DUYỆT)

1. Chủ repo xác nhận bản `vX.Y.Z` đã duyệt (CI xanh, E2E gõ thật PASS).
2. `gh release download vX.Y.Z` cho setup exe + portable zip + SHA256SUMS.txt
   vào thư mục `vX.Y.Z/` trên branch này.
3. Commit + push vào `approved`.
