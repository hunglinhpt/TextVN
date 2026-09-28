// SPDX-License-Identifier: GPL-3.0-or-later
//! Mô hình composition thuần của TSF adapter (không gọi Win32 — test được mọi OS).
//!
//! **Quyết định kiến trúc:** mọi ký tự của từ đang gõ nằm trong MỘT `ITfComposition`
//! do TIP sở hữu; tới ranh giới từ (Space, dấu câu, Enter, phím điều hướng, chord…)
//! composition được đóng (commit) rồi phím ranh giới mới đi tới app.
//!
//! Lý do: engine trả kết quả theo mô hình "xóa `delete_count` ký tự trước con trỏ
//! rồi chèn `insert`". Với app không hỗ trợ TSF đầy đủ (IMM32 qua CUAS: Win32 Edit,
//! WinForms, Delphi, Java, game…) text store chỉ chứa **composition** — ký tự đã
//! commit không thể xóa lại qua `ITfRange`. Giữ cả từ trong composition khiến mọi
//! `delete_count` của engine rơi vào vùng TIP sở hữu, nên cùng một đường code chạy
//! đúng cho app TSF-aware lẫn app IMM32, không cần gửi Backspace giả (không SendInput).
//!
//! Module này chỉ tính *kế hoạch* ([`CompositionPlan`]); lớp COM (`edit_session.rs`)
//! áp kế hoạch trong `DoEditSession`. Bất biến kiểm chứng bằng test:
//! - phím không bị ăn (`eaten == false`) ⇒ composition đã đóng trước khi app nhận phím
//!   (không còn "chữ treo" khi Enter gửi tin nhắn — bug B2);
//! - `delete_before > 0` chỉ xuất hiện khi engine cần xóa text ngoài composition
//!   (macro/emoji có trigger chứa dấu câu) — lớp COM phải xác minh được mới xóa.

use textvn_ffi::{
    ime_result_v1, ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, IME_FLAG_WORD_END,
};

/// Win VK dùng để phân loại phím (khớp `textvn_core::keymap::vk`).
pub mod vk {
    pub const BACK: u32 = 0x08;
    pub const TAB: u32 = 0x09;
    pub const RETURN: u32 = 0x0D;
    pub const SHIFT: u32 = 0x10;
    pub const CONTROL: u32 = 0x11;
    pub const MENU: u32 = 0x12;
    pub const CAPITAL: u32 = 0x14;
    pub const ESCAPE: u32 = 0x1B;
    pub const LWIN: u32 = 0x5B;
    pub const RWIN: u32 = 0x5C;
    pub const LSHIFT: u32 = 0xA0;
    pub const RMENU: u32 = 0xA5;
    /// Unicode do `SendInput(KEYEVENTF_UNICODE)` bơm vào — không phải phím người gõ.
    pub const PACKET: u32 = 0xE7;
    /// App/IME khác đã xử lý phím này.
    pub const PROCESSKEY: u32 = 0xE5;
}

/// Phân loại phím sau khi adapter đã dịch VK → ký tự theo layout bàn phím.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// Phím sinh ký tự in được (gồm Space).
    Char(char),
    Backspace,
    Escape,
    Enter,
    Tab,
    /// Không sinh ký tự: mũi tên, Delete, Home/End, F-key, PageUp…
    Other,
}

impl KeyKind {
    /// `ch` = kết quả `ToUnicodeEx` (0 nếu không sinh ký tự). Ký tự điều khiển
    /// (`\r`, `\t`, `\b`, ESC) được quy về phím tương ứng; VK không sinh ký tự
    /// tuyệt đối không được coi là chữ (trước đây `ch = vk` biến F1 thành `p`,
    /// Delete thành `.`).
    pub fn classify(vk: u32, ch: u32) -> KeyKind {
        match vk {
            vk::BACK => return KeyKind::Backspace,
            vk::TAB => return KeyKind::Tab,
            vk::RETURN => return KeyKind::Enter,
            vk::ESCAPE => return KeyKind::Escape,
            _ => {}
        }
        match char::from_u32(ch) {
            Some('\n' | '\r') => KeyKind::Enter,
            Some('\t') => KeyKind::Tab,
            Some('\u{8}') => KeyKind::Backspace,
            Some('\u{1b}') => KeyKind::Escape,
            Some(c) if ch != 0 && !c.is_control() => KeyKind::Char(c),
            _ => KeyKind::Other,
        }
    }

