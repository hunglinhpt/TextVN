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
        "cộng hòa xã hội chủ nghĩa Việt Nam"
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
        "cộng hòa xã hội chủ nghĩa Việt Nam"
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

// --- R2-120 (audit P6 2026-10-25): lỗi nghiệp vụ dự đoán đã tái hiện bằng probe ---

fn typed(e: &mut Engine, input: &str) -> String {
    let mut doc = Vec::new();
    for c in input.chars() {
        press_key(e, &mut doc, &key_for_char(c));
    }
    doc.iter().collect()
}

fn predict_engine(style: textvn_core::DiacriticStyle) -> Engine {
    Engine::new(EngineOptions {
        method: Method::Telex,
        suggest_vn_complete: true,
        diacritic_style: style,
        ..Default::default()
    })
}

/// Ngữ cảnh nhiều từ vừa gõ xong (KHÔNG cần dấu cách) → cụm nối tiếp đi trước
/// hoàn tất tiền tố của âm tiết cuối. Lỗi cũ: "phát triển"+Tab → "phát triển
/// khai", "hợp đồng"+Tab → "hợp đồng ý", "Kính gửi"+Tab → chèn Tab.
#[test]
fn r2_120_ngu_canh_nhieu_tu_truoc_hoan_tat_tien_to() {
    let old = textvn_core::DiacriticStyle::Old;
    for (input, want) in [
        ("phats trieenr\t", "phát triển phần mềm"),
        ("hopwj ddoongf\t", "hợp đồng kinh tế"),
        ("Kinhs guwir\t", "Kính gửi quý khách hàng"),
        ("coongj hoaf\t", "cộng hòa xã hội chủ nghĩa Việt Nam"),
        ("Thank you\t", "Thank you very much"),
        // Có dấu cách vẫn như cũ.
        ("phats trieenr \t", "phát triển phần mềm"),
        // Từ đơn không có khoá nhiều từ → vẫn hoàn tất tiền tố.
        ("phats\t", "phát triển"),
    ] {
        assert_eq!(typed(&mut predict_engine(old), input), want, "{input:?}");
    }
    // Duyệt vòng sau khi chốt từ: chỉ thay phần cụm, giữ "Kính gửi".
    assert_eq!(
        typed(&mut predict_engine(old), "Kinhs guwir\t\t"),
        "Kính gửi ban giám đốc"
    );
}

/// Cụm nối tiếp không thừa kế viết hoa chữ đầu của từ trước; chữ hoa chuẩn của
/// dữ liệu (Việt Nam, tiêu ngữ) được giữ; HOA TOÀN BỘ kéo theo.
#[test]
fn r2_120_chu_hoa_cum_noi_tiep() {
    let old = textvn_core::DiacriticStyle::Old;
    for (input, want) in [
        ("Xin \t", "Xin chào"),
        ("Xin\t", "Xin chào"),
        ("XIN \t", "XIN CHÀO"),
        ("Coongj hoaf \t", "Cộng hòa xã hội chủ nghĩa Việt Nam"),
        ("DDoocj laapj\t", "Độc lập - Tự do - Hạnh phúc"),
    ] {
        assert_eq!(typed(&mut predict_engine(old), input), want, "{input:?}");
    }
}

/// Xuống dòng là ngữ cảnh mới — Tab đi qua cho app (trước đây chèn
/// " quý khách hàng" ở đầu dòng mới).
#[test]
fn r2_120_khong_du_doan_qua_dong_moi() {
    let mut e = predict_engine(textvn_core::DiacriticStyle::Old);
    assert_eq!(typed(&mut e, "Kinhs guwir\n\t"), "Kính gửi\n\t");
}

/// Kiểu dấu mới: ngữ cảnh "hoà" khớp khoá kiểu cũ và ứng viên chèn theo kiểu
/// người dùng (không trộn "hòa"/"hoà" trong cùng văn bản).
#[test]
fn r2_120_kieu_dau_moi() {
    let new = textvn_core::DiacriticStyle::New;
    assert_eq!(
        typed(&mut predict_engine(new), "Coongj hoaf\t"),
        "Cộng hoà xã hội chủ nghĩa Việt Nam"
    );
    assert_eq!(typed(&mut predict_engine(new), "hoaf\t"), "hoài");
}

/// Đang duyệt vòng ứng viên: Tab kế tiếp đổi ứng viên — kể cả khi phím Tab
/// cũng là phím gõ tắt và đuôi ứng viên trùng một trigger gõ tắt.
#[test]
fn r2_120_duyet_vong_uu_tien_hon_go_tat() {
    let mut e = Engine::new(EngineOptions {
        method: Method::Telex,
        suggest_vn_complete: true,
        macro_trigger: textvn_core::MacroTrigger::Tab,
        macros: vec![textvn_core::MacroDef {
            trigger: "triển".into(),
            expand: "SAI".into(),
            when: textvn_core::MacroWhen::Always,
        }],
        ..Default::default()
    });
    assert_eq!(typed(&mut e, "phats\t\t"), "phát sinh");
}

