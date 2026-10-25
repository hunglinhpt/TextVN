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
    /// OP-006 (P5): từ được nhận diện là **identifier kỹ thuật** (camelCase —
    /// chữ hoa giữa từ như `useS` của `useState`) → phần còn lại của từ gõ
    /// thẳng raw, KHÔNG fold ("ress" của `onPress` không thành "rés").
    /// Nhận diện 1 lần là áp cho trọn từ (flag xoá cùng `clear()`).
    pub identifier: bool,
    /// Chữ đầu từ vừa bị engine **tự viết hoa** (auto-capitalize) — camel
    /// transition không kích hoạt trên từ này ("bAn" sau dấu chấm: 'B' là
    /// của engine, 'A' giữa từ chỉ đi qua — R1-4 bổ sung khi implement).
    pub auto_capped: bool,
}

impl Word {
    pub fn clear(&mut self) {
        self.raw.clear();
        self.passed.clear();
        self.display.clear();
        self.active = false;
        self.owned = 0;
        self.caps_lock = false;
        self.identifier = false;
        self.auto_capped = false;
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
