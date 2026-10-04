# Nộp TextVN lên Microsoft Store — đường MSIX (khi gói .exe bị từ chối)

> Dùng đường này khi gói `.exe` bị Store từ chối ở Package validation (3 mục
> Silent install / ARP / Bundleware). Gói MSIX **bypass toàn bộ 3 check đó** vì
> MSIX có cơ chế cài đặt và identity riêng của Windows — không cần silent
> install parameters, không cần check ARP, không thể có bundleware.

## Tại sao MSIX giải quyết được

| Vấn đề của gói EXE | MSIX giải quyết thế nào |
|---|---|
| Silent install check | MSIX cài đặt ngầm **theo thiết kế** của Windows — không cần switches |
| Entry in add/remove programs | Windows TỰ tạo entry từ manifest (`Identity.Name`) — không phụ thuộc installer |
| Bundleware check | MSIX là gói đóng kín — KHÔNG THỂ cài thêm phần mềm khác |
| Code signing | Store ký lại khi publish — không cần cert |
| UAC / elevation | MSIX cài per-user mặc định, không cần UAC |

## Bước 1 — Tạo product mới dạng MSIX trong Partner Center

1. Đăng nhập [Partner Center](https://partner.microsoft.com/dashboard).
2. **Apps and Games** → **Overview** → **New product**.
3. Chọn loại: **MSIX** (hoặc "Microsoft Store app" tuỳ giao diện).
4. **Reserve app name**: đặt `TextVN - Bộ gõ tiếng Việt`.
5. Sau khi tạo, vào trang sản phẩm → **Product identity** (hoặc App identity)
   → ghi lại 2 giá trị:

   | Trường | Ví dụ | Copy từ |
   |---|---|---|
   | **Package/Identity/Name** | `12345LinhBHCoM.TextVN` | Partner Center → Product identity |
   | **Package/Identity/Publisher** | `CN=E5F3...ABCD` | Partner Center → Product identity |

   ⚠️ **2 giá trị này BẮT BUỘC khớp** trong manifest của file .msix — nếu
   không, Store sẽ từ chối ngay ("package identity mismatch").

## Bước 2 — Build MSIX với đúng Identity

Cách 1 — **Tôi build giúp** (nhanh nhất): gửi 2 giá trị ở trên cho tôi, tôi
chạy 1 lệnh và đưa file .msix đúng lên branch `approved`.

Cách 2 — **Tự build**:

```powershell
cd D:\AppAI\TextVN
powershell -NoProfile -ExecutionPolicy Bypass -File tools\win\build-msix.ps1 `
  -Publisher "CN=E5F3...ABCD" `
  -IdentityName "12345LinhBHCoM.TextVN"
```

File ra: `dist\TextVN-<version>-windows-x64.msix` (bản hiện hành: `TextVN-0.2.20-windows-x64.msix`).
⚠️ File MSIX trong branch `approved` (`v0.2.20/TextVN-0.2.20-windows-x64.msix`) hiện
build với **Identity placeholder** `LinhBH.CoM.TextVN` / `CN=LinhBH.CoM` — phải
build lại bằng **đúng 2 giá trị Product identity** của Partner Center trước khi upload.

## Bước 3 — Nộp file .msix vào submission

1. Trong submission, vào **Packages**.
2. Bấm **Upload package** → chọn file `.msix` vừa build.
   (KHÔNG dùng "Package URL" như lúc nộp .exe — MSIX upload trực tiếp.)
3. Đợi Store validate file (vài giây).
4. Điền phần còn lại của submission: Description, Screenshots, Privacy policy
   URL, v.v. (xem `store/art/` cho ảnh listing).

## Bước 4 — Certification

Store chạy certification tự động:
- **Malware check**: quét file trong gói — TextVN sạch (đã kiểm qua VT).
- **Manifest check**: Identity khớp Partner Center → pass.
- **Capability check**: `runFullTrust` cần giải trình — ghi trong
  Description: *"This is a Vietnamese input method (TSF) that requires full
  trust to integrate with the Windows text services framework. It processes
  all input locally and does not collect any data."*
- **Compatibility check**: Windows 10/11 → pass.

Thông thường 1-3 ngày. Sau khi pass → Publish.

## Lưu ý quan trọng

| Vấn đề | Giải đáp |
|---|---|
| Gói MSIX chưa ký | Nộp Store KHÔNG cần ký — Store ký lại khi publish. |
| Identity sai | Store từ chối ngay "identity mismatch" → build lại đúng 2 giá trị ở Bước 1. |
| Người dùng cài xong không gõ được | Mở TextVN một lần → app tự đăng ký TSF (tray tự đề nghị UAC nếu cần — 0.2.16+). |
| Có cần dùng song song với gói .exe? | Không — chọn MỘT đường. Nếu MSIX pass thì bỏ gói .exe. |
| Sản phẩm hiện tại "EXE or MSI app" có chuyển sang MSIX được không? | Tạo **product mới** loại MSIX (giữ nguyên tên nếu tên cũ chưa publish — nếu đã reserve thì dùng tên khác hoặc xoá sản phẩm cũ). |
