# Bảng điều khiển TextVN — đặc tả UI thống nhất (Windows + Linux)

> Một bảng điều khiển, cùng tuỳ chọn, cùng nhãn, cùng vị trí trên mọi nền tảng.
> Code: Windows `tray/src/settings_dialog.rs` (Win32 thuần) · Linux
> `adapters/linux-settings/src/settings_window.c` (GTK4) + `settings_model.c` (logic, không GTK).
> Test `labels_match_linux_settings_panel` (tray) đối chiếu nhãn giữa hai bên từng chữ.

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
│ [Hướng dẫn] [Thông tin] [Gõ tắt...] [Cài & bật TSF]¹  [Mặc định] [Đóng] [Kết thúc]¹ │
└───────────────────────────────────────────────────────────────────────┘
```

¹ Chỉ Windows. Linux không có nhóm "Hệ thống": IBus/Fcitx5 tự chạy cùng phiên đăng nhập,
không có tiến trình khay riêng; bảng mở từ menu IBus/Fcitx5 ("Cài đặt TextVN…") hoặc
launcher ứng dụng.

## 2. Tuỳ chọn ↔ khoá cấu hình

| Nhãn | Khoá | Mặc định |
|---|---|---|
| Bảng mã: Unicode dựng sẵn / Unicode tổ hợp / TCVN3 (ABC) / VNI Windows | `config.output_charset` = `unicode_precomposed` / `unicode_decomposed` / `tcvn3` / `vni_windows` | Unicode dựng sẵn |
| Kiểu gõ: Telex / VNI / VIQR / Telex đơn giản | `config.method` = `telex` / `vni` / `viqr` / `simple_telex` | Telex |
| Bật gõ tiếng Việt | `state.json` `global_enabled` (cùng file IME ghi khi bấm Ctrl+Shift) | bật |
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
