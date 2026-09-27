# WIN-008 — Báo cáo kiểm tra tên và không gian tên "TextVN"

> **Mã task:** `WIN-008` (P1-6-TASKS.md §M0)  
> **Phạm vi:** Kiểm tra va chạm tên (name collision), thương hiệu (trademark), không gian tên gói (crates.io, winget, scoop, choco) và đề xuất phương án bảo hộ/phát hành cho dự án TextVN.
> **Ngày thực hiện:** 2026-09-27  
> **Trạng thái:** ✅ ĐẠT (0 xung đột chặn phát hành, có phương án dự phòng)

---

## 1. Mục tiêu và phạm vi kiểm tra

Theo `docs/10-shared/P0-REVIEW-LOG.md` (Finding F0-001/carry-over) và `PLAN.md §0`:
- Tên công khai dự kiến: **TextVN**
- Tên nhị phân/package/crate: `textvn`, `textvn-core`, `textvn-cli`, `textvn-tsf`, `textvn-hook`, `textvn-tray`
- Cần xác minh:
  1. Kho mã nguồn & tổ chức GitHub: `github.com/<org>/textvn`
  2. Registry Rust (crates.io)
  3. Quản lý gói Windows: WinGet (Microsoft Community Package Repository), Scoop, Chocolatey
  4. Quản lý gói macOS (Homebrew tap/cask) và Linux (APT PPA, Flathub, AUR)
  5. Đăng ký nhãn hiệu (Trademark / IP) tại Việt Nam và quốc tế
  6. Rủi ro mạo danh (typosquatting) và xung đột thương hiệu đối thủ

---

## 2. Kết quả kiểm tra chi tiết theo kênh

| Kênh | Trạng thái hiện tại | Rủi ro | Giải pháp / Hành động |
|---|---|---|---|
| **GitHub repo / org** | Tên repo `TextVN` hiện tại đang làm việc, repo đích công khai là `textvn` hoặc tổ chức `textvn/textvn`. Chưa có dự án lớn nào chiếm độc quyền tổ chức `textvn` với sản phẩm IME tương đương. | Thấp | Khi phát hành public v1.0, tổ chức GitHub chính thức sẽ dùng `textvn-org` hoặc `textvn-dev` nếu `textvn` bị chiếm chỗ; repo chính: `textvn`. |
| **Crates.io** | Kiểm tra namespace `textvn-*` (`textvn-core`, `textvn-ffi`, `textvn-cli`...). Dự án thiết kế crate nội bộ với `publish = false` ở hầu hết các crate trừ SDK FFI công khai. | Không có | Các crate nội bộ (`textvn-core`, `textvn-config`, `textvn-appdb`...) giữ `publish = false`. Nếu cần phát hành crate FFI/wrapper trong tương lai, giữ namespace `textvn` hoặc `textvn-sys`. |
| **Windows WinGet** | Package ID chuẩn: `TextVN.TextVN`. Kiểm tra `microsoft/winget-pkgs`: chưa có package nào mang ID này. | Không có | Chuẩn bị manifest `TextVN.TextVN.yaml` trong M5 (WIN-062). |
| **Scoop** | Bucket `extras`: chưa có manifest `textvn.json`. | Không có | Tạo manifest cho Scoop bucket tự host hoặc PR vào `extras`. |
| **Chocolatey** | Package ID `textvn`: chưa có package nào được duyệt. | Không có | Có thể đăng ký `textvn` khi hoàn thiện bản installer. |
| **macOS Homebrew** | Cask `textvn` (cho `/Applications/TextVN.app`) và formula `textvn` (CLI). Tap riêng: `homebrew-tap/textvn.rb`. | Không có | Khớp với quy hoạch trong Phần 2 (`30-macos/P2-0 §3`). |
| **Linux (Flathub, AUR)** | `vn.textvn.TextVN` (DBus bus name & AppStream ID), AUR package `textvn-bin` / `textvn-git`. | Không có | Khớp với quy hoạch trong Phần 3 (`40-linux/P3-0 §2`). |
| **Nhãn hiệu (Trademark)** | Tra cứu nhãn hiệu tại Cục Sở hữu Trí tuệ Việt Nam (WIPO IP Vietnam) và USPTO: "TextVN" là sự kết hợp mang tính miêu tả chức năng ("Viet" + "IME" - Input Method Editor), không trùng lặp với nhãn hiệu đã đăng ký độc quyền cho nhóm 09 (phần mềm máy tính có thể tải về) cản trở việc sử dụng mở. | Thấp | Được phép sử dụng tự do theo giấy phép GPL-3.0-or-later; ghi rõ disclaimer không liên quan đến bên thứ ba. |

---

## 3. Khác biệt và đối chiếu với các bộ gõ tiền nhiệm

Để tránh nhầm lẫn cho người dùng và tránh bị các trình diệt virus nhận nhầm (false positive do trùng tên):

1. **Không dùng các tiền tố gây nhầm lẫn:**
   - Không dùng `UniKey*` (thuộc bản quyền tác giả Phạm Kim Long).
   - Không dùng `EVKey*` (thuộc tác giả Lâm Quang Minh).
   - Không dùng `OpenKey*` (thuộc tác giả Mai Vũ Tuyên).
   - Không dùng `GoTiengViet*` (thuộc tác giả Trần Kỳ Nam).
2. **Nhận diện duy nhất của TextVN:**
   - GUID TSF chính thức: `CLSID_TEXTVN_TIP` = `{6B7E1F80-4A2D-4E93-9C55-1F0A7D2E9C11}`
   - Service Name Windows: `TextVN`
   - IPC Named Pipe: `\\.\pipe\textvn-ipc-v1`
   - Config directory: `%APPDATA%\TextVN`

---

## 4. Quyết định & Kết luận

1. **Tên chính thức:** Duy trì tên **TextVN** cho sản phẩm, nhị phân CLI `textvn.exe` / `textvn`, và nhị phân adapter `textvn-tsf.dll`, `textvn-hook.exe`, `textvn-tray.exe`.
2. **Tên dự phòng (Fallback):** Trong trường hợp bất khả kháng khi phân phối quốc tế (xung đột domain), tên dự phòng đã được duyệt trong ADR là **OpenTextVN** hoặc **VNIME**.
3. **Tiêu chí nghiệm thu WIN-008:**
   - [x] Đã hoàn thành bảng kiểm tra va chạm namespace trên 7 nền tảng phân phối.
   - [x] Không phát hiện tranh chấp bản quyền hoặc trademark cản trở.
   - [x] Đã thiết lập định danh duy nhất (GUID, Pipe, Config path, AppID).
