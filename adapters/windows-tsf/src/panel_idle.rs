// SPDX-License-Identifier: GPL-3.0-or-later
//! P9 (vòng 55 — phản hồi chủ repo): panel trợ lý chỉ hiện khi người dùng
//! **dừng gõ ~1 giây**. Trước đây panel hiện NGAY tại phím trigger và đứng
//! đó trong lúc user gõ tiếp ⇒ che mất vùng đang gõ (yêu cầu "panel che mất
//! vùng gõ" + "popup hiện liên tiếp").
//!
//! Cơ chế (giống `selection_check` debounce đã kiểm chứng): phím trigger lên
//! lịch timer 1000 ms; **mọi phím gõ mới re-schedule** (reset đếm) và popup
//! đang sống bị ẨN NGAY khi gõ tiếp; timer fire → check câu bằng
//! `ime_assist_check_v1` rồi `show_assist_overlay_with_recheck`. Auto-fix
//! (tự sửa khi bật) KHÔNG qua debounce — hành động tự động chạy như cũ.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

use crate::edit_session::TsfShared;

/// Thời gian dừng gõ trước khi panel hiện (yêu cầu chủ repo: ~1 giây).
pub const PANEL_IDLE_MS: u32 = 1_000;
/// Timer ID — overlay WndProc so khớp (không hardcode 2 chỗ).
pub(crate) const TIMER_PANEL_IDLE: usize = 0x5EC3;

thread_local! {
    static PENDING: RefCell<Option<(Weak<TsfShared>, windows::Win32::UI::TextServices::ITfContext)>> =
        const { RefCell::new(None) };
}

thread_local! {
    static TIMER_HWND: RefCell<isize> = const { RefCell::new(0) };
}

/// Phím nào (re)lên lịch timer idle: MỌI phím gõ chữ/dấu câu/xoá — kể cả khi đang composing —
/// khi tiếng Việt bật. Enter/Tab/Esc/Ctrl+Space/điều hướng KHÔNG đổi câu nên không đếm lại.
/// Hàm thuần: bộ test `idle_popup_routing` khoá bảng này (R2-154).
pub fn idle_arms(key: crate::compose::KeyKind, vi_enabled: bool) -> bool {
    use crate::compose::KeyKind;
    vi_enabled && matches!(key, KeyKind::Char(_) | KeyKind::Backspace)
}

/// Ghi hwnd dùng cho timer (gọi khi overlay tạo hwnd — như selection_check).
#[cfg(windows)]
pub fn register_timer_hwnd(hwnd: isize) {
    TIMER_HWND.with(|c| *c.borrow_mut() = hwnd);
}

fn hwnd_alive() -> isize {
    let hwnd = TIMER_HWND.with(|c| *c.borrow());
    let alive = hwnd != 0
        && unsafe {
            windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(
                windows::Win32::Foundation::HWND(hwnd as *mut std::ffi::c_void),
            ))
            .as_bool()
        };
    if alive {
        hwnd
    } else {
        0
    }
}

/// Lên lịch (re-arm) panel idle 1 s. Gọi lại ở MỌI phím gõ để đếm lại.
#[cfg(windows)]
pub fn schedule(shared: &Rc<TsfShared>, ctx: &windows::Win32::UI::TextServices::ITfContext) {
    if !shared.is_assist_display_enabled() {
        return;
    }
    let mut hwnd = hwnd_alive();
    if hwnd == 0 {
        hwnd = shared.overlay.ensure_timer_hwnd();
        if hwnd == 0 {
            return;
        }
        register_timer_hwnd(hwnd);
    }
    PENDING.with(|p| *p.borrow_mut() = Some((Rc::downgrade(shared), ctx.clone())));
    // SAFETY: hwnd thuộc thread này.
    unsafe {
        let _ = SetTimer(
            Some(windows::Win32::Foundation::HWND(
                hwnd as *mut std::ffi::c_void,
            )),
            TIMER_PANEL_IDLE,
            PANEL_IDLE_MS,
            None,
        );
    }
}

/// Huỷ pending + timer (phím làm rỗng câu / focus đổi / composition chấm dứt).
#[cfg(windows)]
pub fn cancel() {
    let hwnd = TIMER_HWND.with(|c| *c.borrow());
    if hwnd != 0 {
        unsafe {
            let _ = KillTimer(
                Some(windows::Win32::Foundation::HWND(
                    hwnd as *mut std::ffi::c_void,
                )),
                TIMER_PANEL_IDLE,
            );
        }
    }
    PENDING.with(|p| *p.borrow_mut() = None);
}

/// Timer fire → check câu + show panel (gọi từ `overlay_wnd_proc` WM_TIMER).
#[cfg(windows)]
pub fn on_timer() {
    let hwnd = TIMER_HWND.with(|c| *c.borrow());
    if hwnd != 0 {
        unsafe {
            let _ = KillTimer(
                Some(windows::Win32::Foundation::HWND(
                    hwnd as *mut std::ffi::c_void,
                )),
                TIMER_PANEL_IDLE,
            );
        }
    }
    let Some((weak, ctx)) = PENDING.with(|p| p.borrow_mut().take()) else {
        return;
    };
    let Some(shared) = weak.upgrade() else {
        return;
    };
    // P9: on-demand check tái dùng đường đã kiểm chứng (AS-030): check câu hiện hành →
    // show overlay nếu còn lỗi; không lỗi → im lặng. Timer không có edit cookie ⇒ phải xin
    // session (`ec = 0` làm `GetSelection` lỗi và popup không bao giờ hiện — R2-154).
    let session: windows::Win32::UI::TextServices::ITfEditSession =
        crate::edit_session::PanelIdleSession::new(shared.clone(), ctx.clone()).into();
    let requested = crate::edit_session::request_sync_or_async(&ctx, shared.tid, &session);
    crate::trace::event(
        shared.tid,
        format_args!("panel idle: session requested={requested}"),
    );
}
