// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng chọn Emoji nhẹ trên TextVN Tray (P12) — dùng khi mở từ menu khay hệ thống
//! hoặc từ Windows Hook (`Ctrl+Alt+E` ở terminal/MobaXterm/PuTTY).
//!
//! - Render emoji màu thật bằng Direct2D + DirectWrite (`Segoe UI Emoji` +
//!   `D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT`), tự động fallback GDI nếu không có D2D.
//! - Bo góc chuẩn Windows 11 qua DWM (`DWMWCP_ROUND`).
//! - `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` + `WM_MOUSEACTIVATE => MA_NOACTIVATE`:
//!   KHÔNG bao giờ cướp focus của cửa sổ đang gõ — khi người dùng click chọn emoji,
//!   `SendInput(KEYEVENTF_UNICODE)` bơm cặp UTF-16 surrogate-safe thẳng vào ô gõ đó
//!   kèm `dwExtraInfo = INJECT_TAG` để `textvn-hook` không nuốt ngược (E48).
//! - G8: không ghi log nội dung tìm kiếm hay emoji người dùng chọn; MRU chỉ lưu trong RAM.

use textvn_core::post::emoji_catalog::{search_emojis, EmojiEntry, EmojiMru};

pub const CATEGORY_TABS: &[(u8, &str)] = &[
    (0, "🕒"),
    (1, "😀"),
    (2, "👋"),
    (3, "❤️"),
    (4, "🐶"),
    (5, "🍜"),
    (6, "💻"),
    (7, "🎉"),
];

pub const GRID_COLS: usize = 8;
pub const GRID_VISIBLE_ROWS: usize = 5;
pub const POPUP_WIDTH: i32 = 332;
pub const POPUP_HEIGHT: i32 = 304;
pub const TABS_TOP: i32 = 8;
pub const TABS_HEIGHT: i32 = 34;
pub const GRID_TOP: i32 = TABS_TOP + TABS_HEIGHT + 4;
pub const FOOTER_HEIGHT: i32 = 28;
pub const PAD_X: i32 = 10;
pub const CELL_W: i32 = 39;
pub const CELL_H: i32 = 42;

/// Mã đánh dấu `dwExtraInfo` khớp `textvn_win_hook::INJECT_TAG` ("TVNH").
pub const INJECT_TAG: usize = 0x5456_4E48;

/// Trạng thái thuần của bảng Emoji trên Tray (test được trên mọi OS).
#[derive(Debug, Default)]
pub struct TrayEmojiState {
    pub visible: bool,
    pub category: u8,
    pub mru: EmojiMru,
    pub items: Vec<&'static EmojiEntry>,
    pub hovered: Option<usize>,
    pub scroll_row: usize,
}

impl TrayEmojiState {
    pub fn open(&mut self) {
        self.visible = true;
        self.category = 0;
        self.hovered = None;
        self.scroll_row = 0;
        self.reload();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.hovered = None;
    }

    pub fn reload(&mut self) {
        self.items = search_emojis("", self.category, &self.mru, 120);
        self.scroll_row = 0;
        self.hovered = None;
    }

    pub fn switch_category(&mut self, cat: u8) {
        self.category = cat;
        self.reload();
    }

    pub fn record_pick(&mut self, idx: usize) -> Option<&'static str> {
        let glyph = self.items.get(idx)?.glyph;
        self.mru.record(glyph);
        self.close();
        Some(glyph)
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let total_rows = self.items.len().div_ceil(GRID_COLS);
        let max_row = total_rows.saturating_sub(GRID_VISIBLE_ROWS) as i32;
        self.scroll_row = (self.scroll_row as i32 + delta).clamp(0, max_row) as usize;
    }

    /// Hit-test tọa độ client logic 96 DPI: `Some(Ok(cell_idx))`, `Some(Err(tab_idx))`, hoặc `None`.
    pub fn hit_test(&self, x: i32, y: i32) -> Option<Result<usize, usize>> {
        if (TABS_TOP..TABS_TOP + TABS_HEIGHT).contains(&y)
            && (PAD_X..POPUP_WIDTH - PAD_X).contains(&x)
        {
            let tab_w = (POPUP_WIDTH - 2 * PAD_X) / CATEGORY_TABS.len() as i32;
            if tab_w > 0 {
                let idx = ((x - PAD_X) / tab_w) as usize;
                if idx < CATEGORY_TABS.len() {
                    return Some(Err(idx));
                }
            }
        }
        let grid_bottom = GRID_TOP + GRID_VISIBLE_ROWS as i32 * CELL_H;
        if (GRID_TOP..grid_bottom).contains(&y)
            && (PAD_X..PAD_X + GRID_COLS as i32 * CELL_W).contains(&x)
        {
            let col = ((x - PAD_X) / CELL_W) as usize;
            let row = ((y - GRID_TOP) / CELL_H) as usize;
            let idx = (self.scroll_row + row) * GRID_COLS + col;
            if idx < self.items.len() {
                return Some(Ok(idx));
            }
        }
        None
    }
}

