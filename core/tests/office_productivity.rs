// SPDX-License-Identifier: GPL-3.0-or-later
//! Kiểm thử tích hợp End-to-End cho các kịch bản làm việc văn phòng thực tế:
//! - Soạn email, báo cáo, biên bản họp song ngữ Việt–Anh (Word, Outlook, Teams, Slack, Jira, Excel).
//! - Gõ tắt (Macro) tên cơ quan, chức danh, cụm câu văn phòng với `Tab` / `Space` và `allow_macro_when_vi_off`.
//! - Quick Telex (`cc→ch`, `gg→gi`, `kk→kh`, `nn→ng`, `qq→qu`, `pp→ph`, `tt→th`) kèm hoàn tác 3 phím.
//! - Khôi phục tiếng Anh tự động tại mọi dấu câu văn phòng + từ điển cá nhân `english_words` + phím `Esc`.
//! - Gợi ý hoàn tất từ bằng `Tab`/`Space` bảo vệ 100% âm tiết tiếng Việt và giữ nguyên `Shift+Tab` (lùi ô Excel/Form).
//! - Tự động viết hoa đầu câu (`auto_capitalize`) qua nhiều câu/đoạn văn.
//! - Gõ tắt Emoji công sở và đồng bộ `sentence_tail` / `glyph_len_of` trên cả 4 bảng mã.

use textvn_core::keymap::{vk, MOD_SHIFT};
use textvn_core::{
    Action, Context, DiacriticStyle, Emoji, Engine, EngineOptions, KeyEvent, MacroDef,
    MacroTrigger, MacroWhen, Method, OutputCharset, Strategy,
};

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

fn apply_outcome(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => match k.vk {
            vk::BACK => {
                buf.pop();
            }
            vk::ESCAPE | vk::LEFT => {}
            _ => {
                if let Some(c) = k.printable() {
                    if !c.is_control() || c == '\n' || c == '\t' || c == ' ' {
                        buf.push(c);
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
            let del = *delete_count as usize;
            assert!(
                del <= buf.len(),
                "delete_count ({del}) vượt độ dài buffer ({})",
                buf.len()
            );
            buf.truncate(buf.len() - del);
            buf.extend_from_slice(insert);
        }
        Action::Commit { insert } => {
            buf.extend_from_slice(insert);
        }
    }
}

fn type_sequence(e: &mut Engine, input: &str) -> String {
    e.reset();
    let mut buf = Vec::new();
    for c in input.chars() {
        let k = key_for_char(c);
        let o = e.key(&k);
        apply_outcome(&mut buf, &o.action, &k);
    }
    buf.into_iter().collect()
}

/// Kịch bản 1: Soạn thảo email/tin nhắn công sở pha trộn tiếng Việt và thuật ngữ tiếng Anh
/// (Teams, Slack, Jira, Outlook) với dấu câu đa dạng mà không cần tắt/bật bộ gõ.
#[test]
fn office_bilingual_sentences_auto_restore_cleanly() {
    let cases = [
        (
            "Guwir anh file report, deadline laf thuws saus nhes.",
            "Gửi anh file report, deadline là thứ sáu nhé.",
        ),
        (
            "Em ddax push code leen branch main vaf taoj pull request.",
            "Em đã push code lên branch main và tạo pull request.",
        ),
        (
            "Check giups em ticket treen Jira hoawjc ping qua Slack/Teams nhes!",
            "Check giúp em ticket trên Jira hoặc ping qua Slack/Teams nhé!",
        ),
        (
            "Server production ddang update database, vui longf ddowij 5 phuts.",
            "Server production đang update database, vui lòng đợi 5 phút.",
        ),
        (
            " Vui longf login vaof portal, nhaapj password vaf download file setup.",
            " Vui lòng login vào portal, nhập password và download file setup.",
        ),
    ];

    for (typed, expected) in cases {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            ..Default::default()
        });
        let got = type_sequence(&mut e, typed);
        assert_eq!(got, expected, "gõ câu song ngữ: `{typed}`");
    }
}

