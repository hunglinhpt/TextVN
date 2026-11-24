// SPDX-License-Identifier: GPL-3.0-or-later
//! E62 — Hồi quy toàn diện cho "Math & Science" trên mọi editor / vùng nhập text:
//! bật `math.enabled = true` là gõ được ngay chord (`` `a ``, `` `/ ``), alias (`\can `),
//! inline eval (`=2+3=`), tự đóng/mở `$...$`, `$$...$$`, `\(...\)`, `\text{...}`, Mode EN,
//! WINWORD.EXE, AutoByApp, Esc hủy chord, AltGr, và Dream RSI không double-count.

use textvn_core::keymap::{MOD_ALT, MOD_CTRL};
use textvn_core::{
    keymap, Action, Context, Engine, EngineOptions, KeyEvent, FLAG_CONSUMED, FLAG_WORD_END,
};
use textvn_math::MathOutputMode;

fn type_stream(e: &mut Engine, keys: &str) -> (String, u16) {
    let mut buf: Vec<char> = Vec::new();
    let mut last_cursor_left = 0u16;
    for c in keys.chars() {
        let k = match c {
            ' ' => KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..Default::default()
            },
            '\n' => KeyEvent::key_down(keymap::vk::RETURN),
            '\t' => KeyEvent::key_down(keymap::vk::TAB),
            _ => KeyEvent::char_down(c),
        };
        let out = e.key(&k);
        last_cursor_left = out.cursor_left;
        match out.action {
            Action::Pass => {
                if out.flags & FLAG_CONSUMED == 0 {
                    buf.push(c);
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
                buf.truncate(buf.len().saturating_sub(delete_count as usize));
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
    (buf.into_iter().collect(), last_cursor_left)
}

fn opts_math(mode: MathOutputMode) -> EngineOptions {
    let mut o = EngineOptions::default();
    o.math.enabled = true;
    o.math.output_mode = mode;
    o
}

#[test]
fn e62_immediate_chords_aliases_and_eval_without_sticky() {
    let mut e = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(type_stream(&mut e, "`a + `b = `c").0, "α + β = χ");
    let mut e2 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(type_stream(&mut e2, "`/").0, "÷");
    assert_eq!(type_stream(&mut e2, " `` `1").0, " ` `1");

    let mut e3 = Engine::new(opts_math(MathOutputMode::Latex));
    let (s, left) = type_stream(&mut e3, "`/");
    assert_eq!(s, "\\frac{}{}");
    assert_eq!(left, 3, "Latex \\frac{{}}{{}} dịch con trỏ trái 3 bước");

    let mut e4 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(type_stream(&mut e4, "\\can ").0, "√x ");
    assert_eq!(type_stream(&mut e4, "\\alpha ").0, "α ");
    assert_eq!(type_stream(&mut e4, "=2+3=").0, "5");
    assert_eq!(type_stream(&mut e4, "=cos(0)=").0, "1");
    assert_eq!(type_stream(&mut e4, "\\can\n").0, "√x\n");
    assert_eq!(type_stream(&mut e4, "\\can\t").0, "√x");
}

#[test]
fn e62_math_spans_and_text_exit_preserve_vietnamese() {
    let mut e = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e, "Tongs $aa$ chaof bajn ").0,
        "Tóng $aa$ chào bạn "
    );

    let mut e2 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e2, "$$x^2 + aa$$ chaof ").0,
        "$$x^2 + aa$$ chào "
    );

    let mut e3 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e3, "\\(a+b\\) chaof ").0,
        "\\(a+b\\) chào "
    );

    let mut e4 = Engine::new(opts_math(MathOutputMode::Normal));
    assert_eq!(
        type_stream(&mut e4, "$x + \\text{với } aa$ chaof ").0,
        "$x + \\text{với } aa$ chào "
    );
}

#[test]
fn e62_en_mode_winword_autobyapp_esc_and_altgr() {
    // Mode EN (`enabled = false`, `ctx.enabled = false`) + `math.enabled = true`: vẫn gõ được công thức!
    let mut opts = opts_math(MathOutputMode::Normal);
    opts.enabled = false;
    opts.math.altgr_enabled = false;
    let mut e = Engine::new(opts);
    e.set_context(Context {
        enabled: false,
        secure: false,
        app_id: "notepad.exe".into(),
        ..Default::default()
    });
    assert_eq!(type_stream(&mut e, "`a \\can =2+3=").0, "α √x 5");

    // WINWORD.EXE (document bình thường, không phải Equation OLE): không chặn math chords.
    let mut e_word = Engine::new(opts_math(MathOutputMode::Normal));
    e_word.set_context(Context {
        enabled: true,
        app_id: "WINWORD.EXE".into(),
        ..Default::default()
    });
    assert_eq!(type_stream(&mut e_word, "`a `q =3*4=").0, "α √ 12");

    // AutoByApp trong Code.exe → Latex (`\frac{}{}`), thay vì mắc kẹt ở AutoByApp!
    let mut e_code = Engine::new(opts_math(MathOutputMode::AutoByApp));
    e_code.set_context(Context {
        enabled: true,
        app_id: "Code.exe".into(),
        ..Default::default()
    });
    let (s_code, left_code) = type_stream(&mut e_code, "`/");
    assert_eq!(s_code, "\\frac{}{}");
    assert_eq!(left_code, 3);

    // Esc hủy chord đang armed (`) và không cộng keys_typed.
    let mut e_esc = Engine::new(opts_math(MathOutputMode::Normal));
    let out1 = e_esc.key(&KeyEvent::char_down('`'));
    assert_ne!(out1.flags & FLAG_CONSUMED, 0);
    let keys_before_esc = e_esc.dream_rsi_snapshot().keys_typed;
    let out_esc = e_esc.key(&KeyEvent::key_down(keymap::vk::ESCAPE));
    assert_ne!(out_esc.flags & FLAG_CONSUMED, 0);
    assert_eq!(e_esc.dream_rsi_snapshot().keys_typed, keys_before_esc);
    assert_eq!(type_stream(&mut e_esc, "a").0, "a");

    // AltGr layer (`altgr_enabled = true`) + Ctrl+Alt+'a' -> α
    let mut opts_altgr = opts_math(MathOutputMode::Normal);
    opts_altgr.math.altgr_enabled = true;
    let mut e_alt = Engine::new(opts_altgr);
    let altgr_a = KeyEvent {
        vk: b'A' as u32,
        ch: 'a' as u32,
        mods: MOD_CTRL | MOD_ALT,
        key_down: true,
        ..Default::default()
    };
    let out_alt = e_alt.key(&altgr_a);
    assert_eq!(out_alt.action, Action::Commit { insert: vec!['α'] });
    assert_ne!(out_alt.flags & FLAG_WORD_END, 0);

    // Dream RSI: `a` tiết kiệm >= 0, `=2+3=` / `\can ` đếm accepts > 0 và chars_saved.
    let snap = e.dream_rsi_snapshot();
    assert!(snap.accepts >= 3);
}
