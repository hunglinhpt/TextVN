// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-5 (P0-6 §5 hàng *assist*) — C ABI luật quyết định của Trợ lý dùng chung
//! (`textvn_session::assist`) cho IBus/Fcitx5/IMK: phím khi bảng gợi ý SỐNG đang hiện (R2-129/P9) và
//! phím hẹn lại timer idle 1 s (R2-154/155). Thay `lc_assist_live_key`/`lc_assist_idle_arms` (C) và
//! `assistIdleArms` (Swift) — G22.

use std::panic::{catch_unwind, AssertUnwindSafe};

use textvn_session::assist::{idle_arms, live_action_vk_ch, OverlayAction};
use textvn_session::compose::KeyKind;

// Kết quả `ime_assist_live_key` (KEEP/DISMISS/CLOSE/APPLY_1 khớp `LC_LIVE_*` của Linux).
pub const IME_ALIVE_KEEP: i32 = 0;
pub const IME_ALIVE_DISMISS: i32 = 1;
pub const IME_ALIVE_CLOSE: i32 = 2;
pub const IME_ALIVE_RECALL: i32 = 3;
pub const IME_ALIVE_APPLY_1: i32 = 10;

/// Thân `ime_assist_live_key`: `vk` canonical, `ch` ký tự của phím (0 nếu không sinh ký tự),
/// `mods` bit `IME_MOD_*`. Lỗi nội bộ ⇒ `IME_ALIVE_DISMISS` (fail-open: đóng bảng, phím đi tiếp).
pub(crate) fn ime_assist_live_key_impl(vk: u32, ch: u32, mods: u32) -> i32 {
    match catch_unwind(AssertUnwindSafe(|| live_action_vk_ch(vk, ch, mods))) {
        Ok(OverlayAction::Keep) => IME_ALIVE_KEEP,
        Ok(OverlayAction::Close) => IME_ALIVE_CLOSE,
        Ok(OverlayAction::Recall) => IME_ALIVE_RECALL,
        Ok(OverlayAction::Suggestion(n)) => IME_ALIVE_APPLY_1 + n as i32,
        Ok(_) | Err(_) => IME_ALIVE_DISMISS,
    }
}

/// Thân `ime_assist_idle_arms`: 1 = phím này hẹn lại timer idle. Lỗi nội bộ ⇒ 0.
pub(crate) fn ime_assist_idle_arms_impl(vk: u32, ch: u32, mods: u32, vi_on: i32) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        i32::from(idle_arms(
            KeyKind::classify_with_mods(vk, ch, mods),
            vi_on != 0,
        ))
    }))
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sf5_abi_live_key_codes() {
        assert_eq!(
            ime_assist_live_key_impl(0x41, 'a' as u32, 0),
            IME_ALIVE_KEEP
        );
        assert_eq!(ime_assist_live_key_impl(0x1B, 0, 0), IME_ALIVE_CLOSE);
        assert_eq!(
            ime_assist_live_key_impl(0x32, '2' as u32, 0x4),
            IME_ALIVE_APPLY_1 + 1
        );
        assert_eq!(
            ime_assist_live_key_impl(0x47, 'G' as u32, 0x3),
            IME_ALIVE_RECALL
        );
        assert_eq!(
            ime_assist_live_key_impl(0x0D, '\n' as u32, 0),
            IME_ALIVE_DISMISS
        );
    }

    #[test]
    fn sf5_abi_idle_arms() {
        assert_eq!(ime_assist_idle_arms_impl(0x41, 'a' as u32, 0, 1), 1);
        assert_eq!(ime_assist_idle_arms_impl(0x08, 0, 0, 1), 1);
        assert_eq!(ime_assist_idle_arms_impl(0x0D, 0, 0, 1), 0);
        assert_eq!(ime_assist_idle_arms_impl(0x41, 'a' as u32, 0, 0), 0);
        assert_eq!(
            ime_assist_idle_arms_impl(0x20, ' ' as u32, 0x2, 1),
            0,
            "Ctrl+Space"
        );
        assert_eq!(
            ime_assist_idle_arms_impl(0, 0x85, 0, 1),
            0,
            "C1 không phải chữ"
        );
    }
}
