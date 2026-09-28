// SPDX-License-Identifier: GPL-3.0-or-later
//! Nhật ký chẩn đoán của key sink — TẮT mặc định, không tốn gì khi tắt.
//!
//! Bật bằng biến môi trường `TEXTVN_TSF_TRACE=<đường dẫn file>` cho process của app
//! (app kế thừa từ launcher/phiên đăng nhập). Dùng khi báo lỗi "phím tới sai thứ tự",
//! "Ctrl+Shift không chuyển" — thấy được pha nào (`test`/`down`/`up`) tới, kết quả
//! eaten, context có phải CUAS (transitory) không.
//!
//! S2: KHÔNG BAO GIỜ ghi nội dung gõ. Phím sinh ký tự chỉ ghi lớp `chr`; chỉ modifier
//! và phím điều khiển (Enter, Tab, mũi tên…) mới ghi mã VK.

use std::io::Write;
use std::sync::{Mutex, OnceLock};

fn sink() -> Option<&'static Mutex<std::fs::File>> {
    static SINK: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    SINK.get_or_init(|| {
        let path = std::env::var_os("TEXTVN_TSF_TRACE")?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
            .map(Mutex::new)
    })
    .as_ref()
}

/// Ghi một dòng (`pid tid ms sự-kiện`). Không làm gì khi tắt.
pub fn event(tid: u32, args: std::fmt::Arguments<'_>) {
    let Some(file) = sink() else { return };
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() % 1_000_000)
        .unwrap_or(0);
    if let Ok(mut f) = file.lock() {
        let _ = writeln!(f, "{} {tid} {ms:06} {args}", std::process::id());
    }
}

/// Nhãn VK an toàn để ghi: phím có thể sinh ký tự → `chr`, còn lại → `vkXX`.
pub fn vk_label(vk: u32) -> String {
    let produces_char = matches!(
        vk,
        0x20 | 0x30..=0x39 | 0x41..=0x5A | 0x60..=0x6F | 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2
    );
    if produces_char {
        "chr".to_string()
    } else {
        format!("vk{vk:02X}")
    }
}

#[cfg(test)]
mod tests {
    use super::vk_label;

    #[test]
    fn character_keys_are_never_logged_by_code() {
        for vk in [
            0x20, 0x30, 0x39, 0x41, 0x5A, 0x60, 0x6F, 0xBA, 0xBC, 0xC0, 0xDB, 0xDE, 0xE2,
        ] {
            assert_eq!(vk_label(vk), "chr", "vk {vk:#x}");
        }
        assert_eq!(vk_label(0x0D), "vk0D");
        assert_eq!(vk_label(0x10), "vk10");
        assert_eq!(vk_label(0xA2), "vkA2");
        assert_eq!(vk_label(0x25), "vk25");
    }
}
