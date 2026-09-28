<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Bảng kiểm tra đồng bộ giao diện cài đặt (Settings Parity Checklist)

> Đối chiếu từng control của bảng điều khiển với khoá cấu hình thật (`config/src/lib.rs`,
> `schemas/config.v1.schema.json`) và test chứng minh. Bố cục/nhãn: [ui-spec.md](ui-spec.md).
> Cập nhật 2026-09-28 — bản trước mô tả một UI egui 6 tab chưa từng tồn tại; đã thay bằng
> trạng thái thật.

Chú thích: ✓ = có và có test · — = không áp dụng trên nền tảng đó · ✗ = chưa có.

## 1. Điều khiển & Tùy chọn gõ

| Control | Khoá | Windows (Win32) | Linux (GTK4) | macOS | Test |
|---|---|---|---|---|---|
| Bảng mã (4 bảng) | `output_charset` | ✓ | ✓ | ✗ | `core::transform::charset::tests`, corpus `charset_*` |
| Kiểu gõ (Telex/VNI/VIQR/Telex đơn giản) | `method` | ✓ | ✓ | ✗ | corpus `telex_*`/`vni_*`/`viqr_*`/`simple_telex_*` |
| Phím chuyển Ctrl+Shift / Ctrl+Shift+Space | — (cố định) | ✓ | ✓ | ✗ | `compose::tests::ctrl_shift_tap_toggles_once_and_only_without_other_keys`, e2e IBus/Fcitx5 "Ctrl+Shift → EN/VN" |
| Bật gõ tiếng Việt | `state.json` `global_enabled` | ✓ | ✓ | ✗ | `svc::tests`, `test_compose` `test_state_file`, e2e "state.json → EN/VN" |
| Dấu mới / Dấu cũ | `diacritic_style` | ✓ | ✓ | ✗ | `transform::diacritic_style` tests, corpus `telex_hoaf_new_style_01` |
| Khôi phục từ tiếng Anh khi gõ sai | `auto_restore_english` | ✓ | ✓ | ✗ | `restore_en_*` |
| Đặt dấu tự do | `free_marking` | ✓ | ✓ | ✗ | `method::telex` / `method::vni` tests (fold với `free_marking`) |
| Tự viết hoa chữ đầu câu | `auto_capitalize` | ✓ | ✓ | ✗ | `auto_capitalize_*` |
| Quick Telex | `quick_telex` | ✓ | ✓ | ✗ | `post::quick_telex::tests`, corpus `quick_telex_*` |
| Gõ tắt cả khi tắt tiếng Việt | `allow_macro_when_vi_off` | ✓ | ✓ | ✗ | corpus `macro_vi_off_*` (5 adapter), `test_macro_when_vi_off`, e2e "tắt VN + gõ tắt" |

## 2. Gõ tắt

| Control | Khoá | Windows | Linux | macOS | Test |
|---|---|---|---|---|---|
| Bảng gõ tắt (soạn dạng text, báo dòng lỗi) | `macros[]` | ✓ | ✓ | ✗ | `config::macro_text::tests`, `test_settings_model` |
| Bung bằng Tab / Space | `macro_trigger` | ✓ | ✓ | ✗ | corpus `macro_expand_*` |
| Không bung sau Home/End/Ctrl+V | — | ✓ (engine) | ✓ (engine) | ✗ | `macro_not_expanded_after_caret_jump_or_chord` |

## 3. Hệ thống & nút

| Control | Khoá / hành động | Windows | Linux | macOS |
|---|---|---|---|---|
| Khởi động cùng Windows | `HKCU\…\Run\TextVN` | ✓ | — (IBus/Fcitx5 tự chạy) | ✗ |
| Bật hội thoại này khi khởi động | `show_dialog_on_startup` | ✓ | — (không có tiến trình khay) | ✗ |
| Hướng dẫn / Thông tin | — | ✓ | ✓ | ✗ |
| Mặc định (giữ gõ tắt) | `SettingsDoc::reset_defaults` | ✓ | ✓ | ✗ |
| Cài & bật TSF | `textvn-cli register` | ✓ | — | — |
| Đóng / Kết thúc | ẩn về khay / thoát tray | ✓ | Đóng | ✗ |
| Mở bảng từ menu bộ gõ | "Cài đặt TextVN…" | tray | IBus property · Fcitx5 status action | ✗ |

## 4. Chưa có trên UI (chỉ sửa được trong file)

| Khoá | Ghi chú |
|---|---|
| `emoji[]`, `english_words[]` | Engine hỗ trợ; bảng điều khiển giữ nguyên khi lưu. |
| `hotkeys`, `ignore_apps`, `app_overrides`, `updates`, `suggest` | Schema chấp nhận, engine chưa dùng. |

macOS: chưa có adapter (xem `docs/30-macos/`), nên toàn bộ cột macOS là ✗.