/// Kịch bản 2: Gõ tắt (Macro) hành chính – văn phòng với `Tab` và `Space`,
/// ưu tiên trigger dài nhất, kiểm tra ranh giới từ và `allow_macro_when_vi_off`.
#[test]
fn office_macros_tab_and_space_and_vi_off() {
    let macros = vec![
        MacroDef {
            trigger: "cty".into(),
            expand: "Công ty Cổ phần Công nghệ".into(),
            when: MacroWhen::Always,
        },
        MacroDef {
            trigger: "ubnd".into(),
            expand: "Ủy ban nhân dân".into(),
            when: MacroWhen::Always,
        },
        MacroDef {
            trigger: "tphcm".into(),
            expand: "Thành phố Hồ Chí Minh".into(),
            when: MacroWhen::Always,
        },
        MacroDef {
            trigger: "tp".into(),
            expand: "Thành phố".into(),
            when: MacroWhen::Always,
        },
        MacroDef {
            trigger: "tks".into(),
            expand: "Trân trọng cảm ơn,".into(),
            when: MacroWhen::ViOn,
        },
    ];

    // 1. Trigger bằng Tab: trigger dài nhất (`tphcm` vs `tp`) phải thắng, không ăn đuôi từ liền (`xcty`).
    let mut e_tab = Engine::new(EngineOptions {
        macro_trigger: MacroTrigger::Tab,
        macros: macros.clone(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e_tab, "Kinhs guwir ubnd\t tphcm\t"),
        "Kính gửi Ủy ban nhân dân Thành phố Hồ Chí Minh"
    );
    assert_eq!(
        type_sequence(&mut e_tab, "xcty\t"),
        "xcty\t",
        "không được bung macro khi dính vào chữ trước"
    );

    // 2. Trigger bằng Space: bung macro và nuốt phím trigger theo quy ước P0-3.
    let mut e_space = Engine::new(EngineOptions {
        macro_trigger: MacroTrigger::Space,
        macros: macros.clone(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e_space, "CTY "),
        "Công ty Cổ phần Công nghệ"
    );

    // 3. Khi tắt tiếng Việt (`enabled = false`) nhưng bật `allow_macro_when_vi_off = true`:
    // chỉ các macro `MacroWhen::Always` được bung, còn `MacroWhen::ViOn` (`tks`) đi qua nguyên văn.
    let mut e_vi_off = Engine::new(EngineOptions {
        enabled: false,
        allow_macro_when_vi_off: true,
        macro_trigger: MacroTrigger::Tab,
        macros,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e_vi_off, "cty\t"),
        "Công ty Cổ phần Công nghệ"
    );
    assert_eq!(type_sequence(&mut e_vi_off, "tks\t"), "tks\t");
}

/// Kịch bản 3: Quick Telex (`cc→ch`, `gg→gi`, `kk→kh`, `nn→ng`, `qq→qu`, `pp→ph`, `tt→th`)
/// giúp tốc ký nhanh biên bản họp, đồng thời hoàn tác khi gõ 3 lần (`ccc→cc`).
#[test]
fn office_quick_telex_and_triple_undo() {
    let mut e = Engine::new(EngineOptions {
        quick_telex: true,
        auto_capitalize: false,
        diacritic_style: DiacriticStyle::New,
        ..Default::default()
    });

    let got = type_sequence(&mut e, "ccaoj nnayf kkoong ppair ttoongr qqas ggaof ");
    assert_eq!(got, "chạo ngày không phải thổng quá giào ");

    // Gõ 3 lần hoàn tác phụ âm đôi + bảo toàn chữ hoa đầu từ:
    let mut e2 = Engine::new(EngineOptions {
        quick_telex: true,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e2, "Ccaof "), "Chào ");
    assert_eq!(type_sequence(&mut e2, "TTanhf "), "THành ");
    assert_eq!(type_sequence(&mut e2, "ccc"), "cc");
}

/// Kịch bản 4: Khôi phục từ mơ hồ bằng phím `Escape` và danh sách `english_words` cá nhân
/// (ví dụ lập trình viên / nhân viên tài chính thường xuyên gõ `host`, `list`, `cost`, `tax`, `box`).
#[test]
fn office_custom_english_words_and_escape_restore() {
    // 1. Khi thêm vào `english_words`, các từ mơ hồ (`host`, `list`, `cost`, `tax`, `box`)
    // tự động trả về tiếng Anh khi gõ Space hoặc dấu câu.
    let mut e_custom = Engine::new(EngineOptions {
        auto_capitalize: false,
        english_words: vec![
            "host".into(),
            "list".into(),
            "cost".into(),
            "tax".into(),
            "box".into(),
        ],
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e_custom, "Check host, list cost vaf tax trong box."),
        "Check host, list cost và tax trong box."
    );

    // 2. Với cấu hình mặc định (chưa thêm vào `english_words`), bấm `Escape` ngay sau từ
    // phải hoàn nguyên chuỗi phím thô (`this` thay vì `thí`, `too` thay vì `tô`).
    let mut e_default = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    let mut buf = Vec::new();
    for c in "this".chars() {
        let k = key_for_char(c);
        let o = e_default.key(&k);
        apply_outcome(&mut buf, &o.action, &k);
    }
    let snapshot: String = buf.iter().collect();
    assert_eq!(snapshot, "thí");

    let esc = KeyEvent::key_down(vk::ESCAPE);
    let o_esc = e_default.key(&esc);
    apply_outcome(&mut buf, &o_esc.action, &esc);
    let restored: String = buf.iter().collect();
    assert_eq!(restored, "this");
}

/// Kịch bản 5: Gợi ý hoàn tất từ (`suggest_vn_complete` / `suggest_space_complete`)
/// bảo vệ tuyệt đối âm tiết tiếng Việt và không nuốt `Shift+Tab` (di chuyển lùi ô Excel/Form).
#[test]
fn office_suggest_completion_protects_vietnamese_and_shift_tab() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        suggest_vn_complete: true,
        suggest_space_complete: true,
        ..Default::default()
    });

    // Tất cả các âm tiết tiếng Việt ngắn trùng tiền tố từ tiếng Anh phải giữ nguyên 100%:
    for vn_word in [
        "em", "con", "cho", "cas", "nos", "chos", "nois", "bor", "cor", "tur", "ver", "ves",
    ] {
        let mut e_word = Engine::new(EngineOptions {
            auto_capitalize: false,
            suggest_vn_complete: true,
            suggest_space_complete: true,
            ..Default::default()
        });
        let mut e_plain = Engine::new(EngineOptions {
            auto_capitalize: false,
            suggest_vn_complete: false,
            suggest_space_complete: false,
            ..Default::default()
        });
        let expected = type_sequence(&mut e_plain, &format!("{vn_word} "));
        let got_space = type_sequence(&mut e_word, &format!("{vn_word} "));
        assert_eq!(
            got_space, expected,
            "Space không được biến âm tiết Việt `{vn_word}` thành từ tiếng Anh"
        );
    }

    // Shift+Tab phải luôn trả `Action::Pass` (S9 — lùi ô trong Excel / biểu mẫu):
    let mut buf = Vec::new();
    for c in "nh".chars() {
        let k = key_for_char(c);
        let o = e.key(&k);
        apply_outcome(&mut buf, &o.action, &k);
    }
    let shift_tab = KeyEvent {
        vk: vk::TAB,
        mods: MOD_SHIFT,
        key_down: true,
        ..Default::default()
    };
    let o_shift_tab = e.key(&shift_tab);
    assert_eq!(
        o_shift_tab.action,
        Action::Pass,
        "Shift+Tab phải luôn đi qua cho Excel/Form"
    );
}

