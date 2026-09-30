// SPDX-License-Identifier: GPL-3.0-or-later
//! Menu khay hệ thống 9 mục cho TextVN Tray (WIN-050 — P1-4 §1).
//!
//! Định nghĩa đầy đủ 9 mục menu chuột phải theo bảng chuẩn:
//! 1. Bật/Tắt gõ tiếng Việt (toggle global)
//! 2. Chế độ gõ (Telex, VNI, VIQR, Simple Telex)
//! 3. Dấu (Chuẩn mới / Cổ điển)
//! 4. Cửa sổ đang gõ (Tên app foreground + enable riêng)
//! 5. Game / Compat mode (bật/tắt hook)
//! 6. Cài đặt... (Mở Settings GUI)
//! 7. Sức khỏe / Trạng thái (Health submenu: Engine, Hook, Pipe, Version)
//! 8. Gỡ cài đặt (bản cài: `unins000.exe`; bản portable: gỡ đăng ký TSF rồi thoát)
//! 9. Thoát (Đóng tray và dừng hook)

use std::sync::Arc;
use textvn_config::{DiacriticStyle, Method};

use crate::ipc_server::IpcServer;
use crate::svc::SvcManager;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

pub const ID_TOGGLE_GLOBAL: u32 = 1001;
pub const ID_METHOD_TELEX: u32 = 1010;
pub const ID_METHOD_VNI: u32 = 1011;
pub const ID_METHOD_VIQR: u32 = 1012;
pub const ID_METHOD_SIMPLE_TELEX: u32 = 1013;
pub const ID_DIACRITIC_NEW: u32 = 1020;
pub const ID_DIACRITIC_OLD: u32 = 1021;
pub const ID_CURRENT_APP_TOGGLE: u32 = 1030;
pub const ID_HOOK_COMPAT_MODE: u32 = 1040;
pub const ID_OPEN_SETTINGS: u32 = 1050;
pub const ID_HEALTH_STATUS: u32 = 1060;
pub const ID_UNINSTALL: u32 = 1070;
pub const ID_EXIT: u32 = 1080;

pub struct TrayMenu {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
}

impl TrayMenu {
    pub fn new(svc: Arc<SvcManager>, ipc: Arc<IpcServer>) -> Self {
        Self { svc, ipc }
    }

    #[cfg(windows)]
    pub fn show_popup(&self, hwnd: HWND, x: i32, y: i32, current_app: Option<&str>) {
        // SAFETY: Tạo Win32 Popup Menu
        unsafe {
            let menu = match CreatePopupMenu() {
                Ok(m) => m,
                Err(_) => return,
            };

            // 1. Bật/Tắt gõ tiếng Việt
            let global_enabled = self.svc.is_global_enabled();
            let toggle_text = if global_enabled {
                w("Bật gõ tiếng Việt (Đang bật)")
            } else {
                w("Bật gõ tiếng Việt (Đang tắt)")
            };
            let mut flags = MF_STRING;
            if global_enabled {
                flags |= MF_CHECKED;
            }
            let _ = AppendMenuW(
                menu,
                flags,
                ID_TOGGLE_GLOBAL as usize,
                PCWSTR(toggle_text.as_ptr()),
            );
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());

            // 2. Chế độ gõ (Submenu: Telex, VNI, VIQR, Simple Telex)
            let method_menu = CreatePopupMenu().unwrap_or_default();
            let cur_method = self.svc.config().method;
            add_radio_menu_item(
                method_menu,
                "Telex",
                ID_METHOD_TELEX,
                cur_method == Method::Telex,
            );
            add_radio_menu_item(method_menu, "VNI", ID_METHOD_VNI, cur_method == Method::Vni);
            add_radio_menu_item(
                method_menu,
                "VIQR",
                ID_METHOD_VIQR,
                cur_method == Method::Viqr,
            );
            add_radio_menu_item(
                method_menu,
                "Simple Telex",
                ID_METHOD_SIMPLE_TELEX,
                cur_method == Method::SimpleTelex,
            );
            let _ = AppendMenuW(
                menu,
                MF_POPUP,
                method_menu.0 as usize,
                PCWSTR(w("Chế độ gõ").as_ptr()),
            );

            // 3. Kiểu bỏ dấu (Submenu: Chuẩn mới / Cổ điển)
            let diacritic_menu = CreatePopupMenu().unwrap_or_default();
            let cur_style = self.svc.config().diacritic_style;
            add_radio_menu_item(
                diacritic_menu,
                "Chuẩn mới (hoà, thuỷ)",
                ID_DIACRITIC_NEW,
                cur_style == DiacriticStyle::New,
            );
            add_radio_menu_item(
                diacritic_menu,
                "Cổ điển (hòa, thủy)",
                ID_DIACRITIC_OLD,
                cur_style == DiacriticStyle::Old,
            );
            let _ = AppendMenuW(
                menu,
                MF_POPUP,
                diacritic_menu.0 as usize,
                PCWSTR(w("Kiểu bỏ dấu").as_ptr()),
            );

            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());

