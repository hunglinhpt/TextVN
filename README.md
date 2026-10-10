# approved — bản phát hành đã được duyệt (nguồn nộp Microsoft Store)

Branch này KHÔNG chứa mã nguồn. Nó chỉ chứa **binary của các phiên bản đã được
chủ repo duyệt**, đặt sẵn để lấy **raw content URL** (không qua redirect của
trang Releases) khi khai báo gói cài trên Microsoft Partner Center.

## Cấu trúc

```
vX.Y.Z/
  TextVN-setup-X.Y.Z-windows-x64-machine.exe <- bộ cài NỘP STORE (đường EXE, ARP ở HKLM)
  TextVN-setup-X.Y.Z-windows-x64.exe         <- bộ cài per-user (đối chiếu)
  TextVN-portable-X.Y.Z-windows-x64-*.zip    <- bản chạy ngay (đối chiếu)
  *.asc / *.cosign.sig / *.cosign.cert       <- chữ ký GPG + Sigstore của từng file
  SHA256SUMS.txt                             <- checksum của release gốc (clearsign)
  gpg-release-key.asc                        <- khoá công khai để kiểm chữ ký
```

Đường EXE chỉ còn chờ chữ ký Authenticode (SignPath Foundation) — chính sách 10.2.9
chặn EXE chưa ký.

**MSIX nộp Store** (sản phẩm MSIX trên Partner Center, Store ID `9NV2R7JNNFGK`, identity
`23651Linhi.TextVN`, PFN `23651Linhi.TextVN_qgnrpq341n182`): upload FILE trên Partner
Center › Packages (không cần raw URL). File nộp đặt tại đây cho tiện tải:

```
vX.Y.Z/
  TextVN-X.Y.Z-windows-x64-store.msix          <- GÓI NỘP STORE (identity thật)
  TextVN-X.Y.Z-windows-x64-store.msix.sha256
```

- **v0.2.28** — ngoại lệ DUY NHẤT của quy tắc byte-identical: gói `.msix` kèm GitHub
  Release v0.2.28 mang identity TẠM (build trước khi có identity Partner Center) nên
  KHÔNG chép; `TextVN-0.2.28-windows-x64-store.msix` được build lại từ ĐÚNG tag
  `v0.2.28` với identity thật ([run CI 38038814173](https://github.com/hunglinhpt/TextVN/actions/runs/38038814173),
  artifact `release-windows`; `verify-msix.py --require-store-identity` + sideload
  PASS), Version `1.2.28.0`. Hash ở file `.sha256` cạnh nó — KHÔNG có trong
  `SHA256SUMS.txt` (checksum của release gốc).
- **Từ bản sau v0.2.28**: repo variable `MSIX_IDENTITY_NAME` đã đặt → `.msix` kèm
  release mang identity thật và có chữ ký GPG/Sigstore như mọi asset; chép nó vào
  đây (đổi tên thêm `-store`, kèm `.asc`/`.cosign.*`) như các file khác.

Các thư mục ≤ v0.2.27 giữ nguyên như đã duyệt lúc đó (gói `.msix` trong đó mang identity
tạm — không nộp). Chi tiết: `docs/release/msix-submission.md` trong nhánh `main`.

```
store-art/
  box-art-2160.png (1080.png)      <- ảnh listing 1:1 BẮT BUỘC (nền cờ VN + sao vàng + chữ V)
  poster-art-1440x2160.png (720x1080.png) <- ảnh listing 2:3 khuyến nghị
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
2. `gh release download vX.Y.Z` cho setup exe, `-machine.exe`, portable zip (kèm
   `.asc`/`.cosign.sig`/`.cosign.cert` của từng file), `SHA256SUMS.txt` và
   `gpg-release-key.asc` vào thư mục `vX.Y.Z/` trên branch này; kiểm hash với
   `SHA256SUMS.txt` và `gpg --verify` từng `.asc` trước khi commit.
3. Commit + push vào `approved`.
