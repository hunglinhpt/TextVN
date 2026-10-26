// SPDX-License-Identifier: GPL-3.0-or-later
//! P6-11 · R2-134 — chữ mờ gợi ý sẵn (ghost text) ngay sau con trỏ.
//!
//! Chủ repo: "chữ mờ gợi ý sẵn chưa được" (kiểu Grammarly/Copilot). Cửa sổ popup
//! layered (color key) vẽ phần chữ mà Tab / phím tắt **sẽ** nối thêm
//! (`ime_peek_completion` — chạy thử trên bản sao engine) bằng màu xám, cỡ chữ
//! theo chiều cao caret.
//!
//! Bất biến (cùng triết lý panel trợ lý `overlay.rs`):
//! - **Không chạm document**: chỉ là hình vẽ — Tab mới chèn chữ thật; mất focus,
//!   click, app tự lưu… không bao giờ commit nhầm chữ gợi ý (lý do không dùng
//!   composition có display attribute xám).
//! - **Không chiếm focus/chuột**: `WS_EX_NOACTIVATE | WS_EX_TRANSPARENT` +
//!   `WM_NCHITTEST → HTTRANSPARENT`.
//! - **Không vào accessibility tree**: `WM_GETOBJECT → 0`, `WS_EX_TOOLWINDOW`.
//! - **Tự tắt**: mỗi phím (`key_event` gọi `hide`), đổi focus/composition, app
//!   mất foreground (watchdog).
//! - Không gọi TSF từ window proc; fail-open mọi lỗi Win32/GDI.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::overlay::Anchor;

const CLASS_NAME: PCWSTR = w!("TextVNGhostText");
const TIMER_WATCHDOG: usize = 1101;
const WATCHDOG_INTERVAL_MS: u32 = 200;
/// Nền color key (trong suốt) — màu không bao giờ dùng để vẽ chữ.
const COLOR_KEY: COLORREF = COLORREF(0x00FF_00FF);
/// Xám vừa đủ đọc trên nền sáng lẫn tối.
const COLOR_GHOST: COLORREF = COLORREF(0x0090_9090);
/// Chữ gợi ý dài hơn thế này thì không vẽ (cụm dự đoán ≤ 64 glyph).
const MAX_GHOST_CHARS: usize = 64;

struct State {
    text: Vec<u16>,
    font_px: i32,
    /// HWND gốc của app lúc vẽ — mất foreground thì tắt.
    owner: isize,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Chữ mờ của MỘT thread UI (TIP in-process: mỗi app một instance).
pub struct GhostText {
    hwnd: Cell<HWND>,
}

impl Default for GhostText {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostText {
    pub fn new() -> Self {
        Self {
            hwnd: Cell::new(HWND::default()),
        }
    }

    fn ensure_class() -> bool {
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if REGISTERED.load(Ordering::Acquire) {
            return true;
        }
        // SAFETY: đăng ký class một lần trong process; lỗi "đã tồn tại" coi là xong.
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(ghost_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: CLASS_NAME,
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

    fn get_or_create(&self) -> HWND {
        let current = self.hwnd.get();
        // SAFETY: IsWindow chỉ kiểm handle.
        if !current.0.is_null() && unsafe { IsWindow(Some(current)).as_bool() } {
            return current;
        }
        if !Self::ensure_class() {
            return HWND::default();
        }
        // SAFETY: tạo popup layered trên thread UI hiện tại; tiêu đề không chứa
        // nội dung người dùng.
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let Ok(hwnd) = CreateWindowExW(
                WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE
                    | WS_EX_TOPMOST
                    | WS_EX_LAYERED
                    | WS_EX_TRANSPARENT,
                CLASS_NAME,
                w!("TextVN"),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                Some(hinstance.into()),
                None,
            ) else {
                return HWND::default();
            };
            let _ = SetLayeredWindowAttributes(hwnd, COLOR_KEY, 255, LWA_COLORKEY);
            self.hwnd.set(hwnd);
            hwnd
        }
    }

    /// Vẽ `text` mờ ngay sau caret (`caret` toạ độ màn hình), thuộc app `owner`,
    /// cỡ chữ `font_px` (chiều cao em, ước lượng theo chữ của app).
    pub fn show(&self, text: &str, caret: Anchor, owner: isize, font_px: i32) {
        let units: Vec<u16> = text.encode_utf16().collect();
        if units.is_empty() || text.chars().count() > MAX_GHOST_CHARS || !caret.is_valid() {
            self.hide();
            return;
        }
        let hwnd = self.get_or_create();
        if hwnd.0.is_null() {
            return;
        }
        let line_h = (caret.bottom - caret.top).clamp(12, 160);
        let font_px = font_px.clamp(8, line_h);
        let width = measure(&units, font_px) + 4;
        STATE.with(|cell| {
            *cell.borrow_mut() = Some(State {
                text: units,
                font_px,
                owner,
            });
        });
        // SAFETY: cửa sổ của thread hiện tại; không activate.
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                caret.left + 1,
                caret.top,
                width,
                line_h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            let _ = InvalidateRect(Some(hwnd), None, true);
            let _ = SetTimer(Some(hwnd), TIMER_WATCHDOG, WATCHDOG_INTERVAL_MS, None);
        }
    }