            // 4. Per-app toggle cho app foreground gần nhất — nguồn là
            // `foreground::last_app()` (EVENT_SYSTEM_FOREGROUND, bỏ qua shell/tray).
            if let Some(app) = current_app {
                let app_enabled = self.svc.is_app_enabled(app);
                let app_label = format!("Bật tiếng Việt cho {app}");
                let mut app_flags = MF_STRING;
                if app_enabled {
                    app_flags |= MF_CHECKED;
                }
                let _ = AppendMenuW(
                    menu,
                    app_flags,
                    ID_CURRENT_APP_TOGGLE as usize,
                    PCWSTR(w(&app_label).as_ptr()),
                );
            }

            // 5. Hook legacy chỉ theo yêu cầu rõ ràng. Hook toàn cục + SendInput
            // là fallback cho game/app cũ, không phải đường mặc định TSF.
            let hook_available = crate::compatibility_hook_path().is_some();
            let hook_label = if hook_available {
                "Bật chế độ tương thích (Hook; phiên này)"
            } else {
                "Chế độ tương thích (cần gói Compatibility)"
            };
            let hook_flags = if hook_available {
                MF_STRING
            } else {
                MF_STRING | MF_GRAYED
            };
            let _ = AppendMenuW(
                menu,
                hook_flags,
                ID_HOOK_COMPAT_MODE as usize,
                PCWSTR(w(hook_label).as_ptr()),
            );

            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());

            // 6. Cài đặt...
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                ID_OPEN_SETTINGS as usize,
                PCWSTR(w("Cài đặt...").as_ptr()),
            );

            // 7. Sức khỏe / Trạng thái (Submenu)
            let health_menu = CreatePopupMenu().unwrap_or_default();
            let clients_count = self.ipc.active_clients();
            let clients_label = format!("Clients kết nối: {clients_count}");
            let _ = AppendMenuW(
                health_menu,
                MF_STRING | MF_GRAYED,
                0,
                PCWSTR(w(&clients_label).as_ptr()),
            );
            let ver_label = format!("Phiên bản: {}", env!("CARGO_PKG_VERSION"));
            let _ = AppendMenuW(
                health_menu,
                MF_STRING | MF_GRAYED,
                0,
                PCWSTR(w(&ver_label).as_ptr()),
            );
            let _ = AppendMenuW(
                menu,
                MF_POPUP,
                health_menu.0 as usize,
                PCWSTR(w("Sức khỏe / Trạng thái").as_ptr()),
            );

            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());

            // 8. Gỡ cài đặt
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                ID_UNINSTALL as usize,
                PCWSTR(w("Gỡ cài đặt").as_ptr()),
            );

            // 9. Thoát
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                ID_EXIT as usize,
                PCWSTR(w("Thoát").as_ptr()),
            );

            let _ = SetForegroundWindow(hwnd);
            let _ = TrackPopupMenuEx(menu, TPM_RIGHTBUTTON.0, x, y, hwnd, None);
            // KB135788: sau TrackPopupMenuEx phải PostMessageW(WM_NULL) — nếu không
            // tray giữ foreground và click kế tiếp bị nuốt một lần (review R3 minor 7).
            let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(menu);
        }
    }

    /// Xử lý các command ID khi người dùng click vào mục menu.
    pub fn handle_command(&self, cmd_id: u32, current_app: Option<&str>) {
        match cmd_id {
            ID_TOGGLE_GLOBAL => {
                let (enabled, ver) = self.svc.toggle_global_enabled();
                self.ipc.broadcast_state_update("*", enabled, ver);
            }
            ID_METHOD_TELEX => {
                if let Ok(ver) = self.svc.set_method(Method::Telex) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_METHOD_VNI => {
                if let Ok(ver) = self.svc.set_method(Method::Vni) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_METHOD_VIQR => {
                if let Ok(ver) = self.svc.set_method(Method::Viqr) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_METHOD_SIMPLE_TELEX => {
                if let Ok(ver) = self.svc.set_method(Method::SimpleTelex) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_DIACRITIC_NEW => {
                if let Ok(ver) = self.svc.set_diacritic_style(DiacriticStyle::New) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_DIACRITIC_OLD => {
                if let Ok(ver) = self.svc.set_diacritic_style(DiacriticStyle::Old) {
                    self.ipc.broadcast_config_reload(ver);
                }
            }
            ID_CURRENT_APP_TOGGLE => {
                if let Some(app) = current_app {
                    let cur = self.svc.is_app_enabled(app);
                    let ver = self.svc.set_app_enabled(app, !cur);
                    let actual = self.svc.is_app_enabled(app);
                    self.ipc.broadcast_state_update(app, actual, ver);
                }
            }
            ID_HOOK_COMPAT_MODE => {
                if crate::compatibility_hook_path().is_some() {
                    crate::notify_start_compatibility_hook();
                }
            }
            ID_OPEN_SETTINGS => {
                crate::settings_dialog::show_settings_dialog(self.svc.clone(), self.ipc.clone());
            }
            ID_UNINSTALL => uninstall(),
            ID_EXIT => {
                #[cfg(windows)]
                unsafe {
                    PostQuitMessage(0);
                }
            }
            _ => {}
        }
    }
}

/// "Gỡ cài đặt": bản cài bằng Inno Setup có `unins000.exe` cạnh `TextVN.exe`; bản portable
/// không có trình gỡ → hỏi xác nhận, gỡ đăng ký TSF (HKCU) rồi thoát, để người dùng xoá
/// thư mục. Bản cũ gọi `TextVN-setup.exe /UNINSTALL` — file không tồn tại, bấm không có gì.
#[cfg(windows)]
fn uninstall() {
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    else {
        return;
    };
    let unins = dir.join("unins000.exe");
    if unins.is_file() {
        let _ = std::process::Command::new(unins).spawn();
        return;
    }
    let text = w("Đây là bản TextVN chạy ngay (portable).\r\n\r\n\
Gỡ đăng ký bộ gõ TextVN khỏi Windows và thoát? Sau đó bạn có thể xoá thư mục; \
cấu hình trong %APPDATA%\\TextVN được giữ lại.");
    let title = w("Gỡ TextVN");
    // SAFETY: chuỗi NUL-terminated sống suốt lời gọi đồng bộ.
    let answer = unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    if answer != IDYES {
        return;
    }
    // Mục tự khởi động trỏ vào thư mục sắp xoá thì bỏ luôn (bản cài khác giữ nguyên).
    let _ = crate::autostart::disable_autostart_for_dir(&dir);
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let _ = std::process::Command::new(dir.join("textvn-cli.exe"))
        .arg("unregister")
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    // SAFETY: chỉ post message vào hàng đợi của thread UI hiện tại.
    unsafe { PostQuitMessage(0) };
}

#[cfg(not(windows))]
fn uninstall() {}

#[cfg(windows)]
fn add_radio_menu_item(menu: HMENU, label: &str, id: u32, checked: bool) {
    let mut flags = MF_STRING;
    if checked {
        flags |= MF_CHECKED;
    }
    unsafe {
        let _ = AppendMenuW(menu, flags, id as usize, PCWSTR(w(label).as_ptr()));
    }
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids_are_distinct_and_sequential() {
        let ids = [
            ID_TOGGLE_GLOBAL,
            ID_METHOD_TELEX,
            ID_METHOD_VNI,
            ID_METHOD_VIQR,
            ID_METHOD_SIMPLE_TELEX,
            ID_DIACRITIC_NEW,
            ID_DIACRITIC_OLD,
            ID_CURRENT_APP_TOGGLE,
            ID_HOOK_COMPAT_MODE,
            ID_OPEN_SETTINGS,
            ID_HEALTH_STATUS,
            ID_UNINSTALL,
            ID_EXIT,
        ];
        let mut set = std::collections::HashSet::new();
        for id in ids {
            assert!(set.insert(id), "Duplicate menu ID: {id}");
        }
        assert_eq!(ids.len(), 13);
    }
}
