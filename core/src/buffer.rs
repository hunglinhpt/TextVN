// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng lặp từ ngữ của engine (P0-1 §1 `buffer.rs`).
//!
//! Model:
//! - `raw`   = chuỗi phím người dùng đã gõ trong từ này (gồm cả marker đã bị nuốt) → dùng cho RESTORE.
//! - `passed`= các ký tự đã được engine **PASS** (đang nằm trong document, chưa transform).
//! - `display` = chuỗi kết quả sau fold — là text engine "sở hữu" khi đã activate.
//! - `active`= true khi fold(raw) khác raw (đã có biến đổi) — từ đó mọi thay đổi qua REPLACE.
//! - `owned` = độ dài display trong document (số ký tự REPLACE kế tiếp phải xóa).

#[derive(Debug, Default, Clone)]
pub struct Word {
    pub raw: Vec<char>,
    pub passed: Vec<char>,
    pub display: Vec<char>,
    pub active: bool,
    pub owned: usize,
    /// Có phím nào của từ được gõ khi Caps Lock bật → phím dấu viết hoa (S F R X J W Z)
    /// vẫn là phím dấu, như UniKey. Gõ hoa bằng Shift (`USA`) thì vẫn là chữ.
    pub caps_lock: bool,
}

impl Word {
    pub fn clear(&mut self) {
        self.raw.clear();
        self.passed.clear();
        self.display.clear();
        self.active = false;
        self.owned = 0;
        self.caps_lock = false;
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Ký tự đã PASS và vẫn nằm trong document (chưa activate) — số cần xóa khi activate.
    pub fn pending_delete(&self) -> u16 {
        debug_assert!(!self.active);
        self.passed.len() as u16
    }
}
