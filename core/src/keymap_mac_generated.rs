// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/tables/*.toml · sinh bằng `cargo xtask gen-tables` (P0-1 §3).
// Đổi bảng: sửa file `.toml` rồi chạy lại `cargo xtask gen-tables`.
// `cargo xtask check-tables` (CI) sẽ fail nếu file này lệch với nguồn.
//! Nguồn: `data/tables/keymap_mac.toml` (digest FNV-1a 64 = `0x1061a2f78ed6d967`).
//!
//! Keycode macOS (`kVK_*` Carbon) → VK canonical Windows (`keymap::vk`).
//! Adapter macOS phải chuẩn hóa về đây trước khi gọi engine (P0-2 §1).
//! Phím không có trong bảng → `None` → adapter PASS (không đoán).

/// Số keycode đã map trong bảng.
pub const MAC_KEY_COUNT: usize = 92;

/// `kVK_*` → VK canonical; `None` = không map (engine coi như phím lạ → PASS).
pub fn mac_to_canonical(kvk: u32) -> Option<u32> {
    match kvk {
        0x00 => Some(0x41), // ANSI_A
        0x01 => Some(0x53), // ANSI_S
        0x02 => Some(0x44), // ANSI_D
        0x03 => Some(0x46), // ANSI_F
        0x04 => Some(0x48), // ANSI_H
        0x05 => Some(0x47), // ANSI_G
        0x06 => Some(0x5a), // ANSI_Z
        0x07 => Some(0x58), // ANSI_X
        0x08 => Some(0x43), // ANSI_C
        0x09 => Some(0x56), // ANSI_V
        0x0b => Some(0x42), // ANSI_B
        0x0c => Some(0x51), // ANSI_Q
        0x0d => Some(0x57), // ANSI_W
        0x0e => Some(0x45), // ANSI_E
        0x0f => Some(0x52), // ANSI_R
        0x10 => Some(0x59), // ANSI_Y
        0x11 => Some(0x54), // ANSI_T
        0x1f => Some(0x4f), // ANSI_O
        0x20 => Some(0x55), // ANSI_U
        0x22 => Some(0x49), // ANSI_I
        0x23 => Some(0x50), // ANSI_P
        0x25 => Some(0x4c), // ANSI_L
        0x26 => Some(0x4a), // ANSI_J
        0x28 => Some(0x4b), // ANSI_K
        0x2d => Some(0x4e), // ANSI_N
        0x2e => Some(0x4d), // ANSI_M
        0x12 => Some(0x31), // ANSI_1
        0x13 => Some(0x32), // ANSI_2
        0x14 => Some(0x33), // ANSI_3
        0x15 => Some(0x34), // ANSI_4
        0x17 => Some(0x35), // ANSI_5
        0x16 => Some(0x36), // ANSI_6
        0x1a => Some(0x37), // ANSI_7
        0x1c => Some(0x38), // ANSI_8
        0x19 => Some(0x39), // ANSI_9
        0x1d => Some(0x30), // ANSI_0
        0x18 => Some(0xbb), // ANSI_Equal
        0x1b => Some(0xbd), // ANSI_Minus
        0x21 => Some(0xdb), // ANSI_LeftBracket
        0x1e => Some(0xdd), // ANSI_RightBracket
        0x2a => Some(0xdc), // ANSI_Backslash
        0x29 => Some(0xba), // ANSI_Semicolon
        0x27 => Some(0xde), // ANSI_Quote
        0x2b => Some(0xbc), // ANSI_Comma
        0x2f => Some(0xbe), // ANSI_Period
        0x2c => Some(0xbf), // ANSI_Slash
        0x32 => Some(0xc0), // ANSI_Grave
        0x0a => Some(0xe2), // ISO_Section
        0x24 => Some(0x0d), // Return
        0x30 => Some(0x09), // Tab
        0x31 => Some(0x20), // Space
        0x33 => Some(0x08), // Delete
        0x75 => Some(0x2e), // ForwardDelete
        0x35 => Some(0x1b), // Escape
        0x4c => Some(0x0d), // KeypadEnter
        0x37 => Some(0x5b), // Command
        0x36 => Some(0x5c), // RightCommand
        0x38 => Some(0x10), // Shift
        0x3c => Some(0x10), // RightShift
        0x3a => Some(0x12), // Option
        0x3d => Some(0x12), // RightOption
        0x3b => Some(0x11), // Control
        0x3e => Some(0x11), // RightControl
        0x39 => Some(0x14), // CapsLock
        0x7b => Some(0x25), // LeftArrow
        0x7c => Some(0x27), // RightArrow
        0x7d => Some(0x28), // DownArrow
        0x7e => Some(0x26), // UpArrow
        0x73 => Some(0x24), // Home
        0x77 => Some(0x23), // End
        0x74 => Some(0x21), // PageUp
        0x79 => Some(0x22), // PageDown
        0x7a => Some(0x70), // F1
        0x78 => Some(0x71), // F2
        0x63 => Some(0x72), // F3
        0x76 => Some(0x73), // F4
        0x60 => Some(0x74), // F5
        0x61 => Some(0x75), // F6
        0x62 => Some(0x76), // F7
        0x64 => Some(0x77), // F8
        0x65 => Some(0x78), // F9
        0x6d => Some(0x79), // F10
        0x67 => Some(0x7a), // F11
        0x6f => Some(0x7b), // F12
        0x69 => Some(0x7c), // F13
        0x6b => Some(0x7d), // F14
        0x71 => Some(0x7e), // F15
        0x6a => Some(0x7f), // F16
        0x40 => Some(0x80), // F17
        0x4f => Some(0x81), // F18
        0x50 => Some(0x82), // F19
        0x5a => Some(0x83), // F20
        _ => None,
    }
}