/// Mã hoá chuỗi emoji sang UTF-16 (giữ nguyên cặp surrogate cho emoji ngoài BMP).
pub fn encode_emoji_utf16(glyph: &str) -> Vec<u16> {
    glyph.encode_utf16().collect()
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::cell::RefCell;

    use windows::core::{s, w, Interface, GUID, HRESULT, PCWSTR};
    use windows::Win32::Foundation::{
        COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
    };
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F,
    };
    use windows::Win32::Graphics::Direct2D::{
        ID2D1DCRenderTarget, ID2D1Factory, D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
        D2D1_FACTORY_OPTIONS, D2D1_FACTORY_TYPE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
        D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
        D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
    };
    use windows::Win32::Graphics::DirectWrite::{
        IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE, DWRITE_FACTORY_TYPE_SHARED,
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
        DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
    };
    use windows::Win32::Graphics::Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
    use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
        GetMonitorInfoW, InvalidateRect, MonitorFromPoint, SelectObject, SetBkMode, SetTextColor,
        CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DT_CENTER,
        DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, HDC, MONITORINFO,
        MONITOR_DEFAULTTONEAREST, OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    };
    use windows::Win32::System::LibraryLoader::{
        GetModuleHandleW, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        VIRTUAL_KEY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, GetCursorPos, RegisterClassExW, SetWindowPos, ShowWindow,
        CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST, MA_NOACTIVATE, SWP_NOACTIVATE, SW_HIDE,
        SW_SHOWNOACTIVATE, WM_ERASEBKGND, WM_GETOBJECT, WM_LBUTTONDOWN, WM_MOUSEACTIVATE,
        WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_RBUTTONDOWN, WNDCLASSEXW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    const CLASS_NAME: PCWSTR = w!("TextVNTrayEmojiPopup");

    /// E66: nạp động `GetDpiForWindow` (Win10 1607+) — Win10 1507/LTSB 2015 rơi về 96 DPI.
    fn window_dpi_safe(hwnd: HWND) -> i32 {
        type FnType = unsafe extern "system" fn(HWND) -> u32;
        static FN: std::sync::OnceLock<Option<FnType>> = std::sync::OnceLock::new();
        let f = *FN.get_or_init(|| unsafe {
            let h = GetModuleHandleW(w!("user32.dll")).ok()?;
            let p = GetProcAddress(h, s!("GetDpiForWindow"))?;
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                FnType,
            >(p))
        });
        f.map_or(96, |call| unsafe { call(hwnd) }).max(96) as i32
    }

    /// E66: nạp lười `d2d1.dll` từ System32 chỉ khi mở Emoji Popup.
    fn lazy_d2d1_create_factory() -> Option<ID2D1Factory> {
        type FnType = unsafe extern "system" fn(
            D2D1_FACTORY_TYPE,
            *const GUID,
            *const D2D1_FACTORY_OPTIONS,
            *mut *mut core::ffi::c_void,
        ) -> HRESULT;
        static FN: std::sync::OnceLock<Option<FnType>> = std::sync::OnceLock::new();
        let f = (*FN.get_or_init(|| unsafe {
            let h = LoadLibraryExW(w!("d2d1.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
            let p = GetProcAddress(h, s!("D2D1CreateFactory"))?;
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                FnType,
            >(p))
        }))?;
        let mut raw: *mut core::ffi::c_void = std::ptr::null_mut();
        let hr = unsafe {
            f(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &ID2D1Factory::IID,
                std::ptr::null(),
                &mut raw,
            )
        };
        if hr.is_ok() && !raw.is_null() {
            Some(unsafe { ID2D1Factory::from_raw(raw) })
        } else {
            None
        }
    }

    /// E66: nạp lười `dwrite.dll` từ System32 chỉ khi mở Emoji Popup.
    fn lazy_dwrite_create_factory() -> Option<IDWriteFactory> {
        type FnType = unsafe extern "system" fn(
            DWRITE_FACTORY_TYPE,
            *const GUID,
            *mut *mut core::ffi::c_void,
        ) -> HRESULT;
        static FN: std::sync::OnceLock<Option<FnType>> = std::sync::OnceLock::new();
        let f = (*FN.get_or_init(|| unsafe {
            let h = LoadLibraryExW(w!("dwrite.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
            let p = GetProcAddress(h, s!("DWriteCreateFactory"))?;
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                FnType,
            >(p))
        }))?;
        let mut raw: *mut core::ffi::c_void = std::ptr::null_mut();
        let hr = unsafe { f(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory::IID, &mut raw) };
        if hr.is_ok() && !raw.is_null() {
            Some(unsafe { IDWriteFactory::from_raw(raw) })
        } else {
            None
        }
    }

    /// E66: bo góc Win11 qua `dwmapi.dll` nạp lười (Win10 bỏ qua êm).
    fn lazy_dwm_round_corners(hwnd: HWND) {
        type FnType =
            unsafe extern "system" fn(HWND, u32, *const core::ffi::c_void, u32) -> HRESULT;
        static FN: std::sync::OnceLock<Option<FnType>> = std::sync::OnceLock::new();
        let Some(f) = *FN.get_or_init(|| unsafe {
            let h = LoadLibraryExW(w!("dwmapi.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32).ok()?;
            let p = GetProcAddress(h, s!("DwmSetWindowAttribute"))?;
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                FnType,
            >(p))
        }) else {
            return;
        };
        let pref = DWMWCP_ROUND;
        let _ = unsafe {
            f(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE.0 as u32,
                (&pref as *const _) as *const core::ffi::c_void,
                std::mem::size_of_val(&pref) as u32,
            )
        };
    }

    struct D2d {
        rt: ID2D1DCRenderTarget,
        emoji_fmt: IDWriteTextFormat,
        tab_fmt: IDWriteTextFormat,
        small_fmt: IDWriteTextFormat,
    }

    impl D2d {
        fn new() -> Option<Self> {
            unsafe {
                let d2d = lazy_d2d1_create_factory()?;
                let props = D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                };
                let rt = d2d.CreateDCRenderTarget(&props).ok()?;
                let dw = lazy_dwrite_create_factory()?;
                let emoji_fmt = dw
                    .CreateTextFormat(
                        w!("Segoe UI Emoji"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        22.0,
                        w!(""),
                    )
                    .ok()?;
                let _ = emoji_fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                let _ = emoji_fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

                let tab_fmt = dw
                    .CreateTextFormat(
                        w!("Segoe UI Emoji"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        15.0,
                        w!(""),
                    )
                    .ok()?;
                let _ = tab_fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                let _ = tab_fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

                let small_fmt = dw
                    .CreateTextFormat(
                        w!("Segoe UI"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        11.5,
                        w!(""),
                    )
                    .ok()?;
                let _ = small_fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
                let _ = small_fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

                Some(Self {
                    rt,
                    emoji_fmt,
                    tab_fmt,
                    small_fmt,
                })
            }
        }
    }

    struct PopupSlot {
        hwnd: HWND,
        state: TrayEmojiState,
        d2d: Option<D2d>,
    }

    thread_local! {
        static POPUP: RefCell<Option<PopupSlot>> = const { RefCell::new(None) };
    }

    /// Mở hoặc đóng bảng Emoji nổi trên thread giao diện của Tray (không cướp focus).
    pub fn toggle_tray_emoji_popup() {
        POPUP.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_none() {
                if let Some(hwnd) = create_window() {
                    *slot = Some(PopupSlot {
                        hwnd,
                        state: TrayEmojiState::default(),
                        d2d: D2d::new(),
                    });
                } else {
                    return;
                }
            }
            let Some(p) = slot.as_mut() else { return };
            if p.state.visible {
                p.state.close();
                unsafe {
                    let _ = ShowWindow(p.hwnd, SW_HIDE);
                }
                return;
            }
            p.state.open();
            let mut pt = POINT::default();
            unsafe {
                let _ = GetCursorPos(&mut pt);
            }
            let dpi = window_dpi_safe(p.hwnd);
            let w_px = (POPUP_WIDTH * dpi) / 96;
            let h_px = (POPUP_HEIGHT * dpi) / 96;
            let (x, y) = clamp_near_point(pt.x, pt.y, w_px, h_px);
            unsafe {
                let _ = SetWindowPos(p.hwnd, Some(HWND_TOPMOST), x, y, w_px, h_px, SWP_NOACTIVATE);
                let _ = ShowWindow(p.hwnd, SW_SHOWNOACTIVATE);
                let _ = InvalidateRect(Some(p.hwnd), None, false);
            }
        });
    }

    fn clamp_near_point(px: i32, py: i32, w: i32, h: i32) -> (i32, i32) {
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = unsafe {
            let hmon = MonitorFromPoint(POINT { x: px, y: py }, MONITOR_DEFAULTTONEAREST);
            if GetMonitorInfoW(hmon, &mut mi).as_bool() {
                mi.rcWork
            } else {
                RECT {
                    left: 0,
                    top: 0,
                    right: 1920,
                    bottom: 1080,
                }
            }
        };
        let x = (px - w / 2).clamp(work.left + 8, (work.right - w - 8).max(work.left + 8));
        let y = if py - h - 12 >= work.top {
            py - h - 12
        } else {
            (py + 12).clamp(work.top + 8, (work.bottom - h - 8).max(work.top + 8))
        };
        (x, y)
    }

    fn create_window() -> Option<HWND> {
        unsafe {
            let hmod = GetModuleHandleW(None).ok()?;
            let hinst = HINSTANCE(hmod.0);
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinst,
                lpszClassName: CLASS_NAME,
                ..Default::default()
            };
            let _ = RegisterClassExW(&wc);
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!("TextVN Emoji"),
                WS_POPUP,
                0,
                0,
                POPUP_WIDTH,
                POPUP_HEIGHT,
                None,
                None,
                Some(hinst),
                None,
            )
            .ok()?;
            lazy_dwm_round_corners(hwnd);
            Some(hwnd)
        }
    }

    fn inject_unicode_emoji(glyph: &str) {
        let units = encode_emoji_utf16(glyph);
        if units.is_empty() {
            return;
        }
        let mut inputs = Vec::with_capacity(units.len() * 2);
        for u in units {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: u,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: INJECT_TAG,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: u,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: INJECT_TAG,
                    },
                },
            });
        }
        unsafe {
            let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_GETOBJECT => LRESULT(0),
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
            WM_ERASEBKGND => LRESULT(1),
            WM_RBUTTONDOWN => {
                POPUP.with(|cell| {
                    if let Some(p) = cell.borrow_mut().as_mut() {
                        p.state.close();
                    }
                });
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                POPUP.with(|cell| {
                    if let Some(p) = cell.borrow().as_ref() {
                        paint(hwnd, hdc, p);
                    }
                });
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let dpi = window_dpi_safe(hwnd);
                let lx = ((lparam.0 as i16) as i32 * 96) / dpi;
                let ly = (((lparam.0 >> 16) as i16) as i32 * 96) / dpi;
                POPUP.with(|cell| {
                    if let Some(p) = cell.borrow_mut().as_mut() {
                        let next = match p.state.hit_test(lx, ly) {
                            Some(Ok(i)) => Some(i),
                            _ => None,
                        };
                        if p.state.hovered != next {
                            p.state.hovered = next;
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                    }
                });
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam.0 >> 16) as i16) as i32;
                let step = if delta > 0 { -2 } else { 2 };
                POPUP.with(|cell| {
                    if let Some(p) = cell.borrow_mut().as_mut() {
                        p.state.scroll_by(step);
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                });
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let dpi = window_dpi_safe(hwnd);
                let lx = ((lparam.0 as i16) as i32 * 96) / dpi;
                let ly = (((lparam.0 >> 16) as i16) as i32 * 96) / dpi;
                let mut picked: Option<&'static str> = None;
                POPUP.with(|cell| {
                    if let Some(p) = cell.borrow_mut().as_mut() {
                        match p.state.hit_test(lx, ly) {
                            Some(Err(tab_idx)) => {
                                p.state.switch_category(CATEGORY_TABS[tab_idx].0);
                                let _ = InvalidateRect(Some(hwnd), None, false);
                            }
                            Some(Ok(cell_idx)) => {
                                picked = p.state.record_pick(cell_idx);
                                let _ = ShowWindow(hwnd, SW_HIDE);
                            }
                            None => {}
                        }
                    }
                });
                if let Some(glyph) = picked {
                    inject_unicode_emoji(glyph);
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn rgba(r: u8, g: u8, b: u8, a: f32) -> D2D1_COLOR_F {
        D2D1_COLOR_F {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    unsafe fn paint(hwnd: HWND, hdc: HDC, p: &PopupSlot) {
        let dpi = window_dpi_safe(hwnd);
        let rc = RECT {
            left: 0,
            top: 0,
            right: (POPUP_WIDTH * dpi) / 96,
            bottom: (POPUP_HEIGHT * dpi) / 96,
        };
        if let Some(d2d) = p.d2d.as_ref() {
            if paint_d2d(d2d, hdc, &rc, dpi as f32, &p.state).is_ok() {
                return;
            }
        }
        let bg = CreateSolidBrush(COLORREF(0x0026_201E));
        FillRect(hdc, &rc, bg);
        let _ = DeleteObject(bg.into());
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, COLORREF(0x00F6_F1EE));
        let font = CreateFontW(
            -(18 * dpi / 96),
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
            DEFAULT_QUALITY,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            w!("Segoe UI Emoji"),
        );
        let old = SelectObject(hdc, font.into());
        let start = p.state.scroll_row * GRID_COLS;
        for row in 0..GRID_VISIBLE_ROWS {
            for col in 0..GRID_COLS {
                let Some(it) = p.state.items.get(start + row * GRID_COLS + col) else {
                    break;
                };
                let mut cell = RECT {
                    left: ((PAD_X + col as i32 * CELL_W) * dpi) / 96,
                    top: ((GRID_TOP + row as i32 * CELL_H) * dpi) / 96,
                    right: ((PAD_X + (col as i32 + 1) * CELL_W) * dpi) / 96,
                    bottom: ((GRID_TOP + (row as i32 + 1) * CELL_H) * dpi) / 96,
                };
                let mut u16s = encode_emoji_utf16(it.glyph);
                let _ = DrawTextW(
                    hdc,
                    &mut u16s,
                    &mut cell,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }
        }
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }

    unsafe fn paint_d2d(
        d2d: &D2d,
        hdc: HDC,
        rc: &RECT,
        dpi: f32,
        state: &TrayEmojiState,
    ) -> windows::core::Result<()> {
        d2d.rt.BindDC(hdc, rc)?;
        d2d.rt.SetDpi(dpi, dpi);
        d2d.rt.BeginDraw();
        d2d.rt.Clear(Some(&rgba(30, 32, 38, 1.0)));
        let border = d2d.rt.CreateSolidColorBrush(&rgba(62, 66, 78, 1.0), None)?;
        let accent = d2d
            .rt
            .CreateSolidColorBrush(&rgba(56, 132, 244, 0.35), None)?;
        let line = d2d
            .rt
            .CreateSolidColorBrush(&rgba(88, 166, 255, 1.0), None)?;
        let hover = d2d
            .rt
            .CreateSolidColorBrush(&rgba(255, 255, 255, 0.10), None)?;
        let text = d2d
            .rt
            .CreateSolidColorBrush(&rgba(238, 241, 246, 1.0), None)?;
        let muted = d2d
            .rt
            .CreateSolidColorBrush(&rgba(154, 162, 178, 1.0), None)?;

        d2d.rt.DrawRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.5,
                    top: 0.5,
                    right: POPUP_WIDTH as f32 - 0.5,
                    bottom: POPUP_HEIGHT as f32 - 0.5,
                },
                radiusX: 8.0,
                radiusY: 8.0,
            },
            &border,
            1.0,
            None,
        );

        let tab_w = (POPUP_WIDTH - 2 * PAD_X) as f32 / CATEGORY_TABS.len() as f32;
        for (i, &(cat_id, icon)) in CATEGORY_TABS.iter().enumerate() {
            let x0 = PAD_X as f32 + i as f32 * tab_w;
            let tr = D2D_RECT_F {
                left: x0,
                top: TABS_TOP as f32,
                right: x0 + tab_w,
                bottom: (TABS_TOP + TABS_HEIGHT) as f32,
            };
            if state.category == cat_id {
                d2d.rt.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: tr.left + 2.0,
                            top: tr.top + 2.0,
                            right: tr.right - 2.0,
                            bottom: tr.bottom - 4.0,
                        },
                        radiusX: 5.0,
                        radiusY: 5.0,
                    },
                    &accent,
                );
                d2d.rt.FillRectangle(
                    &D2D_RECT_F {
                        left: tr.left + 8.0,
                        top: tr.bottom - 3.0,
                        right: tr.right - 8.0,
                        bottom: tr.bottom - 1.0,
                    },
                    &line,
                );
            }
            let u16s = encode_emoji_utf16(icon);
            d2d.rt.DrawText(
                &u16s,
                &d2d.tab_fmt,
                &tr,
                &text,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }

        let start = state.scroll_row * GRID_COLS;
        for row in 0..GRID_VISIBLE_ROWS {
            for col in 0..GRID_COLS {
                let idx = start + row * GRID_COLS + col;
                let Some(it) = state.items.get(idx) else {
                    break;
                };
                let cell = D2D_RECT_F {
                    left: (PAD_X + col as i32 * CELL_W) as f32 + 2.0,
                    top: (GRID_TOP + row as i32 * CELL_H) as f32 + 2.0,
                    right: (PAD_X + (col as i32 + 1) * CELL_W) as f32 - 2.0,
                    bottom: (GRID_TOP + (row as i32 + 1) * CELL_H) as f32 - 2.0,
                };
                if state.hovered == Some(idx) {
                    d2d.rt.FillRoundedRectangle(
                        &D2D1_ROUNDED_RECT {
                            rect: cell,
                            radiusX: 6.0,
                            radiusY: 6.0,
                        },
                        &hover,
                    );
                }
                let u16s = encode_emoji_utf16(it.glyph);
                d2d.rt.DrawText(
                    &u16s,
                    &d2d.emoji_fmt,
                    &cell,
                    &text,
                    D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }

        let fy = (POPUP_HEIGHT - FOOTER_HEIGHT) as f32;
        d2d.rt.FillRectangle(
            &D2D_RECT_F {
                left: PAD_X as f32,
                top: fy,
                right: (POPUP_WIDTH - PAD_X) as f32,
                bottom: fy + 1.0,
            },
            &border,
        );
        let footer = match state.hovered.and_then(|i| state.items.get(i)) {
            Some(it) => format!(":{}:  ·  Click chèn · Chuột phải đóng", it.shortcode),
            None => "Bảng Emoji TextVN  ·  Click chèn · Chuột phải đóng".to_string(),
        };
        let fu16 = encode_emoji_utf16(&footer);
        d2d.rt.DrawText(
            &fu16,
            &d2d.small_fmt,
            &D2D_RECT_F {
                left: PAD_X as f32,
                top: fy + 2.0,
                right: (POPUP_WIDTH - PAD_X) as f32,
                bottom: POPUP_HEIGHT as f32 - 2.0,
            },
            &muted,
            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        d2d.rt.EndDraw(None, None)
    }
}

#[cfg(windows)]
pub use win::toggle_tray_emoji_popup;

#[cfg(not(windows))]
pub fn toggle_tray_emoji_popup() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_emoji_state_pick_and_surrogate_utf16() {
        let mut st = TrayEmojiState::default();
        st.open();
        assert!(st.visible);
        assert!(!st.items.is_empty());

        st.switch_category(7);
        let rocket_idx = st
            .items
            .iter()
            .position(|e| e.glyph == "🚀")
            .expect("rocket in activities");
        assert_eq!(st.record_pick(rocket_idx), Some("🚀"));
        assert!(!st.visible);

        // 🚀 (U+1F680) là cặp surrogate 2 code unit UTF-16.
        assert_eq!(encode_emoji_utf16("🚀"), vec![0xD83D, 0xDE80]);

        st.open();
        assert_eq!(st.items[0].glyph, "🚀");
    }
}
