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

use std::sync::Arc;

pub use keymap::KeyEvent;
pub use method::Method;
pub use post::emoji::Emoji;
pub use post::r#macro::{MacroDef, MacroTrigger, MacroWhen};
pub use strategy::{ActionKind, Strategy};
pub use transform::{
    detect, repair_mojibake, transcode, Charset, DiacriticStyle, OutputCharset, TextCase,
};

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
    /// `config.suggest.enabled` (P0-3 §1.1) — Tab hoàn tất từ tiếng Việt
    /// (`post/suggest_vn.rs`). Mặc định tắt: Tab giữ hành vi cũ (đi qua app).
    pub suggest_vn_complete: bool,
    /// `config.suggest.trigger` (vòng 24) — Space cũng hoàn tất từ khi từ
    /// đang gõ chưa phải từ đúng (kèm dấu cách). Mặc định `false` = chỉ Tab.
    pub suggest_space_complete: bool,
    /// `config.output_charset` — bảng mã chữ đi ra document (`transform/charset.rs`).
    pub output_charset: OutputCharset,
    /// `config.style_profile.words` (P7 — vòng 54): từ cá nhân (từ, count).
    /// Rỗng khi USP tắt/mặc định — hành vi prediction giữ nguyên.
    pub style_words: Vec<(String, u32)>,
    /// `config.style_profile.corrections` (P7): cặp sai→đúng cá nhân
    /// (suggestion-only ở assist — P7-DFMEA F8).
    pub style_corrections: Vec<(String, String)>,
    /// P7: master-switch HỌC (riêng với dùng-dữ-đã-config) — ffi gate theo
    /// `style_profile.enabled`. Mặc định false (G8).
    pub style_learning_enabled: bool,
}

impl EngineOptions {
    /// P7 (vòng 54): USP học chỉ khi enabled (config `style_profile.enabled`
    /// map qua `style_words/style_corrections` non-empty?) — dùng cờ riêng
    /// để tắt học nhưng vẫn giữ boost từ đã-config: learning bật khi CÓ
    /// cấu hình style profile (words/corrections được nạp) — và callers
    /// đặt `style_words` rỗng khi USP tắt (ffi gate).
    pub(crate) fn style_enabled_for_learning(&self) -> bool {
        self.style_learning_enabled
    }
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            method: Method::Telex,
            diacritic_style: DiacriticStyle::Old,
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
            suggest_vn_complete: false,
            suggest_space_complete: false,
            output_charset: OutputCharset::UnicodePrecomposed,
            style_words: Vec::new(),
            style_corrections: Vec::new(),
            style_learning_enabled: false,
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

/// Số ký tự **Unicode** tối đa của buffer câu (`Engine::sentence`) — D31 (ADR-013).
/// 256 = đủ một câu dài; buffer câu không phải lịch sử văn bản.
const SENTENCE_MAX: usize = 256;

/// Giới hạn `insert`/`preedit` của `ime_result_v1` — P0-2 §2: **engine tự giữ** ≤ 64
/// (FFI còn một lưới an toàn cuối; macro dài hơn 64 ký tự bị cắt tại đây).
pub const MAX_TEXT: usize = 64;

/// Số phím tối đa của một từ engine còn sở hữu: `raw` + 1 ký tự ranh giới phải vừa
/// `MAX_TEXT` để RESTORE/ESC trả lại đủ chuỗi gõ. Vượt → đóng từ (`close_word`).
const MAX_WORD_KEYS: usize = MAX_TEXT - 1;

/// Trần từ cá nhân sống trong phiên (`opts.style_words`).
const STYLE_SESSION_CAP: usize = 500;
/// Số từ mới nhất được miễn thay thế khi hồ sơ phiên đầy.
const STYLE_FRESH_GUARD: usize = 32;
/// Số lần thay từ giữa hai lần giảm nửa trọng số.
const STYLE_AGE_EVERY: u16 = 64;
/// Trần từ pending giữa hai lần drain.
const STYLE_PENDING_CAP: usize = 2000;
/// Trần mỗi lượt drain — khớp `config::STYLE_WORD_CAP` và trần cặp sửa.
const DRAIN_MAX_WORDS: usize = 200;
const DRAIN_MAX_CORR: usize = 50;

/// Engine — 1 instance = 1 thread (P0-2 §3). `Clone` cho `peek_completion`
/// (chạy thử Tab trên bản sao — trạng thái thật không đổi).
#[derive(Clone)]
pub struct Engine {
    /// `Arc`: `peek_completion` sao chép engine MỖI PHÍM — hồ sơ cá nhân/macro không
    /// được chép theo (R2-140). Ghi qua `Arc::make_mut`.
    opts: Arc<EngineOptions>,
    ctx: Context,
    word: buffer::Word,
    /// Đuôi text engine biết chắc đã nằm trong document (trước con trỏ) — cho macro/emoji
    /// và tự viết hoa đầu câu (`post/caps.rs::sentence_start`).
    ///
    /// Đây là buffer **glyph** (đã encode theo `output_charset`) — khác `sentence`.
    recent: Vec<char>,
    /// Đuôi text **Unicode** của document (ADR-013): buffer câu cho trợ lý ngữ pháp.
    ///
    /// Cap `SENTENCE_MAX` (256 ký tự Unicode). Luôn bật (D31) — chỉ đọc, không I/O.
    sentence: Vec<char>,
    /// Số **glyph** mà mỗi ký tự của `sentence` chiếm trong document (1 hoặc 2:
    /// VNI Windows / Unicode tổ hợp). Song song `sentence` — dùng để xoá đúng số
    /// ký tự Unicode khi engine xoá `delete` glyph khỏi document.
    sentence_widths: Vec<u8>,
    /// Vòng lặp Tab dự đoán từ/cụm từ (P6 Text Prediction).
    last_predict: Option<PredictCycle>,
    /// P7 (vòng 54): dữ liệu học CÁ NHÂN pending (chỉ khi USP bật) —
    /// từ gõ xong + cặp user chấp nhận. Drain qua FFI rồi XÓA (G8: chỉ từ +
    /// count tối thiểu, không giờ/app/câu; Debug không in nội dung).
    style_pending: Arc<StylePending>,
    /// Bản sao chạy thử của `peek_completion`: không học gì (R2-140).
    probe: bool,
    /// Số lần thay từ khi hồ sơ phiên đầy — đủ `STYLE_AGE_EVERY` mới giảm trọng số.
    style_evictions: u16,
    /// Vừa đổi `a--` thành `a—`: ranh giới **ngay** phím kế tiếp không phải chữ ⇒ đó là
    /// code (`i--;`), trả lại `--` (R2-141). `em_dash_next` là cờ đặt ở phím này,
    /// chuyển sang `em_dash_tentative` đúng một phím.
    em_dash_tentative: bool,
    em_dash_next: bool,
    /// Từ vừa đóng có phím ngoặc đã thành nguyên âm (`raw`, chữ hiển thị): nếu ngay sau đó,
    /// không qua khoảng trắng, gặp `]` không có `[` mở thì đó là chỉ mục (`m[n-1]`) — trả
    /// nguyên văn cả cụm (R2-140).
    bracket_word: Option<(Vec<char>, Vec<char>)>,
    /// Vừa học ứng viên: ranh giới Mode EN kế tiếp không học lại chính chữ đó.
    accepted_just_now: bool,

    /// P10 — Dream RSI: counters đo hiệu quả prediction (vĩnh viễn theo
    /// engine instance; drain JSON xuất kèm StyleUpdate — không xóa).
    dream_rsi: DreamRsi,
    /// Phần `dream_rsi` đã giao cho host qua drain (đã `style_clear_pending`) — để mỗi lượt
    /// drain chỉ xuất **delta** (host cộng dồn vào `config.style_profile.rsi`, không đếm đúp).
    rsi_reported: DreamRsi,
    /// Ảnh `dream_rsi` lúc `style_peek_json` xuất gần nhất; `style_clear_pending` chốt nó thành
    /// `rsi_reported` (phím gõ giữa peek và clear không bị mất).
    rsi_peeked: std::cell::Cell<DreamRsi>,
    /// Trạng thái phím Ctrl đơn lẻ (nhấn rồi thả riêng `Ctrl` để khoá từ tiếng Việt hợp lệ
    /// hiện tại và đóng ranh giới từ: `tunx` + `Ctrl` → giữ `tũn`; với âm tiết không hợp lệ
    /// trong tiếng Việt như `sort` → `sỏt`, bấm `Ctrl` sẽ khôi phục về `sort`).
    ctrl_tap_armed: bool,
}

fn valid_style_text(s: &str) -> bool {
    let n = s.chars().count();
    if !(2..=32).contains(&n) {
        return false;
    }
    if s != s.trim() || s.contains("  ") || s.split_whitespace().count() > 3 {
        return false;
    }
    s.chars()
        .all(|c| c.is_alphabetic() || c == ' ' || c == '-' || c == '\'')
}

/// Hình dạng có thể là một từ thật (không phải gõ nhầm/rác bàn phím) — lọc trước khi HỌC từ ASCII:
/// không lặp một chữ ≥ 3 lần liền (`aaaa`, `wwwww`), không chuỗi phụ âm dài ≥ 6 (`sdfghjk`), và từ dài
/// hơn 4 chữ phải có nguyên âm (viết tắt `npm`, `ssh`, `css` thì không cần). Học nhầm rác làm gợi ý
/// sau này đề xuất rác — hồ sơ cá nhân càng dùng càng phải tốt hơn, không tệ đi.
fn plausible_ascii_word(tok: &str) -> bool {
    let letters: Vec<char> = tok
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if letters.windows(3).any(|w| w[0] == w[1] && w[1] == w[2]) {
        return false;
    }
    let is_vowel = |c: &char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
    if letters.len() > 4 && !letters.iter().any(is_vowel) {
        return false;
    }
    let mut run = 0usize;
    for c in &letters {
        run = if is_vowel(c) { 0 } else { run + 1 };
        if run >= 6 {
            return false;
        }
    }
    true
}

/// P7: entry hợp lệ để HỌC vào từ điển cá nhân (`style_words`) — chặn G8 (đoạn văn),
/// số, dấu câu, điều khiển, và chặn tuyệt đối các âm tiết có dấu nhưng sai cấu trúc
/// âm vị học tiếng Việt (như `sỏt`, `cútomer`, `fỏwad` bị Telex biến đổi tạm thời).
fn valid_style_entry(s: &str) -> bool {
    if !valid_style_text(s) {
        return false;
    }
    for tok in s.split_whitespace() {
        if tok.is_ascii() && !plausible_ascii_word(tok) {
            return false;
        }
        if !tok.is_ascii() {
            for part in tok.split('-').filter(|p| !p.is_empty() && !p.is_ascii()) {
                let chars: Vec<char> = part.chars().collect();
                if !validate::is_valid_word(&chars) {
                    return false;
                }
            }
        }
    }
    true
}

/// Pending learning — P7. `words`: từ → delta count; `corrections`: (from,to) → delta.
#[derive(Default, Clone)]
pub struct StylePending {
    words: std::collections::BTreeMap<String, u32>,
    corrections: std::collections::BTreeMap<(String, String), u32>,
}

/// P10 — Dream RSI (vòng 58): đo **số phím tiết kiệm** nhờ prediction.
///
/// Mục tiêu "Dream": bộ gõ tự học ngày càng đoán đúng ⇒ savings rate
/// (chars_saved / keys_typed) **luôn tiến hóa tăng dần**. G8: chỉ counter
/// tổng hợp, không có từ/ nội dung.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DreamRsi {
    /// Phím user gõ thật (không đếm phím engine inject).
    pub keys_typed: u64,
    /// Số lần user nhận prediction (Tab/Space/Ctrl+Space/cycle).
    pub accepts: u64,
    /// Tổng ký tự không phải gõ nhờ prediction hoàn tất.
    pub chars_saved: u64,
}

impl DreamRsi {
    /// Savings rate phần trăm: chars_saved / (chars_saved + keys_typed).
    /// 0 khi chưa có dữ liệu (không chia 0 panic — fail-open).
    pub fn savings_rate_pct(&self) -> u64 {
        let total = self.chars_saved.saturating_add(self.keys_typed);
        if total == 0 {
            return 0;
        }
        self.chars_saved.saturating_mul(100) / total
    }
}

impl std::fmt::Debug for StylePending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // G8: KHÔNG in từ/cặp — chỉ số lượng.
        f.debug_struct("StylePending")
            .field("words", &self.words.len())
            .field("corrections", &self.corrections.len())
            .finish()
    }
}

#[derive(Debug, Clone)]
struct PredictCycle {
    inserted_glyphs: usize,
    /// Ứng viên đã đặt dấu theo kiểu người dùng (`predict::restyle`).
    candidates: Vec<String>,
    current_index: usize,
    sample: String,
    had_leading_space: bool,
    /// `true` = cụm nối tiếp (next-phrase): chữ hoa theo
    /// `adapt_continuation_case`; `false` = hoàn tất chính từ đang gõ.
    continuation: bool,
    /// Phần vòng duyệt này đã cộng vào Dream RSI — gỡ lại nếu người dùng từ chối.
    rsi_accepts: u64,
    rsi_saved: u64,
    /// Ứng viên dài nhất đã chèn: duyệt dài → ngắn → dài không tính tiết kiệm hai lần.
    peak_glyphs: usize,
}

/// Gõ `chunk` + một dấu cách qua engine mới, mô phỏng document (Replace/Restore
/// xoá rồi chèn, Commit chèn, Pass chèn chính ký tự) rồi bỏ dấu cách cuối.
fn retype_chunk(opts: &EngineOptions, chunk: &[char]) -> Vec<char> {
    let mut engine = Engine::new(opts.clone());
    let mut doc: Vec<char> = Vec::with_capacity(chunk.len() + 1);
    for &c in chunk.iter().chain(std::iter::once(&' ')) {
        let key = if c == ' ' {
            KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..KeyEvent::default()
            }
        } else {
            KeyEvent::char_down(c)
        };
        match engine.key(&key).action {
            Action::Pass => doc.push(c),
            Action::Replace {
                delete_count,
                insert,
            }
            | Action::Restore {
                delete_count,
                insert,
            } => {
                let keep = doc.len().saturating_sub(usize::from(delete_count));
                doc.truncate(keep);
                doc.extend(insert);
            }
            Action::Commit { insert } => doc.extend(insert),
        }
    }
    if doc.last() == Some(&' ') {
        doc.pop();
    }
    doc
}

/// Thay đổi đuôi document mà engine ghi nhận (P4-2a · ADR-013).
///
/// `recent` (glyph) và `sentence` (Unicode) là **hai buffer tách bạch**: app làm việc
/// trên **glyph** (bảng mã xuất), trợ lý ngữ pháp làm việc trên **char Unicode**.
enum Tail<'a> {
    /// Xoá `delete` **glyph** cuối của document rồi thêm phần mới.
    Replace {
        /// Số glyph bị xoá (bằng `delete_count` của `Action`).
        delete: usize,
        /// Chính chuỗi **glyph** app nhận vào document (đã encode theo `output_charset`).
        glyph: &'a [char],
        /// Bản **Unicode** tương ứng — nuôi buffer câu (`sentence`).
        unicode: &'a [char],
    },
    /// Xoá hết đuôi engine còn nhớ (focus chuyển, con trỏ nhảy, chord…).
    Clear,
    /// App xoá 1 glyph (Backspace) — xoá 1 glyph + 1 ký tự Unicode (F66).
    Pop,
}

impl Engine {
    pub fn new(opts: EngineOptions) -> Engine {
        Engine {
            opts: Arc::new(opts),
            style_pending: Arc::default(),
            probe: false,
            style_evictions: 0,
            em_dash_tentative: false,
            em_dash_next: false,
            bracket_word: None,
            accepted_just_now: false,
            dream_rsi: DreamRsi::default(),
            rsi_reported: DreamRsi::default(),
            rsi_peeked: std::cell::Cell::new(DreamRsi::default()),
            ctx: Context::default(),
            word: buffer::Word::default(),
            recent: Vec::new(),
            sentence: Vec::new(),
            sentence_widths: Vec::new(),
            last_predict: None,
            ctrl_tap_armed: false,
        }
    }

    /// `ime_reload_config` — đổi options giữa chừng → reset từ đang gõ dở.
    /// P7 (vòng 54): cặp sai→đúng cá nhân cho assist (rỗng khi tắt/mặc định).
    pub fn options_style_corrections(&self) -> Vec<(String, String)> {
        self.opts.style_corrections.clone()
    }

