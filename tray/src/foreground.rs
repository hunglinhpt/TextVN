// SPDX-License-Identifier: GPL-3.0-or-later
//! Theo dõi app foreground gần nhất cho mục menu 4 "Bật tiếng Việt cho {app}"
//! (WIN-050 — P1-4 §1; review R3 minor 6).
//!
//! Lúc người dùng click icon khay, cửa sổ foreground luôn là taskbar của
//! Explorer — `GetForegroundWindow` tại thời điểm đó vô dụng. Vì vậy tray đăng ký
//! `EVENT_SYSTEM_FOREGROUND` (out-of-context, bỏ qua chính tiến trình tray) và
//! nhớ app người dùng đang gõ trước khi click khay. `app_id` = tên file exe
//! viết thường, khớp cách TSF định danh app (`adapters/windows-tsf/src/tip.rs`).

use std::sync::Mutex;

static LAST_APP: Mutex<Option<String>> = Mutex::new(None);

/// Lớp cửa sổ của shell (taskbar, khay tràn, desktop) — không phải app đang gõ.
const SHELL_CLASSES: [&str; 6] = [
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
    "Progman",
    "WorkerW",
];

/// Cửa sổ thuộc shell thì không ghi đè app đã nhớ.
pub fn is_shell_window_class(class: &str) -> bool {
    SHELL_CLASSES.iter().any(|c| c.eq_ignore_ascii_case(class))
}

/// `C:\Program Files\Foo\Chrome.EXE` → `chrome.exe`.
pub fn app_id_from_image_path(path: &str) -> Option<String> {
    let name = path.rsplit(['\\', '/']).next()?.trim();
    (!name.is_empty()).then(|| name.to_lowercase())
}

/// App foreground gần nhất (ngoài shell và chính tray).
pub fn last_app() -> Option<String> {
    LAST_APP.lock().ok().and_then(|g| g.clone())
}

fn remember(app: String) {
    if let Ok(mut g) = LAST_APP.lock() {
        *g = Some(app);
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HWND};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND,
        WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
    };

    fn app_of_window(hwnd: HWND) -> Option<String> {
        if hwnd.0.is_null() {
            return None;
        }
        let mut class = [0u16; 128];
        // SAFETY: buffer hợp lệ, hwnd do hệ thống cấp (có thể đã chết → trả 0).
        let n = unsafe { GetClassNameW(hwnd, &mut class) };
        if n > 0 && is_shell_window_class(&String::from_utf16_lossy(&class[..n as usize])) {
            return None;
        }
        let mut pid = 0u32;
        // SAFETY: out-param PID là biến cục bộ.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == 0 || pid == std::process::id() {
            return None;
        }
        // SAFETY: handle đóng ngay sau khi đọc tên ảnh tiến trình.
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(
                h,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            );
            let _ = CloseHandle(h);
            ok.ok()?;
            app_id_from_image_path(&String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    unsafe extern "system" fn on_foreground(
        _hook: HWINEVENTHOOK,
        _event: u32,
        hwnd: HWND,
        _id_object: i32,
        _id_child: i32,
        _thread: u32,
        _time: u32,
    ) {
        if let Some(app) = app_of_window(hwnd) {
            remember(app);
        }
    }

    /// Hook sống theo thread UI của tray (cần message loop). Drop → gỡ hook.
    pub struct ForegroundTracker(HWINEVENTHOOK);

    impl ForegroundTracker {
        pub fn start() -> Option<Self> {
            // SAFETY: GetForegroundWindow không có tiền điều kiện.
            if let Some(app) = app_of_window(unsafe { GetForegroundWindow() }) {
                remember(app);
            }
            // SAFETY: callback out-of-context chạy trên thread gọi (có message loop).
            let hook = unsafe {
                SetWinEventHook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    None,
                    Some(on_foreground),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                )
            };
            (!hook.is_invalid()).then_some(Self(hook))
        }
    }

    impl Drop for ForegroundTracker {
        fn drop(&mut self) {
            // SAFETY: hook do SetWinEventHook trả về, gỡ đúng một lần.
            let _ = unsafe { UnhookWinEvent(self.0) };
        }
    }
}

#[cfg(windows)]
pub use imp::ForegroundTracker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_id_is_lowercase_file_name() {
        assert_eq!(
            app_id_from_image_path(r"C:\Program Files\Google\Chrome\Application\Chrome.EXE"),
            Some("chrome.exe".into())
        );
        assert_eq!(
            app_id_from_image_path("notepad.exe"),
            Some("notepad.exe".into())
        );
        assert_eq!(app_id_from_image_path(r"C:\dir\"), None);
    }

    #[test]
    fn shell_windows_are_ignored() {
        assert!(is_shell_window_class("Shell_TrayWnd"));
        assert!(is_shell_window_class("shell_traywnd"));
        assert!(is_shell_window_class("NotifyIconOverflowWindow"));
        assert!(!is_shell_window_class("Chrome_WidgetWin_1"));
        assert!(!is_shell_window_class("Notepad"));
    }
}
