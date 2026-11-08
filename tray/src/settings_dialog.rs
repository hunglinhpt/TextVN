// SPDX-License-Identifier: GPL-3.0-or-later
//! Giao diện Bảng điều khiển kiểu UniKey/EVKey thuần Win32 (WIN-052 / P1-4 §3).
//!
//! Thiết kế native Win32 GUI:
//! - Khởi động tức thì (<5ms), không tốn RAM, không cần framework nặng.
//! - Bảng điều khiển chuẩn: Kiểu gõ, Bảng mã, Tùy chọn gõ, Bỏ dấu mới/cũ, Khởi động cùng Windows.
//! - Nút "Ẩn vào khay" hoặc nút [X]/Esc ẩn về khay (bộ gõ vẫn chạy); nút "Thoát" tắt hẳn.
//! - Đồng bộ trạng thái tức thì với SvcManager và broadcast IPC reload.

use std::sync::atomic::AtomicIsize;
#[cfg(windows)]
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[cfg(windows)]
use textvn_config::{AssistMode, DiacriticStyle, DocError, MacroTrigger, Method, OutputCharset};

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
const CLIENT_DESIGN_HEIGHT: i32 = 798;
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
const ID_CHK_AUTO_CAPITALIZE: isize = 2016;
const ID_CHK_QUICK_TELEX: isize = 2017;
const ID_CHK_MACRO_WHEN_OFF: isize = 2018;
const ID_CHK_SUGGEST: isize = 2025;
const ID_CHK_SHOW_ON_STARTUP: isize = 2019;
const ID_BTN_MACROS: isize = 2020;
const ID_CHK_CTRL_SHIFT: isize = 2021;
const ID_BTN_ENGLISH: isize = 2022;
const ID_COMBO_ASSIST_MODE: isize = 2023;
const ID_LBL_ASSIST_HINT: isize = 4103;
const ID_LBL_WORDLIST_HINT: isize = 4101;
// P7/P10 (0.2.74): tự học + Dream RSI nằm trong khung Trợ lý ngữ pháp.
const ID_CHK_STYLE: isize = 2026;
const ID_BTN_STYLE_CLEAR: isize = 2027;
const ID_LBL_STYLE_STATS: isize = 4102;
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
// CÙNG nhãn + thứ tự với Linux (`settings_model.c`) và macOS (`SettingsView.swift`) —
// test `labels_match_linux_settings_panel` đối chiếu cả ba.
const ASSIST_MODE_LABELS: [&str; 3] = ["Tắt", "Chỉ gợi ý", "Tự sửa lỗi an toàn"];
/// Mô tả một dòng cho từng chế độ (cùng chữ Windows/Linux/macOS — test
/// `assist_mode_hints_match_every_platform`). Thay cho ô "Tự động sửa" từng đứng cạnh ô Chế độ:
/// "Tự sửa lỗi an toàn" luôn chỉ sửa lỗi chắc chắn nên không cần lựa chọn thứ hai (0.2.75).
pub const ASSIST_MODE_HINTS: [&str; 3] = [
    "Không kiểm tra ngữ pháp và chính tả.",
    "Dừng gõ 1 giây, panel hiện lỗi và gợi ý; bạn tự chọn có sửa hay không.",
    "Như Chỉ gợi ý, và tự sửa lỗi chắc chắn (lặp từ, thừa dấu cách, viết hoa đầu câu…) khi bạn kết thúc câu; Ctrl+Z hoàn tác trong 5 giây.",
];

/// Popup "Thông tin" — CÙNG nội dung trên Windows/Linux/macOS (test
/// `about_text_matches_every_platform`): một sản phẩm đa nền tảng, không còn câu
/// riêng "cho Windows/macOS/Linux" ở mỗi bản.
const ABOUT_TAGLINE: &str = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)";
const ABOUT_DETAILS: [&str; 3] = [
    "Phát triển bởi: LinhBH.CoM",
    "Giấy phép: GPL-3.0-or-later",
    "https://github.com/hunglinhpt/TextVN",
];

fn about_text(version: &str) -> String {
    format!(
        "TextVN {version}\r\n{ABOUT_TAGLINE}\r\n\r\n{}",
        ABOUT_DETAILS.join("\r\n")
    )
}