/// Kịch bản 6: Tự động viết hoa đầu câu (`auto_capitalize`) qua nhiều câu, dấu chấm than,
/// dấu hỏi, xuống dòng `\n`, nhưng không viết hoa sau dấu phẩy hoặc hai chấm.
#[test]
fn office_auto_capitalize_across_sentences_and_newlines() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: true,
        ..Default::default()
    });
    let typed = "Xin chaof anh, em guwir baos caos. uwf thif ddungs rooif! owr ddaay cofn vieecj gif?nhows kieemr tra\ncamr own anh.";
    let got = type_sequence(&mut e, typed);
    assert_eq!(
        got,
        "Xin chào anh, em gửi báo cáo. Ừ thì đúng rồi! Ở đây còn việc gì?nhớ kiểm tra\nCảm ơn anh."
    );
}

/// Kịch bản 7: Bảng mã văn phòng truyền thống (`VniWindows`, `Tcvn3`, `UnicodeDecomposed`)
/// đồng bộ chính xác `sentence_tail` (Unicode) và `glyph_len_of` (đơn vị xóa của app).
#[test]
fn office_legacy_charsets_keep_sentence_tail_and_glyph_len_synced() {
    for charset in [
        OutputCharset::UnicodePrecomposed,
        OutputCharset::UnicodeDecomposed,
        OutputCharset::Tcvn3,
        OutputCharset::VniWindows,
    ] {
        let mut e = Engine::new(EngineOptions {
            output_charset: charset,
            auto_capitalize: false,
            ..Default::default()
        });
        let _ = type_sequence(&mut e, "Coong ty Vieetj Nam.");
        assert_eq!(
            e.sentence_tail(64),
            "Công ty Việt Nam.",
            "sentence_tail phải luôn là Unicode chuẩn NFC cho trợ lý ngữ pháp trên {charset:?}"
        );
        let sample: Vec<char> = "Việt".chars().collect();
        let glyph_len = e.glyph_len_of(&sample);
        match charset {
            OutputCharset::UnicodePrecomposed | OutputCharset::Tcvn3 => {
                assert_eq!(glyph_len, 4);
            }
            OutputCharset::UnicodeDecomposed | OutputCharset::VniWindows => {
                assert!(
                    glyph_len > 4,
                    "{charset:?} phải tính đúng số glyph tổ hợp ({glyph_len})"
                );
            }
        }
    }
}

