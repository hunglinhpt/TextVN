// SPDX-License-Identifier: GPL-3.0-or-later
//! P10 — Dream RSI (vòng 58): đo tiết kiệm phím của user nhờ prediction.
//!
//! Mọi đường accept (VN Tab sau Telex-transform, style-word/EN Tab/Space,
//! cycle Tab, next-phrase Ctrl+Space) phải tăng `accepts` và `chars_saved`
//! đúng công thức; `keys_typed` chỉ đếm phím gõ chữ thật (loại key-up,
//! injected, modifier, chord, phím điều hướng). Drain JSON phải mang khối
//! `rsi` khớp snapshot. G8: counters tổng hợp, không nội dung phím.
//!
//! Ghi chú path (probe vòng 58): tiền tố ASCII thuần không transform →
//! `try_vn_complete` bỏ qua (word chưa active) → Tab hoàn tất qua
//! `try_english_complete` với style_words (không mở cycle). Path VN thật
//! (mở cycle) cần Telex transform trước (vd "coongj" → "cộng").

use textvn_core::keymap::{self, vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

fn apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            if let Some(c) = k.printable() {
                if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                    buf.push(c);
                }
            }
        }
        Action::Replace {
            delete_count,
            insert,
        }
        | Action::Restore {
            delete_count,
            insert,
        } => {
            for _ in 0..*delete_count {
                buf.pop();
            }
            buf.extend(insert.iter().cloned());
        }
        Action::Commit { insert } => buf.extend(insert.iter().cloned()),
    }
}

fn press(e: &mut Engine, buf: &mut Vec<char>, k: &KeyEvent) {
    let out = e.key(k);
    apply(buf, &out.action, k);
}

fn letter(c: char) -> KeyEvent {
    KeyEvent::char_down(c)
}
fn space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        ch: ' ' as u32,
        key_down: true,
        ..Default::default()
    }
}
fn tab() -> KeyEvent {
    KeyEvent {
        vk: vk::TAB,
        ch: '\t' as u32,
        key_down: true,
        ..Default::default()
    }
}
fn ctrl_space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        mods: keymap::MOD_CTRL,
        key_down: true,
        ..Default::default()
    }
}

/// d100 — 100 từ cá nhân duy nhất theo tiền tố 3 chữ cái ("kqa".."zqa"):
/// Tab hoàn tất qua `try_english_complete` với style_words → accept +
/// savings tích lũy chính xác qua 100 case. Candidate nào thắng trong số
/// (từ cá nhân, tech terms…) không quan trọng với RSI — counters phải khớp
/// ghost thật; từ cá nhân đứng đầu đã có gate riêng (text_prediction).
fn sweep_style_words() -> Vec<(String, u32)> {
    const BASES: [char; 4] = ['k', 'j', 'v', 'z'];
    (0..100u32)
        .map(|i| {
            let b1 = BASES[(i / 26) as usize];
            let b2 = (b'a' + (i % 26) as u8) as char;
            (format!("{b1}q{b2}khoang"), 1)
        })
        .collect()
}

fn rsi_engine(words: Vec<(String, u32)>) -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        style_words: words,
        ..Default::default()
    })
}

