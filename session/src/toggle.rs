// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `toggle` dùng chung 3 OS trong `textvn-session` (P0-6 §5, task SF-2).
//!
//! Thuần logic, không I/O, không đồng hồ (thời gian đến qua `now_ms`).
//! Hợp nhất trạng thái phím chuyển Việt/Anh (`Ctrl+Shift` nhấn-thả không kèm phím khác,
//! `Ctrl+Shift+Space` bỏ qua bit `CapsLock` theo ADR-011) và standalone `Ctrl` tap
//! (nhấn rồi nhả riêng `Ctrl` trong ≤ `400 ms` theo R2-143) kèm debounce `250 ms`.

/// Giữ `Ctrl` lâu hơn ngần này không còn là tap (R2-143: tránh nhầm Ctrl+click / Ctrl+scroll).
pub const CTRL_TAP_MAX_MS: u64 = 400;
/// Cửa sổ chống bật/tắt kép giữa 2 pha hoặc giữa hotkey với tray (khớp Windows `250 ms`).
pub const TOGGLE_DEBOUNCE_MS: u64 = 250;

// Mã phím ảo canonical (Win VK, P0-2 §1) và bit modifier: giá trị lấy từ NGUỒN DUY NHẤT
// `textvn_core::keymap` (G22) — tên `VK_*`/`MOD_*` chỉ là bí danh cho adapter dùng gọn.
use textvn_core::keymap::vk;

/// Shift (chung).
pub const VK_SHIFT: u32 = vk::SHIFT;
/// Ctrl (chung).
pub const VK_CONTROL: u32 = vk::CONTROL;
/// Alt (chung).
pub const VK_MENU: u32 = vk::MENU;
/// CapsLock.
pub const VK_CAPITAL: u32 = vk::CAPSLOCK;
/// Dấu cách.
pub const VK_SPACE: u32 = vk::SPACE;
/// Win trái.
pub const VK_LWIN: u32 = vk::LWIN;
/// Win phải.
pub const VK_RWIN: u32 = vk::RWIN;
/// Shift trái.
pub const VK_LSHIFT: u32 = vk::LSHIFT;
/// Shift phải.
pub const VK_RSHIFT: u32 = vk::RSHIFT;
/// Ctrl trái.
pub const VK_LCONTROL: u32 = vk::LCONTROL;
/// Ctrl phải.
pub const VK_RCONTROL: u32 = vk::RCONTROL;
/// Alt trái.
pub const VK_LMENU: u32 = vk::LMENU;
/// Alt phải.
pub const VK_RMENU: u32 = vk::RMENU;

/// Bit Shift.
pub const MOD_SHIFT: u32 = textvn_core::keymap::MOD_SHIFT;
/// Bit Ctrl.
pub const MOD_CTRL: u32 = textvn_core::keymap::MOD_CTRL;
/// Bit Alt.
pub const MOD_ALT: u32 = textvn_core::keymap::MOD_ALT;
/// Bit Super/Win/Cmd.
pub const MOD_SUPER: u32 = textvn_core::keymap::MOD_SUPER;
/// Bit CapsLock.
pub const MOD_CAPS: u32 = textvn_core::keymap::MOD_CAPS;

/// Phím là Ctrl (chung, trái, phải)?
#[must_use]
pub const fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk, VK_CONTROL | VK_LCONTROL | VK_RCONTROL)
}

/// Phím là Shift (chung, trái, phải)?
#[must_use]
pub const fn is_shift_vk(vk: u32) -> bool {
    matches!(vk, VK_SHIFT | VK_LSHIFT | VK_RSHIFT)
}

/// Phím chỉ là modifier hoặc lock (Shift/Ctrl/Alt/CapsLock/Win)?
#[must_use]
pub const fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        VK_SHIFT
            | VK_CONTROL
            | VK_MENU
            | VK_CAPITAL
            | VK_LWIN
            | VK_RWIN
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_LMENU
            | VK_RMENU
    )
}

/// Tổ hợp `Ctrl+Shift+Space` (bỏ bit `CapsLock` theo ADR-011; có Alt/Super thì loại).
#[must_use]
pub const fn is_toggle_chord(vk: u32, mods: u32) -> bool {
    vk == VK_SPACE
        && (mods & (MOD_CTRL | MOD_SHIFT | MOD_ALT | MOD_SUPER)) == (MOD_CTRL | MOD_SHIFT)
}

