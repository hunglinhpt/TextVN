// SPDX-License-Identifier: GPL-3.0-or-later
//! Crate `textvn-tray` — Ứng dụng khay hệ thống, IPC server và điều phối runtime cho TextVN (Windows).
//!
//! Bao gồm các module chức năng:
//! - [`svc`]: Service layer in-process quản lý trạng thái (`state.json`) và cấu hình (`config.json`).
//! - [`ipc_server`]: Named Pipe server `\\.\pipe\textvn-ipc-v1`, broadcast cấu hình và giám sát hook.
//! - [`menu`]: Menu ngữ cảnh khay hệ thống (9 mục chuẩn Win32).
//! - [`autostart`]: Quản lý registry key tự khởi động `HKCU\...\Run\TextVN` (per-user).
//! - [`store`] / `store_win` / [`package_bootstrap`]: kênh Microsoft Store (MSIX) —
//!   stage ra ngoài container, guard gỡ/cập nhật, dọn dẹp.

// Tray = Windows-only (P1-4): trên non-Windows chỉ build để gate CI --workspace,
// các item Win32 không có caller là bình thường — không phải dead code thật.
#![cfg_attr(not(windows), allow(dead_code))]

pub mod autostart;
pub mod foreground;
pub mod hotkey;
pub mod icons;
pub mod ipc_server;
pub mod menu;
pub mod package_bootstrap;
pub mod settings;
pub mod settings_dialog;
pub mod store;
#[cfg(windows)]
pub mod store_win;
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