/// d100 — sweep 100 case Tab-accept từ cá nhân: mỗi case +1 accept,
/// savings = ghost delta (7), keys_typed = 3 chữ + 1 Tab, tích lũy không trôi.
#[test]
fn d100_sweep_tab_accept_tich_luy() {
    let words = sweep_style_words();
    let mut e = rsi_engine(words.clone());
    let mut buf: Vec<char> = Vec::new();
    let mut exp_keys = 0u64;
    let mut exp_saved = 0u64;
    for (i, (word, _count)) in words.iter().enumerate() {
        let prefix: String = word.chars().take(3).collect();
        for c in prefix.chars() {
            press(&mut e, &mut buf, &letter(c));
        }
        exp_keys += 3;
        let ghost = e
            .peek_completion()
            .unwrap_or_else(|| panic!("case {i}: candidate hoàn tất tồn tại"));
        exp_saved += ghost.chars().count() as u64;
        press(&mut e, &mut buf, &tab());
        exp_keys += 1;
        let s = e.dream_rsi_snapshot();
        assert_eq!(s.accepts, (i + 1) as u64, "case {i}: accepts");
        assert_eq!(s.keys_typed, exp_keys, "case {i}: keys_typed");
        assert_eq!(s.chars_saved, exp_saved, "case {i}: chars_saved");
        // Văn bản cuối = prefix + ghost (Replace xoá prefix, chèn prefix+ghost).
        let tail: Vec<char> = prefix.chars().chain(ghost.chars()).collect();
        assert!(
            buf.len() >= tail.len() && buf[buf.len() - tail.len()..] == tail[..],
            "case {i}: văn bản cuối khớp candidate đã nhận"
        );
    }
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 100);
    assert_eq!(s.chars_saved, exp_saved, "tổng savings khớp tích lũy ghost");
    assert!(s.chars_saved > 100, "mỗi case đều có savings");
    assert!(s.savings_rate_pct() > 0, "savings rate phải dương");
    // Drain JSON mang khối rsi khớp snapshot (adapter persist qua tray).
    let json = e.style_drain_json();
    let want = format!(
        "\"rsi\":{{\"keys_typed\":{},\"accepts\":{},\"chars_saved\":{},\"savings_rate_pct\":{}}}",
        s.keys_typed,
        s.accepts,
        s.chars_saved,
        s.savings_rate_pct()
    );
    assert!(json.contains(&want), "drain thiếu/khớp khối rsi: {json}");
}

/// Style-word Tab-accept (path EN completion với style_words): savings =
/// candidate − đã gõ chính xác từng bước; ghost delta = phần được nối thêm.
#[test]
fn d_style_word_tab_accept_savings_dung_cong_thuc() {
    let mut e = rsi_engine(vec![("khanhpham".into(), 2)]);
    let mut buf = Vec::new();
    for c in "kha".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    let ghost = e.peek_completion().expect("ghost delta");
    assert_eq!(ghost, "nhpham", "ghost = phần nối thêm (delta)");
    press(&mut e, &mut buf, &tab());
    assert_eq!(buf.iter().collect::<String>(), "khanhpham");
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 1);
    assert_eq!(s.keys_typed, 4); // 3 chữ + Tab
    assert_eq!(s.chars_saved, 6); // 9 − 3 == ghost delta
    assert_eq!(s.savings_rate_pct(), (6 * 100) / (6 + 4));
}

/// Space-accept (suggest_space_complete): hoàn tất + chính dấu cách —
/// savings vẫn chỉ đếm ký tự từ (space là phím user đã gõ, nằm trong keys).
#[test]
fn d_space_accept_savings() {
    let mut e = Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        style_words: vec![("khanhpham".into(), 2)],
        ..Default::default()
    });
    let mut buf = Vec::new();
    for c in "kha".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    press(&mut e, &mut buf, &space());
    assert_eq!(buf.iter().collect::<String>(), "khanhpham ");
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 1);
    assert_eq!(s.keys_typed, 4); // 3 chữ + Space
    assert_eq!(s.chars_saved, 6); // completion 9 − typed 3 (không đếm space)
}

/// Path VN thật + CYCLE: "ddk" → "đk" (Telex transform, word active)
/// → Tab hoàn tất từ cá nhân; Tab kế cycle sang candidate dài hơn — savings
/// cộng thêm phần chênh (cand mới − glyphs đã chèn của cand cũ).
/// Từ đuôi ASCII thuần → restyle no-op (dấu đa âm tiết bị restyle dịch — probe vòng 58).
#[test]
fn d_vn_path_cycle_accept_savings() {
    let mut e = rsi_engine(vec![("đkkhoang".into(), 2), ("đkkhoangtv".into(), 1)]);
    let mut buf = Vec::new();
    for c in "ddk".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    assert_eq!(buf.iter().collect::<String>(), "đk", "Telex dd→đ");
    press(&mut e, &mut buf, &tab());
    assert_eq!(buf.iter().collect::<String>(), "đkkhoang");
    let s1 = e.dream_rsi_snapshot();
    assert_eq!(s1.accepts, 1);
    assert_eq!(s1.chars_saved, 6); // 8 − 2 (display "đk")
    press(&mut e, &mut buf, &tab());
    assert_eq!(buf.iter().collect::<String>(), "đkkhoangtv", "cycle Tab");
    let s2 = e.dream_rsi_snapshot();
    // Duyệt vòng vẫn là một lượt nhận (R2-147) — chỉ phần tiết kiệm thêm được cộng.
    assert_eq!(s2.accepts, 1);
    assert_eq!(s2.chars_saved, 8); // 6 + (10 − 8)
    assert_eq!(s2.keys_typed, 5); // 3 chữ + 2 Tab
                                  // Cycle KHÔNG bao giờ làm chars_saved trôi lùi (saturating).
    assert!(s2.chars_saved >= s1.chars_saved);
}

