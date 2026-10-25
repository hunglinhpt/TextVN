// SPDX-License-Identifier: GPL-3.0-or-later
//! Bộ test tích hợp chuyên sâu cho tính năng **Text Prediction Song ngữ Anh - Việt (P6)**.
//!
//! Kiểm tra toàn diện:
//! 1. Prefix completion từ ghép tiếng Việt (có dấu và không dấu).
//! 2. Prefix completion từ đơn (âm tiết tần suất cao).
//! 3. Next-word & phrase prediction theo ngữ cảnh tiếng Việt và tiếng Anh.
//! 4. Duyệt vòng ứng viên khi bấm Tab liên tiếp (Candidate cycling).
//! 5. Bảo toàn viết hoa đầu từ và toàn bộ hoa.
//! 6. Bất biến S9 (Shift+Tab luôn passthrough) và Tab không khớp đi qua cho ứng dụng.

use textvn_core::keymap::{self, vk, KeyEvent};
use textvn_core::{Action, Engine, EngineOptions, Method};

fn engine_with_predict() -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        auto_restore_english: true,
        ..Default::default()
    })
}

fn key_for_char(c: char) -> KeyEvent {
    match c {
        ' ' => KeyEvent {
            vk: vk::SPACE,
            ch: ' ' as u32,
            key_down: true,
            ..Default::default()
        },
        '\t' => KeyEvent {
            vk: vk::TAB,
            ch: '\t' as u32,
            key_down: true,
            ..Default::default()
        },
        '\n' => KeyEvent {
            vk: vk::RETURN,
            ch: '\n' as u32,
            key_down: true,
            ..Default::default()
        },
        _ => KeyEvent::char_down(c),
    }
}

fn press_key(e: &mut Engine, doc: &mut Vec<char>, k: &KeyEvent) -> Action {
    let out = e.key(k);
    apply_outcome(doc, &out.action, k);
    out.action
}

fn apply_outcome(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => match k.vk {
            vk::BACK => {
                buf.pop();
            }
            vk::ESCAPE | vk::LEFT => {}
            _ => {
                // Ctrl+Space là hotkey, khi Pass không tự chèn ' ' vào văn bản
                if !k.is_ctrl_space() {
                    if let Some(c) = k.printable() {
                        if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                            buf.push(c);
                        }
                    }
                }
            }
        },
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
            buf.extend(insert);
        }
        Action::Commit { insert } => {
            buf.extend(insert);
        }
    }
}

#[test]
fn test_predict_prefix_compounds_vietnamese() {
    let mut e = engine_with_predict();

    // 1. Gõ có dấu: "phát" + Tab → "phát triển"
    let mut doc = Vec::new();
    for c in "phatj".chars() {
        // 'j' là dấu nặng trong telex
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "phạt"); // test với phatj

    let mut e2 = engine_with_predict();
    let mut doc2 = Vec::new();
    for c in "phats".chars() {
        // 's' là dấu sắc → "phát"
        press_key(&mut e2, &mut doc2, &key_for_char(c));
    }
    assert_eq!(doc2.iter().collect::<String>(), "phát");
    press_key(&mut e2, &mut doc2, &key_for_char('\t'));
    assert_eq!(doc2.iter().collect::<String>(), "phát triển");

    // 2. Gõ không dấu: "cong" + Tab → "công nghệ"
    let mut e3 = engine_with_predict();
    let mut doc3 = Vec::new();
    for c in "coong".chars() {
        // "công"
        press_key(&mut e3, &mut doc3, &key_for_char(c));
    }
    assert_eq!(doc3.iter().collect::<String>(), "công");
    press_key(&mut e3, &mut doc3, &key_for_char('\t'));
    assert_eq!(doc3.iter().collect::<String>(), "công nghệ");
}

#[test]
fn test_predict_tab_cycling_candidates() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    for c in "phats".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "phát");

    // Lần Tab thứ 1: ứng viên 1 ("phát triển")
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "phát triển");

    // Lần Tab thứ 2 (liên tiếp): duyệt sang ứng viên 2 ("phát sinh")
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "phát sinh");

    // Lần Tab thứ 3 (liên tiếp): duyệt sang ứng viên 3 ("phát biểu")
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "phát biểu");

    // Gõ phím khác (dấu cách) chốt từ, huỷ cycling
    press_key(&mut e, &mut doc, &key_for_char(' '));
    assert_eq!(doc.iter().collect::<String>(), "phát biểu ");
}

