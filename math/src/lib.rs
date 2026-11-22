// SPDX-License-Identifier: GPL-3.0-or-later
//! TextVN Math Engine — Universal Scientific, Engineering & Math Engine

pub mod app_bridge;
pub mod ast;
pub mod chords;
pub mod clipboard;
pub mod config;
pub mod context;
pub mod converter;
pub mod eval;
pub mod lexicon;
pub mod templates;

pub use app_bridge::{resolve_output_mode, AppFormatKind, AppProfile};
pub use chords::{AppGuard, ChordFsm, ChordState};
pub use config::{Discipline, HudHint, MathAction, MathConfig, MathOutputMode};
pub use context::{MathContextFsm, SpanKind};

/// Public facade orchestrator
#[derive(Clone)]
pub struct MathEngine {
    pub config: MathConfig,
    pub chord_fsm: ChordFsm,
    pub context_fsm: MathContextFsm,
    pub guard: AppGuard,
    pub profile: AppProfile,
    /// Word đang gõ (bounded 64 ký tự) để mở rộng alias/template/inline-eval ở
    /// boundary — nguồn sự thật duy nhất cho `word_is_command` và `process_boundary`.
    word_buf: String,
}

/// Ký tự kết thúc word (boundary) — khớp `Engine::is_math_boundary` ở core.
fn is_word_boundary(c: char) -> bool {
    matches!(
        c,
        ' ' | '\n' | '\t' | ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '"' | '\'' | '='
    )
}

/// Cap word buffer — quá dài thì bỏ word (an toàn, không unbounded).
const MAX_WORD_BUF: usize = 64;

impl MathEngine {
    pub fn new(config: MathConfig) -> Self {
        Self {
            chord_fsm: ChordFsm::new(&config),
            context_fsm: MathContextFsm::new(),
            guard: AppGuard::default(),
            profile: AppProfile::default(),
            config,
            word_buf: String::new(),
        }
    }

    /// Đưa 1 ký tự vào word buffer: whitespace/boundary → ngắt word; `=` giữ lại
    /// (phục vụ inline eval `=expr=`).
    fn push_word_char(&mut self, c: char) {
        // Word đang là inline eval (`= ...` chưa khép) thì giữ nguyên qua boundary,
        // kể cả space — chỉ `=` thứ hai (đóng) mới tính kết quả.
        if self.word_buf.starts_with('=') {
            self.word_buf.push(c);
            if self.word_buf.len() > MAX_WORD_BUF {
                self.word_buf.clear();
            }
            return;
        }
        if c.is_whitespace() || (is_word_boundary(c) && c != '=') {
            self.word_buf.clear();
            return;
        }
        self.word_buf.push(c);
        if self.word_buf.len() > MAX_WORD_BUF {
            self.word_buf.clear();
        }
    }

    /// Process a character key through the math engine
    pub fn process_char(&mut self, c: char, now_ms: u64, guard: &AppGuard) -> Option<MathAction> {
        let action = self.chord_fsm.process(c, now_ms, guard);
        match &action {
            // Chord nuốt/emit → word hiện tại kết thúc.
            Some(_) => self.word_buf.clear(),
            // Boundary (space, punctuation...) KHÔNG push ở đây — core gọi
            // process_boundary ngay sau với cùng ký tự; push sẽ clear word trước khi
            // kịp bung alias. Chỉ `=` (mở/đóng inline eval) và ký tự thường mới vào word.
            // E57d: khi eval span đang MỞ (word bắt đầu `=`), boundary chars
            // (`)`, `,`, `.`…) vẫn phải vào word — `=sin(0)=` trước đây mất `)`
            // → eval("sin(0") = None → không bao giờ bung.
            None => {
                if c == '=' || !is_word_boundary(c) || self.word_buf.starts_with('=') {
                    self.push_word_char(c);
                }
            }
        }
        // Inline eval `=expr=`: khi word khép kín bởi `=` thứ hai → tính ngay tại
        // process_char (core gọi process_char trước process_boundary với mọi phím).
        if action.is_none()
            && self.word_buf.len() >= 2
            && c == '='
            && self.word_buf.starts_with('=')
            && self.word_buf.ends_with('=')
        {
            // Hỗ trợ cả `=2+3=` (không space) và `= 2+3 =` (delimited có space).
            let inner = &self.word_buf[1..self.word_buf.len() - 1];
            let v =
                crate::eval::eval(inner).or_else(|| crate::eval::eval_delimited(&self.word_buf));
            if let Some(v) = v {
                // E57b: '=' dong la keystroke HIEN TAI (bi consume, chua vao
                // document) -- delete chi tinh cac ky tu da commit (=2+3 = 4).
                // Truoc day del = 5 khien adapter that xoa lan 1 ky tu truoc word.
                let del = self.word_buf.chars().count().saturating_sub(1);
                let text = crate::eval::format_value(v);
                self.word_buf.clear();
                return Some(MathAction::ReplaceBuffer {
                    delete_chars: del,
                    text,
                    cursor_left_steps: 0,
                });
            }
            // E57e: eval thất bại — CHỐT span thay vì để word kẹt (=max(1,2)=
            // không phải hàm hợp lệ). Trước đây word "=..." không bao giờ được
            // xoá → `word_is_command` treo Telex vĩnh viễn trong sticky mode.
            self.word_buf.clear();
        }
        action
    }

