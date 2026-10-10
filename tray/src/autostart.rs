// SPDX-License-Identifier: GPL-3.0-or-later
//! Quản lý Autostart đa nền tảng cho TextVN Tray (WIN-053 — P1-4 §1 / LNX-050).
//!
//! - Windows: Ghi / đọc / xoá registry key:
//!   `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TextVN` = `"<path>\textvn-tray.exe" --autostart`
//! - Linux: Ghi / đọc / xoá file desktop entry:
//!   `~/.config/autostart/textvn.desktop` (Freedesktop autostart spec)
//!
//! Tuân thủ Rule S5: Phạm vi per-user, không yêu cầu quyền Administrator / sudo.
//!
//! Kênh Microsoft Store (`TextVN-Store`, xem `store.rs`): value `TextVN` LUÔN tồn tại —
//! `--autostart` khi bật, `--msix-guard` khi tắt — để guard gỡ/cập nhật vẫn chạy mỗi lần
//! đăng nhập. Tiến trình còn package identity không bao giờ ghi Run (bị ảo hoá).

use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::System::Registry::*;

pub const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const RUNONCE_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
pub const APP_RUN_VALUE_NAME: &str = "TextVN";
pub const LEGACY_APP_RUN_VALUE_NAME: &str = "TextVN";
pub const LINUX_DESKTOP_FILENAME: &str = "textvn.desktop";

/// Lấy thư mục autostart trên Linux theo chuẩn XDG (~/.config/autostart).
pub fn linux_autostart_dir() -> PathBuf {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg_config.is_empty() {
            return PathBuf::from(xdg_config).join("autostart");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".config").join("autostart");
        }
    }
    PathBuf::from("/tmp").join("autostart")
}

/// Đường dẫn file textvn.desktop trên Linux.
pub fn linux_autostart_desktop_path() -> PathBuf {
    linux_autostart_dir().join(LINUX_DESKTOP_FILENAME)
}

/// Sinh nội dung desktop entry cho Linux Autostart.
pub fn generate_linux_desktop_entry(exe_path: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=TextVN\n\
         Comment=Vietnamese Input Method Tray\n\
         Exec=\"{}\" --autostart\n\
         Icon=textvn\n\
         Terminal=false\n\
         Categories=Utility;\n\
         X-GNOME-Autostart-enabled=true\n",
        exe_path.display()
    )
}

/// Kiểm tra autostart trong một thư mục chỉ định (cho Linux và unit tests).
pub fn is_linux_autostart_enabled_in_dir(
    autostart_dir: &Path,
) -> std::result::Result<bool, String> {
    let desktop_file = autostart_dir.join(LINUX_DESKTOP_FILENAME);
    if !desktop_file.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(&desktop_file)
        .map_err(|e| format!("Failed to read autostart desktop file: {e}"))?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("X-GNOME-Autostart-enabled=false")
            || trimmed.eq_ignore_ascii_case("Hidden=true")
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Bật autostart trong một thư mục chỉ định (cho Linux và unit tests).
pub fn enable_linux_autostart_in_dir(
    autostart_dir: &Path,
    exe_path: &Path,
) -> std::result::Result<(), String> {
    std::fs::create_dir_all(autostart_dir)
        .map_err(|e| format!("Failed to create autostart directory: {e}"))?;
    let desktop_file = autostart_dir.join(LINUX_DESKTOP_FILENAME);
    let content = generate_linux_desktop_entry(exe_path);
    std::fs::write(&desktop_file, content)
        .map_err(|e| format!("Failed to write autostart desktop file: {e}"))?;
    Ok(())
}

/// Tắt autostart trong một thư mục chỉ định (cho Linux và unit tests).
pub fn disable_linux_autostart_in_dir(autostart_dir: &Path) -> std::result::Result<(), String> {
    let desktop_file = autostart_dir.join(LINUX_DESKTOP_FILENAME);
    if desktop_file.exists() {
        std::fs::remove_file(&desktop_file)
            .map_err(|e| format!("Failed to remove autostart desktop file: {e}"))?;
    }
    Ok(())
}

/// R2-98: marker `%APPDATA%\TextVN\autostart_disabled` — tài khoản này đã TẮT tự khởi
/// động trong khi mục Run nằm ở HKLM (bộ cài cho mọi người dùng): không có quyền admin
/// để xoá mục đó, nên tray chạy bằng `--autostart` thấy marker thì thoát ngay. Mỗi tài
/// khoản một lựa chọn; bật lại thì xoá marker.
pub const AUTOSTART_DISABLED_MARKER: &str = "autostart_disabled";

fn autostart_disabled_marker_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .filter(|v| !v.is_empty())
        .map(|d| {
            PathBuf::from(d)
                .join("TextVN")
                .join(AUTOSTART_DISABLED_MARKER)
        })
}

