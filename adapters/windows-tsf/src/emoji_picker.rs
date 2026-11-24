// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng chọn Emoji nhẹ phong cách Windows 11 / MS Teams / iOS cho TextVN TSF (P12).
//!
//! Thiết kế siêu nhẹ & không cướp focus:
//! - Cửa sổ `WS_POPUP` + `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` (mọi click
//!   trả `MA_NOACTIVATE`, `WM_GETOBJECT => 0` tránh treo UIA/screen reader).
//! - Vẽ emoji màu thật qua Direct2D + DirectWrite (`D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT`
//!   với font `Segoe UI Emoji`), tự động fallback sang GDI nếu môi trường không có D2D.
//! - Bo góc chuẩn Fluent qua DWM (`DWMWCP_ROUND`).
//! - Điều khiển hoàn toàn bằng bàn phím (gõ lọc trực tiếp không mất focus app, mũi tên `←↑↓→`,
//!   `Tab`/`Shift+Tab` chuyển nhóm, `Enter` chèn, `Esc` đóng) hoặc chuột (click + cuộn bánh xe).
//! - G8: không bao giờ ghi log query tìm kiếm hay emoji người dùng chọn.

/// 8 nhóm hiển thị giống iOS / MS Teams / Windows Emoji Panel.
pub const CATEGORY_TABS: &[(u8, &str, &str)] = &[
    (0, "🕒", "Gần đây"),
    (1, "😀", "Cảm xúc"),
    (2, "👋", "Cử chỉ"),
    (3, "❤️", "Biểu tượng"),
    (4, "🐶", "Thiên nhiên"),
    (5, "🍜", "Đồ ăn"),
    (6, "💻", "Công việc"),
    (7, "🎉", "Hoạt động"),
];

pub const GRID_COLS: usize = 8;
pub const GRID_VISIBLE_ROWS: usize = 5;
pub const POPUP_WIDTH: i32 = 332;
pub const POPUP_HEIGHT: i32 = 318;
pub const HEADER_HEIGHT: i32 = 40;
pub const TABS_HEIGHT: i32 = 34;
pub const FOOTER_HEIGHT: i32 = 28;
pub const PAD_X: i32 = 10;
pub const CELL_W: i32 = 39;
pub const CELL_H: i32 = 42;
pub const GRID_TOP: i32 = HEADER_HEIGHT + TABS_HEIGHT + 4;

/// Một mục emoji đã parse từ kết quả TSV của `ime_emoji_search`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiItem {
    pub glyph: String,
    pub shortcode: String,
    pub category: u8,
}

/// Parse chuỗi `glyph\tshortcode\tcategory\n` từ FFI `ime_emoji_search`.
pub fn parse_emoji_tsv(tsv: &str) -> Vec<EmojiItem> {
    let mut out = Vec::new();
    for line in tsv.lines() {
        let mut parts = line.split('\t');
        let Some(glyph) = parts.next() else { continue };
        let Some(shortcode) = parts.next() else {
            continue;
        };
        let category = parts.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);
        if !glyph.is_empty() && !shortcode.is_empty() {
            out.push(EmojiItem {
                glyph: glyph.to_string(),
                shortcode: shortcode.to_string(),
                category,
            });
        }
    }
    out
}

/// Kết quả xử lý 1 phím khi Emoji Picker đang mở.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerKeyAction {
    /// Chỉ cập nhật lại giao diện / truy vấn.
    Redraw,
    /// Query hoặc category vừa đổi — caller cần truy vấn lại `ime_emoji_search`.
    Reload,
    /// Người dùng chọn chèn emoji (glyph UTF-8).
    Commit(String),
    /// Đóng bảng chọn.
    Close,
}

/// Kết quả hit-test chuột trên cửa sổ Emoji Picker (tọa độ client logic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerHit {
    None,
    Tab(usize),
    Cell(usize),
}

/// Trạng thái thuần của Emoji Picker (test được đầy đủ không cần Win32).
#[derive(Debug, Clone, Default)]
pub struct EmojiPickerState {
    pub visible: bool,
    pub query: String,
    pub category: u8,
    pub items: Vec<EmojiItem>,
    pub selected: usize,
    pub hovered: Option<usize>,
    pub scroll_row: usize,
}

