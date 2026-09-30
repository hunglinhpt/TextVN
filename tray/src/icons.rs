// SPDX-License-Identifier: GPL-3.0-or-later
//! Nạp HICON của ứng dụng — dùng chung cho icon khay (V/E) và icon title bar
//! cửa sổ Bảng điều khiển/Gõ tắt (trung tính). ID trùng `tray/tray.rc` (resource
//! nhúng vào TextVN.exe qua `tray/build.rs`).

/// ID resource icon trạng thái tiếng Việt (`tray.rc`: `1 ICON`).
pub const IDI_ICON_V: usize = 1;
/// ID resource icon trạng thái English (`tray.rc`: `2 ICON`).
pub const IDI_ICON_E: usize = 2;
/// Icon trung tính cho title bar cửa sổ (`tray.rc`: `3 ICON`).
pub const IDI_ICON_T: usize = 3;

/// Nạp icon theo thứ tự: PE resource (tray.rc) → file `.ico` cạnh exe/cwd →
/// icon mặc định của Windows. Không bao giờ trả HICON invalid.
#[cfg(windows)]
pub fn load_app_icon(
    h_instance: windows::Win32::Foundation::HINSTANCE,
    res_id: usize,
    file_name: &str,
) -> windows::Win32::UI::WindowsAndMessaging::HICON {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        LoadIconW, LoadImageW, HICON, IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
        LR_SHARED,
    };

    unsafe {
        // 1. Thử nạp từ Win32 PE Resource (đã nhúng qua tray.rc)
        let icon_res = LoadImageW(
            Some(h_instance),
            PCWSTR(res_id as *const u16),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        );
        if let Ok(handle) = icon_res {
            let hicon = HICON(handle.0);
            if !hicon.is_invalid() {
                return hicon;
            }
        }

        // 2. Fallback: nạp từ file resources/<file_name> cạnh exe hoặc thư mục dự án
        let mut candidates = Vec::new();
        if let Ok(mut exe) = std::env::current_exe() {
            exe.pop();
            candidates.push(exe.join("resources").join(file_name));
            candidates.push(exe.join(file_name));
        }
        candidates.push(std::path::PathBuf::from("tray/resources").join(file_name));
        candidates.push(std::path::PathBuf::from("resources").join(file_name));

        for path in candidates {
            if path.exists() {
                let path_w: Vec<u16> = path
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(Some(0))
                    .collect();
                let icon_file = LoadImageW(
                    None,
                    PCWSTR(path_w.as_ptr()),
                    IMAGE_ICON,
                    0,
                    0,
                    LR_LOADFROMFILE | LR_DEFAULTSIZE,
                );
                if let Ok(handle) = icon_file {
                    let hicon = HICON(handle.0);
                    if !hicon.is_invalid() {
                        return hicon;
                    }
                }
            }
        }

        // 3. Fallback cuối cùng: default application icon
        LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
    }
}
