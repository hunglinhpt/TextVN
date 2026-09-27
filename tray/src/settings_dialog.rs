// SPDX-License-Identifier: GPL-3.0-or-later
//! Giao diện Bảng điều khiển kiểu UniKey/EVKey thuần Win32 (WIN-052 / P1-4 §3).
//!
//! Thiết kế native Win32 GUI:
//! - Khởi động tức thì (<5ms), không tốn RAM, không cần framework nặng.
//! - Bảng điều khiển chuẩn: Kiểu gõ, Bảng mã, Tùy chọn gõ, Bỏ dấu mới/cũ, Khởi động cùng Windows.
//! - Nút "Đóng" hoặc nút [X] tự động ẩn về khay (Minimize to Tray).
//! - Đồng bộ trạng thái tức thì với SvcManager và broadcast IPC reload.

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;

use vietime_config::{DiacriticStyle, Method, OutputCharset};

use crate::autostart;
use crate::ipc_server::IpcServer;
use crate::svc::SvcManager;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::*;
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

const SETTINGS_CLASS_NAME: &str = "VietIMESettingsDialogClass";

// Control IDs
const ID_COMBO_CHARSET: isize = 2001;
const ID_COMBO_METHOD: isize = 2002;
const ID_CHK_AUTO_RESTORE: isize = 2003;
const ID_CHK_FREE_MARKING: isize = 2004;
const ID_CHK_AUTOSTART: isize = 2005;
const ID_CHK_GLOBAL_ENABLED: isize = 2006;
const ID_RAD_DIACRITIC_NEW: isize = 2007;
const ID_RAD_DIACRITIC_OLD: isize = 2008;
const ID_BTN_CLOSE: isize = 2010;
const ID_BTN_DEFAULT: isize = 2011;
const ID_BTN_EXIT: isize = 2012;

// Win32 Button Styles & Messages
const BS_GROUPBOX: u32 = 0x00000007;
const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BS_AUTORADIOBUTTON: u32 = 0x00000009;
const BS_DEFPUSHBUTTON: u32 = 0x00000001;
const BS_PUSHBUTTON: u32 = 0x00000000;
const CBS_DROPDOWNLIST: u32 = 0x00000003;
const BM_SETCHECK: u32 = 0x00F1;
const BM_GETCHECK: u32 = 0x00F0;
const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;
const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CBN_SELCHANGE: u32 = 1;

static SETTINGS_HWND: AtomicIsize = AtomicIsize::new(0);

struct DialogContext {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
}

static DIALOG_CTX: std::sync::RwLock<Option<DialogContext>> = std::sync::RwLock::new(None);

fn with_ctx<R>(f: impl FnOnce(&DialogContext) -> R) -> Option<R> {
    DIALOG_CTX.read().ok()?.as_ref().map(f)
}

/// Hiển thị cửa sổ Bảng điều khiển VietIME (nếu đang ẩn thì hiện và đưa lên trước).
#[cfg(windows)]
pub fn show_settings_dialog(svc: Arc<SvcManager>, ipc: Arc<IpcServer>) {
    let existing_raw = SETTINGS_HWND.load(Ordering::Acquire);
    if existing_raw != 0 {
        let hwnd = HWND(existing_raw as *mut std::ffi::c_void);
        if unsafe { IsWindow(Some(hwnd)) }.as_bool() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
            return;
        }
    }

    if let Ok(mut lock) = DIALOG_CTX.write() {
        *lock = Some(DialogContext { svc, ipc });
    }

    create_and_show_window();
}

#[cfg(not(windows))]
pub fn show_settings_dialog(_svc: Arc<SvcManager>, _ipc: Arc<IpcServer>) {
    println!("Cài đặt VietIME chỉ khả dụng trên Windows.");
}