/// EN accept ("rele" + Tab → "release"): savings = 7 − 4 == ghost delta.
#[test]
fn d_en_accept_savings() {
    let mut e = rsi_engine(vec![]);
    let mut buf = Vec::new();
    for c in "rele".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    let ghost = e.peek_completion().expect("EN ghost");
    assert_eq!(ghost, "ase");
    press(&mut e, &mut buf, &tab());
    assert_eq!(buf.iter().collect::<String>(), "release");
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 1);
    assert_eq!(s.chars_saved, ghost.chars().count() as u64);
    assert!(s.chars_saved > 0);
}

/// Next-phrase accept (Ctrl+Space sau "cộng hòa " → "xã hội chủ nghĩa
/// Việt Nam"): savings = nguyên phần glyph cụm engine chèn.
#[test]
fn d_next_phrase_accept_savings() {
    let mut e = rsi_engine(vec![]);
    let mut buf = Vec::new();
    for c in "coongj hoaf ".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    assert_eq!(buf.iter().collect::<String>(), "cộng hòa ");
    let before = e.dream_rsi_snapshot();
    assert_eq!(before.accepts, 0);
    let len_before = buf.len();
    press(&mut e, &mut buf, &ctrl_space());
    assert_eq!(
        buf.iter().collect::<String>(),
        "cộng hòa xã hội chủ nghĩa Việt Nam"
    );
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 1);
    assert_eq!(
        s.chars_saved,
        (buf.len() - len_before) as u64,
        "savings = số glyph cụm chèn"
    );
    assert!(s.chars_saved > 10);
    // keys_typed = 12 phím thật của "coongj hoaf " (Ctrl+Space ch=0, không đếm).
    assert_eq!(s.keys_typed, 12);
}

/// keys_typed chỉ đếm phím gõ chữ thật: loại key-up, injected, modifier,
/// chord (Ctrl+V), phím điều hướng (ch=0).
#[test]
fn d_keys_typed_loai_phim_khong_phai_go_chu() {
    let mut e = rsi_engine(vec![]);
    let mut buf = Vec::new();
    press(&mut e, &mut buf, &letter('x'));
    press(&mut e, &mut buf, &space());
    press(&mut e, &mut buf, &tab());
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Key-up: không đếm.
    let up = KeyEvent {
        vk: vk::SPACE,
        ch: ' ' as u32,
        key_down: false,
        ..Default::default()
    };
    press(&mut e, &mut buf, &up);
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Injected (engine tự bơm): không đếm.
    let inj = KeyEvent {
        ch: 'x' as u32,
        key_down: true,
        is_injected: true,
        ..Default::default()
    };
    press(&mut e, &mut buf, &inj);
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Modifier đơn (Shift): không đếm.
    let shift = KeyEvent {
        vk: vk::SHIFT,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut buf, &shift);
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Chord Ctrl+V: không đếm (đổi text/vị trí con trỏ, không phải gõ chữ).
    let chord = KeyEvent {
        vk: 'V' as u32,
        ch: 'v' as u32,
        mods: keymap::MOD_CTRL,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut buf, &chord);
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Phím điều hướng (ch=0): không đếm.
    let nav = KeyEvent {
        vk: vk::LEFT,
        key_down: true,
        ..Default::default()
    };
    press(&mut e, &mut buf, &nav);
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 3);

    // Gõ chữ thật kế tiếp vẫn đếm đúng tiếp.
    press(&mut e, &mut buf, &letter('y'));
    assert_eq!(e.dream_rsi_snapshot().keys_typed, 4);
}

