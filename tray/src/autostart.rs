// SPDX-License-Identifier: GPL-3.0-or-later
//! Quản lý Autostart cho TextVN Tray (WIN-053 — P1-4 §1).
//!
//! Ghi / đọc / xoá registry key:
//! `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TextVN` = `"<path>\textvn-tray.exe" --autostart`
//! Tuân thủ Rule S5: Phạm vi per-user, không yêu cầu quyền Administrator.

use std::path::Path;

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::System::Registry::*;

pub const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const APP_RUN_VALUE_NAME: &str = "TextVN";
pub const LEGACY_APP_RUN_VALUE_NAME: &str = "TextVN";

/// Kiểm tra xem TextVN có đang được cấu hình tự khởi động cùng Windows không.
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

            // TextVN là primary; chỉ đọc key TextVN để migration không làm mất
            // autostart của bản cũ trước khi người dùng lưu cấu hình mới.
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
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// Bật tự khởi động cùng Windows cho TextVN Tray.
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
    #[cfg(not(windows))]
    {
        let _ = exe_path;
        Ok(())
    }
}

/// Tắt tự khởi động cùng Windows cho TextVN Tray.
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
    #[cfg(not(windows))]
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
    use std::path::PathBuf;

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
}
