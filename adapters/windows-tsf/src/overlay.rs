// SPDX-License-Identifier: GPL-3.0-or-later
//! Overlay Trợ lý ngữ pháp cho Windows TSF (AS-030 · D4 / D30 / D38).
//!
//! **D30:** Windows dùng **overlay tự vẽ** (không `ITfCandidateListUI`), hiện
//! cạnh caret của app đang gõ. Panel vẽ đúng spec `textvn-cli assist show`:
//! câu vừa gõ + lỗi **gạch chân đỏ ngay trong overlay** (D4: không sửa format
//! của app) + thông điệp tiếng Việt + 1–3 gợi ý.
//!
//! Bất biến:
//! - **Không chiếm focus**: `WS_EX_NOACTIVATE` + `ShowWindow(SW_SHOWNOACTIVATE)`
//!   + `WM_MOUSEACTIVATE → MA_NOACTIVATE`; vùng tương tác nhận click
//!     (vòng 27), vùng còn lại `HTTRANSPARENT` (click xuyên qua = đóng panel).
//! - **Không vào accessibility tree** (Fassist-23): `WS_EX_TOOLWINDOW` (không
//!   Alt+Tab) + `WM_GETOBJECT → 0` (không cấp MSAA/UIA object) + không bao giờ
//!   gọi `NotifyWinEvent`/UIA provider ⇒ app khác không đọc được câu qua a11y.
//! - **Tự tắt**: click chuột (ngoài hoặc trong panel), 8 s không tương tác
//!   ([`crate::assist::OVERLAY_IDLE_MS`]), app đổi foreground, gõ tiếp
//!   (`key_event` gọi `hide`), app đổi focus (`tip::OnSetFocus`), `Esc`.
//! - **Không gọi lại COM/TSF** từ window proc/timer (P4-10 · reentrancy): panel
//!   chỉ giữ model RAM ([`OverlayView`]) + Win32/GDI. Mọi lời gọi TSF nằm ở
//!   `edit_session`, trước/sau khi hiện panel.
//! - **Fail-open**: API Win32/GDI lỗi → im lặng bỏ qua, không bao giờ làm app sập.

use crate::assist::OverlayView;
#[cfg(windows)]
use crate::assist::{MAX_OVERLAY_SENTENCE_CHARS, OVERLAY_HINT, OVERLAY_IDLE_MS};
#[cfg(windows)]
use std::cell::RefCell;

#[cfg(windows)]
use std::cell::Cell;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::*;
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

/// Vị trí neo của panel (toạ độ màn hình) — kiểu thuần để phần còn lại của
/// crate không phụ thuộc `RECT` của Win32.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Anchor {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Anchor {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Rect hợp lệ (caret rect từ `ITfContextView::GetTextExt`).
    pub fn is_valid(self) -> bool {
        self.right > self.left && self.bottom > self.top
    }
}

#[cfg(windows)]
const OVERLAY_CLASS_NAME: PCWSTR = windows::core::w!("TextVNAssistOverlay");
#[cfg(windows)]
const TIMER_WATCHDOG: usize = 1001;
/// Nhịp kiểm tra tự tắt (click / focus / hết 8 s) — không cần chính xác cao.
#[cfg(windows)]
const WATCHDOG_INTERVAL_MS: u32 = 200;

#[cfg(windows)]
const PAD: i32 = 10;
#[cfg(windows)]
const LINE_H: i32 = 20;
#[cfg(windows)]
const HINT_H: i32 = 18;
/// Dòng tổng kết "N lỗi trong câu vừa gõ" (vòng 26 — kiểu Grammarly).
#[cfg(windows)]
const HEADER_H: i32 = 22;
#[cfg(windows)]
const MIN_W: i32 = 240;
#[cfg(windows)]
const MAX_W: i32 = 560;

#[cfg(windows)]
const COLOR_BG: COLORREF = COLORREF(0x00F8_F9FA);
#[cfg(windows)]
const COLOR_BORDER: COLORREF = COLORREF(0x00DC_D7D2);
#[cfg(windows)]
const COLOR_TEXT: COLORREF = COLORREF(0x0020_2020);
#[cfg(windows)]
const COLOR_MUTED: COLORREF = COLORREF(0x0060_6060);
#[cfg(windows)]
const COLOR_ERROR: COLORREF = COLORREF(0x0025_25D9);
#[cfg(windows)]
const COLOR_SELECTED_BG: COLORREF = COLORREF(0x00FE_F0E8);

