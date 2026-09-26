// SPDX-License-Identifier: GPL-3.0-or-later
//! `vietime-core` — engine gõ tiếng Việt, thuần logic (P0-1 §2).
//!
//! Bất biến (Handbook S1–S4, P0-2 §0):
//! - **Không** network / file I/O / spawn thread / log nội dung text.
//! - `key()` luôn trả `Outcome` — tầng FFI lo catch_unwind → fail-open.
//! - Không global mutable state: mọi trạng thái nằm trong `Engine`.

pub mod buffer;
pub mod keymap;
pub mod method;
pub mod transform;

pub use keymap::KeyEvent;
pub use method::Method;
pub use strategy::{ActionKind, Strategy};
pub use transform::DiacriticStyle;

/// `ime_result_v1.action` — khớp hằng số P0-2 §1.
pub const ACTION_PASS: u32 = 0;
pub const ACTION_REPLACE: u32 = 1;
pub const ACTION_COMMIT: u32 = 2;
pub const ACTION_RESTORE: u32 = 3;

/// `ime_result_v1.flags`.
pub const FLAG_CONSUMED: u32 = 0x1;
pub const FLAG_WORD_END: u32 = 0x2;
pub const FLAG_ERROR: u32 = 0x4;
pub const FLAG_SUGGEST: u32 = 0x8;

/// Kết quả xử lý 1 phím — adapter dịch sang `ime_result_v1` (P0-2 §2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Pass,
    /// Xóa `delete_count` ký tự trước con trỏ rồi chèn `insert`.
    Replace {
        delete_count: u16,
        insert: Vec<char>,
    },
    /// Chèn `insert` (không xóa), đóng preedit — Enter trong chat (bug B2).
    Commit {
        insert: Vec<char>,
    },
    /// Gỡ biến đổi: xóa `delete_count`, chèn lại chuỗi gốc (bug B5 / ESC).
    Restore {
        delete_count: u16,
        insert: Vec<char>,
    },
}

impl Action {
    pub fn kind(&self) -> ActionKind {
        match self {
            Action::Pass => ActionKind::Pass,
            Action::Replace { .. } => ActionKind::Replace,
            Action::Commit { .. } => ActionKind::Commit,
            Action::Restore { .. } => ActionKind::Restore,
        }
    }
}

/// Outcome 1 phím: action + preedit đầy đủ + flags (P0-2 §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub action: Action,
    /// Chuỗi hoàn chỉnh adapter phải hiển thị (không phải delta).
    pub preedit: Vec<char>,
    pub flags: u32,
}

impl Outcome {
    fn pass() -> Outcome {
        Outcome {
            action: Action::Pass,
            preedit: Vec::new(),
            flags: 0,
        }
    }
}

/// Options từ `config.v1` (vietime-config đã validate — ffi dịch sang đây).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOptions {
    pub method: Method,
    pub diacritic_style: DiacriticStyle,
    pub free_marking: bool,
    /// Master switch `config.enabled`.
    pub enabled: bool,
    /// Dự trữ cho `post/restore_en.rs` (bug B5) — slice sau.
    pub auto_restore_english: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            method: Method::Telex,
            diacritic_style: DiacriticStyle::New,
            free_marking: true,
            enabled: true,
            auto_restore_english: true,
        }
    }
}

/// Context hiện tại (`ime_context_v1` — adapter điền, P0-2 §1).
#[derive(Debug, Clone)]
pub struct Context {
    pub enabled: bool,
    pub secure: bool,
    pub field_role: u32,
    pub caps: u32,
    /// Strategy adapter gợi ý; -1 = engine tự resolve.
    pub hint: i64,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            enabled: true,
            secure: false,
            field_role: 0,
            caps: 0,
            hint: -1,
        }
    }
}

/// Engine — 1 instance = 1 thread (P0-2 §3).
pub struct Engine {
    opts: EngineOptions,
    ctx: Context,
    word: buffer::Word,
}

