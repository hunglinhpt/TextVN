# Đối chiếu bộ gõ tham chiếu — tính năng & bug đã biết (2026-09-28)

> Nguồn: README/issue của UniKey, GoTiengViet, OpenKey (`tuyenvm/OpenKey`), ibus-bamboo
> (`BambooEngine/ibus-bamboo`), BambooMintKey (`thatislg/BambooMintKey`, `docs/BUILD_LINUX.md`);
> ma trận bug kinh niên B1–B13 ở `../../PLAN.md` §1.3.
> Mục đích: kế thừa tính năng giá trị, **không lặp lại bug đã biết**. Mỗi dòng có trạng thái
> và bằng chứng kiểm chứng trong repo.

## 1. Bug đã biết của bộ gõ tham chiếu → cách TextVN tránh

| # | Bug (nguồn) | Nguyên nhân gốc | TextVN | Bằng chứng |
|---|---|---|---|---|
| R1 | Lặp chữ ở Messenger/Facebook trên Chrome (ibus-bamboo #352) | chế độ không gạch chân: xóa lùi qua surrounding text/ForwardKey khi app báo caret sai | Mặc định **preedit cả từ**, không xóa lùi text đã commit | `scripts/e2e-linux.sh` (IBus + Fcitx5 thật) |
| R2 | Không gõ được ở VS Code/Electron, Brave (ibus-bamboo #49, #559) | app Electron/Chromium không nạp IM module (thiếu `GTK_IM_MODULE`, Wayland cần `--enable-wayland-ime`) | Engine đúng chuẩn; `textvn doctor` kiểm biến môi trường (P3-4 §7) + hướng dẫn user guide | `docs/user-guide.md` §Linux |
| R3 | Popup "Allow remote interaction" khi chơi game (ibus-bamboo #487) | chế độ gửi phím qua XTest/uinput → portal RemoteDesktop | Không dùng XTest/uinput/SendInput trong đường gõ Linux | review `adapters/linux-*` |
| R4 | Enter lặp từ cuối trong chat (B2, ibus/fcitx) | commit preedit trùng với commit của daemon | IBus `PREEDIT_COMMIT` + chỉ quên ở focus-out/reset; Fcitx5 phân biệt FocusOut/Reset | e2e "Enter commit", "focus-out", "reset 1 lần" |
| R5 | Addon "Not Available" (BambooMintKey Issue 010) | `.so` phụ thuộc thư viện riêng không tìm thấy | Engine link **tĩnh** vào `libtextvn-fcitx5.so`, chỉ phụ thuộc Fcitx5 | `--exclude-libs,ALL`, e2e nạp từ thư mục per-user |
| R6 | Steam 32-bit không nạp `libfcitx5gclient.so` (BambooMintKey) | IM module 64-bit trong process 32-bit | Giới hạn của framework — hướng dẫn `GTK_IM_MODULE=xim` cho Steam | `docs/user-guide.md` |
| R7 | Mất icon khay / taskbar (OpenKey #288, #307) | không thêm lại icon khi Explorer khởi động lại (`TaskbarCreated`) | Tray xử lý `TaskbarCreated` + thử lại khi khởi động sớm | `tray/src/main.rs` |
| R8 | Crash khi tự khởi động Win11 / Task Scheduler 0xC0000409 (OpenKey #273, #308) | tạo icon/UI trước khi shell sẵn sàng | Autostart qua `HKCU\...\Run`; tray thử lại `Shell_NotifyIcon` thay vì thoát | `tray/src/main.rs` |
| R9 | "Unknown publisher" (OpenKey #316), AV false positive | chưa ký số | Ký Authenticode ở cổng release; kiến trúc không hook/không inject (`antivirus-false-positive.md` §8–§9) | `build-release.ps1` |
| R10 | Tự viết hoa sai với nguyên âm có dấu (OpenKey #320) | viết hoa trên ký tự đã dựng sẵn | Viết hoa trên ký tự đầu từ trước khi fold | replay: `xin. as` → `xin. Á`, `xin. uwowng` → `xin. Ương` |
| R11 | Chữ dính gợi ý ở thanh địa chỉ/Excel (B1, UniKey/EVKey) | Backspace giả + autocomplete chèn chữ | TSF composition / preedit, không Backspace giả | corpus `bug_B1_*` qua `--adapter tsf` |
| R12 | Không gõ được ở ô tìm kiếm Start/Settings/app Store (TSF) | thiếu category Immersive + DLL không đọc được từ AppContainer | Đăng ký `IMMERSIVESUPPORT` + ACL AppContainer | `cli/src/register.rs` |
| R13 | Đặt dấu sai vị trí (`cuả`, `nghiã`, `thủy` ở kiểu mới; `đựơc` ở kiểu cũ) — lỗi kinh điển của bộ gõ tự viết | chọn vị trí theo "âm cuối cụm" thay vì quy tắc chính tả | Quy tắc đầy đủ: gi/qu là phụ âm, ưu tiên nguyên âm có dấu phụ, có âm cuối → âm cuối cụm, vần 3 âm → giữa, `oa/oe/uy` theo kiểu dấu, còn lại → âm đầu; dời dấu khi gõ tiếp | `transform/diacritic_style.rs`, `core/tests/common_words.rs` (~330 từ × Telex/VNI/kiểu cũ/uow/dấu giữa từ) |
| R14 | `d` + nguyên âm tự thành `đ` → không gõ được `dân`, `dạy`, `dưới` (lỗi của chính TextVN bản trước, do ghi chép oracle sai) | quy tắc "thông minh" không có trong Telex chuẩn | `đ` chỉ qua `dd` như UniKey/OpenKey/Bamboo | corpus `telex_d_plain_01`, test `d_only_becomes_stroke_when_doubled` |
| R15 | Gõ tắt xoá nhầm chữ sau Home/End/Ctrl+V (bộ gõ giữ "đuôi text" cũ) | đuôi text engine nhớ không còn nằm trước con trỏ | Quên đuôi text khi có phím điều hướng/chord; gõ tắt chỉ khớp đầu từ | `macro_not_expanded_after_caret_jump_or_chord` |

## 2. Tính năng tham chiếu → trạng thái

| Tính năng | UniKey | OpenKey | ibus-bamboo | TextVN |
|---|---|---|---|---|
| Telex / VNI / VIQR / Simple Telex | ✅ | ✅ | ✅ | ✅ |
| Bỏ dấu kiểu mới (`hoà`) / cũ (`hòa`) | ✅ | ✅ | ✅ | ✅ |
| Đặt dấu tự do | ✅ | ✅ | ✅ | ✅ |
| Tự khôi phục từ tiếng Anh / từ sai chính tả | – | ✅ | ✅ | ✅ (`auto_restore_english`) |
| ESC khôi phục phím gõ | – | – | – | ✅ |
| Gõ tắt (macro) | ✅ | ✅ | ✅ | ✅ |
| Tự viết hoa đầu câu | – | ✅ | – | ✅ |
| Bảng mã Unicode tổ hợp / TCVN3 / VNI Windows | ✅ | ✅ | ✅ | ✅ (`output_charset`, bảng UniKey — corpus `charset_*`) |
| Quick Telex (`cc→ch, gg→gi, kk→kh, nn→ng, qq→qu, pp→ph, tt→th`) | – | ✅ | – | ✅ (`quick_telex`) |
| Telex `z` gỡ dấu thanh | ✅ | ✅ | ✅ | ✅ |
| `uow` → `ươ` (`nguowif` → `người`) | ✅ | ✅ | ✅ | ✅ |
| Gõ tắt cả khi tắt tiếng Việt | – | ✅ | – | ✅ (`allow_macro_when_vi_off`) |
| Soạn bảng gõ tắt trong bảng điều khiển | ✅ | ✅ | ✅ | ✅ (Windows + Linux, báo dòng lỗi) |
| Nhớ trạng thái V/E qua lần khởi động | ✅ | ✅ | – | ✅ (`state.json`, cả Windows và Linux) |
| Bật/tắt theo từng ứng dụng | – | ✅ | ✅ | ✅ Windows (menu khay) |
| Phím chuyển Ctrl+Shift | ✅ | ✅ | – | ✅ Windows TSF, IBus, Fcitx5, hook |
| Preedit / không gạch chân | – | – | ✅ (6 chế độ) | Preedit (an toàn nhất, xem R1) |
| Ô mật khẩu tự tắt | – | – | – | ✅ (InputScope/ES_PASSWORD, IBus purpose, Fcitx5 capability) |
