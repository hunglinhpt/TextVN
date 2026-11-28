//! R2-171 — tự viết hoa đầu câu sau Enter phải sống sót qua `reset` của toolkit.
//!
//! GtkTextView (GTK3/GTK4) gọi `gtk_im_context_reset` ngay khi chèn dòng mới, nên IBus
//! `reset` / Fcitx5 `InputContextReset` tới engine GIỮA Enter và chữ kế tiếp
//! (fcitx5-lotus#597: `ban` Enter `tooi` ra `tôi` thay vì `Tôi`). Adapter Linux nhận ra reset
//! đó (tới < `LC_RESET_AFTER_ENTER_US` sau Enter, không phím/focus xen giữa) và gọi
//! `reset_with_tail("\n")` = "con trỏ ở đầu dòng". `Engine::reset` thường (click, đổi focus,
//! hay Enter rồi click vào giữa câu cũ) vẫn xoá sạch ⇒ KHÔNG tự viết hoa (không đoán ngữ cảnh).
//!
//! Cùng vòng đối chiếu (P3-8 §7): R2-172 — địa danh `Đắk Lắk`, `Krông Búk` (xkey#135), cuối file.

use textvn_core::keymap::vk;
use textvn_core::sim::apply;
use textvn_core::{Engine, EngineOptions, KeyEvent};

fn opts() -> EngineOptions {
    EngineOptions {
        auto_capitalize: true,
        ..Default::default()
    }
}

fn type_str(e: &mut Engine, s: &str, buf: &mut Vec<char>) {
    for c in s.chars() {
        let out = e.key(&KeyEvent::char_down(c));
        apply(buf, &out.action, &KeyEvent::char_down(c));
    }
}

fn enter(e: &mut Engine, buf: &mut Vec<char>) {
    let out = e.key(&KeyEvent::key_down(vk::RETURN));
    // Engine nhận Enter thật (vk RETURN, không ký tự); app chèn xuống dòng khi Pass.
    apply(buf, &out.action, &KeyEvent::char_down('\n'));
}

fn text(buf: &[char]) -> String {
    buf.iter().collect()
}

#[test]
fn caps_after_enter_survives_toolkit_reset() {
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "ban", &mut buf);
    enter(&mut e, &mut buf);
    e.reset_with_tail("\n"); // adapter: reset GtkTextView ngay sau khi chèn dòng mới
    type_str(&mut e, "tooi ", &mut buf);
    assert_eq!(text(&buf), "ban\nTôi ");
}

#[test]
fn enter_then_click_elsewhere_does_not_capitalize() {
    // Enter rồi click vào giữa câu cũ: reset thường ⇒ không viết hoa nhầm.
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "ban", &mut buf);
    enter(&mut e, &mut buf);
    e.reset();
    type_str(&mut e, "tooi ", &mut buf);
    assert_eq!(text(&buf), "ban\ntôi ");
}

#[test]
fn caps_after_enter_without_reset_unchanged() {
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "ban", &mut buf);
    enter(&mut e, &mut buf);
    type_str(&mut e, "tooi ", &mut buf);
    assert_eq!(text(&buf), "ban\nTôi ");
}

#[test]
fn reset_mid_sentence_does_not_capitalize() {
    // Click/đổi focus giữa câu: không biết ngữ cảnh ⇒ giữ chữ thường như trước.
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "xin chaof ", &mut buf);
    e.reset();
    type_str(&mut e, "ban ", &mut buf);
    assert_eq!(text(&buf), "xin chào ban ");
}

#[test]
fn reset_after_sentence_end_space_does_not_capitalize() {
    // Chỉ mốc ĐẦU DÒNG được giữ qua reset; ". " rồi click sang chỗ khác ⇒ không đoán.
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "xong. ", &mut buf);
    e.reset();
    type_str(&mut e, "ban ", &mut buf);
    assert_eq!(text(&buf), "xong. ban ");
}

#[test]
fn double_reset_after_enter_still_capitalizes() {
    // GTK có thể reset nhiều lần trong cùng phím Enter; adapter gọi lại `reset_with_tail`.
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    type_str(&mut e, "ban", &mut buf);
    enter(&mut e, &mut buf);
    e.reset_with_tail("\n");
    e.reset_with_tail("\n");
    type_str(&mut e, "dduowcj ", &mut buf);
    assert_eq!(text(&buf), "ban\nĐược ");
}

#[test]
fn auto_capitalize_off_is_untouched() {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    let mut buf = Vec::new();
    type_str(&mut e, "ban", &mut buf);
    enter(&mut e, &mut buf);
    e.reset_with_tail("\n");
    type_str(&mut e, "tooi ", &mut buf);
    assert_eq!(text(&buf), "ban\ntôi ");
}

#[test]
fn other_tails_with_control_chars_still_rejected() {
    // R2-131 giữ nguyên: tail có ký tự điều khiển khác "\n" ⇒ chỉ reset.
    let mut e = Engine::new(opts());
    let mut buf = Vec::new();
    e.reset_with_tail("a\nb");
    type_str(&mut e, "tooi ", &mut buf);
    assert_eq!(text(&buf), "tôi ");
}

// ---- R2-172 (xkey#135): địa danh `Đắk Lắk`, `Krông Búk` — âm cuối `k` / âm đầu `kr` ----

fn typed(s: &str) -> String {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    let mut buf = Vec::new();
    type_str(&mut e, s, &mut buf);
    text(&buf)
}

#[test]
fn r2_172_place_names_type_in_both_tone_orders() {
    assert_eq!(typed("DDawks Lawks "), "Đắk Lắk ");
    assert_eq!(typed("DDawsk Lawsk "), "Đắk Lắk ");
    assert_eq!(typed("DDawk Glei "), "Đăk Glei ");
    assert_eq!(typed("Kroong Buks "), "Krông Búk ");
    assert_eq!(typed("Kroong Pawsc "), "Krông Pắc ");
}

/// Luật hẹp: mọi từ Anh trong `en_words10k.txt` có dạng mà luật mới chạm tới (`awk`, `usk`,
/// `uks`, mở đầu `kr`) vẫn gõ ra đúng nó (`hawk`, `kruger`, `dusk`, `husk`…). `busk` (hát
/// rong, không có trong danh sách) là đánh đổi có chủ đích để gõ được `Krông Búk`.
#[test]
fn r2_172_english_words_untouched() {
    const SOURCE: &str = include_str!("../../data/en_words10k.txt");
    let mut checked = 0;
    for w in SOURCE.lines().map(str::trim).filter(|w| {
        !w.is_empty()
            && !w.starts_with('#')
            && w.chars().all(|c| c.is_ascii_lowercase())
            && (w.contains("awk") || w.contains("usk") || w.contains("uks") || w.starts_with("kr"))
    }) {
        assert_eq!(typed(&format!("{w} ")), format!("{w} "), "{w}");
        checked += 1;
    }
    for w in [
        "dusk", "tusk", "husk", "musk", "hawk", "krill", "kraken", "back", "desk",
    ] {
        assert_eq!(typed(&format!("{w} ")), format!("{w} "), "{w}");
        checked += 1;
    }
    assert!(checked >= 9);
}