/// Kết quả xử lý phím của máy trạng thái `toggle`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleOutcome {
    /// Không kích hoạt chuyển chế độ hay Ctrl-tap.
    #[default]
    None,
    /// Đảo trạng thái Việt/Anh (`Ctrl+Shift` nhấn-thả hoặc `Ctrl+Shift+Space`, đã qua debounce).
    ToggleViEn,
    /// Nhấn-thả riêng lẻ phím `Ctrl` trong ≤ [`CTRL_TAP_MAX_MS`] (chốt từ / khôi phục).
    CtrlTap,
}

/// Trạng thái thuần của phím chuyển Việt/Anh và standalone `Ctrl` tap cho một session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ToggleState {
    armed: bool,
    ctrl_armed: bool,
    ctrl_down_ms: Option<u64>,
    last_toggle_ms: Option<u64>,
}

impl ToggleState {
    /// Tạo trạng thái sạch.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            armed: false,
            ctrl_armed: false,
            ctrl_down_ms: None,
            last_toggle_ms: None,
        }
    }

    /// Quan sát `KeyDown`. Trả [`ToggleOutcome::ToggleViEn`] ngay nếu là chord `Ctrl+Shift+Space`.
    pub fn on_key_down(&mut self, vk: u32, mods: u32, now_ms: u64) -> ToggleOutcome {
        if is_toggle_chord(vk, mods) {
            self.reset();
            return if self.try_fire_toggle(now_ms) {
                ToggleOutcome::ToggleViEn
            } else {
                ToggleOutcome::None
            };
        }

        let has_ctrl = mods & MOD_CTRL != 0;
        let has_shift = mods & MOD_SHIFT != 0;
        let has_alt_or_super = mods & (MOD_ALT | MOD_SUPER) != 0;

        if is_ctrl_vk(vk) {
            self.armed = has_shift && !has_alt_or_super;
            self.ctrl_armed = !has_shift && !has_alt_or_super;
            self.ctrl_down_ms = if self.ctrl_armed {
                self.ctrl_down_ms.or(Some(now_ms))
            } else {
                None
            };
        } else if is_shift_vk(vk) {
            self.armed = has_ctrl && !has_alt_or_super;
            self.ctrl_armed = false;
            self.ctrl_down_ms = None;
        } else {
            self.armed = false;
            self.ctrl_armed = false;
            self.ctrl_down_ms = None;
        }
        ToggleOutcome::None
    }

    /// Quan sát `KeyUp`. Nhả cặp `Ctrl+Shift` hợp lệ ⇒ `ToggleViEn` đúng MỘT lần;
    /// nhả riêng `Ctrl` trong ≤ [`CTRL_TAP_MAX_MS`] ⇒ `CtrlTap`.
    pub fn on_key_up(&mut self, vk: u32, now_ms: u64) -> ToggleOutcome {
        if (is_ctrl_vk(vk) || is_shift_vk(vk)) && self.armed {
            self.armed = false;
            self.ctrl_armed = false;
            self.ctrl_down_ms = None;
            return if self.try_fire_toggle(now_ms) {
                ToggleOutcome::ToggleViEn
            } else {
                ToggleOutcome::None
            };
        }

        if is_ctrl_vk(vk) && self.ctrl_armed {
            self.ctrl_armed = false;
            let since = self.ctrl_down_ms.take();
            if since.is_some_and(|t| now_ms.saturating_sub(t) <= CTRL_TAP_MAX_MS) {
                return ToggleOutcome::CtrlTap;
            }
            return ToggleOutcome::None;
        }

        if is_modifier_vk(vk) {
            self.ctrl_armed = false;
            self.ctrl_down_ms = None;
        }
        ToggleOutcome::None
    }

    /// Huỷ lần nhấn đang chờ (focus đổi, hotkey khác, reset); giữ nguyên mốc debounce.
    pub fn reset(&mut self) {
        self.armed = false;
        self.ctrl_armed = false;
        self.ctrl_down_ms = None;
    }

    /// `Ctrl` đang chờ đã giữ vượt quá [`CTRL_TAP_MAX_MS`]?
    #[must_use]
    pub fn ctrl_held_too_long(&self, now_ms: u64) -> bool {
        self.ctrl_down_ms
            .is_some_and(|t| now_ms.saturating_sub(t) > CTRL_TAP_MAX_MS)
    }

    /// Cho phép bật/tắt nếu cách lần gần nhất ít nhất [`TOGGLE_DEBOUNCE_MS`].
    pub fn try_fire_toggle(&mut self, now_ms: u64) -> bool {
        if self
            .last_toggle_ms
            .is_some_and(|t| now_ms.saturating_sub(t) < TOGGLE_DEBOUNCE_MS)
        {
            return false;
        }
        self.last_toggle_ms = Some(now_ms);
        true
    }
}
