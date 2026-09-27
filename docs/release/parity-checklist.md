<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Bảng kiểm tra tính đồng bộ giao diện cài đặt (Settings Parity Checklist)

> Tài liệu chuẩn cho task WIN-052 (Windows), MAC-052 (macOS), LIN-052 (Linux) theo ADR-004, P1-4 §3, PLAN §2.3 (M6) và PLAN §8.
> Mỗi control cài đặt trên giao diện người dùng phải được đối chiếu 1-1 với cấu hình trong `Config` struct (`config/src/lib.rs`) và có test case tương ứng.

## 1. Tab General (Cài đặt chung)

| Control | Kiểu UI | Trường trong `Config` | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Bật gõ tiếng Việt mặc định | Checkbox | `enabled_default: bool` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_enabled_default` |
| Kiểu gõ (Method) | Dropdown | `method: Method` (Telex, VNI, VIQR, SimpleTelex) | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_method_switch` |
| Kiểu bỏ dấu (Diacritic Style) | Radio/Dropdown | `diacritic_style: DiacriticStyle` (New, Old) | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_diacritic_style` |
| Bỏ dấu tự do (Free marking) | Checkbox | `free_marking: bool` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_free_marking` |
| Kiểm tra chính tả (Spellcheck) | Dropdown | `spellcheck: SpellcheckMode` (Off, Should, Must) | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_spellcheck` |
| Khôi phục từ tiếng Anh (Auto restore) | Checkbox | `auto_restore_english: bool` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_auto_restore` |
| Ngôn ngữ hiển thị giao diện | Dropdown | `ui_language: String` ("vi", "en") | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_general_ui_lang` |

## 2. Tab Applications (Ứng dụng & Loại trừ)

| Control | Kiểu UI | Trường trong `Config` / `AppDB` | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Danh sách ứng dụng tắt tiếng Việt (`ignore_apps`) | List / Tags | `ignore_apps: Vec<String>` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_apps_ignore_list` |
| Ghi đè cấu hình theo ứng dụng (`app_overrides`) | Key-Value Table | `app_overrides: BTreeMap<String, AppOverride>` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_apps_overrides` |
| Nút "Thêm từ cửa sổ đang mở" | Button | Foreground Process ID detection | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_apps_add_foreground` |
| Nạp preset bổ sung (`appdb.json`) | Button / FilePicker | User AppDB path | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_apps_load_preset` |

## 3. Tab Hotkeys (Phím tắt chuyển đổi)

| Control | Kiểu UI | Trường trong `Config` | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Phím tắt chuyển Anh/Việt | Hotkey Input | `hotkey_toggle: String` ("Ctrl+Shift+Space", "Alt+Z") | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_hotkey_toggle` |
| Cảnh báo xung đột phím tắt | Status Label | Runtime key validation | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_hotkey_conflict_detect` |
| Bật gõ tắt ngay cả khi tắt tiếng Việt | Checkbox | `macro_when_disabled: bool` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_hotkey_macro_when_disabled` |

## 4. Tab Hook & Game (Chế độ tương thích sâu)

| Control | Kiểu UI | Trường trong `Config` | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Chế độ Hook LL | Dropdown | `hook_mode: HookMode` (Off, Auto, Always) | ✓ Hoàn thành | N/A (Tap opt-in) | N/A (X11 opt-in) | `test_cfg_hook_mode` |
| Danh sách game/app dùng Hook | List / Tags | `hook_apps: Vec<String>` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_hook_apps` |
| Cảnh báo UIAccess / Administrator | Warning Banner | Process integrity check (Medium vs High) | ✓ Hoàn thành | N/A | N/A | `test_cfg_hook_uiaccess_warning` |

## 5. Tab Update (Cập nhật phần mềm)

| Control | Kiểu UI | Trường trong `Config` | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Kênh cập nhật | Dropdown | `update_channel: String` ("stable", "beta") | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_update_channel` |
| Tự động kiểm tra bản mới | Checkbox | `auto_check_update: bool` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_update_auto_check` |
| Nút "Kiểm tra cập nhật ngay" | Button | Call GitHub API latest release | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_update_check_now` |

## 6. Tab About & Support (Thông tin & Hỗ trợ)

| Control | Kiểu UI | Nguồn dữ liệu | Windows (egui) | macOS (SwiftUI) | Linux (GTK4) | Test Case |
|---|---|---|---|---|---|---|
| Thông tin phiên bản & ABI | Label | `env!("CARGO_PKG_VERSION")`, `IME_ABI_VERSION` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_about_version` |
| Nút "Xuất file chẩn đoán (Export Diagnostics)" | Button | `textvn doctor --export` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_about_export_doctor` |
| Nút "Mở thư mục nhật ký (Open Logs Folder)" | Button | `%LOCALAPPDATA%\TextVN\logs` | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_about_open_logs` |
| Giấy phép & Tác giả | Markdown link | GPL-3.0-or-later, TextVN Authors | ✓ Hoàn thành | ⏳ Planned | ⏳ Planned | `test_cfg_about_license` |
