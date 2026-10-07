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
#[cfg(windows)]
pub mod package_bootstrap;
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
/// LL hook của tray phát hiện Ctrl+Shift tap toàn cục → tray toggle + đổi icon.
pub const WM_TOGGLE_HOTKEY: u32 = 0x8000 + 6;

/// Debounce chéo nguồn cho toggle Ctrl+Shift: một lần bấm tới tray qua HAI
/// đường (LL hook quan sát + TIP in-process gửi IPC) — nguồn đến sau trong
/// cửa sổ 250ms bị bỏ qua để không toggle đôi (E11). Menu/click chuột không
/// đi qua hàm này (hành động tường minh của người dùng).
pub static LAST_GLOBAL_TOGGLE_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Thử chiếm lượt toggle toàn cục: `true` khi cách lần trước ≥ 250ms.
pub fn try_claim_global_toggle() -> bool {
    claim_debounced(&LAST_GLOBAL_TOGGLE_MS, system_millis(), 250)
}

fn system_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Cốt lõi debounce (thuần, kiểm thử được với đồng hồ riêng — test không đụng
/// static toàn cục mà test khác chạy song song có thể claim vào).
fn claim_debounced(cell: &std::sync::atomic::AtomicU64, now: u64, window_ms: u64) -> bool {
    use std::sync::atomic::Ordering;
    let last = cell.load(Ordering::Acquire);
    if now.saturating_sub(last) < window_ms {
        return false;
    }
    // Thread khác vừa claim trong cùng thời điểm = trùng lần bấm → bỏ.
    cell.compare_exchange(last, now, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

/// `exe` nằm **dưới** một trong `roots` — so tiền tố theo ranh giới thư mục, không
/// phân biệt hoa thường, bỏ tiền tố `\\?\`. Không dùng "chứa chuỗi": với
/// `contains("\\program files\\")` thì `D:\x\Program Files\y\TextVN.exe` (thư mục
/// người dùng tự tạo, ghi được) cũng lọt qua gate nâng quyền SEC-02.
pub fn path_is_under_any(exe: &str, roots: &[String]) -> bool {
    fn norm(p: &str) -> String {
        let p = p.replace('/', "\\");
        let p = p.strip_prefix(r"\\?\").unwrap_or(&p);
        p.trim_end_matches('\\').to_lowercase()
    }
    let exe = norm(exe);
    roots
        .iter()
        .map(|r| norm(r))
        .filter(|r| !r.is_empty())
        .any(|root| {
            exe.len() > root.len() && exe.starts_with(&root) && exe.as_bytes()[root.len()] == b'\\'
        })
}

/// Thư mục cài đặt chuẩn (bộ cài máy, MSIX trong `Program Files\WindowsApps`, bộ cài
/// per-user/Store ở `%LOCALAPPDATA%\Programs`) — nơi duy nhất tray được xin UAC để
/// chạy CLI cạnh nó (SEC-02).
pub fn trusted_install_roots() -> Vec<String> {
    let mut roots: Vec<String> = ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|v| v.to_string_lossy().into_owned())
        .collect();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        roots.push(format!("{}\\Programs", local.to_string_lossy()));
    }
    roots
}

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
        } else {
            // Cửa sổ tray CHƯA tồn tại (đang khởi động: đăng ký TSF + dò
            // activation, có thể mất >1s). Trước đây yêu cầu thoát bị MẤT ÂM
            // THẦM → `--stop` ngay sau khi start không có tác dụng (build smoke
            // 0.2.21 bắt được: "Runtime smoke did not stop cleanly"). Ghi thành
            // cờ chờ; tray tiêu thụ ngay trước vòng lặp thông điệp (B17).
            TRAY_EXIT_PENDING.store(true, std::sync::atomic::Ordering::Release);
        }
    }
}

/// Yêu cầu thoát đến khi cửa sổ tray chưa tồn tại (B17). Tray gọi
/// `take_tray_exit_pending()` ngay trước vòng lặp để không bỏ mất `--stop`
/// phát ra trong lúc khởi động.
pub static TRAY_EXIT_PENDING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Lấy (và xoá) yêu cầu thoát đến sớm.
pub fn take_tray_exit_pending() -> bool {
    TRAY_EXIT_PENDING.swap(false, std::sync::atomic::Ordering::AcqRel)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_install_gate_is_prefix_not_substring() {
        let roots = vec![
            r"C:\Program Files".to_string(),
            r"C:\Program Files (x86)".to_string(),
            r"C:\Users\a\AppData\Local\Programs".to_string(),
        ];
        for ok in [
            r"C:\Program Files\TextVN\TextVN.exe",
            r"c:\program files (x86)\TextVN\TextVN.exe",
            r"C:\Program Files\WindowsApps\TextVN_0.2.27\TextVN.exe",
            r"C:\Users\a\AppData\Local\Programs\TextVN\TextVN.exe",
            r"\\?\C:\Program Files\TextVN\TextVN.exe",
        ] {
            assert!(path_is_under_any(ok, &roots), "{ok}");
        }
        for bad in [
            r"D:\x\Program Files\TextVN\TextVN.exe",
            r"C:\Users\a\Downloads\program files\TextVN.exe",
            r"C:\Program FilesEvil\TextVN.exe",
            r"C:\Program Files",
            r"C:\Users\a\AppData\Local\ProgramsX\TextVN.exe",
        ] {
            assert!(!path_is_under_any(bad, &roots), "{bad}");
        }
        assert!(!path_is_under_any(r"C:\x\TextVN.exe", &[String::new()]));
    }

    /// Debounce chéo nguồn: lần đầu claim được; trong 250ms claim lại phải
    /// thất bại (E11 — nguồn sau của CÙNG lần bấm không được toggle lần hai).
    /// Dùng cell RIÊNG + đồng hồ tiêm vào: không nhiễu với test khác chạy song
    /// song claim vào static thật (flake CI 2026-10-02).
    #[test]
    fn global_toggle_claim_debounces_within_window() {
        let cell = std::sync::atomic::AtomicU64::new(0);
        assert!(claim_debounced(&cell, 1_000, 250), "claim đầu tiên phải OK");
        assert!(
            !claim_debounced(&cell, 1_100, 250),
            "claim trong cửa sổ phải bị chặn"
        );
        assert!(
            claim_debounced(&cell, 1_300, 250),
            "claim sau khi hết cửa sổ phải OK"
        );
        // KHÔNG assert trên static toàn cục ở đây: test khác chạy song song
        // (toggle_global_responds_...) cũng claim static thật → flake CI
        // 2026-10-03. Đường thật đã được phủ bởi chính test ipc_server đó.
    }
}