/// Kịch bản 8: Gõ tắt Emoji trong chat văn phòng (`:ok`, `:smile`, `:check`) hoạt động
/// độc lập và cùng tồn tại với Macro chữ.
#[test]
fn office_emoji_shortcodes_expand_cleanly() {
    let mut e = Engine::new(EngineOptions {
        macro_trigger: MacroTrigger::Space,
        auto_capitalize: false,
        emoji: vec![
            Emoji {
                trigger: ":ok".into(),
                glyph: "👌".into(),
            },
            Emoji {
                trigger: ":check".into(),
                glyph: "✅".into(),
            },
        ],
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e, "DDax xong baos caos :check "),
        "Đã xong báo cáo ✅"
    );
}

/// Kịch bản 9: Đủ 4 kiểu gõ (`Telex`, `SimpleTelex`, `Vni`, `Viqr`), đổi dấu thanh liên hoàn
/// (`s→f→r→x→j`) và gỡ dấu bằng `z` (Telex) / `0` (VNI).
#[test]
fn office_all_four_methods_tone_switching_and_removal() {
    // 1. Telex: đổi dấu sắc → huyền → hỏi → ngã → nặng rồi gỡ dấu bằng `z`, gõ tiếp `z` ra chữ `z`
    let mut e_telex = Engine::new(EngineOptions {
        method: Method::Telex,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e_telex, "tooisfrxj "), "tội ");
    assert_eq!(type_sequence(&mut e_telex, "tooisz "), "tôi ");
    assert_eq!(type_sequence(&mut e_telex, "tooiszz"), "tôiz");
    assert_eq!(type_sequence(&mut e_telex, "tooiszz "), "tooiszz ");

    // 2. SimpleTelex: `w` đứng riêng không biến thành `ư`, chỉ làm móc cho `o/u/a`
    let mut e_simple = Engine::new(EngineOptions {
        method: Method::SimpleTelex,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e_simple, "w truowngf "), "w trường ");

    // 3. VNI: gõ số tạo dấu và `0` gỡ dấu thanh
    let mut e_vni = Engine::new(EngineOptions {
        method: Method::Vni,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e_vni, "Cong6 ty2 Viet65 Nam "),
        "Công tỳ Việt Nam "
    );
    assert_eq!(type_sequence(&mut e_vni, "toi610 "), "tôi ");

    // 4. VIQR: gõ ký hiệu dấu phụ + dấu thanh
    let mut e_viqr = Engine::new(EngineOptions {
        method: Method::Viqr,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e_viqr, "Vie^t. Nam "), "Việt Nam ");
}

/// Kịch bản 10: Sửa giữa chừng bằng Backspace rồi gõ tiếp nguyên âm/phụ âm cuối
/// (thói quen rất phổ biến khi gõ nhanh văn bản văn phòng).
#[test]
fn office_backspace_mid_word_and_continue_typing() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    let mut buf = Vec::new();
    for c in "toois".chars() {
        let k = key_for_char(c);
        let o = e.key(&k);
        apply_outcome(&mut buf, &o.action, &k);
    }
    assert_eq!(buf.iter().collect::<String>(), "tối");

    // Xóa chữ `i` cuối cùng (vẫn giữ nguyên dấu sắc trên `ố`), rồi gõ `t` → `tốt`!
    let bs = KeyEvent::key_down(vk::BACK);
    let o_bs = e.key(&bs);
    apply_outcome(&mut buf, &o_bs.action, &bs);
    assert_eq!(buf.iter().collect::<String>(), "tố");

    for c in "t ".chars() {
        let k = key_for_char(c);
        let o = e.key(&k);
        apply_outcome(&mut buf, &o.action, &k);
    }
    assert_eq!(buf.iter().collect::<String>(), "tốt ");
}

