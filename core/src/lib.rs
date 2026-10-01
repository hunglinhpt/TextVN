// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-core` — engine gõ tiếng Việt, thuần logic (P0-1 §2).
//!
//! Bất biến (Handbook S1–S4, P0-2 §0):
//! - **Không** network / file I/O / spawn thread / log nội dung text.
//! - `key()` luôn trả `Outcome` — tầng FFI lo catch_unwind → fail-open.
//! - Không global mutable state: mọi trạng thái nằm trong `Engine`.

pub mod buffer;
pub mod keymap;
pub mod keymap_mac_generated;
pub mod method;
pub mod post;
pub mod transform;
pub mod validate;

pub use keymap::KeyEvent;
pub use method::Method;
pub use post::emoji::Emoji;
pub use post::r#macro::{MacroDef, MacroTrigger, MacroWhen};
pub use strategy::{ActionKind, Strategy};
pub use transform::{DiacriticStyle, OutputCharset};

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

/// Options từ `config.v1` (textvn-config đã validate — ffi dịch sang đây).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOptions {
    pub method: Method,
    pub diacritic_style: DiacriticStyle,
    pub free_marking: bool,
    /// Master switch `config.enabled`.
    pub enabled: bool,
    /// `config.auto_restore_english` (bug B5) — xem `post/restore_en.rs`.
    pub auto_restore_english: bool,
    /// `config.auto_capitalize` (P0-3 §1.1) — sau `. ! ?` + Enter.
    pub auto_capitalize: bool,
    /// `config.macro_trigger` — phím mở rộng macro/emoji.
    pub macro_trigger: MacroTrigger,
    /// `config.allow_macro_when_vi_off` (EVKey spec #5).
    pub allow_macro_when_vi_off: bool,
    /// `config.macros[]`.
    pub macros: Vec<MacroDef>,
    /// `config.emoji[]`.
    pub emoji: Vec<Emoji>,
    /// `config.english_words[]` — danh sách từ **tiếng Anh** mà chủ gõ muốn giữ nguyên
    /// khi Telex biến nó thành chuỗi trông như tiếng Việt (`text` → `tết`).
    ///
    /// **Mặc định RỔNG** (không tự bật) vì đây là ca *mơ hồ*: `test` → `tết` đúng là từ
    /// Việt thật, bật sẵn sẽ phá người đang gõ "Tết". Đây là lựa chọn của người dùng
    /// (mẫu danh sách: `data/stop_en.txt`) — xem `post/restore_en.rs`.
    pub english_words: Vec<String>,
    /// Quick Telex (OpenKey): `cc→ch gg→gi kk→kh nn→ng qq→qu pp→ph tt→th` ở phụ âm đầu.
    /// Chỉ áp cho Telex/Simple Telex (VNI/VIQR không gõ phụ âm đôi). Mặc định tắt.
    pub quick_telex: bool,
    /// `config.output_charset` — bảng mã chữ đi ra document (`transform/charset.rs`).
    pub output_charset: OutputCharset,
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            method: Method::Telex,
            diacritic_style: DiacriticStyle::New,
            free_marking: true,
            enabled: true,
            auto_restore_english: true,
            auto_capitalize: true,
            macro_trigger: MacroTrigger::Tab,
            allow_macro_when_vi_off: false,
            macros: Vec::new(),
            emoji: Vec::new(),
            english_words: Vec::new(),
            quick_telex: false,
            output_charset: OutputCharset::UnicodePrecomposed,
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

/// Số ký tự tối đa engine nhớ để khớp trigger macro/emoji (đủ cho mọi trigger hợp lý).
const RECENT_MAX: usize = 64;

/// Giới hạn `insert`/`preedit` của `ime_result_v1` — P0-2 §2: **engine tự giữ** ≤ 64
/// (FFI còn một lưới an toàn cuối; macro dài hơn 64 ký tự bị cắt tại đây).
pub const MAX_TEXT: usize = 64;

