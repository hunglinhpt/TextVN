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
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetFocus, GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, IsWindow, KillTimer, PostMessageW,
    RegisterClassW, SetTimer, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_KEYDOWN,
    WM_TIMER, WNDCLASSW,
};

const CLASS_NAME: PCWSTR = w!("TextVN.TsfKeyReplay");
const WM_REPLAY: u32 = WM_APP + 0x54;
/// Số lượt đi vòng trước khi post phím cho app.
const HOPS: usize = 3;
/// R2-132: timer chờ người dùng nhả Ctrl/Alt/Shift/Win trước khi post Backspace
/// sửa câu (Scintilla: Ctrl+Backspace = xoá cả từ, Alt+Backspace = Undo — đo
/// thật: Alt+1/Ctrl+Space trên Notepad++ xoá thừa một từ). `WM_TIMER` chỉ tới
/// khi hàng đợi trống ⇒ phím nhả (input) đã được app xử lý trước.
const TIMER_MODIFIERS: usize = 0x5452;
const MODIFIER_POLL_MS: u32 = 25;
/// Tối đa ~2 s chờ nhả phím bổ trợ; quá thì bỏ việc (không sửa — fail-safe).
const MODIFIER_POLL_MAX: u32 = 80;

/// Việc chờ giao sau `HOPS` lượt: trả phím gốc (CUAS) hoặc sửa đuôi câu
/// (R2-132 — `cuas.rs`, xác minh document feed lúc giao).
enum Pending {
    Key {
        target: isize,
        vk: usize,
        lparam: isize,
    },
    TailFix(crate::cuas::TailFix),
    /// R2-133: Alt+Space rảnh ở cửa sổ khung tự vẽ (không `WS_SYSMENU`) — mở
    /// menu hệ thống ngoài callback TSF (`TrackPopupMenu` chạy vòng modal).
    SystemMenu {
        root: isize,
    },
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
    /// Việc sửa câu đang chờ nhả phím bổ trợ + số lần đã thử.
    static WAITING: RefCell<Option<(crate::cuas::TailFix, u32)>> = const { RefCell::new(None) };
}

/// Phím bổ trợ đang giữ theo trạng thái hàng đợi của thread (GetKeyState).
fn modifiers_down() -> bool {
    [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
        .iter()
        .any(|vk| {
            // SAFETY: GetKeyState chỉ đọc trạng thái phím của thread hiện tại.
            (unsafe { GetKeyState(i32::from(vk.0)) } as u16 & 0x8000) != 0
        })
}

/// Giao việc sửa câu khi không còn phím bổ trợ; còn giữ thì hẹn timer.
fn run_or_wait(hwnd: HWND, job: crate::cuas::TailFix, tries: u32) {
    if !modifiers_down() {
        crate::cuas::run(&job);
        return;
    }
    if tries >= MODIFIER_POLL_MAX {
        crate::cuas::give_up(&job, "modifiers held");
        return;
    }
    WAITING.with(|w| *w.borrow_mut() = Some((job, tries + 1)));
    // SAFETY: timer trên cửa sổ message-only của chính thread này.
    unsafe { SetTimer(Some(hwnd), TIMER_MODIFIERS, MODIFIER_POLL_MS, None) };
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
        q.borrow_mut().push_back(Pending::Key {
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

/// R2-132: hẹn sửa đuôi câu ở app CUAS — đi vòng `HOPS` lượt như phím replay để
/// app đã nhận xong kết quả composition trước khi đọc document feed.
pub fn schedule_tail_fix(job: crate::cuas::TailFix) -> bool {
    let Some(wnd) = window() else {
        return false;
    };
    QUEUE.with(|q| q.borrow_mut().push_back(Pending::TailFix(job)));
    // SAFETY: cửa sổ message-only của chính thread này.
    if unsafe { PostMessageW(Some(wnd), WM_REPLAY, WPARAM(HOPS), LPARAM(0)) }.is_err() {
        QUEUE.with(|q| q.borrow_mut().pop_back());
        return false;
    }
    true
}

/// R2-133: hẹn mở menu hệ thống của cửa sổ gốc chứa `focus` (Alt+Space TextVN
/// không dùng, cửa sổ khung tự vẽ bỏ qua `SC_KEYMENU`). `false` = không hẹn được.
pub fn schedule_system_menu(focus: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GA_ROOT};
    // SAFETY: HWND của thread hiện tại.
    let root = unsafe { GetAncestor(focus, GA_ROOT) };
    if root.is_invalid() {
        return false;
    }
    let Some(wnd) = window() else {
        return false;
    };
    QUEUE.with(|q| {
        q.borrow_mut().push_back(Pending::SystemMenu {
            root: root.0 as isize,
        })
    });
    // SAFETY: cửa sổ message-only của chính thread này.
    if unsafe { PostMessageW(Some(wnd), WM_REPLAY, WPARAM(1), LPARAM(0)) }.is_err() {
        QUEUE.with(|q| q.borrow_mut().pop_back());
        return false;
    }
    true
}

/// Mở menu hệ thống tại góc trên-trái cửa sổ (như Alt+Space gốc) và gửi lệnh đã
/// chọn về cửa sổ — cùng cách Chromium làm cho cửa sổ khung tự vẽ.
fn show_system_menu(root: isize) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMenu, GetWindowRect, TrackPopupMenu, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_TOPALIGN,
        WM_SYSCOMMAND,
    };
    let root = HWND(root as *mut _);
    // SAFETY: cửa sổ của chính thread UI này; menu hệ thống thuộc cửa sổ đó.
    unsafe {
        if !IsWindow(Some(root)).as_bool() {
            return;
        }
        let menu = GetSystemMenu(root, false);
        if menu.is_invalid() {
            return;
        }
        let mut rc = RECT::default();
        if GetWindowRect(root, &mut rc).is_err() {
            return;
        }
        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_LEFTALIGN | TPM_TOPALIGN,
            rc.left + 8,
            rc.top + 32,
            None,
            root,
            None,
        );
        if cmd.0 != 0 {
            let _ = PostMessageW(Some(root), WM_SYSCOMMAND, WPARAM(cmd.0 as usize), LPARAM(0));
        }
    }
}

