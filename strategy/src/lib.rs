// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-strategy` — chọn strategy xuất chữ cho một key event.
//!
//! Nguồn sự thật: `docs/10-shared/P0-3-config-preset-strategy.md §3.1` (quy tắc phân quyền)
//! và `docs/10-shared/P0-2-engine-ffi-contract.md §1` (giá trị hằng số).
//! Crate này **không** phụ thuộc OS, không I/O — dùng chung cho mọi adapter.

pub mod action;
pub mod resolve;
pub mod rules_field;

pub use action::ActionKind;
pub use resolve::{resolve, ResolveInput, Strategy};
pub use rules_field::{default_for_field, parse_field_role, FIELD_ROLES};

/// `IME_FIELD_*` — ánh xạ 1-1 với `field_role` JSON (P0-3 §2.1).
pub const IME_FIELD_UNKNOWN: u32 = 0;
pub const IME_FIELD_BODY: u32 = 1;
pub const IME_FIELD_EDITBOX: u32 = 2;
pub const IME_FIELD_ADDRESS_BAR: u32 = 3;
pub const IME_FIELD_SEARCH: u32 = 4;
pub const IME_FIELD_COMBO: u32 = 5;
pub const IME_FIELD_CANDIDATE: u32 = 6;
pub const IME_FIELD_TEXTAREA: u32 = 7;
pub const IME_FIELD_WEB: u32 = 8;
pub const IME_FIELD_TERMINAL: u32 = 9;
pub const IME_FIELD_SECURE: u32 = 10;

/// `IME_CAP_*` — capability adapter báo cáo qua `ime_context_v1.caps`.
pub const IME_CAP_PREEDIT: u32 = 0x1;
pub const IME_CAP_SELECTION: u32 = 0x2;
pub const IME_CAP_FIELD_DETECT: u32 = 0x4;
pub const IME_CAP_INJECT_VK: u32 = 0x8;

/// `IME_STRATEGY_*` — id dùng bên phía FFI (`ime_context_v1.hint`).
pub const IME_STRATEGY_PREEDIT: i64 = 0;
pub const IME_STRATEGY_BACKSPACE_TYPE: i64 = 1;
pub const IME_STRATEGY_SELECTION_REPLACE: i64 = 2;
pub const IME_STRATEGY_FORWARD_AS_COMMIT: i64 = 3;
pub const IME_STRATEGY_PASSTHROUGH: i64 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_values_match_p0_2() {
        assert_eq!(IME_FIELD_SECURE, 10);
        assert_eq!(IME_FIELD_TERMINAL, 9);
        assert_eq!(
            IME_CAP_PREEDIT | IME_CAP_SELECTION | IME_CAP_FIELD_DETECT | IME_CAP_INJECT_VK,
            0xF
        );
        assert_eq!(IME_STRATEGY_PASSTHROUGH, 4);
    }
}
