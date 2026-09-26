// SPDX-License-Identifier: GPL-3.0-or-later
//! Bộ biến đổi ký tự — dùng chung cho mọi method (P0-1 §1 `core/src/transform/`).

pub mod diacritic_style;
pub mod stroke;
pub mod tone;
pub mod undo;
pub mod vowel_table;

pub use diacritic_style::{pick_tone_target, DiacriticStyle};
pub use tone::{apply_tone, is_vowel, key_to_tone, strip_tone, tone_of, tone_to_key};
