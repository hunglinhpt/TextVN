// SPDX-License-Identifier: GPL-3.0-or-later
//! Integration tests cho P12 — Danh mục Emoji nhẹ, mở rộng `:shortcode:`,
//! Dream RSI (G19) và bảo vệ ngữ cảnh (ô mật khẩu, terminal, công thức toán, URL/code).

use textvn_core::keymap::{self, KeyEvent};
use textvn_core::{Action, Context, Engine, EngineOptions, FLAG_CONSUMED};

fn type_stream(e: &mut Engine, s: &str) -> String {
    let mut doc = Vec::<char>::new();
    for c in s.chars() {
        let ev = match c {
            ' ' => KeyEvent::key_down(keymap::vk::SPACE),
            '\n' => KeyEvent::key_down(keymap::vk::RETURN),
            '\t' => KeyEvent::key_down(keymap::vk::TAB),
            _ => KeyEvent::char_down(c),
        };
        let out = e.key(&ev);
        match out.action {
            Action::Pass => {
                if out.flags & FLAG_CONSUMED == 0 {
                    doc.push(c);
                }
            }
            Action::Commit { insert } => doc.extend(insert),
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                let del = usize::from(delete_count).min(doc.len());
                doc.truncate(doc.len() - del);
                doc.extend(insert);
            }
        }
    }
    doc.into_iter().collect()
}

#[test]
fn p12_inline_shortcodes_work_in_telex_and_english_mode() {
    // 1) Chế độ Telex VN: `:fire:` bị Telex tạm đổi `r` -> `fỉe`, vẫn phải mở rộng thành `🔥`.
    let mut e_vi = Engine::new(EngineOptions::default());
    let before = e_vi.dream_rsi_snapshot();
    assert_eq!(type_stream(&mut e_vi, "chào :fire: "), "chào 🔥 ");
    assert_eq!(type_stream(&mut e_vi, ":heart_eyes: "), "😍 ");
    assert_eq!(type_stream(&mut e_vi, ":camon: "), "🙏 ");
    assert_eq!(type_stream(&mut e_vi, ":100: "), "💯 ");
    let after = e_vi.dream_rsi_snapshot();
    assert!(
        after.accepts >= before.accepts + 4 && after.chars_saved >= before.chars_saved + 4,
        "Dream RSI (G19) phải ghi nhận mỗi lần mở rộng emoji"
    );

    // MRU đưa emoji vừa dùng lên đầu nhóm 0 (Gần đây).
    let recent = e_vi.emoji_search("", 0, 10);
    assert_eq!(recent[0].glyph, "💯");
    assert_eq!(recent[1].glyph, "🙏");

    // 2) Chế độ EN (`enabled = false`): `:smile:` và `:vn:` vẫn hoạt động.
    let opts_en = EngineOptions {
        enabled: false,
        ..Default::default()
    };
    let mut e_en = Engine::new(opts_en);
    e_en.set_context(Context {
        enabled: false,
        ..Default::default()
    });
    assert_eq!(type_stream(&mut e_en, "hello :smile: :vn:"), "hello 😄 🇻🇳");
}

#[test]
fn p12_inline_shortcodes_blocked_in_secure_terminal_math_and_urls() {
    // 1) Ô mật khẩu (`secure = true`): tuyệt đối không bung emoji (G8).
    let mut e_sec = Engine::new(EngineOptions::default());
    e_sec.set_context(Context {
        enabled: true,
        secure: true,
        ..Default::default()
    });
    assert_eq!(type_stream(&mut e_sec, ":fire:"), ":fire:");

    // 2) Terminal (`IME_FIELD_TERMINAL`): không can thiệp vào lệnh vim/sed.
    let mut e_term = Engine::new(EngineOptions::default());
    e_term.set_context(Context {
        enabled: true,
        field_role: 9, // IME_FIELD_TERMINAL
        ..Default::default()
    });
    assert_eq!(type_stream(&mut e_term, ":ok:"), ":ok:");

    // 3) Trong công thức toán `$...$` (khi Math bật): không bung emoji.
    let mut opts_math = EngineOptions::default();
    opts_math.math.enabled = true;
    let mut e_math = Engine::new(opts_math);
    assert_eq!(type_stream(&mut e_math, "$a :ok: b$"), "$a :ok: b$");

    // 4) Không kích hoạt nhầm trên URL / giờ / namespace `::` / dính liền chữ.
    let mut e_fp = Engine::new(EngineOptions::default());
    assert_eq!(type_stream(&mut e_fp, "http://a:ok: "), "http://a:ok: ");
    assert_eq!(type_stream(&mut e_fp, "std::ok: "), "std::ok: ");
    assert_eq!(type_stream(&mut e_fp, "foo:ok: "), "foo:ok: ");
    assert_eq!(type_stream(&mut e_fp, "12:30:00 "), "12:30:00 ");
}

#[test]
fn p12_picker_record_use_updates_mru_and_dream_rsi() {
    let mut e = Engine::new(EngineOptions::default());
    let before = e.dream_rsi_snapshot();
    assert!(!e.emoji_record_use("not-an-emoji"));
    assert_eq!(e.dream_rsi_snapshot().accepts, before.accepts);

    assert!(e.emoji_record_use("🚀"));
    let after = e.dream_rsi_snapshot();
    assert_eq!(after.accepts, before.accepts + 1);
    assert_eq!(after.chars_saved, before.chars_saved + 1);
    assert_eq!(e.emoji_search("", 0, 5)[0].glyph, "🚀");
}
