// SPDX-License-Identifier: GPL-3.0-or-later
//! Crate `textvn-tray` — Ứng dụng khay hệ thống, IPC server và điều phối runtime cho TextVN (Windows).
//!
//! Bao gồm các module chức năng:
//! - [`svc`]: Service layer in-process quản lý trạng thái (`state.json`) và cấu hình (`config.json`).
//! - [`ipc_server`]: Named Pipe server `\\.\pipe\textvn-ipc-v1`, broadcast cấu hình và giám sát hook.
//! - [`menu`]: Menu ngữ cảnh khay hệ thống (9 mục chuẩn Win32).
//! - [`autostart`]: Quản lý registry key tự khởi động `HKCU\...\Run\TextVN` (per-user).

// Tray = Windows-only (P1-4): trên non-Windows chỉ build để gate CI --workspace,
// các item Win32 không có caller là bình thường — không phải dead code thật.
#![cfg_attr(not(windows), allow(dead_code))]

pub mod autostart;
pub mod foreground;
pub mod hotkey;
pub mod icons;
pub mod ipc_server;
pub mod menu;
pub mod settings;
pub mod settings_dialog;
pub mod svc;

pub use autostart::{disable_autostart, enable_autostart, is_autostart_enabled};
pub use ipc_server::IpcServer;
pub use menu::TrayMenu;
pub use settings::{SettingsController, SettingsTab};
pub use settings_dialog::show_settings_dialog;
pub use svc::{StateData, SvcManager};

use std::sync::atomic::AtomicIsize;
#[cfg(windows)]
use std::sync::atomic::Ordering;

pub static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
pub const WM_UPDATE_TRAY_STATE: u32 = 0x8000 + 2;
pub const WM_OPEN_SETTINGS: u32 = 0x8000 + 3;
pub const WM_REQUEST_EXIT: u32 = 0x8000 + 4;
/// Yêu cầu khởi động engine hook tương thích. Đây là hành động chủ động theo
/// phiên; bản TextVN chuẩn không tự chạy global keyboard hook.
pub const WM_START_COMPATIBILITY_HOOK: u32 = 0x8000 + 5;

/// Vị trí duy nhất được chấp nhận cho compatibility hook: cạnh `TextVN.exe`.
/// Không tìm trong working directory hay `target/` để bản phát hành không thể
/// vô tình khởi chạy một binary cùng tên nhưng không thuộc gói đang chạy.
pub fn compatibility_hook_path() -> Option<std::path::PathBuf> {
    let mut exe = std::env::current_exe().ok()?;
    exe.pop();
    let hook = exe.join("textvn-hook.exe");
    hook.is_file().then_some(hook)
}

/// Gửi thông điệp cập nhật icon và tooltip cho Tray Window (thread-safe).
pub fn notify_tray_state_changed() {
    #[cfg(windows)]
    {
        let raw = TRAY_HWND.load(Ordering::Acquire);
        if raw != 0 {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
            let hwnd = HWND(raw as *mut _);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_UPDATE_TRAY_STATE, WPARAM(0), LPARAM(0));
            }
        }
    }
}

/// Yêu cầu mở Bảng điều khiển từ thread khác hoặc khi re-launch.
pub fn notify_open_settings() {
    #[cfg(windows)]
    {
        let raw = TRAY_HWND.load(Ordering::Acquire);
        if raw != 0 {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
            let hwnd = HWND(raw as *mut _);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_OPEN_SETTINGS, WPARAM(0), LPARAM(0));
            }
        }
    }
}

/// Request orderly tray shutdown from an IPC worker thread.
pub fn notify_tray_exit_requested() {
    #[cfg(windows)]
    {
        let raw = TRAY_HWND.load(Ordering::Acquire);
        if raw != 0 {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
            let hwnd = HWND(raw as *mut _);
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_REQUEST_EXIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}

/// Yêu cầu Tray khởi động Hook tương thích sau khi người dùng chọn rõ ràng từ
/// menu. Giữ việc tạo tiến trình trong UI thread để menu không có quyền spawn
/// một tiến trình global-hook trực tiếp.
pub fn notify_start_compatibility_hook() {
    #[cfg(windows)]
    {
        let raw = TRAY_HWND.load(Ordering::Acquire);
        if raw != 0 {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
            let hwnd = HWND(raw as *mut _);
            unsafe {
                let _ = PostMessageW(
                    Some(hwnd),
                    WM_START_COMPATIBILITY_HOOK,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        }
    }
}