    /// Có được học từ phím vừa gõ không. Ngoài opt-in của người dùng còn loại mọi
    /// ngữ cảnh có thể là bí mật hoặc không phải văn bản: ô mật khẩu, app passthrough,
    /// **terminal** (`sudo` hỏi mật khẩu không phải ô secure — G8, R2-145).
    pub(crate) fn style_enabled_for_learning(&self) -> bool {
        self.opts.style_enabled_for_learning() && !self.probe && !self.prediction_blocked()
    }

    /// Ngữ cảnh không được dự đoán/chiếm Tab: ô mật khẩu, app passthrough, terminal
    /// (Tab là hoàn tất của shell — `git che`+Tab không được ra `git chef`, R2-144).
    fn prediction_blocked(&self) -> bool {
        self.secure()
            || self.ctx.hint == strategy::IME_STRATEGY_PASSTHROUGH
            || self.ctx.field_role == strategy::IME_FIELD_TERMINAL
    }

    /// P7: ghi cặp user chấp nhận (validate: 2..=32 ký tự, alphabetic+space).
    /// Chỉ ghi khi tính năng học style_profile.enabled bật (G8 opt-in).
    pub fn style_note_correction(&mut self, from: &str, to: &str) {
        if !self.style_enabled_for_learning() {
            return;
        }
        if !valid_style_text(from) || !valid_style_entry(to) {
            return;
        }
        if from.eq_ignore_ascii_case(to) {
            return; // from == to (kể cả chỉ khác hoa/thường) — không phải lỗi sửa
        }
        let pair = (from.trim().to_lowercase(), to.trim().to_lowercase());
        *Arc::make_mut(&mut self.style_pending)
            .corrections
            .entry(pair.clone())
            .or_insert(0) += 1;
        if !self
            .opts
            .style_corrections
            .iter()
            .any(|(f, t)| f == &pair.0 && t == &pair.1)
            && self.opts.style_corrections.len() < 50
        {
            Arc::make_mut(&mut self.opts).style_corrections.push(pair);
        }
    }

