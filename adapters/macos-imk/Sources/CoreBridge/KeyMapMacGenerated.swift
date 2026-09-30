// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/tables/keymap_mac.toml · sinh bằng `cargo xtask gen-tables` (P0-1 §3).
// Đổi bảng: sửa file `.toml` rồi chạy lại `cargo xtask gen-tables`.
// `cargo xtask check-tables` (CI) sẽ fail nếu file này lệch với nguồn.

/// Nguồn: `data/tables/keymap_mac.toml` (digest FNV-1a 64 = 0x6a6f2f15251c8911).
/// Keycode macOS (Carbon `kVK_*`) → VK canonical Windows (P0-2 §1).
/// Phím không có trong bảng → nil → adapter PASS (không đoán).
/// `public`: dùng từ module IMKLib, khác module CoreBridge (MAC-032).
public enum KeyMapMacGenerated {
    public static let keyCount = 92

    public static func canonicalVK(_ kvk: UInt32) -> UInt32? {
        switch kvk {
        case 0x00: return 0x41 // ANSI_A
        case 0x01: return 0x53 // ANSI_S
        case 0x02: return 0x44 // ANSI_D
        case 0x03: return 0x46 // ANSI_F
        case 0x04: return 0x48 // ANSI_H
        case 0x05: return 0x47 // ANSI_G
        case 0x06: return 0x5a // ANSI_Z
        case 0x07: return 0x58 // ANSI_X
        case 0x08: return 0x43 // ANSI_C
        case 0x09: return 0x56 // ANSI_V
        case 0x0b: return 0x42 // ANSI_B
        case 0x0c: return 0x51 // ANSI_Q
        case 0x0d: return 0x57 // ANSI_W
        case 0x0e: return 0x45 // ANSI_E
        case 0x0f: return 0x52 // ANSI_R
        case 0x10: return 0x59 // ANSI_Y
        case 0x11: return 0x54 // ANSI_T
        case 0x1f: return 0x4f // ANSI_O
        case 0x20: return 0x55 // ANSI_U
        case 0x22: return 0x49 // ANSI_I
        case 0x23: return 0x50 // ANSI_P
        case 0x25: return 0x4c // ANSI_L
        case 0x26: return 0x4a // ANSI_J
        case 0x28: return 0x4b // ANSI_K
        case 0x2d: return 0x4e // ANSI_N
        case 0x2e: return 0x4d // ANSI_M
        case 0x12: return 0x31 // ANSI_1
        case 0x13: return 0x32 // ANSI_2
        case 0x14: return 0x33 // ANSI_3
        case 0x15: return 0x34 // ANSI_4
        case 0x17: return 0x35 // ANSI_5
        case 0x16: return 0x36 // ANSI_6
        case 0x1a: return 0x37 // ANSI_7
        case 0x1c: return 0x38 // ANSI_8
        case 0x19: return 0x39 // ANSI_9
        case 0x1d: return 0x30 // ANSI_0
        case 0x18: return 0xbb // ANSI_Equal
        case 0x1b: return 0xbd // ANSI_Minus
        case 0x21: return 0xdb // ANSI_LeftBracket
        case 0x1e: return 0xdd // ANSI_RightBracket
        case 0x2a: return 0xdc // ANSI_Backslash
        case 0x29: return 0xba // ANSI_Semicolon
        case 0x27: return 0xde // ANSI_Quote
        case 0x2b: return 0xbc // ANSI_Comma
        case 0x2f: return 0xbe // ANSI_Period
        case 0x2c: return 0xbf // ANSI_Slash
        case 0x32: return 0xc0 // ANSI_Grave
        case 0x0a: return 0xe2 // ISO_Section
        case 0x24: return 0x0d // Return
        case 0x30: return 0x09 // Tab
        case 0x31: return 0x20 // Space
        case 0x33: return 0x08 // Delete
        case 0x75: return 0x2e // ForwardDelete
        case 0x35: return 0x1b // Escape
        case 0x4c: return 0x0d // KeypadEnter
        case 0x37: return 0x5b // Command
        case 0x36: return 0x5c // RightCommand
        case 0x38: return 0x10 // Shift
        case 0x3c: return 0x10 // RightShift
        case 0x3a: return 0x12 // Option
        case 0x3d: return 0x12 // RightOption
        case 0x3b: return 0x11 // Control
        case 0x3e: return 0x11 // RightControl
        case 0x39: return 0x14 // CapsLock
        case 0x7b: return 0x25 // LeftArrow
        case 0x7c: return 0x27 // RightArrow
        case 0x7d: return 0x28 // DownArrow
        case 0x7e: return 0x26 // UpArrow
        case 0x73: return 0x24 // Home
        case 0x77: return 0x23 // End
        case 0x74: return 0x21 // PageUp
        case 0x79: return 0x22 // PageDown
        case 0x7a: return 0x70 // F1
        case 0x78: return 0x71 // F2
        case 0x63: return 0x72 // F3
        case 0x76: return 0x73 // F4
        case 0x60: return 0x74 // F5
        case 0x61: return 0x75 // F6
        case 0x62: return 0x76 // F7
        case 0x64: return 0x77 // F8
        case 0x65: return 0x78 // F9
        case 0x6d: return 0x79 // F10
        case 0x67: return 0x7a // F11
        case 0x6f: return 0x7b // F12
        case 0x69: return 0x7c // F13
        case 0x6b: return 0x7d // F14
        case 0x71: return 0x7e // F15
        case 0x6a: return 0x7f // F16
        case 0x40: return 0x80 // F17
        case 0x4f: return 0x81 // F18
        case 0x50: return 0x82 // F19
        case 0x5a: return 0x83 // F20
        default: return nil
        }
    }
}