/// Engine — 1 instance = 1 thread (P0-2 §3).
pub struct Engine {
    opts: EngineOptions,
    ctx: Context,
    word: buffer::Word,
    /// Đuôi text engine biết chắc đã nằm trong document (trước con trỏ) — cho macro/emoji.
    recent: Vec<char>,
    /// Cờ "chữ cái kế tiếp viết hoa" (sau `. ! ?` / Enter) — `post/caps.rs`.
    caps_pending: bool,
}

impl Engine {
    pub fn new(opts: EngineOptions) -> Engine {
        Engine {
            opts,
            ctx: Context::default(),
            word: buffer::Word::default(),
            recent: Vec::new(),
            caps_pending: false,
        }
    }

    /// `ime_reload_config` — đổi options giữa chừng → reset từ đang gõ dở.
    pub fn set_options(&mut self, opts: EngineOptions) {
        self.opts = opts;
        self.word.clear();
        self.recent.clear();
        self.caps_pending = false;
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
        self.recent.clear();
        self.caps_pending = false;
    }

    /// Cập nhật đuôi text: xoá `delete` ký tự cuối rồi thêm `insert` (giữ tối đa `RECENT_MAX`).
    fn recent_replace(&mut self, delete: usize, insert: &[char]) {
        let keep = self.recent.len().saturating_sub(delete);
        self.recent.truncate(keep);
        self.recent.extend_from_slice(insert);
        if self.recent.len() > RECENT_MAX {
            let drop = self.recent.len() - RECENT_MAX;
            self.recent.drain(..drop);
        }
    }