/// Tài khoản này đã tắt tự khởi động (marker R2-98).
pub fn autostart_disabled_by_user() -> bool {
    autostart_disabled_marker_path().is_some_and(|p| p.is_file())
}

/// Tray `--autostart` ở `exe_dir` phải thoát ngay: tài khoản đã tắt tự khởi động VÀ
/// mục Run HKLM trỏ đúng thư mục này (chính mục đó khởi chạy nó). Lần chạy `--autostart`
/// khác (smoke build, test gõ, bản portable) không bị chặn.
#[cfg(windows)]
pub fn autostart_suppressed_for(exe_dir: &Path) -> bool {
    autostart_disabled_by_user()
        && machine_autostart_command().is_some_and(|cmd| command_points_into(&cmd, exe_dir))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn set_autostart_disabled_marker(disabled: bool) -> std::result::Result<(), String> {
    let Some(path) = autostart_disabled_marker_path() else {
        return Err("APPDATA không có".to_string());
    };
    if disabled {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("tạo {}: {e}", dir.display()))?;
        }
        std::fs::write(&path, b"1").map_err(|e| format!("ghi {}: {e}", path.display()))
    } else {
        match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                Err(format!("xoá {}: {e}", path.display()))
            }
            _ => Ok(()),
        }
    }
}

/// Bật = mục HKCU (không phải guard Store) HOẶC mục HKLM mà tài khoản này chưa tắt.
pub fn autostart_effective(
    user_cmd: Option<&str>,
    machine_cmd: Option<&str>,
    user_disabled: bool,
) -> bool {
    let user = user_cmd
        .is_some_and(|cmd| !cmd.trim().is_empty() && !crate::store::run_command_is_guard(cmd));
    let machine = machine_cmd.is_some_and(|cmd| !cmd.trim().is_empty()) && !user_disabled;
    user || machine
}

/// Kiểm tra xem TextVN có đang được cấu hình tự khởi động cùng OS không.
/// Value `--msix-guard` (kênh Store, tự khởi động đang tắt) KHÔNG tính là bật; mục HKLM
/// mà tài khoản này đã tắt (marker R2-98) cũng không.
pub fn is_autostart_enabled() -> std::result::Result<bool, String> {
    #[cfg(windows)]
    {
        Ok(autostart_effective(
            autostart_command().as_deref(),
            machine_autostart_command().as_deref(),
            autostart_disabled_by_user(),
        ))
    }
    #[cfg(target_os = "linux")]
    {
        is_linux_autostart_enabled_in_dir(&linux_autostart_dir())
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Ok(false)
    }
}

/// Bật tự khởi động cùng OS cho TextVN Tray.
pub fn enable_autostart(exe_path: Option<&Path>) -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        // R2-27: bộ cài cho mọi người dùng đã đặt HKLM Run → không ghi thêm mục HKCU trùng
        // (hai tiến trình tray lúc đăng nhập, mục thừa còn lại sau khi gỡ). Bật lại = bỏ
        // marker tắt của tài khoản này (R2-98).
        if machine_autostart_command().is_some_and(|cmd| !cmd.trim().is_empty()) {
            return set_autostart_disabled_marker(false);
        }
        let _ = set_autostart_disabled_marker(false);
        let path = match exe_path {
            Some(p) => p.to_path_buf(),
            None => {
                std::env::current_exe().map_err(|e| format!("Cannot get current exe path: {e}"))?
            }
        };
        write_run_value(&format!("\"{}\" --autostart", path.display()))
    }
    #[cfg(target_os = "linux")]
    {
        let path = match exe_path {
            Some(p) => p.to_path_buf(),
            None => {
                std::env::current_exe().map_err(|e| format!("Cannot get current exe path: {e}"))?
            }
        };
        enable_linux_autostart_in_dir(&linux_autostart_dir(), &path)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = exe_path;
        Ok(())
    }
}

