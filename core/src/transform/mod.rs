// SPDX-License-Identifier: GPL-3.0-or-later
//! Bộ biến đổi ký tự — dùng chung cho mọi method (P0-1 §1 `core/src/transform/`).
//!
//! Bảng dữ liệu (72 âm, chỉ số âm) **không viết tay ở đây**: `vowel_table_generated.rs` là
//! output của `cargo xtask gen-tables` từ `data/tables/vowels.toml` (P0-1 §3).
//! `vowel_table.rs` giữ phần **logic** (tìm vị trí, gỡ dạng âm) + test.

pub mod charset;
pub mod diacritic_style;
pub mod stroke;
pub mod tone;
pub mod undo;
pub mod vowel_table;
// == GENERATED từ `data/tables/vowels.toml` (`cargo xtask gen-tables`) — KHÔNG sửa tay.
pub mod vowel_table_generated;

pub use charset::OutputCharset;
pub use diacritic_style::{pick_tone_target, DiacriticStyle};
pub use tone::{apply_tone, is_vowel, key_to_tone, strip_tone, tone_of, tone_to_key};