const HELP_TEXT: &str = "Bật/tắt tiếng Việt: nhấn rồi nhả Ctrl + Shift (không kèm phím khác), \
hoặc Ctrl + Shift + Space, hoặc bấm biểu tượng V/E ở khay hệ thống.\r\n\r\n\
Telex: aa=â  aw=ă  ee=ê  oo=ô  ow=ơ  uw=ư  dd=đ;  s f r x j = sắc huyền hỏi ngã nặng;  z = xoá dấu.\r\n\
VNI: 1-5 = sắc huyền hỏi ngã nặng;  6 = mũ (â ê ô);  7 = móc (ơ ư);  8 = trăng (ă);  9 = đ;  0 = xoá dấu.\r\n\
Quick Telex: cc=ch  gg=gi  kk=kh  nn=ng  qq=qu  pp=ph  tt=th.\r\n\r\n\
Gõ tắt: bấm [Gõ tắt...], mỗi dòng ghi  gõ tắt = nội dung; khi gõ, nhập chữ tắt rồi nhấn Tab \
(hoặc Space) để bung ra.\r\n\r\n\
Ẩn vào khay: đóng cửa sổ này nhưng TextVN vẫn chạy nền (bộ gõ vẫn hoạt động); mở lại từ biểu \
tượng ở khay. Thoát: tắt hẳn TextVN.\r\n\r\n\
Nếu chưa gõ được tiếng Việt: TextVN tự đăng ký bộ gõ khi khởi động — thoát hẳn rồi mở lại, \
chọn TextVN trong danh sách bộ gõ (Win + Space), hoặc chạy `textvn-cli doctor`.";

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

/// Font GUI 9pt theo TỪNG DPI (R2-39): Bảng điều khiển và cửa sổ "Gõ tắt..."/"Từ điển
/// EN" có thể nằm trên hai màn hình khác DPI cùng lúc. Bản cũ giữ MỘT font và xoá nó khi
/// DPI khác được yêu cầu — control của cửa sổ kia còn đang dùng font đó nên rơi về font
/// System, chữ bị cắt. Số DPI khác nhau rất ít (≤ số màn hình) → giữ tới khi Bảng điều
/// khiển bị huỷ (cửa sổ con thuộc nó bị huỷ trước).
#[cfg(windows)]
static UI_FONTS: std::sync::Mutex<Vec<(i32, isize)>> = std::sync::Mutex::new(Vec::new());