    /// P7: học từ user gõ xong (gọi tại word-end khi USP bật).
    pub fn style_note_word(&mut self, word: &str) {
        if !self.style_enabled_for_learning() {
            return;
        }
        if !valid_style_entry(word) {
            return;
        }
        let lower = word.to_lowercase();
        // Host không drain (chưa nối) thì pending vẫn có trần — không phình theo phiên.
        let pending = Arc::make_mut(&mut self.style_pending);
        if pending.words.len() < STYLE_PENDING_CAP || pending.words.contains_key(&lower) {
            *pending.words.entry(lower.clone()).or_insert(0) += 1;
        }

        // Tiến hóa gợi ý thời gian thực trong phiên (in-session continuous learning):
        // Cập nhật opts.style_words ngay lập tức để các lần gõ tiếp theo nhận được dự đoán
        // từ mới mà không cần đợi host app drain và nạp lại cấu hình.
        let mut need_decay = false;
        let opts = Arc::make_mut(&mut self.opts);
        if let Some(entry) = opts.style_words.iter_mut().find(|(w, _)| w == &lower) {
            entry.1 = entry.1.saturating_add(1);
            if entry.1 >= 256 {
                need_decay = true;
            }
        } else {
            if opts.style_words.len() >= STYLE_SESSION_CAP {
                // Đầy: thay từ ít dùng nhất trong phần CŨ của danh sách (thứ tự = thứ tự
                // thêm) — từ vừa học không loại lẫn nhau. Trọng số chỉ giảm nửa sau mỗi
                // `STYLE_AGE_EVERY` lần thay: giảm mỗi lần thì vài từ mới đủ xoá lịch sử
                // của từ dùng nhiều nhất (R2-146).
                let old = opts.style_words.len().saturating_sub(STYLE_FRESH_GUARD);
                if let Some(min_idx) = opts.style_words[..old]
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, (_, count))| *count)
                    .map(|(idx, _)| idx)
                {
                    opts.style_words.remove(min_idx);
                }
                self.style_evictions = self.style_evictions.saturating_add(1);
                if self.style_evictions >= STYLE_AGE_EVERY {
                    self.style_evictions = 0;
                    need_decay = true;
                }
            }
            opts.style_words.push((lower, 1));
        }
        if need_decay {
            for (_, c) in &mut opts.style_words {
                *c = (*c / 2).max(1);
            }
        }
    }

    /// Học cụm hai từ `<từ trước> <current>` khi từ `current` đóng tại `before_idx` trong
    /// buffer câu. Phải có **khoảng trắng thật** ngay trước (`abc-def`, tách từ bằng Ctrl
    /// không phải cụm) và từ trước phải là chữ trọn vẹn, không dính dấu câu.
    fn style_note_bigram(&mut self, before_idx: usize, current: &str) {
        if before_idx == 0 || !matches!(self.sentence[before_idx - 1], ' ' | '\t') {
            return;
        }
        let lookback = before_idx.saturating_sub(40);
        let tail: String = self.sentence[lookback..before_idx].iter().collect();
        let same_line = tail.trim_end_matches([' ', '\t']);
        if same_line.ends_with(['\n', '\r']) {
            return;
        }
        let Some(prev) = same_line.split_whitespace().last() else {
            return;
        };
        let word_like = prev.chars().all(|ch| ch.is_alphabetic() || ch == '-')
            && !prev.starts_with('-')
            && !prev.ends_with('-');
        if word_like {
            let phrase = format!("{prev} {current}");
            if valid_style_entry(&phrase) {
                self.style_note_word(&phrase);
            }
        }
    }

    /// Chốt vòng duyệt ứng viên. Ứng viên đang hiển thị được học ở đây chứ không phải
    /// lúc chèn: ứng viên bị Tab bỏ qua không được vào hồ sơ, và xoá ngay (Backspace/Esc)
    /// là từ chối (R2-147).
    fn settle_predict(&mut self, accepted: bool) {
        let Some(cycle) = self.last_predict.take() else {
            return;
        };
        if !accepted {
            // Bị từ chối: gỡ phần đã tính vào Dream RSI — số đo chỉ gồm chữ người dùng giữ.
            self.dream_rsi.accepts = self.dream_rsi.accepts.saturating_sub(cycle.rsi_accepts);
            self.dream_rsi.chars_saved = self.dream_rsi.chars_saved.saturating_sub(cycle.rsi_saved);
            return;
        }
        if let Some(cand) = cycle.candidates.get(cycle.current_index) {
            self.note_accepted(cand, &cycle.sample, cycle.continuation);
        }
    }

    /// P7 Hook 2: học ứng viên người dùng nhận (bản lowercase chuẩn). Cụm nối tiếp học
    /// cả dạng `<từ trước> <cụm>` để lần sau khớp theo ngữ cảnh.
    fn note_accepted(&mut self, cand: &str, sample: &str, continuation: bool) {
        self.accepted_just_now = true;
        if continuation {
            let full_phrase = format!("{sample} {cand}");
            if valid_style_entry(&full_phrase) {
                self.style_note_word(&full_phrase.to_lowercase());
            }
            if valid_style_entry(cand) {
                self.style_note_word(&cand.to_lowercase());
            }
        } else {
            self.style_note_word(&cand.to_lowercase());
        }
    }

    /// Trả từ đang mở về chuỗi gõ (`boundary` nếu có đi theo sau) — dùng chung cho Ctrl
    /// tap, Ctrl+Space và ranh giới từ (RESTORE = REPLACE, P0-4 §3).
    fn restore_raw(&mut self, boundary: Option<char>) -> Outcome {
        let delete = self.word.owned as u16;
        let mut insert = self.word.raw.clone();
        insert.extend(boundary);
        self.word.clear();
        // Document nhận đúng chuỗi gõ (không encode) → glyph = Unicode.
        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &insert,
            unicode: &insert,
        });
        Outcome {
            action: Action::Restore {
                delete_count: delete,
                insert,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        }
    }

    /// Phím ngoặc Telex (`[` → `ơ`, `]` → `ư`) là **dấu ngoặc thật** (ranh giới từ): ở đầu
    /// từ (ngoặc mở thật phổ biến hơn hẳn vần ơ/ư — những vần đó gõ `ow`/`uw`), khi đóng một
    /// `[` đang mở trên dòng, khi không tạo được âm tiết Việt, hay trong ô lệnh. Giữ
    /// `arr[0]`, `[1, 2, 3]`, `- [ ] việc`, `[a, b]`, `[tôi]` nguyên vẹn (R2-140).
    fn bracket_is_boundary(&self, c: char) -> bool {
        if !matches!(c, '[' | ']') || self.opts.method != Method::Telex {
            return false;
        }
        if self.word.is_empty()
            || self.word.identifier
            || matches!(
                self.ctx.field_role,
                strategy::IME_FIELD_TERMINAL | strategy::IME_FIELD_ADDRESS_BAR
            )
        {
            return true;
        }
        // Bấm lại đúng phím vừa thành nguyên âm = gỡ về dấu ngoặc (`m[[` → `m[`).
        let vowel_undo = self.word.raw.last() == Some(&c)
            && self
                .word
                .display
                .last()
                .is_some_and(|&d| transform::tone::is_vowel(d));
        if vowel_undo {
            return false;
        }
        // Đóng `[` đang mở: `[x]`, `[a, b]`. Từ Việt trong ngoặc (`[… ch]a …]`) có ít nhất
        // hai phím trước `]` — một chữ cái đơn trước `]` là mục danh sách/checkbox.
        if c == ']' && self.word.raw.len() < 2 && self.open_bracket_in_line() {
            return true;
        }
        let mut raw = self.word.raw.clone();
        raw.push(c);
        let mut display = self.fold_keys(&raw);
        if validate::is_valid_word(&display) {
            return false;
        }
        // Âm tiết còn dở cần âm cuối (`thươ` của `th][ng`).
        display.push('n');
        !validate::is_valid_word(&display)
    }

    /// Dòng đang gõ còn `[` mở chưa đóng. Chỉ tính `[` mở cụm (đầu dòng, sau khoảng trắng
    /// hay dấu câu) — `[` dính sau chữ (`m[[` gỡ nguyên âm, `arr[`) là chỉ mục, không phải
    /// ngoặc bao chữ.
    fn open_bracket_in_line(&self) -> bool {
        let mut closes = 0usize;
        for (i, &c) in self.sentence.iter().enumerate().rev() {
            match c {
                '\n' | '\r' => return false,
                ']' => closes += 1,
                '[' if closes > 0 => closes -= 1,
                '[' => {
                    let opens_group = i == 0 || !self.sentence[i - 1].is_alphanumeric();
                    if opens_group {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// `]` ngay sau một từ có phím ngoặc đã thành nguyên âm, không qua khoảng trắng và không
    /// có `[` mở: trả nguyên văn cả cụm chỉ mục — `m[n-1]`, `s[:5]`, `d["k"]`, `x[1]`. Chữ
    /// Việt (`chơi: `, `hơn-kém `) không bao giờ gặp `]` kiểu này nên không bị đụng.
    fn restore_bracket_index(&mut self) -> Option<Outcome> {
        let (raw, display) = self.bracket_word.take()?;
        let n = self.sentence.len();
        let start = n.checked_sub(display.len())?;
        let mut at = None;
        for s in (n.saturating_sub(display.len() + 24)..=start).rev() {
            if self.sentence[s..s + display.len()] == display[..] {
                at = Some(s);
                break;
            }
        }
        let at = at?;
        let rest: Vec<char> = self.sentence[at + display.len()..].to_vec();
        // `chơi[1]` (chú thích): `[` mới mở trong phần sau là ngoặc riêng, chữ trước giữ nguyên.
        if rest.iter().any(|c| c.is_whitespace() || *c == '[') {
            return None;
        }
        let delete = self.emit(&display).len() + self.emit(&rest).len();
        if delete > self.recent.len() {
            return None;
        }
        // Chữ chưa biến đổi sau dấu (`s[i:j]` — `j` đã đi thẳng vào document) nằm trong `rest`.
        self.word.clear();
        let mut insert = raw;
        insert.extend(rest);
        insert.push(']');
        let glyph = self.emit(&insert);
        self.record(Tail::Replace {
            delete,
            glyph: &glyph,
            unicode: &insert,
        });
        Some(Outcome {
            action: Action::Replace {
                delete_count: delete as u16,
                insert: glyph,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// Từ đang mở có phím `[` đã thành nguyên âm, gặp `]` cho thấy đây là chỉ mục: không có
    /// `[` mở (`s[i]`) hoặc `[` dính sau định danh (`foo[bar]`) → trả lại nguyên văn. Chỉ mục
    /// có ký tự khác ở giữa (`m[n-1]`) do `restore_bracket_index` xử lý khi gặp `]`.
    fn bracket_word_is_code(&self, c: char) -> bool {
        let raw = &self.word.raw;
        if !self.word.active {
            return false;
        }
        // `foo[bar]`: `[` dính sau định danh là chỉ mục, không phải ngoặc văn xuôi `[đây]`.
        let index_close = c == ']'
            && self
                .sentence
                .len()
                .checked_sub(self.word.display.len() + 2)
                .is_some_and(|i| self.sentence[i].is_alphanumeric() && self.sentence[i + 1] == '[');
        let unmatched_close = c == ']' && raw.contains(&'[') && !self.open_bracket_in_line();
        index_close || unmatched_close
    }

    /// P7: xem JSON pending hiện tại mà KHÔNG xóa (dùng cho FFI 2 pha / probe size).
    pub fn style_peek_json(&self) -> String {
        let delta = DreamRsi {
            keys_typed: self
                .dream_rsi
                .keys_typed
                .saturating_sub(self.rsi_reported.keys_typed),
            accepts: self
                .dream_rsi
                .accepts
                .saturating_sub(self.rsi_reported.accepts),
            chars_saved: self
                .dream_rsi
                .chars_saved
                .saturating_sub(self.rsi_reported.chars_saved),
        };
        if self.style_pending.words.is_empty()
            && self.style_pending.corrections.is_empty()
            && delta.keys_typed == 0
            && delta.accepts == 0
            && delta.chars_saved == 0
        {
            return "{}".to_string();
        }
        self.rsi_peeked.set(self.dream_rsi);
        let words: Vec<String> = self
            .drain_words()
            .into_iter()
            .map(|(w, n)| format!("{{\"word\":\"{w}\",\"count\":{n}}}"))
            .collect();
        let corr: Vec<String> = self
            .drain_corrections()
            .into_iter()
            .map(|((f, t), n)| format!("{{\"from\":\"{f}\",\"to\":\"{t}\",\"count\":{n}}}"))
            .collect();
        format!(
            "{{\"words\":[{}],\"corrections\":[{}],\"rsi\":{{\"keys_typed\":{},\"accepts\":{},\"chars_saved\":{},\"savings_rate_pct\":{}}},\"rsi_delta\":{{\"keys_typed\":{},\"accepts\":{},\"chars_saved\":{}}}}}",
            words.join(","),
            corr.join(","),
            self.dream_rsi.keys_typed,
            self.dream_rsi.accepts,
            self.dream_rsi.chars_saved,
            self.dream_rsi.savings_rate_pct(),
            delta.keys_typed,
            delta.accepts,
            delta.chars_saved
        )
    }

    /// Lượt drain lấy các từ pending nhiều lần gõ nhất trước (trước đây cắt theo vần
    /// ABC — từ cuối bảng chữ cái luôn mất); JSON vẫn xếp ABC cho ổn định.
    fn drain_words(&self) -> Vec<(&String, &u32)> {
        let mut words: Vec<(&String, &u32)> = self.style_pending.words.iter().collect();
        words.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        words.truncate(DRAIN_MAX_WORDS);
        words.sort();
        words
    }

    fn drain_corrections(&self) -> Vec<(&(String, String), &u32)> {
        let mut corr: Vec<(&(String, String), &u32)> =
            self.style_pending.corrections.iter().collect();
        corr.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        corr.truncate(DRAIN_MAX_CORR);
        corr.sort();
        corr
    }

    /// P7: xóa đúng phần `style_peek_json` vừa xuất sau khi caller đã copy thành công —
    /// phần vượt trần drain ở lại cho lượt sau, không mất (R2-148).
    pub fn style_clear_pending(&mut self) {
        let words: Vec<String> = self
            .drain_words()
            .into_iter()
            .map(|(w, _)| w.clone())
            .collect();
        let corr: Vec<(String, String)> = self
            .drain_corrections()
            .into_iter()
            .map(|(p, _)| p.clone())
            .collect();
        let pending = Arc::make_mut(&mut self.style_pending);
        for w in &words {
            pending.words.remove(w);
        }
        for p in &corr {
            pending.corrections.remove(p);
        }
        // Số đo Dream RSI đã giao cùng lượt peek: lượt sau chỉ xuất phần tăng thêm.
        let peeked = self.rsi_peeked.get();
        self.rsi_reported = DreamRsi {
            keys_typed: self.rsi_reported.keys_typed.max(peeked.keys_typed),
            accepts: self.rsi_reported.accepts.max(peeked.accepts),
            chars_saved: self.rsi_reported.chars_saved.max(peeked.chars_saved),
        };
    }

    /// P7: drain pending dạng JSON `{"words":[..],"corrections":[..]}` rồi XÓA.
    /// Rỗng → `"{}"`. JSON tối thiểu, không thư viện ngoài.
    pub fn style_drain_json(&mut self) -> String {
        let json = self.style_peek_json();
        if json != "{}" {
            self.style_clear_pending();
        }
        json
    }

    /// P10 — Dream RSI: snapshot counters (không xóa — vĩnh viễn theo engine).
    pub fn dream_rsi_snapshot(&self) -> DreamRsi {
        self.dream_rsi
    }

    pub fn set_options(&mut self, opts: EngineOptions) {
        self.settle_predict(true);
        self.opts = Arc::new(opts);
        self.word.clear();
        self.record(Tail::Clear);
    }

    pub fn options(&self) -> &EngineOptions {
        &self.opts
    }

    /// OP-007b · R2-139 (chủ repo: "bôi đen văn bản ở bảng mã khác chuyển về Telex
    /// hoặc mode đang chạy", "mở file lên nhưng chữ bị biến thành ký tự lạ") —
    /// chuyển đổi THÔNG MINH một vùng bôi đen, theo thứ tự:
    ///
    /// 1. chữ lạ do app đọc nhầm UTF-8 theo cp1252/cp1258 → khôi phục
    ///    (`repair_mojibake`);
    /// 2. bảng mã cũ (TCVN3/ABC, VNI-Windows, Unicode tổ hợp — `detect`) → giải mã;
    /// 3. kết quả xuất sang **bảng mã TextVN đang dùng** (`output_charset`);
    /// 4. không có gì ở 1–3 ⇒ các từ **thuần ASCII** được gõ lại theo kiểu gõ
    ///    đang dùng (`tooi ddi hocj` → `tôi đi học` — gõ nhầm chế độ E); từ đã có
    ///    dấu giữ nguyên.
    ///
    /// `None` = không đổi gì (caller không ghi document). Không đổi trạng thái
    /// engine thật (gõ lại chạy trên engine riêng).
    pub fn convert_selection(&self, text: &[char]) -> Option<Vec<char>> {
        let unicode = repair_mojibake(text).or_else(|| match detect(text) {
            Charset::UnicodePrecomposed => None,
            from => Some(transcode(
                text,
                from,
                Charset::UnicodePrecomposed,
                TextCase::AsIs,
            )),
        });
        let encoded = self.emit(unicode.as_deref().unwrap_or(text));
        if encoded != text {
            return Some(encoded);
        }
        let retyped = self.retype_ascii_words(text);
        (retyped != text).then_some(retyped)
    }

    /// Gõ lại từng cụm thuần ASCII (ngăn bởi khoảng trắng) qua một engine mới cùng
    /// kiểu gõ/bảng mã/kiểu dấu — kết quả y như người dùng gõ cụm đó ở chế độ V.
    /// Tắt gõ tắt/emoji/tự viết hoa/hoàn tất bằng Space: chỉ chuyển phím, không
    /// thêm chữ.
    fn retype_ascii_words(&self, text: &[char]) -> Vec<char> {
        let mut opts = EngineOptions::clone(&self.opts);
        opts.enabled = true;
        opts.macros.clear();
        opts.emoji.clear();
        opts.auto_capitalize = false;
        opts.suggest_space_complete = false;
        // Gõ lại không dự đoán, không học: bỏ hồ sơ cá nhân khỏi bản sao (mỗi cụm một engine).
        opts.style_learning_enabled = false;
        opts.style_words.clear();
        opts.style_corrections.clear();
        let mut out = Vec::with_capacity(text.len());
        let mut i = 0;
        while i < text.len() {
            if text[i].is_whitespace() {
                out.push(text[i]);
                i += 1;
                continue;
            }
            let start = i;
            while i < text.len() && !text[i].is_whitespace() {
                i += 1;
            }
            let chunk = &text[start..i];
            if chunk.iter().all(|c| c.is_ascii_graphic()) {
                out.extend(retype_chunk(&opts, chunk));
            } else {
                out.extend_from_slice(chunk);
            }
        }
        out
    }

    pub fn set_context(&mut self, ctx: Context) {
        self.ctx = ctx;
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// Ô mật khẩu: cờ `secure` HOẶC role `secure` (adapter quên một trong hai vẫn an toàn — S3).
    fn secure(&self) -> bool {
        self.ctx.secure || self.ctx.field_role == strategy::IME_FIELD_SECURE
    }

    /// `ime_reset` — xóa trạng thái từ (focus change). Không đụng buffer của app.
    pub fn reset(&mut self) {
        // Đổi focus/click sau khi nhận ứng viên = giữ ứng viên đó.
        self.settle_predict(true);
        self.bracket_word = None;
        self.word.clear();
        self.record(Tail::Clear);
    }

    /// `ime_peek_completion` (P6-11 · R2-134, chủ repo: "chữ mờ gợi ý sẵn") —
    /// phần chữ mà Tab **sẽ** nối thêm ngay sau con trỏ, để adapter vẽ mờ (ghost
    /// text) trước khi người dùng nhấn. Chạy thử Tab trên **bản sao** engine
    /// (giữ cả vòng duyệt đang dở): trạng thái thật không đổi, kết quả đúng y
    /// như Tab sẽ làm. `None` khi dự đoán tắt, Tab không khớp gì, hoặc kết quả
    /// không phải nối thêm thuần (Tab đổi chữ đã gõ — duyệt vòng, khôi phục
    /// tiếng Anh) ⇒ không vẽ.
    pub fn peek_completion(&self) -> Option<String> {
        if !self.opts.suggest_vn_complete {
            return None;
        }
        let mut probe = self.clone();
        probe.probe = true;
        let tab = KeyEvent {
            vk: keymap::vk::TAB,
            ch: '\t' as u32,
            key_down: true,
            ..KeyEvent::default()
        };
        let Action::Replace {
            delete_count,
            insert,
        } = probe.key(&tab).action
        else {
            return None;
        };
        let n = usize::from(delete_count);
        if insert.len() <= n || n > self.recent.len() {
            return None;
        }
        // Phần bị "xoá" phải đúng là chữ đang nằm trước con trỏ ⇒ Tab chỉ nối thêm.
        if insert[..n] != self.recent[self.recent.len() - n..] {
            return None;
        }
        let ghost: String = insert[n..].iter().collect();
        (!ghost.trim().is_empty()).then_some(ghost)
    }

    /// `ime_reset_with_tail` (R2-131) — reset rồi ghi nhận `unicode_tail` là đuôi
    /// document ngay trước con trỏ. Adapter gọi sau khi trợ lý sửa câu (nút "Sửa
    /// tất cả", phím tắt): document giờ chứa câu ĐÃ sửa — nạp lại để lần dự đoán
    /// kế tiếp còn ngữ cảnh ("sửa hết rồi dự đoán" bằng cùng một phím) và lượt
    /// kiểm tra sau không thấy câu cũ. Không mở từ mới (từ đã chốt).
    pub fn reset_with_tail(&mut self, unicode_tail: &str) {
        self.reset();
        let unicode: Vec<char> = unicode_tail.chars().collect();
        if unicode.is_empty() || unicode.iter().any(|c| c.is_control()) {
            return;
        }
        let glyph = self.emit(&unicode);
        self.record(Tail::Replace {
            delete: 0,
            glyph: &glyph,
            unicode: &unicode,
        });
    }

    /// Ghi nhận thay đổi đuôi document vào **cả hai** buffer (`recent` glyph +
    /// `sentence` Unicode) — chỗ **duy nhất** được sửa chúng (P4-2a · ADR-013).
    ///
    /// - `recent`: đếm trên **glyph** (đúng thứ app nhận) — giữ nguyên semantics của
    ///   `recent_replace` cũ, không đổi hành vi gõ.
    /// - `sentence`: đếm trên **char Unicode** (bộ trợ lý ngữ pháp đọc câu này).
    ///
    /// Hai buffer **tách bạch**: 1 chữ có thể là 2 glyph (VNI Windows / Unicode tổ hợp)
    /// nên số ký tự xoá của `sentence` không suy ra được từ số glyph của `recent`
    /// → giữ chiều rộng từng ký tự trong `sentence_widths`.
    fn record(&mut self, edit: Tail<'_>) {
        match edit {
            Tail::Clear => {
                self.recent.clear();
                self.sentence.clear();
                self.sentence_widths.clear();
            }
            Tail::Pop => {
                // App xoá 1 glyph: `recent` bỏ 1 glyph, `sentence` bỏ 1 ký tự Unicode
                // (F66: 1 chữ 2 byte TCVN3/VNI = 1 glyph nhưng 1 ký tự Unicode).
                self.recent.pop();
                self.sentence.pop();
                self.sentence_widths.pop();
            }
            Tail::Replace {
                delete,
                glyph,
                unicode,
            } => {
                // --- buffer glyph (`recent`): giữ nguyên semantics cũ ---
                let keep = self.recent.len().saturating_sub(delete);
                self.recent.truncate(keep);
                self.recent.extend_from_slice(glyph);
                if self.recent.len() > RECENT_MAX {
                    let drop = self.recent.len() - RECENT_MAX;
                    self.recent.drain(..drop);
                }

                // --- buffer câu (`sentence`): đếm trên ký tự Unicode ---
                // Xoá `delete` glyph cuối: 1 ký tự Unicode có thể chiếm 2 glyph
                // → xoá đúng số ký tự Unicode tương ứng (không đồng nhất với glyph).
                let mut left = delete;
                while left > 0 {
                    match self.sentence_widths.pop() {
                        Some(w) => {
                            self.sentence.pop();
                            left = left.saturating_sub(usize::from(w));
                        }
                        None => break,
                    }
                }
                // Thêm phần mới: mỗi ký tự Unicode chiếm `w` glyph trong document.
                for &c in unicode {
                    // Document nhận đúng chuỗi Unicode (không encode) → 1 ký tự = 1 glyph;
                    // document nhận bản đã encode → độ rộng theo bảng mã (1–2 glyph/chữ).
                    let w = if glyph.len() == unicode.len() {
                        1
                    } else {
                        self.emit(&[c]).len() as u8
                    };
                    self.sentence.push(c);
                    self.sentence_widths.push(w);
                }
                // Cap `SENTENCE_MAX` (D31): giữ phần đuôi (gần con trỏ nhất).
                if self.sentence.len() > SENTENCE_MAX {
                    let drop = self.sentence.len() - SENTENCE_MAX;
                    self.sentence.drain(..drop);
                    self.sentence_widths.drain(..drop);
                }
            }
        }
        // Bat bien: `sentence` va `sentence_widths` luon cung do dai (F75).
        debug_assert_eq!(self.sentence.len(), self.sentence_widths.len());
    }

    /// Số ký tự **glyph** (đã encode theo `output_charset`) của chuỗi Unicode.
    ///
    /// Adapter cần biết xoá bao nhiêu ký tự trong document khi áp dụng một gợi ý:
    /// trợ lý ngữ pháp trả **char Unicode**, app nhận **glyph** (D27 · ADR-013).
    pub fn glyph_len_of(&self, unicode: &[char]) -> usize {
        self.emit(unicode).len()
    }

    /// Phần cuối buffer câu (Unicode), tối đa `max` ký tự — chỉ phục vụ vỡ câu.
    ///
    /// Không I/O, không log nội dung (Handbook S1–S4).
    pub fn sentence_tail(&self, max: usize) -> String {
        let start = self.sentence.len().saturating_sub(max);
        self.sentence[start..].iter().collect()
    }

    /// View tham chiếu đến phần cuối buffer câu (Unicode), tối đa `max` ký tự (Fassist-76).
    ///
    /// Không cấp phát heap (0-alloc), dùng làm nguồn câu cho các lần kiểm tra tức thì.
    pub fn sentence_tail_chars(&self, max: usize) -> &[char] {
        let start = self.sentence.len().saturating_sub(max);
        &self.sentence[start..]
    }

    /// Phần **Unicode** tương ứng với `glyph_len` ký tự **glyph** đầu của `unicode`.
    ///
    /// Bảng mã 2 ký tự/chữ (VNI Windows) → cắt theo glyph có thể rơi giữa một chữ:
    /// hàm này chỉ lấy chữ nào **vừa đủ** (`glyph_len` glyph), để buffer câu khớp
    /// đúng phần app đã nhận.
    fn unicode_prefix(&self, unicode: &[char], glyph_len: usize) -> Vec<char> {
        let mut out = Vec::new();
        let mut used = 0usize;
        for &c in unicode {
            let w = self.emit(&[c]).len();
            if used + w > glyph_len {
                break;
            }
            out.push(c);
            used += w;
        }
        out
    }

    /// Ghi nhận một phím **PASS** (adapter tự chèn) vào đuôi text cho macro/emoji.
    ///
    /// Ô mật khẩu (`secure`) → **không** giữ tail nào (S3).
    fn note_pass(&mut self, k: &KeyEvent) {
        if self.secure() {
            return;
        }
        match k.vk {
            keymap::vk::BACK => {
                self.record(Tail::Pop);
            }
            keymap::vk::DELETE
            | keymap::vk::LEFT
            | keymap::vk::RIGHT
            | keymap::vk::UP
            | keymap::vk::DOWN => self.record(Tail::Clear),
            _ => match k.printable() {
                Some(c) if !c.is_control() || matches!(c, ' ' | '\n' | '\t') => {
                    self.record(Tail::Replace {
                        delete: 0,
                        glyph: &[c],
                        unicode: &[c],
                    });
                }
                Some(_) => {}
                // Home/End/PageUp/F-key…: con trỏ có thể đã nhảy — đuôi text engine nhớ
                // không còn nằm ngay trước con trỏ, gõ tắt sau đó sẽ xoá nhầm chỗ.
                None => self.record(Tail::Clear),
            },
        }
    }

    /// Resolve strategy hiện tại (P0-3 §3.1).
    pub fn strategy(&self) -> Strategy {
        strategy::resolve(strategy::ResolveInput {
            secure: self.secure(),
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
        self.fold_keys(&self.word.raw)
    }

    /// Fold một chuỗi phím bất kỳ với đúng tuỳ chọn của từ đang gõ (kiểu gõ, kiểu dấu,
    /// đặt dấu tự do, Caps Lock, Quick Telex).
    fn fold_keys(&self, raw: &[char]) -> Vec<char> {
        let mut display = method::fold_caps(
            raw,
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
        // Key-up: xử lý nhả riêng phím Ctrl (Ctrl tap word-lock: `tunx`, Ctrl -> `tũn`,
        // khoá từ tiếng Việt hợp lệ hiện tại và tách từ mới; nếu từ đang active là âm tiết
        // KHÔNG hợp lệ trong tiếng Việt như `sort` -> `sỏt`, khôi phục ngay về `sort`).
        if !k.key_down {
            if k.is_ctrl_key() && self.ctrl_tap_armed {
                self.ctrl_tap_armed = false;
                if !self.word.is_empty() {
                    if self.word.active
                        && !validate::is_valid_word(&self.word.display)
                        && self.restorable()
                    {
                        let raw_str: String = self.word.raw.iter().collect();
                        self.style_note_word(&raw_str);
                        return self.restore_raw(None);
                    }
                    if self.style_enabled_for_learning() {
                        let current = if self.word.active {
                            self.word.display.iter().collect::<String>()
                        } else {
                            self.word.passed.iter().collect::<String>()
                        };
                        if !current.is_empty() {
                            self.style_note_word(&current);
                        }
                    }
                    self.word.clear();
                    return Outcome {
                        action: Action::Pass,
                        preedit: Vec::new(),
                        flags: FLAG_WORD_END,
                    };
                }
            }
            return Outcome {
                action: Action::Pass,
                preedit: self.preedit(),
                flags: 0,
            };
        }
        self.em_dash_tentative = std::mem::take(&mut self.em_dash_next);
        // P6 Text Prediction: Bất kỳ phím nào khác Tab hoặc Ctrl+Space đều chốt vòng duyệt;
        // Backspace/Esc ngay sau khi chèn là từ chối ứng viên — không học.
        if k.vk != keymap::vk::TAB && !k.is_ctrl_space() {
            self.settle_predict(!matches!(k.vk, keymap::vk::BACK | keymap::vk::ESCAPE));
        }
        // Chống loop (P0-3 §6): phím adapter tự bơm → PASS tuyệt đối.
        if k.is_injected {
            return Outcome::pass();
        }
        // Chord hệ thống / modifier đơn (S9, bug B6) — không bao giờ nuốt.
        if k.is_modifier() {
            self.ctrl_tap_armed =
                k.is_ctrl_key() && (k.mods & !(keymap::MOD_CTRL | keymap::MOD_CAPS)) == 0;
            return Outcome::pass();
        }
        self.ctrl_tap_armed = false;
        if k.is_chord() && !k.is_ctrl_space() {
            // Ctrl+V/Ctrl+Z/Ctrl+←/Ctrl+Backspace/Alt+Tab… đổi text hoặc vị trí con trỏ mà
            // engine không thấy: bỏ quyền sở hữu từ đang gõ (như phím điều hướng — B7) để
            // phím kế tiếp không REPLACE `owned` ký tự ở chỗ khác, và quên đuôi text để gõ
            // tắt kế tiếp không xoá nhầm (`vn` Ctrl+V `abc` Tab ≠ bung `vn`).
            self.word.clear();
            self.record(Tail::Clear);
            return Outcome::pass();
        }
        // P10 Dream RSI: đếm phím gõ chữ thật (đã loại key-up/injected/
        // modifier/chord/điều hướng — printable=None). G8: chỉ counter, không nội dung.
        if k.printable().is_some() && !k.is_ctrl_space() {
            self.dream_rsi.keys_typed = self.dream_rsi.keys_typed.saturating_add(1);
        }

        let strategy = self.strategy();
        let vi_on = self.opts.enabled && self.ctx.enabled;
        // Ô mật khẩu, terminal hoặc app yêu cầu passthrough tuyệt đối (Game...):
        // không chạy dự đoán/duyệt vòng ứng viên. Ngược lại, khi chỉ tắt tiếng Việt
        // (Mode EN: `vi_on == false`), bộ dự đoán tiếng Anh vẫn hoạt động đầy đủ!
        let passthrough_only = self.prediction_blocked();

        // P6: đang duyệt vòng ứng viên → Tab/Ctrl+Space kế tiếp đổi ứng viên,
        // ƯU TIÊN hơn gõ tắt: đuôi `recent` lúc này là ứng viên vừa chèn, không
        // phải trigger người dùng gõ (R2-120).
        if self.last_predict.is_some()
            && (k.vk == keymap::vk::TAB || k.is_ctrl_space())
            && k.mods & keymap::MOD_SHIFT == 0
            && !passthrough_only
        {
            if let Some(out) = self.try_cycle_predict() {
                return out;
            }
            // Một ứng viên: Tab kế tiếp không có gì để đổi — ứng viên đó coi như đã nhận.
            self.settle_predict(true);
        }

        // Gõ tắt / emoji (P0-3 §1.1) — chạy cả khi VN tắt nếu `allow_macro_when_vi_off`
        // (EVKey spec #5); `secure` tuyệt đối không (S3).
        if let Some(out) = self.try_macro(k, strategy, vi_on) {
            return out;
        }

        if strategy == Strategy::Passthrough {
            if !self.word.is_empty() {
                self.word.clear();
            }
            if !passthrough_only
                && matches!(
                    k.vk,
                    keymap::vk::RETURN | keymap::vk::SPACE | keymap::vk::TAB
                )
            {
                let is_ctrl_space = k.is_ctrl_space();
                let is_tab_or_ctrl_space = k.vk == keymap::vk::TAB || is_ctrl_space;
                let c = k.printable().unwrap_or(' ');
                let space_complete =
                    k.vk == keymap::vk::SPACE && !is_ctrl_space && self.opts.suggest_space_complete;
                if is_tab_or_ctrl_space || space_complete {
                    let append: Option<char> = space_complete.then_some(c);
                    if is_tab_or_ctrl_space {
                        if let Some(out) = self.try_predict_next_phrase(k, vi_on, 2) {
                            return out;
                        }
                    }
                    if let Some(out) = self.try_english_complete(k, vi_on, append) {
                        return out;
                    }
                    if is_tab_or_ctrl_space {
                        if let Some(out) = self.try_predict_next_phrase(k, vi_on, 1) {
                            return out;
                        }
                    }
                }
            }
            // Ctrl+Space là phím lệnh (không chèn chữ chắc chắn): như chord —
            // quên đuôi text thay vì ghi một dấu cách ảo vào `recent`/`sentence`.
            if k.is_ctrl_space() {
                self.record(Tail::Clear);
                return Outcome::pass();
            }
            // Ranh giới từ (Space, dấu câu, Enter, phím điều hướng…) báo WORD_END: adapter
            // giữ từ trong composition để bung gõ tắt khi VN tắt sẽ commit từ ở đây thay vì
            // kéo gạch chân sang từ kế tiếp.
            // Tiếng Việt tắt thì không có phím dấu: chỉ chữ cái mới thuộc từ. Ký tự điều
            // khiển (layout macOS trả `\x08`, `\x1C`… cho Backspace/mũi tên) không phải
            // ranh giới in được — học ở đó là học tiền tố dở dang.
            let ends_word = |c: char| !c.is_alphabetic();
            let boundary = k.printable().is_none_or(ends_word);
            // Dấu nháy giữa từ (`don't`) không kết thúc từ để học — học `don` là học rác.
            let learn_boundary = k.printable().is_some_and(|c| {
                ends_word(c)
                    && !matches!(c, '\'' | '’')
                    && (!c.is_control() || matches!(c, '\n' | '\r' | '\t'))
            });
            let just_accepted = std::mem::take(&mut self.accepted_just_now);
            // P7 / Dream RSI: Học từ/cụm tiếng Anh người dùng gõ xong ở Mode EN (khi ranh giới từ in được đến).
            if learn_boundary && !just_accepted && self.style_enabled_for_learning() {
                let mut end = self.sentence.len();
                while end > 0 && self.sentence[end - 1].is_ascii_alphabetic() {
                    end -= 1;
                }
                if self.sentence.len() - end >= 2 {
                    let current: String = self.sentence[end..].iter().collect();
                    self.style_note_word(&current);
                    self.style_note_bigram(end, &current);
                }
            }
            // Vẫn ghi nhận text đi thẳng: macro `always` có thể chạy khi VN tắt
            // (`allow_macro_when_vi_off`) — EVKey spec #5.
            self.note_pass(k);
            return Outcome {
                flags: if boundary { FLAG_WORD_END } else { 0 },
                ..Outcome::pass()
            };
        }

        match k.vk {
            keymap::vk::BACK => return self.on_backspace(),
            keymap::vk::ESCAPE => return self.on_escape(),
            keymap::vk::RETURN | keymap::vk::SPACE | keymap::vk::TAB => {
                let is_ctrl_space = k.is_ctrl_space();
                let is_tab_or_ctrl_space = k.vk == keymap::vk::TAB || is_ctrl_space;
                let c = k.printable().unwrap_or(' ');
                // (Duyệt vòng ứng viên P6 đã chạy ở đầu `key()`, trước gõ tắt.)
                // Vòng 24 (`suggest.trigger = "tab_and_space"`): Space cũng
                // hoàn tất từ — nhưng **nghiêm ngặt hơn Tab** (Space là phím
                // ngầm định, người dùng không yêu cầu gì): chỉ hoàn tất khi từ
                // đang gõ CHƯA phải từ đúng hoàn chỉnh, và kèm chính dấu cách.
                let space_complete =
                    k.vk == keymap::vk::SPACE && !is_ctrl_space && self.opts.suggest_space_complete;
                if is_tab_or_ctrl_space || space_complete {
                    let append: Option<char> = space_complete.then_some(c);
                    // R2-120: ngữ cảnh NHIỀU TỪ vừa gõ xong ("phát triển",
                    // "hợp đồng", "kính gửi") → cụm nối tiếp đi TRƯỚC hoàn tất
                    // tiền tố của riêng âm tiết cuối (trước đây "phát triển"+Tab
                    // ra "phát triển khai", "hợp đồng"+Tab ra "hợp đồng ý").
                    if is_tab_or_ctrl_space {
                        if let Some(out) = self.try_predict_next_phrase(k, vi_on, 2) {
                            return out;
                        }
                    }
                    if let Some(out) = self.try_vn_complete(k, vi_on, append) {
                        return out;
                    }
                    if let Some(out) = self.try_english_complete(k, vi_on, append) {
                        return out;
                    }
                    // P6 Text Prediction: cụm nối tiếp theo khoá một từ ("xin" → "chào").
                    if is_tab_or_ctrl_space {
                        if let Some(out) = self.try_predict_next_phrase(k, vi_on, 1) {
                            return out;
                        }
                    }
                }
                if is_ctrl_space {
                    // Nếu từ đang active là từ tiếng Anh bị Telex biến đổi sai cấu trúc Việt
                    // (ví dụ `sort` -> `sỏt`) mà không khớp dự đoán nào,
                    // phải hoàn nguyên về từ tiếng Anh gốc trước khi nhả quyền sở hữu từ!
                    if self.word.active && self.restorable() {
                        let raw_str: String = self.word.raw.iter().collect();
                        self.style_note_word(&raw_str);
                        return self.restore_raw(None);
                    }
                    // Ctrl+Space không khớp dự đoán: KHÔNG chèn khoảng trắng; adapter
                    // kích hoạt Trợ lý ngữ pháp (Text Correct) hoặc để app nhận phím.
                    // Bỏ quyền sở hữu từ (như chord — app có thể đổi text/caret với
                    // phím này) nhưng GIỮ buffer câu cho lượt kiểm tra on-demand;
                    // WORD_END = adapter chốt composition/preedit trước (R2-121).
                    self.word.clear();
                    return Outcome {
                        flags: FLAG_WORD_END,
                        ..Outcome::pass()
                    };
                }
                self.settle_predict(true);
                return self.on_boundary(c, strategy);
            }
            _ => {}
        }

        // Ký tự điều khiển KHÔNG phải chữ hay ranh giới in được: layout macOS trả
        // `\x1C`–`\x1F` cho mũi tên, `\x01`/`\x04` Home/End, `\x7F` Delete xuôi,
        // `\x03` Enter bàn phím số… Coi là ranh giới thì ở Preedit engine trả
        // COMMIT{"\x1C"} — adapter chèn ký tự điều khiển vào văn bản và nuốt phím.
        // Chỉ `\n` `\r` `\t` (Enter/Tab gửi qua `ch`) còn là ranh giới.
        let printable = k
            .printable()
            .filter(|&c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'));
        let Some(c) = printable else {
            // Delete/arrow/F-key…: app có thể đã đổi selection/cursor mà engine
            // không quan sát được. Bỏ composition ownership trước khi PASS; nếu
            // giữ `word`, key kế tiếp sẽ REPLACE một suffix ở vị trí cũ (B7).
            self.word.clear();
            self.note_pass(k);
            return Outcome::pass();
        };

        if c == ']' && !self.word.active && !self.open_bracket_in_line() {
            if let Some(out) = self.restore_bracket_index() {
                return out;
            }
        }
        if !method::is_word_char(c, self.opts.method) || self.bracket_is_boundary(c) {
            // Số/punctuation (theo method): đóng từ (nếu có) rồi cho ký tự đi thẳng.
            return self.on_boundary(c, strategy);
        }

        // --- Chữ cái: fold path ---
        // Từ dài bất thường (giữ phím lặp `đẹpppp…` trong chat, chuỗi chữ dài không ranh
        // giới): kết quả sẽ không vừa `ime_result_v1` (MAX_TEXT). FFI mà cắt `insert` thì
        // `owned` lệch document và phím kế tiếp xoá lẹm sang chữ phía trước → đóng từ
        // (giữ nguyên trong document), phím này đi vào như ký tự thường.
        if self.word.raw.len() >= MAX_WORD_KEYS {
            return self.close_word(c, strategy);
        }

        // OP-002 (P5): bảo vệ tên hàm công thức bảng tính — từ bắt đầu NGAY SAU
        // dấu `=` ở đầu ô/đầu dòng (`=sum`, `=if`, `=vlookup`) gõ thẳng raw,
        // không fold (Telex `=sum(` → `=ũm(`). KHÔNG chặn sau chữ số (R2-2:
        // VNI dùng `1..9` làm phím dấu — `d9u7o7ng2` → `đường`). Dấu `(` là
        // ranh giới đóng từ nên bên trong ngoặc vẫn gõ tiếng Việt bình thường.
        if self.is_excel_formula_name() {
            self.word.raw.push(c);
            self.word.passed.push(c);
            self.record(Tail::Replace {
                delete: 0,
                glyph: &[c],
                unicode: &[c],
            });
            return Outcome::pass();
        }

        // Auto-capitalize (P0-3 §1.1): chỉ áp cho **chữ đầu từ** ở đầu câu — sau `. ! ?` +
        // khoảng trắng hoặc Enter (R2-59: `google.com`, `3.5 kg` không phải đầu câu).
        let mut caps_fired = false;
        let c = if self.opts.auto_capitalize
            && self.word.is_empty()
            && post::caps::sentence_start(&self.recent)
        {
            caps_fired = true;
            self.word.auto_capped = true; // OP-006: camel không fire trên từ này
            post::caps::capitalize(c)
        } else {
            c
        };

        if k.mods & keymap::MOD_CAPS != 0 {
            self.word.caps_lock = true;
        }

        // OP-006 (P5): identifier kỹ thuật — chữ HOA giữa từ ("use" + 'S' của
        // `useState`, "on" + 'P' của `onPress`, "Mc" + 'D' của `McDonald`).
        // Nhận diện MỘT lần là áp cho trọn từ (word.identifier): phần còn lại
        // ("tate", "ress") gõ thẳng raw — không identifier flag thì "ress"
        // thành "rés" ('s' sau nguyên âm là dấu sắc). Từ đã fold thì hoàn
        // nguyên raw trước ("úe" → "use"). auto_capped chặn từ engine vừa tự
        // viết hoa ("bAn" sau dấu chấm — R1-4).
        // Chỉ CẮM CỜ ở đây: việc hoàn nguyên raw làm nhánh identifier NGAY
        // DƯỚI bằng đúng MỘT `Replace` trọn từ trong cùng phím này. Record kép
        // trong một phím (Replace camel + Replace identifier) khiến adapter chỉ
        // nhận action sau với `delete_count` tính theo raw trong khi document
        // còn giữ glyph precomposed trước fold — lệch sổ ("useState": doc "úe"
        // 2 glyph, delete 3 theo raw 3 phím) đã lọt qua test bisect OP-006.
        if !self.word.identifier && !self.word.auto_capped && self.is_camel_case_transition(c) {
            self.word.identifier = true;
        }

        self.word.raw.push(c);

        // OP-006: identifier đã nhận diện — gõ thẳng raw (không fold), đồng bộ
        // document bằng Replace trọn từ (cơ chế như fold path nhưng
        // display = raw, nên auto-restore ở ranh giới là no-op).
        if self.word.identifier {
            let raw = self.word.raw.clone();
            if self.word.active {
                let out = self.emit(&raw);
                if out.len() > MAX_TEXT {
                    self.word.raw.pop();
                    return self.close_word(c, strategy);
                }
                let delete = self.word.owned as u16;
                self.record(Tail::Replace {
                    delete: self.word.owned,
                    glyph: &out,
                    unicode: &raw,
                });
                self.word.display = raw.clone();
                self.word.owned = out.len();
                return Outcome {
                    action: Action::Replace {
                        delete_count: delete,
                        insert: out.clone(),
                    },
                    preedit: out,
                    flags: FLAG_CONSUMED,
                };
            }
            // Chưa active: các ký tự trước đã PASS — chữ này pass tiếp.
            self.word.passed.push(c);
            self.word.display = raw;
            self.record(Tail::Replace {
                delete: 0,
                glyph: &[c],
                unicode: &[c],
            });
            return Outcome::pass();
        }

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
                self.record(Tail::Replace {
                    delete: 0,
                    glyph: &out,
                    unicode: &display,
                });
                return Outcome {
                    action: Action::Replace {
                        delete_count: 0,
                        insert: out,
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED,
                };
            }
            self.record(Tail::Replace {
                delete: 0,
                glyph: &[c],
                unicode: &[c],
            });
            return Outcome::pass();
        }

        let out = self.emit(&display);
        if out.len() > MAX_TEXT {
            // Bảng mã 2 ký tự/chữ (Unicode tổ hợp, VNI Windows) làm từ ngắn hơn
            // MAX_WORD_KEYS phím vẫn tràn — cùng cách xử lý như trên.
            self.word.raw.pop();
            return self.close_word(c, strategy);
        }
        let delete = if self.word.active {
            self.word.owned as u16
        } else {
            self.word.pending_delete() // activate: xóa những ký tự đã PASS trong từ
        };
        // `display` (Unicode) là nguồn cho buffer câu; `out` (glyph) là thứ app nhận.
        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &out,
            unicode: &display,
        });
        self.word.active = true;
        self.word.passed.clear();
        self.word.display = display;
        self.word.owned = out.len();
        Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: out.clone(),
            },
            preedit: out,
            flags: FLAG_CONSUMED,
        }
    }

    /// OP-002 (P5): từ hiện tại có phải **tên hàm công thức bảng tính** dính
    /// liền sau dấu `=` ở đầu ô/đầu dòng không (`=sum`, `=if`, `=or`,
    /// `=vlookup`). Đầu ô = `=` ở đầu buffer hoặc ngay sau `\n`/`\t`; dấu `=`
    /// sau chữ/số/gạch dưới (`x==1`, `a=b`) KHÔNG phải công thức.
    fn is_excel_formula_name(&self) -> bool {
        let word_glyphs = if self.word.active {
            self.word.owned
        } else {
            self.word.passed.len()
        };
        if self.recent.len() <= word_glyphs {
            return false;
        }
        let eq_idx = self.recent.len() - word_glyphs - 1;
        if self.recent[eq_idx] != '=' {
            return false;
        }
        eq_idx == 0 || matches!(self.recent[eq_idx - 1], '\n' | '\t')
    }

    /// OP-006 (P5): ký tự `c` vừa gõ có tạo **ranh giới camelCase** không —
    /// chữ in hoa NGAY SAU chữ thường trong cùng từ (`use` + 'S', `on` + 'P',
    /// `Mc` + 'D'). Câu tiếng Việt viết hoa ĐẦU từ không bao giờ dính (raw
    /// rỗng ở chữ đầu); acronym toàn hoa (`USA`, `NDA`) không dính ('U' hoa
    /// trước 'S' hoa). `_`/`.`/`-` là ranh giới từ riêng nên không cần soi.
    fn is_camel_case_transition(&self, c: char) -> bool {
        if self.word.raw.is_empty() {
            return false;
        }
        let last = *self.word.raw.last().unwrap();
        last.is_lowercase() && c.is_uppercase()
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
            && !(self.opts.allow_macro_when_vi_off && !self.secure())
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
        // OP-001 (P5): thích ứng hoa/thường theo cách gõ trigger thực tế —
        // display match gõ `Hd`/`HD`; raw match (từ đã fold, "hd"→"hợp đồng"
        // trong document) dùng `self.word.raw` làm "cách gõ". Phân biệt theo
        // NGUỒN match (display/raw), không theo độ dài — `owned` của raw match
        // có thể ≤ recent.len() và slice recent lúc đó là text đã fold, không
        // phải trigger người dùng gõ.
        let (len, expanded) = if let Some((len, text)) = match_display {
            let start = self.recent.len().saturating_sub(len);
            let expanded = post::r#macro::apply_macro_case(&self.recent[start..], text);
            (len, expanded)
        } else if let Some((owned, text)) = match_raw {
            let expanded = post::r#macro::apply_macro_case(&self.word.raw, text);
            (owned, expanded)
        } else {
            // OP-003 (P5) + OP-010 (vòng 56): không khớp macro/emoji → thử
            // cú pháp ngày/tháng (`=ngay`/`=thang`/`=nam`) rồi số tiền
            // `=<số>(d|vnd|bc)`. Vượt MAX_TEXT khi encode → fail-open (R1-2:
            // KHÔNG cắt cụt câu số tiền như đường macro).
            let (sh_len, sh_str) =
                if let Some((dl, dt)) = post::number_vn::find_date_shorthand(&self.recent) {
                    (dl, dt)
                } else {
                    post::number_vn::find_number_shorthand(&self.recent)?
                };
            let sh_chars: Vec<char> = sh_str.chars().collect();
            if self.emit(&sh_chars).len() > MAX_TEXT {
                return None;
            }
            (sh_len, sh_chars)
        };
        // ⚠️ `MAX_TEXT` cat SAU encode (F65): phan app nhan vao document bi cat o day,
        // nen phan Unicode cua buffer cau phai cat theo cung quy tac (chi lay chu vua du).
        let insert: Vec<char> = self.emit(&expanded).into_iter().take(MAX_TEXT).collect();
        let unicode = self.unicode_prefix(&expanded, insert.len());
        self.word.clear();
        self.record(Tail::Replace {
            delete: len,
            glyph: &insert,
            unicode: &unicode,
        });
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
    /// Thứ tự (stage 7 pipeline): auto-restore EN (B5) → đóng từ. Viết hoa đầu câu đọc lại
    /// đuôi text (`recent`) khi chữ kế tiếp được gõ.
    fn on_boundary(&mut self, c: char, strategy: Strategy) -> Outcome {
        let was_active = self.word.active;
        let will_restore = was_active && (self.restorable() || self.bracket_word_is_code(c));

        // P7 (vòng 54): học từ/cụm từ user GÕ XONG (word-end, USP bật) — khi
        // `will_restore` = true thì học chuỗi EN thật (`self.word.raw`), tuyệt đối
        // không học chuỗi bị Telex biến đổi sai cấu trúc (`self.word.display` như `sỏt`).
        // G8: chỉ từ đơn/cụm ngắn <= 3 từ + count; secure check ở caller key() đã chặn ô mật khẩu
        // (đường này không chạy khi secure — engine không nhận context secure).
        if self.style_enabled_for_learning() {
            let (current, sentence_cur_len) = if will_restore {
                (
                    self.word.raw.iter().collect::<String>(),
                    self.word.display.len(),
                )
            } else if was_active {
                let s = self.word.display.iter().collect::<String>();
                let n = self.word.display.len();
                (s, n)
            } else if !self.word.passed.is_empty() {
                let s = self.word.passed.iter().collect::<String>();
                let n = self.word.passed.len();
                (s, n)
            } else {
                (String::new(), 0)
            };
            if !current.is_empty() {
                self.style_note_word(&current);
                if let Some(before_idx) = self.sentence.len().checked_sub(sentence_cur_len) {
                    self.style_note_bigram(before_idx, &current);
                }
            }
        }

        // Auto-restore EN (bug B5): kết quả fold không phải âm tiết Việt → trả lại chuỗi gõ.
        if will_restore {
            return self.restore_raw(Some(c)); // ranh giới vẫn phải vào document
        }

        self.close_word(c, strategy)
    }

    /// Từ đứng trước ` --` ở đuôi document có chữ Việt (có dấu) không.
    fn word_before_dashes_is_vietnamese(&self) -> bool {
        let before = self.recent.len().saturating_sub(3);
        self.recent[..before]
            .iter()
            .rev()
            .take_while(|c| !c.is_whitespace())
            .any(|c| !c.is_ascii())
    }

    /// Từ đang mở có nên trả về chuỗi gõ không (auto-restore tiếng Anh, bug B5).
    fn restorable(&self) -> bool {
        self.opts.auto_restore_english
            && post::restore_en::should_restore(
                &self.word.raw,
                &self.word.display,
                &self.opts.english_words,
            )
    }

    /// Gạch nối dài từ hai dấu `-` (R2-141). Chỉ hai dạng văn xuôi: `a--b` (dính liền
    /// chữ — đổi ngay, ranh giới kế tiếp không phải chữ thì trả lại) và `Nội -- TP` (cách
    /// hai bên sau chữ Việt — đổi khi có dấu cách thứ hai). Không đụng `--force`, `i--;`,
    /// `<!-- -->`, `|--|`, `=A1--B1`, `-- comment`. `---` luôn giữ nguyên cho Markdown.
    fn em_dash_outcome(&mut self, c: char, tentative: bool) -> Option<Outcome> {
        const EM: char = '\u{2014}';
        // RESTORE: adapter tách Enter/Tab cuối `insert` thành phím thật (chat vẫn gửi được).
        let replace = |delete: usize, text: &[char], engine: &mut Engine| {
            engine.record(Tail::Replace {
                delete,
                glyph: text,
                unicode: text,
            });
            Outcome {
                action: Action::Restore {
                    delete_count: delete as u16,
                    insert: text.to_vec(),
                },
                preedit: Vec::new(),
                flags: FLAG_CONSUMED | FLAG_WORD_END,
            }
        };
        let tail = self.recent.as_slice();
        if tentative && tail.last() == Some(&EM) {
            let text: Vec<char> = if c == '-' {
                vec!['-', '-', '-']
            } else {
                vec!['-', '-', c]
            };
            return Some(replace(1, &text, self)); // `a---` hoặc `i--;`
        }
        if !self.opts.enabled
            || !self.ctx.enabled
            || !matches!(
                self.opts.output_charset,
                OutputCharset::UnicodePrecomposed | OutputCharset::UnicodeDecomposed
            )
            || matches!(
                self.ctx.field_role,
                strategy::IME_FIELD_TERMINAL
                    | strategy::IME_FIELD_ADDRESS_BAR
                    | strategy::IME_FIELD_SEARCH
                    | strategy::IME_FIELD_CANDIDATE
            )
        {
            return None;
        }
        let is_letter = |ch: char| ch.is_alphabetic() || ('\u{300}'..='\u{36F}').contains(&ch);
        // `.btn--primary`, `block__elem--mod`: định danh (CSS BEM, biến) — không phải văn xuôi.
        let in_identifier = || {
            let letters = tail[..tail.len().saturating_sub(1)]
                .iter()
                .rev()
                .take_while(|c| is_letter(**c) || c.is_ascii_digit())
                .count();
            let at = tail.len().saturating_sub(1 + letters);
            at > 0 && matches!(tail[at - 1], '.' | '_' | '-' | '#' | '$' | '@' | ':' | '/')
        };
        match (c, tail) {
            ('-', [.., before, '-']) if is_letter(*before) && !in_identifier() => {
                self.em_dash_next = true;
                Some(replace(1, &[EM], self))
            }
            // Dạng cách hai bên chỉ sau chữ Việt: sau ASCII là `x = 1 -- comment`, `git
            // checkout -- file` — code/lệnh, đổi là hỏng.
            (' ', [.., ' ', '-', '-']) if self.word_before_dashes_is_vietnamese() => {
                Some(replace(2, &[EM, ' '], self))
            }
            _ => None,
        }
    }

    /// Đóng từ đang gõ **giữ nguyên** kết quả trong document, rồi cho `c` vào sau nó
    /// (không auto-restore, không đổi cờ viết hoa — phần đó thuộc `on_boundary`).
    fn close_word(&mut self, c: char, strategy: Strategy) -> Outcome {
        let was_active = self.word.active;
        let was_empty = self.word.is_empty();
        let bracket_word = (was_active && self.word.raw.iter().any(|c| matches!(c, '[' | ']')))
            .then(|| (self.word.raw.clone(), self.word.display.clone()));
        self.word.clear();
        if c.is_whitespace() {
            self.bracket_word = None;
        } else if bracket_word.is_some() {
            self.bracket_word = bracket_word;
        }

        let tentative = std::mem::take(&mut self.em_dash_tentative);
        if was_empty {
            if let Some(out) = self.em_dash_outcome(c, tentative) {
                return out;
            }
        }

        self.record(Tail::Replace {
            delete: 0,
            glyph: &[c],
            unicode: &[c],
        });
        if !was_active {
            return Outcome {
                action: Action::Pass,
                preedit: Vec::new(),
                flags: FLAG_WORD_END,
            };
        }
        match strategy {
            // Preedit: đóng composition (preedit thành text) rồi chèn `c`.
            Strategy::Preedit => Outcome {
                action: Action::Commit { insert: vec![c] },
                preedit: Vec::new(),
                flags: FLAG_CONSUMED | FLAG_WORD_END,
            },
            // Các strategy khác: từ đã nằm trong document → chỉ cho `c` đi qua.
            _ => Outcome {
                action: Action::Pass,
                preedit: Vec::new(),
                flags: FLAG_WORD_END,
            },
        }
    }

    /// **Tab gợi ý hoàn tất từ tiếng Việt** (`suggest.enabled` — 0.2.33): từ đang gõ
    /// (phần hiển thị đã fold) là tiền tố nghiêm ngặt của một âm tiết tần suất cao
    /// (`post/suggest_vn::complete_vn_word`) → thay toàn bộ từ bằng âm tiết hoàn
    /// chỉnh và đóng từ. Gõ tắt (macro) có ưu tiên cao hơn (đã chạy trước trong
    /// `key()`); Tab không khớp → đi qua cho app.
    /// P8/F3 (vòng 55 — username identity): từ hiện hành đứng NGAY SAU dấu
    /// chấm/@/_/-/slash/colon (như `truong.giap`, `t.ho`, `ngoc.pt`, đường
    /// dẫn `a/b`) thì **không hoàn tất** — người dùng đang gõ username/id/
    /// đường dẫn, Space/Tab phải giữ nguyên ký tự (chuẩn UniKey: Space không
    /// đổi chữ). Trả `true` = có ranh giới id-boundary ngay trước từ.
    fn word_after_id_boundary(&self) -> bool {
        // P8/F3: từ hiện hành chứa punctuation kiểu id (`truong.giap`, `t.ho`,
        // `ngoc.pt`, đường dẫn `a/b`) → KHÔNG hoàn tất. Space path: word đã
        // nhả — từ hiện hành là suffix của sentence sau space cuối;
        // Tab path: word active — kiểm trực tiếp display.
        const ID_PUNCT: [char; 6] = ['.', '@', '_', '-', '/', ':'];
        if self.word.active {
            return self.word.display.iter().any(|c| ID_PUNCT.contains(c));
        }
        let last_space = self.sentence.iter().rposition(|&c| c == ' ');
        let start = last_space.map(|i| i + 1).unwrap_or(0);
        self.sentence[start..].iter().any(|c| ID_PUNCT.contains(c))
    }

    fn try_vn_complete(
        &mut self,
        k: &KeyEvent,
        vi_on: bool,
        append: Option<char>,
    ) -> Option<Outcome> {
        if !vi_on || !self.opts.suggest_vn_complete || self.prediction_blocked() {
            return None;
        }
        if k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }
        if !self.word.active || self.word.display.len() < 2 {
            return None;
        }
        // P8/F3: username/id sau dấu chấm — KHÔNG hoàn tất.
        if self.word_after_id_boundary() {
            return None;
        }
        let typed: String = self.word.display.iter().collect::<String>().to_lowercase();
        // Nếu từ đang active là một từ tiếng Anh bị Telex fold nhầm (`should_restore` = true,
        // ví dụ `"see"` → `"sê"`, `"is"` → `"í"`, `"yes"` → `"yé"`, `"file"` → `"fỉle"`, `"user"` → `"úer"`)
        // và không phải từ cá nhân người dùng đã học trong `style_words`, nhường cho
        // `try_english_complete` xử lý thay vì hoàn tất nhầm thành từ tiếng Việt!
        if self.restorable()
            && !self
                .opts
                .style_words
                .iter()
                .any(|(w, _)| w.starts_with(&typed) && w != &typed)
        {
            return None;
        }
        // Space (append) nghiêm ngặt hơn Tab: từ ĐÃ là âm tiết hợp lệ
        // ("không", "đi"…) không được viết lại — người dùng gõ đúng rồi.
        if append.is_some() && validate::is_valid_word(&self.word.display) {
            return None;
        }
        // P7 (vòng 54): từ cá nhân (style_profile) tham gia ứng viên —
        // list rỗng (mặc định) → hành vi cũ byte-for-byte.
        let candidates = post::predict::predict_prefix_candidates_with_personal(
            &typed,
            true,
            5,
            &self.opts.style_words,
        );
        let candidates = if !candidates.is_empty() {
            candidates
        } else {
            vec![post::suggest_vn::complete_vn_word(&typed)?]
        };
        // Dữ liệu viết kiểu dấu cũ → chèn theo kiểu người dùng chọn (R2-120).
        let all_candidates: Vec<String> = candidates
            .iter()
            .map(|c| post::predict::restyle(c, self.opts.diacritic_style))
            .collect();
        let completion = all_candidates[0].clone();
        let raw_sample: String = self.word.raw.iter().collect();
        let adapted = post::predict::adapt_capitalization(&raw_sample, &completion);
        let mut insert: Vec<char> = adapted.chars().collect();
        if let Some(c) = append {
            insert.push(c); // Space: hoàn tất + chính dấu cách (như restore-EN)
        }
        let out = self.emit(&insert);
        if out.len() > MAX_TEXT {
            return None;
        }
        // P7 (vòng 54, Hook 2): học prediction-accept — ứng viên user bấm
        // Tab/Space nhận = tín hiệu "tôi hay dùng từ này" (đếm trên bản
        // lowercase chuẩn của completion, G8: chỉ từ + count). Tab chờ người dùng
        // chốt (`settle_predict`); Space đã chốt ngay trong phím này.
        if append.is_some() {
            self.note_accepted(&completion, &raw_sample, false);
        }
        // P10 — Dream RSI: accept tiết kiệm (completion − typed) ký tự.
        let saved = completion
            .chars()
            .count()
            .saturating_sub(typed.chars().count()) as u64;
        self.dream_rsi.accepts = self.dream_rsi.accepts.saturating_add(1);
        self.dream_rsi.chars_saved = self.dream_rsi.chars_saved.saturating_add(saved);
        let delete = self.word.owned as u16;
        self.word.clear();

        if append.is_none() {
            self.last_predict = Some(PredictCycle {
                inserted_glyphs: out.len(),
                candidates: all_candidates,
                current_index: 0,
                sample: raw_sample,
                had_leading_space: false,
                continuation: false,
                rsi_accepts: 1,
                rsi_saved: saved,
                peak_glyphs: out.len(),
            });
        } else {
            self.last_predict = None;
        }

        // Document nhận bản đã encode (`out`), buffer câu nhận bản Unicode (`insert`).
        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &out,
            unicode: &insert,
        });
        Some(Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: out,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// P6 Text Prediction: Duyệt vòng các ứng viên thay thế khi nhấn Tab liên tiếp.
    fn try_cycle_predict(&mut self) -> Option<Outcome> {
        let (cand, sample, had_leading_space, continuation, next_idx) = {
            let cycle = self.last_predict.as_ref()?;
            if cycle.candidates.len() <= 1 {
                return None;
            }
            let next_idx = (cycle.current_index + 1) % cycle.candidates.len();
            (
                cycle.candidates[next_idx].clone(),
                cycle.sample.clone(),
                cycle.had_leading_space,
                cycle.continuation,
                next_idx,
            )
        };
        let adapted = if continuation {
            post::predict::adapt_continuation_case(&sample, &cand)
        } else {
            post::predict::adapt_capitalization(&sample, &cand)
        };
        let mut insert_chars: Vec<char> = Vec::new();
        if had_leading_space {
            insert_chars.push(' ');
        }
        insert_chars.extend(adapted.chars());

        let out = self.emit(&insert_chars);
        if out.len() > MAX_TEXT {
            return None;
        }

        let cycle = self.last_predict.as_mut()?;
        let delete = cycle.inserted_glyphs as u16;
        let seen_glyphs = cycle.peak_glyphs.max(cycle.inserted_glyphs) as u64;
        cycle.peak_glyphs = cycle.peak_glyphs.max(out.len());
        cycle.inserted_glyphs = out.len();
        cycle.current_index = next_idx;
        let gained = (cand.chars().count() as u64).saturating_sub(seen_glyphs);
        cycle.rsi_saved += gained;
        // P10 — Dream RSI: cycle sang candidate dài hơn = tiết kiệm thêm
        // (cand mới − phần đã chèn của candidate cũ).
        // Duyệt vòng không phải lượt nhận mới: `accepts` đếm một lần cho cả vòng.

        self.dream_rsi.chars_saved = self.dream_rsi.chars_saved.saturating_add(gained);

        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &out,
            unicode: &insert_chars,
        });

        // WORD_END: mỗi ứng viên là text đã chốt (adapter commit ngay, như lần
        // hoàn tất đầu) — không để composition mở chứa cả cụm ứng viên.
        Some(Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: out,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// P6 Text Prediction: Dự đoán cụm từ tiếp theo theo ngữ cảnh (Next-Word / Phrase Prediction).
    ///
    /// `min_key_words`: khoá ngữ cảnh khớp tốt nhất phải có ít nhất ngần ấy từ
    /// (2 = lượt chạy TRƯỚC hoàn tất tiền tố, 1 = lượt dự phòng sau cùng).
    ///
    /// Chạy được cả khi từ cuối **vừa gõ xong, chưa có dấu cách** (`Kính gửi`+Tab,
    /// `Thank you`+Tab — R2-120): từ đang mở được chốt nguyên trạng trong cùng một
    /// `Replace` (xoá đúng phần engine đã đưa vào document rồi chèn lại từ + " " +
    /// cụm) — cùng hình dạng action với Tab hoàn tất từ nên mọi adapter/strategy
    /// áp được như nhau.
    fn try_predict_next_phrase(
        &mut self,
        k: &KeyEvent,
        vi_on: bool,
        min_key_words: usize,
    ) -> Option<Outcome> {
        if self.prediction_blocked() || !self.opts.suggest_vn_complete {
            return None;
        }
        if k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }

        let mut sentence_tail = self.sentence_tail(64);
        if sentence_tail.trim().is_empty() {
            return None;
        }

        // Ở Mode VI (`vi_on == true`), khi người dùng đang gõ câu tiếng Anh mà từ cuối
        // chưa bấm Space bị Telex fold nhầm (ví dụ `"looking forward"` → `"looking fỏwad"`,
        // `"in order"` → `"in ỏder"`, `"feel free"` → `"feel frê"`, `"i would"` → `"i ưould"`),
        // khôi phục từ cuối về `raw` để khớp chính xác bảng mẫu câu tiếng Anh `en_phrases.tsv`
        // và hoàn nguyên luôn từ đó trong cùng một `Action::Replace`!
        let restore_active_en = vi_on && self.word.active && self.restorable();
        if restore_active_en {
            let disp_str: String = self.word.display.iter().collect();
            let raw_str: String = self.word.raw.iter().collect();
            if let Some(prefix) = sentence_tail.strip_suffix(&disp_str) {
                sentence_tail = format!("{prefix}{raw_str}");
            }
        }

        let matches = post::predict::predict_next_phrase_matches_with_personal(
            &sentence_tail,
            5,
            &self.opts.style_words,
        );
        if matches.first().is_none_or(|m| m.key_words < min_key_words) {
            return None;
        }
        let candidates: Vec<String> = matches
            .iter()
            .map(|m| post::predict::restyle(&m.prediction, self.opts.diacritic_style))
            .collect();

        let last_word = sentence_tail.split_whitespace().last().unwrap_or("");
        let adapted = post::predict::adapt_continuation_case(last_word, &candidates[0]);

        let had_leading_space = !sentence_tail.ends_with(char::is_whitespace);
        let mut phrase: Vec<char> = Vec::new();
        if had_leading_space {
            phrase.push(' ');
        }
        phrase.extend(adapted.chars());
        let phrase_glyphs = self.emit(&phrase).len();

        // Từ đang mở: chốt nguyên trạng (active → phần hiển thị hoặc raw nếu là từ EN bị fold;
        // chưa biến đổi → ký tự đã PASS) rồi nối cụm vào sau, trong cùng một Replace.
        let (delete, insert) = if self.word.is_empty() {
            (0usize, phrase)
        } else if self.word.active {
            let mut text = if restore_active_en {
                self.word.raw.clone()
            } else {
                self.word.display.clone()
            };
            text.extend(phrase);
            (self.word.owned, text)
        } else {
            let mut text = self.word.passed.clone();
            text.extend(phrase);
            (usize::from(self.word.pending_delete()), text)
        };
        let out = self.emit(&insert);
        if out.len() > MAX_TEXT {
            return None;
        }
        self.word.clear();
        // P10 — Dream RSI: nhận cụm next-phrase (Space/dấu sau từ) = tiết kiệm
        // nguyên phần glyph engine chèn (phrase, gồm cả space engine thêm).
        self.dream_rsi.accepts = self.dream_rsi.accepts.saturating_add(1);
        self.dream_rsi.chars_saved = self
            .dream_rsi
            .chars_saved
            .saturating_add(phrase_glyphs as u64);

        // P7: học khi người dùng chốt (`settle_predict`).
        self.last_predict = Some(PredictCycle {
            inserted_glyphs: phrase_glyphs,
            candidates,
            current_index: 0,
            sample: last_word.to_string(),
            had_leading_space,
            continuation: true,
            rsi_accepts: 1,
            rsi_saved: phrase_glyphs as u64,
            peak_glyphs: phrase_glyphs,
        });

        self.record(Tail::Replace {
            delete,
            glyph: &out,
            unicode: &insert,
        });

        Some(Outcome {
            action: Action::Replace {
                delete_count: delete as u16,
                insert: out,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// **Tab gợi ý hoàn tất từ tiếng Anh** (2026-10-02): từ đang gõ đã bị
    /// transform (word active), chuỗi gõ là tiền tố nghiêm ngặt của một từ EN
    /// thông dụng (`post/restore_en::complete_word`) và fold không đụng âm
    /// tiết Việt thông dụng → thay toàn bộ từ bằng từ hoàn chỉnh và đóng từ.
    /// Macro có ưu tiên cao hơn (đã chạy trước trong `key()`); `Shift`+Tab =
    /// phím hệ thống (S9) — không đụng.
    fn try_english_complete(
        &mut self,
        k: &KeyEvent,
        vi_on: bool,
        append: Option<char>,
    ) -> Option<Outcome> {
        if self.prediction_blocked() {
            return None;
        }
        if vi_on && !self.opts.auto_restore_english {
            return None;
        }
        // Mode EN: chỉ Tab/Ctrl+Space khi dự đoán bật. Space không bao giờ viết lại chữ —
        // người dùng tắt tiếng Việt để gõ đúng chữ mình gõ (`func impl cfg`, tên riêng),
        // và Tab không tự nhận khi dự đoán tắt (thụt lề IDE, chuyển ô Excel — R2-144).
        if !vi_on && (!self.opts.suggest_vn_complete || append.is_some()) {
            return None;
        }
        if k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }
        // P8/F3 (vòng 55): username/id sau dấu chấm — KHÔNG hoàn tất EN
        // (`t.ho` + Space phải giữ `t.ho `, không ra `t.horse `).
        if self.word_after_id_boundary() {
            return None;
        }
        let space_mode = append.is_some();
        let prediction_on = self.opts.suggest_vn_complete;

        // Trích xuất `(typed, folded, delete_count, en_context)` cho cả Mode VI và Mode EN:
        let (typed, folded, delete, en_context) = if vi_on {
            // `sort`+Ctrl+Space khi dự đoán tắt: nhường bước khôi phục về từ gốc (không
            // kéo dài thành `sorted`); các từ khác vẫn hoàn tất theo yêu cầu (`ebit` → `EBITDA`).
            if k.is_ctrl_space() && !prediction_on && self.word.active && self.restorable() {
                return None;
            }
            if !self.word.active && !space_mode && !k.is_ctrl_space() && !prediction_on {
                return None;
            }
            if self.word.is_empty() || self.word.raw.len() < 2 {
                return None;
            }
            let typed_s: String = self.word.raw.iter().collect();
            if space_mode
                && (post::restore_en::is_known_en_word(&typed_s.to_lowercase())
                    || (self.word.active && validate::is_valid_word(&self.word.display)))
            {
                return None;
            }
            let folded_s: String = if self.word.active {
                self.word.display.iter().collect()
            } else {
                typed_s.clone()
            };
            let del = if self.word.active {
                self.word.owned as u16
            } else {
                self.word.pending_delete()
            };
            let tail = self.sentence_tail(64);
            let cur_len = if self.word.active {
                self.word.display.len()
            } else {
                self.word.passed.len()
            };
            let prefix_char_count = tail.chars().count().saturating_sub(cur_len);
            let before_word: String = tail.chars().take(prefix_char_count).collect();
            let en_ctx = post::restore_en::has_en_context(&before_word);
            (typed_s, folded_s, del, en_ctx)
        } else {
            // Mode EN (`!vi_on`): các phím chữ đã đi qua `note_pass` vào `self.sentence` / `self.recent`.
            // Tách từ tiếng Anh ASCII ngay sát trước con trỏ.
            let mut end = self.sentence.len();
            while end > 0 && self.sentence[end - 1].is_ascii_alphabetic() {
                end -= 1;
            }
            let word_len = self.sentence.len() - end;
            if word_len < 2 || word_len > self.recent.len() {
                return None;
            }
            if end > 0
                && !matches!(
                    self.sentence[end - 1],
                    ' ' | '\t' | '\n' | '\r' | '(' | '[' | '"' | '\''
                )
            {
                return None;
            }
            let typed_s: String = self.sentence[end..].iter().collect();
            (typed_s.clone(), typed_s, word_len as u16, true)
        };

        let candidates: Vec<String> = if prediction_on {
            post::restore_en::complete_word_candidates(
                &typed.to_lowercase(),
                &folded.to_lowercase(),
                &self.opts.english_words,
                &self.opts.style_words,
                vi_on,
                space_mode,
                en_context,
                5,
            )
        } else {
            let mut en_words: Vec<String> = self
                .opts
                .style_words
                .iter()
                .map(|(w, _)| w.clone())
                .collect();
            en_words.extend(self.opts.english_words.iter().cloned());
            post::restore_en::complete_word(
                &typed.to_lowercase(),
                &folded.to_lowercase(),
                &en_words,
            )
            .into_iter()
            .collect()
        };
        if candidates.is_empty() {
            return None;
        }
        // Chỉ có đúng từ đã gõ: Tab không thêm gì — để ranh giới từ trả từ gốc rồi Tab tới app
        // (chuyển ô, thụt lề) thay vì nuốt phím. Chỉ khi ranh giới chắc chắn trả từ gốc.
        if candidates.len() == 1
            && candidates[0].eq_ignore_ascii_case(&typed)
            && (!vi_on || self.restorable())
        {
            return None;
        }
        let completion = candidates[0].clone();
        let adapted = post::predict::adapt_capitalization(&typed, &completion);
        let mut insert: Vec<char> = adapted.chars().collect();
        if let Some(c) = append {
            insert.push(c); // Space: hoàn tất + dấu cách
        }
        let out = self.emit(&insert);
        if out.len() > MAX_TEXT {
            return None;
        }
        // P7 Hook 2: Tab chờ người dùng chốt (`settle_predict`); Space chốt ngay.
        if append.is_some() {
            self.note_accepted(&completion, &typed, false);
        }
        // P10 — Dream RSI: EN accept tiết kiệm ký tự khi hoàn tất từ dài hơn chuỗi đã gõ.
        let saved = completion
            .chars()
            .count()
            .saturating_sub(typed.chars().count());
        if saved > 0 {
            self.dream_rsi.accepts = self.dream_rsi.accepts.saturating_add(1);
            self.dream_rsi.chars_saved = self.dream_rsi.chars_saved.saturating_add(saved as u64);
        }
        self.word.clear();

        if append.is_none() {
            self.last_predict = Some(PredictCycle {
                inserted_glyphs: out.len(),
                candidates,
                current_index: 0,
                sample: typed,
                had_leading_space: false,
                continuation: false,
                rsi_accepts: u64::from(saved > 0),
                rsi_saved: saved as u64,
                peak_glyphs: out.len(),
            });
        } else {
            self.last_predict = None;
        }

        // Document nhận bản đã encode (`out`), buffer câu nhận bản Unicode (`insert`).
        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &out,
            unicode: &insert,
        });
        Some(Outcome {
            action: Action::Replace {
                delete_count: delete,
                insert: out,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        })
    }

    /// ESC: khôi phục chuỗi đã gõ (bug B5 / PLAN §8 "ESC khôi phục").
    fn on_escape(&mut self) -> Outcome {
        if !self.word.active {
            return Outcome::pass();
        }
        let raw = self.word.raw.clone();
        let delete = self.word.owned as u16;
        self.word.clear();
        // Document nhận đúng chuỗi gõ (không encode) → glyph = Unicode.
        self.record(Tail::Replace {
            delete: delete as usize,
            glyph: &raw,
            unicode: &raw,
        });
        Outcome {
            action: Action::Restore {
                delete_count: delete,
                insert: raw,
            },
            preedit: Vec::new(),
            flags: FLAG_CONSUMED | FLAG_WORD_END,
        }
    }

    /// Backspace: từ đang active → xoá **ký tự cuối đang hiển thị** (như UniKey
    /// `processBackspace`, R2-55); chưa active → bỏ ký tự đã PASS.
    ///
    /// Dấu thanh nằm trên ký tự vừa xoá thì mất theo; còn lại dời về đúng chỗ của phần còn
    /// lại (`tiếng` → `tiến`, `hoán` → `hoá`, kiểu cũ `hóan` → `hóa`). `raw` được dựng lại
    /// thành chuỗi phím gõ ra đúng chữ còn lại (`method::keys_for`) để phím gõ tiếp vẫn biến
    /// đổi đúng. Bản cũ bỏ **phím** cuối rồi fold lại: `ass`+BS → `á` (thêm dấu), `tiếng`+BS →
    /// `tiêng`, xoá `được` (`dduwowcj`) cần 8 lần Backspace.
    fn on_backspace(&mut self) -> Outcome {
        if self.word.active {
            let delete = self.word.owned as u16;
            let mut target = self.word.display.clone();
            target.pop();
            transform::tone::normalize_tone(&mut target, self.opts.diacritic_style);
            if target.is_empty() {
                self.word.clear();
                self.record(Tail::Replace {
                    delete: delete as usize,
                    glyph: &[],
                    unicode: &[],
                });
                return Outcome {
                    action: Action::Replace {
                        delete_count: delete,
                        insert: Vec::new(),
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED,
                };
            }
            let out = self.emit(&target);
            if out.len() > MAX_TEXT {
                // Dời dấu thanh ở bảng mã tổ hợp không làm từ dài thêm, nhưng giữ lưới an
                // toàn: không vừa result → thôi sở hữu từ, app tự xoá 1 ký tự như thường.
                self.word.clear();
                self.record(Tail::Pop);
                return Outcome::pass();
            }
            let keys =
                method::keys_for(&target, self.opts.method, self.opts.diacritic_style, |k| {
                    self.fold_keys(k)
                })
                .filter(|k| k.len() <= MAX_WORD_KEYS);
            let Some(raw) = keys else {
                // Không có chuỗi phím nào gõ ra đúng chữ còn lại (chữ literal hiếm như `uo`
                // sau `uoww`): giữ chữ trong document, thôi sở hữu từ (như bung gõ tắt).
                self.word.clear();
                self.record(Tail::Replace {
                    delete: delete as usize,
                    glyph: &out,
                    unicode: &target,
                });
                return Outcome {
                    action: Action::Replace {
                        delete_count: delete,
                        insert: out,
                    },
                    preedit: Vec::new(),
                    flags: FLAG_CONSUMED | FLAG_WORD_END,
                };
            };
            // `target` (Unicode) là nguồn buffer câu; `out` (glyph) là thứ app nhận.
            self.record(Tail::Replace {
                delete: delete as usize,
                glyph: &out,
                unicode: &target,
            });
            self.word.raw = raw;
            self.word.display = target;
            self.word.owned = out.len();
            return Outcome {
                action: Action::Replace {
                    delete_count: delete,
                    insert: out.clone(),
                },
                preedit: out,
                flags: FLAG_CONSUMED,
            };
        }
        // Chưa activate: bỏ 1 ký tự đã gõ thẳng. Đuôi text `recent` bớt theo nên viết hoa
        // đầu câu tự đúng (`hi.<Backspace>ban` → `hiban`).
        self.word.raw.pop();
        self.word.passed.pop();
        self.word.display.clear();
        self.record(Tail::Pop);
        Outcome::pass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keymap::{MOD_CTRL, MOD_SHIFT};
    use transform::DiacriticStyle;

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
    fn secure_role_without_flag_is_still_secure() {
        // Adapter đặt role `secure` nhưng quên cờ: macro (kể cả `allow_macro_when_vi_off`)
        // không được chạy, phím không được biến đổi.
        let mut opts = macro_opts(MacroTrigger::Tab);
        opts.allow_macro_when_vi_off = true;
        let mut e = Engine::new(opts);
        e.set_context(Context {
            field_role: strategy::IME_FIELD_SECURE,
            caps: strategy::IME_CAP_PREEDIT | strategy::IME_CAP_FIELD_DETECT,
            hint: strategy::IME_STRATEGY_PREEDIT,
            ..Default::default()
        });
        assert_eq!(e.strategy(), Strategy::Passthrough);
        let mut buf = type_buf(&mut e, "cty");
        assert_eq!(press(&mut e, &mut buf, keymap::vk::TAB), Action::Pass);
        assert_eq!(text(&buf), "cty\t");
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

    /// Bấm Backspace `n` lần, trả document sau mỗi lần.
    fn backspaces(e: &mut Engine, buf: &mut Vec<char>, n: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let k = KeyEvent::key_down(keymap::vk::BACK);
                let o = e.key(&k);
                match o.action {
                    // PASS: app tự xoá 1 ký tự.
                    Action::Pass => {
                        buf.pop();
                    }
                    ref a => apply(buf, a, &k),
                }
                text(buf)
            })
            .collect()
    }

    /// R2-55: Backspace xoá **một ký tự đang hiển thị** (UniKey `processBackspace`), không
    /// bỏ phím cuối rồi fold lại.
    #[test]
    fn backspace_deletes_last_visible_char() {
        // Xoá cả từ cần đúng số Backspace bằng số chữ, bất kể đã gõ bao nhiêu phím.
        let mut e = engine();
        let mut buf = type_buf(&mut e, "dduwowcj");
        assert_eq!(text(&buf), "được");
        assert_eq!(backspaces(&mut e, &mut buf, 4), ["đượ", "đư", "đ", ""]);

        let mut e = engine();
        let mut buf = type_buf(&mut e, "nguwowif");
        assert_eq!(
            backspaces(&mut e, &mut buf, 5),
            ["ngườ", "ngư", "ng", "n", ""]
        );

        // Không bao giờ thêm dấu: phím undo không "sống lại".
        for (keys, shown, after) in [("ass", "as", "a"), ("asss", "ass", "as")] {
            let mut e = engine();
            let mut buf = type_buf(&mut e, keys);
            assert_eq!(text(&buf), shown, "{keys}");
            assert_eq!(backspaces(&mut e, &mut buf, 1), [after], "{keys}");
        }

        // Dấu thanh không nằm trên chữ vừa xoá → giữ (dời về đúng chỗ nếu cần);
        // nằm trên chữ vừa xoá → mất theo.
        // Style New ghim tường minh: nhóm này kiểm cơ chế backspace, không phụ
        // thuộc default dấu (0.2.33 đổi mặc định sang dấu cũ).
        for (keys, after) in [
            ("tieengs", "tiến"),
            ("hoafn", "hoà"),
            ("hoaf", "ho"),
            ("chaof", "chà"),
            ("vieetj", "việ"),
        ] {
            let mut e = Engine::new(EngineOptions {
                diacritic_style: DiacriticStyle::New,
                ..Default::default()
            });
            let mut buf = type_buf(&mut e, keys);
            assert_eq!(backspaces(&mut e, &mut buf, 1), [after], "{keys}");
        }
        let mut e = Engine::new(EngineOptions {
            diacritic_style: DiacriticStyle::Old,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "hoafn");
        assert_eq!(text(&buf), "hoàn");
        assert_eq!(backspaces(&mut e, &mut buf, 1), ["hòa"], "kiểu cũ: dời dấu");

        // VNI giống hệt.
        let mut e = Engine::new(EngineOptions {
            method: Method::Vni,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "d9u7o7c5");
        assert_eq!(text(&buf), "được");
        assert_eq!(backspaces(&mut e, &mut buf, 4), ["đượ", "đư", "đ", ""]);
    }

    /// Sau Backspace, phím gõ tiếp vẫn biến đổi trên chữ còn lại.
    #[test]
    fn typing_continues_after_backspace() {
        for (method, keys, fix, want) in [
            (Method::Telex, "dduocj", "c", "được"),
            (Method::Telex, "tieengs", "g", "tiếng"),
            (Method::Telex, "chaof", "o", "chào"),
            (Method::Telex, "ddaatj", "c", "đậc"),
            // Dấu cũ: BS xoá 'a' nhưng huyền còn trên 'o' (`hòa` BS → `hò`) — gõ lại `a` là đủ.
            (Method::Telex, "hoaf", "a", "hòa"),
            // Gõ tiếp `f` khi từ đã có huyền = phím undo (như `ass` → `as`): gỡ dấu + `f` literal.
            (Method::Telex, "hoaf", "af", "hoaf"),
            (Method::Telex, "ass", "s", "á"),
            (Method::Vni, "d9u7o7c5", "c", "được"),
            (Method::Viqr, "ddu+o+c.", "c", "được"),
        ] {
            let mut e = Engine::new(EngineOptions {
                method,
                ..Default::default()
            });
            let mut buf = type_buf(&mut e, keys);
            backspaces(&mut e, &mut buf, 1);
            for c in fix.chars() {
                let k = KeyEvent::char_down(c);
                let o = e.key(&k);
                apply(&mut buf, &o.action, &k);
            }
            assert_eq!(text(&buf), want, "{method:?} {keys}+BS+{fix}");
        }
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
    fn chord_drops_word_ownership() {
        // `tooi` → `tôi` (engine sở hữu 3 ký tự) rồi Ctrl+← (con trỏ nhảy về đầu từ):
        // `s` kế tiếp không được REPLACE 3 ký tự ở vị trí mới.
        let mut e = engine();
        let (buf, _) = type_keys(&mut e, "tooi");
        assert_eq!(buf, "tôi");
        let ctrl_left = KeyEvent {
            vk: keymap::vk::LEFT,
            mods: MOD_CTRL,
            key_down: true,
            ..Default::default()
        };
        assert_eq!(e.key(&ctrl_left).action, Action::Pass);
        assert_eq!(e.key(&KeyEvent::char_down('s')).action, Action::Pass);
    }

    #[test]
    fn control_chars_from_layout_are_navigation_not_boundary() {
        // macOS UCKeyTranslate: mũi tên trái = `\x1C`. Ở Preedit, bản cũ trả
        // COMMIT{"\x1C"} → adapter chèn ký tự điều khiển vào văn bản.
        let mut e = engine();
        e.set_context(Context {
            caps: strategy::IME_CAP_PREEDIT | strategy::IME_CAP_FIELD_DETECT,
            ..Default::default()
        });
        let _ = type_keys(&mut e, "tooi");
        for (vk, ch) in [(keymap::vk::LEFT, 0x1Cu32), (0, 0x7F), (0, 0x03)] {
            let o = e.key(&KeyEvent {
                vk,
                ch,
                key_down: true,
                ..Default::default()
            });
            assert_eq!(o.action, Action::Pass, "ch={ch:#x}");
            assert!(o.preedit.is_empty());
        }
        // Từ đã bỏ sở hữu: phím kế tiếp không REPLACE chữ cũ.
        assert_eq!(e.key(&KeyEvent::char_down('s')).action, Action::Pass);
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

    /// Tab hoàn tất từ tiếng Việt (`suggest.enabled` — 0.2.33): mặc định TẮT
    /// (Tab đi qua), bật thì tiền tố âm tiết → từ tần suất cao, giữ hoa đầu từ.
    #[test]
    fn vn_tab_complete() {
        // Mặc định tắt: Tab không hoàn tất — đi qua như ranh giới thường.
        let mut e = engine();
        let mut buf = type_buf(&mut e, "nguw");
        assert_eq!(text(&buf), "ngư");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "ngư\t", "suggest tắt → Tab đi qua");
        assert!(!e.word.active, "Tab đóng từ");

        // Bật: "nguw" → `ngư` + Tab → `ngưa` (vòng 52: pool mở rộng từ dic —
        // âm tiết ngắn nhất theo (độ dài, bảng chữ), trước đây là "ngưng").
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "nguw");
        let action = press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "ngưa");
        assert!(
            matches!(action, Action::Replace { .. }),
            "phải REPLACE toàn bộ từ"
        );
        assert!(!e.word.active, "hoàn tất xong phải đóng từ");

        // Hoa đầu từ được giữ: "Nguw" + Tab → "Ngưa".
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "Nguw");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "Ngưa");

        // Tiền tố không khớp ứng viên nào → Tab đi qua (indent/chuyển ô).
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "zz");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "zz\t");

        // Vòng 52: pool âm tiết VN mở rộng — "tes" gõ Telex ra "té" và nay có
        // âm tiết dài hơn ("téc") → VN hoàn tất trước EN (R2-56: VN thắng).
        // Đối chứng EN vẫn hoạt động với tiền tố KHÔNG có ứng viên VN ("rel").
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "tes");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "téc", "VN pool mở rộng hoàn tất trước");
        let mut e2 = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf2 = type_buf(&mut e2, "rele");
        press(&mut e2, &mut buf2, keymap::vk::TAB);
        assert_eq!(text(&buf2), "release", "EN completion vẫn hoạt động");
    }

    /// Vòng 24 (`suggest.trigger = "tab_and_space"`): Space cũng hoàn tất từ.
    #[test]
    fn space_complete_khi_bat_tab_and_space() {
        // Mặc định (chỉ Tab): Space là ranh giới thường — không hoàn tất.
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "rep");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "rep ", "mặc định Space không hoàn tất");

        // Bật Space — VN: display "ngư" (gõ "nguw") ĐÃ là âm tiết hợp lệ
        // → guard chặn viết lại (an toàn); Space đi qua như ranh giới.
        // (Hoàn tất VN qua Space gần như không bao giờ kích hoạt — đúng thiết
        // kế "không bao giờ viết lại từ đúng"; giá trị chính là đường EN.)
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            suggest_space_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "nguw");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "ngư ", "âm tiết hợp lệ + Space = giữ nguyên");

        // Gõ trực tiếp từ có dấu đúng + Space → giữ nguyên.
        let mut buf = type_buf(&mut e, "không");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "không ", "từ đã đúng + Space = giữ nguyên");

        // EN: "examp" (chưa phải từ) + Space → "example " (kèm space).
        let mut e = Engine::new(EngineOptions {
            suggest_space_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "examp");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "example ");

        // EN từ đã đầy đủ trong từ điển ("report", "test") + Space → KHÔNG
        // kéo dài ("reports ", "testing ").
        let mut buf = type_buf(&mut e, "report");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "report ");
        let mut buf = type_buf(&mut e, "test");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "test ", "từ điển biết \"test\" — không kéo dài");

        // Không khớp ứng viên ("zz") + Space → ranh giới thường.
        let mut buf = type_buf(&mut e, "zz");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "zz ");

        // Tab vẫn hoạt động khi bật Space (đường cũ không đổi).
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            suggest_space_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "nguw");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "ngưa", "Tab vẫn hoàn tất");

        // Bảo vệ từ tiếng Việt hợp lệ ("em", "con", "cho", "cá", "nó", "chó")
        // khỏi bị biến thành từ tiếng Anh trong EN_FREQ_DATA ("email", "confirm",
        // "choice", "case", "nose", "choose") khi bấm Space hoặc Tab.
        for (keys, expected_space, expected_tab) in [
            ("em", "em ", "em\t"),
            ("con", "con ", "con\t"),
            ("cho", "cho ", "cho\t"),
            ("cas", "cá ", "cá\t"),
            ("nos", "nó ", "nó\t"),
            ("chos", "chó ", "chó\t"),
        ] {
            let mut e_sp = Engine::new(EngineOptions {
                suggest_vn_complete: false,
                suggest_space_complete: true,
                ..Default::default()
            });
            let mut buf_sp = type_buf(&mut e_sp, keys);
            press(&mut e_sp, &mut buf_sp, keymap::vk::SPACE);
            assert_eq!(text(&buf_sp), expected_space, "{keys}+Space");

            let mut e_tab = Engine::new(EngineOptions::default());
            let mut buf_tab = type_buf(&mut e_tab, keys);
            press(&mut e_tab, &mut buf_tab, keymap::vk::TAB);
            assert_eq!(text(&buf_tab), expected_tab, "{keys}+Tab");
        }
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

    /// Từ điển EN thông dụng dựng sẵn (2026-10-02): `test` được restore dù
    /// người dùng chưa khai báo gì; từ Việt thật vẫn giữ fold.
    #[test]
    fn danh_sach_khong_dung_cho_tu_khac() {
        // `test` ∈ en_common (bật mặc định) → Space → trả lại "test"
        let mut e = Engine::new(EngineOptions::default());
        let mut buf = type_buf(&mut e, "test");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "test ");

        // Cặp mơ hồ hai chiều vẫn ưu tiên tiếng Việt: `cow` là cách gõ Telex
        // của `cơ` → giữ fold, KHÔNG restore
        let mut e = Engine::new(EngineOptions::default());
        let mut buf = type_buf(&mut e, "cow");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "cơ ");

        // và từ Việt thật vẫn đúng
        let mut e = engine();
        let mut buf = type_buf(&mut e, "dduocj");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "được ");
    }

    /// Ưu tiên CẤP 5 (user dict thắng vn_common): người dùng khai báo "cow"
    /// trong english_words → Space trả lại "cow" dù fold "cơ" là âm tiết Việt
    /// thông dụng. Xem docs/specs/language-detection.md.
    #[test]
    fn user_dictionary_beats_vn_common_protection() {
        let mut e = Engine::new(EngineOptions {
            english_words: vec!["cow".to_string()],
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "cow");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "cow ");
    }

    /// Tắt `auto_restore_english` → giữ fold kiểu UniKey cổ điển; Escape vẫn cứu.
    #[test]
    fn restore_tat_thi_giu_fold() {
        let mut e = Engine::new(EngineOptions {
            auto_restore_english: false,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "text");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "tẽt ");
    }

    /// Tab gợi ý hoàn tất từ EN: "tes" (fold "té") + Tab → "test" và đóng từ;
    /// Shift+Tab không đụng (phím hệ thống S9).
    #[test]
    fn tab_hoan_tat_tu_en_va_escape_restore() {
        let mut e = Engine::new(EngineOptions::default());
        let mut buf = type_buf(&mut e, "tes");
        assert_eq!(text(&buf), "té");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "test");

        // ESC: restore raw của từ đang active
        let mut e = Engine::new(EngineOptions::default());
        let mut buf = type_buf(&mut e, "dduocj");
        press(&mut e, &mut buf, keymap::vk::ESCAPE);
        assert_eq!(text(&buf), "dduocj");
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

    /// Gõ cả chuỗi (Space là phím Space) với cấu hình mặc định, trả document.
    fn type_doc(e: &mut Engine, input: &str) -> String {
        let mut buf: Vec<char> = Vec::new();
        for c in input.chars() {
            let k = if c == ' ' {
                KeyEvent::key_down(keymap::vk::SPACE)
            } else {
                KeyEvent::char_down(c)
            };
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        text(&buf)
    }

    /// R2-59: dấu chấm không kèm khoảng trắng (URL, e-mail, tên file, số thập phân, `v.v.`)
    /// không kết câu; `. ` / `.") ` / Enter vẫn viết hoa chữ kế tiếp.
    #[test]
    fn auto_capitalize_needs_space_after_dot() {
        let mut e = engine();
        assert_eq!(
            type_doc(&mut e, "google.com file.txt a@b.com gias 3.5 kg v.v... ok"),
            "google.com file.txt a@b.com giá 3.5 kg v.v... Ok"
        );
        let mut e = engine();
        assert_eq!(type_doc(&mut e, "chao.\") ban"), "chao.\") Ban");
        // Xoá Space sau dấu chấm → hết đầu câu; gõ lại Space → đầu câu lại.
        let mut e = engine();
        let mut buf = type_buf(&mut e, "hi.");
        press(&mut e, &mut buf, keymap::vk::SPACE);
        press(&mut e, &mut buf, keymap::vk::BACK);
        buf.pop();
        buf.extend(type_buf(&mut e, "com"));
        assert_eq!(text(&buf), "hi.com");
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

    /// R2-61: Simple Telex vẫn gõ được ă/ơ/ư bằng `w` (UniKey `vneHookAll`), kể cả khi
    /// auto-restore bật mặc định.
    #[test]
    fn simple_telex_w_is_horn_through_engine() {
        let mut e = Engine::new(EngineOptions {
            method: Method::SimpleTelex,
            auto_capitalize: false,
            ..Default::default()
        });
        let buf = type_buf(&mut e, "trawngs mowf tuw muwa nawm cuwar ");
        assert_eq!(text(&buf), "trắng mờ tư mưa năm cửa ");
    }

    // ---- P4-2a: buffer cau Unicode (`sentence`) + `glyph_len_of` (ADR-013) ----

    /// Buffer cau giu **Unicode** va khop dung duoi document (khong phai glyph).
    #[test]
    fn sentence_buffer_mirrors_typed_text() {
        let mut e = engine();
        let doc = type_doc(&mut e, "chaof ban");
        assert_eq!(doc, "chào ban");
        assert_eq!(e.sentence_tail(100), "chào ban");
    }

    /// Bat bien: `sentence` va `sentence_widths` luon cung do dai (F75).
    /// 3 buffer phai dong bo — lech la mau hong im lang.
    #[test]
    fn sentence_widths_stay_in_sync() {
        let mut e = Engine::new(EngineOptions {
            output_charset: OutputCharset::VniWindows,
            ..Default::default()
        });
        let _ = type_doc(&mut e, "dduocj ban ");
        assert_eq!(e.sentence.len(), e.sentence_widths.len());
        let _ = type_doc(&mut e, "abc");
        assert_eq!(e.sentence.len(), e.sentence_widths.len());
        e.reset();
        assert_eq!(e.sentence.len(), e.sentence_widths.len());
    }

    /// Buffer cau giu **Unicode** du bang ma xuat la 2 byte/chu (VNI Windows):
    /// app nhan glyph, tro ly ngu phap nhan char Unicode.
    #[test]
    fn sentence_keeps_unicode_under_legacy_charset() {
        let mut e = Engine::new(EngineOptions {
            output_charset: OutputCharset::VniWindows,
            ..Default::default()
        });
        let doc = type_doc(&mut e, "dduocj");
        assert_eq!(doc, "\u{f1}\u{f6}\u{f4}\u{ef}c", "document nhan glyph VNI");
        assert_eq!(e.sentence_tail(100), "được", "buffer cau giu Unicode");
    }

    /// `glyph_len_of` = so ky tu **glyph** app phai xoa khi ap dung goi y Unicode (D27).
    #[test]
    fn glyph_len_of_counts_encoded_glyphs() {
        let word: Vec<char> = "được".chars().collect();
        for (charset, want) in [
            (OutputCharset::UnicodePrecomposed, 4),
            (OutputCharset::UnicodeDecomposed, 5),
            (OutputCharset::Tcvn3, 4),
            (OutputCharset::VniWindows, 5),
        ] {
            let e = Engine::new(EngineOptions {
                output_charset: charset,
                ..Default::default()
            });
            assert_eq!(e.glyph_len_of(&word), want, "{charset:?}");
        }
    }

    /// `glyph_len_of` phai bang dung do dai `insert` ma engine tra ve cho app.
    #[test]
    fn glyph_len_of_matches_engine_insert() {
        for charset in [
            OutputCharset::UnicodePrecomposed,
            OutputCharset::UnicodeDecomposed,
            OutputCharset::Tcvn3,
            OutputCharset::VniWindows,
        ] {
            let mut e = Engine::new(EngineOptions {
                output_charset: charset,
                ..Default::default()
            });
            let mut insert_len = 0usize;
            for c in "dduocj".chars() {
                let k = KeyEvent::char_down(c);
                let o = e.key(&k);
                if let Action::Replace { insert, .. } = &o.action {
                    insert_len = insert.len();
                }
            }
            assert_eq!(
                e.glyph_len_of(&"được".chars().collect::<Vec<_>>()),
                insert_len
            );
        }
    }

    /// Focus chuyen / con tro nhay → xoa het ca 2 buffer.
    #[test]
    fn sentence_cleared_on_reset() {
        let mut e = engine();
        let _ = type_doc(&mut e, "chao");
        assert!(!e.sentence_tail(100).is_empty());
        e.reset();
        assert_eq!(e.sentence_tail(100), "");
    }

    /// Backspace xoa 1 **glyph** (`recent`) va 1 **ky tu Unicode** (`sentence`) — 2 buffer
    /// tach bach (F66).
    #[test]
    fn sentence_backspace_pops_one_unicode_char() {
        let mut e = engine();
        let _ = type_doc(&mut e, "abc");
        assert_eq!(e.sentence_tail(100), "abc");
        let _ = e.key(&KeyEvent::key_down(keymap::vk::BACK));
        assert_eq!(e.sentence_tail(100), "ab");
    }

    /// Buffer cau cap 256 ky tu Unicode (D31) — giu phan duoi.
    #[test]
    fn sentence_capped_at_256_chars() {
        let mut e = engine();
        // `b` khong phai dau Telex → moi ky tu giong 1 ky tu trong document.
        let doc = type_doc(&mut e, &"b".repeat(400));
        assert_eq!(doc.len(), 400);
        let tail = e.sentence_tail(1000);
        assert_eq!(tail.chars().count(), 256);
        assert_eq!(tail, "b".repeat(256));
    }

    /// Ghi tat (macro) cung vao buffer cau (Unicode), thay cho chuoi trigger da go.
    #[test]
    fn sentence_includes_macro_expansion() {
        let mut e = Engine::new(macro_opts(MacroTrigger::Tab));
        let mut buf = type_buf(&mut e, "cty");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "Công ty TNHH");
        assert_eq!(e.sentence_tail(100), "Công ty TNHH");
    }

    /// Thay tu: buffer cau giu chu **hien thi** (Unicode), khong phai chuoi phim go.
    #[test]
    fn sentence_holds_display_text_not_typed_keys() {
        let mut e = engine();
        let _ = type_doc(&mut e, "dduocj");
        assert_eq!(e.sentence_tail(100), "được");
    }

    /// sentence_tail_chars tra ve &[char] khong cap phat heap (Fassist-76).
    #[test]
    fn sentence_tail_chars_zero_alloc() {
        let mut e = engine();
        let _ = type_doc(&mut e, "xin chao");
        let chars = e.sentence_tail_chars(4);
        assert_eq!(chars, &['c', 'h', 'a', 'o']);
    }

    #[test]
    fn telex_bracket_keys_and_double_bracket_escape_in_engine() {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            ..Default::default()
        });
        assert_eq!(type_doc(&mut e, "ch]a "), "chưa ");
        assert_eq!(type_doc(&mut e, "m[f "), "mờ ");
        assert_eq!(type_doc(&mut e, "h[n "), "hơn ");
        assert_eq!(type_doc(&mut e, "th][ng "), "thương ");
        // Đầu từ ngoặc là ngoặc thật — không có gì để gỡ, `[[link]]` giữ nguyên (R2-140).
        assert_eq!(type_doc(&mut e, "[[ "), "[[ ");
        assert_eq!(type_doc(&mut e, "]] "), "]] ");
        assert_eq!(type_doc(&mut e, "[] "), "[] ");

        // Sau phụ âm ngoặc đã thành nguyên âm: bấm lại gỡ về dấu ngoặc.
        e.reset();
        assert_eq!(type_doc(&mut e, "m[[ "), "m[ ");
        e.reset();

        // Backspace giữa từ dùng `[` `]` rồi gõ tiếp vẫn giữ dấu sừng và ghép vần đúng.
        let mut buf = type_buf(&mut e, "ch]a");
        assert_eq!(text(&buf), "chưa");
        press(&mut e, &mut buf, keymap::vk::BACK);
        assert_eq!(text(&buf), "chư");
        for c in "af ".chars() {
            let k = KeyEvent::char_down(c);
            let o = e.key(&k);
            apply(&mut buf, &o.action, &k);
        }
        assert_eq!(text(&buf), "chừa ");
    }

    #[test]
    fn standalone_ctrl_tap_locks_word_and_prevents_auto_restore() {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            auto_restore_english: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "tunx");
        assert_eq!(text(&buf), "tũn");
        let _ = e.key(&KeyEvent::key_down(keymap::vk::CONTROL));
        let out_up = e.key(&KeyEvent::key_up(keymap::vk::CONTROL));
        assert_eq!(out_up.flags & FLAG_WORD_END, FLAG_WORD_END);
        press(&mut e, &mut buf, keymap::vk::SPACE);
        assert_eq!(text(&buf), "tũn ");

        // `see` -> `sê` là âm tiết Việt hợp lệ về cấu trúc (`validate::is_valid_word("sê") == true`),
        // bình thường tự restore về `see ` do nằm trong `en_common`, nhưng nếu chủ đích bấm thả Ctrl
        // thì khoá giữ nguyên âm tiết Việt hợp lệ `sê `.
        let mut e_valid = Engine::new(EngineOptions {
            auto_capitalize: false,
            auto_restore_english: true,
            ..Default::default()
        });
        let mut buf_valid = type_buf(&mut e_valid, "see");
        assert_eq!(text(&buf_valid), "sê");
        let _ = e_valid.key(&KeyEvent::key_down(keymap::vk::CONTROL));
        let _ = e_valid.key(&KeyEvent::key_up(keymap::vk::CONTROL));
        press(&mut e_valid, &mut buf_valid, keymap::vk::SPACE);
        assert_eq!(text(&buf_valid), "sê ");

        // Ngược lại, `sort` -> `sỏt` KHÔNG phải âm tiết Việt hợp lệ ("sỏt" không có nghĩa trong tiếng Việt:
        // âm cuối tắc `-t` không thể đi với thanh hỏi). Bấm thả riêng `Ctrl` rồi `Space` hoặc bấm `Ctrl+Space`
        // bắt buộc phải trả về `sort` chứ tuyệt đối không được giữ nguyên `sỏt`!
        let mut e2 = Engine::new(EngineOptions {
            auto_capitalize: false,
            auto_restore_english: true,
            ..Default::default()
        });
        let mut buf2 = type_buf(&mut e2, "sort");
        assert_eq!(text(&buf2), "sỏt");
        let _ = e2.key(&KeyEvent::key_down(keymap::vk::CONTROL));
        let ctrl_up = e2.key(&KeyEvent::key_up(keymap::vk::CONTROL));
        apply(
            &mut buf2,
            &ctrl_up.action,
            &KeyEvent::key_up(keymap::vk::CONTROL),
        );
        assert_eq!(text(&buf2), "sort");
        press(&mut e2, &mut buf2, keymap::vk::SPACE);
        assert_eq!(text(&buf2), "sort ");

        // `sort` (`sỏt`) + `Ctrl+Space` (chord) cũng bắt buộc trả về `sort` (cả khi bật và tắt dự đoán).
        for suggest_on in [false, true] {
            let mut e_cs = Engine::new(EngineOptions {
                auto_capitalize: false,
                auto_restore_english: true,
                suggest_vn_complete: suggest_on,
                ..Default::default()
            });
            let mut buf_cs = type_buf(&mut e_cs, "sort");
            assert_eq!(text(&buf_cs), "sỏt");
            let cs = KeyEvent {
                vk: keymap::vk::SPACE,
                mods: keymap::MOD_CTRL,
                key_down: true,
                ..Default::default()
            };
            let out_cs = e_cs.key(&cs);
            apply(&mut buf_cs, &out_cs.action, &cs);
            assert_eq!(text(&buf_cs), "sort", "Ctrl+Space với suggest={suggest_on}");
        }

        // `tun` + Ctrl tap + `x` -> `tunx` (x không còn ghép dấu ngã vào `tun`).
        let mut e3 = Engine::new(EngineOptions {
            auto_capitalize: false,
            ..Default::default()
        });
        let mut buf3 = type_buf(&mut e3, "tun");
        let _ = e3.key(&KeyEvent::key_down(keymap::vk::CONTROL));
        let _ = e3.key(&KeyEvent::key_up(keymap::vk::CONTROL));
        for c in "x ".chars() {
            let k = KeyEvent::char_down(c);
            let o = e3.key(&k);
            apply(&mut buf3, &o.action, &k);
        }
        assert_eq!(text(&buf3), "tunx ");
    }

    #[test]
    fn fast_double_dash_becomes_em_dash_and_triple_dash_restores() {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            ..Default::default()
        });
        assert_eq!(type_doc(&mut e, "a--b"), "a—b");
        e.reset();
        assert_eq!(type_doc(&mut e, "Hà Nội -- TP.HCM"), "Hà Nội — TP.HCM");
        e.reset();
        assert_eq!(type_doc(&mut e, "---"), "---");
    }

    #[test]
    fn mode_en_backspace_does_not_learn_incomplete_prefix() {
        let mut e = Engine::new(EngineOptions {
            auto_capitalize: false,
            style_learning_enabled: true,
            ..Default::default()
        });
        e.set_context(Context {
            enabled: false,
            ..Default::default()
        });
        for c in "devel".chars() {
            let _ = e.key(&KeyEvent::char_down(c));
        }
        // Backspace xóa 'l' — không được ghi nhận "devel" vào style profile.
        let _ = e.key(&KeyEvent::key_down(keymap::vk::BACK));
        let drained_after_bs = e.style_drain_json();
        assert!(
            !drained_after_bs.contains("\"devel\""),
            "Backspace ở Mode EN không được học tiền tố dở dang: {drained_after_bs}"
        );
        // Gõ tiếp "lopment " (Space là ranh giới từ in được) → học "development".
        for c in "lopment ".chars() {
            let _ = e.key(&KeyEvent::char_down(c));
        }
        let drained_after_space = e.style_drain_json();
        assert!(
            drained_after_space.contains("\"development\""),
            "Space ở Mode EN phải học từ hoàn chỉnh: {drained_after_space}"
        );
    }
}