/// Huỷ cửa sổ của thread (TIP Deactivate). Phím còn chờ được trả luôn cho app;
/// việc sửa câu còn chờ bị bỏ (TIP sắp đi — không post Backspace mù).
pub fn shutdown() {
    WAITING.with(|w| w.borrow_mut().take());
    while let Some(p) = QUEUE.with(|q| q.borrow_mut().pop_front()) {
        if let Pending::Key { .. } = p {
            deliver(&p);
        }
        // Menu hệ thống còn chờ: bỏ (TIP sắp đi — không mở vòng modal lúc huỷ).
    }
    let wnd = WINDOW.with(|w| w.replace(0));
    if wnd != 0 {
        // SAFETY: cửa sổ do thread này tạo.
        let _ = unsafe { DestroyWindow(HWND(wnd as *mut _)) };
    }
}

fn deliver(p: &Pending) {
    match p {
        Pending::Key { target, vk, lparam } => {
            // SAFETY: post tới cửa sổ của app trong cùng process; lỗi (cửa sổ đã đóng) bỏ qua.
            let _ = unsafe {
                PostMessageW(
                    Some(HWND(*target as *mut _)),
                    WM_KEYDOWN,
                    WPARAM(*vk),
                    LPARAM(*lparam),
                )
            };
        }
        // Việc sửa câu đi qua `run_or_wait` (chờ nhả phím bổ trợ), không qua đây.
        Pending::TailFix(_) => {}
        Pending::SystemMenu { root } => show_system_menu(*root),
    }
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
    if msg == WM_TIMER && wparam.0 == TIMER_MODIFIERS {
        let _ = KillTimer(Some(hwnd), TIMER_MODIFIERS);
        if let Some((job, tries)) = WAITING.with(|w| w.borrow_mut().take()) {
            run_or_wait(hwnd, job, tries);
        }
        return LRESULT(0);
    }
    if msg != WM_REPLAY {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let hops = wparam.0;
    if hops > 1 && PostMessageW(Some(hwnd), WM_REPLAY, WPARAM(hops - 1), lparam).is_ok() {
        return LRESULT(0);
    }
    match QUEUE.with(|q| q.borrow_mut().pop_front()) {
        Some(Pending::TailFix(job)) => run_or_wait(hwnd, job, 0),
        Some(p) => deliver(&p),
        None => {}
    }
    LRESULT(0)
}
