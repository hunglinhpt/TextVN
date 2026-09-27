// SPDX-License-Identifier: GPL-3.0-or-later
//! Post-processing (P0-1 §1 `core/src/post/`) — **stage 7** pipeline (PLAN §4.2):
//! word boundary · gõ tắt/emoji · auto-restore EN (B5) · viết hoa tự động.
//!
//! Macro/emoji nằm trong `post/` vì chúng chạy **sau** transform và cần biết text đã vào
//! document (`Engine::recent`).

pub mod caps;
pub mod emoji;
pub mod r#macro;
pub mod restore_en;
