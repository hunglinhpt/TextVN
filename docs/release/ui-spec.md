# Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux + macOS)

> Một bảng điều khiển, cùng tuỳ chọn, cùng nhãn, cùng thứ tự trên CẢ 3 nền tảng
> (0.2.14 — yêu cầu chủ repo "buộc phải giống nhau").
> Code: Windows `tray/src/settings_dialog.rs` (Win32 thuần) · Linux
> `adapters/linux-settings/src/settings_window.c` (GTK4) + `settings_model.c` (logic, không GTK)
> · macOS `adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift` (SwiftUI).
> Test `labels_match_linux_settings_panel` (tray) đối chiếu nhãn từng chữ giữa
> Windows ↔ Linux ↔ macOS trong CI.

## 1. Bố cục

```
┌ TextVN - Bảng điều khiển ─────────────────────────────────────────────┐
│ ┌ Điều khiển ───────────────────────────────────────────────────────┐ │
│ │ Bảng mã: [Unicode dựng sẵn ▾]          Kiểu gõ: [Telex ▾]          │ │
│ │ Phím chuyển:  Ctrl + Shift   (hoặc Ctrl + Shift + Space)          │ │
│ └───────────────────────────────────────────────────────────────────┘ │
│ ┌ Tùy chọn gõ ──────────────────────────────────────────────────────┐ │
│ │ [x] Bật gõ tiếng Việt                 (•) Dấu mới (hoà, thuỷ)      │ │
│ │ [x] Khôi phục từ tiếng Anh khi gõ sai ( ) Dấu cũ (hòa, thủy)       │ │
│ │ [x] Đặt dấu tự do                     [x] Tự viết hoa chữ đầu câu  │ │
│ │ [ ] Quick Telex (cc→ch, nn→ng…)       [ ] Gõ tắt cả khi tắt tiếng Việt │
│ └───────────────────────────────────────────────────────────────────┘ │
│ ┌ Hệ thống (chỉ Windows) ───────────────────────────────────────────┐ │
│ │ [ ] Khởi động cùng Windows            [x] Bật hội thoại này khi khởi động │
│ │ [x] Dành Ctrl + Shift cho TextVN (tắt phím đổi bàn phím của Windows)  │ │
│ └───────────────────────────────────────────────────────────────────┘ │
│ [Hướng dẫn] [Thông tin] [Gõ tắt...] [Từ điển EN...] [Cài & bật TSF]¹ [Mặc định] [Đóng] [Kết thúc]¹ │
└───────────────────────────────────────────────────────────────────────┘
```

¹ Chỉ Windows. Linux không có nhóm "Hệ thống": IBus/Fcitx5 tự chạy cùng phiên đăng nhập,
không có tiến trình khay riêng; bảng mở từ menu IBus/Fcitx5 ("Cài đặt TextVN…") hoặc
launcher ứng dụng. ² macOS dùng bố cục SwiftUI riêng nhưng **cùng nhãn + cùng thứ tự
tuỳ chọn**; các hàng riêng của macOS (`Khởi động cùng OS`, `Gõ không gạch chân
(Non-preedit)`, `Chạy ngầm trong menu bar`) nằm ở nhóm "Hệ thống" và không phá parity.

> Quy tắc cứng cho nhãn dài (sự cố "gõ sa" 2026-10-03): **nhãn checkbox/radio luôn
> chiếm trọn bề rộng cột** — không bao giờ thu hẹp để nhét thêm nút cùng hàng, vì ở
> màn hình DPI nhỏ chữ sẽ bị cắt. Nút phụ (Từ điển EN...) đặt ở HÀNG NÚT dưới.

## 2. Tuỳ chọn ↔ khoá cấu hình

| Nhãn | Khoá | Mặc định |
|---|---|---|
| Bảng mã: Unicode dựng sẵn / Unicode tổ hợp / TCVN3 (ABC) / VNI Windows | `config.output_charset` = `unicode_precomposed` / `unicode_decomposed` / `tcvn3` / `vni_windows` | Unicode dựng sẵn |
| Kiểu gõ: Telex / VNI / VIQR / Telex đơn giản | `config.method` = `telex` / `vni` / `viqr` / `simple_telex` | Telex |
| Bật gõ tiếng Việt | `state.json` `global_enabled` (Windows/Linux); macOS: trạng thái toàn cục trong AppDelegate (menu bar + SettingsView cùng một nguồn) | bật |
| Dấu mới / Dấu cũ | `config.diacritic_style` = `new` / `old` | Dấu mới |
| Khôi phục từ tiếng Anh khi gõ sai | `config.auto_restore_english` | bật |
| Đặt dấu tự do | `config.free_marking` | bật |
| Tự viết hoa chữ đầu câu | `config.auto_capitalize` | bật |
| Quick Telex | `config.quick_telex` | tắt |
| Gõ tắt cả khi tắt tiếng Việt | `config.allow_macro_when_vi_off` | tắt |
| Khởi động cùng Windows | `HKCU\…\Run\TextVN` | tắt |
| Bật hội thoại này khi khởi động | `config.show_dialog_on_startup` | bật |
| Dành Ctrl + Shift cho TextVN | `HKCU\Keyboard Layout\Toggle` (`Layout Hotkey`/`Language Hotkey` ≠ Ctrl + Shift) — cài đặt của Windows, không nằm trong config | bộ cài chọn sẵn |
| Gõ tắt... | `config.macros[]`, `config.macro_trigger` (`tab`/`space`) | trống, Tab |
| Từ điển EN... | `config.english_words[]` (chuẩn hoá trim/lowercase/a–z/dedupe — CÙNG quy tắc 3 nền tảng) | trống |

