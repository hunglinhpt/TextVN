// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `reset` dùng chung (P0-6 §5 hàng *reset*, task SF-3; R2-171, E78).
//!
//! GtkTextView (và toolkit tương tự) reset input context NGAY khi chèn dòng mới — cùng phím Enter,
//! vài ms sau. Reset đó tới trong [`RESET_AFTER_ENTER_MS`] sau Enter, không phím nào khác và không
//! đổi focus xen giữa ⇒ giữ mốc "đầu dòng" (adapter gọi `ime_reset_with_tail("\n")`) để chữ đầu
//! dòng mới vẫn tự viết hoa. Mọi reset khác xoá sạch ngữ cảnh (E78). Trước SF-3 luật này được viết
//! lặp ở IBus (`engine_reset`) và Fcitx5 (`TextVNEngine::reset`).

use crate::ResetKind;

/// Cửa sổ "reset ngay sau Enter" (= `LC_RESET_AFTER_ENTER_US` / 1000).
pub const RESET_AFTER_ENTER_MS: u64 = 250;

/// Theo dõi mốc nhấn Enter gần nhất (thuần, thời gian đến qua `now_ms`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResetTracker {
    enter_at_ms: Option<u64>,
}

impl ResetTracker {
    /// Tracker sạch.
    #[must_use]
    pub const fn new() -> Self {
        Self { enter_at_ms: None }
    }

    /// Mọi key-down (kể cả modifier): Enter ghi mốc, phím khác xoá mốc — reset tới sau đó không
    /// còn là reset của Enter.
    pub fn on_key_down(&mut self, is_enter: bool, now_ms: u64) {
        self.enter_at_ms = is_enter.then_some(now_ms);
    }

    /// Đổi focus / đổi bộ gõ ⇒ ngữ cảnh mới.
    pub fn on_context_change(&mut self) {
        self.enter_at_ms = None;
    }

    /// Quyết định một reset: `true` = giữ mốc đầu dòng (chỉ reset của toolkit trong cửa sổ sau
    /// Enter); `false` = xoá sạch. Reset do focus cũng xoá mốc.
    pub fn on_reset(&mut self, kind: ResetKind, now_ms: u64) -> bool {
        let line_start = kind == ResetKind::Toolkit
            && self
                .enter_at_ms
                .is_some_and(|t| now_ms.saturating_sub(t) < RESET_AFTER_ENTER_MS);
        if kind == ResetKind::Focus {
            self.enter_at_ms = None;
        }
        line_start
    }
}
