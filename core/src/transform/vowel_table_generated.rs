// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/tables/*.toml · sinh bằng `cargo xtask gen-tables` (P0-1 §3).
// Đổi bảng: sửa file `.toml` rồi chạy lại `cargo xtask gen-tables`.
// `cargo xtask check-tables` (CI) sẽ fail nếu file này lệch với nguồn.
//! Nguồn: `data/tables/vowels.toml` (digest FNV-1a 64 = `0xfe5318e15330844d`).
//!
//! Index âm trong bảng là **hợp đồng** với `transform::undo` (`mark_vowel`,
//! `mark_horn`) — đổi thứ tự trong `.toml` là đổi hành vi gõ.

use crate::transform::vowel_table::VowelEntry;

pub const A: usize = 0;
pub const A_BREVE: usize = 1; // ă
pub const A_CIRC: usize = 2; // â
pub const E: usize = 3;
pub const E_CIRC: usize = 4; // ê
pub const I: usize = 5;
pub const O: usize = 6;
pub const O_CIRC: usize = 7; // ô
pub const O_HOOK: usize = 8; // ơ
pub const U: usize = 9;
pub const U_HOOK: usize = 10; // ư
pub const Y: usize = 11;

pub const VOWEL_ENTRY_COUNT: usize = 12;
/// Số dạng mỗi âm: 0 = không dấu, 1..=5 = sắc · huyền · hỏi · ngã · nặng.
pub const TONES_PER_VOWEL: usize = 6;

pub const VOWELS: [VowelEntry; VOWEL_ENTRY_COUNT] = [
    VowelEntry {
        base: 'a',
        forms: ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
    },
    VowelEntry {
        base: 'ă',
        forms: ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    },
    VowelEntry {
        base: 'â',
        forms: ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    },
    VowelEntry {
        base: 'e',
        forms: ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    },
    VowelEntry {
        base: 'ê',
        forms: ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
    },
    VowelEntry {
        base: 'i',
        forms: ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
    },
    VowelEntry {
        base: 'o',
        forms: ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
    },
    VowelEntry {
        base: 'ô',
        forms: ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    },
    VowelEntry {
        base: 'ơ',
        forms: ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    },
    VowelEntry {
        base: 'u',
        forms: ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
    },
    VowelEntry {
        base: 'ư',
        forms: ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
    },
    VowelEntry {
        base: 'y',
        forms: ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
    },
];
