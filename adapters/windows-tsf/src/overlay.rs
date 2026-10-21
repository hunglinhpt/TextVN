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
//!   + `WM_MOUSEACTIVATE → MA_NOACTIVATE`; `HTTRANSPARENT` để click xuyên qua.
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
}

#[cfg(windows)]
thread_local! {
    static SHOWN: RefCell<Option<Shown>> = const { RefCell::new(None) };
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
    let height = PAD * 2 + text_h + lines * LINE_H + HINT_H;
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
fn should_dismiss() -> bool {
    let down = left_button_down();
    let (expired, owner, clicked) = SHOWN.with(|cell| {
        let Ok(mut borrowed) = cell.try_borrow_mut() else {
            return (false, 0, false);
        };
        let Some(shown) = borrowed.as_mut() else {
            return (false, 0, false);
        };
        // Chỉ **lần nhấn mới** đóng panel (panel không nhận chuột —
        // `HTTRANSPARENT` — nên click nào cũng là "rời đi").
        let clicked = down && !shown.lbutton_down;
        shown.lbutton_down = down;
        (Instant::now() >= shown.deadline, shown.owner, clicked)
    });
    if expired || clicked {
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
        // Click xuyên qua panel (panel chỉ để đọc; chọn bằng bàn phím).
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        // Không bao giờ activate — focus ở lại app đang gõ.
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_TIMER => {
            if wparam.0 == TIMER_WATCHDOG && should_dismiss() {
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
        let Ok(borrowed) = cell.try_borrow() else {
            return;
        };
        let Some(shown) = borrowed.as_ref() else {
            return;
        };
        let view = &shown.view;
        let left = rect.left + PAD;
        let right = rect.right - PAD;

        // --- câu vừa gõ + gạch chân đỏ đúng span (char Unicode) ---
        let (skip, visible) = view.visible_sentence(MAX_OVERLAY_SENTENCE_CHARS);
        let mut xs = Vec::with_capacity(visible.len() + 1);
        let mut x = left;
        let mut drawn = 0usize;
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
            let _ = TextOutW(hdc, x, rect.top + PAD, units);
            x += width;
            drawn += 1;
        }
        xs.push(x);
        let _ = drawn;

        let pen = CreatePen(PS_SOLID, 2, COLOR_ERROR);
        let old_pen = SelectObject(hdc, pen.into());
        for issue in &view.issues {
            let a = issue.start.saturating_sub(skip).min(xs.len() - 1);
            let b = issue.end.saturating_sub(skip).min(xs.len() - 1);
            if a < b {
                let y = rect.top + PAD + line_h - 3;
                // GDI: MoveToEx + LineTo (không dùng font underline để chỉ đúng span).
                let _ = MoveToEx(hdc, xs[a], y, None);
                let _ = LineTo(hdc, xs[b], y);
            }
        }
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen.into());

        // --- danh sách lỗi + gợi ý (chọn bằng ↑/↓, Alt+1..3) ---
        let mut y = rect.top + PAD + line_h + 2;
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