    /// `ime_key_v1.ch` gửi engine: chỉ ký tự in được; phím đặc biệt để engine tự
    /// nhận qua `vk` (engine map Enter→`\n`, Tab→`\t`).
    pub fn engine_ch(self) -> u32 {
        match self {
            KeyKind::Char(c) => c as u32,
            _ => 0,
        }
    }

    /// Ký tự ranh giới engine gắn vào cuối `insert` khi RESTORE/COMMIT tại ranh giới.
    fn boundary_char(self) -> Option<char> {
        match self {
            KeyKind::Char(c) => Some(c),
            KeyKind::Enter => Some('\n'),
            KeyKind::Tab => Some('\t'),
            _ => None,
        }
    }

    /// Phím cần edit session khi chưa có composition (có thể sinh text/macro).
    pub fn needs_session_when_idle(self) -> bool {
        matches!(self, KeyKind::Char(_) | KeyKind::Enter | KeyKind::Tab)
    }
}

/// Phím chỉ là modifier/lock: không đóng composition, không tới engine.
pub fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        vk::SHIFT | vk::CONTROL | vk::MENU | vk::CAPITAL | vk::LWIN | vk::RWIN
    ) || (vk::LSHIFT..=vk::RMENU).contains(&vk)
}

fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk, vk::CONTROL | 0xA2 | 0xA3)
}

fn is_shift_vk(vk: u32) -> bool {
    matches!(vk, vk::SHIFT | vk::LSHIFT | 0xA1)
}

/// Phím chuyển kiểu UniKey/EVKey: nhấn Ctrl và Shift (thứ tự bất kỳ) rồi nhả, KHÔNG
/// kèm phím nào khác. Hộp thoại cài đặt và hook đều hứa "Ctrl + Shift"; TSF trước đây
/// chỉ có Ctrl+Shift+Space nên phím chuyển quen thuộc không có tác dụng.
/// Không bao giờ ăn phím modifier — chỉ quan sát.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierToggle {
    armed: bool,
}

impl ModifierToggle {
    /// Key-down bất kỳ. `ctrl_down`/`shift_down` = trạng thái phím còn lại của cặp
    /// (đã được xử lý trước phím hiện tại); `alt_or_win` = Alt/Win đang giữ.
    pub fn on_key_down(&mut self, vk: u32, ctrl_down: bool, shift_down: bool, alt_or_win: bool) {
        self.armed = if is_ctrl_vk(vk) {
            shift_down && !alt_or_win
        } else if is_shift_vk(vk) {
            ctrl_down && !alt_or_win
        } else {
            false
        };
    }

    /// Key-up: `true` đúng MỘT lần khi nhả Ctrl/Shift của một lần bấm hợp lệ
    /// (gọi lại ở pha thứ hai `OnKeyUp` sẽ trả `false`).
    pub fn on_key_up(&mut self, vk: u32) -> bool {
        if (is_ctrl_vk(vk) || is_shift_vk(vk)) && self.armed {
            self.armed = false;
            return true;
        }
        false
    }

    /// Focus đổi / hotkey khác đã xử lý: hủy lần bấm đang chờ.
    pub fn reset(&mut self) {
        self.armed = false;
    }
}

/// Kết quả engine đã giải mã (không phụ thuộc layout C của `ime_result_v1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineStep {
    pub action: u32,
    pub delete_count: u16,
    pub insert: Vec<char>,
    pub flags: u32,
}

impl EngineStep {
    pub fn pass() -> EngineStep {
        EngineStep {
            action: ACTION_PASS,
            delete_count: 0,
            insert: Vec::new(),
            flags: 0,
        }
    }

    pub fn from_result(r: &ime_result_v1) -> EngineStep {
        let len = usize::from(r.insert_len).min(r.insert.len());
        EngineStep {
            action: r.action,
            delete_count: r.delete_count,
            insert: r.insert[..len]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect(),
            flags: r.flags,
        }
    }

    fn word_end(&self) -> bool {
        self.flags & IME_FLAG_WORD_END != 0
    }
}

/// Việc lớp COM phải làm trong edit session cho một phím.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionPlan {
    /// Số ký tự đã commit NGAY TRƯỚC composition phải xóa (macro/emoji vượt ranh giới
    /// composition). Lớp COM phải xác minh dịch chuyển đủ, nếu không → fail-open.
    pub delete_before: u16,
    /// Text composition sau phím này; `None` = giữ nguyên.
    pub text: Option<Vec<char>>,
    /// Đóng composition sau khi áp `text`.
    pub end: bool,
    /// `true` = phím bị ăn; `false` = app nhận phím gốc (sau khi composition đã đóng).
    pub eaten: bool,
}