/// Ghi `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TextVN` = `cmd`.
/// Từ chối khi tiến trình còn package identity (ghi sẽ vào hive ảo của package).
#[cfg(windows)]
pub fn write_run_value(cmd: &str) -> std::result::Result<(), String> {
    if crate::store_win::refuse_if_packaged("ghi HKCU Run") {
        return Err("package identity: không ghi Run key".to_string());
    }
    set_hkcu_string(RUN_KEY_PATH, APP_RUN_VALUE_NAME, cmd)
}

/// Ghi `HKCU\...\RunOnce\<name>` = `cmd` (kênh Store hẹn xoá thư mục khi gỡ).
#[cfg(windows)]
pub fn set_runonce_value(name: &str, cmd: &str) -> std::result::Result<(), String> {
    if crate::store_win::refuse_if_packaged("ghi HKCU RunOnce") {
        return Err("package identity: không ghi RunOnce".to_string());
    }
    set_hkcu_string(RUNONCE_KEY_PATH, name, cmd)
}

/// Xoá `HKCU\...\RunOnce\<name>` (không có thì thôi).
#[cfg(windows)]
pub fn delete_runonce_value(name: &str) -> std::result::Result<(), String> {
    if crate::store_win::refuse_if_packaged("xoá HKCU RunOnce") {
        return Err("package identity: không ghi RunOnce".to_string());
    }
    delete_hkcu_values(RUNONCE_KEY_PATH, &[name])
}

#[cfg(windows)]
fn set_hkcu_string(subkey: &str, name: &str, value: &str) -> std::result::Result<(), String> {
    let cmd_wide = to_wide(value);
    // SAFETY: chuỗi NUL-terminated sống suốt các lời gọi; key đóng ở mọi nhánh.
    unsafe {
        let mut hkey = HKEY::default();
        let subkey_wide = to_wide(subkey);
        let status = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );
        if status != ERROR_SUCCESS {
            return Err(format!("RegCreateKeyExW failed with code {:?}", status));
        }

        let val_name_wide = to_wide(name);
        let byte_len = cmd_wide.len() * 2;
        let slice = std::slice::from_raw_parts(cmd_wide.as_ptr() as *const u8, byte_len);

        let set_status = RegSetValueExW(
            hkey,
            PCWSTR(val_name_wide.as_ptr()),
            None,
            REG_SZ,
            Some(slice),
        );

        let _ = RegCloseKey(hkey);

        if set_status != ERROR_SUCCESS {
            return Err(format!("RegSetValueExW failed with code {:?}", set_status));
        }

        Ok(())
    }
}

#[cfg(windows)]
fn delete_hkcu_values(subkey: &str, names: &[&str]) -> std::result::Result<(), String> {
    // SAFETY: chuỗi NUL-terminated sống suốt các lời gọi; key đóng trước khi trả về.
    unsafe {
        let mut hkey = HKEY::default();
        let subkey_wide = to_wide(subkey);
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            KEY_WRITE,
            &mut hkey,
        );
        if status != ERROR_SUCCESS {
            return Ok(());
        }
        let mut result = Ok(());
        for name in names {
            let val_name_wide = to_wide(name);
            let del_status = RegDeleteValueW(hkey, PCWSTR(val_name_wide.as_ptr()));
            if del_status != ERROR_SUCCESS && del_status != ERROR_FILE_NOT_FOUND {
                result = Err(format!("RegDeleteValueW failed with code {:?}", del_status));
            }
        }
        let _ = RegCloseKey(hkey);
        result
    }
}

