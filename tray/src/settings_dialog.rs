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
use textvn_config::{DiacriticStyle, DocError, MacroTrigger, Method, OutputCharset};

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
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetDpiForWindow};
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

const SETTINGS_CLASS_NAME: &str = "TextVNSettingsDialogClass";

/// Bố cục được thiết kế trên lưới 900 đơn vị ngang, thu 2/3 rồi nhân DPI khi tạo
/// (manifest PerMonitorV2 → tọa độ là pixel thật, Windows không scale hộ).
/// Kích thước dưới đây là **vùng client**; khung cửa sổ tính bằng AdjustWindowRectEx
/// để thanh tiêu đề dày/mỏng (theme, DPI) không cắt mất hàng nút cuối.
const CLIENT_DESIGN_WIDTH: i32 = 900;
const CLIENT_DESIGN_HEIGHT: i32 = 560;
const MACRO_DESIGN_WIDTH: i32 = 720;
const MACRO_DESIGN_HEIGHT: i32 = 560;
const COMPACT_SCALE: f64 = 2.0 / 3.0;

// Control IDs — bảng điều khiển
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
const ID_CHK_AUTO_CAPITALIZE: isize = 2016;
const ID_CHK_QUICK_TELEX: isize = 2017;
const ID_CHK_MACRO_WHEN_OFF: isize = 2018;
const ID_CHK_SHOW_ON_STARTUP: isize = 2019;
const ID_BTN_MACROS: isize = 2020;
const ID_CHK_CTRL_SHIFT: isize = 2021;
const ID_BTN_ENGLISH: isize = 2022;
const ID_LBL_WORDLIST_HINT: isize = 4101;
// ID cho STATIC/GROUPBOX (không tương tác) — cần để re-layout theo DPI (WM_DPICHANGED).
const ID_LBL_BASE: isize = 3000;
// Control IDs — cửa sổ Gõ tắt
const ID_EDIT_MACROS: isize = 2101;
const ID_LBL_MACRO_HINT: isize = 4001;
const ID_LBL_MACRO_TRIGGER: isize = 4002;
const ID_RAD_TRIGGER_TAB: isize = 2102;
const ID_RAD_TRIGGER_SPACE: isize = 2103;
const ID_BTN_MACRO_SAVE: isize = 2104;
const ID_BTN_MACRO_CANCEL: isize = 2105;
/// IsDialogMessage gửi IDCANCEL khi nhấn Esc.
const IDCANCEL_CMD: isize = 2;

// Win32 Button/Edit Styles & Messages
const BS_GROUPBOX: u32 = 0x00000007;
const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BS_AUTORADIOBUTTON: u32 = 0x00000009;
const BS_DEFPUSHBUTTON: u32 = 0x00000001;
const BS_PUSHBUTTON: u32 = 0x00000000;
const CBS_DROPDOWNLIST: u32 = 0x00000003;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_NOHIDESEL: u32 = 0x0100;
const ES_WANTRETURN: u32 = 0x1000;
const BM_SETCHECK: u32 = 0x00F1;
const BM_GETCHECK: u32 = 0x00F0;
const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;
const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CBN_SELCHANGE: u32 = 1;
const EM_SETSEL: u32 = 0x00B1;
const EM_SCROLLCARET: u32 = 0x00B7;
const EM_LINEINDEX: u32 = 0x00BB;
const EM_LINELENGTH: u32 = 0x00C1;
const EM_SETLIMITTEXT: u32 = 0x00C5;

/// Nhãn hiển thị — **giống hệt** bảng cài đặt Linux (`adapters/linux-settings`);
/// đổi ở đây phải đổi cả bên đó và `docs/release/ui-spec.md`.
const CHARSET_LABELS: [&str; 4] = [
    "Unicode dựng sẵn",
    "Unicode tổ hợp",
    "TCVN3 (ABC)",
    "VNI Windows",
];
const METHOD_LABELS: [&str; 4] = ["Telex", "VNI", "VIQR", "Telex đơn giản"];

const HELP_TEXT: &str = "Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (không kèm phím khác), \
hoặc Ctrl + Shift + Space, hoặc bấm biểu tượng V/E ở khay hệ thống.\r\n\r\n\
Telex: aa=â  aw=ă  ee=ê  oo=ô  ow=ơ  uw=ư  dd=đ;  s f r x j = sắc huyền hỏi ngã nặng;  z = xoá dấu.\r\n\
VNI: 1-5 = sắc huyền hỏi ngã nặng;  6 = mũ (â ê ô);  7 = móc (ơ ư);  8 = trăng (ă);  9 = đ;  0 = xoá dấu.\r\n\
Quick Telex: cc=ch  gg=gi  kk=kh  nn=ng  qq=qu  pp=ph  tt=th.\r\n\r\n\
Gõ tắt: bấm [Gõ tắt...], mỗi dòng ghi  gõ tắt = nội dung; khi gõ, nhập chữ tắt rồi nhấn Tab \
(hoặc Space) để bung ra.\r\n\r\n\
Nếu chưa gõ được tiếng Việt: bấm [Cài & bật TSF], rồi chọn TextVN trong danh sách bộ gõ \
(Win + Space).";

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