/// Font GUI 9pt cho DPI yêu cầu (tạo một lần cho mỗi DPI); lỗi thì dùng stock font.
#[cfg(windows)]
fn scaled_gui_font(dpi: i32) -> HGDIOBJ {
    let stock = || unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    let Ok(mut fonts) = UI_FONTS.lock() else {
        return stock();
    };
    if let Some(&(_, font)) = fonts.iter().find(|(d, _)| *d == dpi) {
        return HGDIOBJ(font as *mut std::ffi::c_void);
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
    fonts.push((dpi, new_font.0 as isize));
    new_font
}

/// Xoá mọi font đã tạo — chỉ gọi khi Bảng điều khiển bị huỷ.
#[cfg(windows)]
fn delete_ui_fonts() {
    if let Ok(mut fonts) = UI_FONTS.lock() {
        for (_, font) in fonts.drain(..) {
            let _ = unsafe { DeleteObject(HGDIOBJ(font as *mut std::ffi::c_void)) };
        }
    }
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
const DIALOG_LAYOUT: [Ctl; 35] = [
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
        (15, 142, 870, 247),
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
    // P6 (parity Windows/macOS/Linux): dự đoán văn bản opt-in — thiếu toggle
    // này làm dự đoán KHÔNG THỂ bật (bug 0.2.49).
    ctl(
        "BUTTON",
        "Gợi ý hoàn tất từ (Tab / Ctrl+Space)",
        CHK,
        (L, 344, 830, 32),
        ID_CHK_SUGGEST,
    ),
    // 3. Trợ lý ngữ pháp
    ctl(
        "BUTTON",
        "Trợ lý ngữ pháp",
        BS_GROUPBOX,
        (15, 399, 870, 192),
        ID_LBL_BASE + 7,
    ),
    ctl("STATIC", "Chế độ:", 0, (L, 437, 95, 30), ID_LBL_BASE + 8),
    ctl(
        "COMBOBOX",
        "",
        CBS_DROPDOWNLIST | WS_VSCROLL.0 | WS_TABSTOP.0 | WS_GROUP.0,
        (135, 434, 735, 200),
        ID_COMBO_ASSIST_MODE,
    ),
    // Một lựa chọn duy nhất ở trên; dòng này giải thích chế độ đang chọn (đổi theo lựa chọn).
    ctl("STATIC", "", 0, (L, 474, 838, 30), ID_LBL_ASSIST_HINT),
    // Tự học (P7) + Dream RSI (P10): công tắt, nút xoá dữ liệu, dòng số liệu. Bật mặc định
    // (0.2.74); chỉ học trên máy này, không học ở ô mật khẩu/terminal.
    ctl(
        "BUTTON",
        STYLE_CHECKBOX_LABEL,
        CHK,
        (L, 517, 600, 32),
        ID_CHK_STYLE,
    ),
    ctl(
        "BUTTON",
        STYLE_CLEAR_LABEL,
        BTN,
        (650, 513, 220, 38),
        ID_BTN_STYLE_CLEAR,
    ),
    ctl("STATIC", "", 0, (L, 557, 838, 28), ID_LBL_STYLE_STATS),
    // 4. Hệ thống (chỉ Windows — Linux do IBus/Fcitx5 tự khởi động)
    ctl(
        "BUTTON",
        "Hệ thống",
        BS_GROUPBOX,
        (15, 601, 870, 124),
        ID_LBL_BASE + 6,
    ),
    ctl(
        "BUTTON",
        "Khởi động cùng Windows",
        CHK,
        (L, 635, CW, 32),
        ID_CHK_AUTOSTART,
    ),
    ctl(
        "BUTTON",
        "Bật hội thoại này khi khởi động",
        CHK,
        (R, 635, 400, 32),
        ID_CHK_SHOW_ON_STARTUP,
    ),
    ctl(
        "BUTTON",
        "Dành Ctrl + Shift cho TextVN (tắt phím đổi bàn phím của Windows)",
        CHK,
        (L, 677, 830, 32),
        ID_CHK_CTRL_SHIFT,
    ),
    // 5. Hàng nút: thông tin/công cụ (trái) · thao tác (phải). Mỗi nút có biểu tượng + gợi ý
    // (`BUTTON_HINTS`). "Ẩn vào khay" ≠ "Thoát": tên cũ "Đóng"/"Kết thúc" khiến người dùng
    // tưởng Đóng là tắt ứng dụng.
    ctl(
        "BUTTON",
        "\u{2753}  Hướng dẫn",
        BTN,
        (15, 735, 118, 48),
        ID_BTN_HELP,
    ),
    ctl(
        "BUTTON",
        "\u{2139}  Thông tin",
        BTN,
        (139, 735, 112, 48),
        ID_BTN_ABOUT,
    ),
    ctl(
        "BUTTON",
        "\u{2328}  Gõ tắt...",
        BTN,
        (257, 735, 112, 48),
        ID_BTN_MACROS,
    ),
    // Cùng vị trí tương đối như Linux (hàng nút, sau "Gõ tắt...") — 3 nền tảng thống nhất.
    ctl(
        "BUTTON",
        "\u{270E}  Từ điển EN...",
        BTN,
        (375, 735, 140, 48),
        ID_BTN_ENGLISH,
    ),
    ctl(
        "BUTTON",
        "\u{21BA}  Mặc định",
        BTN,
        (521, 735, 100, 48),
        ID_BTN_DEFAULT,
    ),
    // Ẩn vào khay đưa cửa sổ về khay (bộ gõ vẫn chạy); Thoát tắt hẳn ứng dụng.
    ctl(
        "BUTTON",
        "\u{2B07}  Ẩn vào khay",
        BS_DEFPUSHBUTTON | WS_TABSTOP.0,
        (632, 735, 150, 48),
        ID_BTN_CLOSE,
    ),
    ctl(
        "BUTTON",
        "\u{23FB}  Thoát",
        BTN,
        (788, 735, 98, 48),
        ID_BTN_EXIT,
    ),
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

/// Gợi ý (tooltip) của hàng nút — người dùng phải phân biệt được "Ẩn vào khay" và "Thoát".
#[cfg(windows)]
const BUTTON_HINTS: [(isize, &str); 7] = [
    (ID_BTN_HELP, "Cách gõ Telex/VNI, gõ tắt và các phím tắt"),
    (ID_BTN_ABOUT, "Phiên bản và thông tin TextVN"),
    (ID_BTN_MACROS, "Thêm/sửa gõ tắt (ví dụ hd = hợp đồng)"),
    (
        ID_BTN_ENGLISH,
        "Từ tiếng Anh luôn giữ nguyên khi gõ (ví dụ text, cow)",
    ),
    (
        ID_BTN_DEFAULT,
        "Đặt mọi tuỳ chọn về mặc định (không xoá gõ tắt)",
    ),
    (
        ID_BTN_CLOSE,
        "Đóng cửa sổ này, TextVN VẪN CHẠY nền ở khay hệ thống và bộ gõ vẫn hoạt động. \
         (Nút X và phím Esc cũng làm vậy.)",
    ),
    (
        ID_BTN_EXIT,
        "Tắt hẳn TextVN, kể cả biểu tượng ở khay. Muốn giữ bộ gõ chạy nền thì bấm Ẩn vào khay.",
    ),
];

/// Tooltip cho hàng nút (`TTF_SUBCLASS`: control tự chuyển chuột cho tooltip).
#[cfg(windows)]
fn add_button_tooltips(parent: HWND, h_instance: HINSTANCE) {
    use windows::Win32::UI::Controls::{
        TOOLTIPS_CLASSW, TTDT_INITIAL, TTF_IDISHWND, TTF_SUBCLASS, TTM_ADDTOOLW, TTM_SETDELAYTIME,
        TTM_SETMAXTIPWIDTH, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
    };
    let tip = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASSW,
            PCWSTR::null(),
            WINDOW_STYLE(WS_POPUP.0 | TTS_ALWAYSTIP | TTS_NOPREFIX),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Some(parent),
            None,
            Some(h_instance),
            None,
        )
    };
    let Ok(tip) = tip else {
        return;
    };
    unsafe {
        let _ = SendMessageW(tip, TTM_SETMAXTIPWIDTH, Some(WPARAM(0)), Some(LPARAM(380)));
        let _ = SendMessageW(
            tip,
            TTM_SETDELAYTIME,
            Some(WPARAM(TTDT_INITIAL as usize)),
            Some(LPARAM(300)),
        );
    }
    for (id, text) in BUTTON_HINTS {
        let Ok(button) = (unsafe { GetDlgItem(Some(parent), id as i32) }) else {
            continue;
        };
        let mut wide = w(text);
        let info = TTTOOLINFOW {
            cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
            uFlags: TTF_IDISHWND | TTF_SUBCLASS,
            hwnd: parent,
            uId: button.0 as usize,
            lpszText: PWSTR(wide.as_mut_ptr()),
            ..Default::default()
        };
        // TTM_ADDTOOL chép chuỗi — `wide` chỉ cần sống tới hết lời gọi.
        unsafe {
            let _ = SendMessageW(
                tip,
                TTM_ADDTOOLW,
                Some(WPARAM(0)),
                Some(LPARAM(&info as *const TTTOOLINFOW as isize)),
            );
        }
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
    add_button_tooltips(parent, h_instance);

    for (id, labels) in [
        (ID_COMBO_CHARSET, &CHARSET_LABELS[..]),
        (ID_COMBO_METHOD, &METHOD_LABELS[..]),
        (ID_COMBO_ASSIST_MODE, &ASSIST_MODE_LABELS[..]),
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
        set_chk(hwnd, ID_CHK_SUGGEST, cfg.suggest.enabled);
        set_chk(hwnd, ID_CHK_STYLE, cfg.style_profile.effective_enabled());
        refresh_style_stats(hwnd, &cfg.style_profile);
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

        let assist_mode_idx = match cfg.assist.mode {
            AssistMode::Off => 0,
            AssistMode::Highlight => 1,
            AssistMode::Autocorrect => 2,
        };
        send_dlg_msg(hwnd, ID_COMBO_ASSIST_MODE, CB_SETCURSEL, assist_mode_idx, 0);
        set_assist_hint(hwnd, assist_mode_idx);
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
    // R2-38: cả cửa sổ "Từ điển EN" — thiếu nó thì Tab chèn ký tự Tab vào ô soạn (từ đó
    // bị bỏ khi chuẩn hoá lúc lưu) và Esc không đóng cửa sổ.
    for slot in [&WORDLIST_HWND, &MACRO_HWND, &SETTINGS_HWND] {
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
        ID_CHK_SUGGEST => svc.set_suggest_enabled(checked),
        ID_CHK_STYLE => svc.set_style_learning(checked),
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
                    &about_text(env!("CARGO_PKG_VERSION")),
                ),
                ID_BTN_MACROS => show_macro_editor(hwnd),
                ID_BTN_ENGLISH => show_word_list_editor(hwnd),
                ID_BTN_STYLE_CLEAR => {
                    // Xoá không hoàn tác được → hỏi lại (mặc định "Không").
                    let title = w("Xóa dữ liệu đã học?");
                    let body = w(STYLE_CLEAR_CONFIRM);
                    let answer = MessageBoxW(
                        Some(hwnd),
                        PCWSTR(body.as_ptr()),
                        PCWSTR(title.as_ptr()),
                        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
                    );
                    if answer == IDYES
                        && save_config_change(hwnd, |ctx| ctx.svc.clear_learned_style())
                    {
                        populate_controls_from_config(hwnd);
                    }
                }

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
                ID_COMBO_ASSIST_MODE if notify_code == CBN_SELCHANGE => {
                    let sel = send_dlg_msg(hwnd, ID_COMBO_ASSIST_MODE, CB_GETCURSEL, 0, 0).0;
                    let mode = match sel {
                        1 => AssistMode::Highlight,
                        2 => AssistMode::Autocorrect,
                        _ => AssistMode::Off,
                    };
                    let _ = save_config_change(hwnd, |ctx| ctx.svc.set_assist_mode(mode));
                    set_assist_hint(hwnd, sel.clamp(0, 2) as usize);
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
                    // R2-40: ghi lại giá trị cũ để gỡ cài đặt trả đúng; bỏ chọn trả theo
                    // bản ghi (cả Language Hotkey) rồi chắc chắn Windows giữ Ctrl + Shift.
                    let result = if get_chk(hwnd, ID_CHK_CTRL_SHIFT) {
                        crate::hotkey::free_ctrl_shift().map(crate::record_ctrl_shift_freed)
                    } else {
                        crate::give_ctrl_shift_back_to_windows()
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
            delete_ui_fonts();
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

/// Nhãn công tắt tự học — CÙNG chữ trên Windows/Linux/macOS (test `labels_match_linux_settings_panel`).
pub const STYLE_CHECKBOX_LABEL: &str = "Tự học từ và cách sửa bạn hay dùng (chỉ trên máy này)";
/// Nhãn nút xoá dữ liệu học — cùng chữ 3 nền tảng.
pub const STYLE_CLEAR_LABEL: &str = "Xóa dữ liệu đã học";
/// Hộp thoại xác nhận xoá — cùng nội dung 3 nền tảng.
pub const STYLE_CLEAR_CONFIRM: &str = "Xóa toàn bộ từ, cặp sửa và số liệu Dream RSI đã học trên máy này? Việc này không hoàn tác được. Gõ tắt và từ điển EN của bạn được giữ nguyên.";

/// Dòng số liệu dưới công tắt tự học (cùng định dạng 3 nền tảng): số mục đã học + Dream RSI.
pub fn style_stats_text(
    words: usize,
    corrections: usize,
    rsi: &textvn_config::RsiTotals,
) -> String {
    format!(
        "Đã học: {words} từ · {corrections} cặp sửa  |  Dream RSI: tiết kiệm {}% số phím ({} ký tự)",
        rsi.savings_rate_pct(),
        rsi.chars_saved
    )
}

/// Dòng mô tả dưới ô Chế độ theo chế độ đang chọn (0 = Tắt, 1 = Chỉ gợi ý, 2 = Tự sửa lỗi an toàn).
#[cfg(windows)]
fn set_assist_hint(hwnd: HWND, mode_idx: usize) {
    let text = w(ASSIST_MODE_HINTS[mode_idx.min(ASSIST_MODE_HINTS.len() - 1)]);
    if let Ok(h) = unsafe { GetDlgItem(Some(hwnd), ID_LBL_ASSIST_HINT as i32) } {
        let _ = unsafe { SetWindowTextW(h, PCWSTR(text.as_ptr())) };
    }
}

/// Cập nhật dòng số liệu + trạng thái nút xoá theo hồ sơ hiện tại.
#[cfg(windows)]
fn refresh_style_stats(hwnd: HWND, profile: &textvn_config::StyleProfileSettings) {
    let text = w(&style_stats_text(
        profile.words.len(),
        profile.corrections.len(),
        &profile.rsi,
    ));
    if let Ok(h) = unsafe { GetDlgItem(Some(hwnd), ID_LBL_STYLE_STATS as i32) } {
        let _ = unsafe { SetWindowTextW(h, PCWSTR(text.as_ptr())) };
    }
    let has_data =
        !profile.words.is_empty() || !profile.corrections.is_empty() || !profile.rsi.is_zero();
    if let Ok(h) = unsafe { GetDlgItem(Some(hwnd), ID_BTN_STYLE_CLEAR as i32) } {
        let _ = unsafe { EnableWindow(h, has_data) };
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
            ID_CHK_AUTO_CAPITALIZE,
            ID_CHK_QUICK_TELEX,
            ID_CHK_MACRO_WHEN_OFF,
            ID_CHK_SHOW_ON_STARTUP,
            ID_BTN_MACROS,
            ID_COMBO_ASSIST_MODE,
            ID_LBL_ASSIST_HINT,
            ID_CHK_STYLE,
            ID_BTN_STYLE_CLEAR,
            ID_LBL_STYLE_STATS,
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
        // R2-106: bảng mã/kiểu gõ phải CÙNG nhãn VÀ CÙNG thứ tự trên cả Linux lẫn macOS
        // (trước chỉ kiểm Linux — macOS lệch từ đầu: "1. Unicode dựng sẵn", TCVN3 đứng
        // thứ 2, "4. Microsoft" thay "Telex đơn giản").
        let same_order = |src: &str, labels: &[&str], wrap: &dyn Fn(&str) -> String, os: &str| {
            let mut last = 0usize;
            for label in labels {
                let pos = src
                    .find(&wrap(label))
                    .unwrap_or_else(|| panic!("nhãn `{label}` thiếu trong bảng cài đặt {os}"));
                assert!(
                    pos >= last,
                    "nhãn `{label}` sai thứ tự trong bảng cài đặt {os}"
                );
                last = pos;
            }
        };
        let quoted = |l: &str| format!("\"{l}\"");
        let swift_text = |l: &str| format!("Text(\"{l}\")");
        for labels in [
            &CHARSET_LABELS[..],
            &METHOD_LABELS[..],
            &ASSIST_MODE_LABELS[..],
        ] {
            same_order(&linux, labels, &quoted, "Linux");
            same_order(macos, labels, &swift_text, "macOS");
        }
        // Khung + nhãn hàng của Trợ lý ngữ pháp (AS-060, parity-checklist §58).
        for label in ["Trợ lý ngữ pháp", "Chế độ:"] {
            assert!(
                linux.contains(&format!("\"{label}\"")),
                "thiếu `{label}` (Linux)"
            );
            assert!(
                macos.contains(&format!("Text(\"{label}\")")),
                "thiếu `{label}` (macOS)"
            );
        }
        // Phím chuyển CỐ ĐỊNH (ui-spec.md §1) — macOS từng có 3 nút riêng
        // Ctrl + Shift / Alt + Z / Caps Lock mà Windows/Linux không có.
        let switch_hint = "Ctrl + Shift   (hoặc Ctrl + Shift + Space)";
        for (os, src) in [("Linux", linux.as_str()), ("macOS", macos)] {
            assert!(
                src.contains("Phím chuyển:"),
                "thiếu hàng Phím chuyển ({os})"
            );
            assert!(
                src.contains(switch_hint),
                "Phím chuyển phải ghi `{switch_hint}` ({os})"
            );
        }
        assert!(
            !macos.contains("\"Alt + Z\"") && !macos.contains("\"Caps Lock\""),
            "macOS không được có lựa chọn phím chuyển riêng (R2-106)"
        );
        for label in [
            "Bật gõ tiếng Việt",
            "Dấu mới (hoà, thuỷ)",
            "Dấu cũ (hòa, thủy)",
            "Khôi phục từ tiếng Anh khi gõ sai",
            "Đặt dấu tự do",
            "Tự viết hoa chữ đầu câu",
            "Quick Telex (cc→ch, nn→ng…)",
            "Gợi ý hoàn tất từ (Tab / Ctrl+Space)",
            "Gõ tắt cả khi tắt tiếng Việt",
            "Gõ tắt...",
            "Từ điển EN...",
            STYLE_CHECKBOX_LABEL,
            STYLE_CLEAR_LABEL,
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

    /// 0.2.75: không còn ô "Tự động sửa" — ba chế độ có cùng mô tả trên 3 OS và không OS nào còn nhãn cũ.
    #[test]
    fn assist_mode_hints_match_every_platform() {
        let linux = [
            include_str!("../../adapters/linux-settings/src/settings_model.c"),
            include_str!("../../adapters/linux-settings/src/settings_window.c"),
        ]
        .concat();
        let mac = include_str!("../../adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift");
        for hint in ASSIST_MODE_HINTS {
            assert!(linux.contains(hint), "Linux thiếu mô tả `{hint}`");
            assert!(mac.contains(hint), "macOS thiếu mô tả `{hint}`");
        }
        for src in [linux.as_str(), mac] {
            assert!(!src.contains("Tự động sửa:"), "còn nhãn `Tự động sửa:`");
            assert!(!src.contains("An toàn (lỗi khách quan)"));
        }
        #[cfg(windows)]
        assert!(!DIALOG_LAYOUT.iter().any(|c| c.text == "Tự động sửa:"));
        assert_eq!(ASSIST_MODE_LABELS[2], "Tự sửa lỗi an toàn");
    }

    /// Dòng số liệu tự học + Dream RSI: cùng định dạng 3 nền tảng (macOS dựng ở `ConfigModel.swift`).
    #[test]
    fn style_stats_line_matches_every_platform() {
        let linux = include_str!("../../adapters/linux-settings/src/settings_window.c");
        let mac = include_str!("../../adapters/macos-app/Sources/TextVNAppLib/ConfigModel.swift");
        for part in [
            "Đã học:",
            "từ ·",
            "cặp sửa",
            "Dream RSI: tiết kiệm",
            "số phím",
            "ký tự)",
        ] {
            assert!(linux.contains(part), "Linux thiếu `{part}`");
            assert!(mac.contains(part), "macOS thiếu `{part}`");
        }
        let rsi = textvn_config::RsiTotals {
            keys_typed: 60,
            accepts: 4,
            chars_saved: 40,
        };
        assert_eq!(
            style_stats_text(1, 0, &rsi),
            "Đã học: 1 từ · 0 cặp sửa  |  Dream RSI: tiết kiệm 40% số phím (40 ký tự)"
        );
        assert_eq!(
            style_stats_text(0, 0, &textvn_config::RsiTotals::default()),
            "Đã học: 0 từ · 0 cặp sửa  |  Dream RSI: tiết kiệm 0% số phím (0 ký tự)"
        );
        // Hộp thoại xác nhận cùng nội dung 3 nền tảng.
        assert!(linux.contains("Xóa toàn bộ từ, cặp sửa và số liệu Dream RSI đã học trên máy này?"));
        assert!(
            include_str!("../../adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift")
                .contains("Xóa toàn bộ từ, cặp sửa và số liệu Dream RSI đã học trên máy này?")
        );
        assert!(STYLE_CLEAR_CONFIRM
            .contains("Xóa toàn bộ từ, cặp sửa và số liệu Dream RSI đã học trên máy này?"));
    }

    /// Popup "Thông tin": CÙNG câu giới thiệu + chi tiết trên 3 nền tảng (yêu cầu chủ
    /// repo 2026-10-11 — sản phẩm đa nền tảng, không còn "cho Windows/macOS/Linux").
    #[test]
    fn about_text_matches_every_platform() {
        let linux = include_str!("../../adapters/linux-settings/src/settings_model.c");
        let mac_info = include_str!("../../adapters/macos-app/Sources/TextVNAppLib/AppInfo.swift");
        let mac_settings =
            include_str!("../../adapters/macos-app/Sources/TextVNAppLib/SettingsView.swift");
        let mac_delegate =
            include_str!("../../adapters/macos-app/Sources/TextVNAppLib/AppDelegate.swift");
        let windows = about_text("9.9.9");
        assert!(windows.starts_with("TextVN 9.9.9\r\n"));
        for part in std::iter::once(ABOUT_TAGLINE).chain(ABOUT_DETAILS) {
            assert!(windows.contains(part), "Windows thiếu `{part}`");
            assert!(linux.contains(part), "Linux thiếu `{part}`");
            assert!(mac_info.contains(part), "macOS thiếu `{part}`");
        }
        // Hai popup macOS (Settings + menu bar) cùng đọc AppInfo — không chép chuỗi riêng.
        for src in [mac_settings, mac_delegate] {
            assert!(src.contains("AppInfo.aboutMessage"));
        }
        for (os, src) in [
            ("Windows", windows.as_str()),
            ("Linux", linux),
            ("macOS", mac_settings),
            ("macOS", mac_delegate),
        ] {
            assert!(
                !src.contains("Bộ gõ tiếng Việt cho "),
                "{os} còn câu giới thiệu riêng theo nền tảng"
            );
        }
    }

    /// Hàng nút (0.2.72): mỗi nút có biểu tượng + gợi ý; "Ẩn vào khay" và "Thoát" không còn dễ
    /// nhầm là cùng tắt ứng dụng; không còn nút "Cài & bật TSF" (tray tự đăng ký khi khởi động).
    #[cfg(windows)]
    #[test]
    fn button_row_has_icons_hints_and_no_tsf_button() {
        let row = [
            (ID_BTN_HELP, "Hướng dẫn"),
            (ID_BTN_ABOUT, "Thông tin"),
            (ID_BTN_MACROS, "Gõ tắt..."),
            (ID_BTN_ENGLISH, "Từ điển EN..."),
            (ID_BTN_DEFAULT, "Mặc định"),
            (ID_BTN_CLOSE, "Ẩn vào khay"),
            (ID_BTN_EXIT, "Thoát"),
        ];
        let find = |id: isize| {
            DIALOG_LAYOUT
                .iter()
                .find(|c| c.id == id)
                .expect("thiếu nút")
        };
        for (id, label) in row {
            let text = find(id).text;
            let first = text.chars().next().unwrap();
            assert!(!first.is_ascii(), "`{text}` thiếu biểu tượng đầu nhãn");
            assert!(
                text.split_once("  ").is_some_and(|(_, rest)| rest == label),
                "{text}"
            );
            let hint = BUTTON_HINTS.iter().find(|(h, _)| *h == id).map(|(_, t)| *t);
            assert!(
                hint.is_some_and(|t| t.chars().count() > 15),
                "nút `{label}` thiếu gợi ý"
            );
        }
        assert_eq!(BUTTON_HINTS.len(), row.len());
        for c in &DIALOG_LAYOUT {
            assert!(!c.text.contains("TSF"), "còn nút/nhãn TSF: {}", c.text);
            assert_ne!(
                c.text.trim(),
                "Đóng",
                "nhãn `Đóng` gây hiểu nhầm là tắt ứng dụng"
            );
            assert_ne!(c.text.trim(), "Kết thúc");
        }
        let hint = |id: isize| BUTTON_HINTS.iter().find(|(h, _)| *h == id).unwrap().1;
        assert!(hint(ID_BTN_CLOSE).contains("VẪN CHẠY"));
        assert!(hint(ID_BTN_EXIT).contains("Tắt hẳn"));
        // Không nút nào chồng lên nút khác và không tràn bề rộng thiết kế.
        let mut rects: Vec<_> = row.iter().map(|(id, _)| find(*id).rect).collect();
        rects.sort();
        for pair in rects.windows(2) {
            assert!(
                pair[0].0 + pair[0].2 <= pair[1].0,
                "nút chồng nhau: {pair:?}"
            );
        }
        let last = rects.last().unwrap();
        assert!(last.0 + last.2 <= CLIENT_DESIGN_WIDTH);
        assert!(!HELP_TEXT.contains("Cài & bật"));
    }

    #[test]
    fn crlf_conversion_round_trips() {
        let lf = "vn = Việt Nam\ncty = Công ty\n";
        assert_eq!(to_crlf(lf).replace("\r\n", "\n"), lf);
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