/// Thư mục cài đặt chỉ admin ghi được — Program Files / Program Files (x86) theo
/// **known folder** (registry HKLM do hệ thống quản lý, không đọc biến môi trường mà
/// người dùng tự đặt được qua `HKCU\Environment`) — nơi duy nhất tray được xin UAC
/// để chạy CLI cạnh nó (SEC-02). R2-03/R2-15: KHÔNG còn `%LOCALAPPDATA%\Programs`
/// (bộ cài per-user, bản Store staged): thư mục đó người dùng ghi được, đăng ký HKLM
/// trỏ vào đó là DLL do user thường kiểm soát nạp vào tiến trình của mọi tài khoản
/// (cả elevated). CLI tự chặn lần nữa (`register --scope machine` → exit 3).
pub fn trusted_install_roots() -> Vec<String> {
    #[cfg(windows)]
    {
        use windows::Win32::System::Com::CoTaskMemFree;
        use windows::Win32::UI::Shell::{
            FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, SHGetKnownFolderPath, KF_FLAG_DEFAULT,
        };
        let mut roots = Vec::new();
        for id in [FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86] {
            // SAFETY: GUID hằng hợp lệ; chuỗi trả về được giải phóng bằng CoTaskMemFree.
            let Ok(raw) = (unsafe { SHGetKnownFolderPath(&id, KF_FLAG_DEFAULT, None) }) else {
                continue;
            };
            // SAFETY: chuỗi NUL-terminated do shell cấp, còn sống tới CoTaskMemFree.
            let text = unsafe { raw.to_string() }.ok();
            // SAFETY: con trỏ do SHGetKnownFolderPath cấp phát bằng CoTaskMemAlloc.
            unsafe { CoTaskMemFree(Some(raw.0 as *const core::ffi::c_void)) };
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                roots.push(text);
            }
        }
        roots
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// `exe` thuộc bản cài phạm vi máy tin cậy: dưới một trong `roots` (Program Files)
/// và KHÔNG dưới `WindowsApps` (thư mục package Store, đổi theo version).
pub fn is_trusted_machine_install(exe: &str, roots: &[String]) -> bool {
    path_is_under_any(exe, roots)
        && !exe
            .replace('/', "\\")
            .to_lowercase()
            .contains("\\windowsapps\\")
}

/// Marker `%APPDATA%\TextVN\ctrl_shift_default_applied`: TextVN đã quyết định về
/// Ctrl + Shift của Windows (lần chạy đầu bản portable/Store, task bộ cài, checkbox).
/// Nội dung:
/// - `freed` + mỗi dòng `<tên giá trị>=<giá trị trước đó>` (rỗng = trước đó không có):
///   TextVN đã gỡ phím tắt của Windows; gỡ cài đặt trả lại ĐÚNG các giá trị đó (R2-40,
///   gồm cả `Language Hotkey`). Đọc cùng định dạng: `installer/windows/portable/
///   uninstall.ps1`, `TextVN-setup.iss` (`RestoreCtrlShift`).
/// - [`CTRL_SHIFT_MARKER_FREED`] (`1`, bản ≤ 0.2.27): như trên, chỉ biết `Layout Hotkey`.
/// - [`CTRL_SHIFT_MARKER_DECLINED`]: người dùng kênh Store chọn "Không" (R2-06) — không
///   hỏi lại, và gỡ cài đặt KHÔNG được đổi phím tắt của họ.
pub const CTRL_SHIFT_MARKER: &str = "ctrl_shift_default_applied";
pub const CTRL_SHIFT_MARKER_FREED: &str = "1";
pub const CTRL_SHIFT_MARKER_RECORD: &str = "freed";
pub const CTRL_SHIFT_MARKER_DECLINED: &str = "declined";

/// Tên giá trị được phép trong marker (file người dùng sửa được — không ghi tên lạ).
const TOGGLE_VALUE_NAMES: [&str; 3] = ["Layout Hotkey", "Language Hotkey", "Hotkey"];

/// Marker dạng bản ghi (R2-40).
pub fn format_ctrl_shift_marker(record: &[hotkey::FreedEntry]) -> String {
    let mut out = format!("{CTRL_SHIFT_MARKER_RECORD}\n");
    for (name, previous) in record {
        out.push_str(name);
        out.push('=');
        out.push_str(previous.as_deref().unwrap_or(""));
        out.push('\n');
    }
    out
}

/// Bản ghi những gì TextVN đã đổi; `None` = marker không nói TextVN lấy Ctrl + Shift
/// (`declined`, rỗng, lạ). Marker `1` của bản cũ = `Layout Hotkey` trước đó không có.
pub fn parse_ctrl_shift_marker(content: &str) -> Option<Vec<hotkey::FreedEntry>> {
    let mut lines = content.lines().map(str::trim);
    match lines.next()? {
        CTRL_SHIFT_MARKER_FREED => Some(vec![("Layout Hotkey".to_string(), None)]),
        CTRL_SHIFT_MARKER_RECORD => Some(
            lines
                .filter_map(|line| {
                    let (name, previous) = line.split_once('=')?;
                    let (name, previous) = (name.trim(), previous.trim());
                    let valid = TOGGLE_VALUE_NAMES.contains(&name)
                        && (previous.is_empty() || matches!(previous, "1" | "2" | "3" | "4"));
                    valid.then(|| {
                        (
                            name.to_string(),
                            (!previous.is_empty()).then(|| previous.to_string()),
                        )
                    })
                })
                .collect(),
        ),
        _ => None,
    }
}

/// Gộp bản ghi mới vào bản cũ: mục lần này đổi dùng giá trị trước-đó mới nhất; mục lần
/// này không đổi (đã "không gán" sẵn) giữ bản ghi cũ.
pub fn merge_ctrl_shift_record(
    old: Vec<hotkey::FreedEntry>,
    new: Vec<hotkey::FreedEntry>,
) -> Vec<hotkey::FreedEntry> {
    let mut out: Vec<hotkey::FreedEntry> = old
        .into_iter()
        .filter(|(name, _)| !new.iter().any(|(n, _)| n == name))
        .collect();
    out.extend(new);
    out
}

/// R2-92: bản ≤ 0.2.27 dành Ctrl + Shift (`Layout Hotkey` = `3`) mà KHÔNG ghi marker —
/// portable mỗi lần khởi động, kênh Store ở lần đầu; gỡ cài đặt của chúng xoá
/// `Layout Hotkey` khi còn là `3`. Nâng cấp lên bản có marker thấy "không gán" sẵn nên
/// `free_ctrl_shift` không đổi gì (bản ghi rỗng) → phải nhận lại đúng mục bản cũ đã đặt,
/// nếu không gỡ cài đặt không trả Ctrl + Shift cho Windows.
pub fn legacy_freed_layout_hotkey(previous_version: Option<&str>, layout: Option<&str>) -> bool {
    layout == Some("3") && previous_version.is_some_and(version_before_ctrl_shift_marker)
}

/// Phiên bản `a.b.c` < 0.2.28 (bản đầu tiên phát hành có marker bản ghi). Không parse
/// được → `false` (không đoán).
fn version_before_ctrl_shift_marker(version: &str) -> bool {
    let mut parts = version.trim().split('.').map(|p| p.parse::<u32>().ok());
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Some(a)), Some(Some(b)), Some(Some(c)), None) => (a, b, c) < (0, 2, 28),
        _ => false,
    }
}