impl CompositionPlan {
    /// Không làm gì với composition, phím đi thẳng.
    pub fn pass_through(composing: bool) -> CompositionPlan {
        CompositionPlan {
            delete_before: 0,
            text: None,
            end: composing,
            eaten: false,
        }
    }
}

/// Tính kế hoạch cho một phím. `current` = text composition hiện tại (rỗng =
/// không composing). Hàm thuần, không đoán text của app.
pub fn plan_key(current: &[char], key: KeyKind, step: &EngineStep) -> CompositionPlan {
    let composing = !current.is_empty();
    match step.action {
        ACTION_PASS => match key {
            // Engine đã nhận ký tự vào từ nhưng chưa biến đổi → TIP tự đưa vào composition.
            KeyKind::Char(c) if !step.word_end() => {
                let mut text = current.to_vec();
                text.push(c);
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end: false,
                    eaten: true,
                }
            }
            // Engine đã bỏ 1 ký tự chưa biến đổi khỏi từ → bỏ khỏi composition.
            KeyKind::Backspace if composing => {
                let mut text = current.to_vec();
                text.pop();
                let end = text.is_empty();
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end,
                    eaten: true,
                }
            }
            // Ranh giới / phím điều hướng: commit nguyên văn rồi để app xử lý phím.
            _ => CompositionPlan::pass_through(composing),
        },
        ACTION_REPLACE | ACTION_RESTORE | ACTION_COMMIT => {
            let (mut text, delete_before) = if step.action == ACTION_COMMIT {
                (current.to_vec(), 0)
            } else {
                let del = usize::from(step.delete_count);
                if del <= current.len() {
                    (current[..current.len() - del].to_vec(), 0)
                } else {
                    (Vec::new(), (del - current.len()) as u16)
                }
            };
            text.extend_from_slice(&step.insert);
            let mut eaten = true;
            // RESTORE/COMMIT ở ranh giới: engine đặt ký tự ranh giới cuối `insert`.
            // Bỏ nó ra và để app nhận phím gốc — Enter phải là phím Enter thật
            // (chat gửi tin), không phải ký tự `\n` chèn vào text.
            // REPLACE có WORD_END là macro/emoji: trigger bị nuốt theo P0-3 §1.1.
            if step.word_end() && step.action != ACTION_REPLACE {
                if let Some(b) = key.boundary_char() {
                    if text.last() == Some(&b) {
                        text.pop();
                        eaten = false;
                    }
                }
            }
            let end = step.word_end() || text.is_empty() || !eaten;
            CompositionPlan {
                delete_before,
                text: Some(text),
                end,
                eaten,
            }
        }
        // Action lạ (ABI mới hơn adapter): an toàn nhất là commit và để phím đi thẳng.
        _ => CompositionPlan::pass_through(composing),
    }
}

