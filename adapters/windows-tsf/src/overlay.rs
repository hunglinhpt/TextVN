// SPDX-License-Identifier: GPL-3.0-or-later
//! Candidate / Assist Overlay UI cho Windows TSF (AS-030 · D4 / D30).
//!
//! Hiển thị panel gợi ý / sửa lỗi cạnh con trỏ (caret) của ứng dụng mà không chiếm focus:
//! - Window styles: WS_POPUP, WS_BORDER
//! - Extended styles: WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST
//! - Xử lý WM_MOUSEACTIVATE -> MA_NOACTIVATE (không chiếm focus của app đang gõ)
//! - Tự đóng khi: người dùng gõ tiếp, bấm Esc, chuyển focus, hoặc sau 8 giây (WM_TIMER).
//! - Fail-open: nếu API Win32 / GDI lỗi, im lặng bỏ qua, không bao giờ làm app sập.

use std::cell::RefCell;

#[cfg(windows)]
use std::cell::Cell;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};

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

#[cfg(windows)]
const OVERLAY_CLASS_NAME: PCWSTR = windows::core::w!("TextVNAssistOverlay");
#[cfg(windows)]
const TIMER_AUTO_DISMISS: usize = 1001;
#[cfg(windows)]
const AUTO_DISMISS_TIMEOUT_MS: u32 = 8000;

thread_local! {
    static CURRENT_OVERLAY_TEXT: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
}

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

    /// Đảm bảo window class được đăng ký một lần trong process.
    fn ensure_class_registered() -> bool {
        static REGISTERED: AtomicBool = AtomicBool::new(false);
        if REGISTERED.load(Ordering::Acquire) {
            return true;
        }
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
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

    /// Lấy hoặc tạo HWND của overlay (fail-open: trả HWND(0) nếu lỗi).
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
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                OVERLAY_CLASS_NAME,
                windows::core::w!("TextVNAssistCandidate"),
                WS_POPUP | WS_BORDER,
                0,
                0,
                100,
                30,
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

    /// Hiển thị overlay tại vị trí caret `(x, y)` kèm nội dung gợi ý/sửa lỗi.
    pub fn show(&self, x: i32, y: i32, text: &str) {
        let hwnd = self.get_or_create_hwnd();
        if hwnd.0.is_null() {
            return;
        }

        let wide: Vec<u16> = text.encode_utf16().collect();
        let width = ((wide.len() as i32) * 8 + 24).clamp(120, 450);
        let height = 28;

        CURRENT_OVERLAY_TEXT.with(|cell| {
            let mut buf = cell.borrow_mut();
            *buf = wide;
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
            let _ = SetTimer(
                Some(hwnd),
                TIMER_AUTO_DISMISS,
                AUTO_DISMISS_TIMEOUT_MS,
                None,
            );
        }
    }

    /// Ẩn overlay nếu đang hiển thị.
    pub fn hide(&self) {
        let hwnd = self.hwnd.get();
        if !hwnd.0.is_null() && unsafe { IsWindow(Some(hwnd)).as_bool() } {
            unsafe {
                let _ = KillTimer(Some(hwnd), TIMER_AUTO_DISMISS);
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    /// Kiểm tra overlay có đang hiển thị hay không.
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
    }
}

#[cfg(windows)]
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_TIMER => {
            if wparam.0 == TIMER_AUTO_DISMISS {
                let _ = KillTimer(Some(hwnd), TIMER_AUTO_DISMISS);
                let _ = ShowWindow(hwnd, SW_HIDE);
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
    let _ = GetClientRect(hwnd, &mut rect);

    // Nền sáng nhẹ phong cách tooltip hiện đại (RGB 248, 249, 250)
    let bg_brush = CreateSolidBrush(COLORREF(0x00FAF9F8));
    let _ = FillRect(hdc, &rect, bg_brush);
    let _ = DeleteObject(bg_brush.into());

    // Viền nhẹ (RGB 210, 215, 220)
    let border_brush = CreateSolidBrush(COLORREF(0x00DCD7D2));
    let _ = FrameRect(hdc, &rect, border_brush);
    let _ = DeleteObject(border_brush.into());

    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, COLORREF(0x00202020));

    CURRENT_OVERLAY_TEXT.with(|cell| {
        let text = cell.borrow();
        if !text.is_empty() {
            let mut text_rect = RECT {
                left: rect.left + 8,
                top: rect.top + 2,
                right: rect.right - 8,
                bottom: rect.bottom - 2,
            };
            let mut copy = text.clone();
            let _ = DrawTextW(
                hdc,
                &mut copy,
                &mut text_rect,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }
    });
}

#[cfg(not(windows))]
pub struct CandidateOverlay;

#[cfg(not(windows))]
impl CandidateOverlay {
    pub fn new() -> Self {
        Self
    }
    pub fn show(&self, _x: i32, _y: i32, _text: &str) {}
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

    #[test]
    fn overlay_creation_and_visibility() {
        let overlay = CandidateOverlay::new();
        assert!(!overlay.is_visible());
        overlay.hide();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn overlay_shows_sample_issue_and_hides() {
        let overlay = CandidateOverlay::new();
        assert!(!overlay.is_visible());
        overlay.show(100, 200, "[1] Sửa: tôi đi học");
        overlay.hide();
        assert!(!overlay.is_visible());
    }
}