/// Hành động chuột trên panel (vòng 27 — "như Grammarly": bấm vào được).
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMouseAction {
    /// Chọn lỗi thứ `i` (click lên từ gạch chân / dòng lỗi chưa chọn).
    Select(usize),
    /// Áp gợi ý của lỗi thứ `i` (click lần nữa trên lỗi đang chọn).
    Apply(usize),
    /// Nút "Sửa tất cả" — tự sửa cả câu (như Mode 2 nhưng user chủ động).
    FixAll,
}

/// Kết quả hit-test thuần — test được không cần Win32.
///
/// - Click vào nút "Sửa tất cả" → [`OverlayMouseAction::FixAll`].
/// - Click vào dòng lỗi `i`: chưa chọn → Select(i); đang chọn → Apply(i).
/// - Click lên ký tự trong span của lỗi `i`: chưa chọn → Select(i);
///   đang chọn → Apply(i).
/// - Ngoài các vùng trên → `None` (click xuyên qua panel).
///
/// Hình học hit-test ghi trong paint (vòng 27) — gom nhóm để hàm hit-test
/// không quá 7 tham số.
#[cfg(windows)]
struct HitGeometry<'a> {
    view: &'a OverlayView,
    skip: usize,
    char_xs: &'a [i32],
    content_top: i32,
    line_h: i32,
    row_rects: &'a [RECT],
    button_rect: Option<RECT>,
    /// Đã có click-chọn trong lần hiện này chưa (vòng 28): `false` → click
    /// lên lỗi được chọn sẵn phải Select chứ không phải Apply.
    mouse_selected: bool,
}

#[cfg(windows)]
fn mouse_hit_test(g: &HitGeometry, x: i32, y: i32) -> Option<OverlayMouseAction> {
    let view = g.view;
    let skip = g.skip;
    let char_xs = g.char_xs;
    let content_top = g.content_top;
    let line_h = g.line_h;
    let row_rects = g.row_rects;
    let button_rect = g.button_rect;
    // 1. Nút "Sửa tất cả".
    if let Some(r) = button_rect {
        if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
            return Some(OverlayMouseAction::FixAll);
        }
    }
    // 2. Dòng lỗi: click chọn; click lần nữa trên dòng đang chọn → áp.
    for (i, r) in row_rects.iter().enumerate() {
        if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
            return Some(if view.selected == i && g.mouse_selected {
                OverlayMouseAction::Apply(i)
            } else {
                OverlayMouseAction::Select(i)
            });
        }
    }
    // 3. Ký tự trong span lỗi (dòng câu): xác định char index bằng `char_xs`.
    if y >= content_top && y < content_top + line_h && char_xs.len() >= 2 {
        let mut char_idx = None;
        for (ci, w) in char_xs.windows(2).enumerate() {
            if x >= w[0] && x < w[1] {
                char_idx = Some(skip + ci);
                break;
            }
        }
        if let Some(pos) = char_idx {
            for (i, issue) in view.issues.iter().enumerate() {
                if pos >= issue.start && pos < issue.end {
                    return Some(if view.selected == i && g.mouse_selected {
                        OverlayMouseAction::Apply(i)
                    } else {
                        OverlayMouseAction::Select(i)
                    });
                }
            }
        }
    }
    None
}

/// Dispatch hành động chuột về edit_session (gọi từ WndProc — cùng thread UI).
#[cfg(windows)]
fn dispatch_mouse_action(action: OverlayMouseAction) {
    ACTION_SINK.with(|cell| {
        if let Some(sink) = cell.borrow().as_ref() {
            sink(action);
        }
    });
}

/// Model + chủ sở hữu + hạn tự tắt của panel đang hiện (thread UI của app).
#[cfg(windows)]
struct Shown {
    view: OverlayView,
    /// HWND của app lúc hiện panel — foreground đổi ⇒ panel của app cũ phải tắt.
    owner: isize,
    /// `Instant` hết hạn 8 s không tương tác.
    deadline: Instant,
    /// Trạng thái chuột trái ở lần kiểm tra trước: chỉ **lần nhấn mới** mới đóng
    /// (giữ chuột từ trước khi panel hiện không làm panel tắt ngay).
    lbutton_down: bool,
    /// WM_LBUTTONDOWN vừa được panel xử lý (vùng tương tác) → watchdog
    /// không được coi là click-rời-đi (vòng 28).
    click_handled: bool,
    /// Đã có click-chọn một lỗi lần nào trong lần hiện này (vòng 28):
    /// click ĐẦU TIÊN trên lỗi được chọn sẵn (selected=0) phải là Select
    /// chứ không phải Apply (tránh 1 click vô tình sửa document).
    mouse_selected: bool,
    /// Vòng 26: hình học hit-test ghi trong paint — nút "Sửa tất cả",
    /// các dòng lỗi, toạ độ từng ký tự dòng câu (để bắt click lên từ gạch chân).
    button_rect: Option<RECT>,
    row_rects: Vec<RECT>,
    char_xs: Vec<i32>,
    skip: usize,
    content_top: i32,
    line_h: i32,
}