/// Bản ghi trong marker hiện tại (không có/không phải bản ghi → rỗng).
pub fn read_ctrl_shift_record() -> Vec<hotkey::FreedEntry> {
    ctrl_shift_marker_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| parse_ctrl_shift_marker(&c))
        .unwrap_or_default()
}

/// Ghi lại việc TextVN vừa dành Ctrl + Shift (gộp với bản ghi trước đó).
pub fn record_ctrl_shift_freed(record: Vec<hotkey::FreedEntry>) {
    let merged = merge_ctrl_shift_record(read_ctrl_shift_record(), record);
    write_ctrl_shift_marker(&format_ctrl_shift_marker(&merged));
}

pub fn ctrl_shift_marker_path() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA")
        .filter(|v| !v.is_empty())
        .map(|d| {
            std::path::PathBuf::from(d)
                .join("TextVN")
                .join(CTRL_SHIFT_MARKER)
        })
}

/// Marker cho biết chính TextVN đã gỡ Ctrl + Shift khỏi Windows.
pub fn ctrl_shift_marker_says_freed(content: &str) -> bool {
    parse_ctrl_shift_marker(content).is_some()
}

/// Ghi marker (tạo `%APPDATA%\TextVN` nếu thiếu).
pub fn write_ctrl_shift_marker(value: &str) {
    if let Some(marker) = ctrl_shift_marker_path() {
        if let Some(dir) = marker.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&marker, value.as_bytes());
    }
}

/// Gỡ cài đặt (portable/Store): trả Ctrl + Shift cho Windows CHỈ khi marker nói TextVN
/// đã lấy nó (BUG-07), rồi xoá marker (cài lại sẽ hỏi/áp lại từ đầu).
pub fn restore_ctrl_shift_if_we_freed() {
    #[cfg(windows)]
    {
        let Some(marker) = ctrl_shift_marker_path() else {
            return;
        };
        let Ok(content) = std::fs::read_to_string(&marker) else {
            return;
        };
        match parse_ctrl_shift_marker(&content) {
            Some(record) => {
                if hotkey::restore_freed(&record).is_ok() {
                    let _ = std::fs::remove_file(&marker);
                }
            }
            None => {
                let _ = std::fs::remove_file(&marker);
            }
        }
    }
}