impl EmojiPickerState {
    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.category = 0;
        self.selected = 0;
        self.hovered = None;
        self.scroll_row = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.hovered = None;
        self.scroll_row = 0;
    }

    pub fn set_items(&mut self, items: Vec<EmojiItem>) {
        self.items = items;
        if self.selected >= self.items.len() {
            self.selected = 0;
        }
        self.scroll_row = 0;
        self.hovered = None;
        self.ensure_selected_visible();
    }

    pub fn selected_item(&self) -> Option<&EmojiItem> {
        self.items.get(self.selected)
    }

    pub fn total_rows(&self) -> usize {
        self.items.len().div_ceil(GRID_COLS)
    }

    pub fn max_scroll_row(&self) -> usize {
        self.total_rows().saturating_sub(GRID_VISIBLE_ROWS)
    }

    fn ensure_selected_visible(&mut self) {
        let row = self.selected / GRID_COLS;
        if row < self.scroll_row {
            self.scroll_row = row;
        } else if row >= self.scroll_row + GRID_VISIBLE_ROWS {
            self.scroll_row = row + 1 - GRID_VISIBLE_ROWS;
        }
        if self.scroll_row > self.max_scroll_row() {
            self.scroll_row = self.max_scroll_row();
        }
    }

    pub fn scroll_by(&mut self, delta_rows: i32) {
        let max_r = self.max_scroll_row() as i32;
        let next = (self.scroll_row as i32 + delta_rows).clamp(0, max_r);
        self.scroll_row = next as usize;
    }

    pub fn switch_category(&mut self, cat: u8) {
        self.category = cat;
        self.query.clear();
        self.selected = 0;
        self.scroll_row = 0;
        self.hovered = None;
    }

    pub fn cycle_category(&mut self, backward: bool) {
        let cur = CATEGORY_TABS
            .iter()
            .position(|&(id, _, _)| id == self.category)
            .unwrap_or(0);
        let len = CATEGORY_TABS.len();
        let next = if backward {
            (cur + len - 1) % len
        } else {
            (cur + 1) % len
        };
        self.switch_category(CATEGORY_TABS[next].0);
    }

    /// Xử lý phím khi bảng đang mở (VK chuẩn Windows).
    pub fn on_key(&mut self, vk: u32, ch: Option<char>, shift: bool) -> PickerKeyAction {
        match vk {
            0x1B => {
                // VK_ESCAPE
                self.close();
                PickerKeyAction::Close
            }
            0x0D => {
                // VK_RETURN
                if let Some(item) = self.selected_item() {
                    let glyph = item.glyph.clone();
                    self.close();
                    PickerKeyAction::Commit(glyph)
                } else {
                    self.close();
                    PickerKeyAction::Close
                }
            }
            0x09 => {
                // VK_TAB: đổi nhóm emoji
                self.cycle_category(shift);
                PickerKeyAction::Reload
            }
            0x08 => {
                // VK_BACK
                if self.query.pop().is_some() {
                    self.selected = 0;
                    self.scroll_row = 0;
                    PickerKeyAction::Reload
                } else {
                    PickerKeyAction::Redraw
                }
            }
            0x25 => {
                // VK_LEFT
                if self.selected > 0 {
                    self.selected -= 1;
                    self.ensure_selected_visible();
                }
                PickerKeyAction::Redraw
            }
            0x27 => {
                // VK_RIGHT
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                    self.ensure_selected_visible();
                }
                PickerKeyAction::Redraw
            }
            0x26 => {
                // VK_UP
                if self.selected >= GRID_COLS {
                    self.selected -= GRID_COLS;
                    self.ensure_selected_visible();
                }
                PickerKeyAction::Redraw
            }
            0x28 => {
                // VK_DOWN
                if !self.items.is_empty() {
                    let next = (self.selected + GRID_COLS).min(self.items.len() - 1);
                    self.selected = next;
                    self.ensure_selected_visible();
                }
                PickerKeyAction::Redraw
            }
            _ => {
                if let Some(c) = ch {
                    if !c.is_control() && self.query.chars().count() < 32 {
                        self.query.push(c);
                        self.selected = 0;
                        self.scroll_row = 0;
                        return PickerKeyAction::Reload;
                    }
                }
                PickerKeyAction::Redraw
            }
        }
    }

    /// Hit-test tọa độ logic `(x, y)` (tính theo pixel 96 DPI).
    pub fn hit_test(&self, x: i32, y: i32) -> PickerHit {
        if (HEADER_HEIGHT..HEADER_HEIGHT + TABS_HEIGHT).contains(&y)
            && (PAD_X..POPUP_WIDTH - PAD_X).contains(&x)
        {
            let tab_w = (POPUP_WIDTH - 2 * PAD_X) / (CATEGORY_TABS.len() as i32);
            if tab_w > 0 {
                let idx = ((x - PAD_X) / tab_w) as usize;
                if idx < CATEGORY_TABS.len() {
                    return PickerHit::Tab(idx);
                }
            }
        }
        let grid_bottom = GRID_TOP + (GRID_VISIBLE_ROWS as i32) * CELL_H;
        if (GRID_TOP..grid_bottom).contains(&y)
            && (PAD_X..PAD_X + (GRID_COLS as i32) * CELL_W).contains(&x)
        {
            let col = ((x - PAD_X) / CELL_W) as usize;
            let vis_row = ((y - GRID_TOP) / CELL_H) as usize;
            let idx = (self.scroll_row + vis_row) * GRID_COLS + col;
            if idx < self.items.len() {
                return PickerHit::Cell(idx);
            }
        }
        PickerHit::None
    }
}

