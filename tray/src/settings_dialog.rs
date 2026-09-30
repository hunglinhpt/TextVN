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
// Control IDs — cửa sổ Gõ tắt
const ID_EDIT_MACROS: isize = 2101;
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

    create_dialog_controls(hwnd, h_instance, dpi);
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

#[cfg(windows)]
fn scaled_gui_font(dpi: i32) -> HGDIOBJ {
    let existing = UI_FONT.load(Ordering::Acquire);
    if existing != 0 {
        // Cửa sổ Gõ tắt dùng lại font của bảng điều khiển (cùng DPI hệ thống).
        return HGDIOBJ(existing as *mut std::ffi::c_void);
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
    new_font
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
fn create_dialog_controls(parent: HWND, h_instance: HINSTANCE, dpi: i32) {
    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    let font = scaled_gui_font(dpi);

    const CHK: u32 = BS_AUTOCHECKBOX | WS_TABSTOP.0;
    const BTN: u32 = BS_PUSHBUTTON | WS_TABSTOP.0;
    const L: i32 = 32; // cột trái
    const R: i32 = 470; // cột phải
    const CW: i32 = 420; // bề rộng một ô trong cột
                         // Thứ tự tạo = thứ tự Tab. WS_GROUP mở nhóm radio mới và đóng nhóm trước đó.
    let layout = [
        // 1. Điều khiển
        ctl("BUTTON", "Điều khiển", BS_GROUPBOX, (15, 12, 870, 120), 0),
        ctl("STATIC", "Bảng mã:", 0, (L, 49, 85, 30), 0),
        ctl(
            "COMBOBOX",
            "",
            CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0 | WS_GROUP.0,
            (122, 46, 300, 200),
            ID_COMBO_CHARSET,
        ),
        ctl("STATIC", "Kiểu gõ:", 0, (R, 49, 85, 30), 0),
        ctl(
            "COMBOBOX",
            "",
            CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0,
            (560, 46, 300, 200),
            ID_COMBO_METHOD,
        ),
        ctl("STATIC", "Phím chuyển:", 0, (L, 93, 130, 30), 0),
        ctl(
            "STATIC",
            "Ctrl + Shift   (hoặc Ctrl + Shift + Space)",
            0,
            (170, 93, 690, 30),
            0,
        ),
        // 2. Tùy chọn gõ — 2 cột, 4 hàng
        ctl("BUTTON", "Tùy chọn gõ", BS_GROUPBOX, (15, 142, 870, 205), 0),
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
        ctl("BUTTON", "Hệ thống", BS_GROUPBOX, (15, 357, 870, 124), 0),
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
        ctl("BUTTON", "Hướng dẫn", BTN, (15, 497, 120, 48), ID_BTN_HELP),
        ctl(
            "BUTTON",
            "Thông tin",
            BTN,
            (143, 497, 120, 48),
            ID_BTN_ABOUT,
        ),
        ctl(
            "BUTTON",
            "Gõ tắt...",
            BTN,
            (271, 497, 120, 48),
            ID_BTN_MACROS,
        ),
        ctl(
            "BUTTON",
            "Cài & bật TSF",
            BTN,
            (399, 497, 160, 48),
            ID_BTN_SETUP_TSF,
        ),
        ctl(
            "BUTTON",
            "Mặc định",
            BTN,
            (567, 497, 105, 48),
            ID_BTN_DEFAULT,
        ),
        // Đóng đưa cửa sổ về khay; Kết thúc tắt hẳn ứng dụng.
        ctl(
            "BUTTON",
            "Đóng",
            BS_DEFPUSHBUTTON | WS_TABSTOP.0,
            (680, 497, 100, 48),
            ID_BTN_CLOSE,
        ),
        ctl("BUTTON", "Kết thúc", BTN, (788, 497, 97, 48), ID_BTN_EXIT),
    ];
    for c in &layout {
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
                         Phát triển bởi: hunglinhpt\r\n\
                         Giấy phép: GPL-3.0-or-later\r\n\
                         https://github.com/hunglinhpt/TextVN",
                        env!("CARGO_PKG_VERSION")
                    ),
                ),
                ID_BTN_MACROS => show_macro_editor(hwnd),
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

    let scale = COMPACT_SCALE * dpi as f64 / 96.0;
    let k = |v: i32| -> i32 { ((v as f64) * scale).round() as i32 };
    let font = scaled_gui_font(dpi);
    let add = |ex, class, text: &str, style, (x, y, w, h), id| {
        let c = create_control_ex(
            ex,
            class,
            text,
            style,
            (k(x), k(y), k(w), k(h)),
            hwnd,
            id,
            h_instance,
        );
        set_font(c, font);
        c
    };
    let none = WINDOW_EX_STYLE::default();
    add(
        none,
        "STATIC",
        "Mỗi dòng một mục:   gõ tắt = nội dung      (ví dụ:  vn = Việt Nam)\r\n\
         Dòng bắt đầu bằng # là ghi chú.  \\n trong nội dung = xuống dòng.  Tối đa 64 ký tự.",
        0,
        (20, 12, 680, 64),
        0,
    );
    let edit = add(
        WS_EX_CLIENTEDGE,
        "EDIT",
        &to_crlf(&text),
        ES_MULTILINE
            | ES_AUTOVSCROLL
            | ES_WANTRETURN
            | ES_NOHIDESEL
            | WS_VSCROLL.0
            | WS_TABSTOP.0
            | WS_GROUP.0,
        (20, 84, 680, 370),
        ID_EDIT_MACROS,
    );
    // Mặc định EDIT giới hạn 32K ký tự — đủ, nhưng nâng lên để không cắt bảng lớn.
    unsafe {
        let _ = SendMessageW(
            edit,
            EM_SETLIMITTEXT,
            Some(WPARAM(1 << 20)),
            Some(LPARAM(0)),
        );
    }
    add(
        none,
        "STATIC",
        "Bung gõ tắt bằng phím:",
        0,
        (20, 474, 230, 30),
        0,
    );
    add(
        none,
        "BUTTON",
        "Tab",
        BS_AUTORADIOBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (255, 470, 90, 32),
        ID_RAD_TRIGGER_TAB,
    );
    add(
        none,
        "BUTTON",
        "Space",
        BS_AUTORADIOBUTTON,
        (350, 470, 110, 32),
        ID_RAD_TRIGGER_SPACE,
    );
    add(
        none,
        "BUTTON",
        "Lưu",
        BS_DEFPUSHBUTTON | WS_TABSTOP.0 | WS_GROUP.0,
        (480, 500, 105, 46),
        ID_BTN_MACRO_SAVE,
    );
    add(
        none,
        "BUTTON",
        "Hủy",
        BS_PUSHBUTTON | WS_TABSTOP.0,
        (595, 500, 105, 46),
        ID_BTN_MACRO_CANCEL,
    );
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
        Some(Ok(())) | None => true,
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

#[cfg(windows)]
unsafe extern "system" fn macro_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
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
    let start = lines.len().saturating_sub(max_lines);
    Some(lines[start..].join("\r\n"))
}

/// Gợi ý theo nguyên nhân THẬT trong log/exit code (giữ cả gợi ý HKCU cho lỗi
/// ACL thật — `ERROR_ACCESS_DENIED (5)` khi tài khoản bị policy chặn ghi).
#[cfg_attr(not(windows), allow(dead_code))]
fn advice_for_failure(exit_code: i32, detail: &str) -> String {
    if exit_code == 3 {
        return "Thiếu quyền Administrator cho phạm vi máy (--scope machine). \
                Hãy bấm [Cài & bật TSF] từ tài khoản thường (đăng ký per-user không cần admin)."
            .to_string();
    }
    if detail.contains("ACCESS_DENIED") {
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
        ] {
            assert!(
                linux.contains(label),
                "nhãn `{label}` thiếu trong bảng cài đặt Linux"
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

        std::fs::write(&log, "  \n\n").unwrap();
        assert!(read_log_tail(&log, 5).is_none(), "log rỗng → None");
        assert!(read_log_tail(&dir.join("missing.log"), 5).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Gợi ý phải khớp nguyên nhân THẬT (exit code 3 / ACL / thiếu DLL).
    #[test]
    fn advice_matches_real_cause() {
        assert!(advice_for_failure(3, "").contains("Administrator"));
        assert!(advice_for_failure(
            1,
            "RegCreateKeyExW HKCU\\...\\CLSID → ERROR_ACCESS_DENIED (5)"
        )
        .contains("Classes\\CLSID"));
        assert!(
            advice_for_failure(1, "Không tìm thấy textvn-tsf.dll (hoặc textvn_win_tsf.dll)")
                .contains("textvn-tsf.dll")
        );
        assert!(advice_for_failure(1, "lỗi lạ").contains("register.log"));
    }
}
