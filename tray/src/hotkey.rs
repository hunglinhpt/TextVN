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

/// Một mục TextVN đã đổi khi dành Ctrl + Shift: tên giá trị registry + giá trị trước đó
/// (`None` = trước đó không có giá trị, tức mặc định của Windows).
pub type FreedEntry = (String, Option<String>);

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

    /// Giá trị hiện tại của một mục trong `HKCU\Keyboard Layout\Toggle` theo tên registry.
    pub fn get(&self, name: &str) -> Option<&str> {
        match name {
            "Layout Hotkey" => self.layout.as_deref(),
            "Language Hotkey" => self.language.as_deref(),
            "Hotkey" => self.legacy.as_deref(),
            _ => None,
        }
    }

    /// Bản ghi những gì [`ToggleHotkeys::freed`] sắp đổi: tên + giá trị TRƯỚC đó
    /// (`None` = trước đó không có giá trị) — để gỡ cài đặt trả lại đúng (R2-40).
    pub fn freed_record(&self) -> Vec<FreedEntry> {
        self.freed()
            .into_iter()
            .map(|(name, _)| (name.to_string(), self.get(name).map(str::to_string)))
            .collect()
    }

    /// Trả lại Windows theo bản ghi: chỉ mục VẪN là "không gán" (TextVN đặt); người
    /// dùng đã tự đổi sau đó thì giữ lựa chọn của họ.
    pub fn restore_plan(&self, record: &[FreedEntry]) -> Vec<FreedEntry> {
        record
            .iter()
            .filter(|(name, _)| self.get(name) == Some(NOT_ASSIGNED))
            .cloned()
            .collect()
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
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
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

    /// Ghi (`Some`) hoặc xoá (`None`) từng giá trị rồi báo Windows đọc lại.
    fn write(values: &[(&str, Option<&str>)]) -> Result<(), String> {
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
            let Some(value) = value else {
                // SAFETY: key mở với KEY_SET_VALUE; tên NUL-terminated.
                let status = unsafe { RegDeleteValueW(key, PCWSTR(name.as_ptr())) };
                if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
                    result = Err(format!("RegDeleteValueW: {status:?}"));
                }
                continue;
            };
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

    /// Gỡ Ctrl + Shift khỏi phím tắt đổi ngôn ngữ/bố cục của Windows. Trả bản ghi giá
    /// trị trước đó của những mục đã đổi (rỗng = không có gì phải đổi).
    pub fn free_ctrl_shift() -> Result<Vec<FreedEntry>, String> {
        let now = current();
        let record = now.freed_record();
        let values: Vec<(&str, Option<&str>)> =
            now.freed().into_iter().map(|(n, v)| (n, Some(v))).collect();
        write(&values)?;
        Ok(record)
    }

    /// Trả Ctrl + Shift về cho việc đổi bố cục bàn phím (mặc định của Windows).
    pub fn restore_windows_ctrl_shift() -> Result<(), String> {
        write(&[("Layout Hotkey", Some(CTRL_SHIFT))])
    }

    /// Trả lại đúng các giá trị trong bản ghi (chỉ mục còn "không gán") — R2-40.
    pub fn restore_freed(record: &[FreedEntry]) -> Result<(), String> {
        let plan = current().restore_plan(record);
        let values: Vec<(&str, Option<&str>)> = plan
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_deref()))
            .collect();
        write(&values)
    }
}

#[cfg(windows)]
pub use win::{current, free_ctrl_shift, restore_freed, restore_windows_ctrl_shift};

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

    /// R2-40: bản ghi nhớ giá trị TRƯỚC khi dành Ctrl + Shift, gồm cả `Language Hotkey`.
    #[test]
    fn freed_record_keeps_previous_values() {
        assert_eq!(
            hk(None, Some("2"), None).freed_record(),
            vec![
                ("Layout Hotkey".to_string(), None),
                ("Language Hotkey".to_string(), Some("2".to_string())),
            ]
        );
        assert!(hk(Some("1"), Some("1"), Some("3"))
            .freed_record()
            .is_empty());
    }

    /// R2-40: chỉ trả mục còn "không gán"; người dùng tự đổi sau đó (ví dụ Alt+Shift)
    /// thì giữ nguyên lựa chọn của họ.
    #[test]
    fn restore_plan_only_touches_values_textvn_still_owns() {
        let record = vec![
            ("Layout Hotkey".to_string(), None),
            ("Language Hotkey".to_string(), Some("2".to_string())),
        ];
        assert_eq!(hk(None, Some("3"), Some("3")).restore_plan(&record), record);
        assert_eq!(
            hk(None, Some("1"), Some("3")).restore_plan(&record),
            vec![("Layout Hotkey".to_string(), None)]
        );
        assert!(hk(None, Some("1"), Some("2"))
            .restore_plan(&record)
            .is_empty());
    }
}