// =========================================================================
// Win32 Native Floating Popup (Direct2D/DirectWrite Color Emoji + GDI fallback)
// =========================================================================

#[cfg(windows)]
mod win {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{
        COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
    };
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F,
    };
    use windows::Win32::Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory,
        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT, D2D1_FACTORY_TYPE_SINGLE_THREADED,
        D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
        D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
    };
    use windows::Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
        DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
        DWRITE_TEXT_ALIGNMENT_LEADING,
    };
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
        GetMonitorInfoW, InvalidateRect, MonitorFromPoint, SelectObject, SetBkMode, SetTextColor,
        CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DT_CENTER, DT_LEFT,
        DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_NORMAL, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        OUT_DEFAULT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::GetDpiForWindow;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetWindowLongPtrW,
        RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW, CS_HREDRAW,
        CS_VREDRAW, GWLP_USERDATA, HWND_TOPMOST, MA_NOACTIVATE, SWP_NOACTIVATE, SW_HIDE,
        SW_SHOWNOACTIVATE, WM_ERASEBKGND, WM_GETOBJECT, WM_LBUTTONDOWN, WM_MOUSEACTIVATE,
        WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_PAINT, WNDCLASSEXW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    const CLASS_NAME: PCWSTR = w!("TextVNEmojiPickerWnd");

    type ClickCallback = Rc<dyn Fn(PickerClickEvent)>;

    #[derive(Debug, Clone)]
    pub enum PickerClickEvent {
        SwitchTab(u8),
        CommitGlyph(String),
    }

    struct D2dResources {
        rt: ID2D1DCRenderTarget,
        emoji_fmt: IDWriteTextFormat,
        tab_fmt: IDWriteTextFormat,
        ui_fmt: IDWriteTextFormat,
        small_fmt: IDWriteTextFormat,
    }

    impl D2dResources {
        fn new() -> Option<Self> {
            unsafe {
                let d2d: ID2D1Factory =
                    D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).ok()?;
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
                let dwrite: IDWriteFactory =
                    DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).ok()?;

                let emoji_fmt = dwrite
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

                let tab_fmt = dwrite
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

                let ui_fmt = dwrite
                    .CreateTextFormat(
                        w!("Segoe UI"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        13.0,
                        w!(""),
                    )
                    .ok()?;
                let _ = ui_fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
                let _ = ui_fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

                let small_fmt = dwrite
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
                    ui_fmt,
                    small_fmt,
                })
            }
        }
    }

    struct Inner {
        hwnd: HWND,
        state: EmojiPickerState,
        d2d: Option<D2dResources>,
        on_click: Option<ClickCallback>,
    }

    /// Quản lý vòng đời cửa sổ Emoji Picker cho 1 luồng UI TSF.
    pub struct EmojiPicker {
        inner: Rc<RefCell<Inner>>,
    }

    impl Default for EmojiPicker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl EmojiPicker {
        pub fn new() -> Self {
            Self {
                inner: Rc::new(RefCell::new(Inner {
                    hwnd: HWND::default(),
                    state: EmojiPickerState::default(),
                    d2d: None,
                    on_click: None,
                })),
            }
        }

        pub fn set_click_handler<F: Fn(PickerClickEvent) + 'static>(&self, f: F) {
            if let Ok(mut inner) = self.inner.try_borrow_mut() {
                inner.on_click = Some(Rc::new(f));
            }
        }

        pub fn is_visible(&self) -> bool {
            self.inner
                .try_borrow()
                .map(|i| i.state.visible)
                .unwrap_or(false)
        }

        pub fn query_and_category(&self) -> (String, u8) {
            self.inner
                .try_borrow()
                .map(|i| (i.state.query.clone(), i.state.category))
                .unwrap_or_else(|_| (String::new(), 0))
        }

        pub fn show(&self, anchor: Option<(i32, i32, i32)>, items: Vec<EmojiItem>) {
            let Ok(mut inner) = self.inner.try_borrow_mut() else {
                return;
            };
            if inner.hwnd.is_invalid() {
                let raw = Rc::as_ptr(&self.inner) as isize;
                if let Some(hwnd) = create_popup_window(raw) {
                    inner.hwnd = hwnd;
                    inner.d2d = D2dResources::new();
                } else {
                    return;
                }
            }
            inner.state.open();
            inner.state.set_items(items);

            let hwnd = inner.hwnd;
            let (ax, ay_top, ay_bot) = anchor.unwrap_or_else(|| {
                let mut pt = POINT::default();
                unsafe {
                    let _ = GetCursorPos(&mut pt);
                }
                (pt.x, pt.y, pt.y + 20)
            });
            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as i32;
            let w_px = (POPUP_WIDTH * dpi) / 96;
            let h_px = (POPUP_HEIGHT * dpi) / 96;
            let (x, y) = clamp_to_monitor(ax, ay_top, ay_bot, w_px, h_px);
            unsafe {
                let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, w_px, h_px, SWP_NOACTIVATE);
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
        }

        pub fn hide(&self) {
            let Ok(mut inner) = self.inner.try_borrow_mut() else {
                return;
            };
            inner.state.close();
            if !inner.hwnd.is_invalid() {
                unsafe {
                    let _ = ShowWindow(inner.hwnd, SW_HIDE);
                }
            }
        }

        pub fn set_items_and_redraw(&self, items: Vec<EmojiItem>) {
            let Ok(mut inner) = self.inner.try_borrow_mut() else {
                return;
            };
            inner.state.set_items(items);
            if !inner.hwnd.is_invalid() && inner.state.visible {
                unsafe {
                    let _ = InvalidateRect(Some(inner.hwnd), None, false);
                }
            }
        }

        pub fn handle_key(&self, vk: u32, ch: Option<char>, shift: bool) -> PickerKeyAction {
            let Ok(mut inner) = self.inner.try_borrow_mut() else {
                return PickerKeyAction::Close;
            };
            let action = inner.state.on_key(vk, ch, shift);
            let hwnd = inner.hwnd;
            match &action {
                PickerKeyAction::Close | PickerKeyAction::Commit(_) => {
                    if !hwnd.is_invalid() {
                        unsafe {
                            let _ = ShowWindow(hwnd, SW_HIDE);
                        }
                    }
                }
                PickerKeyAction::Redraw | PickerKeyAction::Reload => {
                    if !hwnd.is_invalid() {
                        unsafe {
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                    }
                }
            }
            action
        }
    }

    impl Drop for EmojiPicker {
        fn drop(&mut self) {
            if let Ok(mut inner) = self.inner.try_borrow_mut() {
                if !inner.hwnd.is_invalid() {
                    let hwnd = inner.hwnd;
                    inner.hwnd = HWND::default();
                    unsafe {
                        let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                        let _ = DestroyWindow(hwnd);
                    }
                }
            }
        }
    }

    fn clamp_to_monitor(ax: i32, ay_top: i32, ay_bot: i32, w: i32, h: i32) -> (i32, i32) {
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = unsafe {
            let hmon = MonitorFromPoint(POINT { x: ax, y: ay_bot }, MONITOR_DEFAULTTONEAREST);
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
        let mut x = ax;
        if x + w > work.right {
            x = work.right - w - 6;
        }
        if x < work.left {
            x = work.left + 6;
        }
        let y = if ay_bot + 6 + h <= work.bottom {
            ay_bot + 6
        } else if ay_top - 6 - h >= work.top {
            ay_top - 6 - h
        } else {
            (work.bottom - h - 6).max(work.top + 6)
        };
        (x, y)
    }

    fn create_popup_window(user_data: isize) -> Option<HWND> {
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
                Some(user_data as *const std::ffi::c_void),
            )
            .ok()?;
            let pref = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                (&pref as *const _) as *const std::ffi::c_void,
                std::mem::size_of_val(&pref) as u32,
            );
            Some(hwnd)
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                if !cs.is_null() {
                    let ptr = (*cs).lpCreateParams as isize;
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_GETOBJECT => LRESULT(0),
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Inner>;
                if !ptr.is_null() {
                    if let Ok(inner) = (*ptr).try_borrow() {
                        paint_popup(hwnd, hdc, &inner);
                    }
                }
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Inner>;
                if !ptr.is_null() {
                    let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                    let lx = ((lparam.0 as i16) as i32 * 96) / dpi;
                    let ly = (((lparam.0 >> 16) as i16) as i32 * 96) / dpi;
                    if let Ok(mut inner) = (*ptr).try_borrow_mut() {
                        let next = match inner.state.hit_test(lx, ly) {
                            PickerHit::Cell(idx) => Some(idx),
                            _ => None,
                        };
                        if inner.state.hovered != next {
                            inner.state.hovered = next;
                            let _ = InvalidateRect(Some(hwnd), None, false);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Inner>;
                if !ptr.is_null() {
                    let delta = ((wparam.0 >> 16) as i16) as i32;
                    let step = if delta > 0 { -2 } else { 2 };
                    if let Ok(mut inner) = (*ptr).try_borrow_mut() {
                        inner.state.scroll_by(step);
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Inner>;
                if !ptr.is_null() {
                    let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                    let lx = ((lparam.0 as i16) as i32 * 96) / dpi;
                    let ly = (((lparam.0 >> 16) as i16) as i32 * 96) / dpi;
                    let mut event: Option<PickerClickEvent> = None;
                    if let Ok(mut inner) = (*ptr).try_borrow_mut() {
                        match inner.state.hit_test(lx, ly) {
                            PickerHit::Tab(idx) => {
                                let cat = CATEGORY_TABS[idx].0;
                                inner.state.switch_category(cat);
                                event = Some(PickerClickEvent::SwitchTab(cat));
                            }
                            PickerHit::Cell(idx) => {
                                if let Some(item) = inner.state.items.get(idx) {
                                    let glyph = item.glyph.clone();
                                    inner.state.close();
                                    let _ = ShowWindow(hwnd, SW_HIDE);
                                    event = Some(PickerClickEvent::CommitGlyph(glyph));
                                }
                            }
                            PickerHit::None => {}
                        }
                    }
                    if let Some(ev) = event {
                        // Clone Rc và thả Ref TRƯỚC khi gọi callback để callback có thể
                        // borrow_mut (vd. SwitchTab -> set_items_and_redraw) không bị RefCell conflict.
                        let cb = (*ptr)
                            .try_borrow()
                            .ok()
                            .and_then(|inner| inner.on_click.clone());
                        if let Some(cb) = cb {
                            cb(ev);
                        }
                    }
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

    fn to_utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    unsafe fn paint_popup(hwnd: HWND, hdc: windows::Win32::Graphics::Gdi::HDC, inner: &Inner) {
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let w_px = (POPUP_WIDTH * dpi) / 96;
        let h_px = (POPUP_HEIGHT * dpi) / 96;
        let rc = RECT {
            left: 0,
            top: 0,
            right: w_px,
            bottom: h_px,
        };
        if let Some(d2d) = inner.d2d.as_ref() {
            if paint_d2d(d2d, hdc, &rc, dpi as f32, &inner.state).is_ok() {
                return;
            }
        }
        paint_gdi_fallback(hdc, &rc, dpi, &inner.state);
    }

    unsafe fn paint_d2d(
        d2d: &D2dResources,
        hdc: windows::Win32::Graphics::Gdi::HDC,
        rc: &RECT,
        dpi: f32,
        state: &EmojiPickerState,
    ) -> windows::core::Result<()> {
        d2d.rt.BindDC(hdc, rc)?;
        d2d.rt.SetDpi(dpi, dpi);
        d2d.rt.BeginDraw();

        let bg = rgba(30, 32, 38, 1.0);
        d2d.rt.Clear(Some(&bg));

        let border_brush = d2d.rt.CreateSolidColorBrush(&rgba(62, 66, 78, 1.0), None)?;
        let surface_brush = d2d.rt.CreateSolidColorBrush(&rgba(43, 46, 56, 1.0), None)?;
        let accent_brush = d2d
            .rt
            .CreateSolidColorBrush(&rgba(56, 132, 244, 0.35), None)?;
        let accent_line = d2d
            .rt
            .CreateSolidColorBrush(&rgba(88, 166, 255, 1.0), None)?;
        let hover_brush = d2d
            .rt
            .CreateSolidColorBrush(&rgba(255, 255, 255, 0.10), None)?;
        let text_brush = d2d
            .rt
            .CreateSolidColorBrush(&rgba(238, 241, 246, 1.0), None)?;
        let muted_brush = d2d
            .rt
            .CreateSolidColorBrush(&rgba(154, 162, 178, 1.0), None)?;

        // Viền ngoài mảnh.
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
            &border_brush,
            1.0,
            None,
        );

        // Thanh tìm kiếm.
        let search_rect = D2D_RECT_F {
            left: PAD_X as f32,
            top: 8.0,
            right: (POPUP_WIDTH - PAD_X) as f32,
            bottom: 34.0,
        };
        d2d.rt.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: search_rect,
                radiusX: 6.0,
                radiusY: 6.0,
            },
            &surface_brush,
        );
        let search_text = if state.query.is_empty() {
            "🔍 Gõ tìm emoji (cười, tim, fire, ok...)".to_string()
        } else {
            format!("🔍 {}", state.query)
        };
        let search_u16 = to_utf16(&search_text);
        d2d.rt.DrawText(
            &search_u16,
            &d2d.ui_fmt,
            &D2D_RECT_F {
                left: search_rect.left + 8.0,
                top: search_rect.top,
                right: search_rect.right - 8.0,
                bottom: search_rect.bottom,
            },
            if state.query.is_empty() {
                &muted_brush
            } else {
                &text_brush
            },
            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
        );

        // Dải 8 tab danh mục.
        let tab_w = (POPUP_WIDTH - 2 * PAD_X) as f32 / CATEGORY_TABS.len() as f32;
        for (i, &(cat_id, icon, _)) in CATEGORY_TABS.iter().enumerate() {
            let x0 = PAD_X as f32 + i as f32 * tab_w;
            let tr = D2D_RECT_F {
                left: x0,
                top: HEADER_HEIGHT as f32,
                right: x0 + tab_w,
                bottom: (HEADER_HEIGHT + TABS_HEIGHT) as f32,
            };
            if state.query.is_empty() && state.category == cat_id {
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
                    &accent_brush,
                );
                d2d.rt.FillRectangle(
                    &D2D_RECT_F {
                        left: tr.left + 8.0,
                        top: tr.bottom - 3.0,
                        right: tr.right - 8.0,
                        bottom: tr.bottom - 1.0,
                    },
                    &accent_line,
                );
            }
            let icon_u16 = to_utf16(icon);
            d2d.rt.DrawText(
                &icon_u16,
                &d2d.tab_fmt,
                &tr,
                &text_brush,
                D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
            );
        }

        // Lưới 8×5 emoji màu thật.
        let start_idx = state.scroll_row * GRID_COLS;
        for vis_row in 0..GRID_VISIBLE_ROWS {
            for col in 0..GRID_COLS {
                let idx = start_idx + vis_row * GRID_COLS + col;
                let Some(item) = state.items.get(idx) else {
                    break;
                };
                let x0 = (PAD_X + (col as i32) * CELL_W) as f32;
                let y0 = (GRID_TOP + (vis_row as i32) * CELL_H) as f32;
                let cell = D2D_RECT_F {
                    left: x0 + 2.0,
                    top: y0 + 2.0,
                    right: x0 + CELL_W as f32 - 2.0,
                    bottom: y0 + CELL_H as f32 - 2.0,
                };
                if idx == state.selected {
                    d2d.rt.FillRoundedRectangle(
                        &D2D1_ROUNDED_RECT {
                            rect: cell,
                            radiusX: 6.0,
                            radiusY: 6.0,
                        },
                        &accent_brush,
                    );
                } else if state.hovered == Some(idx) {
                    d2d.rt.FillRoundedRectangle(
                        &D2D1_ROUNDED_RECT {
                            rect: cell,
                            radiusX: 6.0,
                            radiusY: 6.0,
                        },
                        &hover_brush,
                    );
                }
                let glyph_u16 = to_utf16(&item.glyph);
                d2d.rt.DrawText(
                    &glyph_u16,
                    &d2d.emoji_fmt,
                    &cell,
                    &text_brush,
                    D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }

        // Thanh chân: shortcode & phím tắt.
        let footer_y = (POPUP_HEIGHT - FOOTER_HEIGHT) as f32;
        d2d.rt.FillRectangle(
            &D2D_RECT_F {
                left: PAD_X as f32,
                top: footer_y,
                right: (POPUP_WIDTH - PAD_X) as f32,
                bottom: footer_y + 1.0,
            },
            &border_brush,
        );
        let active = state
            .hovered
            .and_then(|i| state.items.get(i))
            .or_else(|| state.selected_item());
        let footer_text = match active {
            Some(it) => format!(
                ":{}:  ·  Enter chèn · Tab đổi nhóm · Esc đóng",
                it.shortcode
            ),
            None => "Không tìm thấy emoji  ·  Esc đóng".to_string(),
        };
        let footer_u16 = to_utf16(&footer_text);
        d2d.rt.DrawText(
            &footer_u16,
            &d2d.small_fmt,
            &D2D_RECT_F {
                left: PAD_X as f32,
                top: footer_y + 2.0,
                right: (POPUP_WIDTH - PAD_X) as f32,
                bottom: POPUP_HEIGHT as f32 - 2.0,
            },
            &muted_brush,
            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
        );

        d2d.rt.EndDraw(None, None)
    }

    unsafe fn paint_gdi_fallback(
        hdc: windows::Win32::Graphics::Gdi::HDC,
        rc: &RECT,
        dpi: i32,
        state: &EmojiPickerState,
    ) {
        let bg = CreateSolidBrush(COLORREF(0x0026_201E));
        FillRect(hdc, rc, bg);
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
        let start_idx = state.scroll_row * GRID_COLS;
        for vis_row in 0..GRID_VISIBLE_ROWS {
            for col in 0..GRID_COLS {
                let idx = start_idx + vis_row * GRID_COLS + col;
                let Some(item) = state.items.get(idx) else {
                    break;
                };
                let mut cell = RECT {
                    left: ((PAD_X + (col as i32) * CELL_W) * dpi) / 96,
                    top: ((GRID_TOP + (vis_row as i32) * CELL_H) * dpi) / 96,
                    right: ((PAD_X + ((col + 1) as i32) * CELL_W) * dpi) / 96,
                    bottom: ((GRID_TOP + ((vis_row + 1) as i32) * CELL_H) * dpi) / 96,
                };
                let mut u16s = to_utf16(&item.glyph);
                let _ = DrawTextW(
                    hdc,
                    &mut u16s,
                    &mut cell,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }
        }
        let mut hdr = RECT {
            left: (PAD_X * dpi) / 96,
            top: (8 * dpi) / 96,
            right: ((POPUP_WIDTH - PAD_X) * dpi) / 96,
            bottom: (34 * dpi) / 96,
        };
        let mut q = to_utf16(if state.query.is_empty() {
            "Emoji (Esc đóng)"
        } else {
            &state.query
        });
        let _ = DrawTextW(hdc, &mut q, &mut hdr, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font.into());
    }

    pub use PickerClickEvent as ClickEvent;
}

#[cfg(windows)]
pub use win::{ClickEvent, EmojiPicker};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items(n: usize) -> Vec<EmojiItem> {
        (0..n)
            .map(|i| EmojiItem {
                glyph: format!("😀{i}"),
                shortcode: format!("e{i}"),
                category: 1,
            })
            .collect()
    }

    #[test]
    fn parse_tsv_handles_valid_and_malformed_lines() {
        let items = parse_emoji_tsv("🔥\tfire\t3\n🚀\trocket\t7\nbadline\n\t\t\n");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].glyph, "🔥");
        assert_eq!(items[0].shortcode, "fire");
        assert_eq!(items[0].category, 3);
        assert_eq!(items[1].glyph, "🚀");
    }

    #[test]
    fn keyboard_navigation_scrolling_search_and_commit() {
        let mut st = EmojiPickerState::default();
        st.open();
        st.set_items(sample_items(50));
        assert!(st.visible);
        assert_eq!(st.selected, 0);

        // Đi xuống 5 hàng (selected = 40) → scroll_row tự dịch xuống 1.
        for _ in 0..5 {
            assert_eq!(st.on_key(0x28, None, false), PickerKeyAction::Redraw);
        }
        assert_eq!(st.selected, 40);
        assert_eq!(st.scroll_row, 1);

        // Sang phải 1 ô, rồi Enter → Commit đúng mục 41 và đóng.
        assert_eq!(st.on_key(0x27, None, false), PickerKeyAction::Redraw);
        assert_eq!(
            st.on_key(0x0D, None, false),
            PickerKeyAction::Commit("😀41".to_string())
        );
        assert!(!st.visible);

        // Mở lại, gõ tìm kiếm 'f','i', Backspace, Tab đổi nhóm, Esc đóng.
        st.open();
        st.set_items(sample_items(10));
        assert_eq!(st.on_key(0x46, Some('f'), false), PickerKeyAction::Reload);
        assert_eq!(st.query, "f");
        assert_eq!(st.on_key(0x08, None, false), PickerKeyAction::Reload);
        assert_eq!(st.query, "");
        assert_eq!(st.on_key(0x09, None, false), PickerKeyAction::Reload);
        assert_eq!(st.category, 1);
        assert_eq!(st.on_key(0x1B, None, false), PickerKeyAction::Close);
        assert!(!st.visible);
    }

    #[test]
    fn hit_test_tabs_and_grid_cells() {
        let mut st = EmojiPickerState::default();
        st.open();
        st.set_items(sample_items(20));
        assert_eq!(
            st.hit_test(PAD_X + 5, HEADER_HEIGHT + 10),
            PickerHit::Tab(0)
        );
        assert_eq!(st.hit_test(PAD_X + 5, GRID_TOP + 5), PickerHit::Cell(0));
        assert_eq!(
            st.hit_test(PAD_X + CELL_W + 5, GRID_TOP + CELL_H + 5),
            PickerHit::Cell(GRID_COLS + 1)
        );
        assert_eq!(st.hit_test(0, 0), PickerHit::None);
    }
}