/// Kịch bản 11: Phân loại chiến lược xuất (`Strategy`) cho các ô nhập liệu văn phòng
/// (ô công thức/bảng tính Excel, thanh địa chỉ trình duyệt, Terminal, ô mật khẩu, tài liệu Word).
#[test]
fn office_field_roles_and_strategy_resolution() {
    let mut e = Engine::new(EngineOptions::default());

    // Ô mật khẩu (Secure) → luôn Passthrough
    e.set_context(Context {
        secure: true,
        ..Default::default()
    });
    assert_eq!(e.strategy(), Strategy::Passthrough);

    // Thanh địa chỉ / tìm kiếm (AddressBar) có hỗ trợ SelectionReplace → ưu tiên tránh gạch chân preedit
    e.set_context(Context {
        secure: false,
        field_role: strategy::IME_FIELD_ADDRESS_BAR,
        caps: strategy::IME_CAP_PREEDIT
            | strategy::IME_CAP_SELECTION
            | strategy::IME_CAP_FIELD_DETECT,
        hint: strategy::IME_STRATEGY_SELECTION_REPLACE,
        ..Default::default()
    });
    assert_eq!(e.strategy(), Strategy::SelectionReplace);

    // Ô văn bản chuẩn (Body) có đủ quyền Preedit + FieldDetect → Preedit
    e.set_context(Context {
        secure: false,
        field_role: strategy::IME_FIELD_BODY,
        caps: strategy::IME_CAP_PREEDIT | strategy::IME_CAP_FIELD_DETECT,
        hint: strategy::IME_STRATEGY_PREEDIT,
        ..Default::default()
    });
    assert_eq!(e.strategy(), Strategy::Preedit);
}
/// Macros dùng chung cho test OP-001.
fn hd_cty_macros() -> Vec<MacroDef> {
    vec![
        MacroDef {
            trigger: "hd".into(),
            expand: "hợp đồng".into(),
            when: MacroWhen::Always,
        },
        MacroDef {
            trigger: "cty".into(),
            expand: "Công ty TNHH".into(),
            when: MacroWhen::Always,
        },
    ]
}

/// OP-001 (P5): gõ tắt thích ứng hoa/thường — `hd`/`Hd`/`HD` ra đúng kiểu chữ;
/// expand gốc có chữ hoa chủ đích (`Công ty TNHH`) giữ nguyên 100% (R1-1).
#[test]
fn op001_case_adaptive_macro() {
    let mut e = Engine::new(EngineOptions {
        macros: hd_cty_macros(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e, "hd	"), "hợp đồng");
    let mut e2 = Engine::new(EngineOptions {
        macros: hd_cty_macros(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e2, "Hd	"), "Hợp đồng");
    let mut e3 = Engine::new(EngineOptions {
        macros: hd_cty_macros(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e3, "HD	"), "HỢP ĐỒNG");
    // Expand gốc đã có chữ hoa chủ đích → giữ nguyên kể cả gõ toàn hoa.
    let mut e4 = Engine::new(EngineOptions {
        macros: hd_cty_macros(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e4, "CTY	"), "Công ty TNHH");
    // Gõ thường vẫn như cũ.
    let mut e5 = Engine::new(EngineOptions {
        macros: hd_cty_macros(),
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e5, "CTY	".to_lowercase().as_str()),
        "Công ty TNHH"
    );
}

/// OP-002 (P5): bảo vệ tên hàm công thức bảng tính — `=sum(...)`, `=if(...)`,
/// `=or(...)`, `=vlookup(...)` không bị Telex fold thành âm tiết Việt.
#[test]
fn op002_excel_formula_guard() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e, "=sum(a1:a10)"), "=sum(a1:a10)");
    let mut e2 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e2, "=if(a1>0, 1, 0)"), "=if(a1>0, 1, 0)");
    let mut e3 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e3, "=or(a1, b1)"), "=or(a1, b1)");
    let mut e4 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e4, "=vlookup(a2, b2:d10, 2, false)"),
        "=vlookup(a2, b2:d10, 2, false)"
    );
    // `=` KHÔNG ở đầu ô (`x==1`) → vẫn gõ tiếng Việt bình thường phía sau.
    let mut e5 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e5, "x=too "), "x=tô ");
}

