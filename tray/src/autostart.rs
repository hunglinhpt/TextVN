// SPDX-License-Identifier: GPL-3.0-or-later
//! Quản lý Autostart đa nền tảng cho TextVN Tray (WIN-053 — P1-4 §1 / LNX-050).
//!
//! - Windows: Ghi / đọc / xoá registry key:
//!   `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TextVN` = `"<path>\textvn-tray.exe" --autostart`
//! - Linux: Ghi / đọc / xoá file desktop entry:
//!   `~/.config/autostart/textvn.desktop` (Freedesktop autostart spec)
//!
//! Tuân thủ Rule S5: Phạm vi per-user, không yêu cầu quyền Administrator / sudo.

use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::System::Registry::*;

pub const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
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

/// Kiểm tra xem TextVN có đang được cấu hình tự khởi động cùng OS không.
pub fn is_autostart_enabled() -> std::result::Result<bool, String> {
    #[cfg(windows)]
    {
        unsafe {
            let mut hkey = HKEY::default();
            let subkey_wide = to_wide(RUN_KEY_PATH);
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey_wide.as_ptr()),
                None,
                KEY_READ,
                &mut hkey,
            );
            if status != ERROR_SUCCESS {
                return Ok(false);
            }

            let val_name_wide = to_wide(APP_RUN_VALUE_NAME);
            let mut val_type = REG_VALUE_TYPE::default();
            let mut data_len = 0u32;

            let mut query_status = RegQueryValueExW(
                hkey,
                PCWSTR(val_name_wide.as_ptr()),
                None,
                Some(&mut val_type),
                None,
                Some(&mut data_len),
            );

            if query_status != ERROR_SUCCESS {
                let legacy_name_wide = to_wide(LEGACY_APP_RUN_VALUE_NAME);
                query_status = RegQueryValueExW(
                    hkey,
                    PCWSTR(legacy_name_wide.as_ptr()),
                    None,
                    Some(&mut val_type),
                    None,
                    Some(&mut data_len),
                );
            }

            let _ = RegCloseKey(hkey);

            Ok(query_status == ERROR_SUCCESS && data_len > 0)
        }
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
        let path = match exe_path {
            Some(p) => p.to_path_buf(),
            None => {
                std::env::current_exe().map_err(|e| format!("Cannot get current exe path: {e}"))?
            }
        };

        let cmd_str = format!("\"{}\" --autostart", path.display());
        let cmd_wide = to_wide(&cmd_str);

        unsafe {
            let mut hkey = HKEY::default();
            let subkey_wide = to_wide(RUN_KEY_PATH);
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

            let val_name_wide = to_wide(APP_RUN_VALUE_NAME);
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

/// Lệnh tự khởi động đang đăng ký (`HKCU\...\Run\TextVN`), nếu có.
#[cfg(windows)]
pub fn autostart_command() -> Option<String> {
    // SAFETY: buffer đủ `len` byte do chính RegQueryValueExW báo; key được đóng mọi nhánh.
    unsafe {
        let mut hkey = HKEY::default();
        let subkey_wide = to_wide(RUN_KEY_PATH);
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            KEY_READ,
            &mut hkey,
        ) != ERROR_SUCCESS
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

/// Gỡ bản portable: bỏ tự khởi động NẾU nó trỏ vào thư mục sắp xoá — không đụng mục
/// tự khởi động của một bản TextVN khác (ví dụ bản đã cài).
pub fn disable_autostart_for_dir(dir: &Path) -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        match autostart_command() {
            Some(cmd) if command_points_into(&cmd, dir) => disable_autostart(),
            _ => Ok(()),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Ok(())
    }
}

/// Tắt tự khởi động cùng OS cho TextVN Tray.
pub fn disable_autostart() -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        unsafe {
            let mut hkey = HKEY::default();
            let subkey_wide = to_wide(RUN_KEY_PATH);
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

            let val_name_wide = to_wide(APP_RUN_VALUE_NAME);
            let del_status = RegDeleteValueW(hkey, PCWSTR(val_name_wide.as_ptr()));
            let legacy_name_wide = to_wide(LEGACY_APP_RUN_VALUE_NAME);
            let _ = RegDeleteValueW(hkey, PCWSTR(legacy_name_wide.as_ptr()));
            let _ = RegCloseKey(hkey);

            if del_status != ERROR_SUCCESS && del_status != ERROR_FILE_NOT_FOUND {
                return Err(format!("RegDeleteValueW failed with code {:?}", del_status));
            }

            Ok(())
        }
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

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