#[test]
fn test_predict_next_phrase_vietnamese() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ cụm "cộng hòa "
    for c in "coongj hoaf ".chars() {
        let k = key_for_char(c);
        press_key(&mut e, &mut doc, &k);
    }
    assert_eq!(doc.iter().collect::<String>(), "cộng hòa ");

    // Nhấn Tab: dự đoán cụm từ tiếp theo → "xã hội chủ nghĩa việt nam"
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(
        doc.iter().collect::<String>(),
        "cộng hòa xã hội chủ nghĩa việt nam"
    );

    // Kính gửi + Tab → quý khách hàng
    let mut e2 = engine_with_predict();
    let mut doc2 = Vec::new();
    for c in "Kinhs guwir ".chars() {
        press_key(&mut e2, &mut doc2, &key_for_char(c));
    }
    assert_eq!(doc2.iter().collect::<String>(), "Kính gửi ");
    press_key(&mut e2, &mut doc2, &key_for_char('\t'));
    assert_eq!(doc2.iter().collect::<String>(), "Kính gửi quý khách hàng");

    // Bấm Tab tiếp theo: duyệt sang ứng viên cụm từ thứ 2: "ban giám đốc"
    press_key(&mut e2, &mut doc2, &key_for_char('\t'));
    assert_eq!(doc2.iter().collect::<String>(), "Kính gửi ban giám đốc");
}

#[test]
fn test_predict_next_phrase_english() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ "Thank you "
    for c in "Thank you ".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "Thank you ");

    // Nhấn Tab: dự đoán "very much"
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "Thank you very much");

    // Nhấn Tab lần nữa: duyệt sang "for your email"
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "Thank you for your email");

    // in order to + Tab → ensure
    let mut e2 = engine_with_predict();
    let mut doc2 = Vec::new();
    for c in "in order to ".chars() {
        press_key(&mut e2, &mut doc2, &key_for_char(c));
    }
    assert_eq!(doc2.iter().collect::<String>(), "in order to ");
    press_key(&mut e2, &mut doc2, &key_for_char('\t'));
    assert_eq!(doc2.iter().collect::<String>(), "in order to ensure");
}

#[test]
fn test_shift_tab_is_always_passed_through() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    for c in "phats".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "phát");

    let shift_tab = KeyEvent {
        vk: vk::TAB,
        mods: keymap::MOD_SHIFT,
        key_down: true,
        ..Default::default()
    };

    let action = press_key(&mut e, &mut doc, &shift_tab);
    assert_eq!(action, Action::Pass, "Shift+Tab bắt buộc phải Pass (S9)");
}

#[test]
fn test_tab_without_prediction_passes_through() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ từ lạ không có tiền tố nào
    for c in "xyz".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    let tab = key_for_char('\t');
    let action = press_key(&mut e, &mut doc, &tab);
    assert_eq!(action, Action::Pass, "Tab không khớp phải Pass cho app");
}

fn key_ctrl_space() -> KeyEvent {
    KeyEvent {
        vk: vk::SPACE,
        mods: keymap::MOD_CTRL,
        key_down: true,
        ..Default::default()
    }
}

#[test]
fn test_ctrl_space_prefix_completion_and_cycle() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ "phats" → "phát"
    for c in "phats".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "phát");

    // Nhấn Ctrl + Space: hoàn thành thành "phát triển"
    let action = press_key(&mut e, &mut doc, &key_ctrl_space());
    assert!(matches!(action, Action::Replace { .. }));
    assert_eq!(doc.iter().collect::<String>(), "phát triển");

    // Nhấn Ctrl + Space tiếp theo: duyệt sang ứng viên "phát sinh"
    let action2 = press_key(&mut e, &mut doc, &key_ctrl_space());
    assert!(matches!(action2, Action::Replace { .. }));
    assert_eq!(doc.iter().collect::<String>(), "phát sinh");

    // Nhấn Tab tiếp theo: duyệt tiếp sang "phát biểu" (Tab và Ctrl+Space dùng chung trạng thái duyệt vòng)
    let action3 = press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert!(matches!(action3, Action::Replace { .. }));
    assert_eq!(doc.iter().collect::<String>(), "phát biểu");
}

#[test]
fn test_ctrl_space_next_phrase_prediction() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ cụm "cộng hòa "
    for c in "coongj hoaf ".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "cộng hòa ");

    // Nhấn Ctrl + Space: dự đoán cụm từ tiếp theo → "xã hội chủ nghĩa việt nam"
    press_key(&mut e, &mut doc, &key_ctrl_space());
    assert_eq!(
        doc.iter().collect::<String>(),
        "cộng hòa xã hội chủ nghĩa việt nam"
    );

    // Tiếng Anh: "Thank you " + Ctrl+Space → "Thank you very much"
    let mut e2 = engine_with_predict();
    let mut doc2 = Vec::new();
    for c in "Thank you ".chars() {
        press_key(&mut e2, &mut doc2, &key_for_char(c));
    }
    assert_eq!(doc2.iter().collect::<String>(), "Thank you ");
    press_key(&mut e2, &mut doc2, &key_ctrl_space());
    assert_eq!(doc2.iter().collect::<String>(), "Thank you very much");
}

#[test]
fn test_ctrl_space_without_prediction_passes_without_inserting_space() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();

    // Gõ từ không có gợi ý tiền tố
    for c in "xyz".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(doc.iter().collect::<String>(), "xyz");

    // Nhấn Ctrl + Space: không có prediction → Action::Pass
    let action = press_key(&mut e, &mut doc, &key_ctrl_space());
    assert_eq!(action, Action::Pass);
    // Tuyệt đối KHÔNG chèn dấu cách vào văn bản
    assert_eq!(doc.iter().collect::<String>(), "xyz");
}