/// Lệnh tự khởi động đang đăng ký (`HKCU\...\Run\TextVN`), nếu có.
#[cfg(windows)]
pub fn autostart_command() -> Option<String> {
    read_run_value(HKEY_CURRENT_USER, KEY_READ)
}

/// R2-27: lệnh tự khởi động do bộ cài cho MỌI người dùng ghi (`HKLM\...\Run\TextVN`,
/// view 64-bit) — checkbox phải thấy nó, nếu không người dùng bật lại sẽ tạo mục HKCU
/// trùng.
#[cfg(windows)]
pub fn machine_autostart_command() -> Option<String> {
    read_run_value(HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY)
}

#[cfg(windows)]
fn read_run_value(root: HKEY, access: REG_SAM_FLAGS) -> Option<String> {
    // SAFETY: buffer đủ `len` byte do chính RegQueryValueExW báo; key được đóng mọi nhánh.
    unsafe {
        let mut hkey = HKEY::default();
        let subkey_wide = to_wide(RUN_KEY_PATH);
        if RegOpenKeyExW(root, PCWSTR(subkey_wide.as_ptr()), None, access, &mut hkey)
            != ERROR_SUCCESS
        {
            return None;
        }
        let name = to_wide(APP_RUN_VALUE_NAME);
        let mut len = 0u32;
        let mut buf: Vec<u16> = Vec::new();
        let mut ok = RegQueryValueExW(
            hkey,
            PCWSTR(name.as_ptr()),
            None,
            None,
            None,
            Some(&mut len),
        ) == ERROR_SUCCESS;
        if ok && len > 0 {
            buf = vec![0u16; (len as usize).div_ceil(2)];
            ok = RegQueryValueExW(
                hkey,
                PCWSTR(name.as_ptr()),
                None,
                None,
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut len),
            ) == ERROR_SUCCESS;
        }
        let _ = RegCloseKey(hkey);
        if !ok {
            return None;
        }
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..end]))
    }
}

/// `true` nếu lệnh tự khởi động chạy một file nằm trong `dir`.
pub fn command_points_into(command: &str, dir: &Path) -> bool {
    let dir = dir
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase();
    let exe = command.trim().trim_start_matches('"').to_lowercase();
    !dir.is_empty()
        && exe
            .strip_prefix(&dir)
            .is_some_and(|rest| rest.starts_with(['\\', '/']))
}

/// Gỡ bản portable / dọn kênh Store: XOÁ value tự khởi động NẾU nó trỏ vào thư mục sắp
/// xoá — không đụng mục tự khởi động của một bản TextVN khác (ví dụ bản đã cài).
pub fn disable_autostart_for_dir(dir: &Path) -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        match autostart_command() {
            Some(cmd) if command_points_into(&cmd, dir) => delete_run_value(),
            _ => Ok(()),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Ok(())
    }
}

/// Tắt tự khởi động cùng OS cho TextVN Tray. Kênh Store: không xoá value mà đổi sang
/// `--msix-guard` (guard gỡ/cập nhật vẫn chạy lúc đăng nhập, tray thì không).
pub fn disable_autostart() -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        if let Some(ctx) = crate::store_win::detect_store_context() {
            return write_run_value(&crate::store::run_command(&ctx.exe, false));
        }
        delete_run_value()?;
        // R2-98: mục HKLM (bộ cài cho mọi người dùng) không xoá được khi thiếu quyền
        // admin — trước đây checkbox "đã lưu" mà lần đăng nhập sau tray vẫn chạy.
        if machine_autostart_command().is_some_and(|cmd| !cmd.trim().is_empty()) {
            set_autostart_disabled_marker(true)?;
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        disable_linux_autostart_in_dir(&linux_autostart_dir())
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Ok(())
    }
}

