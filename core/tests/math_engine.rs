// SPDX-License-Identifier: GPL-3.0-or-later
//! P11 — regression test Math Engine trong core Engine (corpus thật, G19).

use textvn_config::parse_config;
use textvn_core::{Action, Context, Engine, EngineOptions, KeyEvent, Outcome};
use textvn_math::MathConfig;

/// Gõ chuỗi phím thật (key_down, printable) — mô phỏng luồng TSF.
fn type_str(e: &mut Engine, s: &str) -> Vec<Outcome> {
    let mut outs = Vec::new();
    for c in s.chars() {
        let k = KeyEvent {
            vk: 0,
            ch: c as u32,
            mods: 0,
            key_down: true,
            is_repeat: false,
            is_injected: false,
        };
        outs.push(e.key(&k));
    }
    outs
}

fn engine_with_math(math: MathConfig) -> Engine {
    let opts = EngineOptions {
        math,
        ..EngineOptions::default()
    };
    Engine::new(opts)
}

fn math_on() -> MathConfig {
    MathConfig {
        enabled: true,
        sticky_math_mode: true,
        ..MathConfig::default()
    }
}

#[test]
fn math_off_is_exact_legacy_behavior() {
    // Math TẮT: hành vi gõ Telex phải GIỐNG HỆT engine không có math section.
    let src = "xin chao the gioi";
    let a = {
        let mut e = Engine::new(EngineOptions::default());
        for out in type_str(&mut e, src) {
            assert_ne!(out.action, Action::Pass, "Telex phải biến đổi");
        }
        let mut e = Engine::new(EngineOptions::default());
        type_str(&mut e, "homsau"); // Telex: hoà su…
        e
    };
    let b = engine_with_math(MathConfig::default()); // enabled=false
    let mut legacy = Engine::new(EngineOptions::default());
    let _ = legacy;
    let _ = (a, b, src);
    // So sánh đầu ra trên corpus Telex: math-off vs default
    let corpus = ["homsau", "ddang", "toefl", "test", "caaaaa"];
    let mut o1 = engine_with_math(MathConfig::default());
    let mut o2 = Engine::new(EngineOptions::default());
    for w in corpus {
        let r1: Vec<String> = type_str(&mut o1, w)
            .iter()
            .map(|o| match &o.action {
                Action::Replace { insert, .. } | Action::Commit { insert } => {
                    insert.iter().collect()
                }
                _ => String::new(),
            })
            .collect();
        let r2: Vec<String> = type_str(&mut o2, w)
            .iter()
            .map(|o| match &o.action {
                Action::Replace { insert, .. } | Action::Commit { insert } => {
                    insert.iter().collect()
                }
                _ => String::new(),
            })
            .collect();
        assert_eq!(r1, r2, "math OFF phải identity với legacy cho {w}");
    }
}

#[test]
fn math_leader_greek_commit() {
    let mut e = engine_with_math(math_on());
    // Leader ` + a → α commit, phím bị nuốt (FLAG_CONSUMED cho leader)
    let outs = type_str(&mut e, "`a");
    assert_eq!(outs.len(), 2);
    match &outs[0].action {
        Action::Pass => assert_ne!(outs[0].flags & 0x1, 0, "leader phải bị consume"),
        other => panic!("leader phải pass+consume, got {other:?}"),
    }
    match &outs[1].action {
        Action::Commit { insert } => {
            let s: String = insert.iter().collect();
            assert_eq!(s, "α", "got {s}");
        }
        other => panic!("a phải commit α, got {other:?}"),
    }
}

#[test]
fn math_alias_expansion_on_space() {
    let mut e = engine_with_math(math_on());
    let outs = type_str(&mut e, "\\tong ");
    // 5 phím đầu: pass (VN suspended cho word \tong), phím Space: Replace
    let last = outs.last().unwrap();
    match &last.action {
        Action::Replace { delete_count, insert } => {
            assert_eq!(*delete_count, 5, "xoá \\tong");
            let s: String = insert.iter().collect();
            assert_eq!(s, "∑ ", "got {s}");
        }
        other => panic!("space phải expand alias, got {other:?}"),
    }
}

#[test]
fn math_eval_inline() {
    let mut e = engine_with_math(math_on());
    let outs = type_str(&mut e, "=2^10=");
    // Phím '=' cuối: Replace "= 1024 "
    let last = outs.last().unwrap();
    match &last.action {
        Action::Replace { delete_count, insert } => {
            assert_eq!(*delete_count, 5, "xoá '=2^10' (phím = cuối chưa vào doc)");
            let s: String = insert.iter().collect();
            assert_eq!(s, "= 1024 ", "got {s}");
        }
        other => panic!("'=' đóng phải eval, got {other:?}"),
    }
}

#[test]
fn math_bad_eval_passes_through() {
    let mut e = engine_with_math(math_on());
    let outs = type_str(&mut e, "=abc=");
    let last = outs.last().unwrap();
    assert!(
        matches!(last.action, Action::Pass | Action::Replace { .. }),
        "biểu thức sai: pass hoặc replace gốc"
    );
    if let Action::Replace { insert, .. } = &last.action {
        let s: String = insert.iter().collect();
        assert_eq!(s, "=abc=", "không được phát kết quả nào cho expr sai");
    }
}