/// DPI để ước lượng TRƯỚC khi có cửa sổ. Dùng `GetDpiForSystem` (thật theo
/// phiên PerMonitorV2); `GetDeviceCaps(LOGPIXELSX)` chỉ là fallback vì trên
/// máy có scaling theo màn hình nó trả system-DPI CŨ (96) — nguồn của lỗi
/// "dialog mở trên màn 200%, hình học tính cho 96% → chữ to gấp đôi, bị cắt"
/// (báo cáo chủ repo 2026-10-03, B9/B10).
#[cfg(windows)]
fn system_dpi() -> i32 {
    let dpi = unsafe { GetDpiForSystem() } as i32;
    if dpi > 0 {
        return dpi;
    }
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

/// DPI thật của cửa sổ đang nằm trên màn hình nào — nguồn chuẩn để layout
/// control SAU khi cửa sổ được tạo (PerMonitorV2: DPI theo monitor).
#[cfg(windows)]
fn window_dpi(hwnd: HWND) -> i32 {
    let dpi = unsafe { GetDpiForWindow(hwnd) } as i32;
    if dpi > 0 {
        dpi
    } else {
        system_dpi()
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

/// Kích thước khung cửa sổ (pixel thật) cho vùng client thiết kế `design_w`x`design_h`.
#[cfg(windows)]
fn window_size_for_client(
    design_w: i32,
    design_h: i32,
    dpi: i32,
    style: WINDOW_STYLE,
) -> (i32, i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: (design_w as f64 * scale).round() as i32,
        bottom: (design_h as f64 * scale).round() as i32,
    };
    // SAFETY: rc là biến cục bộ hợp lệ trong suốt lời gọi.
    let _ = unsafe { AdjustWindowRectEx(&mut rc, style, false, WINDOW_EX_STYLE::default()) };
    (rc.right - rc.left, rc.bottom - rc.top)
}

#[cfg(windows)]
fn register_class(name: &[u16], proc: WNDPROC, h_instance: HINSTANCE) {
    let wc = WNDCLASSW {
        lpfnWndProc: proc,
        hInstance: h_instance,
        lpszClassName: PCWSTR(name.as_ptr()),
        hbrBackground: HBRUSH((COLOR_BTNFACE.0 + 1) as *mut std::ffi::c_void),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        // Icon title bar / Alt-Tab: trước đây thiếu → cửa sổ dùng icon Windows
        // mặc định thay vì icon TextVN (Windows có sẵn qua tray + PE resource).
        hIcon: crate::icons::load_app_icon(h_instance, crate::icons::IDI_ICON_T, "textvn.ico"),
        ..Default::default()
    };
    // Đăng ký lần hai (mở lại dialog) trả lỗi "class đã tồn tại" — vô hại.
    let _ = unsafe { RegisterClassW(&wc) };
}

#[cfg(windows)]
fn create_and_show_window() {
    let class_name = w(SETTINGS_CLASS_NAME);
    let title = w("TextVN - Bảng điều khiển");
    let h_instance: HINSTANCE = unsafe { GetModuleHandleW(None).unwrap_or_default() }.into();
    register_class(&class_name, Some(dialog_wnd_proc), h_instance);

    let dpi = system_dpi();
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let (width, height) =
        window_size_for_client(CLIENT_DESIGN_WIDTH, CLIENT_DESIGN_HEIGHT, dpi, style);
    // Canh giữa vùng làm việc (không bị thanh tác vụ che)
    let (x, y) = center_on_work_area(width, height);

    let hwnd = match unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(h_instance),
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

    // DPI theo MONITOR thật của cửa sổ (khác ước lượng system khi máy có
    // nhiều màn hình scaling khác nhau): resize lại đúng cỡ rồi mới layout
    // control — chống tràn/cắt chữ (B9/B10).
    let win_dpi = window_dpi(hwnd);
    if win_dpi != dpi {
        let (width, height) =
            window_size_for_client(CLIENT_DESIGN_WIDTH, CLIENT_DESIGN_HEIGHT, win_dpi, style);
        let (x, y) = center_on_work_area(width, height);
        let _ = unsafe {
            SetWindowPos(
                HWND(hwnd.0),
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
    create_dialog_controls(hwnd, h_instance, win_dpi);
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
    create_control_ex(
        WINDOW_EX_STYLE::default(),
        class,
        text,
        style,
        (x, y, w, h),
        parent,
        id,
        h_instance,
    )
}

#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
fn create_control_ex(
    ex_style: WINDOW_EX_STYLE,
    class: &str,
    text: &str,
    style: u32,
    (x, y, w, h): (i32, i32, i32, i32),
    parent: HWND,
    id: isize,
    h_instance: HINSTANCE,
) -> HWND {
    let class_w = self::w(class);
    let text_w = self::w(text);
    unsafe {
        CreateWindowExW(
            ex_style,
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

/// Font GUI 9pt theo DPI hiện tại. Font sống trong static để xóa khi dialog
/// destroy (tránh leak GDI handle); nếu tạo thất bại thì dùng lại stock font.
#[cfg(windows)]
static UI_FONT: AtomicIsize = AtomicIsize::new(0);
/// DPI mà font hiện tại được tạo cho — khác DPI thì phải tạo lại (đa màn).
#[cfg(windows)]
static UI_FONT_DPI: AtomicIsize = AtomicIsize::new(0);

/// Font GUI 9pt cho DPI yêu cầu; DPI đổi so với lần tạo trước → xoá font cũ,
/// tạo font mới (dialog bị kéo sang màn hình khác DPI — WM_DPICHANGED).
#[cfg(windows)]
fn scaled_gui_font(dpi: i32) -> HGDIOBJ {
    let existing = UI_FONT.load(Ordering::Acquire);
    let font_dpi = UI_FONT_DPI.load(Ordering::Acquire);
    if existing != 0 && font_dpi == dpi as isize {
        return HGDIOBJ(existing as *mut std::ffi::c_void);
    }
    if existing != 0 {
        let _ = unsafe { DeleteObject(HGDIOBJ(existing as *mut std::ffi::c_void)) };
        UI_FONT.store(0, Ordering::Release);
    }
    let new_font = unsafe {
        let stock = GetStockObject(DEFAULT_GUI_FONT);
        let mut lf: LOGFONTW = std::mem::zeroed();
        let got = GetObjectW(
            stock,
            std::mem::size_of::<LOGFONTW>() as i32,
            Some(&mut lf as *mut LOGFONTW as *mut std::ffi::c_void),
        );
        if got == 0 {
            return stock;
        }
        lf.lfHeight = -(9 * dpi / 72); // 9pt tại DPI hiện tại
        let f = CreateFontIndirectW(&lf);
        if f.is_invalid() {
            return stock;
        }
        HGDIOBJ(f.0)
    };
    UI_FONT.store(new_font.0 as isize, Ordering::Release);
    UI_FONT_DPI.store(dpi as isize, Ordering::Release);
    new_font
}

/// Xử lý WM_DPICHANGED: resize theo RECT hệ thống đề xuất, tạo lại font rồi
/// re-layout toàn bộ control theo DPI mới (bug: dialog di chuyển giữa màn
/// hình có độ phân giải/DPI khác nhau bị lệch kích thước hoặc tràn màn hình).
#[cfg(windows)]
unsafe fn apply_dpi_change(hwnd: HWND, wparam: WPARAM, lparam: LPARAM, relayout: fn(HWND, i32)) {
    // HIWORD(wParam) = DPI Y mới (PerMonitorV2).
    let new_dpi = ((wparam.0 >> 16) & 0xffff) as i32;
    if new_dpi <= 0 {
        return;
    }
    let rc = *(lparam.0 as *const RECT);
    let _ = SetWindowPos(
        hwnd,
        None,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
    relayout(hwnd, new_dpi);
}

#[cfg(windows)]
fn set_font(ctrl: HWND, font: HGDIOBJ) {
    unsafe {
        let _ = SendMessageW(
            ctrl,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
}

/// Một control trong bảng layout (tọa độ theo lưới thiết kế 900 đơn vị).
#[cfg(windows)]
struct Ctl {
    class: &'static str,
    text: &'static str,
    style: u32,
    rect: (i32, i32, i32, i32),
    id: isize,
}

#[cfg(windows)]
const fn ctl(
    class: &'static str,
    text: &'static str,
    style: u32,
    rect: (i32, i32, i32, i32),
    id: isize,
) -> Ctl {
    Ctl {
        class,
        text,
        style,
        rect,
        id,
    }
}

#[cfg(windows)]
const CHK: u32 = BS_AUTOCHECKBOX | WS_TABSTOP.0;
#[cfg(windows)]
const BTN: u32 = BS_PUSHBUTTON | WS_TABSTOP.0;
#[cfg(windows)]
const L: i32 = 32; // cột trái
#[cfg(windows)]
const R: i32 = 470; // cột phải
#[cfg(windows)]
const CW: i32 = 420; // bề rộng một ô trong cột

/// Bảng layout bảng điều khiển (tọa độ theo lưới thiết kế 900 đơn vị).
/// Dùng ở MỌI lần tạo control VÀ khi re-layout sau WM_DPICHANGED — sửa một chỗ.
/// Thứ tự trong mảng = thứ tự Tab; WS_GROUP mở nhóm radio mới.
#[cfg(windows)]
const DIALOG_LAYOUT: [Ctl; 28] = [
    // 1. Điều khiển
    ctl(
        "BUTTON",
        "Điều khiển",
        BS_GROUPBOX,
        (15, 12, 870, 120),
        ID_LBL_BASE,
    ),
    ctl("STATIC", "Bảng mã:", 0, (L, 49, 85, 30), ID_LBL_BASE + 1),
    ctl(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0 | WS_GROUP.0,
        (122, 46, 300, 200),
        ID_COMBO_CHARSET,
    ),
    ctl("STATIC", "Kiểu gõ:", 0, (R, 49, 85, 30), ID_LBL_BASE + 2),
    ctl(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
        (560, 46, 300, 200),
        ID_COMBO_METHOD,
    ),
    ctl(
        "STATIC",
        "Phím chuyển:",
        0,
        (L, 93, 130, 30),
        ID_LBL_BASE + 3,
    ),
    ctl(
        "STATIC",
        "Ctrl + Shift   (hoặc Ctrl + Shift + Space)",
        0,
        (170, 93, 690, 30),
        ID_LBL_BASE + 4,
    ),
    // 2. Tùy chọn gõ — 2 cột, 4 hàng
    ctl(
        "BUTTON",
        "Tùy chọn gõ",
        BS_GROUPBOX,
        (15, 142, 870, 205),
        ID_LBL_BASE + 5,
    ),
    ctl(
        "BUTTON",
        "Bật gõ tiếng Việt",
        CHK,
        (L, 176, CW, 32),
        ID_CHK_GLOBAL_ENABLED,
    ),
    ctl(
        "BUTTON",
        "Dấu mới (hoà, thuỷ)",
        BS_AUTORADIOBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (R, 176, 400, 32),
        ID_RAD_DIACRITIC_NEW,
    ),
    ctl(
        "BUTTON",
        "Dấu cũ (hòa, thủy)",
        BS_AUTORADIOBUTTON,
        (R, 218, 400, 32),
        ID_RAD_DIACRITIC_OLD,
    ),
    ctl(
        "BUTTON",
        "Khôi phục từ tiếng Anh khi gõ sai",
        CHK | WS_GROUP.0,
        // Full bề rộng cột: nhãn dài bị CẮT khi thu hẹp ("...gõ sa") — sự cố
        // 2026-10-03, xem win-test-common-errors B9. Nút "Từ điển EN..." nằm
        // ở HÀNG NÚT dưới như Linux.
        (L, 218, CW, 32),
        ID_CHK_AUTO_RESTORE,
    ),
    ctl(
        "BUTTON",
        "Đặt dấu tự do",
        CHK,
        (L, 260, CW, 32),
        ID_CHK_FREE_MARKING,
    ),
    ctl(
        "BUTTON",
        "Tự viết hoa chữ đầu câu",
        CHK,
        (R, 260, 400, 32),
        ID_CHK_AUTO_CAPITALIZE,
    ),
    ctl(
        "BUTTON",
        "Quick Telex (cc→ch, nn→ng…)",
        CHK,
        (L, 302, CW, 32),
        ID_CHK_QUICK_TELEX,
    ),
    ctl(
        "BUTTON",
        "Gõ tắt cả khi tắt tiếng Việt",
        CHK,
        (R, 302, 400, 32),
        ID_CHK_MACRO_WHEN_OFF,
    ),
    // 3. Hệ thống (chỉ Windows — Linux do IBus/Fcitx5 tự khởi động)
    ctl(
        "BUTTON",
        "Hệ thống",
        BS_GROUPBOX,
        (15, 357, 870, 124),
        ID_LBL_BASE + 6,
    ),
    ctl(
        "BUTTON",
        "Khởi động cùng Windows",
        CHK,
        (L, 391, CW, 32),
        ID_CHK_AUTOSTART,
    ),
    ctl(
        "BUTTON",
        "Bật hội thoại này khi khởi động",
        CHK,
        (R, 391, 400, 32),
        ID_CHK_SHOW_ON_STARTUP,
    ),
    ctl(
        "BUTTON",
        "Dành Ctrl + Shift cho TextVN (tắt phím đổi bàn phím của Windows)",
        CHK,
        (L, 433, 830, 32),
        ID_CHK_CTRL_SHIFT,
    ),
    // 4. Hàng nút: thông tin/công cụ (trái) · thao tác (phải)
    ctl("BUTTON", "Hướng dẫn", BTN, (15, 497, 100, 48), ID_BTN_HELP),
    ctl(
        "BUTTON",
        "Thông tin",
        BTN,
        (121, 497, 100, 48),
        ID_BTN_ABOUT,
    ),
    ctl(
        "BUTTON",
        "Gõ tắt...",
        BTN,
        (227, 497, 100, 48),
        ID_BTN_MACROS,
    ),
    // Cùng vị trí tương đối như Linux (hàng nút, sau "Gõ tắt...") — 3 nền tảng thống nhất.
    ctl(
        "BUTTON",
        "Từ điển EN...",
        BTN,
        (333, 497, 130, 48),
        ID_BTN_ENGLISH,
    ),
    ctl(
        "BUTTON",
        "Cài & bật TSF",
        BTN,
        (469, 497, 130, 48),
        ID_BTN_SETUP_TSF,
    ),
    ctl(
        "BUTTON",
        "Mặc định",
        BTN,
        (605, 497, 95, 48),
        ID_BTN_DEFAULT,
    ),
    // Đóng đưa cửa sổ về khay; Kết thúc tắt hẳn ứng dụng.
    ctl(
        "BUTTON",
        "Đóng",
        BS_DEFPUSHBUTTON | WS_TABSTOP.0,
        (706, 497, 85, 48),
        ID_BTN_CLOSE,
    ),
    ctl("BUTTON", "Kết thúc", BTN, (797, 497, 88, 48), ID_BTN_EXIT),
];

/// Đặt lại vị trí/kích thước toàn bộ control theo DPI — chạy lúc tạo VÀ khi
/// dialog bị kéo sang màn hình khác DPI (WM_DPICHANGED, bug di chuyển đa màn).
#[cfg(windows)]
fn layout_dialog_controls(parent: HWND, dpi: i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    for c in &DIALOG_LAYOUT {
        let Ok(h) = (unsafe { GetDlgItem(Some(parent), c.id as i32) }) else {
            continue;
        };
        let (x, y, cw, ch) = c.rect;
        let _ = unsafe {
            SetWindowPos(
                h,
                None,
                k(x),
                k(y),
                k(cw),
                k(ch),
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
}

#[cfg(windows)]
fn create_dialog_controls(parent: HWND, h_instance: HINSTANCE, dpi: i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    let font = scaled_gui_font(dpi);

    for c in &DIALOG_LAYOUT {
        let (x, y, cw, ch) = c.rect;
        let hwnd = create_control(
            c.class,
            c.text,
            c.style,
            k(x),
            k(y),
            k(cw),
            k(ch),
            parent,
            c.id,
            h_instance,
        );
        set_font(hwnd, font);
    }

    for (id, labels) in [
        (ID_COMBO_CHARSET, &CHARSET_LABELS),
        (ID_COMBO_METHOD, &METHOD_LABELS),
    ] {
        for label in labels.iter() {
            let text_w = self::w(label);
            send_dlg_msg(parent, id, CB_ADDSTRING, 0, text_w.as_ptr() as isize);
        }
    }
}

#[cfg(windows)]
fn populate_controls_from_config(hwnd: HWND) {
    with_ctx(|ctx| {
        let cfg = ctx.svc.config();

        let method_idx = match cfg.method {
            Method::Telex => 0,
            Method::Vni => 1,
            Method::Viqr => 2,
            Method::SimpleTelex => 3,
        };
        send_dlg_msg(hwnd, ID_COMBO_METHOD, CB_SETCURSEL, method_idx, 0);

        let charset_idx = match cfg.output_charset {
            OutputCharset::UnicodePrecomposed => 0,
            OutputCharset::UnicodeDecomposed => 1,
            OutputCharset::Tcvn3 => 2,
            OutputCharset::VniWindows => 3,
        };
        send_dlg_msg(hwnd, ID_COMBO_CHARSET, CB_SETCURSEL, charset_idx, 0);

        set_chk(hwnd, ID_CHK_GLOBAL_ENABLED, ctx.svc.is_global_enabled());
        set_chk(hwnd, ID_CHK_AUTO_RESTORE, cfg.auto_restore_english);
        set_chk(hwnd, ID_CHK_FREE_MARKING, cfg.free_marking);
        set_chk(hwnd, ID_CHK_AUTO_CAPITALIZE, cfg.auto_capitalize);
        set_chk(hwnd, ID_CHK_QUICK_TELEX, cfg.quick_telex);
        set_chk(hwnd, ID_CHK_MACRO_WHEN_OFF, cfg.allow_macro_when_vi_off);
        set_chk(
            hwnd,
            ID_CHK_AUTOSTART,
            autostart::is_autostart_enabled().unwrap_or(false),
        );
        set_chk(hwnd, ID_CHK_SHOW_ON_STARTUP, cfg.show_dialog_on_startup);
        set_chk(
            hwnd,
            ID_CHK_CTRL_SHIFT,
            !crate::hotkey::current().ctrl_shift_taken(),
        );

        let is_new = cfg.diacritic_style == DiacriticStyle::New;
        set_chk(hwnd, ID_RAD_DIACRITIC_NEW, is_new);
        set_chk(hwnd, ID_RAD_DIACRITIC_OLD, !is_new);
    });
}

/// Đồng bộ lại bảng điều khiển (nếu đang mở) sau khi trạng thái đổi từ nơi khác —
/// phím tắt Ctrl+Shift, menu khay, IPC từ TSF.
#[cfg(windows)]
pub fn refresh_if_open() {
    let raw = SETTINGS_HWND.load(Ordering::Acquire);
    if raw == 0 {
        return;
    }
    let hwnd = HWND(raw as *mut std::ffi::c_void);
    if unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        populate_controls_from_config(hwnd);
    }
}

#[cfg(not(windows))]
pub fn refresh_if_open() {}

/// Cho phép Tab/Shift+Tab/Esc/Enter/phím mũi tên trong bảng điều khiển và cửa sổ Gõ tắt
/// (chúng là cửa sổ thường, không phải DialogBox). Gọi trong vòng lặp message của tray;
/// trả true nếu message đã được xử lý.
#[cfg(windows)]
pub fn pre_translate_message(msg: &MSG) -> bool {
    for slot in [&MACRO_HWND, &SETTINGS_HWND] {
        let raw = slot.load(Ordering::Acquire);
        if raw == 0 {
            continue;
        }
        let hwnd = HWND(raw as *mut std::ffi::c_void);
        // SAFETY: msg là MSG hợp lệ từ GetMessageW; hwnd đã kiểm IsWindow.
        unsafe {
            if IsWindow(Some(hwnd)).as_bool()
                && (msg.hwnd == hwnd || IsChild(hwnd, msg.hwnd).as_bool())
                && IsDialogMessageW(hwnd, msg).as_bool()
            {
                return true;
            }
        }
    }
    false
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

/// Checkbox tuỳ chọn gõ → setter tương ứng (trả version để broadcast `ConfigReload`).
#[cfg(windows)]
fn apply_option_checkbox(
    svc: &SvcManager,
    id: isize,
    checked: bool,
) -> Option<std::result::Result<u64, DocError>> {
    Some(match id {
        ID_CHK_AUTO_RESTORE => svc.set_auto_restore_english(checked),
        ID_CHK_FREE_MARKING => svc.set_free_marking(checked),
        ID_CHK_AUTO_CAPITALIZE => svc.set_auto_capitalize(checked),
        ID_CHK_QUICK_TELEX => svc.set_quick_telex(checked),
        ID_CHK_MACRO_WHEN_OFF => svc.set_allow_macro_when_vi_off(checked),
        ID_CHK_SHOW_ON_STARTUP => svc.set_show_dialog_on_startup(checked),
        _ => return None,
    })
}

/// Chỉ báo TSF reload sau khi config đã lưu thành công. Nếu lỗi I/O, trả control
/// về giá trị thật thay vì hiển thị lựa chọn mà engine không thể dùng.
#[cfg(windows)]
fn save_config_change(
    hwnd: HWND,
    change: impl FnOnce(&DialogContext) -> std::result::Result<u64, DocError>,
) -> bool {
    match with_ctx(|ctx| change(ctx).map(|ver| ctx.ipc.broadcast_config_reload(ver))) {
        Some(Ok(())) => true,
        Some(Err(_)) => {
            populate_controls_from_config(hwnd);
            show_information(
                hwnd,
                "Không thể lưu cấu hình TextVN",
                "Thay đổi chưa được áp dụng. Kiểm tra dung lượng đĩa và quyền thư mục AppData rồi thử lại.",
            );
            false
        }
        None => false,
    }
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
                ID_BTN_CLOSE | IDCANCEL_CMD => {
                    // Đóng / Esc: ẩn về khay (Minimize to Tray)
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                ID_BTN_EXIT => {
                    // Kết thúc: đóng hẳn ứng dụng
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    PostQuitMessage(0);
                }
                ID_BTN_HELP => show_information(hwnd, "Hướng dẫn TextVN", HELP_TEXT),
                ID_BTN_ABOUT => show_information(
                    hwnd,
                    "Thông tin TextVN",
                    &format!(
                        "TextVN {} — Bộ gõ tiếng Việt cho Windows\r\n\r\n\
                         Phát triển bởi: LinhBH.CoM\r\n\
                         Giấy phép: GPL-3.0-or-later\r\n\
                         https://github.com/hunglinhpt/TextVN",
                        env!("CARGO_PKG_VERSION")
                    ),
                ),
                ID_BTN_MACROS => show_macro_editor(hwnd),
                ID_BTN_ENGLISH => show_word_list_editor(hwnd),
                ID_BTN_SETUP_TSF => match register_and_activate_tsf() {
                    Ok(()) => show_information(
                        hwnd,
                        "TextVN TSF",
                        "Đã đăng ký và kích hoạt bộ gõ TextVN.\r\n\r\nHãy thử gõ Telex trong Notepad. Nếu vẫn chưa hoạt động, chọn TextVN trong danh sách bộ gõ (Win + Space) hoặc kiểm tra phần mềm bảo mật.",
                    ),
                    Err(reason) => {
                        show_information(hwnd, "Không thể đăng ký TextVN TSF", &reason)
                    }
                },

                ID_BTN_DEFAULT => {
                    if save_config_change(hwnd, |ctx| ctx.svc.reset_config_defaults()) {
                        populate_controls_from_config(hwnd);
                        crate::notify_tray_state_changed();
                    }
                }
                ID_COMBO_METHOD if notify_code == CBN_SELCHANGE => {
                    let sel = send_dlg_msg(hwnd, ID_COMBO_METHOD, CB_GETCURSEL, 0, 0).0;
                    let m = match sel {
                        1 => Method::Vni,
                        2 => Method::Viqr,
                        3 => Method::SimpleTelex,
                        _ => Method::Telex,
                    };
                    if save_config_change(hwnd, |ctx| ctx.svc.set_method(m)) {
                        crate::notify_tray_state_changed();
                    }
                }
                ID_COMBO_CHARSET if notify_code == CBN_SELCHANGE => {
                    let sel = send_dlg_msg(hwnd, ID_COMBO_CHARSET, CB_GETCURSEL, 0, 0).0;
                    let cs = match sel {
                        1 => OutputCharset::UnicodeDecomposed,
                        2 => OutputCharset::Tcvn3,
                        3 => OutputCharset::VniWindows,
                        _ => OutputCharset::UnicodePrecomposed,
                    };
                    let _ = save_config_change(hwnd, |ctx| ctx.svc.set_output_charset(cs));
                }
                ID_CHK_AUTOSTART => {
                    let checked = get_chk(hwnd, ID_CHK_AUTOSTART);
                    let result = if checked {
                        autostart::enable_autostart(None)
                    } else {
                        autostart::disable_autostart()
                    };
                    if result.is_err() {
                        // Không ghi được HKCU\...\Run → trả checkbox về trạng thái thật.
                        set_chk(
                            hwnd,
                            ID_CHK_AUTOSTART,
                            autostart::is_autostart_enabled().unwrap_or(false),
                        );
                    }
                }
                ID_CHK_CTRL_SHIFT => {
                    // Phím tắt đổi bố cục của Windows nuốt Ctrl + Shift trước TextVN.
                    let result = if get_chk(hwnd, ID_CHK_CTRL_SHIFT) {
                        crate::hotkey::free_ctrl_shift()
                    } else {
                        crate::hotkey::restore_windows_ctrl_shift()
                    };
                    if result.is_err() {
                        set_chk(
                            hwnd,
                            ID_CHK_CTRL_SHIFT,
                            !crate::hotkey::current().ctrl_shift_taken(),
                        );
                    }
                }
                ID_CHK_GLOBAL_ENABLED => {
                    let checked = get_chk(hwnd, ID_CHK_GLOBAL_ENABLED);
                    with_ctx(|ctx| {
                        let (actual, ver) = ctx.svc.set_global_enabled(checked);
                        ctx.ipc.broadcast_state_update("*", actual, ver);
                    });
                    crate::notify_tray_state_changed();
                }
                ID_RAD_DIACRITIC_NEW | ID_RAD_DIACRITIC_OLD => {
                    let style = if cmd_id == ID_RAD_DIACRITIC_NEW {
                        DiacriticStyle::New
                    } else {
                        DiacriticStyle::Old
                    };
                    let _ = save_config_change(hwnd, |ctx| ctx.svc.set_diacritic_style(style));
                }
                id => {
                    let checked = get_chk(hwnd, id);
                    let saved = with_ctx(|ctx| {
                        apply_option_checkbox(&ctx.svc, id, checked)
                            .map(|result| result.map(|ver| ctx.ipc.broadcast_config_reload(ver)))
                    });
                    if matches!(saved, Some(Some(Err(_)))) {
                        populate_controls_from_config(hwnd);
                        show_information(
                            hwnd,
                            "Không thể lưu cấu hình TextVN",
                            "Thay đổi chưa được áp dụng. Kiểm tra dung lượng đĩa và quyền thư mục AppData rồi thử lại.",
                        );
                    }
                }
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            // Kéo dialog sang màn hình khác DPI: resize + re-layout + font mới.
            apply_dpi_change(hwnd, wparam, lparam, layout_dialog_controls);
            for c in &DIALOG_LAYOUT {
                let Ok(ctl_h) = (unsafe { GetDlgItem(Some(hwnd), c.id as i32) }) else {
                    continue;
                };
                set_font(ctl_h, scaled_gui_font(((wparam.0 >> 16) & 0xffff) as i32));
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // Nút [X]: ẩn về khay hệ thống thay vì thoát
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_DESTROY => {
            SETTINGS_HWND.store(0, Ordering::Release);
            // Controls con đã bị hủy trước khi parent nhận WM_DESTROY
            // nên font không còn được tham chiếu - an toàn để xóa.
            let font = UI_FONT.swap(0, Ordering::AcqRel);
            if font != 0 {
                let _ = DeleteObject(HGDIOBJ(font as *mut std::ffi::c_void));
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ---------------------------------------------------------------------------
// Cửa sổ Gõ tắt — cùng định dạng với bảng cài đặt Linux (`textvn_config::macro_text`).
// ---------------------------------------------------------------------------

#[cfg(windows)]
const MACRO_CLASS_NAME: &str = "TextVNMacroEditorClass";
#[cfg(windows)]
static MACRO_HWND: AtomicIsize = AtomicIsize::new(0);

/// Text của EDIT nhiều dòng dùng CRLF; bảng gõ tắt lưu LF.
fn to_crlf(s: &str) -> String {
    s.replace('\n', "\r\n")
}

/// Bảng layout cửa sổ Gõ tắt — label: `Some` (text tĩnh) | `None` (EDIT nội
/// dung động). Dùng cả lúc tạo lẫn re-layout sau WM_DPICHANGED.
#[cfg(windows)]
const MACRO_EX_NONE: WINDOW_EX_STYLE = WINDOW_EX_STYLE(0);
/// Một control của cửa sổ Gõ tắt: (ex-style, class, label tĩnh, style, rect, id).
#[cfg(windows)]
type MacroCtl = (
    WINDOW_EX_STYLE,
    &'static str,
    Option<&'static str>,
    u32,
    (i32, i32, i32, i32),
    isize,
);

/// Bảng layout cửa sổ Gõ tắt — label: `Some` (text tĩnh) | `None` (EDIT nội
/// dung động). Dùng cả lúc tạo lẫn re-layout sau WM_DPICHANGED.
#[cfg(windows)]
const MACRO_LAYOUT: [MacroCtl; 7] = [
    (
        MACRO_EX_NONE,
        "STATIC",
        Some(
            "Mỗi dòng một mục:   gõ tắt = nội dung      (ví dụ:  vn = Việt Nam)\r\n\
         Dòng bắt đầu bằng # là ghi chú.  \\n trong nội dung = xuống dòng.  Tối đa 64 ký tự.",
        ),
        0,
        (20, 12, 680, 64),
        ID_LBL_MACRO_HINT,
    ),
    (
        WS_EX_CLIENTEDGE,
        "EDIT",
        None,
        ES_MULTILINE
            | ES_AUTOVSCROLL
            | ES_WANTRETURN
            | ES_NOHIDESEL
            | WS_VSCROLL.0
            | WS_TABSTOP.0
            | WS_GROUP.0,
        (20, 84, 680, 370),
        ID_EDIT_MACROS,
    ),
    (
        MACRO_EX_NONE,
        "STATIC",
        Some("Bung gõ tắt bằng phím:"),
        0,
        (20, 474, 230, 30),
        ID_LBL_MACRO_TRIGGER,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Tab"),
        BS_AUTORADIOBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (255, 470, 90, 32),
        ID_RAD_TRIGGER_TAB,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Space"),
        BS_AUTORADIOBUTTON,
        (350, 470, 110, 32),
        ID_RAD_TRIGGER_SPACE,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Lưu"),
        BS_DEFPUSHBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (480, 500, 105, 46),
        ID_BTN_MACRO_SAVE,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Hủy"),
        BS_PUSHBUTTON | WS_TABSTOP.0,
        (595, 500, 105, 46),
        ID_BTN_MACRO_CANCEL,
    ),
];

/// Re-layout cửa sổ Gõ tắt theo DPI (WM_DPICHANGED — đa màn hình).
#[cfg(windows)]
fn layout_macro_controls(parent: HWND, dpi: i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    for (_, _, _, _, rect, id) in MACRO_LAYOUT {
        let Ok(h) = (unsafe { GetDlgItem(Some(parent), id as i32) }) else {
            continue;
        };
        let _ = unsafe {
            SetWindowPos(
                h,
                None,
                k(rect.0),
                k(rect.1),
                k(rect.2),
                k(rect.3),
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
        set_font(h, scaled_gui_font(dpi));
    }
}

#[cfg(windows)]
fn show_macro_editor(owner: HWND) {
    let existing = MACRO_HWND.load(Ordering::Acquire);
    if existing != 0 {
        let hwnd = HWND(existing as *mut std::ffi::c_void);
        if unsafe { IsWindow(Some(hwnd)) }.as_bool() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
            return;
        }
    }
    let Some((text, trigger)) = with_ctx(|ctx| {
        let cfg = ctx.svc.config();
        (
            textvn_config::macro_text::format(&cfg.macros),
            cfg.macro_trigger,
        )
    }) else {
        return;
    };

    let class_name = w(MACRO_CLASS_NAME);
    let title = w("TextVN - Gõ tắt");
    let h_instance: HINSTANCE = unsafe { GetModuleHandleW(None).unwrap_or_default() }.into();
    register_class(&class_name, Some(macro_wnd_proc), h_instance);

    let dpi = system_dpi();
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
    let (width, height) =
        window_size_for_client(MACRO_DESIGN_WIDTH, MACRO_DESIGN_HEIGHT, dpi, style);
    let (x, y) = center_on_work_area(width, height);
    let Ok(hwnd) = (unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            x,
            y,
            width,
            height,
            Some(owner),
            None,
            Some(h_instance),
            None,
        )
    }) else {
        return;
    };
    MACRO_HWND.store(hwnd.0 as isize, Ordering::Release);

    // DPI theo monitor thật (B9/B10) — resize lại đúng cỡ trước khi layout.
    let dpi = window_dpi(hwnd);
    if dpi != system_dpi() {
        let (width, height) =
            window_size_for_client(MACRO_DESIGN_WIDTH, MACRO_DESIGN_HEIGHT, dpi, style);
        let (x, y) = center_on_work_area(width, height);
        let _ = unsafe {
            SetWindowPos(
                HWND(hwnd.0),
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    let font = scaled_gui_font(dpi);
    for (ex, class, label, style, rect, id) in MACRO_LAYOUT {
        // EDIT: nội dung động nạp sau; label None → chuỗi rỗng.
        let content = label.map(to_crlf).unwrap_or_default();
        let c = create_control_ex(
            ex,
            class,
            &content,
            style,
            (k(rect.0), k(rect.1), k(rect.2), k(rect.3)),
            hwnd,
            id,
            h_instance,
        );
        set_font(c, font);
    }
    let edit = (unsafe { GetDlgItem(Some(hwnd), ID_EDIT_MACROS as i32) }).unwrap_or_default();
    // Nội dung động của EDIT nạp sau khi tạo (MACRO_LAYOUT chỉ khai khung).
    let content_w = w(&to_crlf(&text));
    unsafe {
        let _ = SendMessageW(
            edit,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(content_w.as_ptr() as isize)),
        );
        // Mặc định EDIT giới hạn 32K ký tự — đủ, nhưng nâng lên để không cắt bảng lớn.
        let _ = SendMessageW(
            edit,
            EM_SETLIMITTEXT,
            Some(WPARAM(1 << 20)),
            Some(LPARAM(0)),
        );
    }
    let is_tab = trigger == MacroTrigger::Tab;
    set_chk(hwnd, ID_RAD_TRIGGER_TAB, is_tab);
    set_chk(hwnd, ID_RAD_TRIGGER_SPACE, !is_tab);

    unsafe {
        // Cửa sổ con kiểu modal: khoá bảng điều khiển tới khi Lưu/Hủy.
        let _ = EnableWindow(owner, false);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(edit));
    }
}

#[cfg(windows)]
fn read_window_text(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let got = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        String::from_utf16_lossy(&buf[..got.min(len)])
    }
}

/// Lưu bảng gõ tắt; lỗi → báo dòng sai và bôi đen dòng đó, cửa sổ vẫn mở.
#[cfg(windows)]
fn save_macros(hwnd: HWND) -> bool {
    let edit = unsafe { GetDlgItem(Some(hwnd), ID_EDIT_MACROS as i32) }.unwrap_or_default();
    let text = read_window_text(edit).replace("\r\n", "\n");
    let trigger = if get_chk(hwnd, ID_RAD_TRIGGER_SPACE) {
        MacroTrigger::Space
    } else {
        MacroTrigger::Tab
    };
    let result = with_ctx(|ctx| {
        let previous = ctx.svc.config().macros;
        match textvn_config::macro_text::parse(&text, &previous) {
            Ok(macros) => match ctx.svc.set_macros(macros, trigger) {
                Ok(ver) => {
                    ctx.ipc.broadcast_config_reload(ver);
                    Ok(())
                }
                Err(_) => Err(None),
            },
            Err(e) => Err(Some(e)),
        }
    });
    match result {
        Some(Ok(())) => true,
        // `None` = không lấy được ctx (khóa nội bộ poisoned) — KHÔNG coi là
        // thành công: trước đây cửa sổ đóng mà bảng gõ tắt vừa sửa bị vứt
        // im lặng (audit 2026-10-01 m8).
        None => {
            show_information(
                hwnd,
                "Không thể lưu bảng gõ tắt TextVN",
                "Không truy cập được trạng thái ứng dụng (khóa nội bộ bị lỗi).\r\n\
                 Đóng rồi mở lại TextVN, sau đó thử lưu lại.",
            );
            false
        }
        Some(Err(None)) => {
            show_information(
                hwnd,
                "Không thể lưu bảng gõ tắt TextVN",
                "Bảng gõ tắt chưa được áp dụng. Kiểm tra dung lượng đĩa và quyền thư mục AppData rồi thử lại.",
            );
            false
        }
        Some(Err(Some(e))) => {
            unsafe {
                let line = e.line.saturating_sub(1);
                let start = SendMessageW(edit, EM_LINEINDEX, Some(WPARAM(line)), Some(LPARAM(0))).0;
                let len = SendMessageW(
                    edit,
                    EM_LINELENGTH,
                    Some(WPARAM(start as usize)),
                    Some(LPARAM(0)),
                )
                .0;
                let _ = SendMessageW(
                    edit,
                    EM_SETSEL,
                    Some(WPARAM(start as usize)),
                    Some(LPARAM(start + len)),
                );
                let _ = SendMessageW(edit, EM_SCROLLCARET, Some(WPARAM(0)), Some(LPARAM(0)));
                let _ = SetFocus(Some(edit));
            }
            show_information(hwnd, "Bảng gõ tắt chưa hợp lệ", &e.message_vi());
            false
        }
    }
}

#[cfg(windows)]
fn close_macro_editor(hwnd: HWND) {
    unsafe {
        if let Ok(owner) = GetWindow(hwnd, GW_OWNER) {
            let _ = EnableWindow(owner, true);
            let _ = SetForegroundWindow(owner);
        }
        let _ = DestroyWindow(hwnd);
    }
}

// ---------------------------------------------------------------------------
// Cửa sổ Từ điển EN — danh sách từ tiếng Anh bổ sung của người dùng
// (`config.english_words`). Đây là "quyết định tường minh": mọi từ trong danh
// sách được restore/gợi ý bất kể fold có trùng âm tiết Việt thông dụng hay không.
// ---------------------------------------------------------------------------

#[cfg(windows)]
const WORDLIST_CLASS_NAME: &str = "TextVNWordListEditorClass";
#[cfg(windows)]
static WORDLIST_HWND: AtomicIsize = AtomicIsize::new(0);
#[cfg(windows)]
const WORDLIST_DESIGN_WIDTH: i32 = 720;
#[cfg(windows)]
const WORDLIST_DESIGN_HEIGHT: i32 = 520;
#[cfg(windows)]
const ID_EDIT_WORDLIST: isize = 2102;
#[cfg(windows)]
const ID_BTN_WORDLIST_SAVE: isize = 2106;
#[cfg(windows)]
const ID_BTN_WORDLIST_CANCEL: isize = 2107;

/// Một control của cửa sổ Từ điển EN — cùng dạng `MacroCtl`.
#[cfg(windows)]
type WordListCtl = (
    WINDOW_EX_STYLE,
    &'static str,
    Option<&'static str>,
    u32,
    (i32, i32, i32, i32),
    isize,
);

/// Layout cửa sổ Từ điển EN (DPI đích = lưới thiết kế 720 đơn vị).
#[cfg(windows)]
const WORDLIST_LAYOUT: [WordListCtl; 4] = [
    (
        MACRO_EX_NONE,
        "STATIC",
        Some(
            "Mỗi dòng một từ tiếng Anh bạn muốn TextVN GIỮ NGUYÊN (không tự nhận là từ Việt).\r\n\
         Ví dụ: text, test, list, download, cowork… Chữ thường, không dấu cách, tối đa 15 ký tự.",
        ),
        0,
        (20, 12, 680, 44),
        ID_LBL_WORDLIST_HINT,
    ),
    (
        WS_EX_CLIENTEDGE,
        "EDIT",
        None,
        ES_MULTILINE
            | ES_AUTOVSCROLL
            | ES_WANTRETURN
            | ES_NOHIDESEL
            | WS_VSCROLL.0
            | WS_TABSTOP.0
            | WS_GROUP.0,
        (20, 64, 680, 396),
        ID_EDIT_WORDLIST,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Lưu"),
        BS_DEFPUSHBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (480, 470, 105, 42),
        ID_BTN_WORDLIST_SAVE,
    ),
    (
        MACRO_EX_NONE,
        "BUTTON",
        Some("Hủy"),
        BS_PUSHBUTTON | WS_TABSTOP.0,
        (595, 470, 105, 42),
        ID_BTN_WORDLIST_CANCEL,
    ),
];

/// Re-layout cửa sổ Từ điển EN theo DPI (WM_DPICHANGED).
#[cfg(windows)]
fn layout_wordlist_controls(parent: HWND, dpi: i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    for (_, _, _, _, rect, id) in WORDLIST_LAYOUT {
        let Ok(h) = (unsafe { GetDlgItem(Some(parent), id as i32) }) else {
            continue;
        };
        let _ = unsafe {
            SetWindowPos(
                h,
                None,
                k(rect.0),
                k(rect.1),
                k(rect.2),
                k(rect.3),
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
        set_font(h, scaled_gui_font(dpi));
    }
}

#[cfg(windows)]
fn show_word_list_editor(owner: HWND) {
    let existing = WORDLIST_HWND.load(Ordering::Acquire);
    if existing != 0 {
        let hwnd = HWND(existing as *mut std::ffi::c_void);
        if unsafe { IsWindow(Some(hwnd)) }.as_bool() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
            return;
        }
    }
    let Some(words) = with_ctx(|ctx| ctx.svc.config().english_words) else {
        return;
    };
    let text = words.join("\r\n");

    let class_name = w(WORDLIST_CLASS_NAME);
    let title = w("TextVN - Từ điển tiếng Anh");
    let h_instance: HINSTANCE = unsafe { GetModuleHandleW(None).unwrap_or_default() }.into();
    register_class(&class_name, Some(wordlist_wnd_proc), h_instance);

    let dpi = system_dpi();
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
    let (width, height) =
        window_size_for_client(WORDLIST_DESIGN_WIDTH, WORDLIST_DESIGN_HEIGHT, dpi, style);
    let (x, y) = center_on_work_area(width, height);
    let Ok(hwnd) = (unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            style,
            x,
            y,
            width,
            height,
            Some(owner),
            None,
            Some(h_instance),
            None,
        )
    }) else {
        return;
    };
    WORDLIST_HWND.store(hwnd.0 as isize, Ordering::Release);

    // DPI theo monitor thật (B9/B10).
    let dpi = window_dpi(hwnd);
    if dpi != system_dpi() {
        let (width, height) =
            window_size_for_client(WORDLIST_DESIGN_WIDTH, WORDLIST_DESIGN_HEIGHT, dpi, style);
        let (x, y) = center_on_work_area(width, height);
        let _ = unsafe {
            SetWindowPos(
                HWND(hwnd.0),
                None,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    let font = scaled_gui_font(dpi);
    for (ex, class, label, style, rect, id) in WORDLIST_LAYOUT {
        let content = label.map(to_crlf).unwrap_or_default();
        let c = create_control_ex(
            ex,
            class,
            &content,
            style,
            (k(rect.0), k(rect.1), k(rect.2), k(rect.3)),
            hwnd,
            id,
            h_instance,
        );
        set_font(c, font);
    }
    let edit = (unsafe { GetDlgItem(Some(hwnd), ID_EDIT_WORDLIST as i32) }).unwrap_or_default();
    let content_w = w(&text);
    unsafe {
        let _ = SendMessageW(
            edit,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(content_w.as_ptr() as isize)),
        );
        let _ = EnableWindow(owner, false);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(edit));
    }
}

/// Chuẩn hoá danh sách từ người dùng nhập: trim, bỏ dòng trống/`#`, lowercase,
/// chỉ giữ chữ cái ASCII, khử trùng lặp (giữ thứ tự nhập).
fn normalize_word_list(text: &str) -> Vec<String> {
    textvn_config::doc::normalize_english_words(text)
}

/// Lưu từ điển EN; luôn thành công nếu persist OK (danh sách không có cú pháp sai).
#[cfg(windows)]
fn save_word_list(hwnd: HWND) -> bool {
    let edit = unsafe { GetDlgItem(Some(hwnd), ID_EDIT_WORDLIST as i32) }.unwrap_or_default();
    let text = read_window_text(edit);
    let words = normalize_word_list(&text);
    let result = with_ctx(|ctx| {
        ctx.svc
            .set_english_words(words)
            .map(|ver| ctx.ipc.broadcast_config_reload(ver))
            .map_err(|_| ())
    });
    match result {
        Some(Ok(())) => true,
        Some(Err(())) => {
            show_information(
                hwnd,
                "Không thể lưu từ điển TextVN",
                "Từ điển chưa được áp dụng. Kiểm tra dung lượng đĩa và quyền thư mục AppData rồi thử lại.",
            );
            false
        }
        None => {
            show_information(
                hwnd,
                "Không thể lưu từ điển TextVN",
                "Không truy cập được trạng thái ứng dụng (khóa nội bộ bị lỗi).\r\n\
                 Đóng rồi mở lại TextVN, sau đó thử lưu lại.",
            );
            false
        }
    }
}

#[cfg(windows)]
fn close_word_list_editor(hwnd: HWND) {
    unsafe {
        if let Ok(owner) = GetWindow(hwnd, GW_OWNER) {
            let _ = EnableWindow(owner, true);
            let _ = SetForegroundWindow(owner);
        }
        let _ = DestroyWindow(hwnd);
    }
}

#[cfg(windows)]
unsafe extern "system" fn wordlist_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DPICHANGED => {
            apply_dpi_change(hwnd, wparam, lparam, layout_wordlist_controls);
            LRESULT(0)
        }
        WM_COMMAND => {
            match (wparam.0 & 0xffff) as isize {
                ID_BTN_WORDLIST_SAVE => {
                    if save_word_list(hwnd) {
                        close_word_list_editor(hwnd);
                    }
                }
                ID_BTN_WORDLIST_CANCEL | IDCANCEL_CMD => close_word_list_editor(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            close_word_list_editor(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            WORDLIST_HWND.store(0, Ordering::Release);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
unsafe extern "system" fn macro_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DPICHANGED => {
            apply_dpi_change(hwnd, wparam, lparam, layout_macro_controls);
            LRESULT(0)
        }
        WM_COMMAND => {
            match (wparam.0 & 0xffff) as isize {
                ID_BTN_MACRO_SAVE => {
                    if save_macros(hwnd) {
                        close_macro_editor(hwnd);
                    }
                }
                ID_BTN_MACRO_CANCEL | IDCANCEL_CMD => close_macro_editor(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            close_macro_editor(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            MACRO_HWND.store(0, Ordering::Release);
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
///
/// Trả `Err(lý do thật)`: exit code + đuôi `register.log` — trước đây dialog luôn
/// đổ cho "quyền ghi HKCU" kể cả khi nguyên nhân là thiếu DLL/CLI, khiến người
/// dùng sửa sai chỗ (ảnh lỗi thực tế: sandbox chặn HKCU + thiếu log).
#[cfg(windows)]
fn register_and_activate_tsf() -> std::result::Result<(), String> {
    let Ok(mut cli_path) = std::env::current_exe() else {
        return Err("Không xác định được thư mục cài đặt (current_exe lỗi). \
                    Hãy mở TextVN từ đúng thư mục đã cài/giải nén."
            .to_string());
    };
    cli_path.set_file_name("textvn-cli.exe");
    if !cli_path.is_file() {
        return Err(format!(
            "Thiếu textvn-cli.exe cạnh TextVN.exe ({}).\r\n\r\nGói cài/portable bị thiếu tệp — \
             hãy tải lại gói đầy đủ và bấm [Cài & bật TSF] lại.",
            cli_path.display()
        ));
    }

    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let status = std::process::Command::new(&cli_path)
        .arg("register")
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    let exit_code = match status {
        Ok(st) if st.success() => return Ok(()),
        Ok(st) => st.code().unwrap_or(-1),
        Err(err) => return Err(format!("Không chạy được textvn-cli.exe: {err}")),
    };

    let mut message = format!("textvn-cli register thất bại (exit code {exit_code}).\r\n");
    let log_path = register_log_path();
    match &log_path {
        Some(path) => {
            if let Some(tail) = read_log_tail(path, 12) {
                message.push_str("\r\nChi tiết từ register.log:\r\n");
                message.push_str(&tail);
            } else {
                message.push_str("\r\nKhông đọc được register.log (file trống/thiếu quyền).");
            }
            message.push_str(&format!("\r\n\r\nLog đầy đủ: {}", path.display()));
        }
        None => message.push_str("\r\nKhông xác định được %LOCALAPPDATA% để đọc register.log."),
    }
    message.push_str("\r\n\r\n");
    message.push_str(&advice_for_failure(exit_code, &message));
    Err(message)
}

/// `%LOCALAPPDATA%\TextVN\logs\register.log` — CLI ghi mọi bước vào đây (kể cả
/// khi setup chạy CLI ẩn console, lý do thật chỉ nằm trong file).
#[cfg_attr(not(windows), allow(dead_code))]
fn register_log_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(
        std::path::PathBuf::from(base)
            .join("TextVN")
            .join("logs")
            .join("register.log"),
    )
}

/// `max_lines` dòng cuối của log (thuần path → test được trên mọi OS).
#[cfg_attr(not(windows), allow(dead_code))]
fn read_log_tail(path: &std::path::Path, max_lines: usize) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }
    // register.log duoc append qua nhieu lan chay. Khong de loi ACL/DLL cua
    // lan cu lot vao advice cho lan hien tai khi log moi ngan hon max_lines.
    let run_start = lines
        .iter()
        .rposition(|line| line.contains("=== TextVN register ("))
        .unwrap_or(0);
    let start = lines.len().saturating_sub(max_lines).max(run_start);
    Some(lines[start..].join("\r\n"))
}

/// Gợi ý theo nguyên nhân THẬT trong log/exit code (giữ cả gợi ý HKCU cho lỗi
/// ACL thật — `ERROR_ACCESS_DENIED (5)` khi tài khoản bị policy chặn ghi).
#[cfg_attr(not(windows), allow(dead_code))]
fn advice_for_failure(exit_code: i32, detail: &str) -> String {
    if exit_code == 3 {
        return "Windows yêu cầu quyền Administrator để đăng ký profile TSF ở phạm vi máy. \
                Hãy dùng bộ cài TextVN chính thức; không nâng quyền trực tiếp cho file trong thư mục portable."
            .to_string();
    }
    if detail.contains("Đăng ký qua API TSF → 0x80004005")
        && detail.contains("hậu kiểm đăng ký TSF không đạt")
    {
        return "Đã ghi được HKCU nhưng Windows chưa nhận profile TSF; đây không phải lỗi quyền ghi HKCU. \
                Hãy cài bằng bộ cài TextVN mới (đăng ký machine một lần rồi bật cho tài khoản hiện tại). \
                Không chạy textvn-cli.exe trong thư mục portable với quyền Administrator."
            .to_string();
    }
    if detail.contains("ACCESS_DENIED") || detail.contains("0x00000005") {
        // Nhánh phải khớp log THẬT: CLI bản mới in
        // `FAIL 0x00000005 (ERROR_ACCESS_DENIED)`, bản 0.2.0/0.2.1 đã cài chỉ in
        // hex `FAIL 0x00000005` — thiếu hex thì nhánh này là dead code và người
        // dùng bị ACL thật luôn nhận gợi ý generic.
        return "Windows từ chối ghi HKCU\\Software\\Classes\\CLSID cho tài khoản hiện tại.\r\n\
                Kiểm tra quyền tài khoản và chính sách phần mềm bảo mật, sau đó bấm [Cài & bật TSF] lại."
            .to_string();
    }
    if detail.contains("Không tìm thấy textvn-tsf.dll") {
        return "Thiếu textvn-tsf.dll cạnh textvn-cli.exe.\r\n\
                Hãy cài lại bằng installer hoặc giải nén lại gói portable đầy đủ rồi thử lại."
            .to_string();
    }
    "Xem register.log (đường dẫn ở trên) để biết bước thất bại; \
     nếu cần hỗ trợ, gửi kèm file log này."
        .to_string()
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
            ID_CHK_CTRL_SHIFT,
            ID_CHK_GLOBAL_ENABLED,
            ID_RAD_DIACRITIC_NEW,
            ID_RAD_DIACRITIC_OLD,
            ID_BTN_CLOSE,
            ID_BTN_DEFAULT,
            ID_BTN_EXIT,
            ID_BTN_HELP,
            ID_BTN_ABOUT,
            ID_BTN_SETUP_TSF,
            ID_CHK_AUTO_CAPITALIZE,
            ID_CHK_QUICK_TELEX,
            ID_CHK_MACRO_WHEN_OFF,
            ID_CHK_SHOW_ON_STARTUP,
            ID_BTN_MACROS,
            ID_EDIT_MACROS,
            ID_RAD_TRIGGER_TAB,
            ID_RAD_TRIGGER_SPACE,
            ID_BTN_MACRO_SAVE,
            ID_BTN_MACRO_CANCEL,
        ];
        let mut set = std::collections::HashSet::new();
        for id in ids {
            assert!(set.insert(id), "Duplicate dialog control ID: {id}");
            // IDOK/IDCANCEL (1/2) do IsDialogMessage tự gửi — không được trùng.
            assert!(id > IDCANCEL_CMD);
        }
    }

    /// Nhãn phải khớp bảng cài đặt Linux từng chữ (UI thống nhất — ui-spec.md).
    #[test]
    fn labels_match_linux_settings_panel() {
        let linux = [
            include_str!("../../adapters/linux-settings/src/settings_window.c"),
            include_str!("../../adapters/linux-settings/src/settings_model.c"),
        ]
        .concat();
        // 0.2.14: kiểm luôn macOS — ba nền tảng PHẢI cùng nhãn (yêu cầu chủ repo:
        // "thống nhất UI cho cả 3 nền tảng, buộc phải giống nhau").
        let macos =
            include_str!("../../adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift");
        for label in CHARSET_LABELS.iter().chain(METHOD_LABELS.iter()) {
            assert!(
                linux.contains(&format!("\"{label}\"")),
                "nhãn `{label}` thiếu trong bảng cài đặt Linux"
            );
        }
        for label in [
            "Bật gõ tiếng Việt",
            "Dấu mới (hoà, thuỷ)",
            "Dấu cũ (hòa, thủy)",
            "Khôi phục từ tiếng Anh khi gõ sai",
            "Đặt dấu tự do",
            "Tự viết hoa chữ đầu câu",
            "Quick Telex (cc→ch, nn→ng…)",
            "Gõ tắt cả khi tắt tiếng Việt",
            "Gõ tắt...",
            "Từ điển EN...",
        ] {
            assert!(
                linux.contains(label),
                "nhãn `{label}` thiếu trong bảng cài đặt Linux"
            );
            assert!(
                macos.contains(label),
                "nhãn `{label}` thiếu trong bảng cài đặt macOS"
            );
        }
    }

    #[test]
    fn crlf_conversion_round_trips() {
        let lf = "vn = Việt Nam\ncty = Công ty\n";
        assert_eq!(to_crlf(lf).replace("\r\n", "\n"), lf);
    }

    /// Lỗi đăng ký TSF: dialog phải hiện lý do THẬT từ `register.log` thay vì
    /// luôn đổ cho HKCU ACL. `read_log_tail` là hàm thuần path → test mọi OS.
    #[test]
    fn register_log_tail_reads_last_lines() {
        let dir = std::env::temp_dir().join(format!("textvn-reglog-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let log = dir.join("register.log");
        let content = (1..=20)
            .map(|i| format!("line-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&log, content).unwrap();
        let tail = read_log_tail(&log, 5).expect("phải đọc được log");
        assert_eq!(tail.lines().count(), 5, "chỉ lấy 5 dòng cuối");
        assert!(tail.contains("line-20"), "dòng mới nhất phải có");
        assert!(!tail.contains("line-14"), "dòng cũ hơn ngoài cửa sổ bị cắt");

        std::fs::write(
            &log,
            "[pid=1] === TextVN register (HKCU) ===\n[pid=1] Registry create → FAIL 0x00000005\n[pid=2] === TextVN register (HKCU) ===\n[pid=2] COM server HKCU → OK\n[pid=2] FAIL: hậu kiểm đăng ký TSF không đạt\n",
        )
        .unwrap();
        let latest = read_log_tail(&log, 12).unwrap();
        assert!(
            !latest.contains("0x00000005"),
            "không trộn lỗi của lần trước"
        );
        assert!(latest.contains("hậu kiểm đăng ký TSF không đạt"));

        std::fs::write(&log, "  \n\n").unwrap();
        assert!(read_log_tail(&log, 5).is_none(), "log rỗng → None");
        assert!(read_log_tail(&dir.join("missing.log"), 5).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Gợi ý phải khớp nguyên nhân THẬT (exit code 3 / ACL / thiếu DLL), kể cả
    /// định dạng log của bản cũ đã cài trên máy người dùng.
    #[test]
    fn advice_matches_real_cause() {
        assert!(advice_for_failure(3, "").contains("Administrator"));
        // Bản mới: log in tên ký hiệu kèm hex.
        assert!(advice_for_failure(
            1,
            "  Registry create HKCU\\Software\\Classes\\CLSID\\{6F2B…} → \
             FAIL 0x00000005 (ERROR_ACCESS_DENIED)"
        )
        .contains("Classes\\CLSID"));
        // Bản 0.2.0/0.2.1: log chỉ in hex — vẫn phải nhận gợi ý ACL đúng chỗ.
        assert!(advice_for_failure(
            1,
            "  Registry create Software\\Classes\\CLSID\\{…} → FAIL 0x00000005"
        )
        .contains("Classes\\CLSID"));
        assert!(advice_for_failure(1, "FAIL 0x0000002e").contains("register.log"));
        assert!(
            advice_for_failure(1, "Không tìm thấy textvn-tsf.dll (hoặc textvn_win_tsf.dll)")
                .contains("textvn-tsf.dll")
        );
        assert!(advice_for_failure(1, "lỗi lạ").contains("register.log"));
        let tsf_log = "COM server HKCU → OK\nĐăng ký qua API TSF → 0x80004005 (cần quyền admin cho HKLM)\nFAIL: hậu kiểm đăng ký TSF không đạt";
        assert!(advice_for_failure(1, tsf_log).contains("không phải lỗi quyền ghi HKCU"));
    }
}

#[cfg(test)]
mod word_list_tests {
    use super::normalize_word_list;

    /// Lần 1/3: chuẩn hoá cơ bản — trim, bỏ comment/trống, lowercase.
    #[test]
    fn normalize_trims_lowercases_and_skips_comments() {
        assert_eq!(
            normalize_word_list("# ghi chú\r\n\r\n  Text  \r\nTEST\n\ndownload\n"),
            vec![
                "text".to_string(),
                "test".to_string(),
                "download".to_string()
            ]
        );
        // Text/Text trùng thật → chỉ giữ một
        assert_eq!(
            normalize_word_list("Text\r\nTEXT\ntext\n"),
            vec!["text".to_string()]
        );
    }

    /// Lần 1/3: khử trùng lặp giữ thứ tự + bỏ từ có ký tự lạ.
    #[test]
    fn normalize_dedupes_and_drops_invalid() {
        assert_eq!(
            normalize_word_list(
                "cow
Cow
ców
co w
cow2
"
            ),
            vec!["cow".to_string()]
        );
    }

    /// Lần 1/3: rỗng/không có gì hợp lệ → danh sách rỗng (lưu được, xoá sạch).
    #[test]
    fn normalize_empty_input_yields_empty_list() {
        assert!(normalize_word_list("").is_empty());
        assert!(normalize_word_list(
            "# chỉ có ghi chú

"
        )
        .is_empty());
    }
}