/// Xoá hẳn value `TextVN` (và tên cũ) khỏi HKCU Run.
#[cfg(windows)]
pub fn delete_run_value() -> std::result::Result<(), String> {
    if crate::store_win::refuse_if_packaged("xoá HKCU Run") {
        return Err("package identity: không ghi Run key".to_string());
    }
    delete_hkcu_values(
        RUN_KEY_PATH,
        &[APP_RUN_VALUE_NAME, LEGACY_APP_RUN_VALUE_NAME],
    )
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R2-98: mục HKLM mà tài khoản đã tắt (marker) không tính là bật; mục HKCU vẫn
    /// tính; guard Store không tính.
    #[test]
    fn autostart_effective_honours_per_user_disable_of_machine_entry() {
        let machine = Some(r#""C:\Program Files\TextVN\TextVN.exe" --autostart"#);
        let user = Some(r#""D:\TextVN\TextVN.exe" --autostart"#);
        let guard =
            Some(r#""C:\U\AppData\Local\Programs\TextVN-Store\1.2.28.0\TextVN.exe" --msix-guard"#);
        assert!(autostart_effective(None, machine, false));
        assert!(!autostart_effective(None, machine, true));
        assert!(autostart_effective(user, None, false));
        assert!(autostart_effective(user, machine, true));
        assert!(!autostart_effective(guard, None, false));
        assert!(!autostart_effective(None, Some("  "), false));
        assert!(!autostart_effective(None, None, false));
    }

    #[test]
    fn autostart_owner_check_matches_only_that_folder() {
        let dir = Path::new(r"D:\Tools\TextVN");
        assert!(command_points_into(
            r#""D:\Tools\TextVN\TextVN.exe" --autostart"#,
            dir
        ));
        assert!(command_points_into(
            r#""d:\tools\textvn\TextVN.exe" --autostart"#,
            dir
        ));
        assert!(!command_points_into(
            r#""D:\Tools\TextVN2\TextVN.exe" --autostart"#,
            dir
        ));
        assert!(!command_points_into(
            r#""C:\Users\a\AppData\Local\Programs\TextVN\TextVN.exe" --autostart"#,
            dir
        ));
        assert!(!command_points_into("", Path::new("")));
    }

    #[test]
    fn run_key_path_and_value_name_constants() {
        assert_eq!(
            RUN_KEY_PATH,
            r"Software\Microsoft\Windows\CurrentVersion\Run"
        );
        assert_eq!(APP_RUN_VALUE_NAME, "TextVN");
        assert_eq!(LEGACY_APP_RUN_VALUE_NAME, "TextVN");
    }

    #[test]
    fn autostart_command_string_formatting() {
        let fake_path = PathBuf::from(r"C:\Program Files\TextVN\TextVN.exe");
        let cmd = format!("\"{}\" --autostart", fake_path.display());
        assert_eq!(cmd, r#""C:\Program Files\TextVN\TextVN.exe" --autostart"#);
    }

    #[test]
    fn linux_desktop_entry_generation() {
        let fake_path = PathBuf::from("/usr/bin/textvn");
        let entry = generate_linux_desktop_entry(&fake_path);
        assert!(entry.contains("Exec=\"/usr/bin/textvn\" --autostart"));
        assert!(entry.contains("Type=Application"));
        assert!(entry.contains("Name=TextVN"));
        assert!(entry.contains("X-GNOME-Autostart-enabled=true"));
    }

    #[test]
    fn linux_autostart_lifecycle_in_dir() {
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_test_autostart_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Ban đầu chưa có file
        assert_eq!(is_linux_autostart_enabled_in_dir(&temp_dir), Ok(false));

        // Bật autostart
        let fake_bin = PathBuf::from("/usr/bin/textvn");
        assert_eq!(enable_linux_autostart_in_dir(&temp_dir, &fake_bin), Ok(()));
        assert_eq!(is_linux_autostart_enabled_in_dir(&temp_dir), Ok(true));

        // Tắt autostart
        assert_eq!(disable_linux_autostart_in_dir(&temp_dir), Ok(()));
        assert_eq!(is_linux_autostart_enabled_in_dir(&temp_dir), Ok(false));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