    /// Ẩn + bỏ chữ (không giữ gợi ý trong RAM lâu hơn cần — S2).
    pub fn hide(&self) {
        STATE.with(|cell| cell.borrow_mut().take());
        let hwnd = self.hwnd.get();
        // SAFETY: handle của thread hiện tại; IsWindow chặn handle đã huỷ.
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER_WATCHDOG);
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        let hwnd = self.hwnd.get();
        // SAFETY: chỉ đọc trạng thái hiển thị.
        !hwnd.0.is_null()
            && unsafe { IsWindow(Some(hwnd)).as_bool() && IsWindowVisible(hwnd).as_bool() }
    }
}

impl Drop for GhostText {
    fn drop(&mut self) {
        let hwnd = self.hwnd.get();
        // SAFETY: cửa sổ do thread này tạo.
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            let _ = unsafe { DestroyWindow(hwnd) };
        }
        STATE.with(|cell| cell.borrow_mut().take());
    }
}

/// Font chữ mờ: không khử răng cưa — nền color key, ClearType sẽ để viền màu key.
unsafe fn ghost_font(px: i32) -> HFONT {
    CreateFontW(
        -px,
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        NONANTIALIASED_QUALITY,
        DEFAULT_PITCH.0 as u32,
        w!("Segoe UI"),
    )
}

fn measure(units: &[u16], font_px: i32) -> i32 {
    // SAFETY: DC màn hình chỉ để đo; font/DC được trả lại ngay.
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() {
            return 0;
        }
        let font = ghost_font(font_px);
        let old = SelectObject(dc, font.into());
        let mut size = SIZE::default();
        let ok = GetTextExtentPoint32W(dc, units, &mut size).as_bool();
        SelectObject(dc, old);
        let _ = DeleteObject(font.into());
        ReleaseDC(None, dc);
        if ok {
            size.cx
        } else {
            0
        }
    }
}

unsafe extern "system" fn ghost_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_GETOBJECT => LRESULT(0),
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => LRESULT(1),
        WM_TIMER if wparam.0 == TIMER_WATCHDOG => {
            let owner = STATE.with(|cell| cell.borrow().as_ref().map(|s| s.owner));
            let lost = match owner {
                Some(owner) => owner != 0 && GetForegroundWindow().0 as isize != owner,
                None => true,
            };
            if lost {
                let _ = KillTimer(Some(hwnd), TIMER_WATCHDOG);
                let _ = ShowWindow(hwnd, SW_HIDE);
                STATE.with(|cell| cell.borrow_mut().take());
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_invalid() {
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let key = CreateSolidBrush(COLOR_KEY);
                let _ = FillRect(hdc, &rect, key);
                let _ = DeleteObject(key.into());
                STATE.with(|cell| {
                    if let Some(state) = cell.borrow().as_ref() {
                        let font = ghost_font(state.font_px);
                        let old = SelectObject(hdc, font.into());
                        let _ = SetBkMode(hdc, TRANSPARENT);
                        let _ = SetTextColor(hdc, COLOR_GHOST);
                        let mut text = state.text.clone();
                        let _ = DrawTextW(
                            hdc,
                            &mut text,
                            &mut rect,
                            // Đáy ô chữ = đáy caret ⇒ chân chữ mờ ngang chân chữ
                            // thật (caret cao vì giãn dòng nằm phía trên chữ).
                            DT_LEFT | DT_SINGLELINE | DT_BOTTOM | DT_NOPREFIX,
                        );
                        SelectObject(hdc, old);
                        let _ = DeleteObject(font.into());
                    }
                });
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