#[cfg(windows)]
fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(windows)]
fn create_and_show_window() {
    let class_name = w(SETTINGS_CLASS_NAME);
    let title = w("VietIME - Bảng điều khiển");
    let h_instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };

    let wc = WNDCLASSW {
        lpfnWndProc: Some(dialog_wnd_proc),
        hInstance: h_instance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hbrBackground: HBRUSH((COLOR_BTNFACE.0 + 1) as *mut std::ffi::c_void),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        ..Default::default()
    };

    let _ = unsafe { RegisterClassW(&wc) };

    // Kích thước chuẩn gọn gàng kiểu UniKey: 440 x 360
    let width = 450;
    let height = 370;

    // Canh giữa màn hình
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let x = (screen_w - width) / 2;
    let y = (screen_h - height) / 2;

    let hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(h_instance.into()),
            None,
        )
    } {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to create settings window: {e}");
            return;
        }
    };

    SETTINGS_HWND.store(hwnd.0 as isize, Ordering::Release);

    create_dialog_controls(hwnd, h_instance.into());
    populate_controls_from_config(hwnd);

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
fn create_control(
    class: &str,
    text: &str,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    parent: HWND,
    id: isize,
    h_instance: HINSTANCE,
) -> HWND {
    let class_w = self::w(class);
    let text_w = self::w(text);
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_w.as_ptr()),
            PCWSTR(text_w.as_ptr()),
            WINDOW_STYLE(style | WS_CHILD.0 | WS_VISIBLE.0),
            x,
            y,
            w,
            h,
            Some(parent),
            Some(HMENU(id as *mut std::ffi::c_void)),
            Some(h_instance),
            None,
        )
        .unwrap_or_default()
    }
}