File: Windows `%APPDATA%\TextVN\`, Linux `$XDG_CONFIG_HOME/TextVN/` (mặc định `~/.config/TextVN/`).

## 3. Hành vi chung

- **Lưu ngay** khi đổi (không có nút Áp dụng). IME đọc lại file ở phím kế tiếp (Linux: theo
  mtime; Windows: tray phát `ConfigReload`/`StateUpdate` qua IPC).
- **Vá từng khoá** trên file mới nhất (`textvn_config::SettingsDoc`, C API
  `ffi/include/textvn_settings.h`): giữ nguyên gõ tắt, emoji, `english_words` và khoá người
  dùng tự thêm; ghi nguyên tử (tmp → fsync → rename); file hỏng được giữ thành `.bak`.
- **Theo trạng thái ngoài**: bấm Ctrl+Shift khi bảng đang mở → ô "Bật gõ tiếng Việt" cập nhật
  (Windows: `WM_UPDATE_TRAY_STATE`; Linux: `GFileMonitor` trên thư mục cấu hình).
- **Mặc định**: mọi tuỳ chọn gõ về mặc định; **không** xoá bảng gõ tắt (như UniKey).
- **Bàn phím**: Tab/Shift+Tab di chuyển, Esc đóng cửa sổ.

## 4. Cửa sổ Gõ tắt

- Ô văn bản, mỗi dòng `gõ tắt = nội dung`; dòng trống / bắt đầu bằng `#` bỏ qua;
  `\n` = xuống dòng, `\\` = dấu `\`. Định dạng dùng chung: `config/src/macro_text.rs`.
- Kiểm tra khi bấm **Lưu**: thiếu `=`, chữ tắt rỗng/có khoảng trắng/dài quá 32 ký tự,
  nội dung rỗng/dài quá 64 ký tự (giới hạn `ime_result_v1`), trùng chữ tắt (không phân biệt
  hoa thường). Lỗi → báo "Dòng N: …", bôi đen dòng sai, không lưu gì.
- "Bung gõ tắt bằng phím: (•) Tab ( ) Space" → `macro_trigger`.
- Gõ tắt khớp **đầu từ** (không bung giữa chữ `xcty`), và bị huỷ sau Home/End/phím điều
  hướng/Ctrl+V… để không bao giờ xoá nhầm chữ.
- "Gõ tắt cả khi tắt tiếng Việt": khi tắt VN, chữ đang gõ được giữ trong vùng soạn (gạch
  chân) tới hết từ để có thể bung gõ tắt — không xoá lùi chữ đã nằm trong ứng dụng.

## 5. Cửa sổ Từ điển EN (cả 3 nền tảng, 0.2.13+)

- Mỗi dòng một từ tiếng Anh; chỉ chữ a–z; `#` = ghi chú; chuẩn hoá lowercase + khử
  trùng lặp khi lưu (cùng quy tắc ở `tray::normalize_word_list`,
  `config::SettingsDoc::set_english_words_text` — một hành vi cho cả 3 UI).
- Từ trong danh sách thắng mọi phỏng đoán engine (kể cả `cow`→`cơ`) — xem
  `docs/specs/language-detection.md` §2 cấp 6c.
- Windows: nút hàng dưới → hộp thoại; Linux: nút hàng dưới → cửa sổ GTK; macOS:
  nút cạnh "Gõ tắt..." → sheet. Lưu → broadcast reload ngay.

## 6. Nhận diện EN/VI (không có UI riêng — hành vi engine)

- Bật/tắt bằng checkbox "Khôi phục từ tiếng Anh khi gõ sai" (`auto_restore_english`).
- Tab gợi ý từ EN + Escape khôi phục: hành vi engine, không cần cấu hình.
- Quy tắc đầy đủ: `docs/specs/language-detection.md`.

## 7. Icon — một thiết kế cho 3 nền tảng

| Ngữ cảnh | Thiết kế | Nguồn |
|---|---|---|
| Icon ứng dụng | Badge bo góc chữ V (hồng #C2185B), gradient trắng→#FCE4EC | `resources/icons/textvn_v.svg` → PNG/ICO/ICNS |
| Trạng thái **V** (đang gõ tiếng Việt) | Badge chữ V, màu hồng–tím | Windows `textvn_v.ico`; Linux `textvn_v.svg/png`; macOS menu bar chữ "[V]" + tint `systemPink` |
| Trạng thái **E** (tiếng Anh) | Badge chữ E, màu xanh dương #0288D1 | Windows `textvn_e.ico`; Linux `textvn_e.svg/png`; macOS "[E]" + tint `systemBlue` |
| Lỗi | — | macOS tint `systemOrange` (riêng macOS) |

Sinh lại PNG/ICNS: `python scripts/generate_app_icons.py` (nguồn sự thật là 2 SVG).
