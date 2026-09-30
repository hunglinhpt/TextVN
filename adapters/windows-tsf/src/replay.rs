// SPDX-License-Identifier: GPL-3.0-or-later
//! Trả lại phím ranh giới cho app IMM32 chạy qua CUAS (Notepad cổ điển, WinForms, Delphi…).
//!
//! Với CUAS, phím mà TIP nhận ở pha test (`OnTestKeyDown` = TRUE) bị đổi thành
//! `VK_PROCESSKEY`: app không bao giờ thấy phím gốc, kể cả khi `OnKeyDown` sau đó trả FALSE
//! (test gõ thật trên Windows: `chaof⏎banj` → `chàoBạn`, mất Enter). Còn nếu để phím đi
//! thẳng ở pha test thì kết quả composition tới SAU phím (`⏎chào`). Nên với Enter, Tab, phím
//! điều hướng, Ctrl+… khi đang composing: TIP nhận phím, commit từ, rồi trả phím gốc về đúng
//! cửa sổ đang focus bằng `PostMessageW(WM_KEYDOWN)`.
//!
//! Thứ tự: thông điệp đi vòng qua một cửa sổ message-only của TIP vài lượt trước khi tới
//! app, để phím nằm SAU các thông điệp kết quả composition mà CUAS đưa vào hàng đợi. Thông
//! điệp posted luôn được lấy trước input phần cứng, nên phím người dùng gõ tiếp không chen
//! vào giữa.
//!
//! An toàn: chỉ post trong CHÍNH process và thread UI của app, tới cửa sổ app đang focus —
//! không `SendInput`, không hook, không đụng process khác.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, IsWindow, PostMessageW, RegisterClassW,
    HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_KEYDOWN, WNDCLASSW,
};

const CLASS_NAME: PCWSTR = w!("TextVN.TsfKeyReplay");
const WM_REPLAY: u32 = WM_APP + 0x54;
/// Số lượt đi vòng trước khi post phím cho app.
const HOPS: usize = 3;

struct Pending {
    target: isize,
    vk: usize,
    lparam: isize,
}

/// Phím đã được CUAS giữ ở `OnTestKeyDown`. Capture target/lParam ngay trong
/// callback test để không gửi phím tới control mới nếu commit làm app đổi focus.
#[derive(Clone, Copy)]
pub struct DeferredReplay {
    target: isize,
    vk: u32,
    lparam: isize,
}

impl DeferredReplay {
    pub fn matches_vk(self, vk: u32) -> bool {
        self.vk == vk
    }
}

thread_local! {
    static WINDOW: Cell<isize> = const { Cell::new(0) };
    static QUEUE: RefCell<VecDeque<Pending>> = const { RefCell::new(VecDeque::new()) };
}

/// Dành trước đích replay trước khi `OnTestKeyDown` trả TRUE. Nếu không tạo được
/// message window hoặc không có focus thì caller phải để phím đi thẳng; sau TRUE
/// CUAS đã thay nó bằng VK_PROCESSKEY và không thể "fail open" nữa.
pub fn prepare(vk: u32, lparam: LPARAM) -> Option<DeferredReplay> {
    // SAFETY: gọi trên thread UI của app (callback key sink).
    let target = unsafe { GetFocus() };
    if target.is_invalid() {
        return None;
    }
    window()?;
    Some(DeferredReplay {
        target: target.0 as isize,
        vk,
        lparam: lparam.0,
    })
}

/// Hẹn trả phím đã preflight. Khi message-only window bất ngờ không nhận post,
/// thử post trực tiếp tới đúng HWND đã capture. Kết quả `false` chỉ còn khi đích
/// đã đóng; caller vẫn trả TRUE vì CUAS đã nuốt phím gốc.
pub fn schedule(event: DeferredReplay) -> bool {
    let target = HWND(event.target as *mut _);
    // SAFETY: HWND đã capture từ GetFocus trên cùng UI thread. IsWindow bảo vệ
    // trường hợp control đã bị phá hủy re-entrant giữa hai callback.
    if unsafe { !IsWindow(Some(target)).as_bool() } {
        return false;
    }
    let Some(wnd) = window() else {
        return unsafe {
            PostMessageW(
                Some(target),
                WM_KEYDOWN,
                WPARAM(event.vk as usize),
                LPARAM(event.lparam),
            )
            .is_ok()
        };
    };
    QUEUE.with(|q| {
        q.borrow_mut().push_back(Pending {
            target: event.target,
            vk: event.vk as usize,
            lparam: event.lparam,
        })
    });
    // SAFETY: cửa sổ message-only của chính thread này.
    if unsafe { PostMessageW(Some(wnd), WM_REPLAY, WPARAM(HOPS), LPARAM(0)) }.is_err() {
        QUEUE.with(|q| q.borrow_mut().pop_back());
        return unsafe {
            PostMessageW(
                Some(target),
                WM_KEYDOWN,
                WPARAM(event.vk as usize),
                LPARAM(event.lparam),
            )
            .is_ok()
        };
    }
    true
}

/// Huỷ cửa sổ của thread (TIP Deactivate). Phím còn chờ được trả luôn cho app.
pub fn shutdown() {
    while let Some(p) = QUEUE.with(|q| q.borrow_mut().pop_front()) {
        deliver(&p);
    }
    let wnd = WINDOW.with(|w| w.replace(0));
    if wnd != 0 {
        // SAFETY: cửa sổ do thread này tạo.
        let _ = unsafe { DestroyWindow(HWND(wnd as *mut _)) };
    }
}

fn deliver(p: &Pending) {
    // SAFETY: post tới cửa sổ của app trong cùng process; lỗi (cửa sổ đã đóng) bỏ qua.
    let _ = unsafe {
        PostMessageW(
            Some(HWND(p.target as *mut _)),
            WM_KEYDOWN,
            WPARAM(p.vk),
            LPARAM(p.lparam),
        )
    };
}

fn window() -> Option<HWND> {
    let existing = WINDOW.with(|w| w.get());
    if existing != 0 {
        return Some(HWND(existing as *mut _));
    }
    // SAFETY: đăng ký class (lần đầu trong process; lần sau trả lỗi "đã tồn tại", bỏ qua)
    // và tạo cửa sổ message-only trên thread hiện tại, với HINSTANCE của chính DLL này.
    unsafe {
        let hinst = module_instance();
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst,
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };
        let _ = RegisterClassW(&class);
        let wnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            PCWSTR::null(),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinst),
            None,
        )
        .ok()?;
        WINDOW.with(|w| w.set(wnd.0 as isize));
        Some(wnd)
    }
}

unsafe fn module_instance() -> HINSTANCE {
    let mut module = Default::default();
    let _ = GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        PCWSTR(wndproc as *const u16),
        &mut module,
    );
    module.into()
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg != WM_REPLAY {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let hops = wparam.0;
    if hops > 1 && PostMessageW(Some(hwnd), WM_REPLAY, WPARAM(hops - 1), lparam).is_ok() {
        return LRESULT(0);
    }
    if let Some(p) = QUEUE.with(|q| q.borrow_mut().pop_front()) {
        deliver(&p);
    }
    LRESULT(0)
}