#[cfg(windows)]
fn create_dialog_controls(parent: HWND, h_instance: HINSTANCE) {
    let default_font = unsafe { GetStockObject(DEFAULT_GUI_FONT) };

    // 1. Nhóm Điều khiển
    let gb1 = create_control(
        "BUTTON",
        "Điều khiển",
        BS_GROUPBOX,
        15,
        10,
        405,
        95,
        parent,
        0,
        h_instance,
    );
    let lbl_charset = create_control(
        "STATIC",
        "Bảng mã:",
        0,
        30,
        35,
        75,
        20,
        parent,
        0,
        h_instance,
    );
    let cb_charset = create_control(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
        110,
        32,
        290,
        150,
        parent,
        ID_COMBO_CHARSET,
        h_instance,
    );
    let lbl_method = create_control(
        "STATIC",
        "Kiểu gõ:",
        0,
        30,
        68,
        75,
        20,
        parent,
        0,
        h_instance,
    );
    let cb_method = create_control(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
        110,
        65,
        290,
        150,
        parent,
        ID_COMBO_METHOD,
        h_instance,
    );

    // 2. Nhóm Tùy chọn gõ
    let gb2 = create_control(
        "BUTTON",
        "Tùy chọn",
        BS_GROUPBOX,
        15,
        115,
        405,
        140,
        parent,
        0,
        h_instance,
    );
    let chk_restore = create_control(
        "BUTTON",
        "Khôi phục từ tiếng Anh khi gõ sai",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        30,
        138,
        230,
        20,
        parent,
        ID_CHK_AUTO_RESTORE,
        h_instance,
    );
    let chk_free = create_control(
        "BUTTON",
        "Đặt dấu tự do",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        30,
        163,
        230,
        20,
        parent,
        ID_CHK_FREE_MARKING,
        h_instance,
    );
    let chk_auto = create_control(
        "BUTTON",
        "Khởi động cùng Windows",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        30,
        188,
        230,
        20,
        parent,
        ID_CHK_AUTOSTART,
        h_instance,
    );
    let chk_global = create_control(
        "BUTTON",
        "Bật gõ tiếng Việt",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        30,
        213,
        230,
        20,
        parent,
        ID_CHK_GLOBAL_ENABLED,
        h_instance,
    );

    let rad_new = create_control(
        "BUTTON",
        "Dấu mới (hoà, thuỷ)",
        BS_AUTORADIOBUTTON | WS_TABSTOP.0,
        270,
        142,
        140,
        20,
        parent,
        ID_RAD_DIACRITIC_NEW,
        h_instance,
    );
    let rad_old = create_control(
        "BUTTON",
        "Dấu cũ (hòa, thủy)",
        BS_AUTORADIOBUTTON,
        270,
        168,
        140,
        20,
        parent,
        ID_RAD_DIACRITIC_OLD,
        h_instance,
    );

    let _lbl_shortcut_title = create_control(
        "STATIC",
        "Phím chuyển:",
        0,
        270,
        196,
        140,
        18,
        parent,
        0,
        h_instance,
    );
    let _lbl_shortcut_val = create_control(
        "STATIC",
        "[ Ctrl + Shift ]",
        0,
        270,
        214,
        140,
        20,
        parent,
        0,
        h_instance,
    );

    // 3. Nút hành động
    let btn_close = create_control(
        "BUTTON",
        "Đóng",
        BS_DEFPUSHBUTTON | WS_TABSTOP.0,
        135,
        275,
        85,
        30,
        parent,
        ID_BTN_CLOSE,
        h_instance,
    );
    let btn_default = create_control(
        "BUTTON",
        "Mặc định",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        230,
        275,
        85,
        30,
        parent,
        ID_BTN_DEFAULT,
        h_instance,
    );
    let btn_exit = create_control(
        "BUTTON",
        "Kết thúc",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        325,
        275,
        85,
        30,
        parent,
        ID_BTN_EXIT,
        h_instance,
    );

    // Gán font chuẩn Windows cho toàn bộ controls
    let controls = [
        gb1,
        lbl_charset,
        cb_charset,
        lbl_method,
        cb_method,
        gb2,
        chk_restore,
        chk_free,
        chk_auto,
        chk_global,
        rad_new,
        rad_old,
        btn_close,
        btn_default,
        btn_exit,
    ];
    for &ctrl in &controls {
        unsafe {
            let _ = SendMessageW(
                ctrl,
                WM_SETFONT,
                Some(WPARAM(default_font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
    }

    // Thêm các mục vào ComboBox Bảng mã
    let charsets = [
        "Unicode dựng sẵn (Precomposed)",
        "Unicode tổ hợp (Decomposed)",
        "TCVN3 (ABC)",
        "VNI Windows",
    ];
    for cs in charsets {
        let text_w = self::w(cs);
        unsafe {
            let _ = SendMessageW(
                cb_charset,
                CB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(text_w.as_ptr() as isize)),
            );
        }
    }

    // Thêm các mục vào ComboBox Kiểu gõ
    let methods = ["Telex", "VNI", "VIQR", "Simple Telex"];
    for m in methods {
        let text_w = self::w(m);
        unsafe {
            let _ = SendMessageW(
                cb_method,
                CB_ADDSTRING,
                Some(WPARAM(0)),
                Some(LPARAM(text_w.as_ptr() as isize)),
            );
        }
    }
}

#[cfg(windows)]
fn populate_controls_from_config(hwnd: HWND) {
    with_ctx(|ctx| {
        let cfg = ctx.svc.config();

        // Kiểu gõ
        let method_idx = match cfg.method {
            Method::Telex => 0,
            Method::Vni => 1,
            Method::Viqr => 2,
            Method::SimpleTelex => 3,
        };
        send_dlg_msg(hwnd, ID_COMBO_METHOD, CB_SETCURSEL, method_idx, 0);

        // Bảng mã
        let charset_idx = match cfg.output_charset {
            OutputCharset::UnicodePrecomposed => 0,
            OutputCharset::UnicodeDecomposed => 1,
            OutputCharset::Tcvn3 => 2,
            OutputCharset::VniWindows => 3,
        };
        send_dlg_msg(hwnd, ID_COMBO_CHARSET, CB_SETCURSEL, charset_idx, 0);

        // Checkboxes
        set_chk(hwnd, ID_CHK_AUTO_RESTORE, cfg.auto_restore_english);
        set_chk(hwnd, ID_CHK_FREE_MARKING, cfg.free_marking);
        set_chk(
            hwnd,
            ID_CHK_AUTOSTART,
            autostart::is_autostart_enabled().unwrap_or(false),
        );
        set_chk(hwnd, ID_CHK_GLOBAL_ENABLED, ctx.svc.is_global_enabled());

        // Diacritic radio
        let is_new = cfg.diacritic_style == DiacriticStyle::New;
        set_chk(hwnd, ID_RAD_DIACRITIC_NEW, is_new);
        set_chk(hwnd, ID_RAD_DIACRITIC_OLD, !is_new);
    });
}

#[cfg(windows)]
fn send_dlg_msg(parent: HWND, id: isize, msg: u32, wparam: usize, lparam: isize) -> LRESULT {
    unsafe {
        let ctrl = GetDlgItem(Some(parent), id as i32).unwrap_or_default();
        SendMessageW(ctrl, msg, Some(WPARAM(wparam)), Some(LPARAM(lparam)))
    }
}

#[cfg(windows)]
fn set_chk(parent: HWND, id: isize, checked: bool) {
    let val = if checked { BST_CHECKED } else { BST_UNCHECKED };
    send_dlg_msg(parent, id, BM_SETCHECK, val, 0);
}

#[cfg(windows)]
fn get_chk(parent: HWND, id: isize) -> bool {
    let res = send_dlg_msg(parent, id, BM_GETCHECK, 0, 0);
    res.0 == BST_CHECKED as isize
}

#[cfg(windows)]
unsafe extern "system" fn dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let cmd_id = (wparam.0 & 0xffff) as isize;
            let notify_code = ((wparam.0 >> 16) & 0xffff) as u32;

            match cmd_id {
                ID_BTN_CLOSE => {
                    // Nút Đóng: Thu nhỏ về khay (Minimize to Tray)
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    return LRESULT(0);
                }
                ID_BTN_EXIT => {
                    // Nút Kết thúc: Đóng hẳn ứng dụng
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
                ID_BTN_DEFAULT => {
                    with_ctx(|ctx| {
                        ctx.svc.set_method(Method::Telex);
                        ctx.svc.set_diacritic_style(DiacriticStyle::New);
                        ctx.svc
                            .set_output_charset(OutputCharset::UnicodePrecomposed);
                        populate_controls_from_config(hwnd);
                        ctx.ipc.broadcast_config_reload(1);
                    });
                    crate::notify_tray_state_changed();
                    return LRESULT(0);
                }
                ID_COMBO_METHOD if notify_code == CBN_SELCHANGE => {
                    let sel = send_dlg_msg(hwnd, ID_COMBO_METHOD, CB_GETCURSEL, 0, 0).0;
                    let m = match sel {
                        1 => Method::Vni,
                        2 => Method::Viqr,
                        3 => Method::SimpleTelex,
                        _ => Method::Telex,
                    };
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_method(m);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }
                ID_COMBO_CHARSET if notify_code == CBN_SELCHANGE => {
                    let sel = send_dlg_msg(hwnd, ID_COMBO_CHARSET, CB_GETCURSEL, 0, 0).0;
                    let cs = match sel {
                        1 => OutputCharset::UnicodeDecomposed,
                        2 => OutputCharset::Tcvn3,
                        3 => OutputCharset::VniWindows,
                        _ => OutputCharset::UnicodePrecomposed,
                    };
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_output_charset(cs);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }

                ID_CHK_AUTO_RESTORE => {
                    let checked = get_chk(hwnd, ID_CHK_AUTO_RESTORE);
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_auto_restore_english(checked);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }
                ID_CHK_FREE_MARKING => {
                    let checked = get_chk(hwnd, ID_CHK_FREE_MARKING);
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_free_marking(checked);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }
                ID_CHK_AUTOSTART => {
                    let checked = get_chk(hwnd, ID_CHK_AUTOSTART);
                    if checked {
                        let _ = autostart::enable_autostart(None);
                    } else {
                        let _ = autostart::disable_autostart();
                    }
                }
                ID_CHK_GLOBAL_ENABLED => {
                    let checked = get_chk(hwnd, ID_CHK_GLOBAL_ENABLED);
                    with_ctx(|ctx| {
                        let (_, ver) = ctx.svc.set_global_enabled(checked);
                        ctx.ipc.broadcast_state_update("*", checked, ver);
                    });
                    crate::notify_tray_state_changed();
                }
                ID_RAD_DIACRITIC_NEW => {
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_diacritic_style(DiacriticStyle::New);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }
                ID_RAD_DIACRITIC_OLD => {
                    with_ctx(|ctx| {
                        let ver = ctx.svc.set_diacritic_style(DiacriticStyle::Old);
                        ctx.ipc.broadcast_config_reload(ver);
                    });
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // Nút [X] góc trên bên phải: Ẩn về khay hệ thống (Minimize to Tray) thay vì thoát
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_DESTROY => {
            SETTINGS_HWND.store(0, Ordering::Release);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_ids_are_distinct() {
        let ids = [
            ID_COMBO_CHARSET,
            ID_COMBO_METHOD,
            ID_CHK_AUTO_RESTORE,
            ID_CHK_FREE_MARKING,
            ID_CHK_AUTOSTART,
            ID_CHK_GLOBAL_ENABLED,
            ID_RAD_DIACRITIC_NEW,
            ID_RAD_DIACRITIC_OLD,
            ID_BTN_CLOSE,
            ID_BTN_DEFAULT,
            ID_BTN_EXIT,
        ];
        let mut set = std::collections::HashSet::new();
        for id in ids {
            assert!(set.insert(id), "Duplicate dialog control ID: {id}");
        }
        assert_eq!(ids.len(), 11);
    }
}