/// Callback dispatch hành động chuột về edit_session (giữ `Weak<TsfShared>`).
#[cfg(windows)]
type ActionSink = Box<dyn Fn(OverlayMouseAction)>;

#[cfg(windows)]
thread_local! {
    static SHOWN: RefCell<Option<Shown>> = const { RefCell::new(None) };
    /// Đăng ký một lần khi TsfShared dựng — panel không giữ Rc vòng.
    static ACTION_SINK: RefCell<Option<ActionSink>> = const { RefCell::new(None) };
}

/// Panel overlay của MỘT thread UI (TIP in-process: mỗi app một instance).
#[cfg(windows)]
pub struct CandidateOverlay {
    hwnd: Cell<HWND>,
}

#[cfg(windows)]
impl CandidateOverlay {
    pub fn new() -> Self {
        Self {
            hwnd: Cell::new(HWND::default()),
        }
    }

    /// Đăng ký nơi nhận hành động chuột (gọi MỘT lần khi dựng `TsfShared`;
    /// closure giữ `Weak<TsfShared>` — panel không giữ Rc vòng).
    pub fn set_action_sink(&self, sink: ActionSink) {
        ACTION_SINK.with(|cell| {
            *cell.borrow_mut() = Some(sink);
        });
    }

    /// Đăng ký window class một lần trong process (fail-open: `false` = bỏ).
    fn ensure_class_registered() -> bool {
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if REGISTERED.load(Ordering::Acquire) {
            return true;
        }
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
                lpfnWndProc: Some(overlay_wnd_proc),
                hInstance: hinstance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: OVERLAY_CLASS_NAME,
                ..Default::default()
            };
            if RegisterClassExW(&wc) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS {
                REGISTERED.store(true, Ordering::Release);
                true
            } else {
                false
            }
        }
    }

    /// HWND của panel (tạo lần đầu, fail-open: HWND rỗng nếu lỗi).
    fn get_or_create_hwnd(&self) -> HWND {
        let current = self.hwnd.get();
        if !current.0.is_null() && unsafe { IsWindow(Some(current)).as_bool() } {
            return current;
        }
        if !Self::ensure_class_registered() {
            return HWND::default();
        }
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            // Tiêu đề KHÔNG chứa nội dung người dùng (Fassist-23): nó là tên
            // window, không phải câu đang sửa.
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                OVERLAY_CLASS_NAME,
                windows::core::w!("TextVN"),
                WS_POPUP | WS_BORDER,
                0,
                0,
                MIN_W,
                80,
                None,
                None,
                Some(hinstance.into()),
                None,
            );
            match hwnd {
                Ok(h) => {
                    self.hwnd.set(h);
                    h
                }
                Err(_) => HWND::default(),
            }
        }
    }

    /// Hiện (hoặc cập nhật) panel cạnh caret `anchor`, thuộc cửa sổ app `owner`
    /// (`0` = không kiểm tra foreground).
    ///
    /// Gọi **trong `DoEditSession`** — chỉ Win32/GDI, không gọi lại TSF.
    pub fn show(&self, view: &OverlayView, anchor: Anchor, owner: isize) {
        let hwnd = self.get_or_create_hwnd();
        if hwnd.0.is_null() {
            return;
        }
        let (width, height) = measure(hwnd, view);
        let (x, y) = place(hwnd, anchor, width, height);

        SHOWN.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                *slot = Some(Shown {
                    view: view.clone(),
                    owner,
                    deadline: Instant::now() + Duration::from_millis(OVERLAY_IDLE_MS),
                    lbutton_down: left_button_down(),
                    button_rect: None,
                    row_rects: Vec::new(),
                    char_xs: Vec::new(),
                    skip: 0,
                    content_top: 0,
                    line_h: LINE_H,
                    click_handled: false,
                    mouse_selected: false,
                });
            }
        });

        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(Some(hwnd), None, true);
            let _ = SetTimer(Some(hwnd), TIMER_WATCHDOG, WATCHDOG_INTERVAL_MS, None);
        }
    }

    /// Đổi lỗi đang chọn (↑/↓, Alt+1..3) — vẽ lại, coi như có tương tác (dời
    /// hạn 8 s).
    pub fn set_view(&self, view: &OverlayView) {
        let hwnd = self.hwnd.get();
        if hwnd.0.is_null() || !unsafe { IsWindowVisible(hwnd).as_bool() } {
            return;
        }
        SHOWN.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                if let Some(shown) = slot.as_mut() {
                    shown.view = view.clone();
                    shown.deadline = Instant::now() + Duration::from_millis(OVERLAY_IDLE_MS);
                }
            }
        });
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
    }

    /// Ghi nhận tương tác (dời hạn 8 s) — không vẽ lại.
    pub fn touch(&self) {
        SHOWN.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                if let Some(shown) = slot.as_mut() {
                    shown.deadline = Instant::now() + Duration::from_millis(OVERLAY_IDLE_MS);
                }
            }
        });
    }

    /// Ẩn panel + bỏ model (không giữ text người dùng trong RAM lâu hơn cần).
    pub fn hide(&self) {
        SHOWN.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                *slot = None;
            }
        });
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER_WATCHDOG);
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            unsafe { IsWindowVisible(hwnd).as_bool() }
        } else {
            false
        }
    }
}