/// Engine mới = counters 0; savings_rate 0 khi chưa có dữ liệu (không chia 0).
#[test]
fn d_engine_moi_zero_va_no_div() {
    let e = rsi_engine(vec![]);
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.keys_typed, 0);
    assert_eq!(s.accepts, 0);
    assert_eq!(s.chars_saved, 0);
    assert_eq!(s.savings_rate_pct(), 0);
}

/// DreamRsi::savings_rate_pct — công thức + biên saturating (không panic).
#[test]
fn d_savings_rate_cong_thuc() {
    use textvn_core::DreamRsi;
    assert_eq!(DreamRsi::default().savings_rate_pct(), 0);
    assert_eq!(
        DreamRsi {
            keys_typed: 100,
            accepts: 0,
            chars_saved: 0
        }
        .savings_rate_pct(),
        0
    );
    assert_eq!(
        DreamRsi {
            keys_typed: 90,
            accepts: 10,
            chars_saved: 10
        }
        .savings_rate_pct(),
        10
    );
    assert_eq!(
        DreamRsi {
            keys_typed: 0,
            accepts: 1,
            chars_saved: 25
        }
        .savings_rate_pct(),
        100
    );
    // Biên u64: total saturate về MAX → MAX/MAX = 1 (không panic, không 100).
    let big = DreamRsi {
        keys_typed: u64::MAX,
        accepts: 1,
        chars_saved: u64::MAX,
    };
    assert_eq!(big.savings_rate_pct(), 1);
}

/// G8 — snapshot/drain không mang nội dung: chỉ counter tổng hợp.
#[test]
fn d_g8_chi_counter_khong_noi_dung() {
    let mut e = rsi_engine(vec![("khanhpham".into(), 2)]);
    let mut buf = Vec::new();
    for c in "kha".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    press(&mut e, &mut buf, &tab());
    let s = e.dream_rsi_snapshot();
    // Debug của DreamRsi chỉ có 3 trường counter.
    let dbg = format!("{s:?}");
    assert!(dbg.contains("keys_typed") && dbg.contains("accepts") && dbg.contains("chars_saved"));
    assert!(!dbg.contains("khanh"), "G8: không lộ từ: {dbg}");
}

/// Drain JSON tổng: words/corrections và rsi cùng tồn tại — tray parse
/// (DrainShape) phải bỏ qua rsi an toàn, không chết merge.
#[test]
fn d_drain_json_words_va_rsi_song_chung() {
    let mut e = Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        style_learning_enabled: true,
        style_words: vec![("khanhpham".into(), 2)],
        ..Default::default()
    });
    let mut buf = Vec::new();
    for c in "triển".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    press(&mut e, &mut buf, &space());
    for c in "kha".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    press(&mut e, &mut buf, &tab());
    // Ứng viên được học khi người dùng chốt bằng phím kế tiếp (R2-147).
    press(&mut e, &mut buf, &space());
    let json = e.style_drain_json();
    assert!(json.contains("\"word\":\"triển\""), "word-end học: {json}");
    assert!(
        json.contains("\"word\":\"khanhpham\""),
        "accept học: {json}"
    );
    assert!(
        json.contains("\"rsi\":{\"keys_typed\":"),
        "rsi block: {json}"
    );
    assert!(json.contains("\"savings_rate_pct\":"), "rsi rate: {json}");
    // JSON hợp lệ (serde parse — cùng đường tray sẽ đi).
    let v: serde_json::Value = serde_json::from_str(&json).expect("drain JSON hợp lệ");
    assert!(v["rsi"]["accepts"].as_u64().is_some());
}

/// 100 phím gõ tay KHÔNG accept nào → savings rate 0 (metrics trung thực:
/// prediction không đoán trúng thì rate phải phản ánh đúng).
#[test]
fn d_khong_accept_thi_rate_0() {
    let mut e = rsi_engine(vec![]);
    let mut buf = Vec::new();
    for c in "xin chao the gioi van phong bo go textvn".chars() {
        press(&mut e, &mut buf, &letter(c));
    }
    let s = e.dream_rsi_snapshot();
    assert_eq!(s.accepts, 0, "không Tab/Space-complete nào");
    assert_eq!(s.chars_saved, 0);
    assert_eq!(s.savings_rate_pct(), 0);
    assert!(s.keys_typed > 30);
}
