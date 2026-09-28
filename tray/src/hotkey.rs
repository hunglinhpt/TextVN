// SPDX-License-Identifier: GPL-3.0-or-later
//! Ctrl + Shift của Windows và phím chuyển V/E của TextVN.
//!
//! Windows mặc định dùng Ctrl + Shift để **đổi bố cục bàn phím** trong cùng một ngôn ngữ
//! (`HKCU\Keyboard Layout\Toggle`, giá trị `Layout Hotkey` = `2`); có người đặt cả đổi
//! **ngôn ngữ** (`Language Hotkey`) bằng Ctrl + Shift. Khi ngôn ngữ của TextVN có hơn một
//! bố cục (ví dụ bàn phím "Vietnamese" của Windows còn lại sau một lần gỡ), mỗi lần bấm
//! Ctrl + Shift vừa chuyển V/E vừa đổi sang bố cục khác — lần bấm sau không tới TextVN
//! (thấy bằng test gõ thật trên Windows). "Dành Ctrl + Shift cho TextVN" gỡ phím tắt đó,
//! đúng như người dùng tự làm trong Settings → Typing → Advanced keyboard settings →
//! Input language hot keys; bỏ chọn thì trả `Layout Hotkey` về Ctrl + Shift.

/// `HKCU\Keyboard Layout\Toggle`.
pub const TOGGLE_KEY_PATH: &str = r"Keyboard Layout\Toggle";
/// Mã phím tắt trong các giá trị của `Toggle`: `"1"` Alt trái + Shift, `"2"` Ctrl + Shift,
/// `"3"` không gán, `"4"` dấu huyền (`).
const CTRL_SHIFT: &str = "2";
const NOT_ASSIGNED: &str = "3";

/// Các giá trị đọc được từ `HKCU\Keyboard Layout\Toggle` (`None` = không có giá trị).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToggleHotkeys {
    /// `Hotkey` (tên cũ của `Language Hotkey`, Windows vẫn đọc khi thiếu tên mới).
    pub legacy: Option<String>,
    pub language: Option<String>,
    pub layout: Option<String>,
}

impl ToggleHotkeys {
    /// Windows có đang giữ Ctrl + Shift cho việc đổi ngôn ngữ/bố cục không.
    /// Thiếu `Layout Hotkey` = mặc định của Windows = Ctrl + Shift.
    pub fn ctrl_shift_taken(&self) -> bool {
        let language = self.language.as_deref().or(self.legacy.as_deref());
        language == Some(CTRL_SHIFT) || self.layout.as_deref().is_none_or(|v| v == CTRL_SHIFT)
    }