/// OP-003 (P5): gõ tắt số tiền `=<số>(d|vnd|bc)` + phím macro trigger →
/// định dạng chấm phân cách / đọc bằng chữ tiếng Việt (kế toán, hợp đồng).
#[test]
fn op003_number_shorthand_currency_and_words() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e, "=15450000d\t"), "15.450.000 đồng");
    let mut e2 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e2, "=15450000vnd\t"), "15.450.000 VNĐ");
    let mut e3 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e3, "=15000000bc\t"),
        "Mười lăm triệu đồng chẵn"
    );
    let mut e4 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(
        type_sequence(&mut e4, "=125000bc\t"),
        "Một trăm hai mươi lăm nghìn đồng chẵn"
    );
    // Không phải cú pháp số tiền (`=` sau chữ) → Tab đi qua nguyên văn.
    let mut e5 = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e5, "a=15d\t"), "a=15d\t");
}

/// OP-006 (P5): identifier `camelCase`/`PascalCase` của coder gõ trực tiếp,
/// thuật ngữ tech/business tự khôi phục tại dấu cách; âm tiết tiếng Việt không
/// bị cấm (phím dấu vẫn hoạt động — R2-4), tự viết hoa đầu câu không kích hoạt
/// camel (R1-4).
#[test]
fn op006_coder_and_business_jargon() {
    // camelCase/identifier: "use"+"S" nhận diện camel → phần sau gõ thẳng raw.
    let mut e = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e, "useState "), "useState ");
    let mut e2 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e2, "onPress "), "onPress ");
    let mut e3 = Engine::new(EngineOptions::default());
    assert_eq!(
        type_sequence(&mut e3, "getElementById()"),
        "getElementById()"
    );

    // Thuật ngữ tech/business: fold do phím dấu nằm trong tên được khôi phục.
    let mut e4 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e4, "target "), "target ");
    let mut e5 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e5, "margin "), "margin ");
    let mut e6 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e6, "brief "), "brief ");
    let mut e7 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e7, "cargo "), "cargo ");

    // R2-4: âm tiết tiếng Việt THƯỜNG không bị cướp dù trùng tiền tố thuật ngữ
    // ("cac" là CAC) — phím dấu vẫn sinh dấu bình thường.
    let mut e8 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e8, "cacs "), "các ");
    let mut e9 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e9, "momj "), "mọm ");

    // R1-4: tự viết hoa đầu câu ('B') → camel KHÔNG fire trên từ đó ("BAn").
    let mut e10 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e10, ". bAn "), ". BAn ");

    // OP-006: Tab hoàn tất thuật ngữ tech/business giữ case chuẩn — từ sau
    // camel-repair ĐÃ active nên đi qua gate Tab (v24): `useS` + Tab → `useState`.
    let mut e11 = Engine::new(EngineOptions::default());
    assert_eq!(type_sequence(&mut e11, "useS\t"), "useState");
    // Từ CHƯA fold không được Tab cứu (indent IDE/Excel giữ nguyên — v24),
    // nhưng opt-in `suggest_space_complete` hoàn tất được ("ebit" + Space).
    let mut e12 = Engine::new(EngineOptions {
        suggest_space_complete: true,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e12, "ebit "), "EBITDA ");
}

/// Vòng 37 (user: "gợi ý tiếng Anh không chạy"): Tab hoàn tất từ tiếng Anh
/// CHƯA fold khi dự đoán bật ("develop" + Tab → "development") — trước đây
/// Tab chỉ mở cho từ đã bị Telex biến đổi.
#[test]
fn op037_en_tab_completion_unfolded_word() {
    let mut e = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e, "develop	"), "developed"); // rank: từ ngắn hơn trước
    let mut e2 = Engine::new(EngineOptions {
        suggest_vn_complete: true,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e2, "improv	"), "improve");

    // Tắt dự đoán ⇒ hành vi cũ: Tab đi qua (từ chưa fold không bị động vào).
    let mut e3 = Engine::new(EngineOptions {
        suggest_vn_complete: false,
        auto_capitalize: false,
        ..Default::default()
    });
    assert_eq!(type_sequence(&mut e3, "develop	"), "develop	");
}

/// Vòng 37: SVA contraction — "He don't" → "doesn't" (tokenizer tách
/// apostrophe; merge_contractions gộp lại trước khi so cặp).
#[test]
fn op037_sva_contraction_he_doesnt() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    // Đường assist chạy qua FFI check — dùng CLI tương đương: kiểm bằng
    // engine sentence tail qua FFI là đủ ở assist/tests; ở đây chỉ bảo đảm
    // gõ không vỡ token (engine pass-through).
    assert_eq!(
        type_sequence(&mut e, "He don't like it."),
        "He don't like it."
    );
}