/// Ctrl+Space không khớp dự đoán: bỏ quyền sở hữu từ (WORD_END — adapter chốt
/// composition) nhưng giữ buffer câu cho trợ lý ngữ pháp on-demand (R2-121).
#[test]
fn r2_121_ctrl_space_khong_khop_nha_tu_giu_cau() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();
    for c in "tooi ddi hocj".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    let out = e.key(&key_ctrl_space());
    assert_eq!(out.action, Action::Pass);
    assert_ne!(out.flags & textvn_core::FLAG_WORD_END, 0);
    assert_eq!(e.sentence_tail(64), "tôi đi học");
    // Phím dấu kế tiếp KHÔNG được sửa lại từ "học" đã nhả (chord semantics).
    let out = e.key(&key_for_char('s'));
    assert_eq!(out.action, Action::Pass);
}

/// VN tắt: Ctrl+Space không ghi dấu cách ảo vào đuôi text.
#[test]
fn r2_121_ctrl_space_khi_tat_tieng_viet_khong_ghi_cach_ao() {
    let mut e = Engine::new(EngineOptions {
        enabled: false,
        ..Default::default()
    });
    for c in "abc".chars() {
        e.key(&key_for_char(c));
    }
    assert_eq!(e.key(&key_ctrl_space()).action, Action::Pass);
    assert_eq!(e.sentence_tail(64), "");
}

/// R2-131 (chủ repo: phím tắt "sửa hết cùng với text prediction"): sau khi
/// trợ lý sửa câu, adapter nạp câu đã sửa vào engine — Ctrl+Space bấm tiếp phải
/// dự đoán theo ngữ cảnh đó (trước đây engine bị reset sạch ⇒ không dự đoán).
#[test]
fn r2_131_reset_with_tail_giu_ngu_canh_du_doan_sau_khi_sua() {
    let mut e = engine_with_predict();
    e.reset_with_tail("tôi muốn phát triển");
    assert_eq!(e.sentence_tail(64), "tôi muốn phát triển");
    let mut doc: Vec<char> = "tôi muốn phát triển".chars().collect();
    press_key(&mut e, &mut doc, &key_ctrl_space());
    assert_eq!(
        doc.iter().collect::<String>(),
        "tôi muốn phát triển phần mềm",
        "dự đoán nối tiếp theo câu đã sửa"
    );
    // Không nạp: engine reset sạch ⇒ Ctrl+Space không có ngữ cảnh để dự đoán.
    let mut bare = engine_with_predict();
    bare.reset();
    let mut doc2: Vec<char> = "tôi muốn phát triển".chars().collect();
    press_key(&mut bare, &mut doc2, &key_ctrl_space());
    assert_eq!(doc2.iter().collect::<String>(), "tôi muốn phát triển");
    // Ký tự điều khiển trong tail ⇒ chỉ reset (không nạp rác vào buffer câu).
    e.reset_with_tail("a\u{7}b");
    assert_eq!(e.sentence_tail(64), "");
}

/// R2-134 (chủ repo: "chữ mờ gợi ý sẵn chưa được"): `peek_completion` trả đúng
/// phần chữ Tab SẼ nối thêm — để adapter vẽ mờ trước khi nhấn — và KHÔNG đổi
/// trạng thái engine (Tab thật sau đó cho đúng kết quả đã xem trước).
#[test]
fn r2_134_peek_completion_xem_truoc_dung_phan_tab_noi_them() {
    let mut e = engine_with_predict();
    let mut doc = Vec::new();
    for c in "phats trieenr".chars() {
        press_key(&mut e, &mut doc, &key_for_char(c));
    }
    assert_eq!(e.peek_completion().as_deref(), Some(" phần mềm"));
    // Xem trước nhiều lần không đổi gì.
    assert_eq!(e.peek_completion().as_deref(), Some(" phần mềm"));
    assert_eq!(doc.iter().collect::<String>(), "phát triển");
    // Tab thật = đúng phần đã xem trước.
    press_key(&mut e, &mut doc, &key_for_char('\t'));
    assert_eq!(doc.iter().collect::<String>(), "phát triển phần mềm");
    // Đang duyệt vòng: Tab kế THAY ứng viên (không nối thêm) ⇒ không vẽ ghost.
    assert_eq!(e.peek_completion(), None);

    // Hoàn tất tiền tố của từ đang gõ.
    let mut e2 = engine_with_predict();
    let mut doc2 = Vec::new();
    for c in "phats".chars() {
        press_key(&mut e2, &mut doc2, &key_for_char(c));
    }
    assert_eq!(e2.peek_completion().as_deref(), Some(" triển"));

    // Không có gì để dự đoán / dự đoán tắt ⇒ None.
    let mut e3 = engine_with_predict();
    assert_eq!(typed(&mut e3, "zzz"), "zzz");
    assert_eq!(e3.peek_completion(), None);
    let mut off = Engine::new(EngineOptions {
        method: Method::Telex,
        enabled: true,
        ..Default::default()
    });
    assert_eq!(typed(&mut off, "phats trieenr"), "phát triển");
    assert_eq!(off.peek_completion(), None);
}
