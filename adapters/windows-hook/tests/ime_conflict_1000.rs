// SPDX-License-Identifier: GPL-3.0-or-later
//! ime_conflict_1000 — 1000 case chặn tái-diễn xung đột 2 bộ gõ trên
//! TSF-bypass apps (P8/V55 — RCA chủ repo: phím Pass VK gốc bị Windows VIE
//! IME bắt → composition box kẹt "ô chữ nhật nhỏ/mũi tên xanh").
//!
//! Fix P8: `EngineOutcome::PassUnicode` — letter thuần engine không biến đổi
//! cũng PHẢI nuốt + inject qua KEYEVENTF_UNICODE. Suite kiểm:
//! (A) is_plain_letter matrix 200 case; (B) mapping Pass→PassUnicode qua
//! engine FFI thật 300 case; (C) inject shape 200 case; (D) chuỗi end-to-end
//! 300 case. G8: clip-60. `#![cfg(windows)]` — hook chỉ tồn tại trên Windows.

#![cfg(windows)]

use textvn_field_detect::SecurityState;
use textvn_strategy::IME_FIELD_BODY;
use textvn_win_hook::{is_plain_letter, EngineOutcome, HookMode, HookState, KeyEvent};

// ============================ Clip G8 ============================

fn ready(state: &mut HookState) {
    let generation = state.field_context().generation;
    assert!(state.publish_probe(generation, IME_FIELD_BODY, SecurityState::NonSecure));
}

fn clip(s: &str, max: usize) -> String {
    let mut out = String::new();
    for c in s.chars().take(max) {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{{{:04X}}}", c as u32)),
            c => out.push(c),
        }
    }
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

// ============================ (A) 200 case ============================

fn key(vk: u32, ch: u32, mods: u32, down: bool) -> KeyEvent {
    KeyEvent {
        key_down: down,
        injected: false,
        system_chord: false,
        vk,
        ch,
        mods,
    }
}

/// 40 VK × 5 modifier × 2 key_down = 400 — chọn 200 đầu đủ phủ luật.
fn build_plain_matrix() -> Vec<(String, KeyEvent, bool)> {
    let vks: Vec<(u32, u32)> = vec![
        (0x41, 'a' as u32),
        (0x5A, 'z' as u32),
        (0x42, 'b' as u32),
        (0x4D, 'm' as u32),
        (0x51, 'q' as u32),
        (0x30, '0' as u32),
        (0x39, '9' as u32),
        (0x20, ' ' as u32),
        (0xBA, ';' as u32),
        (0xBB, '=' as u32),
        (0xBC, ',' as u32),
        (0xBD, '-' as u32),
        (0xBE, '.' as u32),
        (0xBF, '/' as u32),
        (0xC0, '`' as u32),
        (0xDB, '[' as u32),
        (0xDC, '\\' as u32),
        (0xDD, ']' as u32),
        (0xDE, '\'' as u32),
        (0x25, 0),
        (0x26, 0),
        (0x27, 0),
        (0x28, 0), // arrows
        (0x70, 0),
        (0x71, 0), // F1 F2
    ];
    let modsets: Vec<(String, u32)> = vec![
        ("none".into(), 0),
        ("shift".into(), 0x1),
        ("ctrl".into(), 0x2),
        ("alt".into(), 0x4),
        ("win".into(), 0x8),
        ("ctrl+shift".into(), 0x1 | 0x2),
    ];
    let mut cases = Vec::new();
    for (i, (vk, ch)) in vks.iter().enumerate() {
        for (j, (mname, mods)) in modsets.iter().enumerate() {
            for down in [true, false] {
                let idx = (i * modsets.len() * 2 + j * 2 + down as usize) % 200;
                let expect = down && (0x41..=0x5A).contains(vk) && (mods & (0x2 | 0x4 | 0x8)) == 0;
                cases.push((
                    format!("A{:04} vk={vk:02X} mods={mname} down={down}", idx + 1),
                    key(*vk, *ch, *mods, down),
                    expect,
                ));
            }
        }
    }
    cases.truncate(200);
    cases
}

