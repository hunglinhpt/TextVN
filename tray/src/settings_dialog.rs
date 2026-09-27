// SPDX-License-Identifier: GPL-3.0-or-later
//! Giao diện Bảng điều khiển kiểu UniKey/EVKey thuần Win32 (WIN-052 / P1-4 §3).
//!
//! Thiết kế native Win32 GUI:
//! - Khởi động tức thì (<5ms), không tốn RAM, không cần framework nặng.
//! - Bảng điều khiển chuẩn: Kiểu gõ, Bảng mã, Tùy chọn gõ, Bỏ dấu mới/cũ, Khởi động cùng Windows.
//! - Nút "Đóng" hoặc nút [X] tự động ẩn về khay (Minimize to Tray).
//! - Đồng bộ trạng thái tức thì với SvcManager và broadcast IPC reload.

use std::sync::atomic::AtomicIsize;
#[cfg(windows)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[cfg(windows)]
use textvn_config::{DiacriticStyle, Method, OutputCharset};

#[cfg(windows)]
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

const SETTINGS_CLASS_NAME: &str = "TextVNSettingsDialogClass";

/// Kích thước cơ sở Bảng điều khiển @96 DPI (pixel thật vì manifest là
/// PerMonitorV2): compact 600x367 @96 DPI, khoảng 2/3 bản trước.
/// Toàn bộ tọa độ layout tính trên cơ sở này rồi nhân DPI-scale lúc tạo.
const DIALOG_BASE_WIDTH: i32 = 600;
const DIALOG_BASE_HEIGHT: i32 = 367;
const COMPACT_SCALE: f64 = 2.0 / 3.0;

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
const ID_BTN_HELP: isize = 2013;
const ID_BTN_ABOUT: isize = 2014;
const ID_BTN_SETUP_TSF: isize = 2015;

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

/// Hiển thị cửa sổ Bảng điều khiển TextVN (nếu đang ẩn thì hiện và đưa lên trước).
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
    println!("Cài đặt TextVN chỉ khả dụng trên Windows.");
}

#[cfg(windows)]
fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// DPI hiện tại (pixel/inch). Manifest PerMonitorV2 nên LOGPIXELSX trả DPI
/// thật; fallback 96 nếu không lấy được DC.
#[cfg(windows)]
fn system_dpi() -> i32 {
    unsafe {
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return 96;
        }
        let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSX);
        let _ = ReleaseDC(None, hdc);
        if dpi <= 0 {
            96
        } else {
            dpi
        }
    }
}

/// Tọa độ (x, y) để cửa sổ `width`x`height` nằm giữa vùng làm việc của
/// màn hình chính (tránh thanh tác vụ). Lấy work area qua
/// SystemParametersInfoW(SPI_GETWORKAREA), fallback về tâm màn hình chính.
#[cfg(windows)]
fn center_on_work_area(width: i32, height: i32) -> (i32, i32) {
    let mut wa: RECT = unsafe { std::mem::zeroed() };
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut wa as *mut RECT as *mut std::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    let (cx, cy) = match ok {
        Ok(()) => ((wa.left + wa.right) / 2, (wa.top + wa.bottom) / 2),
        Err(_) => unsafe {
            (
                GetSystemMetrics(SM_CXSCREEN) / 2,
                GetSystemMetrics(SM_CYSCREEN) / 2,
            )
        },
    };
    (cx - width / 2, cy - height / 2)
}

#[cfg(windows)]
fn create_and_show_window() {
    let class_name = w(SETTINGS_CLASS_NAME);
    let title = w("TextVN - Bảng điều khiển");
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

    // Kích thước compact theo DPI: 600x367 @96 DPI nhân hệ số DPI hệ thống
    // (PerMonitorV2 -> tọa độ pixel thật, Windows không scale hộ).
    let dpi = system_dpi();
    let scale = dpi as f64 / 96.0;
    let width = ((DIALOG_BASE_WIDTH as f64) * scale).round() as i32;
    let height = ((DIALOG_BASE_HEIGHT as f64) * scale).round() as i32;

    // Canh giữa vùng làm việc (không bị thanh tác vụ che)
    let (x, y) = center_on_work_area(width, height);

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

    create_dialog_controls(hwnd, h_instance.into(), dpi);
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

/// Font GUI 9pt theo DPI hiện tại. Đây là cỡ chữ chuẩn, dễ đọc cho dialog
/// compact; không dùng cỡ 7pt vì bị quá nhỏ ở màn hình mật độ cao.
/// Với PerMonitorV2 cần tạo font theo DPI thay vì dựa vào stock font.
/// theo DPI. Font sống trong static để xóa khi dialog destroy (tránh leak
/// GDI handle); nếu tạo thất bại thì dùng lại stock font.
#[cfg(windows)]
static UI_FONT: AtomicIsize = AtomicIsize::new(0);

#[cfg(windows)]
fn scaled_gui_font(dpi: i32) -> HGDIOBJ {
    let new_font = unsafe {
        let stock = GetStockObject(DEFAULT_GUI_FONT);
        let mut lf: LOGFONTW = std::mem::zeroed();
        let got = GetObjectW(
            stock,
            std::mem::size_of::<LOGFONTW>() as i32,
            Some(&mut lf as *mut LOGFONTW as *mut std::ffi::c_void),
        );
        if got == 0 {
            stock
        } else {
            lf.lfHeight = -(9 * dpi / 72); // 9pt tại DPI hiện tại
            let f = CreateFontIndirectW(&lf);
            if f.is_invalid() {
                stock
            } else {
                HGDIOBJ(f.0)
            }
        }
    };
    if new_font.is_invalid() {
        return unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    }
    // Đóng font cũ (nếu có) để không rò rỉ handle giữa các lần mở dialog.
    let old = UI_FONT.swap(new_font.0 as isize, Ordering::AcqRel);
    if old != 0 && old != new_font.0 as isize {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(old as *mut std::ffi::c_void));
        }
    }
    new_font
}

