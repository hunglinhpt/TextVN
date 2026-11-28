// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `ghost` dùng chung 3 OS (P0-6 §5 hàng *ghost*, task SF-4): chính sách chữ mờ gợi ý.
//!
//! Nguồn hợp nhất: TSF `edit_session.rs` (`update_ghost`, `ghost_waits_for_key_up`), IBus/Fcitx5
//! `update_preview` + `lc_preview_label`, IMK `updatePredictionPreview` + `previewLabel`.
//!
//! Luật (mỗi luật có test ở `session/tests/ghost.rs`):
//! - **R2-173 — chữ mờ = đúng phần Tab sẽ nối**: chữ mờ là `peek_completion` của engine NGUYÊN VĂN
//!   (không cắt, không thêm); session không tự đoán.
//! - **R2-134 — sống tới phím kế tiếp**: mỗi phím gõ chữ/xoá/Tab/Ctrl+Space vẽ lại; phím khác giữ
//!   nguyên (adapter ẩn khi chốt từ).
//! - **E79 — phím đi thẳng app**: phím sinh/xoá ký tự mà bộ gõ KHÔNG ăn ⇒ app chưa chèn lúc bộ gõ
//!   xử lý ⇒ vẽ ở key-up (con trỏ mới).
//! - **Cổng**: không vẽ khi ô secure, terminal, trợ lý đang hiện, tiếng Việt tắt mà gợi ý cũng tắt,
//!   hoặc sau caret còn chữ (đè lên chữ thật).

use crate::compose::KeyKind;

/// Khi nào vẽ lại chữ mờ sau một phím.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostTiming {
    /// Phím không ảnh hưởng chữ mờ.
    Keep,
    /// Vẽ lại ngay (bộ gõ đã áp xong thay đổi).
    Now,
    /// Hoãn tới key-up của phím này (E79/R2-173: app chưa chèn ký tự).
    AfterKeyUp,
}

/// Thời điểm vẽ chữ mờ cho phím `key`; `eaten` = bộ gõ ăn phím.
#[must_use]
pub fn ghost_timing(key: KeyKind, eaten: bool) -> GhostTiming {
    match key {
        KeyKind::Char(_) | KeyKind::Backspace if !eaten => GhostTiming::AfterKeyUp,
        KeyKind::Char(_) | KeyKind::Backspace | KeyKind::Tab | KeyKind::CtrlSpace => {
            GhostTiming::Now
        }
        _ => GhostTiming::Keep,
    }
}

/// Bit cổng (khớp `IME_GHOST_*` của C ABI).
pub mod gate {
    /// Ô mật khẩu/secure (S3).
    pub const SECURE: u32 = 0x01;
    /// App terminal (R2-144/145: không dự đoán).
    pub const TERMINAL: u32 = 0x02;
    /// Trợ lý ngữ pháp đang hiện.
    pub const ASSIST_VISIBLE: u32 = 0x04;
    /// Tiếng Việt tắt VÀ gợi ý tắt.
    pub const SUGGEST_OFF: u32 = 0x08;
    /// Sau caret còn chữ trên dòng (adapter đo được).
    pub const TEXT_AFTER_CARET: u32 = 0x10;
}

/// Chữ mờ cần vẽ (nguyên văn `peek`), hoặc `None` = ẩn. `gates` = OR các bit [`gate`].
#[must_use]
pub fn ghost_text(peek: Option<&str>, gates: u32) -> Option<String> {
    if gates != 0 {
        return None;
    }
    let p = peek?;
    (!p.trim().is_empty()).then(|| p.to_string())
}

/// Nhãn xem trước cho adapter không vẽ được chữ mờ tại caret (IBus aux, Fcitx5 aux-down, IMK bảng
/// ứng viên): `"⇥ "` + chữ mờ bỏ khoảng trắng hai đầu; `None` = không có gì để hiện.
#[must_use]
pub fn preview_label(ghost: &str) -> Option<String> {
    let t = ghost.trim();
    (!t.is_empty()).then(|| format!("\u{21E5} {t}"))
}