impl Engine {
    pub fn new(opts: EngineOptions) -> Engine {
        Engine {
            opts,
            ctx: Context::default(),
            word: buffer::Word::default(),
        }
    }

    /// `ime_reload_config` — đổi options giữa chừng → reset từ đang gõ dở.
    pub fn set_options(&mut self, opts: EngineOptions) {
        self.opts = opts;
        self.word.clear();
    }

    pub fn options(&self) -> &EngineOptions {
        &self.opts
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.ctx = ctx;
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// `ime_reset` — xóa trạng thái từ (focus change). Không đụng buffer của app.
    pub fn reset(&mut self) {
        self.word.clear();
    }

    /// Resolve strategy hiện tại (P0-3 §3.1).
    pub fn strategy(&self) -> Strategy {
        strategy::resolve(strategy::ResolveInput {
            secure: self.ctx.secure,
            enabled: self.opts.enabled && self.ctx.enabled,
            user_preset: None,
            system_preset: None,
            hint: self.ctx.hint,
            field_role: self.ctx.field_role,
            caps: self.ctx.caps,
        })
    }

    fn preedit(&self) -> Vec<char> {
        if self.word.active {
            self.word.display.clone()
        } else {
            Vec::new()
        }
    }

    fn fold_current(&self) -> Vec<char> {
        method::fold(
            &self.word.raw,
            self.opts.method,
            self.opts.diacritic_style,
            self.opts.free_marking,
        )
    }
}

/// Xử lý 1 phím — luôn trả Outcome (fail-open do FFI đảm nhiệm, engine không panic).
impl Engine {
    pub fn key(&mut self, k: &KeyEvent) -> Outcome {
        // Key-up: không transform, giữ nguyên trạng thái.
        if !k.key_down {
            return Outcome {
                action: Action::Pass,
                preedit: self.preedit(),
                flags: 0,
            };
        }
        // Chống loop (P0-3 §6): phím adapter tự bơm → PASS tuyệt đối.
        if k.is_injected {
            return Outcome::pass();
        }
        // Chord hệ thống / modifier đơn (S9, bug B6) — không bao giờ nuốt.
        if k.is_chord() || k.is_modifier() {
            return Outcome::pass();
        }

        let strategy = self.strategy();
        if strategy == Strategy::Passthrough {
            if !self.word.is_empty() {
                self.word.clear();
            }
            return Outcome::pass();
        }

        match k.vk {
            keymap::vk::BACK => return self.on_backspace(),
            keymap::vk::ESCAPE => return self.on_escape(),
            keymap::vk::RETURN | keymap::vk::SPACE | keymap::vk::TAB => {
                let c = k.printable().unwrap_or(' ');
                return self.on_boundary(c, strategy);
            }
            _ => {}
        }

        let Some(c) = k.printable() else {
            // Delete/arrow/F-key…: PASS, từ giữ nguyên
            return Outcome::pass();
        };

        if !c.is_alphabetic() {
            // Số/punctuation: đóng từ (nếu có) rồi cho ký tự đi thẳng.
            if self.word.is_empty() {
                return Outcome::pass();
            }
            return self.on_boundary(c, strategy);
        }

        // --- Chữ cái: fold path ---
        self.word.raw.push(c);
        let display = self.fold_current();

        if !self.word.active && display == self.word.raw {
            // Chưa có transform nào → chữ đi thẳng (tiếng Anh không đổi — WIN-011).
            self.word.passed.push(c);
            self.word.display = display;
            return Outcome::pass();
        }

        let delete = if self.word.active {
            self.word.owned as u16
        } else {
            self.word.pending_delete() // activate: xóa những ký tự đã PASS trong từ
        };
        self.word.active = true;
        self.word.passed.clear();
        self.word.display = display.clone();
        self.word.owned = display.len();
        Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: display.clone(),
            },
            preedit: display,
            flags: FLAG_CONSUMED,
        }
    }

    /// Ranh giới từ: Space/Enter/Tab/punct (P0-2 §4, bug B2 cho Enter+Preedit).
    fn on_boundary(&mut self, c: char, strategy: Strategy) -> Outcome {
        let was_active = self.word.active;
        self.word.clear();
        if !was_active {
            return Outcome {
                action: Action::Pass,
                preedit: Vec::new(),
                flags: FLAG_WORD_END,
            };
        }
        match strategy {
            // Preedit: đóng composition (preedit thành text) rồi chèn ký tự ranh giới.
            Strategy::Preedit => Outcome {
                action: Action::Commit { insert: vec![c] },
                preedit: Vec::new(),
                flags: FLAG_CONSUMED | FLAG_WORD_END,
            },
            // Các strategy khác: từ đã nằm trong document → chỉ cho ranh giới đi qua.
            _ => Outcome {
                action: Action::Pass,
                preedit: Vec::new(),
                flags: FLAG_WORD_END,
            },
        }
    }

    /// ESC: khôi phục chuỗi đã gõ (bug B5 / PLAN §8 "ESC khôi phục").
    fn on_escape(&mut self) -> Outcome {
        if !self.word.active {
            return Outcome::pass();
        }
        let raw = self.word.raw.clone();
        let delete = self.word.owned as u16;
        self.word.clear();
        Outcome {
            action: Action::Restore {
                delete_count: delete,
                insert: raw,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        }
    }

    /// Backspace: từ đang active → fold lại từ chưa; chưa active → bỏ ký tự đã PASS.
    fn on_backspace(&mut self) -> Outcome {
        if self.word.active {
            self.word.raw.pop();
            let delete = self.word.owned as u16;
            if self.word.raw.is_empty() {
                self.word.clear();
                return Outcome {
                    action: Action::Replace {
                        delete_count: delete,
                        insert: Vec::new(),
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED,
                };
            }
            let display = self.fold_current();
            self.word.display = display.clone();
            self.word.owned = display.len();
            return Outcome {
                action: Action::Replace {
                    delete_count: delete,
                    insert: display.clone(),
                },
                preedit: display,
                flags: FLAG_CONSUMED,
            };
        }
        // Chưa activate: bỏ 1 ký tự đã gõ thẳng
        self.word.raw.pop();
        self.word.passed.pop();
        self.word.display.clear();
        Outcome::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keymap::{MOD_CTRL, MOD_SHIFT};

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
                let del = *delete_count as usize;
                assert!(del <= buf.len(), "delete_count vượt buffer (đừng mask bug)");
                buf.truncate(buf.len() - del);
                buf.extend_from_slice(insert);
            }
            Action::Commit { insert } => buf.extend_from_slice(insert),
        }
    }

    /// Gõ cả down+up từng ký tự (đúng mô tả P0-4 §2.2), áp action vào buffer.
    fn type_keys(e: &mut Engine, input: &str) -> (String, String) {
        let mut buf: Vec<char> = Vec::new();
        let mut preedit = String::new();
        for c in input.chars() {
            for down in [true, false] {
                let k = KeyEvent {
                    ch: c as u32,
                    key_down: down,
                    ..Default::default()
                };
                let o = e.key(&k);
                if down {
                    apply(&mut buf, &o.action, &k);
                    preedit = o.preedit.iter().collect();
                }
            }
        }
        (buf.into_iter().collect(), preedit)
    }

    fn engine() -> Engine {
        Engine::new(EngineOptions::default())
    }

    #[test]
    fn golden_duocj_with_preedit() {
        let mut e = engine();
        let (buf, preedit) = type_keys(&mut e, "duocj");
        assert_eq!(buf, "được");
        assert_eq!(preedit, "được");
    }

    #[test]
    fn english_passes_through_untouched() {
        let mut e = engine();
        let (buf, preedit) = type_keys(&mut e, "hello");
        assert_eq!(buf, "hello");
        assert_eq!(preedit, "");
    }

    #[test]
    fn secure_context_all_pass() {
        let mut e = engine();
        e.set_context(Context {
            secure: true,
            ..Default::default()
        });
        let (buf, _) = type_keys(&mut e, "ass");
        assert_eq!(buf, "ass");
        assert_eq!(e.strategy(), Strategy::Passthrough);
    }

    #[test]
    fn disabled_all_pass() {
        let mut e = engine();
        e.set_options(EngineOptions {
            enabled: false,
            ..Default::default()
        });
        let (buf, _) = type_keys(&mut e, "duocj");
        assert_eq!(buf, "duocj");
    }

    #[test]
    fn injected_key_always_pass() {
        let mut e = engine();
        let k = KeyEvent {
            ch: 's' as u32,
            key_down: true,
            is_injected: true,
            ..Default::default()
        };
        assert_eq!(e.key(&k).action, Action::Pass);
    }

    #[test]
    fn chord_never_eaten() {
        let mut e = engine();
        let k = KeyEvent {
            vk: keymap::vk::SPACE,
            mods: MOD_CTRL | MOD_SHIFT,
            key_down: true,
            ..Default::default()
        };
        assert_eq!(e.key(&k).action, Action::Pass);
    }

    #[test]
    fn backspace_folds_back() {
        let mut e = engine();
        let (buf, _) = type_keys(&mut e, "duocj");
        assert_eq!(buf, "được");
        // Backspace ×4 → raw "duoc"→"đươc", "duo"→"đuơ", "du"→"đu", "d"→"d"
        let mut buf: Vec<char> = buf.chars().collect();
        for _ in 0..4 {
            let k = KeyEvent::key_down(keymap::vk::BACK);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        assert_eq!(buf.into_iter().collect::<String>(), "d");
    }

    #[test]
    fn escape_restores_raw() {
        let mut e = engine();
        let (buf, _) = type_keys(&mut e, "duocj");
        assert_eq!(buf, "được");
        let mut buf: Vec<char> = buf.chars().collect();
        let k = KeyEvent::key_down(keymap::vk::ESCAPE);
        let o = e.key(&k);
        assert_eq!(o.action.kind(), ActionKind::Restore);
        apply(&mut buf, &o.action, &k);
        assert_eq!(buf.into_iter().collect::<String>(), "duocj");
    }

    #[test]
    fn boundary_space_pass_with_default_caps() {
        let mut e = engine(); // caps=0 → BackspaceType → ranh giới = PASS
        let mut buf: Vec<char> = Vec::new();
        for c in "duocj".chars() {
            let k = KeyEvent::char_down(c);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        let sp = KeyEvent {
            vk: keymap::vk::SPACE,
            key_down: true,
            ..Default::default()
        };
        let o = e.key(&sp);
        assert_eq!(o.action, Action::Pass);
        apply(&mut buf, &o.action, &sp);
        assert_eq!(buf.into_iter().collect::<String>(), "được ");
        assert!(o.preedit.is_empty(), "preedit phải clear sau ranh giới");
    }

    #[test]
    fn boundary_space_commits_with_preedit_cap() {
        let mut e = engine();
        e.set_context(Context {
            caps: strategy::IME_CAP_PREEDIT | strategy::IME_CAP_FIELD_DETECT,
            ..Default::default()
        });
        assert_eq!(e.strategy(), Strategy::Preedit);
        let mut buf: Vec<char> = Vec::new();
        for c in "duocj".chars() {
            let k = KeyEvent::char_down(c);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        let sp = KeyEvent {
            vk: keymap::vk::SPACE,
            key_down: true,
            ..Default::default()
        };
        let o = e.key(&sp);
        assert_eq!(o.action.kind(), ActionKind::Commit);
        apply(&mut buf, &o.action, &sp);
        assert_eq!(buf.into_iter().collect::<String>(), "được ");
    }

    #[test]
    fn keyup_does_not_change_state() {
        let mut e = engine();
        let k = KeyEvent {
            ch: 'd' as u32,
            key_down: false,
            ..Default::default()
        };
        assert_eq!(e.key(&k).action, Action::Pass);
        let (buf, _) = type_keys(&mut e, "d");
        assert_eq!(buf, "d");
    }
}