#[cfg(windows)]
fn create_dialog_controls(parent: HWND, h_instance: HINSTANCE, dpi: i32) {
    let scale = dpi as f64 / 96.0;
    // Thu toàn bộ bố cục 900x550 cũ xuống đúng 2/3 rồi scale DPI.
    let k = |v: i32| -> i32 { ((v as f64) * COMPACT_SCALE * scale).round() as i32 };

    let default_font = scaled_gui_font(dpi);

    // 1. Nhóm Điều khiển: 2 cột song song (Bảng mã | Kiểu gõ), tận dụng bề rộng 900
    let gb1 = create_control(
        "BUTTON",
        "Điều khiển",
        BS_GROUPBOX,
        k(15),
        k(12),
        k(860),
        k(95),
        parent,
        0,
        h_instance,
    );
    let lbl_charset = create_control(
        "STATIC",
        "Bảng mã:",
        0,
        k(32),
        k(49),
        k(85),
        k(30),
        parent,
        0,
        h_instance,
    );
    let cb_charset = create_control(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
        k(122),
        k(46),
        k(300),
        k(200),
        parent,
        ID_COMBO_CHARSET,
        h_instance,
    );
    let lbl_method = create_control(
        "STATIC",
        "Kiểu gõ:",
        0,
        k(470),
        k(49),
        k(85),
        k(30),
        parent,
        0,
        h_instance,
    );
    let cb_method = create_control(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
        k(560),
        k(46),
        k(300),
        k(200),
        parent,
        ID_COMBO_METHOD,
        h_instance,
    );

    // 2. Nhóm Tùy chọn gõ: 4 checkbox cột trái, radio + phím tắt cột phải
    let gb2 = create_control(
        "BUTTON",
        "Tùy chọn gõ",
        BS_GROUPBOX,
        k(15),
        k(122),
        k(860),
        k(240),
        parent,
        0,
        h_instance,
    );
    let chk_restore = create_control(
        "BUTTON",
        "Khôi phục từ tiếng Anh khi gõ sai",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        k(32),
        k(158),
        k(420),
        k(32),
        parent,
        ID_CHK_AUTO_RESTORE,
        h_instance,
    );
    let chk_free = create_control(
        "BUTTON",
        "Đặt dấu tự do",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        k(32),
        k(205),
        k(420),
        k(32),
        parent,
        ID_CHK_FREE_MARKING,
        h_instance,
    );
    let chk_auto = create_control(
        "BUTTON",
        "Khởi động cùng Windows",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        k(32),
        k(252),
        k(420),
        k(32),
        parent,
        ID_CHK_AUTOSTART,
        h_instance,
    );
    let chk_global = create_control(
        "BUTTON",
        "Bật gõ tiếng Việt",
        BS_AUTOCHECKBOX | WS_TABSTOP.0,
        k(32),
        k(299),
        k(420),
        k(32),
        parent,
        ID_CHK_GLOBAL_ENABLED,
        h_instance,
    );

    let rad_new = create_control(
        "BUTTON",
        "Dấu mới (hoà, thuỷ)",
        BS_AUTORADIOBUTTON | WS_TABSTOP.0,
        k(510),
        k(163),
        k(350),
        k(32),
        parent,
        ID_RAD_DIACRITIC_NEW,
        h_instance,
    );
    let rad_old = create_control(
        "BUTTON",
        "Dấu cũ (hòa, thủy)",
        BS_AUTORADIOBUTTON,
        k(510),
        k(210),
        k(350),
        k(32),
        parent,
        ID_RAD_DIACRITIC_OLD,
        h_instance,
    );

    let lbl_shortcut_title = create_control(
        "STATIC",
        "Phím chuyển:",
        0,
        k(510),
        k(262),
        k(350),
        k(30),
        parent,
        0,
        h_instance,
    );
    let lbl_shortcut_val = create_control(
        "STATIC",
        "[ Ctrl + Shift ]",
        0,
        k(510),
        k(286),
        k(350),
        k(32),
        parent,
        0,
        h_instance,
    );

    // 3. Nút hành động. Hàng chân cửa sổ: nhóm thông tin (trái),
    //    nhóm thao tác (phải) - không phải tìm trong menu tray.
    let btn_help = create_control(
        "BUTTON",
        "Hướng dẫn",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        k(25),
        k(440),
        k(130),
        k(48),
        parent,
        ID_BTN_HELP,
        h_instance,
    );
    let btn_about = create_control(
        "BUTTON",
        "Thông tin",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        k(165),
        k(440),
        k(130),
        k(48),
        parent,
        ID_BTN_ABOUT,
        h_instance,
    );
    let btn_setup_tsf = create_control(
        "BUTTON",
        "Cài & bật TSF",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        k(305),
        k(440),
        k(170),
        k(48),
        parent,
        ID_BTN_SETUP_TSF,
        h_instance,
    );
    // Nút Đóng đưa cửa sổ về khay; Kết thúc tắt hẳn ứng dụng.
    let btn_close = create_control(
        "BUTTON",
        "Đóng",
        BS_DEFPUSHBUTTON | WS_TABSTOP.0,
        k(655),
        k(440),
        k(115),
        k(48),
        parent,
        ID_BTN_CLOSE,
        h_instance,
    );
    let btn_default = create_control(
        "BUTTON",
        "Mặc định",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        k(530),
        k(440),
        k(115),
        k(48),
        parent,
        ID_BTN_DEFAULT,
        h_instance,
    );
    let btn_exit = create_control(
        "BUTTON",
        "Kết thúc",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        k(765),
        k(440),
        k(105),
        k(48),
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
        lbl_shortcut_title,
        lbl_shortcut_val,
        btn_help,
        btn_about,
        btn_setup_tsf,
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
                ID_BTN_HELP => {
                    show_information(
                        hwnd,
                        "Hướng dẫn TextVN",
                        "Chọn kiểu gõ và bảng mã, sau đó bật/tắt tiếng Việt từ khay hệ thống.\r\n\r\nNếu đây là lần dùng đầu tiên hoặc chưa gõ được, bấm [Cài & bật TSF].\r\nPhím chuyển mặc định: Ctrl + Shift.\r\nTextVN không yêu cầu mở cửa sổ Terminal.",
                    );
                    return LRESULT(0);
                }
                ID_BTN_ABOUT => {
                    show_information(
                        hwnd,
                        "Thông tin TextVN",
                        "TextVN — Bộ gõ tiếng Việt cho Windows\r\n\r\nPhát triển bởi: hunglinhpt\r\nGiấy phép: GPL-3.0-or-later\r\nhttps://github.com/hunglinhpt/TextVN",
                    );
                    return LRESULT(0);
                }
                ID_BTN_SETUP_TSF => {
                    if register_and_activate_tsf() {
                        show_information(
                            hwnd,
                            "TextVN TSF",
                            "Đã đăng ký và kích hoạt bộ gõ TextVN.\r\n\r\nHãy thử gõ Telex trong Notepad. Nếu vẫn chưa hoạt động, kiểm tra phần mềm bảo mật hoặc quyền ghi HKCU của tài khoản Windows.",
                        );
                    } else {
                        show_information(
                            hwnd,
                            "Không thể đăng ký TextVN TSF",
                            "Windows đã từ chối đăng ký bộ gõ cho tài khoản hiện tại. TextVN không thể nhận phím cho đến khi TSF được đăng ký.\r\n\r\nKiểm tra quyền ghi HKCU\\Software\\Classes\\CLSID và chính sách phần mềm bảo mật, sau đó bấm [Cài & bật TSF] lại.",
                        );
                    }
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
            // Controls con đã bị hủy trước khi parent nhận WM_DESTROY
            // nên font không còn được tham chiếu - an toàn để xóa.
            let font = UI_FONT.swap(0, Ordering::AcqRel);
            if font != 0 {
                unsafe {
                    let _ = DeleteObject(HGDIOBJ(font as *mut std::ffi::c_void));
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
fn show_information(owner: HWND, title: &str, content: &str) {
    let title = w(title);
    let content = w(content);
    // SAFETY: buffers are NUL-terminated and live for this synchronous call.
    unsafe {
        let _ = MessageBoxW(
            Some(owner),
            PCWSTR(content.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

/// Đăng ký TIP theo user từ chính dialog để lỗi quyền hiện rõ trong UI thay vì
/// thất bại im lặng. CLI là thành phần cùng gói và được chạy ẩn.
#[cfg(windows)]
fn register_and_activate_tsf() -> bool {
    let Ok(mut cli_path) = std::env::current_exe() else {
        return false;
    };
    cli_path.set_file_name("textvn-cli.exe");
    if !cli_path.is_file() {
        return false;
    }

    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new(cli_path)
        .arg("register")
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|status| status.success())
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
            ID_BTN_HELP,
            ID_BTN_ABOUT,
            ID_BTN_SETUP_TSF,
        ];
        let mut set = std::collections::HashSet::new();
        for id in ids {
            assert!(set.insert(id), "Duplicate dialog control ID: {id}");
        }
        assert_eq!(ids.len(), 14);
    }
}
