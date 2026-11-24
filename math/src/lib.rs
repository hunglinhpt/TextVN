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
pub fn is_math_boundary(c: char) -> bool {
    matches!(
        c,
        ' ' | '\n'
            | '\t'
            | ','
            | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | ')'
            | ']'
            | '}'
            | '('
            | '{'
            | '['
            | '"'
            | '\''
            | '='
            | '+'
            | '-'
            | '*'
            | '/'
            | '^'
            | '_'
            | '<'
            | '>'
            | '|'
            | '$'
            | '&'
    )
}

fn is_word_boundary(c: char) -> bool {
    is_math_boundary(c)
}

/// Cap word buffer — quá dài thì bỏ word (an toàn, không unbounded).
const MAX_WORD_BUF: usize = 64;

/// Các lệnh LaTeX nhận đối số là định danh/tên môi trường/nhãn/ký hiệu (KHÔNG phải văn xuôi
/// tiếng Việt) — khi gặp `{`, giữ lệnh mở tới `}` để Telex không bẻ dấu tên môi trường hay nhãn
/// (vd. `\begin{subequations}`, `\end{align*}`, `\label{eq:1}`, `\cite{euler}`, `\mathbb{R}`).
fn is_non_text_latex_cmd(word: &str) -> bool {
    matches!(
        word,
        "\\begin"
            | "\\end"
            | "\\label"
            | "\\ref"
            | "\\eqref"
            | "\\pageref"
            | "\\autoref"
            | "\\cref"
            | "\\cite"
            | "\\citet"
            | "\\citep"
            | "\\usepackage"
            | "\\documentclass"
            | "\\input"
            | "\\include"
            | "\\bibliography"
            | "\\bibliographystyle"
            | "\\mathbb"
            | "\\mathcal"
            | "\\mathfrak"
            | "\\mathrm"
            | "\\mathbf"
            | "\\mathsf"
            | "\\mathtt"
            | "\\operatorname"
            | "\\color"
            | "\\textcolor"
            | "\\href"
            | "\\url"
    )
}

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

    /// Đưa 1 ký tự vào word buffer: whitespace/boundary → ngắt word; `\` luôn bắt đầu
    /// lệnh mới (hoặc hủy lệnh nếu là `\\` xuống dòng LaTeX); `=` mở inline eval chỉ khi
    /// đang ở đầu word và tự hủy ngay khi ký tự phá vỡ cú pháp biểu thức toán học.
    fn push_word_char(&mut self, c: char) {
        // Word đang là inline eval (`= ...` chưa khép) thì giữ nguyên qua boundary,
        // kể cả space — chỉ `=` thứ hai (đóng) mới tính kết quả. Nếu ký tự mới phá vỡ
        // biểu thức toán học (vd. `x = 5, nên...` hoặc `x = \pi/2`), xóa ngay word_buf
        // và tiếp tục xử lý chính `c` như ký tự khởi đầu mới (`\` bắt đầu lệnh `\pi`!).
        if self.word_buf.starts_with('=') {
            if c == '=' {
                self.word_buf.push(c);
                if self.word_buf.len() > MAX_WORD_BUF {
                    self.word_buf.clear();
                }
                return;
            }
            self.word_buf.push(c);
            if self.word_buf.len() <= MAX_WORD_BUF
                && crate::eval::is_valid_eval_prefix(&self.word_buf[1..])
            {
                return;
            }
            self.word_buf.clear();
            // Fall through để chính ký tự `c` (vd. `\` trong `x = \pi/2`) được nạp sạch!
        }
        // Đang nằm bên trong đối số định danh LaTeX (`\begin{subequations}`, `\label{eq:1}`...):
        // giữ word_buf bắt đầu bằng `\` cho tới khi gặp `}` để Telex luôn treo sạch.
        if self.word_buf.starts_with('\\') && self.word_buf.contains('{') {
            if c == '}' || c.is_whitespace() {
                self.word_buf.clear();
            } else {
                self.word_buf.push(c);
                if self.word_buf.len() > MAX_WORD_BUF {
                    self.word_buf.clear();
                }
            }
            return;
        }
        if c == '\\' {
            // `\\` là lệnh xuống dòng LaTeX — xóa buffer để chữ tiếng Việt ngay sau gõ bình thường.
            if self.word_buf == "\\" {
                self.word_buf.clear();
            } else {
                // `(\alpha`, `x+\alpha`, `$\alpha`: luôn khởi đầu lệnh sạch từ `\`.
                self.word_buf.clear();
                self.word_buf.push('\\');
            }
            return;
        }
        if c == '=' {
            // Chỉ mở inline-eval khi `=` đứng đầu từ (tránh `x=5` hay `a==b` mở eval sai).
            if self.word_buf.is_empty() {
                self.word_buf.push('=');
            } else {
                self.word_buf.clear();
            }
            return;
        }
        if c.is_whitespace() || is_word_boundary(c) {
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
            // Boundary (space, punctuation, toán tử...) KHÔNG push ở đây — core gọi
            // process_boundary ngay sau với cùng ký tự; push sẽ clear word trước khi
            // kịp bung alias. Ngoại lệ:
            // 1) `=` mở/đóng inline eval hoặc eval span đang MỞ (`word_buf.starts_with('=')`);
            // 2) ion hóa học chứa `+`/`-` (`\Fe2+`, `\OH-`, `\SO4-2`): chỉ giữ trong word_buf
            //    khi phần trước `+`/`-` CHƯA phải một lệnh toán đã khớp hoàn chỉnh (`\alpha+1`
            //    thì `alpha` đã khớp nên để `process_boundary` bung thành `α+1`!).
            None => {
                let keep_chem_sign = matches!(c, '+' | '-')
                    && self.word_buf.starts_with('\\')
                    && self.word_buf.len() > 1
                    && crate::lexicon::lookup_token(&self.word_buf[1..]).is_none();
                if keep_chem_sign {
                    self.word_buf.push(c);
                    if self.word_buf.len() > MAX_WORD_BUF {
                        self.word_buf.clear();
                    }
                } else if c == '=' || !is_word_boundary(c) || self.word_buf.starts_with('=') {
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
            // E57e + E63: eval thất bại — nếu là `= =` (khoảng trắng giữa dấu `=` của phép gán
            // và dấu `=` mở máy tính nhanh như `x = =2+3=`), giữ dấu `=` mới làm điểm mở eval;
            // ngược lại (`==`, `=max(1,2)=`) xóa sạch để không treo Telex.
            if !inner.is_empty() && inner.trim().is_empty() {
                self.word_buf.clear();
                self.word_buf.push('=');
            } else {
                self.word_buf.clear();
            }
        }
        action
    }

    /// Process a boundary character (space, enter, punctuation, toán tử...):
    /// mở rộng alias/template khi word đang là lệnh `\...` (`\can`, `\mat3x3`, `\alpha+1`...).
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
        // Inline eval chưa khép: giữ word lại chờ `=` đóng — chỉ khi `word` vẫn là tiền tố
        // biểu thức hợp lệ (`is_valid_eval_prefix`).
        if word.starts_with('=') && (c != '=' || word.len() == 1) {
            if word.len() == 1 || crate::eval::is_valid_eval_prefix(&word[1..]) {
                self.word_buf = word;
            }
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
            // Đang bên trong đối số định danh LaTeX (`\begin{subequations}`, `\label{eq:1}`):
            // mọi boundary trừ `}` và whitespace giữ nguyên trong buffer để không bật Telex giữa chừng.
            if word.contains('{') {
                if c != '}' && !c.is_whitespace() {
                    let mut next = word;
                    next.push(c);
                    if next.len() <= MAX_WORD_BUF {
                        self.word_buf = next;
                    }
                }
                return None;
            }
            // `{` và `[` là cú pháp mở đối số LaTeX (`\frac{1}{2}`, `\sqrt[3]{x}`,
            // `\section{Giới thiệu}`, `\textbf{chào}`) — KHÔNG bung snippet đè lên đối số.
            // Với lệnh nhận tên môi trường/nhãn (`\begin{...}`, `\end{...}`, `\label{...}`...),
            // giữ lệnh mở tới `}` để Telex treo sạch bên trong tên định danh!
            if matches!(c, '{' | '[') {
                if c == '{' && is_non_text_latex_cmd(&word) {
                    let mut next = word;
                    next.push('{');
                    self.word_buf = next;
                }
                return None;
            }
            // Ion hóa học (`\Fe2+`, `\OH-`, `\SO4-2`): nếu `word` chưa khớp và vừa giữ `+`/`-`
            // vào buffer ở `process_char`, giữ lại word chờ ký tự tiếp theo.
            if matches!(c, '+' | '-') && word.ends_with(c) {
                self.word_buf = word;
                return None;
            }
            let cfg = self.config.clone();
            if let Some(action) = chords::expand_lexicon_word(&word, mode, |t| {
                cfg.custom_output(t).map(str::to_string)
            }) {
                // Trong chế độ LaTeX, nếu lệnh đã đúng nguyên văn (vd. `\alpha` → `\alpha`)
                // thì không phát Replace thừa khi người nghiên cứu gõ tiếp `+`, `^`, `_`, `(`, `)`.
                if mode == MathOutputMode::Latex && action.inserted_text() == word {
                    return None;
                }
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

    /// Word đang gõ có phải inline-eval `=expr=` đang mở không.
    pub fn word_is_eval(&self) -> bool {
        self.word_buf.starts_with('=')
    }

    /// Xoá word đang theo dõi (khi gõ tắt/emoji/command không còn hợp lệ).
    pub fn reset_word(&mut self) {
        self.word_buf.clear();
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