#[cfg(windows)]
impl Default for CandidateOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl Drop for CandidateOverlay {
    fn drop(&mut self) {
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            let _ = unsafe { DestroyWindow(hwnd) };
        }
        SHOWN.with(|cell| {
            if let Ok(mut slot) = cell.try_borrow_mut() {
                *slot = None;
            }
        });
    }
}

/// Font UI của panel; caller phải `DeleteObject`.
#[cfg(windows)]
unsafe fn create_font(height: i32, weight: FONT_WEIGHT) -> HFONT {
    CreateFontW(
        -height,
        0,
        0,
        0,
        weight.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        DEFAULT_PITCH.0 as u32,
        windows::core::w!("Segoe UI"),
    )
}

/// Kích thước panel theo nội dung (đo trên DC màn hình, không cần HWND).
#[cfg(windows)]
fn measure(hwnd: HWND, view: &OverlayView) -> (i32, i32) {
    let dc = unsafe { GetDC(None) };
    let mut max_w = MIN_W - PAD * 2;
    let mut text_h = LINE_H;
    if !dc.is_invalid() {
        unsafe {
            let font = create_font(15, FW_NORMAL);
            let old = SelectObject(dc, font.into());
            let (_, visible) = view.visible_sentence(MAX_OVERLAY_SENTENCE_CHARS);
            let mut sentence_w = 0;
            for ch in visible {
                let mut buf = [0u16; 2];
                let units = ch.encode_utf16(&mut buf);
                let mut size = SIZE::default();
                if GetTextExtentPoint32W(dc, units, &mut size).as_bool() {
                    sentence_w += size.cx;
                    text_h = LINE_H.max(size.cy + 4);
                }
            }
            max_w = max_w.max(sentence_w);
            for i in 0..view.issues.len() {
                let mut text: Vec<u16> = view.issue_line(i).encode_utf16().collect();
                let mut rc = RECT::default();
                let h = DrawTextW(
                    dc,
                    &mut text,
                    &mut rc,
                    DT_CALCRECT | DT_SINGLELINE | DT_LEFT,
                );
                max_w = max_w.max(rc.right - rc.left);
                text_h = text_h.max(h + 4);
            }
            // Dòng nhắc phím vẽ bằng font nhỏ: đo đúng cỡ đó để panel đủ rộng
            // (không cắt mất phím tắt quan trọng).
            let small = create_font(12, FW_NORMAL);
            SelectObject(dc, small.into());
            let mut hint: Vec<u16> = OVERLAY_HINT.encode_utf16().collect();
            let mut hint_rc = RECT::default();
            let _ = DrawTextW(
                dc,
                &mut hint,
                &mut hint_rc,
                DT_CALCRECT | DT_SINGLELINE | DT_LEFT,
            );
            max_w = max_w.max(hint_rc.right - hint_rc.left);
            SelectObject(dc, old);
            let _ = DeleteObject(small.into());
            let _ = DeleteObject(font.into());
        }
        unsafe {
            ReleaseDC(None, dc);
        }
    }
    let _ = hwnd;
    let lines = view.issues.len() as i32 + i32::from(view.hidden_count() > 0);
    let height = PAD * 2 + HEADER_H + text_h + lines * LINE_H + HINT_H;
    let width = (max_w + PAD * 2).clamp(MIN_W, MAX_W);
    (width, height)
}

