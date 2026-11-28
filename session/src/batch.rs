// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `batch` dùng chung 3 OS trong `textvn-session` (P0-6 §5, R2-165, R2-167, task SF-2).
//!
//! Port hợp nhất của `adapters/linux-common/src/edit_batch.c`: gom nhiều chỉnh sửa
//! trong cùng một phím thành TỐI ĐA 1 `DeleteBefore` + TỐI ĐA 1 `Commit` và luôn
//! đẩy `DeleteBefore` trước `Commit` (và `Commit` trước `SetPreedit{""}`).

use crate::{Commands, SessionCommand};

/// Giới hạn byte UTF-8 tối đa của bộ đệm commit chờ trong một batch — khớp `LC_EDIT_BATCH_MAX`
/// (8192) của `adapters/linux-common/include/lc_edit_batch.h` (bản WIP từng ghi nhầm 1024).
pub const EDIT_BATCH_MAX_BYTES: usize = 8192;

/// Đếm số ký tự Unicode trong chuỗi UTF-8 (khớp `lc_utf8_char_count`).
#[must_use]
pub fn utf8_char_count(s: &str) -> usize {
    s.chars().count()
}

/// Bộ gom chỉnh sửa cho một sự kiện phím — giữ lồng nhau (`begin`/`end`) và luật R2-165.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditBatch {
    depth: u32,
    delete_before: u32,
    text: String,
}

impl EditBatch {
    /// Tạo batch rỗng ở mức lồng `0`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mở một mức batch mới. Nếu đang ở ngoài (`depth == 0`), dọn phần tồn trước đó.
    pub fn begin(&mut self) {
        if self.depth == 0 {
            self.delete_before = 0;
            self.text.clear();
        }
        self.depth += 1;
    }

    /// Batch có đang mở hay không (`depth > 0`).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.depth > 0
    }

    /// Số ký tự đang xếp lịch xoá trước caret trong batch hiện tại.
    #[must_use]
    pub fn pending_delete_before(&self) -> u32 {
        self.delete_before
    }

    /// Chuỗi UTF-8 đang chờ commit trong batch hiện tại.
    #[must_use]
    pub fn pending_text(&self) -> &str {
        &self.text
    }

    /// Xoá `chars` ký tự ngay trước caret: ăn vào phần chờ `text` trước (vì phần đó
    /// nằm ngay trước caret của document ảo), phần còn dư mới cộng vào `delete_before`.
    pub fn delete(&mut self, mut chars: u32) {
        while chars > 0 && !self.text.is_empty() {
            let _ = self.text.pop();
            chars -= 1;
        }
        self.delete_before = self.delete_before.saturating_add(chars);
    }

    /// Gộp chuỗi `utf8` vào phần chờ commit khi batch đang mở và không tràn [`EDIT_BATCH_MAX_BYTES`].
    pub fn commit(&mut self, utf8: &str) -> bool {
        if !self.is_active() {
            return false;
        }
        if utf8.is_empty() {
            return true;
        }
        if self.text.len() + utf8.len() > EDIT_BATCH_MAX_BYTES {
            return false;
        }
        self.text.push_str(utf8);
        true
    }

    /// Lấy và xoá trắng phần chờ (`(delete_before, text)`), không đổi `depth`.
    /// Trả `None` khi cả hai đều rỗng.
    pub fn take(&mut self) -> Option<(u32, String)> {
        if self.delete_before == 0 && self.text.is_empty() {
            return None;
        }
        let d = self.delete_before;
        let t = std::mem::take(&mut self.text);
        self.delete_before = 0;
        Some((d, t))
    }

    /// Đóng một mức batch; chỉ trả kết quả khi về mức ngoài cùng (`depth == 0`).
    pub fn end(&mut self) -> Option<(u32, String)> {
        if self.depth == 0 {
            return None;
        }
        self.depth -= 1;
        if self.depth > 0 {
            return None;
        }
        self.take()
    }

    /// Xả phần chờ hiện tại thành lệnh session (`<= 1 DeleteBefore`, rồi `<= 1 Commit`),
    /// tuân thủ R2-165/R2-167 trước khi đẩy thêm `ForwardKey`/`SetPreedit{""}`.
    pub fn flush_into(&mut self, cmds: &mut Commands) {
        if let Some((delete_before, text)) = self.take() {
            if delete_before > 0 {
                let chars = u16::try_from(delete_before).unwrap_or(u16::MAX);
                cmds.push(SessionCommand::DeleteBefore { chars });
            }
            if !text.is_empty() {
                cmds.push(SessionCommand::Commit { text });
            }
        }
    }
}
