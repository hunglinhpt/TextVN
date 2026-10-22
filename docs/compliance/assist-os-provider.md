# Tuân thủ Giấy phép & Riêng tư — Các Nhà cung cấp Chính tả Hệ điều hành (OS Spellcheck Providers)

> Tài liệu tuân thủ bản quyền và quyền riêng tư theo yêu cầu P4-10 §7.3 & Finding P4-10-F8 (`docs/50-assist/P4-REVIEW-LOG.md`).

## 1. Tổng quan kiến trúc

Tính năng Trợ lý ngữ pháp (Phần 4 — Assist) của TextVN hỗ trợ kết nối tùy chọn tới hệ thống kiểm tra chính tả có sẵn của hệ điều hành thông qua C-ABI `ime_os_spell_v1` (`ffi/include/textvn_ffi.h`). TextVN **không** đóng gói sẵn bộ từ điển nhị phân có bản quyền hạn chế vào gói cài đặt chính thức (tuân thủ ADR-014).

| Nền tảng | Thư viện / API hệ thống | Cơ chế liên kết | Giấy phép | Trạng thái phân phối |
|---|---|---|---|---|
| **Linux** | `libenchant-2` (`enchant-2`) / `libspelling` | Liên kết động (`dlopen` / dynamic link `libenchant-2.so`) | LGPL-2.1-or-later | Thư viện hệ thống người dùng tự cài đặt (`libenchant-2-2`), TextVN **không** phân phối kèm file nhị phân static |
| **Windows** | Windows Spell Checking API (`ISpellCheckerFactory`) | COM In-proc / Local RPC (`ole32.dll` / `combase.dll`) | Microsoft Proprietary OS Component | Thành phần hệ thống tích hợp sẵn của Windows 8/10/11 |
| **macOS** | AppKit `NSSpellChecker` | Dynamic link (`AppKit.framework`) | Apple Proprietary OS Component | Thành phần hệ thống tích hợp sẵn của macOS |

---

## 2. Tuân thủ Giấy phép LGPL (Linux — enchant-2 & libspelling)

### 2.1 Không đóng gói tĩnh (No Static Linking)
TextVN tuân thủ nghiêm ngặt các điều khoản của GNU Lesser General Public License v2.1 (LGPL-2.1-or-later):
- `adapters/linux-common` và các adapter Linux (`fcitx5`, `ibus`) giao tiếp với `libenchant-2` thông qua liên kết động (`dlopen` hoặc dynamic link đối với thư viện chia sẻ của hệ thống).
- Người dùng hoàn toàn có quyền thay thế, cập nhật hoặc tùy biến phiên bản `libenchant-2.so` trên máy của họ mà không ảnh hưởng tới tính toàn vẹn của TextVN.
- Mã nguồn của TextVN được phát hành công khai toàn bộ theo giấy phép `GPL-3.0-or-later` tại [https://github.com/hunglinhpt/TextVN](https://github.com/hunglinhpt/TextVN).

### 2.2 Nguồn gốc Upstream
- **Enchant**: Dự án mã nguồn mở do Reuben Thomas duy trì.
  - Trang chủ: [https://rrthomas.github.io/enchant/](https://rrthomas.github.io/enchant/)
  - Kho mã nguồn: [https://github.com/rrthomas/enchant](https://github.com/rrthomas/enchant)
  - Giấy phép: `LGPL-2.1-or-later` với điều khoản ngoại lệ cho các backend kiểm tra chính tả.
- **Libspelling**: Dự án của GNOME.
  - Kho mã nguồn: [https://gitlab.gnome.org/GNOME/libspelling](https://gitlab.gnome.org/GNOME/libspelling)
  - Giấy phép: `LGPL-2.1-or-later`.

---

## 3. Chính sách Quyền riêng tư & Luồng Dữ liệu (Privacy & Data Flow)

### 3.1 Xử lý cục bộ & Không gửi qua mạng
- Toàn bộ các cuộc gọi kiểm tra từ ngữ (`contains`, `suggest`) tới các provider hệ thống được thực thi **cục bộ 100% trên thiết bị**.
- TextVN **tuyệt đối không truyền dữ liệu văn bản hay nội dung gõ phím của người dùng ra mạng Internet** (Quy tắc S2 & Nguyên tắc Kiến trúc A14).

### 3.2 Phạm vi tiến trình (Process Boundaries)
- **Linux & macOS**: Việc gọi `enchant-2` hoặc `NSSpellChecker` diễn ra in-process trong không gian bộ nhớ của tiến trình bộ gõ.
- **Windows**: Windows Spell Checking API có thể gọi tới dịch vụ kiểm tra chính tả out-of-proc của Windows (`dllhost.exe` hoặc COM server hệ thống) thông qua RPC cục bộ. Đây là hành vi tiêu chuẩn của hệ điều hành Windows dành cho mọi ứng dụng gọi API này.

### 3.3 Quyền kiểm soát & Kill-Switch
- Tính năng Trợ lý ngữ pháp **mặc định tắt hoàn toàn** (`assist.mode = "off"`, `assist.auto_fix = "off"`).
- Người dùng có thể bật/tắt toàn bộ tính năng Trợ lý ngay lập tức thông qua giao diện Cài đặt hoặc menu khay hệ thống (tray menu). Khi tắt, TextVN không thực hiện bất kỳ lệnh gọi API nào tới các thư viện kiểm tra chính tả nêu trên.