/// Đặt panel dưới caret; hết chỗ thì lên trên; luôn nằm trong work area.
#[cfg(windows)]
fn place(hwnd: HWND, anchor: Anchor, width: i32, height: i32) -> (i32, i32) {
    let anchor_rect = RECT {
        left: anchor.left,
        top: anchor.top,
        right: anchor.right,
        bottom: anchor.bottom,
    };
    let mut x = anchor.left;
    let mut y = anchor.bottom + 4;
    unsafe {
        let mon = MonitorFromRect(&anchor_rect, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(mon, &mut info).as_bool() {
            let work = info.rcWork;
            if x + width > work.right {
                x = work.right - width;
            }
            if y + height > work.bottom {
                y = anchor.top - height - 4;
            }
            x = x.max(work.left);
            y = y.max(work.top);
        }
    }
    let _ = hwnd;
    (x, y)
}

/// Có nên tự tắt panel ngay bây giờ (hết 8 s · app mất foreground · click chuột).
#[cfg(windows)]
fn should_dismiss(_hwnd: HWND) -> bool {
    let down = left_button_down();
    let (expired, owner, clicked_outside) = SHOWN.with(|cell| {
        let Ok(mut borrowed) = cell.try_borrow_mut() else {
            return (false, 0, false);
        };
        let Some(shown) = borrowed.as_mut() else {
            return (false, 0, false);
        };
        // Vòng 26+28: click NGOÀI panel (HTTRANSPARENT — đi vào app) hoặc
        // click vùng KHÔNG tương tác trong panel → đóng. Click vùng tương tác
        // do WndProc xử lý (đặt `click_handled`) → không đóng.
        let clicked = down && !shown.lbutton_down;
        let handled = clicked && shown.click_handled;
        shown.click_handled = false;
        shown.lbutton_down = down;
        (
            Instant::now() >= shown.deadline,
            shown.owner,
            clicked && !handled,
        )
    });
    if expired || clicked_outside {
        return true;
    }
    unsafe { owner != 0 && GetForegroundWindow().0 as isize != owner }
}

/// Chuột trái đang được nhấn (trạng thái async của thread hiện tại).
#[cfg(windows)]
fn left_button_down() -> bool {
    // SAFETY: GetAsyncKeyState chỉ đọc trạng thái phím của phiên người dùng.
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(windows)]
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Fassist-23: KHÔNG cấp MSAA/UIA object ⇒ nội dung overlay không vào
        // accessibility tree (app khác không đọc được câu người dùng).
        WM_GETOBJECT => LRESULT(0),
        // Vòng 26: panel NHẬN chuột trên vùng tương tác (nút "Sửa tất cả",
        // dòng lỗi, từ gạch chân) — vùng còn lại click xuyên qua như cũ.
        WM_NCHITTEST => {
            let pt = POINT {
                x: (lparam.0 & 0xFFFF) as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
            };
            let mut client = pt;
            let _ = ScreenToClient(hwnd, &mut client);
            let interactive = SHOWN.with(|cell| {
                cell.borrow().as_ref().is_some_and(|shown| {
                    let g = HitGeometry {
                        view: &shown.view,
                        skip: shown.skip,
                        char_xs: &shown.char_xs,
                        content_top: shown.content_top,
                        line_h: shown.line_h,
                        row_rects: &shown.row_rects,
                        button_rect: shown.button_rect,
                        mouse_selected: shown.mouse_selected,
                    };
                    mouse_hit_test(&g, client.x, client.y).is_some()
                })
            });
            if interactive {
                LRESULT(HTCLIENT as isize)
            } else {
                LRESULT(HTTRANSPARENT as isize)
            }
        }
        WM_LBUTTONDOWN => {
            let pt = POINT {
                x: (lparam.0 & 0xFFFF) as i16 as i32,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
            };
            let action = SHOWN.with(|cell| {
                cell.borrow().as_ref().and_then(|shown| {
                    let g = HitGeometry {
                        view: &shown.view,
                        skip: shown.skip,
                        char_xs: &shown.char_xs,
                        content_top: shown.content_top,
                        line_h: shown.line_h,
                        row_rects: &shown.row_rects,
                        button_rect: shown.button_rect,
                        mouse_selected: shown.mouse_selected,
                    };
                    mouse_hit_test(&g, pt.x, pt.y)
                })
            });
            if let Some(action) = action {
                SHOWN.with(|cell| {
                    if let Ok(mut shown) = cell.try_borrow_mut() {
                        if let Some(shown) = shown.as_mut() {
                            shown.click_handled = true;
                            if matches!(action, OverlayMouseAction::Select(_)) {
                                shown.mouse_selected = true;
                            }
                        }
                    }
                });
                dispatch_mouse_action(action);
                let _ = InvalidateRect(Some(hwnd), None, true);
            }
            LRESULT(0)
        }
        // Không bao giờ activate — focus ở lại app đang gõ.
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_TIMER => {
            if wparam.0 == TIMER_WATCHDOG && should_dismiss(hwnd) {
                let _ = KillTimer(Some(hwnd), TIMER_WATCHDOG);
                let _ = ShowWindow(hwnd, SW_HIDE);
                SHOWN.with(|cell| {
                    if let Ok(mut slot) = cell.try_borrow_mut() {
                        *slot = None;
                    }
                });
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_invalid() {
                paint_overlay(hwnd, hdc);
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Dòng tổng kết kiểu Grammarly (vòng 26): đếm lỗi hiện + ẩn.
#[cfg(windows)]
fn summary_text(view: &OverlayView) -> String {
    let total = view.total_issues;
    let hidden = view.hidden_count();
    if hidden > 0 {
        format!("{total} lỗi trong câu vừa gõ (+{hidden} đoạn khác)")
    } else if total > 1 {
        format!("{total} lỗi trong câu vừa gõ")
    } else {
        "1 lỗi trong câu vừa gõ".to_string()
    }
}

#[cfg(windows)]
unsafe fn paint_overlay(hwnd: HWND, hdc: HDC) {
    let mut rect = RECT::default();
    if GetClientRect(hwnd, &mut rect).is_err() {
        return;
    }
    let bg = CreateSolidBrush(COLOR_BG);
    let _ = FillRect(hdc, &rect, bg);
    let _ = DeleteObject(bg.into());
    let border = CreateSolidBrush(COLOR_BORDER);
    let _ = FrameRect(hdc, &rect, border);
    let _ = DeleteObject(border.into());

    let font = create_font(15, FW_NORMAL);
    let small = create_font(12, FW_NORMAL);
    let old_font = SelectObject(hdc, font.into());
    let _ = SetBkMode(hdc, TRANSPARENT);

    SHOWN.with(|cell| {
        let Ok(mut borrowed) = cell.try_borrow_mut() else {
            return;
        };
        let Some(shown) = borrowed.as_mut() else {
            return;
        };
        let view = &shown.view;
        let left = rect.left + PAD;
        let right = rect.right - PAD;
        shown.button_rect = None;
        shown.row_rects.clear();
        shown.char_xs.clear();

        // --- tổng kết kiểu Grammarly (vòng 26): "N lỗi trong câu vừa gõ" ---
        {
            let header_font = create_font(14, FW_SEMIBOLD);
            SelectObject(hdc, header_font.into());
            let mut header: Vec<u16> = summary_text(view).encode_utf16().collect();
            let mut hr = RECT {
                left,
                top: rect.top + PAD - 2,
                right,
                bottom: rect.top + PAD - 2 + HEADER_H,
            };
            let _ = SetTextColor(hdc, COLOR_ERROR);
            let _ = DrawTextW(
                hdc,
                &mut header,
                &mut hr,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            SelectObject(hdc, old_font);
            let _ = DeleteObject(header_font.into());
            // Nút "Sửa tất cả" (vòng 26): hiện khi có ≥1 lỗi tự sửa được —
            // click → FixAll (áp cả câu, có Undo 5 giây).
            if view.issues.iter().any(|i| i.auto_fixable) {
                let btn_font = create_font(13, FW_SEMIBOLD);
                SelectObject(hdc, btn_font.into());
                let mut units: Vec<u16> = "Sửa tất cả".encode_utf16().collect();
                let mut size = SIZE::default();
                let tw = if GetTextExtentPoint32W(hdc, &units, &mut size).as_bool() {
                    size.cx
                } else {
                    0
                };
                let bw = tw + 16;
                let bh = HEADER_H - 2;
                let br = RECT {
                    left: right - bw,
                    top: rect.top + PAD - 3,
                    right,
                    bottom: rect.top + PAD - 3 + bh,
                };
                let bg = CreateSolidBrush(COLOR_SELECTED_BG);
                let _ = FillRect(hdc, &br, bg);
                let _ = DeleteObject(bg.into());
                let border = CreateSolidBrush(COLOR_BORDER);
                let _ = FrameRect(hdc, &br, border);
                let _ = DeleteObject(border.into());
                let _ = SetTextColor(hdc, COLOR_TEXT);
                let mut tr = br;
                let _ = DrawTextW(
                    hdc,
                    &mut units,
                    &mut tr,
                    DT_CENTER | DT_SINGLELINE | DT_VCENTER,
                );
                SelectObject(hdc, old_font);
                let _ = DeleteObject(btn_font.into());
                shown.button_rect = Some(br);
            }
        }
        let content_top = rect.top + PAD + HEADER_H;

        // --- câu vừa gõ + gạch chân đỏ đúng span (char Unicode) ---
        let (skip, visible) = view.visible_sentence(MAX_OVERLAY_SENTENCE_CHARS);
        let mut xs = Vec::with_capacity(visible.len() + 1);
        let mut x = left;
        let mut line_h = LINE_H;
        for ch in visible {
            let mut buf = [0u16; 2];
            let units = ch.encode_utf16(&mut buf);
            let mut size = SIZE::default();
            let width = if GetTextExtentPoint32W(hdc, units, &mut size).as_bool() {
                line_h = line_h.max(size.cy + 4);
                size.cx
            } else {
                0
            };
            if x + width > right {
                break; // panel hẹp: cắt phần đầu dòng, không tràn
            }
            xs.push(x);
            let _ = SetTextColor(hdc, COLOR_TEXT);
            let _ = TextOutW(hdc, x, content_top, units);
            x += width;
        }
        xs.push(x);
        shown.char_xs = xs.clone();
        shown.skip = skip;
        shown.content_top = content_top;
        shown.line_h = line_h;

        let pen = CreatePen(PS_SOLID, 2, COLOR_ERROR);
        let old_pen = SelectObject(hdc, pen.into());
        for issue in &view.issues {
            let a = issue.start.saturating_sub(skip).min(xs.len() - 1);
            let b = issue.end.saturating_sub(skip).min(xs.len() - 1);
            if a < b {
                let y = content_top + line_h - 3;
                // GDI: MoveToEx + LineTo (không dùng font underline để chỉ đúng span).
                let _ = MoveToEx(hdc, xs[a], y, None);
                let _ = LineTo(hdc, xs[b], y);
            }
        }
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen.into());

        // --- danh sách lỗi + gợi ý (chọn bằng ↑/↓, Alt+1..3) ---
        let mut y = content_top + line_h + 2;
        shown.row_rects.clear();
        for (idx, _) in view.issues.iter().enumerate() {
            if idx == view.selected {
                let sel = CreateSolidBrush(COLOR_SELECTED_BG);
                let row = RECT {
                    left: rect.left + 4,
                    top: y - 2,
                    right: rect.right - 4,
                    bottom: y + LINE_H - 2,
                };
                let _ = FillRect(hdc, &row, sel);
                let _ = DeleteObject(sel.into());
            }
            let mut text: Vec<u16> = view.issue_line(idx).encode_utf16().collect();
            let mut r = RECT {
                left,
                top: y,
                right,
                bottom: y + LINE_H,
            };
            let _ = SetTextColor(
                hdc,
                if idx == view.selected {
                    COLOR_TEXT
                } else {
                    COLOR_MUTED
                },
            );
            let _ = DrawTextW(
                hdc,
                &mut text,
                &mut r,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            shown.row_rects.push(r);
            y += LINE_H;
        }
        if view.hidden_count() > 0 {
            let mut text: Vec<u16> = format!("  (+{} lỗi khác)", view.hidden_count())
                .encode_utf16()
                .collect();
            let mut r = RECT {
                left,
                top: y,
                right,
                bottom: y + LINE_H,
            };
            let _ = SetTextColor(hdc, COLOR_MUTED);
            let _ = DrawTextW(
                hdc,
                &mut text,
                &mut r,
                DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        }

        // --- nhắc phím ---
        SelectObject(hdc, small.into());
        let mut hint: Vec<u16> = OVERLAY_HINT.encode_utf16().collect();
        let mut r = RECT {
            left,
            top: rect.bottom - PAD - HINT_H,
            right,
            bottom: rect.bottom - PAD,
        };
        let _ = SetTextColor(hdc, COLOR_MUTED);
        let _ = DrawTextW(
            hdc,
            &mut hint,
            &mut r,
            DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    });

    SelectObject(hdc, old_font);
    let _ = DeleteObject(font.into());
    let _ = DeleteObject(small.into());
}

#[cfg(not(windows))]
pub struct CandidateOverlay;

#[cfg(not(windows))]
impl CandidateOverlay {
    pub fn new() -> Self {
        Self
    }
    pub fn show(&self, _view: &OverlayView, _anchor: Anchor, _owner: isize) {}
    pub fn set_view(&self, _view: &OverlayView) {}
    pub fn touch(&self) {}
    pub fn hide(&self) {}
    pub fn is_visible(&self) -> bool {
        false
    }
}

#[cfg(not(windows))]
impl Default for CandidateOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, windows))]
mod mouse_tests {
    use super::*;
    use crate::assist::IssueView;

    fn view() -> OverlayView {
        let chars: Vec<char> = "tôi tôi đi".chars().collect();
        let issues = vec![IssueView {
            start: 4,
            end: 7,
            message: "Lặp từ".into(),
            suggestion: Some("tôi".into()),
            auto_fixable: true,
            kind: textvn_ffi::IME_ASSIST_KIND_DIACRITIC_TYPO,
        }];
        OverlayView::from_parts(chars, issues, 1).expect("view")
    }

    /// Vòng 27 — hit-test thuần: nút / dòng lỗi / từ gạch chân / ngoài panel.
    #[test]
    fn hit_test_vung_tuong_tac() {
        let v = view();
        let rows = vec![RECT {
            left: 10,
            top: 40,
            right: 200,
            bottom: 60,
        }];
        let button = Some(RECT {
            left: 150,
            top: 6,
            right: 230,
            bottom: 28,
        });
        // Click nút "Sửa tất cả" → FixAll.
        macro_rules! geo {
            ($v:expr, $char_xs:expr, $rows:expr, $button:expr) => {
                HitGeometry {
                    view: $v,
                    skip: 0,
                    char_xs: $char_xs,
                    content_top: 30,
                    line_h: 20,
                    row_rects: $rows,
                    button_rect: $button,
                    mouse_selected: false,
                }
            };
        }
        macro_rules! geo_sel {
            ($v:expr, $char_xs:expr, $rows:expr, $button:expr) => {
                HitGeometry {
                    view: $v,
                    skip: 0,
                    char_xs: $char_xs,
                    content_top: 30,
                    line_h: 20,
                    row_rects: $rows,
                    button_rect: $button,
                    mouse_selected: true,
                }
            };
        }
        assert_eq!(
            mouse_hit_test(&geo!(&v, &[0, 10], &rows, button), 200, 20),
            Some(OverlayMouseAction::FixAll)
        );
        // Vòng 28: click ĐẦU TIÊN trên lỗi được chọn sẵn (selected=0) →
        // Select (không phải Apply — 1 click vô tình không sửa document).
        assert_eq!(
            mouse_hit_test(&geo!(&v, &[0, 10], &rows, button), 100, 50),
            Some(OverlayMouseAction::Select(0))
        );
        // Sau khi đã click-chọn (mouse_selected=true) → click lại = Apply.
        assert_eq!(
            mouse_hit_test(&geo_sel!(&v, &[0, 10], &rows, button), 100, 50),
            Some(OverlayMouseAction::Apply(0))
        );
        // Click dòng chưa chọn → Select.
        let mut v_other = v.clone();
        v_other.selected = 0;
        let rows2 = vec![
            RECT {
                left: 10,
                top: 40,
                right: 200,
                bottom: 60,
            },
            RECT {
                left: 10,
                top: 60,
                right: 200,
                bottom: 80,
            },
        ];
        assert_eq!(
            mouse_hit_test(&geo!(&v_other, &[0, 10], &rows2, button), 100, 70),
            Some(OverlayMouseAction::Select(1))
        );
        // Click lên ký tự trong span (char_xs phủ span 4..7) → lỗi đang chọn
        // → Apply(0).
        let char_xs = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110];
        assert_eq!(
            mouse_hit_test(&geo_sel!(&v, &char_xs, &rows, button), 45, 35),
            Some(OverlayMouseAction::Apply(0))
        );
        // Click ngoài panel → None (xuyên qua như cũ).
        assert_eq!(
            mouse_hit_test(&geo!(&v, &char_xs, &rows, button), 5, 300),
            None
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assist::IssueView;

    fn sample() -> OverlayView {
        OverlayView::from_parts(
            "Hôm nay tôi tôi đi học .".chars().collect(),
            vec![IssueView {
                start: 12,
                end: 15,
                message: "Lặp từ: \"tôi\"".into(),
                suggestion: Some("tôi".into()),
                auto_fixable: true,
                kind: textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD,
            }],
            1,
        )
        .expect("view")
    }

    #[test]
    fn overlay_creation_and_visibility() {
        let overlay = CandidateOverlay::new();
        assert!(!overlay.is_visible());
        overlay.hide();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn overlay_shows_view_and_hides() {
        let overlay = CandidateOverlay::new();
        assert!(!overlay.is_visible());
        overlay.show(&sample(), Anchor::new(100, 200, 102, 220), 0);
        overlay.set_view(&sample());
        overlay.touch();
        overlay.hide();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn anchor_hop_le_khi_co_be_rong() {
        assert!(Anchor::new(10, 10, 12, 30).is_valid());
        assert!(!Anchor::new(10, 10, 10, 30).is_valid());
        assert!(!Anchor::default().is_valid());
    }
}
