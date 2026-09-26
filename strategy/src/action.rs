// SPDX-License-Identifier: GPL-3.0-or-later
//! Hành vi adapter xử lý — 4 action của `ime_result_v1.action` (P0-2 §2).
//! Giữ đúng tên để corpus (`:expect_action`) và FFI map thẳng.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Forward phím nguyên vẹn cho OS/app.
    Pass,
    /// Xóa `delete_count` ký tự trước con trỏ, chèn `insert[]` (P0-2 §2).
    Replace,
    /// Giữ nguyên preedit hiển thị, chèn `insert[]` và đóng composition (bug B2).
    Commit,
    /// Gỡ biến đổi trước đó — auto-restore tiếng Anh (bug B5).
    Restore,
}

impl ActionKind {
    pub fn id(self) -> u32 {
        match self {
            ActionKind::Pass => 0,
            ActionKind::Replace => 1,
            ActionKind::Commit => 2,
            ActionKind::Restore => 3,
        }
    }

    pub fn from_id(id: u32) -> Option<ActionKind> {
        match id {
            0 => Some(ActionKind::Pass),
            1 => Some(ActionKind::Replace),
            2 => Some(ActionKind::Commit),
            3 => Some(ActionKind::Restore),
            _ => None,
        }
    }

    /// Tên in corpus (`:expect_action`).
    pub fn as_str(self) -> &'static str {
        match self {
            ActionKind::Pass => "PASS",
            ActionKind::Replace => "REPLACE",
            ActionKind::Commit => "COMMIT",
            ActionKind::Restore => "RESTORE",
        }
    }
}