    /// Ghi nhận một phím **PASS** (adapter tự chèn) vào đuôi text cho macro/emoji.
    ///
    /// Ô mật khẩu (`secure`) → **không** giữ tail nào (S3).
    fn note_pass(&mut self, k: &KeyEvent) {
        if self.ctx.secure {
            return;
        }
        match k.vk {
            keymap::vk::BACK => {
                self.recent.pop();
            }
            keymap::vk::DELETE
            | keymap::vk::LEFT
            | keymap::vk::RIGHT
            | keymap::vk::UP
            | keymap::vk::DOWN => self.recent.clear(),
            _ => match k.printable() {
                Some(c) if !c.is_control() || matches!(c, ' ' | '\n' | '\t') => {
                    self.recent_replace(0, &[c]);
                }
                Some(_) => {}
                // Home/End/PageUp/F-key…: con trỏ có thể đã nhảy — đuôi text engine nhớ
                // không còn nằm ngay trước con trỏ, gõ tắt sau đó sẽ xoá nhầm chỗ.
                None => self.recent.clear(),
            },
        }
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
            self.emit(&self.word.display)
        } else {
            Vec::new()
        }
    }

    /// Chữ đi ra document theo bảng mã xuất. `word.owned`/`delete_count`/`recent` luôn
    /// đếm trên chuỗi này (một chữ Việt có thể là 2 ký tự ở VNI Windows/Unicode tổ hợp).
    fn emit(&self, text: &[char]) -> Vec<char> {
        transform::charset::encode(text, self.opts.output_charset)
    }

    fn fold_current(&self) -> Vec<char> {
        let mut display = method::fold_caps(
            &self.word.raw,
            self.opts.method,
            self.opts.diacritic_style,
            self.opts.free_marking,
            self.word.caps_lock,
        );
        if self.opts.quick_telex && matches!(self.opts.method, Method::Telex | Method::SimpleTelex)
        {
            post::quick_telex::apply(&mut display);
        }
        display
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
        if k.is_modifier() {
            return Outcome::pass();
        }
        if k.is_chord() {
            // Ctrl+V/Ctrl+Z/Alt+Tab… đổi text quanh con trỏ mà engine không thấy: quên đuôi
            // text để gõ tắt kế tiếp không xoá nhầm (`vn` Ctrl+V `abc` Tab ≠ bung `vn`).
            self.recent.clear();
            return Outcome::pass();
        }

        let strategy = self.strategy();
        let vi_on = self.opts.enabled && self.ctx.enabled;

        // Gõ tắt / emoji (P0-3 §1.1) — chạy cả khi VN tắt nếu `allow_macro_when_vi_off`
        // (EVKey spec #5); `secure` tuyệt đối không (S3).
        if let Some(out) = self.try_macro(k, strategy, vi_on) {
            return out;
        }

        if strategy == Strategy::Passthrough {
            if !self.word.is_empty() {
                self.word.clear();
            }
            // Vẫn ghi nhận text đi thẳng: macro `always` có thể chạy khi VN tắt
            // (`allow_macro_when_vi_off`) — EVKey spec #5.
            self.note_pass(k);
            // Ranh giới từ (Space, dấu câu, Enter, phím điều hướng…) báo WORD_END: adapter
            // giữ từ trong composition để bung gõ tắt khi VN tắt sẽ commit từ ở đây thay vì
            // kéo gạch chân sang từ kế tiếp.
            let boundary = k
                .printable()
                .is_none_or(|c| !method::is_word_char(c, self.opts.method));
            return Outcome {
                flags: if boundary { FLAG_WORD_END } else { 0 },
                ..Outcome::pass()
            };
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
            // Delete/arrow/F-key…: app có thể đã đổi selection/cursor mà engine
            // không quan sát được. Bỏ composition ownership trước khi PASS; nếu
            // giữ `word`, key kế tiếp sẽ REPLACE một suffix ở vị trí cũ (B7).
            self.word.clear();
            self.note_pass(k);
            return Outcome::pass();
        };

        if !method::is_word_char(c, self.opts.method) {
            // Số/punctuation (theo method): đóng từ (nếu có) rồi cho ký tự đi thẳng.
            return self.on_boundary(c, strategy);
        }

        // --- Chữ cái: fold path ---
        // Auto-capitalize (P0-3 §1.1): chỉ áp cho **chữ đầu từ** ngay sau `. ! ?` / Enter.
        let mut caps_fired = false;
        let c = if self.opts.auto_capitalize && self.caps_pending && self.word.is_empty() {
            caps_fired = true;
            self.caps_pending = false;
            post::caps::capitalize(c)
        } else {
            c
        };

        if k.mods & keymap::MOD_CAPS != 0 {
            self.word.caps_lock = true;
        }
        self.word.raw.push(c);
        let display = self.fold_current();

        if !self.word.active && display == self.word.raw {
            // Chưa có transform nào → chữ đi thẳng (tiếng Anh không đổi — WIN-011).
            self.word.passed.push(c);
            self.word.display = display.clone();
            if caps_fired {
                // Chữ đã bị engine ĐỔI (hoa) → không thể PASS (PASS = adapter chèn phím gốc).
                // `delete_count = 0`: chèn `insert` tại con trỏ; `passed` giữ đúng ký tự này
                // để lần activate sau xoá đúng số ký tự engine đã đưa vào document.
                let out = self.emit(&display);
                self.recent_replace(0, &out);
                return Outcome {
                    action: Action::Replace {
                        delete_count: 0,
                        insert: out,
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED,
                };
            }
            self.recent_replace(0, &[c]);
            return Outcome::pass();
        }

        let delete = if self.word.active {
            self.word.owned as u16
        } else {
            self.word.pending_delete() // activate: xóa những ký tự đã PASS trong từ
        };
        self.word.active = true;
        self.word.passed.clear();
        let out = self.emit(&display);
        self.word.display = display;
        self.word.owned = out.len();
        self.recent_replace(delete as usize, &out);
        Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: out.clone(),
            },
            preedit: out,
            flags: FLAG_CONSUMED,
        }
    }

    /// Trigger key của macro/emoji → mở rộng text (P0-3 §1.1, `post/macro.rs`).
    ///
    /// Trigger key bị **nuốt**; `Shift`+trigger = combo hệ thống → không đụng (S9, bug B6).
    fn try_macro(&mut self, k: &KeyEvent, strategy: Strategy, vi_on: bool) -> Option<Outcome> {
        let is_trigger = matches!(
            (k.vk, self.opts.macro_trigger),
            (keymap::vk::TAB, MacroTrigger::Tab) | (keymap::vk::SPACE, MacroTrigger::Space)
        );
        if !is_trigger || k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }
        if strategy == Strategy::Passthrough
            && !(self.opts.allow_macro_when_vi_off && !self.ctx.secure)
        {
            return None;
        }
        let match_display =
            post::r#macro::find(&self.opts.macros, &self.opts.emoji, &self.recent, vi_on);
        // Trigger ASCII co the da bi Telex/VNI fold truoc khi bam Tab:
        // "too" -> "tô" trong document. Thu khop raw cua dung tu dang
        // active; xoa so glyph DA PHAT RA (owned), khong phai do dai raw.
        let match_raw = if self.word.active {
            post::r#macro::find(&self.opts.macros, &[], &self.word.raw, vi_on)
                .filter(|(raw_len, _)| *raw_len == self.word.raw.len())
                .map(|(_, text)| (self.word.owned, text))
        } else {
            None
        };
        let (len, text) = match_display.or(match_raw)?;
        let expanded: Vec<char> = text.chars().collect();
        let insert: Vec<char> = self.emit(&expanded).into_iter().take(MAX_TEXT).collect();
        self.word.clear();
        self.recent_replace(len, &insert);
        Some(Outcome {
            action: Action::Replace {
                delete_count: len as u16,
                insert: insert.clone(),
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// Ranh giới từ: Space/Enter/Tab/punct (P0-2 §4, bug B2 cho Enter+Preedit).
    ///
    /// Thứ tự (stage 7 pipeline): caps (`. ! ?` + Enter) → auto-restore EN (B5) → đóng từ.
    fn on_boundary(&mut self, c: char, strategy: Strategy) -> Outcome {
        // `.` `!` `?` `\n` → chữ cái kế tiếp viết hoa (P0-3 §1.1)
        if post::caps::is_sentence_end(c) {
            self.caps_pending = true;
        }
        let was_active = self.word.active;

        // Auto-restore EN (bug B5): kết quả fold không phải âm tiết Việt → trả lại chuỗi gõ.
        if was_active && self.opts.auto_restore_english {
            let raw = self.word.raw.clone();
            let display = self.word.display.clone();
            if post::restore_en::should_restore(&raw, &display, &self.opts.english_words) {
                let delete = self.word.owned as u16;
                let mut insert = raw;
                insert.push(c); // ranh giới vẫn phải vào document (RESTORE = REPLACE — P0-4 §3)
                self.word.clear();
                self.recent_replace(delete as usize, &insert);
                return Outcome {
                    action: Action::Restore {
                        delete_count: delete,
                        insert,
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED | FLAG_WORD_END,
                };
            }
        }

        self.word.clear();
        self.recent_replace(0, &[c]);
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
        self.recent_replace(delete as usize, &raw);
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
                self.recent_replace(delete as usize, &[]);
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
            let out = self.emit(&display);
            self.word.display = display;
            self.word.owned = out.len();
            self.recent_replace(delete as usize, &out);
            return Outcome {
                action: Action::Replace {
                    delete_count: delete,
                    insert: out.clone(),
                },
                preedit: out,
                flags: FLAG_CONSUMED,
            };
        }
        // Chưa activate: bỏ 1 ký tự đã gõ thẳng
        // Nếu người dùng vừa xóa dấu kết câu thì không được giữ cờ viết hoa
        // cho ký tự kế tiếp (ví dụ `hi.<Backspace>ban` phải là `hiban`).
        if self.word.is_empty()
            && self
                .recent
                .last()
                .is_some_and(|&c| post::caps::is_sentence_end(c))
        {
            self.caps_pending = false;
        }
        self.word.raw.pop();
        self.word.passed.pop();
        self.word.display.clear();
        self.recent.pop();
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
        let (buf, preedit) = type_keys(&mut e, "dduocj");
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
        let (buf, _) = type_keys(&mut e, "dduocj");
        assert_eq!(buf, "được");
        // Backspace ×5 → raw "dduoc"→"đươc", "dduo"→"đuơ", "ddu"→"đu", "dd"→"đ", "d"→"d"
        let mut buf: Vec<char> = buf.chars().collect();
        for _ in 0..5 {
            let k = KeyEvent::key_down(keymap::vk::BACK);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        assert_eq!(buf.into_iter().collect::<String>(), "d");
    }

    #[test]
    fn escape_restores_raw() {
        let mut e = engine();
        let (buf, _) = type_keys(&mut e, "dduocj");
        assert_eq!(buf, "được");
        let mut buf: Vec<char> = buf.chars().collect();
        let k = KeyEvent::key_down(keymap::vk::ESCAPE);
        let o = e.key(&k);
        assert_eq!(o.action.kind(), ActionKind::Restore);
        apply(&mut buf, &o.action, &k);
        assert_eq!(buf.into_iter().collect::<String>(), "dduocj");
    }

    #[test]
    fn navigation_cancels_preedit_before_next_typing() {
        let mut e = engine();
        let (buf, _) = type_keys(&mut e, "tooi");
        assert_eq!(buf, "tôi");

        assert_eq!(
            e.key(&KeyEvent::key_down(keymap::vk::LEFT)).action,
            Action::Pass
        );
        // Không còn owner range sau khi cursor đã di chuyển; x phải PASS, không
        // được REPLACE `tôi` bằng delete_count theo vị trí con trỏ cũ.
        assert_eq!(e.key(&KeyEvent::char_down('x')).action, Action::Pass);
    }

    #[test]
    fn boundary_space_pass_with_default_caps() {
        let mut e = engine(); // caps=0 → BackspaceType → ranh giới = PASS
        let mut buf: Vec<char> = Vec::new();
        for c in "dduocj".chars() {
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
        for c in "dduocj".chars() {
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

    /// Gõ chuỗi ký tự và trả **buffer** (để test tiếp ranh giới/trigger key).
    fn type_buf(e: &mut Engine, input: &str) -> Vec<char> {
        let mut buf: Vec<char> = Vec::new();
        for c in input.chars() {
            let k = KeyEvent::char_down(c);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        buf
    }

    /// Bấm 1 phím đặc biệt, trả action (đã áp vào `buf`).
    fn press(e: &mut Engine, buf: &mut Vec<char>, vk: u32) -> Action {
        let k = KeyEvent::key_down(vk);
        let o = e.key(&k);
        apply(buf, &o.action, &k);
        o.action
    }

    fn text(buf: &[char]) -> String {
        buf.iter().collect()
    }

    // ---- stage 7: auto-restore EN (bug B5) ----

    #[test]
    fn restore_en_at_boundary_on_invalid_word() {
        let mut e = engine();
        let mut buf = type_buf(&mut e, "asdf");
        assert_eq!(text(&buf), "àd"); // fold tạm: không phải âm tiết Việt
        let action = press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(action.kind(), ActionKind::Restore);
        assert_eq!(text(&buf), "asdf "); // trả lại chuỗi gõ + ranh giới
    }

    #[test]
    fn restore_en_keeps_valid_vietnamese() {
        let mut e = engine();
        let mut buf = type_buf(&mut e, "dduocj");
        assert_eq!(text(&buf), "được");
        let action = press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(action, Action::Pass, "từ hợp lệ → không restore");
        assert_eq!(text(&buf), "được ");
    }

    #[test]
    fn restore_en_can_be_disabled() {
        let mut e = Engine::new(EngineOptions {
            auto_restore_english: false,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "asdf");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "àd "); // tắt option → giữ kết quả fold
    }

    /// Nhánh từ điển B5: danh sách từ EN do chủ gõ khai báo → `text` trả lại `text`
    /// dù kết quả fold (`tết`) **hợp lệ về cấu trúc**.
    #[test]
    fn restore_en_danh_sach_tieng_anh() {
        let mut e = Engine::new(EngineOptions {
            english_words: vec!["text".to_string()],
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "text");
        assert_eq!(text(&buf), "tẽt"); // trước ranh giới: vẫn là kết quả fold
        let action = press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "text "); // Space → trả lại chuỗi gõ
        assert!(matches!(action, Action::Restore { .. }), "phải RESTORE");
    }

    #[test]
    fn macro_trigger_matches_raw_even_after_telex_fold() {
        let mut e = Engine::new(EngineOptions {
            macros: vec![post::r#macro::MacroDef {
                trigger: "too".into(),
                expand: "TOOO".into(),
                when: post::r#macro::MacroWhen::Always,
            }],
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "too");
        assert_eq!(text(&buf), "tô");
        let action = press(&mut e, &mut buf, keymap::vk::TAB);
        assert!(matches!(
            action,
            Action::Replace {
                delete_count: 2,
                ..
            }
        ));
        assert_eq!(text(&buf), "TOOO");
    }

    /// Bật danh sách **không** được phá tiếng Việt: từ không có trong danh sách thì giữ fold.
    #[test]
    fn danh_sach_khong_dung_cho_tu_khac() {
        let mut e = Engine::new(EngineOptions {
            english_words: vec!["text".to_string()],
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "test");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "tét "); // `test` không có trong danh sách → giữ `tét`

        // và từ Việt thật vẫn đúng
        let mut e = engine();
        let mut buf = type_buf(&mut e, "dduocj");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "được ");
    }

    // ---- stage 7: auto-capitalize (P0-3 §1.1) ----

    #[test]
    fn auto_capitalize_after_dot_and_enter() {
        let mut e = engine();
        let mut buf = type_buf(&mut e, "chao.");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        buf.extend(type_buf(&mut e, "ban"));
        assert_eq!(text(&buf), "chao. Ban"); // sau `.` → viết hoa

        let mut e = engine();
        let mut buf = type_buf(&mut e, "chao");
        press(&mut e, &mut buf, keymap::vk::RETURN);
        buf.extend(type_buf(&mut e, "ban"));
        assert_eq!(text(&buf), "chao\nBan"); // sau Enter → viết hoa
    }

    #[test]
    fn auto_capitalize_only_first_letter_of_word() {
        let mut e = engine();
        let mut buf = type_buf(&mut e, "hi.");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        buf.extend(type_buf(&mut e, "bAn"));
        assert_eq!(text(&buf), "hi. BAn"); // chỉ chữ đầu bị đổi
    }

    #[test]
    fn backspace_after_sentence_end_cancels_auto_capitalize() {
        let mut e = engine();
        let mut buf = type_buf(&mut e, "hi.");
        press(&mut e, &mut buf, keymap::vk::BACK);
        // `press` mô phỏng engine action; BACK là PASS nên app mới là bên xóa glyph.
        buf.pop();
        buf.extend(type_buf(&mut e, "ban"));
        assert_eq!(text(&buf), "hiban");
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

    // ---- stage 7: gõ tắt / emoji ----

    fn macro_opts(trigger: MacroTrigger) -> EngineOptions {
        EngineOptions {
            macro_trigger: trigger,
            macros: vec![MacroDef {
                trigger: "cty".into(),
                expand: "Công ty TNHH".into(),
                when: MacroWhen::Always,
            }],
            emoji: vec![Emoji {
                trigger: ":smile".into(),
                glyph: "😊".into(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn macro_expands_on_tab_and_eats_trigger() {
        let mut e = Engine::new(macro_opts(MacroTrigger::Tab));
        let mut buf = type_buf(&mut e, "cty");
        let action = press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(
            action,
            Action::Replace {
                delete_count: 3,
                insert: "Công ty TNHH".chars().collect()
            }
        );
        assert_eq!(text(&buf), "Công ty TNHH");
    }

    #[test]
    fn emoji_expands_on_space_trigger() {
        let mut e = Engine::new(macro_opts(MacroTrigger::Space));
        let mut buf = type_buf(&mut e, ":smile");
        let action = press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(
            action,
            Action::Replace {
                delete_count: 6,
                insert: "😊".chars().collect()
            }
        );
        assert_eq!(text(&buf), "😊");
    }

    #[test]
    fn macro_trigger_without_match_passes_through() {
        let mut e = Engine::new(macro_opts(MacroTrigger::Tab));
        let mut buf = type_buf(&mut e, "abc");
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);
        assert_eq!(text(&buf), "abc\t");
    }

    #[test]
    fn macro_not_expanded_after_caret_jump_or_chord() {
        // `vn` rồi Home (0x24) → con trỏ đã ở đầu dòng: Tab không được xoá 2 ký tự
        // của dòng trước. Tương tự sau Ctrl+V (text dán vào engine không thấy).
        const HOME: u32 = 0x24;
        let mut opts = macro_opts(MacroTrigger::Tab);
        opts.macros.push(MacroDef {
            trigger: "vn".into(),
            expand: "Việt Nam".into(),
            when: MacroWhen::Always,
        });
        let mut e = Engine::new(opts.clone());
        let mut buf = type_buf(&mut e, "vn");
        let _ = e.key(&KeyEvent::key_down(HOME));
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);

        let mut e = Engine::new(opts.clone());
        let _ = type_buf(&mut e, "vn");
        let paste = KeyEvent {
            vk: 0x56,
            ch: 'v' as u32,
            mods: keymap::MOD_CTRL,
            key_down: true,
            ..Default::default()
        };
        let _ = e.key(&paste);
        let mut buf = Vec::new();
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);

        // Không có bước nhảy → vẫn bung bình thường.
        let mut e = Engine::new(opts);
        let mut buf = type_buf(&mut e, "vn");
        assert_eq!(
            press(&mut e, &mut buf, keymap::vk::TAB).kind(),
            ActionKind::Replace
        );
        assert_eq!(text(&buf), "Việt Nam");
    }

    #[test]
    fn passthrough_marks_word_boundaries() {
        let mut e = Engine::new(EngineOptions {
            enabled: false,
            ..Default::default()
        });
        let letter = e.key(&KeyEvent::char_down('a'));
        assert_eq!(letter.action, Action::Pass);
        assert_eq!(letter.flags & FLAG_WORD_END, 0);
        for k in [
            KeyEvent::char_down(' '),
            KeyEvent::char_down('.'),
            KeyEvent::key_down(keymap::vk::LEFT),
        ] {
            let o = e.key(&k);
            assert_eq!(o.action, Action::Pass);
            assert_ne!(o.flags & FLAG_WORD_END, 0, "{k:?}");
        }
    }

    #[test]
    fn macro_when_vi_off_follows_allow_flag() {
        // VN tắt + không cho phép → Tab đi thẳng
        let mut opts = macro_opts(MacroTrigger::Tab);
        opts.enabled = false;
        let mut e = Engine::new(opts.clone());
        let mut buf = type_buf(&mut e, "cty");
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);
        assert_eq!(text(&buf), "cty\t");

        // VN tắt + allow_macro_when_vi_off → macro vẫn chạy (EVKey spec #5)
        opts.allow_macro_when_vi_off = true;
        let mut e = Engine::new(opts);
        let mut buf = type_buf(&mut e, "cty");
        assert_eq!(
            press(&mut e, &mut buf, keymap::vk::TAB).kind(),
            ActionKind::Replace
        );
        assert_eq!(text(&buf), "Công ty TNHH");
    }

    #[test]
    fn macro_never_runs_in_secure_field() {
        let mut opts = macro_opts(MacroTrigger::Tab);
        opts.allow_macro_when_vi_off = true;
        let mut e = Engine::new(opts);
        e.set_context(Context {
            secure: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "cty");
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);
        assert_eq!(text(&buf), "cty\t");
    }

    // ---- method khác đi qua engine (digit/marker là ký tự của từ) ----

    #[test]
    fn vni_digits_are_word_chars() {
        let mut e = Engine::new(EngineOptions {
            method: Method::Vni,
            ..Default::default()
        });
        let buf = type_buf(&mut e, "d9uo7ng2");
        assert_eq!(text(&buf), "đường");
    }

    #[test]
    fn viqr_markers_are_word_chars() {
        let mut e = Engine::new(EngineOptions {
            method: Method::Viqr,
            ..Default::default()
        });
        let buf = type_buf(&mut e, "ddu+o+ng`");
        assert_eq!(text(&buf), "đường");
    }

    #[test]
    fn simple_telex_w_is_literal_through_engine() {
        let mut e = Engine::new(EngineOptions {
            method: Method::SimpleTelex,
            ..Default::default()
        });
        let buf = type_buf(&mut e, "tuw");
        assert_eq!(text(&buf), "tuw");
    }
}