    /// Process a boundary character (space, enter, punctuation...):
    /// mở rộng alias/template khi word đang là lệnh `\...` (`\can`, `\mat3x3`...).
    pub fn process_boundary(
        &mut self,
        c: char,
        mode: MathOutputMode,
        guard: &AppGuard,
    ) -> Option<MathAction> {
        let word = std::mem::take(&mut self.word_buf);
        if guard.blocks_chords() {
            return None;
        }
        // Inline eval chưa khép: giữ word lại chờ `=` đóng — kể cả `=` MỞ đầu
        // (word == "=") và mọi boundary khác (space...).
        if word.starts_with('=') && (c != '=' || word.len() == 1) {
            self.word_buf = word;
            return None;
        }
        // Fallback eval nếu `=` kết thúc đến qua đường boundary.
        if c == '=' && word.starts_with('=') {
            let candidate = format!("{word}=");
            let inner = &word[1..];
            let v = crate::eval::eval(inner).or_else(|| crate::eval::eval_delimited(&candidate));
            if let Some(v) = v {
                // E57b: '=' cua keystroke hien tai bi consume -- khong dem.
                let del = word.chars().count();
                let text = crate::eval::format_value(v);
                return Some(MathAction::ReplaceBuffer {
                    delete_chars: del,
                    text,
                    cursor_left_steps: 0,
                });
            }
        }
        if word.starts_with('\\') && word.len() > 1 {
            let cfg = self.config.clone();
            if let Some(action) = chords::expand_lexicon_word(&word, mode, |t| {
                cfg.custom_output(t).map(str::to_string)
            }) {
                return Some(action);
            }
        }
        None
    }

    /// Feed context character (for delimiter tracking + word tracking khi math chưa active)
    pub fn feed_context(&mut self, c: char) {
        self.context_fsm.feed_text_char(c);
        self.push_word_char(c);
    }

    /// Word đang gõ có phải lệnh math (`\...`) không — core dùng để treo Telex
    /// khi sticky mode (spec: sticky + word là command → pass-through tới boundary).
    pub fn word_is_command(&self) -> bool {
        // E57c: từ lệnh là `\...` HOẶC inline-eval `=expr=` đang gõ —
        // cả hai đều cần Telex bị treo (sticky mode) để chữ không bị fold
        // (`=sin(0)=` sẽ thành `=sín(0)=` nếu để Telex xử lý `s` sau `i`).
        self.word_buf.starts_with('\\') || self.word_buf.starts_with('=')
    }

    /// Set new config — dựng lại chord FSM với leader/timeout mới.
    pub fn set_config(&mut self, config: MathConfig) {
        self.chord_fsm = ChordFsm::new(&config);
        self.config = config;
    }

    /// E57a: Backspace -- xoa 1 ky tu cuoi khoi word dang theo doi. Khong co lenh
    /// nay, `\\can` + Backspace + go tiep de lai word_buf lech document:
    /// boundary ke tiep bung sai lenh (hoac te hon: ReplaceBuffer delete lan
    /// vao text phia truoc, mat ky tu cua nguoi dung).
    pub fn process_backspace(&mut self) {
        self.word_buf.pop();
        // Leader dang armed ma Backspace -> huy chord (zero stuck-state).
        self.chord_fsm.reset();
    }

    /// E57a: caret nhay cho khac (mui ten/Home/End/Delete/chuot) -- vi tri trong
    /// word khong con tuong ung -> xoa sach word dang theo doi thay vi doan mo.
    pub fn caret_moved(&mut self) {
        self.word_buf.clear();
    }

    /// Reset all state
    pub fn reset_all(&mut self) {
        self.chord_fsm.reset();
        self.context_fsm = MathContextFsm::new();
        self.word_buf.clear();
    }
}
