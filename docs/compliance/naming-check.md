# WIN-008 — Báo cáo kiểm tra tên và không gian tên "VietIME"

> **Mã task:** `WIN-008` (P1-6-TASKS.md §M0)  
> **Phạm vi:** Kiểm tra va chạm tên (name collision), thương hiệu (trademark), không gian tên gói (crates.io, winget, scoop, choco) và đề xuất phương án bảo hộ/phát hành cho dự án VietIME.  
> **Ngày thực hiện:** 2026-09-27  
> **Trạng thái:** ✅ ĐẠT (0 xung đột chặn phát hành, có phương án dự phòng)

---

## 1. Mục tiêu và phạm vi kiểm tra

Theo `docs/10-shared/P0-REVIEW-LOG.md` (Finding F0-001/carry-over) và `PLAN.md §0`:
- Tên công khai dự kiến: **VietIME**
- Tên nhị phân/package/crate: `vietime`, `vietime-core`, `vietime-cli`, `vietime-tsf`, `vietime-hook`, `vietime-tray`
- Cần xác minh:
  1. Kho mã nguồn & tổ chức GitHub: `github.com/<org>/vietime`
  2. Registry Rust (crates.io)
  3. Quản lý gói Windows: WinGet (Microsoft Community Package Repository), Scoop, Chocolatey
  4. Quản lý gói macOS (Homebrew tap/cask) và Linux (APT PPA, Flathub, AUR)
  5. Đăng ký nhãn hiệu (Trademark / IP) tại Việt Nam và quốc tế
  6. Rủi ro mạo danh (typosquatting) và xung đột thương hiệu đối thủ

---

## 2. Kết quả kiểm tra chi tiết theo kênh

| Kênh | Trạng thái hiện tại | Rủi ro | Giải pháp / Hành động |
|---|---|---|---|
| **GitHub repo / org** | Tên repo `TextVN` hiện tại đang làm việc, repo đích công khai là `vietime` hoặc tổ chức `vietime/vietime`. Chưa có dự án lớn nào chiếm độc quyền tổ chức `vietime` với sản phẩm IME tương đương. | Thấp | Khi phát hành public v1.0, tổ chức GitHub chính thức sẽ dùng `vietime-org` hoặc `vietime-dev` nếu `vietime` bị chiếm chỗ; repo chính: `vietime`. |
| **Crates.io** | Kiểm tra namespace `vietime-*` (`vietime-core`, `vietime-ffi`, `vietime-cli`...). Dự án thiết kế crate nội bộ với `publish = false` ở hầu hết các crate trừ SDK FFI công khai. | Không có | Các crate nội bộ (`vietime-core`, `vietime-config`, `vietime-appdb`...) giữ `publish = false`. Nếu cần phát hành crate FFI/wrapper trong tương lai, giữ namespace `vietime` hoặc `vietime-sys`. |
| **Windows WinGet** | Package ID chuẩn: `VietIME.VietIME`. Kiểm tra `microsoft/winget-pkgs`: chưa có package nào mang ID này. | Không có | Chuẩn bị manifest `VietIME.VietIME.yaml` trong M5 (WIN-062). |
| **Scoop** | Bucket `extras`: chưa có manifest `vietime.json`. | Không có | Tạo manifest cho Scoop bucket tự host hoặc PR vào `extras`. |
| **Chocolatey** | Package ID `vietime`: chưa có package nào được duyệt. | Không có | Có thể đăng ký `vietime` khi hoàn thiện bản installer. |
| **macOS Homebrew** | Cask `vietime` (cho `/Applications/VietIME.app`) và formula `vietime` (CLI). Tap riêng: `homebrew-tap/vietime.rb`. | Không có | Khớp với quy hoạch trong Phần 2 (`30-macos/P2-0 §3`). |
| **Linux (Flathub, AUR)** | `vn.vietime.VietIME` (DBus bus name & AppStream ID), AUR package `vietime-bin` / `vietime-git`. | Không có | Khớp với quy hoạch trong Phần 3 (`40-linux/P3-0 §2`). |
| **Nhãn hiệu (Trademark)** | Tra cứu nhãn hiệu tại Cục Sở hữu Trí tuệ Việt Nam (WIPO IP Vietnam) và USPTO: "VietIME" là sự kết hợp mang tính miêu tả chức năng ("Viet" + "IME" - Input Method Editor), không trùng lặp với nhãn hiệu đã đăng ký độc quyền cho nhóm 09 (phần mềm máy tính có thể tải về) cản trở việc sử dụng mở. | Thấp | Được phép sử dụng tự do theo giấy phép GPL-3.0-or-later; ghi rõ disclaimer không liên quan đến bên thứ ba. |

---

## 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm

Để tránh nhầm lẫn cho người dùng và tránh bị các trình diệt virus nhận nhầm (false positive do trùng tên):

1. **Không dùng các tiền tố gây nhầm lẫn:**
   - Không dùng `UniKey*` (thuộc bản quyền tác giả Phạm Kim Long).
   - Không dùng `EVKey*` (thuộc tác giả Lâm Quang Minh).
   - Không dùng `OpenKey*` (thuộc tác giả Mai Vũ Tuyên).
   - Không dùng `GoTiengViet*` (thuộc tác giả Trần Kỳ Nam).
2. **Nhận diện duy nhất của VietIME:**
   - GUID TSF chính thức: `CLSID_VIETIME_TIP` = `{6B7E1F80-4A2D-4E93-9C55-1F0A7D2E9C11}`
   - Service Name Windows: `VietIME`
   - IPC Named Pipe: `\\.\pipe\vietime-ipc-v1`
   - Config directory: `%APPDATA%\VietIME`

---

## 4. Quyết định & Kết luận

1. **Tên chính thức:** Duy trì tên **VietIME** cho sản phẩm, nhị phân CLI `vietime.exe` / `vietime`, và nhị phân adapter `vietime-tsf.dll`, `vietime-hook.exe`, `vietime-tray.exe`.
2. **Tên dự phòng (Fallback):** Trong trường hợp bất khả kháng khi phân phối quốc tế (xung đột domain), tên dự phòng đã được duyệt trong ADR là **OpenVietIME** hoặc **VNIME**.
3. **Tiêu chí nghiệm thu WIN-008:**
   - [x] Đã hoàn thành bảng kiểm tra va chạm namespace trên 7 nền tảng phân phối.
   - [x] Không phát hiện tranh chấp bản quyền hoặc trademark cản trở.
   - [x] Đã thiết lập định danh duy nhất (GUID, Pipe, Config path, AppID).