    /// Giá trị cần ghi để Ctrl + Shift chỉ còn cho TextVN (chỉ đổi mục đang là Ctrl + Shift).
    pub fn freed(&self) -> Vec<(&'static str, &'static str)> {
        let mut out = Vec::new();
        if self.layout.as_deref().is_none_or(|v| v == CTRL_SHIFT) {
            out.push(("Layout Hotkey", NOT_ASSIGNED));
        }
        if self.language.as_deref() == Some(CTRL_SHIFT) {
            out.push(("Language Hotkey", NOT_ASSIGNED));
        }
        if self.legacy.as_deref() == Some(CTRL_SHIFT) {
            out.push(("Hotkey", NOT_ASSIGNED));
        }
        out
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::*;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPI_SETLANGTOGGLE,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    fn read(key: HKEY, name: &str) -> Option<String> {
        let name = wide(name);
        let mut buf = [0u16; 16];
        let mut len = std::mem::size_of_val(&buf) as u32;
        // SAFETY: buffer cố định, `len` là kích thước byte của nó.
        let status = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(name.as_ptr()),
                None,
                None,
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut len),
            )
        };
        if status != ERROR_SUCCESS {
            return None;
        }
        let chars = (len as usize / 2).min(buf.len());
        let end = buf[..chars].iter().position(|&c| c == 0).unwrap_or(chars);
        Some(String::from_utf16_lossy(&buf[..end]).trim().to_string())
    }

    /// Đọc cấu hình phím tắt ngôn ngữ của người dùng hiện tại.
    pub fn current() -> ToggleHotkeys {
        let path = wide(TOGGLE_KEY_PATH);
        let mut key = HKEY::default();
        // SAFETY: mở khoá chỉ đọc của HKCU; đóng ngay sau khi đọc.
        let opened = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                KEY_READ,
                &mut key,
            )
        };
        if opened != ERROR_SUCCESS {
            return ToggleHotkeys::default();
        }
        let out = ToggleHotkeys {
            legacy: read(key, "Hotkey"),
            language: read(key, "Language Hotkey"),
            layout: read(key, "Layout Hotkey"),
        };
        // SAFETY: key vừa mở ở trên.
        let _ = unsafe { RegCloseKey(key) };
        out
    }

    fn write(values: &[(&str, &str)]) -> Result<(), String> {
        if values.is_empty() {
            return Ok(());
        }
        let path = wide(TOGGLE_KEY_PATH);
        let mut key = HKEY::default();
        // SAFETY: tạo/mở khoá HKCU để ghi; đóng ở cuối hàm.
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("RegCreateKeyExW: {status:?}"));
        }
        let mut result = Ok(());
        for (name, value) in values {
            let name = wide(name);
            let data = wide(value);
            // SAFETY: dữ liệu REG_SZ gồm cả NUL cuối, sống suốt lời gọi.
            let status = unsafe {
                RegSetValueExW(
                    key,
                    PCWSTR(name.as_ptr()),
                    None,
                    REG_SZ,
                    Some(std::slice::from_raw_parts(
                        data.as_ptr() as *const u8,
                        data.len() * 2,
                    )),
                )
            };
            if status != ERROR_SUCCESS {
                result = Err(format!("RegSetValueExW: {status:?}"));
            }
        }
        // SAFETY: key mở ở trên.
        let _ = unsafe { RegCloseKey(key) };
        result?;
        // Báo Windows đọc lại phím tắt ngay (như trang Settings làm), không cần đăng xuất.
        // SAFETY: SPI_SETLANGTOGGLE không dùng tham số con trỏ.
        unsafe { SystemParametersInfoW(SPI_SETLANGTOGGLE, 0, None, SPIF_SENDCHANGE) }
            .map_err(|e| format!("SystemParametersInfoW: {e}"))
    }

    /// Gỡ Ctrl + Shift khỏi phím tắt đổi ngôn ngữ/bố cục của Windows.
    pub fn free_ctrl_shift() -> Result<(), String> {
        write(&current().freed())
    }

    /// Trả Ctrl + Shift về cho việc đổi bố cục bàn phím (mặc định của Windows).
    pub fn restore_windows_ctrl_shift() -> Result<(), String> {
        write(&[("Layout Hotkey", CTRL_SHIFT)])
    }
}

#[cfg(windows)]
pub use win::{current, free_ctrl_shift, restore_windows_ctrl_shift};

#[cfg(test)]
mod tests {
    use super::*;

    fn hk(legacy: Option<&str>, language: Option<&str>, layout: Option<&str>) -> ToggleHotkeys {
        ToggleHotkeys {
            legacy: legacy.map(str::to_string),
            language: language.map(str::to_string),
            layout: layout.map(str::to_string),
        }
    }

    #[test]
    fn windows_default_takes_ctrl_shift_for_layouts() {
        // Mặc định Windows 10/11: Alt+Shift đổi ngôn ngữ, Ctrl+Shift đổi bố cục.
        let default = hk(Some("1"), Some("1"), Some("2"));
        assert!(default.ctrl_shift_taken());
        assert_eq!(default.freed(), vec![("Layout Hotkey", "3")]);
        // Không có giá trị nào = mặc định của Windows.
        assert!(hk(None, None, None).ctrl_shift_taken());
    }

    #[test]
    fn language_hotkey_on_ctrl_shift_is_freed_too() {
        let v = hk(Some("2"), Some("2"), Some("3"));
        assert!(v.ctrl_shift_taken());
        assert_eq!(v.freed(), vec![("Language Hotkey", "3"), ("Hotkey", "3")]);
        // Tên cũ `Hotkey` được dùng khi thiếu `Language Hotkey`.
        assert!(hk(Some("2"), None, Some("3")).ctrl_shift_taken());
    }

    #[test]
    fn freed_configuration_leaves_other_hotkeys_alone() {
        let v = hk(Some("1"), Some("1"), Some("3"));
        assert!(!v.ctrl_shift_taken());
        assert!(v.freed().is_empty());
        // Dấu huyền (Thai) / Alt+Shift giữ nguyên.
        assert!(!hk(Some("4"), Some("1"), Some("4")).ctrl_shift_taken());
    }
}