/// Text sau khi áp kế hoạch — dùng cho model của adapter sau khi COM áp thành công.
pub fn text_after(current: &[char], plan: &CompositionPlan) -> Vec<char> {
    if plan.end {
        return Vec::new();
    }
    plan.text.clone().unwrap_or_else(|| current.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineSession;

    /// Tài liệu mô phỏng: `committed` + composition ở cuối, caret sau composition.
    #[derive(Default)]
    struct Doc {
        committed: Vec<char>,
        comp: Vec<char>,
    }

    impl Doc {
        fn text(&self) -> String {
            self.committed.iter().chain(self.comp.iter()).collect()
        }
    }

    fn press(doc: &mut Doc, engine: &mut EngineSession, vk: u32, ch: u32) -> CompositionPlan {
        let key = KeyKind::classify(vk, ch);
        let r = engine
            .key_event_raw(vk, key.engine_ch(), 0)
            .expect("ime_key");
        let plan = plan_key(&doc.comp, key, &EngineStep::from_result(&r));
        if plan.delete_before > 0 {
            let n = usize::from(plan.delete_before);
            assert!(
                n <= doc.committed.len(),
                "delete_before vượt text đã commit"
            );
            let keep = doc.committed.len() - n;
            doc.committed.truncate(keep);
        }
        if let Some(t) = &plan.text {
            doc.comp = t.clone();
        }
        if plan.end {
            let comp = std::mem::take(&mut doc.comp);
            doc.committed.extend(comp);
        }
        if !plan.eaten {
            assert!(
                doc.comp.is_empty(),
                "phím đi tới app khi composition còn mở: {:?}",
                doc.comp
            );
            match key {
                KeyKind::Char(c) => doc.committed.push(c),
                KeyKind::Enter => doc.committed.push('\n'),
                KeyKind::Tab => doc.committed.push('\t'),
                KeyKind::Backspace => {
                    doc.committed.pop();
                }
                KeyKind::Escape | KeyKind::Other => {}
            }
        }
        plan
    }

    fn type_str(doc: &mut Doc, engine: &mut EngineSession, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => press(doc, engine, vk::RETURN, 0),
                '\u{8}' => press(doc, engine, vk::BACK, 0),
                '\u{1b}' => press(doc, engine, vk::ESCAPE, 0),
                '\t' => press(doc, engine, vk::TAB, 0),
                ' ' => press(doc, engine, 0x20, ' ' as u32),
                c => press(doc, engine, 0, c as u32),
            };
        }
    }

    fn run(config: &[u8], input: &str) -> Doc {
        let mut engine = EngineSession::new().unwrap();
        if !config.is_empty() {
            engine.reload_config(config).unwrap();
        }
        let mut doc = Doc::default();
        type_str(&mut doc, &mut engine, input);
        doc
    }

    const NO_CAPS: &[u8] = br#"{"config_version":1,"auto_capitalize":false}"#;

    #[test]
    fn classify_never_turns_non_char_vk_into_letters() {
        assert_eq!(KeyKind::classify(0x70, 0), KeyKind::Other); // F1 ≠ 'p'
        assert_eq!(KeyKind::classify(0x2E, 0), KeyKind::Other); // Delete ≠ '.'
        assert_eq!(KeyKind::classify(0x25, 0), KeyKind::Other); // Left ≠ '%'
        assert_eq!(KeyKind::classify(0x0D, '\r' as u32), KeyKind::Enter);
        assert_eq!(KeyKind::classify(0x08, 8), KeyKind::Backspace);
        assert_eq!(KeyKind::classify(0x41, 'a' as u32), KeyKind::Char('a'));
        assert_eq!(KeyKind::classify(0x20, ' ' as u32), KeyKind::Char(' '));
        assert_eq!(KeyKind::Enter.engine_ch(), 0);
        assert!(is_modifier_vk(vk::SHIFT) && is_modifier_vk(0xA3) && !is_modifier_vk(0x41));
    }

    #[test]
    fn telex_word_is_composed_then_committed_on_space() {
        let doc = run(NO_CAPS, "dduocj ");
        assert_eq!(doc.text(), "được ");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn composition_holds_whole_word_until_boundary() {
        let doc = run(NO_CAPS, "Vieetj");
        assert_eq!(doc.committed, Vec::<char>::new(), "chưa commit giữa từ");
        assert_eq!(doc.text(), "Việt");
        let doc = run(NO_CAPS, "Vieetj Nam");
        assert_eq!(doc.text(), "Việt Nam");
    }

    #[test]
    fn english_letters_pass_unchanged() {
        assert_eq!(run(NO_CAPS, "hello world ").text(), "hello world ");
    }

    #[test]
    fn enter_commits_before_app_receives_key_b2() {
        let doc = run(NO_CAPS, "chaof banj\n");
        assert_eq!(doc.text(), "chào bạn\n");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn backspace_inside_word_refolds_and_empty_word_ends_composition() {
        let doc = run(NO_CAPS, "dduocj\u{8}");
        assert_eq!(doc.text(), "đươc");
        let doc = run(NO_CAPS, "ab\u{8}\u{8}\u{8}");
        assert_eq!(doc.text(), "");
        let doc = run(NO_CAPS, "ab \u{8}\u{8}");
        assert_eq!(
            doc.text(),
            "a",
            "Backspace sau commit là Backspace thật của app"
        );
    }

    #[test]
    fn escape_restores_raw_keys() {
        let doc = run(NO_CAPS, "dduocj\u{1b}");
        assert_eq!(doc.text(), "dduocj");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn auto_restore_english_keeps_boundary_key_native() {
        let cfg = br#"{"config_version":1,"auto_capitalize":false,"auto_restore_english":true}"#;
        assert_eq!(run(cfg, "asdf ").text(), "asdf ");
        assert_eq!(run(cfg, "asdf\n").text(), "asdf\n");
    }

    #[test]
    fn digits_and_punctuation_are_boundaries_in_telex() {
        assert_eq!(run(NO_CAPS, "vieetj2026.").text(), "việt2026.");
    }

    #[test]
    fn vni_markers_stay_inside_composition() {
        let cfg = br#"{"config_version":1,"method":"vni","auto_capitalize":false}"#;
        assert_eq!(run(cfg, "tie6ng1 Vie65t ").text(), "tiếng Việt ");
    }

    #[test]
    fn navigation_key_commits_as_is_and_starts_new_word() {
        let mut engine = EngineSession::new().unwrap();
        engine.reload_config(NO_CAPS).unwrap();
        let mut doc = Doc::default();
        type_str(&mut doc, &mut engine, "vie");
        let plan = press(&mut doc, &mut engine, 0x25, 0); // Left
        assert!(!plan.eaten && plan.end);
        assert_eq!(doc.text(), "vie");
        type_str(&mut doc, &mut engine, "ej");
        // "ej" là từ mới → "ẹ"; "vie" đã commit giữ nguyên (không thành "viẹ").
        assert_eq!(doc.committed.iter().collect::<String>(), "vie");
        assert_eq!(doc.text(), "vieẹ");
    }

    #[test]
    fn macro_trigger_is_swallowed_and_expanded() {
        let cfg = r#"{"config_version":1,"auto_capitalize":false,"macro_trigger":"tab","macros":[{"trigger":"cty","expand":"Công ty TNHH","when":"always"}]}"#;
        let doc = run(cfg.as_bytes(), "cty\t");
        assert_eq!(doc.text(), "Công ty TNHH");
    }

    #[test]
    fn auto_capitalize_goes_into_composition() {
        let cfg = br#"{"config_version":1,"auto_capitalize":true}"#;
        assert_eq!(run(cfg, "chao. ban").text(), "chao. Ban");
    }

    #[test]
    fn ctrl_shift_tap_toggles_once_and_only_without_other_keys() {
        let mut t = ModifierToggle::default();
        // Ctrl ↓, Shift ↓, Shift ↑ → toggle; pha thứ hai (OnKeyUp) không toggle lại.
        t.on_key_down(0xA2, false, false, false);
        t.on_key_down(0xA0, true, false, false);
        assert!(t.on_key_up(0xA0));
        assert!(!t.on_key_up(0xA0));
        assert!(!t.on_key_up(0xA2));

        // Shift trước Ctrl cũng được; auto-repeat của modifier giữ trạng thái.
        t.on_key_down(vk::SHIFT, false, false, false);
        t.on_key_down(vk::CONTROL, false, true, false);
        t.on_key_down(vk::CONTROL, false, true, false);
        assert!(t.on_key_up(vk::CONTROL));

        // Ctrl+Shift+Z (redo) / Ctrl+Shift+Space: phím thứ ba hủy lần bấm.
        t.on_key_down(vk::CONTROL, false, false, false);
        t.on_key_down(vk::SHIFT, true, false, false);
        t.on_key_down(0x5A, true, true, false);
        assert!(!t.on_key_up(vk::SHIFT));

        // Kèm Alt/Win không tính; reset() hủy khi focus đổi.
        t.on_key_down(vk::SHIFT, true, false, true);
        assert!(!t.on_key_up(vk::SHIFT));
        t.on_key_down(vk::SHIFT, true, false, false);
        t.reset();
        assert!(!t.on_key_up(vk::SHIFT));

        // Chỉ một modifier: không toggle.
        t.on_key_down(vk::SHIFT, false, false, false);
        assert!(!t.on_key_up(vk::SHIFT));
    }

    #[test]
    fn plan_invariant_unknown_action_is_fail_open() {
        let step = EngineStep {
            action: 99,
            delete_count: 3,
            insert: vec!['x'],
            flags: 0,
        };
        let plan = plan_key(&['a', 'b'], KeyKind::Char('c'), &step);
        assert_eq!(plan, CompositionPlan::pass_through(true));
    }

    #[test]
    fn delete_beyond_composition_is_reported_not_guessed() {
        let step = EngineStep {
            action: ACTION_REPLACE,
            delete_count: 4,
            insert: vec!['😀'],
            flags: IME_FLAG_WORD_END,
        };
        let plan = plan_key(&['a'], KeyKind::Tab, &step);
        assert_eq!(plan.delete_before, 3);
        assert_eq!(plan.text, Some(vec!['😀']));
        assert!(plan.end && plan.eaten);
        assert!(text_after(&['a'], &plan).is_empty());
    }
}