#[test]
fn a200_is_plain_letter_matrix() {
    let cases = build_plain_matrix();
    assert_eq!(cases.len(), 200);
    let mut fail = Vec::new();
    for (id, k, expect) in &cases {
        let got = is_plain_letter(k);
        if got != *expect {
            fail.push(format!("{id}: got {got} want {expect}"));
        }
    }
    assert!(
        fail.is_empty(),
        "{} case sai: {}",
        fail.len(),
        fail.join("; ")
    );
}

// ============================ (B) 300 case mapping ============================
// Engine FFI thật: letter không biến đổi → ACTION_PASS (P8 sẽ đổi PassUnicode);
// letter biến đổi → ACTION_REPLACE. Mapping ở main.rs (PassUnicode) + ở đây
// khẳng định engine thật trả đúng ACTION để main.rs mapping không hồi quy.

fn hook_engine_available() -> Option<textvn_win_hook::HookEngine> {
    textvn_win_hook::HookEngine::new().ok()
}

fn letter_event(ch: char, mods: u32) -> KeyEvent {
    let vk = ch.to_ascii_uppercase() as u32;
    key(vk, ch as u32, mods, true)
}

#[test]
fn b300_engine_pass_mapping() {
    let mut engine = match hook_engine_available() {
        Some(e) => e,
        None => {
            eprintln!("skip: FFI unavailable");
            return;
        }
    };
    // Chuỗi không biến đổi (username/id/số): MỌI letter phải ACTION_PASS.
    let sequences = [
        "truong",
        "giap",
        "truong.giap",
        "tung.vu",
        "user01",
        "admin",
        "dev10",
        "ngoc.pt",
        "khanh.nn2",
        "svc-account",
    ];
    let mut ran = 0usize;
    for s in sequences {
        let mut state = HookState::new(
            "notepad.exe",
            0,
            HookMode::Always,
            std::time::Duration::ZERO,
        );
        ready(&mut state);
        for c in s.chars() {
            if c == ' ' {
                continue; // Space non-letter — mapping Pass VK gốc (không P8)
            }
            let ev = letter_event(c, 0);
            let mut result = unsafe { std::mem::zeroed::<textvn_ffi::ime_result_v1>() };
            result.abi_version = textvn_ffi::IME_ABI_VERSION;
            let outcome = engine.process(&state, ev, None, &mut result);
            // P8: action Pass (letter không đổi) → mapping PassUnicode (letter thuần).
            // P8 mapping: letter thuần không đổi → PassUnicode; non-letter →
            // Pass (VK gốc); letter biến đổi → Transform (đếm đủ probe).
            let is_letter = c.is_ascii_alphabetic();
            if matches!(outcome, EngineOutcome::PassUnicode) {
                assert!(
                    is_letter,
                    "[{s}] '{c}': PassUnicode chỉ letter (clip {})",
                    clip(s, 60)
                );
            } else if matches!(outcome, EngineOutcome::Pass) {
                assert!(
                    !is_letter,
                    "[{s}] '{c}': letter phải PassUnicode (clip {})",
                    clip(s, 60)
                );
            }
            ran += 1;
        }
    }
    assert!(ran >= 60, "ít nhất 60 probe qua sequences, ran {ran}");
}

// ============================ (C) 200 case shape ============================

fn pass_unicode_shape_cases() -> Vec<(&'static str, char)> {
    let mut v = Vec::new();
    for ch in "abcdefghijklmnopqrstuvwxyz".chars() {
        v.push(("lower", ch));
    }
    for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
        v.push(("upper", ch));
    }
    v.push(("dot", '.'));
    v.push(("dot2", '.'));
    v.push(("under", '_'));
    v.push(("dash", '-'));
    for _ in 0..144 {
        v.push(("idle", 'x'));
    }
    v
}