#[test]
fn math_in_dollar_suspends_vietnamese() {
    let mut e = engine_with_math(math_on());
    // "$homsau $" — trong $...$ Telex KHÔNG được biến đổi
    let outs = type_str(&mut e, "$homsau");
    // 'h','o','m','s','a','u' đều phải Pass (không Replace thành hoà su)
    for (i, out) in outs.iter().enumerate().skip(1) {
        let ok = match &out.action {
            Action::Pass => true,
            Action::Commit { insert } => {
                // chỉ commit ký tự thường (không dấu)
                insert.iter().all(|c| !is_vn_mark(*c))
            }
            _ => false,
        };
        assert!(ok, "phím {i} trong $..$ phải treo Telex, got {:?}", out.action);
    }
}

fn is_vn_mark(c: char) -> bool {
    "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ".contains(c)
}

#[test]
fn math_text_span_reenables_vietnamese() {
    let mut e = engine_with_math(math_on());
    // Trong \text{} Telex hoạt động: "$\text{homsau}$" → "hoà su" trong text
    let outs = type_str(&mut e, "$\\text{homsau}$");
    // Tìm phím 's' (sau 'hom'): trong \text{} — phải biến đổi Telex (ở → ò…)
    let mut replaced = false;
    for out in &outs {
        if let Action::Replace { insert, .. } = &out.action {
            let s: String = insert.iter().collect();
            if s.contains('ò') || s.contains('à') || s.contains('o') && s.chars().count() > 1 {
                replaced = true;
            }
        }
    }
    assert!(replaced, "\\text{} phải bật lại Telex");
}

#[test]
fn math_secure_field_disables_math() {
    let mut e = engine_with_math(math_on());
    e.set_context(Context {
        secure: true,
        ..Context::default()
    });
    let outs = type_str(&mut e, "`a");
    // Ô secure: math tắt — leader đi qua như ký tự thường
    let any_commit_alpha = outs.iter().any(|o| {
        matches!(&o.action, Action::Commit { insert } if insert.iter().collect::<String>() == "α")
    });
    assert!(!any_commit_alpha, "secure phải tắt math");
}

#[test]
fn math_template_expand_in_core() {
    let mut e = engine_with_math(math_on());
    let outs = type_str(&mut e, "\\mat2x2 ");
    let last = outs.last().unwrap();
    match &last.action {
        Action::Replace { insert, cursor_left, .. } => {
            let s: String = insert.iter().collect();
            assert!(s.contains("\\begin{matrix}"), "got {s}");
            assert!(*cursor_left > 0, "template phải có cursor_left");
        }
        other => panic!("template phải expand, got {other:?}"),
    }
}

#[test]
fn math_dream_rsi_counts_on_accept() {
    let mut e = engine_with_math(math_on());
    let _ = type_str(&mut e, "\\tong ");
    let snap = e.dream_rsi_snapshot();
    assert_eq!(snap.accepts, 1, "alias expand = 1 accept");
    // keys_typed: 6 phím thật (5 word + space) — không phá bộ đếm (G19)
    assert_eq!(snap.keys_typed, 6);
    assert!(snap.chars_saved > 0);
    assert!(snap.savings_rate_pct() > 0);
}

#[test]
fn math_protected_app_via_context_app_id() {
    let mut e = engine_with_math(math_on());
    e.set_context(Context {
        app_id: "MathType.exe".into(),
        ..Context::default()
    });
    let outs = type_str(&mut e, "`a");
    // MathType: KHÔNG nuốt gì — leader + a đi qua nguyên văn
    let alpha = outs.iter().any(|o| {
        matches!(&o.action, Action::Commit { insert } if insert.iter().collect::<String>() == "α")
    });
    assert!(!alpha, "MathType protected: không được emit α");
}

#[test]
fn math_config_via_json_ffi_shape() {
    // Config JSON có math section → parse_config → EngineOptions (luồng ffi).
    let cfg = parse_config(
        r#"{"config_version":1,"math":{"enabled":true,"sticky_math_mode":true,"output_mode":"Latex"}}"#,
    )
    .unwrap();
    assert!(cfg.math.enabled);
    assert_eq!(
        cfg.math.output_mode,
        textvn_math::MathOutputMode::Latex
    );
    let mut e = engine_with_math(cfg.math.clone());
    let outs = type_str(&mut e, "`/");
    match &outs[1].action {
        Action::Commit { insert } => {
            let s: String = insert.iter().collect();
            assert_eq!(s, "\\frac{}{}", "Latex mode: / → frac, got {s}");
        }
        other => panic!("got {other:?}"),
    }
}

#[test]
fn math_normal_typing_unaffected_when_sticky_on() {
    // Sticky ON nhưng text thường KHÔNG bắt đầu `\` → Telex vẫn chạy bình thường
    let mut e = engine_with_math(math_on());
    let outs = type_str(&mut e, "homsau ");
    let mut got_vn = false;
    for out in &outs {
        if let Action::Replace { insert, .. } = &out.action {
            let s: String = insert.iter().collect();
            if s.contains('ò') || s.contains('à') {
                got_vn = true;
            }
        }
    }
    assert!(got_vn, "sticky ON không được phá Telex trên text thường");
}
