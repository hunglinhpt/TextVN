// SPDX-License-Identifier: GPL-3.0-or-later
//! Normalize sự kiện phím của adapter → `KeyEvent` cho engine (P0-1 §1 `keymap.rs`).
//!
//! `vk` dùng **giá trị Win VK làm canonical** (P0-2 §1: "Win VK / mac keycode / keysym" —
//! adapter macOS/Linux chuẩn hóa về đây trước khi gọi engine).
//!
//! Bảng keycode macOS → VK canonical nằm ở [`crate::keymap_mac_generated`]
//! (sinh từ `data/tables/keymap_mac.toml`) — dùng chung với Swift adapter
//! (`adapters/macos-imk/Sources/CoreBridge/KeyMapMacGenerated.swift`).

/// Virtual key canonical (Win VK code — adapter khác map về đây).
pub mod vk {
    pub const BACK: u32 = 0x08;
    pub const TAB: u32 = 0x09;
    pub const RETURN: u32 = 0x0D;
    pub const SHIFT: u32 = 0x10;
    pub const CONTROL: u32 = 0x11;
    pub const MENU: u32 = 0x12; // Alt
    pub const CAPSLOCK: u32 = 0x14;
    pub const ESCAPE: u32 = 0x1B;
    pub const SPACE: u32 = 0x20;
    pub const LEFT: u32 = 0x25;
    pub const UP: u32 = 0x26;
    pub const RIGHT: u32 = 0x27;
    pub const DOWN: u32 = 0x28;
    pub const DELETE: u32 = 0x2E;
    pub const LWIN: u32 = 0x5B;
    pub const RWIN: u32 = 0x5C;
    pub const F1: u32 = 0x70; // F1..F12 = 0x70..0x7B
}

/// `IME_MOD_*` (P0-2 §1).
pub const MOD_SHIFT: u32 = 0x1;
pub const MOD_CTRL: u32 = 0x2;
pub const MOD_ALT: u32 = 0x4;
pub const MOD_SUPER: u32 = 0x8;
pub const MOD_META: u32 = 0x10;
pub const MOD_CAPS: u32 = 0x20;
pub const MOD_FN: u32 = 0x40;

/// Phím đã normalize — 1 sự kiện key down/up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyEvent {
    pub vk: u32,
    /// Ký tự Unicode sau layout; 0 nếu không sinh ký tự.
    pub ch: u32,
    pub mods: u32,
    pub key_down: bool,
    pub is_repeat: bool,
    /// Adapter tự bơm phím này → engine BỎ QUA (chống loop — P0-3 §6).
    pub is_injected: bool,
}

impl KeyEvent {
    pub fn char_down(c: char) -> KeyEvent {
        KeyEvent {
            ch: c as u32,
            key_down: true,
            ..Default::default()
        }
    }

    pub fn key_down(vk: u32) -> KeyEvent {
        KeyEvent {
            vk,
            key_down: true,
            ..Default::default()
        }
    }

    /// Ký tự của phím đặc biệt (Space→' ', Enter→'\n', Tab→'\t'); không có → None.
    pub fn printable(&self) -> Option<char> {
        if self.ch != 0 {
            return char::from_u32(self.ch);
        }
        match self.vk {
            vk::SPACE => Some(' '),
            vk::RETURN => Some('\n'),
            vk::TAB => Some('\t'),
            _ => None,
        }
    }

    /// Chord hệ thống / modifier giữ phím — engine không được nuốt (S9, bug B6).
    pub fn is_chord(&self) -> bool {
        self.mods & (MOD_CTRL | MOD_ALT | MOD_SUPER | MOD_META) != 0
    }

    /// Phím modifier đơn lẻ (Shift/Ctrl/Alt/Super/CapsLock/Fn) — không sinh chữ.
    pub fn is_modifier(&self) -> bool {
        matches!(
            self.vk,
            vk::SHIFT | vk::CONTROL | vk::MENU | vk::LWIN | vk::RWIN | vk::CAPSLOCK
        ) && self.ch == 0
    }
}