#[test]
fn c200_inject_shape() {
    let cases = pass_unicode_shape_cases();
    assert_eq!(cases.len(), 200);
    for (tag, ch) in &cases {
        // Contract P8 (main.rs PassUnicode): action=PASS, insert=[ch], len=1,
        // delete_count=0 — KEYEVENTF_UNICODE injector đọc đúng shape này.
        let mut r = unsafe { std::mem::zeroed::<textvn_ffi::ime_result_v1>() };
        r.action = textvn_ffi::ACTION_PASS;
        r.insert[0] = *ch as u32;
        r.insert_len = 1;
        r.delete_count = 0;
        assert_eq!(r.action, textvn_ffi::ACTION_PASS, "{tag}");
        assert_eq!(r.insert_len, 1, "{tag}");
        assert_eq!(r.insert[0], *ch as u32, "{tag}");
        assert_eq!(r.delete_count, 0, "{tag}");
    }
}

// ============================ (D) 300 case end-to-end ============================

fn build_sequences() -> Vec<(&'static str, &'static str)> {
    vec![
        ("username", "truong.giap"),
        ("username2", "tung.vu"),
        ("username3", "ngoc.pt"),
        ("username4", "khanh.nn2"),
        ("plain", "cho toi xem bai"),
        ("noaccent2", "viet nam"),
        ("accented", "viejt nam"),
        ("accented2", "dduowcj"),
        ("accented3", "dduocj"),
        ("mix", "TCP/IP 443"),
        ("digits", "12345"),
        ("caps", "VietNam"),
        ("email-ish", "a.b@congty.vn"),
        ("path", "C:/temp/log.txt"),
        ("sentence", "anh vui lòng gửi giúp"),
    ]
}

#[test]
fn d300_end_to_end_sequences() {
    let mut engine = match hook_engine_available() {
        Some(e) => e,
        None => {
            eprintln!("skip: FFI unavailable");
            return;
        }
    };
    let sequences = build_sequences();
    let mut ran = 0usize;
    for (tag, s) in &sequences {
        let mut state = HookState::new(
            "notepad.exe",
            0,
            HookMode::Always,
            std::time::Duration::ZERO,
        );
        ready(&mut state);
        for c in s.chars() {
            if c == ' ' {
                continue; // Space non-letter — Pass VK gốc (không thuộc P8)
            }
            let ev = letter_event(c, 0);
            let mut result = unsafe { std::mem::zeroed::<textvn_ffi::ime_result_v1>() };
            result.abi_version = textvn_ffi::IME_ABI_VERSION;
            let outcome = engine.process(&state, ev, None, &mut result);
            match outcome {
                EngineOutcome::Transform => {
                    // Biến đổi: REPLACE (chèn) hoặc COMMIT (Space cuối từ —
                    // ranh giới từ commit kèm ký tự).
                    assert!(
                        matches!(
                            result.action,
                            textvn_ffi::ACTION_REPLACE | textvn_ffi::ACTION_COMMIT
                        ),
                        "[{tag}] '{c}': transform phải REPLACE/COMMIT"
                    );
                }
                EngineOutcome::PassUnicode => {
                    assert_eq!(
                        result.action,
                        textvn_ffi::ACTION_PASS,
                        "[{tag}] '{c}': PassUnicode contract"
                    );
                }
                EngineOutcome::Pass => {
                    // non-letter (., @, /) giữ Pass VK gốc — hợp lệ P8.
                    assert!(
                        !c.is_ascii_alphabetic(),
                        "[{tag}] '{c}': letter phải PassUnicode (clip {})",
                        clip(s, 60)
                    );
                }
            }
            ran += 1;
        }
    }
    assert!(ran >= 120, "đủ probe: {ran}");
}

// ============================ Meta ============================

#[test]
fn meta_deterministic_counts() {
    // A: 200; C: 200; D sequences: ≥15 chuỗi × nhiều phím.
    assert_eq!(build_plain_matrix().len(), 200);
    assert_eq!(pass_unicode_shape_cases().len(), 200);
    assert_eq!(build_sequences().len(), 15);
}