/// Checkbox "Dành Ctrl + Shift cho TextVN" bỏ chọn: trả lại theo bản ghi, rồi chắc chắn
/// Windows giữ Ctrl + Shift (người dùng chủ động chọn). Marker vẫn còn (bản ghi rỗng) để
/// lần khởi động sau không áp lại mặc định.
#[cfg(windows)]
pub fn give_ctrl_shift_back_to_windows() -> Result<(), String> {
    hotkey::restore_freed(&read_ctrl_shift_record())?;
    if !hotkey::current().ctrl_shift_taken() {
        hotkey::restore_windows_ctrl_shift()?;
    }
    write_ctrl_shift_marker(&format_ctrl_shift_marker(&[]));
    Ok(())
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
        ];
        for ok in [
            r"C:\Program Files\TextVN\TextVN.exe",
            r"c:\program files (x86)\TextVN\TextVN.exe",
            r"\\?\C:\Program Files\TextVN\TextVN.exe",
        ] {
            assert!(path_is_under_any(ok, &roots), "{ok}");
            assert!(is_trusted_machine_install(ok, &roots), "{ok}");
        }
        for bad in [
            r"D:\x\Program Files\TextVN\TextVN.exe",
            r"C:\Users\a\Downloads\program files\TextVN.exe",
            r"C:\Program FilesEvil\TextVN.exe",
            r"C:\Program Files",
            // R2-03/R2-15: bộ cài per-user + bản Store staged — người dùng ghi được.
            r"C:\Users\a\AppData\Local\Programs\TextVN\TextVN.exe",
            r"C:\Users\a\AppData\Local\Programs\TextVN-Store\1.2.27.0\TextVN.exe",
        ] {
            assert!(!is_trusted_machine_install(bad, &roots), "{bad}");
        }
        // WindowsApps nằm dưới Program Files nhưng không phải bản cài máy tin cậy.
        let pkg = r"C:\Program Files\WindowsApps\TextVN_0.2.27\TextVN.exe";
        assert!(path_is_under_any(pkg, &roots));
        assert!(!is_trusted_machine_install(pkg, &roots));
        assert!(!path_is_under_any(r"C:\x\TextVN.exe", &[String::new()]));
    }

    /// Không còn gốc nào lấy từ LOCALAPPDATA (R2-03/R2-15).
    #[test]
    fn trusted_roots_never_include_localappdata() {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
        for root in trusted_install_roots() {
            assert!(!root.to_lowercase().contains("appdata"), "{root}");
            if !local.is_empty() {
                assert!(!root.eq_ignore_ascii_case(&format!("{local}\\Programs")));
            }
        }
    }

    /// R2-06: chỉ marker "1" (TextVN đã gỡ phím tắt) mới khiến gỡ cài đặt trả lại
    /// Ctrl + Shift; "declined" (người dùng Store chọn Không) thì không đụng.
    #[test]
    fn ctrl_shift_marker_content_semantics() {
        assert!(ctrl_shift_marker_says_freed("1"));
        assert!(ctrl_shift_marker_says_freed("1\r\n"));
        assert!(ctrl_shift_marker_says_freed("freed\n"));
        assert!(!ctrl_shift_marker_says_freed(CTRL_SHIFT_MARKER_DECLINED));
        assert!(!ctrl_shift_marker_says_freed(""));
    }

    /// R2-40: bản ghi đi vòng format → parse; marker `1` của bản cũ = `Layout Hotkey`
    /// trước đó không có; tên/giá trị lạ trong file bị bỏ qua.
    #[test]
    fn ctrl_shift_marker_record_round_trip() {
        let record = vec![
            ("Layout Hotkey".to_string(), None),
            ("Language Hotkey".to_string(), Some("2".to_string())),
        ];
        let text = format_ctrl_shift_marker(&record);
        assert_eq!(text, "freed\nLayout Hotkey=\nLanguage Hotkey=2\n");
        assert_eq!(parse_ctrl_shift_marker(&text), Some(record));
        assert_eq!(
            parse_ctrl_shift_marker("1\r\n"),
            Some(vec![("Layout Hotkey".to_string(), None)])
        );
        assert_eq!(
            parse_ctrl_shift_marker("freed\r\nEvil=2\r\nHotkey=x\r\nHotkey=1\r\n"),
            Some(vec![("Hotkey".to_string(), Some("1".to_string()))])
        );
        assert_eq!(parse_ctrl_shift_marker("freed"), Some(vec![]));
        assert_eq!(parse_ctrl_shift_marker("declined"), None);
    }

    #[test]
    fn ctrl_shift_record_merge_keeps_untouched_entries() {
        let old = vec![
            ("Layout Hotkey".to_string(), None),
            ("Language Hotkey".to_string(), Some("2".to_string())),
        ];
        let new = vec![("Layout Hotkey".to_string(), Some("2".to_string()))];
        assert_eq!(
            merge_ctrl_shift_record(old, new),
            vec![
                ("Language Hotkey".to_string(), Some("2".to_string())),
                ("Layout Hotkey".to_string(), Some("2".to_string())),
            ]
        );
    }

    /// R2-92: nâng cấp từ bản ≤ 0.2.27 (không marker) mà Layout Hotkey đã là 3 → nhận
    /// lại mục bản cũ; bản mới hơn, lần đầu chạy, hay người dùng tự đặt khác 3 → không.
    #[test]
    fn legacy_freed_layout_hotkey_only_for_pre_marker_upgrades() {
        assert!(legacy_freed_layout_hotkey(Some("0.2.27"), Some("3")));
        assert!(legacy_freed_layout_hotkey(Some("0.2.9"), Some("3")));
        assert!(legacy_freed_layout_hotkey(Some(" 0.1.40 "), Some("3")));
        assert!(!legacy_freed_layout_hotkey(Some("0.2.28"), Some("3")));
        assert!(!legacy_freed_layout_hotkey(Some("0.3.0"), Some("3")));
        assert!(!legacy_freed_layout_hotkey(None, Some("3")));
        assert!(!legacy_freed_layout_hotkey(Some("0.2.27"), Some("2")));
        assert!(!legacy_freed_layout_hotkey(Some("0.2.27"), None));
        assert!(!legacy_freed_layout_hotkey(Some("0.2.27-dev"), Some("3")));
        assert!(!legacy_freed_layout_hotkey(Some("garbage"), Some("3")));
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
