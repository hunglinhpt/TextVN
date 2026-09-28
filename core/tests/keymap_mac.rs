// SPDX-License-Identifier: GPL-3.0-or-later
//! Test bảng keycode macOS → VK canonical (MAC-013, P2-1 §5).
//!
//! Bảng sinh từ `data/tables/keymap_mac.toml` — test này chốt **hành vi**:
//! giá trị kỳ vọng là VK canonical mà engine/`is_modifier`/`is_chord` dựa vào.
//! Đổi bảng ra kết quả khác ở đây = đổi hành vi adapter mac → phải conscious.

use textvn_core::keymap_mac_generated::{mac_to_canonical, MAC_KEY_COUNT};

#[test]
fn chu_cai_va_so_dung_canonical() {
    // Con trỏ gõ: chữ hoa ASCII (Win VK) — replay/rely trên contract này.
    for (kvk, expect) in [
        (0x00, 'A'), // kVK_ANSI_A
        (0x0D, 'W'), // kVK_ANSI_W
        (0x1F, 'O'),
        (0x2E, 'M'), // kVK_ANSI_M
    ] {
        assert_eq!(mac_to_canonical(kvk), Some(expect as u32), "kvk {kvk:#x}");
    }
    for (kvk, expect) in [
        (0x12, '1'),
        (0x13, '2'),
        (0x14, '3'),
        (0x15, '4'),
        (0x16, '6'),
        (0x17, '5'), // 6 trước 5 theo Events.h — chốt thứ tự đặc trưng Apple
        (0x1A, '7'),
        (0x1C, '8'),
        (0x19, '9'),
        (0x1D, '0'),
    ] {
        assert_eq!(mac_to_canonical(kvk), Some(expect as u32), "kvk {kvk:#x}");
    }
}

#[test]
fn phim_dieu_khien_loi_dung_win_vk() {
    // Engine special-case các VK này (BACK/ESCAPE/RETURN/SPACE/TAB + arrows).
    assert_eq!(mac_to_canonical(0x24), Some(0x0D)); // Return
    assert_eq!(mac_to_canonical(0x31), Some(0x20)); // Space
    assert_eq!(mac_to_canonical(0x33), Some(0x08)); // Delete = Backspace
    assert_eq!(mac_to_canonical(0x75), Some(0x2E)); // ForwardDelete ≠ Backspace
    assert_eq!(mac_to_canonical(0x35), Some(0x1B)); // Escape
    assert_eq!(mac_to_canonical(0x30), Some(0x09)); // Tab
    assert_eq!(mac_to_canonical(0x7B), Some(0x25)); // Left
    assert_eq!(mac_to_canonical(0x7C), Some(0x27)); // Right
    assert_eq!(mac_to_canonical(0x7D), Some(0x28)); // Down
    assert_eq!(mac_to_canonical(0x7E), Some(0x26)); // Up
}

#[test]
fn modifier_dung_vk_is_modifier_nhan_duoc() {
    use textvn_core::keymap::KeyEvent;
    // trái + phải về cùng VK — KeyEvent::is_modifier dựa trên VK này.
    for kvk in [0x38, 0x3C] {
        assert_eq!(mac_to_canonical(kvk), Some(0x10), "Shift {kvk:#x}");
    }
    for kvk in [0x3B, 0x3E] {
        assert_eq!(mac_to_canonical(kvk), Some(0x11), "Control {kvk:#x}");
    }
    for kvk in [0x3A, 0x3D] {
        assert_eq!(mac_to_canonical(kvk), Some(0x12), "Option {kvk:#x}");
    }
    assert_eq!(mac_to_canonical(0x37), Some(0x5B)); // Cmd → LWIN
    assert_eq!(mac_to_canonical(0x36), Some(0x5C)); // RightCmd → RWIN
    assert_eq!(mac_to_canonical(0x39), Some(0x14)); // CapsLock
    for kvk in [0x38u32, 0x3B, 0x3A, 0x37, 0x39] {
        let ev = KeyEvent {
            vk: mac_to_canonical(kvk).unwrap(),
            ch: 0,
            ..Default::default()
        };
        assert!(ev.is_modifier(), "kvk {kvk:#x} phải là modifier");
    }
}

#[test]
fn f_key_va_phim_khong_map() {
    assert_eq!(mac_to_canonical(0x7A), Some(0x70)); // F1
    assert_eq!(mac_to_canonical(0x6F), Some(0x7B)); // F12
                                                    // Không map → None → adapter PASS (keypad/media/Function…).
    for kvk in [0x3Fu32, 0x41, 0x49, 0x48, 0x51, 0x52, 0x53, 0x54] {
        assert_eq!(mac_to_canonical(kvk), None, "kvk {kvk:#x} phải không map");
    }
}

#[test]
fn bang_khong_vi_pham_kich_thuoc() {
    // 92 entry hiện tại; biên dưới chỉ chặn "bảng bị xoá trắng".
    assert!(MAC_KEY_COUNT >= 80, "bảng quá ít: {MAC_KEY_COUNT}");
    // Quét toàn dải keycode 0..=0x7F: không panic, không map sai kiểu.
    let mapped: Vec<u32> = (0u32..=0x7F).filter_map(mac_to_canonical).collect();
    assert_eq!(mapped.len(), MAC_KEY_COUNT);
    assert!(mapped.iter().all(|&vk| (1..=0xFFFF).contains(&vk)));
}
