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
    /// `config.suggest.enabled` (P0-3 §1.1) — Tab hoàn tất từ tiếng Việt
    /// (`post/suggest_vn.rs`). Mặc định tắt: Tab giữ hành vi cũ (đi qua app).
    pub suggest_vn_complete: bool,
    /// `config.suggest.trigger` (vòng 24) — Space cũng hoàn tất từ khi từ
    /// đang gõ chưa phải từ đúng (kèm dấu cách). Mặc định `false` = chỉ Tab.
    pub suggest_space_complete: bool,
    /// `config.output_charset` — bảng mã chữ đi ra document (`transform/charset.rs`).
    pub output_charset: OutputCharset,
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

/// Engine — 1 instance = 1 thread (P0-2 §3).
pub struct Engine {
    opts: EngineOptions,
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
            opts,
            ctx: Context::default(),
            word: buffer::Word::default(),
            recent: Vec::new(),
            sentence: Vec::new(),
            sentence_widths: Vec::new(),
        }
    }

    /// `ime_reload_config` — đổi options giữa chừng → reset từ đang gõ dở.
    pub fn set_options(&mut self, opts: EngineOptions) {
        self.opts = opts;
        self.word.clear();
        self.record(Tail::Clear);
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

    /// Ô mật khẩu: cờ `secure` HOẶC role `secure` (adapter quên một trong hai vẫn an toàn — S3).
    fn secure(&self) -> bool {
        self.ctx.secure || self.ctx.field_role == strategy::IME_FIELD_SECURE
    }

    /// `ime_reset` — xóa trạng thái từ (focus change). Không đụng buffer của app.
    pub fn reset(&mut self) {
        self.word.clear();
        self.record(Tail::Clear);
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
            // Ctrl+V/Ctrl+Z/Ctrl+←/Ctrl+Backspace/Alt+Tab… đổi text hoặc vị trí con trỏ mà
            // engine không thấy: bỏ quyền sở hữu từ đang gõ (như phím điều hướng — B7) để
            // phím kế tiếp không REPLACE `owned` ký tự ở chỗ khác, và quên đuôi text để gõ
            // tắt kế tiếp không xoá nhầm (`vn` Ctrl+V `abc` Tab ≠ bung `vn`).
            self.word.clear();
            self.record(Tail::Clear);
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
                // Vòng 24 (`suggest.trigger = "tab_and_space"`): Space cũng
                // hoàn tất từ — nhưng **nghiêm ngặt hơn Tab** (Space là phím
                // ngầm định, người dùng không yêu cầu gì): chỉ hoàn tất khi từ
                // đang gõ CHƯA phải từ đúng hoàn chỉnh, và kèm chính dấu cách.
                let space_complete = k.vk == keymap::vk::SPACE && self.opts.suggest_space_complete;
                if k.vk == keymap::vk::TAB || space_complete {
                    let append: Option<char> = space_complete.then_some(c);
                    if let Some(out) = self.try_vn_complete(k, vi_on, append) {
                        return out;
                    }
                    if let Some(out) = self.try_english_complete(k, vi_on, append) {
                        return out;
                    }
                }
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

        if !method::is_word_char(c, self.opts.method) {
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

        // Auto-capitalize (P0-3 §1.1): chỉ áp cho **chữ đầu từ** ở đầu câu — sau `. ! ?` +
        // khoảng trắng hoặc Enter (R2-59: `google.com`, `3.5 kg` không phải đầu câu).
        let mut caps_fired = false;
        let c = if self.opts.auto_capitalize
            && self.word.is_empty()
            && post::caps::sentence_start(&self.recent)
        {
            caps_fired = true;
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
        let (len, text) = match_display.or(match_raw)?;
        let expanded: Vec<char> = text.chars().collect();
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

        // Auto-restore EN (bug B5): kết quả fold không phải âm tiết Việt → trả lại chuỗi gõ.
        if was_active && self.opts.auto_restore_english {
            let raw = self.word.raw.clone();
            let display = self.word.display.clone();
            if post::restore_en::should_restore(&raw, &display, &self.opts.english_words) {
                let delete = self.word.owned as u16;
                let mut insert = raw;
                insert.push(c); // ranh giới vẫn phải vào document (RESTORE = REPLACE — P0-4 §3)
                self.word.clear();
                // Document nhận đúng chuỗi gõ (không encode) → glyph = Unicode.
                self.record(Tail::Replace {
                    delete: delete as usize,
                    glyph: &insert,
                    unicode: &insert,
                });
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

        self.close_word(c, strategy)
    }

    /// Đóng từ đang gõ **giữ nguyên** kết quả trong document, rồi cho `c` vào sau nó
    /// (không auto-restore, không đổi cờ viết hoa — phần đó thuộc `on_boundary`).
    fn close_word(&mut self, c: char, strategy: Strategy) -> Outcome {
        let was_active = self.word.active;
        self.word.clear();
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
    fn try_vn_complete(
        &mut self,
        k: &KeyEvent,
        vi_on: bool,
        append: Option<char>,
    ) -> Option<Outcome> {
        if !vi_on || !self.opts.suggest_vn_complete {
            return None;
        }
        if k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }
        if !self.word.active || self.word.display.len() < 2 {
            return None;
        }
        let typed: String = self.word.display.iter().collect::<String>().to_lowercase();
        // Space (append) nghiêm ngặt hơn Tab: từ ĐÃ là âm tiết hợp lệ
        // ("không", "đi"…) không được viết lại — người dùng gõ đúng rồi.
        if append.is_some() && validate::is_valid_word(&self.word.display) {
            return None;
        }
        let completion = post::suggest_vn::complete_vn_word(&typed)?;
        let mut insert: Vec<char> = completion.chars().collect();
        if self.word.raw.first().is_some_and(|c| c.is_uppercase()) {
            if let Some(first) = insert.first_mut() {
                if let Some(up) = first.to_uppercase().next() {
                    *first = up;
                }
            }
        }
        if let Some(c) = append {
            insert.push(c); // Space: hoàn tất + chính dấu cách (như restore-EN)
        }
        let out = self.emit(&insert);
        if out.len() > MAX_TEXT {
            return None;
        }
        let delete = self.word.owned as u16;
        self.word.clear();
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
        if !vi_on || !self.opts.auto_restore_english {
            return None;
        }
        if k.mods & keymap::MOD_SHIFT != 0 {
            return None;
        }
        // Tab (append = None) giữ semantics cũ: chỉ từ ĐÃ transform (word
        // active) — giải cứu từ bị Telex biến đổi nhầm. Space (append =
        // Some) theo yêu cầu vòng 24: hoàn tất cả từ đang gõ dở CHƯA biến
        // đổi (kí tự đã PASS nằm trong document — xoá bằng pending_delete).
        let space_mode = append.is_some();
        if !self.word.active && !space_mode {
            return None;
        }
        if self.word.is_empty() || self.word.raw.len() < 2 {
            return None;
        }
        let typed: String = self.word.raw.iter().collect::<String>();
        // Space nghiêm ngặt hơn Tab: chuỗi gõ ĐÃ là từ EN thông dụng đúng
        // ("report") không được kéo dài ("reports") — người dùng gõ xong từ rồi;
        // và từ đang active đã là âm tiết tiếng Việt hợp lệ ("té", "chó", "nói")
        // thì Space chỉ đóng từ, không bao giờ tự biến thành từ EN.
        if space_mode
            && (post::restore_en::is_known_en_word(&typed.to_lowercase())
                || (self.word.active && validate::is_valid_word(&self.word.display)))
        {
            return None;
        }
        // Từ chưa biến đổi: display rỗng — fold chính là typed.
        let folded: String = if self.word.active {
            self.word.display.iter().collect::<String>()
        } else {
            typed.clone()
        };
        let completion = post::restore_en::complete_word(
            &typed.to_lowercase(),
            &folded.to_lowercase(),
            &self.opts.english_words,
        )?;
        let mut insert: Vec<char> = completion.chars().collect();
        if self.word.raw.first().is_some_and(|c| c.is_uppercase()) {
            if let Some(first) = insert.first_mut() {
                *first = first.to_ascii_uppercase();
            }
        }
        if let Some(c) = append {
            insert.push(c); // Space: hoàn tất + dấu cách
        }
        let out = self.emit(&insert);
        if out.len() > MAX_TEXT {
            return None;
        }
        let delete = if self.word.active {
            self.word.owned as u16
        } else {
            self.word.pending_delete()
        };
        self.word.clear();
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

        // Bật: "nguw" → `ngư` + Tab → `ngưng` (từ ngắn nhất, hoà độ dài thì
        // lấy mục đứng trước trong từ điển tần suất).
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "nguw");
        let action = press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "ngưng");
        assert!(
            matches!(action, Action::Replace { .. }),
            "phải REPLACE toàn bộ từ"
        );
        assert!(!e.word.active, "hoàn tất xong phải đóng từ");

        // Hoa đầu từ được giữ: "Nguw" + Tab → "Ngưng".
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "Nguw");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "Ngưng");

        // Tiền tố không khớp ứng viên nào → Tab đi qua (indent/chuyển ô).
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "zz");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "zz\t");

        // Viết tắt EN vẫn ưu tiên restore-EN: "tes" + Tab khi cả hai bật —
        // `tés` không phải prefix âm tiết VN nào → rơi xuống gợi ý EN `test`.
        let mut e = Engine::new(EngineOptions {
            suggest_vn_complete: true,
            ..Default::default()
        });
        let mut buf = type_buf(&mut e, "tes");
        press(&mut e, &mut buf, keymap::vk::TAB);
        assert_eq!(text(&buf), "test", "EN completion vẫn hoạt động");
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
        assert_eq!(text(&buf), "ngưng", "Tab vẫn hoàn tất");

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
}
