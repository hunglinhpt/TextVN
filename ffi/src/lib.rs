// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-ffi` — C-ABI v1, **nguồn sự thật duy nhất**: `docs/10-shared/P0-2-engine-ffi-contract.md`.
//!
//! Bất biến (P0-2 §0):
//! 1. Không alloc chéo ranh giới — mọi `*_v1` do caller cấp phát.
//! 2. Không callback từ Rust sang adapter.
//! 3. Fail-open: panic → `IME_OK` + `action=PASS` + `IME_FLAG_ERROR` (ngoại trừ NULL/ABI — lỗi caller).
//! 4. Không network/I/O/global mutable state trong instance.
//! 5. `ime_last_error()` không chứa text người dùng.
//!
//! Panic policy: `catch_unwind` quanh mọi entry (P0-2 §5) — **không** đặt `panic=abort`.
//!
//! `clippy::not_unsafe_ptr_arg_deref`: cố ý allow — các hàm `pub extern "C"` bắt buộc nhận con trỏ
//! thô (header C không có khái niệm `unsafe fn`); mọi entry đều null-check + ABI-check (P0-2 §2).

#![allow(clippy::not_unsafe_ptr_arg_deref)]

/// C API bảng cài đặt — header riêng `textvn_settings.h`, ngoài ABI engine P0-2.
pub mod settings;

use std::ffi::{c_char, c_void, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

use textvn_config::{parse_config, Config};
use textvn_core::{Action, Context, Engine, EngineOptions, KeyEvent, Outcome};
use textvn_strategy::{ResolveInput, Strategy};

/// Hằng modifier — header C gọi là `IME_MOD_*` (`P0-2 §1`).
pub use textvn_core::keymap::{
    MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_FN, MOD_META, MOD_SHIFT, MOD_SUPER,
};
/// OP-007 (P5): bộ chuyển mã vùng bôi đen — adapter (Windows TSF, sau này
/// macOS/Linux) dùng chung, không phải khai báo phụ thuộc `textvn-core`
/// riêng. Hàm thuần logic (S1), không đổi ABI C.
pub use textvn_core::{detect, transcode, Charset, TextCase};
/// Hằng **action** — 1 nguồn sự thật: `core` định nghĩa, header C khai báo `IME_ACTION_*`
/// với cùng giá trị (P0-2 §6). Re-export để adapter Rust, test và fuzz target không
/// phải chép số trần — chép tay là nguồn lệch ABI kinh điển.
pub use textvn_core::{ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE};
/// Hằng `field_role` / capability / strategy — header C khai báo đúng tên này
/// (`P0-2 §1`, `P0-3 §2.1`). Nguồn sự thật là crate `textvn-strategy`.
pub use textvn_strategy::{
    IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_CAP_SELECTION,
    IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO, IME_FIELD_EDITBOX,
    IME_FIELD_SEARCH, IME_FIELD_SECURE, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN,
    IME_FIELD_WEB, IME_STRATEGY_BACKSPACE_TYPE, IME_STRATEGY_FORWARD_AS_COMMIT,
    IME_STRATEGY_PASSTHROUGH, IME_STRATEGY_PREEDIT, IME_STRATEGY_SELECTION_REPLACE,
};

// ---- hằng số (bản 1-1 với header) ----
pub const IME_ABI_VERSION: u32 = 1;
pub const IME_MAX_TEXT: usize = 64;
pub const IME_MAX_SUGGEST: usize = 8;

pub const IME_OK: i32 = 0;
pub const IME_ERR_INVALID_ARG: i32 = -1;
pub const IME_ERR_CONFIG: i32 = -2;
pub const IME_ERR_ABI: i32 = -3;
pub const IME_ERR_INTERNAL: i32 = -4;

pub const IME_FLAG_CONSUMED: u32 = 0x1;
pub const IME_FLAG_WORD_END: u32 = 0x2;
pub const IME_FLAG_ERROR: u32 = 0x4;
pub const IME_FLAG_SUGGEST: u32 = 0x8;

// ---- assist constants (P4-0 §4 / §5) ----
pub const IME_ASSIST_KIND_DUPLICATE_WORD: u32 = 1;
pub const IME_ASSIST_KIND_SPACE_BEFORE_PUNCT: u32 = 2;
pub const IME_ASSIST_KIND_SPACE_AFTER_PUNCT: u32 = 3;
pub const IME_ASSIST_KIND_DOUBLE_SPACE: u32 = 4;
pub const IME_ASSIST_KIND_LOWERCASE_SENTENCE: u32 = 5;
pub const IME_ASSIST_KIND_DIACRITIC_TYPO: u32 = 6;
/// L9 - lap tu o ranh gioi cau (chi goi y).
pub const IME_ASSIST_KIND_SENTENCE_REPEAT: u32 = 7;
/// L10 - phu dinh kep (chi goi y).
pub const IME_ASSIST_KIND_DOUBLE_NEGATIVE: u32 = 8;
/// L12 - dau phay o cuoi cau (chi goi y).
pub const IME_ASSIST_KIND_COMMA_AT_END: u32 = 9;
/// AS-040 - tu la khong co trong tu dien (chi goi y, confidence 60).
pub const IME_ASSIST_KIND_SPELLING: u32 = 10;
/// EN - ma tu "a/an" lech am dau (vong 24; chi chay tren cau En-dominant).
pub const IME_ASSIST_KIND_ARTICLE_AN: u32 = 11;

pub const IME_MAX_ASSIST_ISSUES: usize = 8;
pub const IME_MAX_ASSIST_MSG: usize = 128;
pub const IME_MAX_ASSIST_SUGGEST: usize = 64;
pub const IME_MAX_SENTENCE_FIX_CHARS: usize = 1024;

pub const IME_ASSIST_TRIGGER_NONE: u32 = 0;
pub const IME_ASSIST_TRIGGER_WORD: u32 = 1;
pub const IME_ASSIST_TRIGGER_SENTENCE: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_key_v1 {
    pub abi_version: u32,
    pub vk: u32,
    pub ch: u32,
    pub mods: u32,
    pub key_down: u8,
    pub is_repeat: u8,
    pub is_injected: u8,
    pub _reserved: u8,
}

#[repr(C)]
pub struct ime_context_v1 {
    pub abi_version: u32,
    pub enabled: u32,
    pub secure: u32,
    pub field_role: u32,
    pub caps: u32,
    pub app_id: *const c_char,
    pub element_name: *const c_char,
    pub hint: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_result_v1 {
    pub abi_version: u32,
    pub action: u32,
    pub delete_count: u16,
    pub insert_len: u16,
    pub preedit_len: u16,
    pub _reserved: u16,
    pub flags: u32,
    pub insert: [u32; IME_MAX_TEXT],
    pub preedit: [u32; IME_MAX_TEXT],
}

impl ime_result_v1 {
    fn zeroed_pass() -> ime_result_v1 {
        // Toàn bộ số nguyên → an toàn khi zeroed
        unsafe { std::mem::zeroed::<ime_result_v1>() }
    }
}

#[repr(C)]
pub struct ime_suggest_v1 {
    pub abi_version: u32,
    pub count: u32,
    pub lens: [u32; IME_MAX_SUGGEST],
    pub items: [[u32; IME_MAX_TEXT]; IME_MAX_SUGGEST],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_assist_issue_v1 {
    pub kind: u32,
    pub start: u32,
    pub end: u32,
    pub auto_fixable: u32,
    pub confidence: u32,
    pub message: [c_char; IME_MAX_ASSIST_MSG],
    pub suggest_len: u32,
    pub suggest: [c_char; IME_MAX_ASSIST_SUGGEST],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_assist_result_v1 {
    pub abi_version: u32,
    pub count: u32,
    pub issues: [ime_assist_issue_v1; IME_MAX_ASSIST_ISSUES],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_assist_plan_v1 {
    pub abi_version: u32,
    pub fixed_count: u32,
    pub delete_from: u32,
    pub delete_chars: u32,
    pub delete_glyphs: u32,
    pub replacement: [c_char; IME_MAX_SENTENCE_FIX_CHARS],
    pub original_tail: [c_char; IME_MAX_SENTENCE_FIX_CHARS],
}

impl ime_assist_plan_v1 {
    #[allow(dead_code)]
    pub fn zeroed() -> Self {
        unsafe { std::mem::zeroed::<ime_assist_plan_v1>() }
    }
}

/// C-ABI callback provider từ điển cho spell checking OS (P4-10 §5.2 · AS-042).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ime_os_spell_v1 {
    pub abi_version: u32,
    pub contains:
        Option<unsafe extern "C" fn(ctx: *mut c_void, word: *const c_char, byte_len: u32) -> i32>,
    pub suggest: Option<
        unsafe extern "C" fn(
            ctx: *mut c_void,
            word: *const c_char,
            byte_len: u32,
            out: *mut c_char,
            out_len: u32,
            limit: u32,
        ) -> u32,
    >,
    pub ctx: *mut c_void,
}

/// Wrapper chuyển C-ABI [`ime_os_spell_v1`] thành [`textvn_assist::dict::Dictionary`].
pub struct CSpellProvider {
    provider: ime_os_spell_v1,
}

impl CSpellProvider {
    pub fn new(provider: ime_os_spell_v1) -> Self {
        Self { provider }
    }
}

impl textvn_assist::dict::Dictionary for CSpellProvider {
    fn contains(&self, word: &str) -> bool {
        let Some(contains_fn) = self.provider.contains else {
            return true;
        };
        let bytes = word.as_bytes();
        let len = bytes.len() as u32;
        let res = catch_unwind(AssertUnwindSafe(|| unsafe {
            contains_fn(self.provider.ctx, bytes.as_ptr() as *const c_char, len)
        }))
        .unwrap_or(1);
        res != 0
    }

    fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let Some(suggest_fn) = self.provider.suggest else {
            return Vec::new();
        };
        let bytes = word.as_bytes();
        let len = bytes.len() as u32;
        let mut out_buf = [0u8; 1024];
        let limit_u32 = (limit as u32).min(16);
        let count = catch_unwind(AssertUnwindSafe(|| unsafe {
            suggest_fn(
                self.provider.ctx,
                bytes.as_ptr() as *const c_char,
                len,
                out_buf.as_mut_ptr() as *mut c_char,
                out_buf.len() as u32,
                limit_u32,
            )
        }))
        .unwrap_or(0) as usize;

        if count == 0 {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(count.min(limit));
        let mut slice = &out_buf[..];
        for _ in 0..count {
            if let Some(nul_pos) = slice.iter().position(|&b| b == 0) {
                if nul_pos > 0 {
                    if let Ok(s) = std::str::from_utf8(&slice[..nul_pos]) {
                        results.push(s.to_string());
                    }
                }
                slice = &slice[nul_pos + 1..];
            } else {
                break;
            }
        }
        results
    }
}

impl ime_assist_result_v1 {
    #[allow(dead_code)]
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed::<ime_assist_result_v1>() }
    }
}

fn map_issue_kind(kind: textvn_assist::IssueKind) -> u32 {
    match kind {
        textvn_assist::IssueKind::DuplicateWord => IME_ASSIST_KIND_DUPLICATE_WORD,
        textvn_assist::IssueKind::SpaceBeforePunct => IME_ASSIST_KIND_SPACE_BEFORE_PUNCT,
        textvn_assist::IssueKind::SpaceAfterPunct => IME_ASSIST_KIND_SPACE_AFTER_PUNCT,
        textvn_assist::IssueKind::DoubleSpace => IME_ASSIST_KIND_DOUBLE_SPACE,
        textvn_assist::IssueKind::LowercaseSentenceStart => IME_ASSIST_KIND_LOWERCASE_SENTENCE,
        textvn_assist::IssueKind::DiacriticTypo => IME_ASSIST_KIND_DIACRITIC_TYPO,
        textvn_assist::IssueKind::SentenceRepeat => IME_ASSIST_KIND_SENTENCE_REPEAT,
        textvn_assist::IssueKind::DoubleNegative => IME_ASSIST_KIND_DOUBLE_NEGATIVE,
        textvn_assist::IssueKind::CommaAtEnd => IME_ASSIST_KIND_COMMA_AT_END,
        textvn_assist::IssueKind::Spelling => IME_ASSIST_KIND_SPELLING,
        textvn_assist::IssueKind::ArticleAn => IME_ASSIST_KIND_ARTICLE_AN,
    }
}

fn copy_str_to_c_buf(src: &str, dst: &mut [c_char]) -> usize {
    let bytes = src.as_bytes();
    let max = dst.len().saturating_sub(1);
    let mut len = bytes.len().min(max);
    while len > 0 && !src.is_char_boundary(len) {
        len -= 1;
    }
    for (d, &b) in dst[..len].iter_mut().zip(&bytes[..len]) {
        *d = b as c_char;
    }
    dst[len] = 0;
    len
}

fn fill_assist_result(issues: &[textvn_assist::Issue], out: &mut ime_assist_result_v1) {
    let selected: Vec<&textvn_assist::Issue> = if issues.len() <= IME_MAX_ASSIST_ISSUES {
        issues.iter().collect()
    } else {
        let mut by_prio: Vec<&textvn_assist::Issue> = issues.iter().collect();
        by_prio.sort_by_key(|i| (!i.auto_fixable, i.span.start, i.span.end, i.kind));
        by_prio.truncate(IME_MAX_ASSIST_ISSUES);
        by_prio.sort_by_key(|i| (i.span.start, i.span.end, i.kind));
        by_prio
    };
    let count = selected.len();
    out.abi_version = IME_ABI_VERSION;
    out.count = count as u32;
    for (i, issue) in selected.into_iter().enumerate() {
        let item = &mut out.issues[i];
        *item = unsafe { std::mem::zeroed::<ime_assist_issue_v1>() };
        item.kind = map_issue_kind(issue.kind);
        item.start = issue.span.start as u32;
        item.end = issue.span.end as u32;
        item.auto_fixable = if issue.auto_fixable { 1 } else { 0 };
        item.confidence = issue.confidence as u32;
        copy_str_to_c_buf(&issue.message, &mut item.message);
        if let Some(sugg) = issue.suggestions.first() {
            let written = copy_str_to_c_buf(sugg, &mut item.suggest);
            item.suggest_len = written as u32;
        }
    }
}

/// Opaque instance (P0-2 §1) — 1 instance = 1 thread (§3).
/// Tên `ime_instance` phải khớp header C (P0-2 §6).
#[allow(non_camel_case_types)]
pub struct ime_instance {
    engine: Engine,
    last_error: CString,
    spell_provider: Option<ime_os_spell_v1>,
}

fn options_from_config(cfg: &Config) -> EngineOptions {
    EngineOptions {
        method: match cfg.method {
            textvn_config::Method::Telex => textvn_core::Method::Telex,
            textvn_config::Method::Vni => textvn_core::Method::Vni,
            textvn_config::Method::Viqr => textvn_core::Method::Viqr,
            textvn_config::Method::SimpleTelex => textvn_core::Method::SimpleTelex,
        },
        diacritic_style: match cfg.diacritic_style {
            textvn_config::DiacriticStyle::New => textvn_core::DiacriticStyle::New,
            textvn_config::DiacriticStyle::Old => textvn_core::DiacriticStyle::Old,
        },
        free_marking: cfg.free_marking,
        enabled: cfg.enabled,
        auto_restore_english: cfg.auto_restore_english,
        auto_capitalize: cfg.auto_capitalize,
        macro_trigger: match cfg.macro_trigger {
            textvn_config::MacroTrigger::Tab => textvn_core::MacroTrigger::Tab,
            textvn_config::MacroTrigger::Space => textvn_core::MacroTrigger::Space,
        },
        allow_macro_when_vi_off: cfg.allow_macro_when_vi_off,
        // `config.macros[]` / `config.emoji[]` (P0-3 §1.1) — gõ tắt/emoji
        macros: cfg
            .macros
            .iter()
            .map(|m| textvn_core::MacroDef {
                trigger: m.trigger.clone(),
                expand: m.expand.clone(),
                when: match m.when {
                    textvn_config::MacroWhen::Always => textvn_core::MacroWhen::Always,
                    textvn_config::MacroWhen::ViOn => textvn_core::MacroWhen::ViOn,
                },
            })
            .collect(),
        emoji: cfg
            .emoji
            .iter()
            .map(|e| textvn_core::Emoji {
                trigger: e.trigger.clone(),
                glyph: e.glyph.clone(),
            })
            .collect(),
        // `config.english_words[]` — danh sách từ EN giữ nguyên (opt-in, xem core/post/restore_en.rs)
        english_words: cfg.english_words.clone(),
        quick_telex: cfg.quick_telex,
        suggest_vn_complete: cfg.suggest.enabled,
        suggest_space_complete: cfg.suggest.enabled
            && cfg.suggest.trigger == textvn_config::SuggestTrigger::TabAndSpace,
        output_charset: match cfg.output_charset {
            textvn_config::OutputCharset::UnicodePrecomposed => {
                textvn_core::OutputCharset::UnicodePrecomposed
            }
            textvn_config::OutputCharset::UnicodeDecomposed => {
                textvn_core::OutputCharset::UnicodeDecomposed
            }
            textvn_config::OutputCharset::Tcvn3 => textvn_core::OutputCharset::Tcvn3,
            textvn_config::OutputCharset::VniWindows => textvn_core::OutputCharset::VniWindows,
        },
    }
}

/// Chạy `f` dưới `catch_unwind` — P0-2 §5: MỌI entry chặn panic, kể cả đoạn parse dữ
/// liệu ngoài (config/appdb). Panic vượt `extern "C"` là abort cả process chủ — với TIP
/// in-process đó là app người dùng đang gõ.
fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

fn set_error(inst: &mut ime_instance, msg: &str) {
    // Chỉ chứa kind — không text người dùng (S2)
    inst.last_error = CString::new(msg).unwrap_or_else(|_| CString::new("").unwrap());
}

// ---- API (P0-2 §1) ----

#[no_mangle]
pub extern "C" fn ime_abi_version() -> u32 {
    IME_ABI_VERSION
}

/// config sai schema → VẪN tạo instance với config mặc định, trả `IME_ERR_CONFIG`
/// (non-fatal — F0-015), `*out` luôn set. Panic → `IME_ERR_INTERNAL`, `*out` = NULL.
#[no_mangle]
pub extern "C" fn ime_instance_new(
    config_utf8: *const u8,
    len: usize,
    out: *mut *mut ime_instance,
) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    unsafe { *out = std::ptr::null_mut() };

    let cfg_result: Result<Config, ()> = if config_utf8.is_null() {
        if len == 0 {
            Ok(Config::default())
        } else {
            return IME_ERR_INVALID_ARG;
        }
    } else {
        let bytes = unsafe { slice::from_raw_parts(config_utf8, len) };
        match std::str::from_utf8(bytes) {
            Ok(s) => guarded(|| parse_config(s)).and_then(Result::ok).ok_or(()),
            Err(_) => Err(()),
        }
    };

    let (options, status, err_msg) = match cfg_result {
        Ok(cfg) => (options_from_config(&cfg), IME_OK, ""),
        // Non-fatal: default config + IME_ERR_CONFIG (P0-2 §1/§5)
        Err(()) => (
            EngineOptions::default(),
            IME_ERR_CONFIG,
            "config: invalid schema",
        ),
    };

    let result = catch_unwind(AssertUnwindSafe(|| {
        Box::new(ime_instance {
            engine: Engine::new(options),
            last_error: CString::new("").unwrap(),
            spell_provider: None,
        })
    }));

    match result {
        Ok(boxed) => {
            unsafe { *out = Box::into_raw(boxed) };
            if status == IME_ERR_CONFIG {
                // *out đã set; ghi last_error
                let inst = unsafe { &mut **out };
                set_error(inst, err_msg);
            }
            status
        }
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// `ime_instance_free(NULL)` là no-op (P0-2 §3).
#[no_mangle]
pub extern "C" fn ime_instance_free(inst: *mut ime_instance) {
    if inst.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe { drop(Box::from_raw(inst)) }));
}

#[no_mangle]
pub extern "C" fn ime_set_context(inst: *mut ime_instance, ctx: *const ime_context_v1) -> i32 {
    if inst.is_null() || ctx.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &mut *inst };
    let ctx = unsafe { &*ctx };
    if ctx.abi_version != IME_ABI_VERSION {
        set_error(inst, "abi mismatch: context");
        return IME_ERR_ABI;
    }
    // IME_FIELD_SECURE = giá trị field_role lớn nhất hợp lệ (trùng hằng
    // `strategy` — trước đây là số ma thuật `10`, lệch khi mở rộng role).
    if ctx.field_role > textvn_strategy::IME_FIELD_SECURE {
        set_error(inst, "invalid field_role");
        return IME_ERR_INVALID_ARG;
    }
    let core_ctx = Context {
        enabled: ctx.enabled != 0,
        secure: ctx.secure != 0,
        field_role: ctx.field_role,
        caps: ctx.caps,
        hint: ctx.hint,
    };
    match catch_unwind(AssertUnwindSafe(|| inst.engine.set_context(core_ctx))) {
        Ok(()) => IME_OK,
        Err(_) => {
            inst.engine.reset();
            IME_ERR_INTERNAL
        }
    }
}

/// Entry point nóng — mọi đường đi đều trả lời (fail-open, P0-2 §0.3):
/// - NULL args → `IME_ERR_INVALID_ARG` (out điền PASS+ERROR nếu được).
/// - `k.abi_version` lệch → `IME_ERR_ABI`.
/// - Panic → `IME_OK` + PASS + `IME_FLAG_ERROR` (đúng invariant §0.3).
#[no_mangle]
pub extern "C" fn ime_key(
    inst: *mut ime_instance,
    k: *const ime_key_v1,
    out: *mut ime_result_v1,
) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let result = unsafe { &mut *out };
    *result = ime_result_v1::zeroed_pass();
    result.abi_version = IME_ABI_VERSION;
    result.action = ACTION_PASS;
    result.flags = IME_FLAG_ERROR;

    if inst.is_null() || k.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &mut *inst };
    let k = unsafe { &*k };
    if k.abi_version != IME_ABI_VERSION {
        set_error(inst, "abi mismatch: key");
        return IME_ERR_ABI;
    }

    let key = KeyEvent {
        vk: k.vk,
        ch: k.ch,
        mods: k.mods,
        key_down: k.key_down != 0,
        is_repeat: k.is_repeat != 0,
        is_injected: k.is_injected != 0,
    };

    let failure = match catch_unwind(AssertUnwindSafe(|| inst.engine.key(&key))) {
        Ok(outcome) if fill_result(result, &outcome) => return IME_OK,
        // Engine phải tự giữ ≤ IME_MAX_TEXT (P0-2 §2). Cắt bớt `insert` sẽ làm lệch
        // số ký tự engine tin là đang sở hữu → phím sau xoá lẹm text của người dùng;
        // thà bỏ từ đang gõ và cho phím đi thẳng.
        Ok(_) => "internal: result overflow",
        Err(_) => "internal: panic caught",
    };
    // Fail-open: trạng thái treo → reset, phím đi thẳng (S4)
    let _ = catch_unwind(AssertUnwindSafe(|| inst.engine.reset()));
    *result = ime_result_v1::zeroed_pass();
    result.abi_version = IME_ABI_VERSION;
    result.action = ACTION_PASS;
    result.flags = IME_FLAG_ERROR;
    set_error(inst, failure);
    IME_OK
}

/// Ghi `outcome` vào `result`; `false` (không ghi gì) nếu text vượt `IME_MAX_TEXT`.
fn fill_result(result: &mut ime_result_v1, outcome: &Outcome) -> bool {
    let mut insert: Vec<u32> = Vec::new();
    let mut delete_count: u16 = 0;
    result.action = match &outcome.action {
        Action::Pass => ACTION_PASS,
        Action::Replace {
            delete_count: d,
            insert: i,
        } => {
            delete_count = *d;
            insert.extend(i.iter().map(|&c| c as u32));
            ACTION_REPLACE
        }
        Action::Commit { insert: i } => {
            insert.extend(i.iter().map(|&c| c as u32));
            ACTION_COMMIT
        }
        Action::Restore {
            delete_count: d,
            insert: i,
        } => {
            delete_count = *d;
            insert.extend(i.iter().map(|&c| c as u32));
            ACTION_RESTORE
        }
    };
    if insert.len() > IME_MAX_TEXT || outcome.preedit.len() > IME_MAX_TEXT {
        return false;
    }
    let preedit: Vec<u32> = outcome.preedit.iter().map(|&c| c as u32).collect();

    result.delete_count = delete_count;
    result.insert_len = insert.len() as u16;
    result.preedit_len = preedit.len() as u16;
    result.flags = outcome.flags;
    result.insert[..insert.len()].copy_from_slice(&insert);
    result.preedit[..preedit.len()].copy_from_slice(&preedit);
    true
}

#[no_mangle]
pub extern "C" fn ime_reset(inst: *mut ime_instance) -> i32 {
    if inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &mut *inst };
    match catch_unwind(AssertUnwindSafe(|| inst.engine.reset())) {
        Ok(()) => IME_OK,
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// R2-131: `ime_reset` + ghi nhận `utf8` (câu trợ lý vừa sửa, đã nằm trong
/// document trước con trỏ) làm đuôi buffer câu — phím tắt bấm tiếp dự đoán được
/// theo ngữ cảnh. `utf8` NULL/không hợp lệ ⇒ chỉ reset (fail-safe).
#[no_mangle]
pub extern "C" fn ime_reset_with_tail(
    inst: *mut ime_instance,
    utf8: *const c_char,
    len: usize,
) -> i32 {
    if inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &mut *inst };
    let tail = if utf8.is_null() || len == 0 {
        ""
    } else {
        // SAFETY: caller bảo đảm `len` byte đọc được tại `utf8`.
        let bytes = unsafe { std::slice::from_raw_parts(utf8.cast::<u8>(), len) };
        std::str::from_utf8(bytes).unwrap_or("")
    };
    match catch_unwind(AssertUnwindSafe(|| inst.engine.reset_with_tail(tail))) {
        Ok(()) => IME_OK,
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// P6-11 · R2-134: ghost text — phần chữ Tab SẼ nối thêm sau con trỏ, ghi UTF-8
/// kèm NUL vào `out` (`cap` byte). Trả số byte đã ghi (0 = không có gợi ý /
/// không vừa `cap`); lỗi caller = `IME_ERR_*`. Không đổi trạng thái engine,
/// không log nội dung (S2).
#[no_mangle]
pub extern "C" fn ime_peek_completion(
    inst: *const ime_instance,
    out: *mut c_char,
    cap: usize,
) -> i32 {
    if inst.is_null() || out.is_null() || cap == 0 {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &*inst };
    // SAFETY: caller cấp `cap` byte ghi được tại `out`.
    let buf = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), cap) };
    buf[0] = 0;
    let ghost = catch_unwind(AssertUnwindSafe(|| inst.engine.peek_completion()))
        .ok()
        .flatten();
    let Some(ghost) = ghost else {
        return 0;
    };
    let bytes = ghost.as_bytes();
    if bytes.len() + 1 > cap {
        return 0; // không cắt giữa ký tự UTF-8 — không vừa thì không vẽ
    }
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[bytes.len()] = 0;
    bytes.len() as i32
}

/// OP-007b · R2-139: chuyển đổi vùng bôi đen theo kiểu gõ + bảng mã của instance
/// (`Engine::convert_selection` — chữ lạ do đọc nhầm UTF-8, bảng mã cũ → bảng mã
/// đang dùng, từ ASCII gõ nhầm chế độ E → gõ lại). Vào/ra UTF-8; ghi kèm NUL vào
/// `out` (`cap` byte). Trả số byte đã ghi; 0 = không đổi gì / không vừa `cap`;
/// lỗi caller = `IME_ERR_*`. Không đổi trạng thái engine, không log nội dung (S2).
#[no_mangle]
pub extern "C" fn ime_convert_selection(
    inst: *const ime_instance,
    text_utf8: *const c_char,
    len: usize,
    out: *mut c_char,
    cap: usize,
) -> i32 {
    if inst.is_null() || text_utf8.is_null() || out.is_null() || cap == 0 {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &*inst };
    // SAFETY: caller cấp `len` byte đọc được tại `text_utf8`, `cap` byte ghi được tại `out`.
    let input = unsafe { std::slice::from_raw_parts(text_utf8.cast::<u8>(), len) };
    let buf = unsafe { std::slice::from_raw_parts_mut(out.cast::<u8>(), cap) };
    buf[0] = 0;
    let Ok(text) = std::str::from_utf8(input) else {
        return 0;
    };
    let converted = catch_unwind(AssertUnwindSafe(|| {
        let chars: Vec<char> = text.chars().collect();
        inst.engine
            .convert_selection(&chars)
            .map(|v| v.into_iter().collect::<String>())
    }))
    .ok()
    .flatten();
    let Some(converted) = converted else {
        return 0;
    };
    let bytes = converted.as_bytes();
    if bytes.len() + 1 > cap {
        return 0;
    }
    buf[..bytes.len()].copy_from_slice(bytes);
    buf[bytes.len()] = 0;
    bytes.len() as i32
}

/// Config mới không hợp lệ → **giữ config đang chạy** + `IME_ERR_CONFIG`
/// (P0-3 §1.3 "sai schema → giữ config cũ"; WIN-053).
#[no_mangle]
pub extern "C" fn ime_reload_config(inst: *mut ime_instance, cfg: *const u8, len: usize) -> i32 {
    if inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    if cfg.is_null() {
        return if len == 0 {
            IME_OK
        } else {
            IME_ERR_INVALID_ARG
        };
    }
    let inst = unsafe { &mut *inst };
    let bytes = unsafe { slice::from_raw_parts(cfg, len) };
    let parsed = match std::str::from_utf8(bytes) {
        Ok(s) => match guarded(|| parse_config(s)) {
            Some(r) => r,
            None => return IME_ERR_INTERNAL,
        },
        Err(_) => Err(textvn_config::ConfigError::Schema),
    };
    match parsed {
        Ok(c) => match guarded(|| inst.engine.set_options(options_from_config(&c))) {
            Some(()) => IME_OK,
            None => IME_ERR_INTERNAL,
        },
        Err(e) => {
            set_error(inst, &e.to_string());
            IME_ERR_CONFIG
        }
    }
}

/// Feature `suggest` tắt mặc định → `IME_ERR_INTERNAL` (P0-2 §1, P0-3 §1.1).
#[no_mangle]
pub extern "C" fn ime_suggest(
    inst: *mut ime_instance,
    _word: *const u32,
    _word_len: u32,
    out: *mut ime_suggest_v1,
) -> i32 {
    if inst.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    *out = ime_suggest_v1 {
        abi_version: IME_ABI_VERSION,
        count: 0,
        lens: [0; IME_MAX_SUGGEST],
        items: [[0; IME_MAX_TEXT]; IME_MAX_SUGGEST],
    };
    IME_ERR_INTERNAL
}

/// Kiểm tra appdb trước khi adapter dùng nó. Chưa có `data/preset.pub` trong
/// repository/release artifact, nên hàm cố ý từ chối mọi chữ ký (fail-closed)
/// thay vì nhận một khóa tự sinh hoặc xác thực giả. `ime_strategy_resolve` chỉ
/// nên được gọi sau khi loader nhận `IME_OK` ở một bản phát hành có public key.
///
/// Thứ tự khai báo khớp khối `/* ---- API ---- */` trong header
/// (verify → resolve → last_error, P0-2 §1) — `textvn verify` enforce.
#[no_mangle]
pub extern "C" fn ime_appdb_verify(
    json: *const u8,
    json_len: usize,
    sig: *const u8,
    sig_len: usize,
) -> i32 {
    if (json.is_null() && json_len != 0) || (sig.is_null() && sig_len != 0) {
        return IME_ERR_INVALID_ARG;
    }
    if json.is_null() || sig.is_null() {
        return IME_ERR_INVALID_ARG;
    }

    let json = unsafe { slice::from_raw_parts(json, json_len) };
    let text = match std::str::from_utf8(json) {
        Ok(value) => value,
        Err(_) => return IME_ERR_CONFIG,
    };
    if !guarded(|| textvn_appdb::AppDb::parse(text).is_ok()).unwrap_or(false) {
        return IME_ERR_CONFIG;
    }
    let _signature = unsafe { slice::from_raw_parts(sig, sig_len) };
    IME_ERR_INTERNAL
}

/// Resolve strategy tập trung cho các adapter Swift/C/C++. Không giữ state và
/// không tin dữ liệu appdb: lỗi UTF-8/schema trả `IME_ERR_CONFIG`, còn `*out`
/// luôn được đặt Passthrough trước để adapter có thể fail-open an toàn.
#[no_mangle]
pub extern "C" fn ime_strategy_resolve(
    ctx: *const ime_context_v1,
    appdb_utf8: *const u8,
    len: usize,
    out_strategy: *mut i64,
) -> i32 {
    if out_strategy.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    unsafe { *out_strategy = Strategy::Passthrough.id() };
    if ctx.is_null() || (appdb_utf8.is_null() && len != 0) {
        return IME_ERR_INVALID_ARG;
    }

    let ctx = unsafe { &*ctx };
    if ctx.abi_version != IME_ABI_VERSION {
        return IME_ERR_ABI;
    }
    if ctx.field_role > textvn_strategy::IME_FIELD_SECURE {
        return IME_ERR_INVALID_ARG;
    }

    let app_id = if ctx.app_id.is_null() {
        ""
    } else {
        match unsafe { CStr::from_ptr(ctx.app_id) }.to_str() {
            Ok(value) => value,
            Err(_) => return IME_ERR_INVALID_ARG,
        }
    };
    let db = if appdb_utf8.is_null() {
        None
    } else {
        let bytes = unsafe { slice::from_raw_parts(appdb_utf8, len) };
        let json = match std::str::from_utf8(bytes) {
            Ok(value) => value,
            Err(_) => return IME_ERR_CONFIG,
        };
        match guarded(|| textvn_appdb::AppDb::parse(json)) {
            Some(Ok(value)) => Some(value),
            Some(Err(_)) => return IME_ERR_CONFIG,
            None => return IME_ERR_INTERNAL,
        }
    };

    let input = ResolveInput {
        secure: ctx.secure != 0,
        enabled: ctx.enabled != 0,
        user_preset: None,
        system_preset: None,
        hint: ctx.hint,
        field_role: ctx.field_role,
        caps: ctx.caps,
    };
    match catch_unwind(AssertUnwindSafe(|| match db {
        Some(db) => db.resolve(input, app_id),
        None => textvn_strategy::resolve(input),
    })) {
        Ok(strategy) => {
            unsafe { *out_strategy = strategy.id() };
            IME_OK
        }
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// Trả NULL thay vì panic khi instance NULL (adapter không được crash).
///
/// Đứng cuối file export — khớp khối `/* ---- API ---- */` trong header
/// (verify → resolve → last_error, P0-2 §1). `textvn verify` enforce thứ tự.
///
/// Contract con trỏ: trả về pointer vào bộ đệm `CString` NỘI TẠI instance —
/// hợp lệ cho đến lần gọi `ime_key`/`ime_reload_config`/`ime_*` kế tiếp trên
/// cùng instance (set_error có thể realloc). Caller dùng ngay rồi sao chép,
/// KHÔNG giữ pointer qua các lần gọi sau và KHÔNG free.
#[no_mangle]
pub extern "C" fn ime_last_error(inst: *const ime_instance) -> *const c_char {
    if inst.is_null() {
        return c"".as_ptr();
    }
    let inst = unsafe { &*inst };
    inst.last_error.as_ptr()
}

/// Xác định loại trigger của ký tự Unicode (D11 · P4-0 §4.1).
///
/// Trả về:
/// - `IME_ASSIST_TRIGGER_SENTENCE` (2): kết thúc câu ('.', '!', '?') -> điểm hiển thị UI.
/// - `IME_ASSIST_TRIGGER_WORD` (1): ranh giới từ/vế (space, ',', ';', ':') -> điểm kiểm tra silent.
/// - `IME_ASSIST_TRIGGER_NONE` (0): không phải trigger.
#[no_mangle]
pub extern "C" fn ime_assist_trigger_kind(ch: u32) -> u32 {
    let Some(c) = char::from_u32(ch) else {
        return IME_ASSIST_TRIGGER_NONE;
    };
    match c {
        '.' | '!' | '?' => IME_ASSIST_TRIGGER_SENTENCE,
        ' ' | ',' | ';' | ':' => IME_ASSIST_TRIGGER_WORD,
        _ => IME_ASSIST_TRIGGER_NONE,
    }
}

/// Kiểm tra ngữ pháp chuỗi UTF-8 bất kỳ (D23).
///
/// Caller cấp phát `out` trên stack với `out.abi_version = IME_ABI_VERSION`.
/// Fail-open: panic hoặc lỗi UTF-8 -> `count = 0` + mã lỗi tương ứng.
#[no_mangle]
pub extern "C" fn ime_assist_check(
    text_utf8: *const c_char,
    len: usize,
    out: *mut ime_assist_result_v1,
) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    if out.abi_version != IME_ABI_VERSION {
        return IME_ERR_ABI;
    }
    if text_utf8.is_null() {
        return if len == 0 {
            out.count = 0;
            IME_OK
        } else {
            IME_ERR_INVALID_ARG
        };
    }
    let bytes = unsafe { slice::from_raw_parts(text_utf8 as *const u8, len) };
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return IME_ERR_INVALID_ARG,
    };
    match catch_unwind(AssertUnwindSafe(|| {
        let issues = textvn_assist::check_sentence(text);
        fill_assist_result(&issues, out);
    })) {
        Ok(()) => IME_OK,
        Err(_) => {
            out.count = 0;
            IME_ERR_INTERNAL
        }
    }
}

/// Kiểm tra buffer câu hiện tại của instance (D23 · D24).
///
/// Đọc `sentence_tail(256)` từ engine của instance, chạy bộ kiểm tra ngữ pháp.
#[no_mangle]
pub extern "C" fn ime_assist_check_instance(
    inst: *mut ime_instance,
    out: *mut ime_assist_result_v1,
) -> i32 {
    if inst.is_null() || out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    if out.abi_version != IME_ABI_VERSION {
        return IME_ERR_ABI;
    }
    let inst = unsafe { &mut *inst };
    match catch_unwind(AssertUnwindSafe(|| {
        let text = inst.engine.sentence_tail(256);
        let issues = match inst.spell_provider {
            Some(ref sp) => {
                let provider = CSpellProvider::new(*sp);
                textvn_assist::check_sentence_with(&text, &provider)
            }
            None => textvn_assist::check_sentence(&text),
        };
        fill_assist_result(&issues, out);
    })) {
        Ok(()) => IME_OK,
        Err(_) => {
            out.count = 0;
            IME_ERR_INTERNAL
        }
    }
}

/// Lấy phần đuôi buffer câu của instance dưới dạng chuỗi UTF-8 (Fassist-76).
///
/// Ghi tối đa `buf_len - 1` byte vào `out_buf` kèm byte 0 kết thúc.
/// `*out_len` nhận số byte UTF-8 thực tế (không tính byte 0).
/// Nếu `out_buf == NULL && buf_len == 0`, hàm chỉ ghi độ dài cần thiết vào `*out_len`.
#[no_mangle]
pub extern "C" fn ime_assist_sentence_tail(
    inst: *const ime_instance,
    max_chars: usize,
    out_buf: *mut c_char,
    buf_len: usize,
    out_len: *mut usize,
) -> i32 {
    if inst.is_null() || out_len.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &*inst };
    match catch_unwind(AssertUnwindSafe(|| {
        let text = inst.engine.sentence_tail(max_chars);
        let bytes = text.as_bytes();
        unsafe { *out_len = bytes.len() };
        if !out_buf.is_null() && buf_len > 0 {
            let mut copy_len = bytes.len().min(buf_len.saturating_sub(1));
            while copy_len > 0 && !text.is_char_boundary(copy_len) {
                copy_len -= 1;
            }
            let out_slice = unsafe { slice::from_raw_parts_mut(out_buf as *mut u8, buf_len) };
            out_slice[..copy_len].copy_from_slice(&bytes[..copy_len]);
            out_slice[copy_len] = 0;
        }
    })) {
        Ok(()) => IME_OK,
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// Tính số ký tự glyph (đã encode theo bảng mã xuất của instance) của chuỗi Unicode (D27).
///
/// Dùng để adapter biết cần xoá bao nhiêu ký tự trong document khi áp dụng gợi ý.
#[no_mangle]
pub extern "C" fn ime_assist_glyph_len(
    inst: *const ime_instance,
    text_utf8: *const c_char,
    len: usize,
    out_glyphs: *mut usize,
) -> i32 {
    if inst.is_null() || out_glyphs.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    if text_utf8.is_null() && len != 0 {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &*inst };
    let text = if len == 0 || text_utf8.is_null() {
        ""
    } else {
        let bytes = unsafe { slice::from_raw_parts(text_utf8 as *const u8, len) };
        match std::str::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => return IME_ERR_INVALID_ARG,
        }
    };
    match catch_unwind(AssertUnwindSafe(|| {
        let chars: Vec<char> = text.chars().collect();
        let glyphs = inst.engine.glyph_len_of(&chars);
        unsafe { *out_glyphs = glyphs };
    })) {
        Ok(()) => IME_OK,
        Err(_) => IME_ERR_INTERNAL,
    }
}

/// Gắn provider spell-check của hệ điều hành vào instance (AS-042 · P4-10 §5.2).
///
/// Truyền `provider = NULL` để hủy provider hiện tại (quay về `NoDictionary`).
#[no_mangle]
pub extern "C" fn ime_assist_set_spell_provider(
    inst: *mut ime_instance,
    provider: *const ime_os_spell_v1,
) -> i32 {
    if inst.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let inst = unsafe { &mut *inst };
    if provider.is_null() {
        inst.spell_provider = None;
        return IME_OK;
    }
    let p = unsafe { *provider };
    if p.abi_version != IME_ABI_VERSION {
        return IME_ERR_ABI;
    }
    inst.spell_provider = Some(p);
    IME_OK
}

/// Chuyển UTF-16 thành `String` — trả `Err` khi dãy lệch (unpaired surrogate).
fn utf16_to_string(units: &[u16]) -> Result<String, ()> {
    String::from_utf16(units).map_err(|_| ())
}

/// Kiểm tra ngữ pháp câu ở dạng **UTF-16** (D29 — biên chuẩn TSF/AX/NSString),
/// trả về **số issue** đã ghi vào `*out` (0..`IME_MAX_ASSIST_ISSUES`).
///
/// Contract:
/// - `out` do caller cấp phát, đặt `abi_version = IME_ABI_VERSION` trước khi gọi.
/// - Lỗi caller (NULL out, `text` NULL với `len != 0`, ABI lệch) → mã `IME_ERR_*`
///   như các entry P0-2 khác; **không panic**.
/// - Lỗi dữ liệu/runtime (UTF-16 lệch, panic bên trong) → **fail-open**: ghi
///   `out.count = 0` và trả 0 (S4 — "thà im lặng còn hơn báo sai/chặn app").
/// - Offset trong issue là **char Unicode** (D27), đã quy đổi từ UTF-16 trong
///   hàm — adapter không phải tự đếm lại.
/// - Không log nội dung câu (S2).
#[no_mangle]
pub extern "C" fn ime_assist_check_v1(
    text_utf16: *const u16,
    len: i32,
    out: *mut ime_assist_result_v1,
) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    if out.abi_version != IME_ABI_VERSION {
        out.count = 0;
        return IME_ERR_ABI;
    }
    if text_utf16.is_null() {
        return if len == 0 {
            out.count = 0;
            IME_OK
        } else {
            IME_ERR_INVALID_ARG
        };
    }
    out.count = 0;
    if len < 0 {
        return IME_OK; // fail-open: len âm là dữ liệu bất định → 0 issue
    }
    let units = unsafe { slice::from_raw_parts(text_utf16, len as usize) };
    let text = match guarded(|| utf16_to_string(units)) {
        Some(Ok(text)) => text,
        _ => return IME_OK, // fail-open: UTF-16 lệch → 0 issue, không panic
    };
    let count = match catch_unwind(AssertUnwindSafe(|| {
        let issues = textvn_assist::check_sentence(&text);
        fill_assist_result(&issues, out);
        issues.len().min(IME_MAX_ASSIST_ISSUES)
    })) {
        Ok(issue_count) => issue_count,
        Err(_) => {
            out.count = 0; // fail-open: mọi panic → 0 issue (S4)
            0
        }
    };
    count as i32
}

/// Như [`ime_assist_check_v1`] nhưng nhận **provider từ điển OS** tùy chọn
/// (D39 · P4-10 §5.2): `provider == NULL` ⇒ đường `NoDictionary` như cũ; ABI
/// lệch ⇒ `IME_ERR_ABI`. Provider chỉ mở khóa luật `spelling` (chỉ-gợi ý,
/// confidence 60 — A1/D12: không tự sửa) trên từ tiếng Việt có dấu — mọi gate
/// FPR khác nằm trong luật + adapter (cổng capability `vi-VN`, W2).
#[no_mangle]
pub extern "C" fn ime_assist_check_v1_with(
    text_utf16: *const u16,
    len: i32,
    provider: *const ime_os_spell_v1,
    out: *mut ime_assist_result_v1,
) -> i32 {
    if out.is_null() {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { &mut *out };
    if out.abi_version != IME_ABI_VERSION {
        out.count = 0;
        return IME_ERR_ABI;
    }
    if text_utf16.is_null() {
        return if len == 0 {
            out.count = 0;
            IME_OK
        } else {
            IME_ERR_INVALID_ARG
        };
    }
    out.count = 0;
    if len < 0 {
        return IME_OK; // fail-open: len âm là dữ liệu bất định → 0 issue
    }
    let units = unsafe { slice::from_raw_parts(text_utf16, len as usize) };
    let text = match guarded(|| utf16_to_string(units)) {
        Some(Ok(text)) => text,
        _ => return IME_OK, // fail-open: UTF-16 lệch → 0 issue, không panic
    };
    // Provider hỏng (ABI lệch / trỏ tới dữ liệu rác) → im lặng theo provider,
    // KHÔNG làm chết đường check chính: `CSpellProvider` fail-open mọi callback.
    let provider = if provider.is_null() {
        None
    } else {
        let p = unsafe { *provider };
        if p.abi_version != IME_ABI_VERSION {
            return IME_ERR_ABI;
        }
        Some(CSpellProvider::new(p))
    };
    let count = match catch_unwind(AssertUnwindSafe(|| {
        let issues = match provider {
            Some(ref p) => textvn_assist::check_sentence_with(&text, p),
            None => textvn_assist::check_sentence(&text),
        };
        fill_assist_result(&issues, out);
        issues.len().min(IME_MAX_ASSIST_ISSUES)
    })) {
        Ok(issue_count) => issue_count,
        Err(_) => {
            out.count = 0; // fail-open: mọi panic → 0 issue (S4)
            0
        }
    };
    count as i32
}

/// Lấy **phần đuôi câu** của chuỗi UTF-16, tối đa `max_chars` ký tự Unicode
/// (phần tử cuối của `Engine::sentence_tail` — cho adapter dùng làm "câu vừa gõ").
///
/// Ghi dạng UTF-16 kèm **NUL kết thúc** vào `out_utf16` (cap = `out_cap` code unit,
/// chừa 1 unit cho NUL); trả về **số code unit đã ghi** (không tính NUL).
/// Cắt luôn tại **ranh giới ký tự**: không bao giờ ở giữa surrogate pair (astral).
/// Lỗi caller (NULL out / cap ≤ 0 / `text` NULL với `len != 0`) → `IME_ERR_*`;
/// lỗi dữ liệu (UTF-16 lệch, panic) → trả 0 (fail-open, không panic). Không log câu (S2).
#[no_mangle]
pub extern "C" fn ime_assist_sentence_tail_v1(
    text_utf16: *const u16,
    len: i32,
    max_chars: i32,
    out_utf16: *mut u16,
    out_cap: i32,
) -> i32 {
    if out_utf16.is_null() || out_cap <= 0 {
        return IME_ERR_INVALID_ARG;
    }
    let out = unsafe { slice::from_raw_parts_mut(out_utf16, out_cap as usize) };
    if text_utf16.is_null() {
        return if len == 0 {
            out[0] = 0;
            0
        } else {
            IME_ERR_INVALID_ARG
        };
    }
    out[0] = 0;
    if len < 0 || max_chars <= 0 {
        return 0; // fail-open: dữ liệu bất định → đuôi rỗng
    }
    let units = unsafe { slice::from_raw_parts(text_utf16, len as usize) };
    let max_chars = max_chars as usize;
    let cap_units = out_cap.saturating_sub(1) as usize; // chừa NUL
    match catch_unwind(AssertUnwindSafe(|| {
        let text = match utf16_to_string(units) {
            Ok(text) => text,
            // fail-open: UTF-16 lệch → đuôi rỗng, không panic
            Err(_) => return 0usize,
        };
        let chars: Vec<char> = text.chars().collect();
        let take = max_chars.min(chars.len());
        // Đi TỪ CUỐI về đầu (đuôi câu ưu tiên, đúng chiều cắt) chỉ để TÌM viền
        // cắt; sau đó encode XUÔI để cặp surrogate giữ đúng thứ tự D83C-DF9C —
        // đóng gói ngược rồi reverse() sẽ chẻ nối D83C-DF9C thành DF9C-D83C.
        let mut start = chars.len();
        let mut acc = 0usize;
        for &ch in chars[(chars.len() - take)..].iter().rev() {
            let cost = ch.len_utf16();
            if acc + cost > cap_units {
                break;
            }
            acc += cost;
            start -= 1;
        }
        let tail: String = chars[start..].iter().collect();
        let packed: Vec<u16> = tail.encode_utf16().collect();
        let end = packed.len();
        out[..end].copy_from_slice(&packed);
        out[end] = 0;
        end
    })) {
        Ok(written) => written as i32,
        Err(_) => {
            out[0] = 0; // fail-open: mọi panic → đuôi rỗng
            0
        }
    }
}

/// Một lỗi "thô" đầu vào cho kế hoạch sửa câu — **biên chung** của
/// [`ime_assist_result_v1`] (FFI) và `IssueView` (Windows TSF).
///
/// R-7 (G15 — một quyết định, một nơi): toàn bộ logic "sửa kiểu gì, xoá đâu"
/// chỉ còn **MỘT** bản sống trong [`plan_issues_from_raw`]; macOS/Linux đi qua
/// FFI (`ime_assist_plan_sentence_fix`), Windows đi qua hàm này rồi tự bọc
/// [`textvn_assist::plan_sentence_fix`]. Trước đây logic này chép ở 2 nơi
/// (ffi + windows-tsf) ⇒ sớm muộn lệch nhau (và đã lệch lần: Q-2 heuristic).
#[derive(Debug, Clone)]
pub struct PlanIssueRaw {
    /// `IME_ASSIST_KIND_*` — quyết định `Fix::Delete` hay `Fix::Replace`.
    pub kind: u32,
    /// Span trong câu (char Unicode, inclusive-exclusive).
    pub start: usize,
    pub end: usize,
    /// Chỉ lỗi auto-fixable mới vào kế hoạch.
    pub auto_fixable: bool,
    pub confidence: u8,
    /// Gợi ý đầu tiên (rỗng = không có).
    pub suggest: String,
}

/// Cờ của [`ime_assist_plan_sentence_fix_ex`]: kế hoạch cho hành động **người
/// dùng chủ động** (nút "Sửa tất cả", phím tắt, chọn một gợi ý) — gồm cả lỗi
/// chỉ-gợi-ý có cách sửa máy móc ([`explicit_fix_text`]). Không cờ = Mode 2 tự
/// sửa: chỉ lỗi objective (`auto_fixable`).
pub const IME_ASSIST_PLAN_EXPLICIT: u32 = 1;

/// Chữ thay span khi **người dùng chủ động** sửa một lỗi chỉ-gợi-ý (R2-131).
///
/// Chỉ loại có cách sửa máy móc: chính tả (gợi ý đầu = từ thay thế) và dấu phẩy
/// cuối câu (thay bằng dấu chấm — gợi ý của luật là câu hướng dẫn, không phải
/// chữ thay). Phủ định kép / câu lặp cần viết lại ⇒ `None` (chỉ hiển thị).
pub fn explicit_fix_text(kind: u32, suggest: &str) -> Option<String> {
    match kind {
        IME_ASSIST_KIND_SPELLING => (!suggest.is_empty() && !suggest.chars().any(char::is_control))
            .then(|| suggest.to_string()),
        IME_ASSIST_KIND_COMMA_AT_END => Some(".".to_string()),
        _ => None,
    }
}

/// Quy đổi lỗi thô → list [`textvn_assist::Issue`] kèm `Fix` — **nguồn duy nhất**
/// của ngữ nghĩa sửa (theo `kind`, KHÔNG đoán bằng text — Q-2).
pub fn plan_issues_from_raw(sentence: &str, raws: &[PlanIssueRaw]) -> Vec<textvn_assist::Issue> {
    plan_issues_from_raw_ex(sentence, raws, false)
}

/// Như [`plan_issues_from_raw`]; `explicit` = hành động người dùng chủ động
/// (R2-131): lỗi chỉ-gợi-ý có [`explicit_fix_text`] cũng vào kế hoạch.
pub fn plan_issues_from_raw_ex(
    sentence: &str,
    raws: &[PlanIssueRaw],
    explicit: bool,
) -> Vec<textvn_assist::Issue> {
    let chars: Vec<char> = sentence.chars().collect();
    let mut issues = Vec::new();

    for raw in raws {
        if !raw.auto_fixable {
            if !explicit {
                continue;
            }
            let (start, end) = (raw.start, raw.end);
            if start >= end || end > chars.len() {
                continue;
            }
            let Some(text) = explicit_fix_text(raw.kind, &raw.suggest) else {
                continue;
            };
            if chars[start..end].iter().copied().eq(text.chars()) {
                continue;
            }
            let kind = if raw.kind == IME_ASSIST_KIND_COMMA_AT_END {
                textvn_assist::IssueKind::CommaAtEnd
            } else {
                textvn_assist::IssueKind::Spelling
            };
            issues.push(textvn_assist::Issue {
                kind,
                span: textvn_assist::Span { start, end },
                message: String::new(),
                suggestions: vec![text.clone()],
                auto_fixable: true,
                confidence: raw.confidence,
                fix: Some(textvn_assist::Fix::Replace { start, end, text }),
            });
            continue;
        }
        let start = raw.start;
        let end = raw.end;
        if start >= end || end > chars.len() {
            continue;
        }
        let suggest_str = &raw.suggest;
        if suggest_str.chars().any(char::is_control) {
            continue;
        }

        let issue_kind = match raw.kind {
            IME_ASSIST_KIND_DUPLICATE_WORD => textvn_assist::IssueKind::DuplicateWord,
            IME_ASSIST_KIND_SPACE_BEFORE_PUNCT => textvn_assist::IssueKind::SpaceBeforePunct,
            IME_ASSIST_KIND_SPACE_AFTER_PUNCT => textvn_assist::IssueKind::SpaceAfterPunct,
            IME_ASSIST_KIND_DOUBLE_SPACE => textvn_assist::IssueKind::DoubleSpace,
            IME_ASSIST_KIND_LOWERCASE_SENTENCE => textvn_assist::IssueKind::LowercaseSentenceStart,
            IME_ASSIST_KIND_DIACRITIC_TYPO => textvn_assist::IssueKind::DiacriticTypo,
            IME_ASSIST_KIND_SENTENCE_REPEAT => textvn_assist::IssueKind::SentenceRepeat,
            IME_ASSIST_KIND_DOUBLE_NEGATIVE => textvn_assist::IssueKind::DoubleNegative,
            IME_ASSIST_KIND_COMMA_AT_END => textvn_assist::IssueKind::CommaAtEnd,
            IME_ASSIST_KIND_SPELLING => textvn_assist::IssueKind::Spelling,
            IME_ASSIST_KIND_ARTICLE_AN => textvn_assist::IssueKind::ArticleAn,
            _ => continue,
        };

        let fix =
            if raw.kind == IME_ASSIST_KIND_DUPLICATE_WORD && start > 0 && chars[start - 1] == ' ' {
                Some(textvn_assist::Fix::Delete {
                    start: start - 1,
                    end,
                })
            } else if raw.kind == IME_ASSIST_KIND_DUPLICATE_WORD
                || raw.kind == IME_ASSIST_KIND_SPACE_BEFORE_PUNCT
            {
                Some(textvn_assist::Fix::Delete { start, end })
            } else if raw.kind == IME_ASSIST_KIND_DOUBLE_SPACE {
                if start > 0 && chars[start - 1] == ' ' {
                    Some(textvn_assist::Fix::Delete { start, end })
                } else {
                    Some(textvn_assist::Fix::Replace {
                        start,
                        end,
                        text: " ".to_string(),
                    })
                }
            } else if raw.kind == IME_ASSIST_KIND_SPACE_AFTER_PUNCT {
                Some(textvn_assist::Fix::Insert {
                    at: end,
                    text: " ".to_string(),
                })
            } else {
                let span_text: String = chars[start..end].iter().collect();
                if span_text == *suggest_str {
                    None
                } else {
                    Some(textvn_assist::Fix::Replace {
                        start,
                        end,
                        text: suggest_str.to_string(),
                    })
                }
            };

        let Some(fix) = fix else {
            continue;
        };

        issues.push(textvn_assist::Issue {
            kind: issue_kind,
            span: textvn_assist::Span { start, end },
            message: String::new(),
            suggestions: vec![suggest_str.to_string()],
            auto_fixable: true,
            confidence: raw.confidence,
            fix: Some(fix),
        });
    }

    issues
}

/// Tính kế hoạch tự sửa cả câu cho các lỗi objective (G15 / AS-050 · D12 · D22 · D27).
///
/// Bọc trong catch_unwind (P0-2 §5).
/// Trả về 1 nếu có kế hoạch hợp lệ ghi vào out_plan, 0 nếu không có hoặc lỗi.
#[no_mangle]
pub extern "C" fn ime_assist_plan_sentence_fix(
    inst: *const ime_instance,
    sentence_utf8: *const c_char,
    result: *const ime_assist_result_v1,
    out_plan: *mut ime_assist_plan_v1,
) -> i32 {
    ime_assist_plan_sentence_fix_ex(inst, sentence_utf8, result, 0, out_plan)
}

/// Như [`ime_assist_plan_sentence_fix`] kèm `flags`: [`IME_ASSIST_PLAN_EXPLICIT`]
/// = kế hoạch cho hành động người dùng chủ động (R2-131 — nút "Sửa tất cả",
/// phím tắt sửa câu, chọn một gợi ý): gồm cả lỗi chính tả có gợi ý.
#[no_mangle]
pub extern "C" fn ime_assist_plan_sentence_fix_ex(
    inst: *const ime_instance,
    sentence_utf8: *const c_char,
    result: *const ime_assist_result_v1,
    flags: u32,
    out_plan: *mut ime_assist_plan_v1,
) -> i32 {
    if sentence_utf8.is_null() || result.is_null() || out_plan.is_null() {
        return 0;
    }
    let res = unsafe { &*result };
    if res.abi_version != IME_ABI_VERSION {
        return 0;
    }
    let out = unsafe { &mut *out_plan };
    out.abi_version = IME_ABI_VERSION;

    catch_unwind(AssertUnwindSafe(|| {
        let c_str = unsafe { CStr::from_ptr(sentence_utf8) };
        let sentence = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        if sentence.is_empty() {
            return 0;
        }

        // Quy đổi C-ABI → biên chung (R-7): toàn bộ quyết định sửa nằm trong
        // [`plan_issues_from_raw`] — một bản cho cả 3 OS.
        let count = (res.count as usize).min(res.issues.len());
        let raws: Vec<PlanIssueRaw> = res.issues[..count]
            .iter()
            .map(|raw| {
                let suggest = unsafe { CStr::from_ptr(raw.suggest.as_ptr()) }
                    .to_str()
                    .unwrap_or("")
                    .to_string();
                PlanIssueRaw {
                    kind: raw.kind,
                    start: raw.start as usize,
                    end: raw.end as usize,
                    auto_fixable: raw.auto_fixable != 0,
                    confidence: raw.confidence.min(100) as u8,
                    suggest,
                }
            })
            .collect();
        let explicit = flags & IME_ASSIST_PLAN_EXPLICIT != 0;
        let issues = plan_issues_from_raw_ex(sentence, &raws, explicit);

        let Some(plan) = textvn_assist::plan_sentence_fix(sentence, &issues) else {
            return 0;
        };

        if plan.replacement.len() >= IME_MAX_SENTENCE_FIX_CHARS
            || plan.original_tail.len() >= IME_MAX_SENTENCE_FIX_CHARS
        {
            return 0;
        }

        let tail_chars: Vec<char> = plan.original_tail.chars().collect();
        let glyphs = if !inst.is_null() {
            let inst_ref = unsafe { &*inst };
            inst_ref.engine.glyph_len_of(&tail_chars) as u32
        } else {
            plan.delete_chars as u32
        };

        if glyphs == 0 {
            return 0;
        }

        // Audit vòng 1 (tháng 10-2026): số glyph cần xoá phải **trùng** số char
        // Unicode của phần bị xoá. Khi bảng mã xuất vẫn là Unicode thì luôn khớp;
        // bảng mã khác (VNI/TCVN3: 1 chữ Việt = 2 glyph) ⇒ Linux xoá theo CHAR sẽ
        // xoá thừa ⇒ **không có kế hoạch** (fail-safe — cùng triết lý với
        // `verify_delete_range` của Windows chặn ở biên document).
        if glyphs as usize != tail_chars.len() {
            return 0;
        }

        out.fixed_count = plan.fixed_count as u32;
        out.delete_from = plan.delete_from as u32;
        out.delete_chars = plan.delete_chars as u32;
        out.delete_glyphs = glyphs;
        copy_str_to_c_buf(&plan.replacement, &mut out.replacement);
        copy_str_to_c_buf(&plan.original_tail, &mut out.original_tail);

        1
    }))
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    // ---- P0-2 §6: CI enforce kích thước & thứ tự field ----
    #[test]
    fn abi_sizes_match_contract() {
        assert_eq!(size_of::<ime_result_v1>(), 532);
        assert_eq!(size_of::<ime_key_v1>(), 20);
    }

    #[test]
    fn abi_result_field_offsets() {
        assert_eq!(offset_of!(ime_result_v1, abi_version), 0);
        assert_eq!(offset_of!(ime_result_v1, action), 4);
        assert_eq!(offset_of!(ime_result_v1, delete_count), 8);
        assert_eq!(offset_of!(ime_result_v1, insert_len), 10);
        assert_eq!(offset_of!(ime_result_v1, preedit_len), 12);
        assert_eq!(offset_of!(ime_result_v1, _reserved), 14);
        assert_eq!(offset_of!(ime_result_v1, flags), 16);
        assert_eq!(offset_of!(ime_result_v1, insert), 20);
        assert_eq!(offset_of!(ime_result_v1, preedit), 20 + 64 * 4);
    }

    #[test]
    fn abi_key_field_offsets() {
        assert_eq!(offset_of!(ime_key_v1, abi_version), 0);
        assert_eq!(offset_of!(ime_key_v1, vk), 4);
        assert_eq!(offset_of!(ime_key_v1, ch), 8);
        assert_eq!(offset_of!(ime_key_v1, mods), 12);
        assert_eq!(offset_of!(ime_key_v1, key_down), 16);
        assert_eq!(offset_of!(ime_key_v1, is_repeat), 17);
        assert_eq!(offset_of!(ime_key_v1, is_injected), 18);
        assert_eq!(offset_of!(ime_key_v1, _reserved), 19);
    }

    fn new_default() -> *mut ime_instance {
        let mut inst: *mut ime_instance = std::ptr::null_mut();
        let rc = ime_instance_new(std::ptr::null(), 0, &mut inst);
        assert_eq!(rc, IME_OK);
        assert!(!inst.is_null());
        inst
    }

    fn key_char(inst: *mut ime_instance, c: char) -> ime_result_v1 {
        let k = ime_key_v1 {
            abi_version: IME_ABI_VERSION,
            vk: 0,
            ch: c as u32,
            mods: 0,
            key_down: 1,
            is_repeat: 0,
            is_injected: 0,
            _reserved: 0,
        };
        let mut out = ime_result_v1::zeroed_pass();
        let rc = ime_key(inst, &k, &mut out);
        assert_eq!(rc, IME_OK);
        out
    }

    fn result_text(r: &ime_result_v1) -> String {
        r.insert[..r.insert_len as usize]
            .iter()
            .filter_map(|&u| char::from_u32(u))
            .collect()
    }

    fn resolve_context(app_id: *const c_char, field_role: u32, caps: u32) -> ime_context_v1 {
        ime_context_v1 {
            abi_version: IME_ABI_VERSION,
            enabled: 1,
            secure: 0,
            field_role,
            caps,
            app_id,
            element_name: std::ptr::null(),
            hint: -1,
        }
    }

    #[test]
    fn flow_new_key_reset_free() {
        let inst = new_default();
        let mut buf: Vec<char> = Vec::new();
        for c in "dduocj".chars() {
            let r = key_char(inst, c);
            match r.action {
                ACTION_PASS => buf.push(c),
                ACTION_REPLACE => {
                    buf.truncate(buf.len() - r.delete_count as usize);
                    buf.extend(result_text(&r).chars());
                }
                other => panic!("unexpected action {other}"),
            }
        }
        assert_eq!(buf.into_iter().collect::<String>(), "được");
        assert_eq!(ime_reset(inst), IME_OK);
        ime_instance_free(inst);
        ime_instance_free(std::ptr::null_mut()); // no-op
    }

    /// Lưới an toàn cuối: kết quả vượt `IME_MAX_TEXT` bị TỪ CHỐI (adapter nhận PASS +
    /// ERROR) chứ không cắt bớt — cắt làm lệch số ký tự engine sở hữu.
    #[test]
    fn oversized_outcome_is_rejected_not_truncated() {
        let mut r = ime_result_v1::zeroed_pass();
        let too_long = Outcome {
            action: Action::Replace {
                delete_count: 1,
                insert: vec!['a'; IME_MAX_TEXT + 1],
            },
            preedit: Vec::new(),
            flags: 0,
        };
        assert!(!fill_result(&mut r, &too_long));
        let fits = Outcome {
            action: Action::Replace {
                delete_count: 1,
                insert: vec!['a'; IME_MAX_TEXT],
            },
            preedit: vec!['a'; IME_MAX_TEXT],
            flags: 0,
        };
        assert!(fill_result(&mut r, &fits));
        assert_eq!(usize::from(r.insert_len), IME_MAX_TEXT);
        assert_eq!(usize::from(r.preedit_len), IME_MAX_TEXT);
    }

    /// `dej` + giữ `p`: mọi kết quả qua ABI thật vừa 64 ký tự và không bao giờ xoá
    /// nhiều hơn phần đã gõ.
    #[test]
    fn held_key_long_word_never_deletes_past_typed_text() {
        let inst = new_default();
        let mut buf: Vec<char> = Vec::new();
        for c in format!("dej{}", "p".repeat(100)).chars() {
            let r = key_char(inst, c);
            assert_eq!(r.flags & IME_FLAG_ERROR, 0);
            match r.action {
                ACTION_PASS => buf.push(c),
                ACTION_REPLACE | ACTION_COMMIT => {
                    let d = usize::from(r.delete_count);
                    assert!(d <= buf.len(), "delete {d} > {}", buf.len());
                    buf.truncate(buf.len() - d);
                    buf.extend(result_text(&r).chars());
                }
                other => panic!("unexpected action {other}"),
            }
        }
        assert_eq!(
            buf.into_iter().collect::<String>(),
            format!("dẹ{}", "p".repeat(100))
        );
        ime_instance_free(inst);
    }

    #[test]
    fn invalid_config_nonfatal_with_default_instance() {
        let cfg = b"{\"method\":\"dvorak\"}";
        let mut inst: *mut ime_instance = std::ptr::null_mut();
        let rc = ime_instance_new(cfg.as_ptr(), cfg.len(), &mut inst);
        assert_eq!(rc, IME_ERR_CONFIG);
        assert!(!inst.is_null(), "instance vẫn phải tạo (F0-015)");
        let err = unsafe { std::ffi::CStr::from_ptr(ime_last_error(inst)) };
        assert_eq!(err.to_str().unwrap(), "config: invalid schema");
        let r = key_char(inst, 'x');
        assert_eq!(r.action, ACTION_PASS);
        ime_instance_free(inst);
    }

    #[test]
    fn null_args_and_abi_errors_fail_open() {
        let mut out = ime_result_v1::zeroed_pass();
        let rc = ime_key(std::ptr::null_mut(), std::ptr::null(), &mut out);
        assert_eq!(rc, IME_ERR_INVALID_ARG);
        assert_eq!(out.action, ACTION_PASS);
        assert_ne!(out.flags & IME_FLAG_ERROR, 0);

        let inst = new_default();
        let mut k = ime_key_v1 {
            abi_version: 99,
            vk: 0,
            ch: 'a' as u32,
            mods: 0,
            key_down: 1,
            is_repeat: 0,
            is_injected: 0,
            _reserved: 0,
        };
        let mut out = ime_result_v1::zeroed_pass();
        assert_eq!(ime_key(inst, &k, &mut out), IME_ERR_ABI);
        assert_eq!(out.action, ACTION_PASS);

        let ctx = ime_context_v1 {
            abi_version: 99,
            enabled: 1,
            secure: 0,
            field_role: 1,
            caps: 0,
            app_id: std::ptr::null(),
            element_name: std::ptr::null(),
            hint: -1,
        };
        assert_eq!(ime_set_context(inst, &ctx), IME_ERR_ABI);
        assert_eq!(ime_set_context(inst, std::ptr::null()), IME_ERR_INVALID_ARG);

        k.abi_version = IME_ABI_VERSION;
        let _ = k;
        ime_instance_free(inst);
    }

    #[test]
    fn reload_config_keeps_old_on_error() {
        let inst = new_default();
        let bad = b"{\"config_version\":9}";
        assert_eq!(
            ime_reload_config(inst, bad.as_ptr(), bad.len()),
            IME_ERR_CONFIG
        );
        let r = key_char(inst, 'a');
        assert_eq!(r.action, ACTION_PASS);
        ime_instance_free(inst);
    }

    #[test]
    fn suggest_feature_disabled() {
        let inst = new_default();
        let mut s = ime_suggest_v1 {
            abi_version: 0,
            count: 0,
            lens: [0; IME_MAX_SUGGEST],
            items: [[0; IME_MAX_TEXT]; IME_MAX_SUGGEST],
        };
        assert_eq!(
            ime_suggest(inst, std::ptr::null(), 0, &mut s),
            IME_ERR_INTERNAL
        );
        assert_eq!(s.count, 0);
        ime_instance_free(inst);
    }

    #[test]
    fn strategy_resolve_uses_appdb_and_applies_gates() {
        let app_id = c"chrome.exe";
        let db = br#"{
          "appdb_version": 1,
          "entries": [{
            "match": {"exe": "chrome.exe"},
            "when": {"field_role": "address_bar"},
            "strategy": "SelectionReplace"
          }]
        }"#;
        let mut ctx = resolve_context(
            app_id.as_ptr(),
            textvn_strategy::IME_FIELD_ADDRESS_BAR,
            textvn_strategy::IME_CAP_SELECTION,
        );
        let mut strategy = -1;
        assert_eq!(
            ime_strategy_resolve(&ctx, db.as_ptr(), db.len(), &mut strategy),
            IME_OK
        );
        assert_eq!(strategy, textvn_strategy::IME_STRATEGY_SELECTION_REPLACE);

        ctx.secure = 1;
        assert_eq!(
            ime_strategy_resolve(&ctx, db.as_ptr(), db.len(), &mut strategy),
            IME_OK
        );
        assert_eq!(strategy, textvn_strategy::IME_STRATEGY_PASSTHROUGH);
    }

    #[test]
    fn strategy_resolve_rejects_bad_input_with_safe_output() {
        let mut strategy = -1;
        let ctx = resolve_context(
            std::ptr::null(),
            textvn_strategy::IME_FIELD_BODY,
            textvn_strategy::IME_CAP_PREEDIT,
        );
        let malformed = b"{}";
        assert_eq!(
            ime_strategy_resolve(&ctx, malformed.as_ptr(), malformed.len(), &mut strategy),
            IME_ERR_CONFIG
        );
        assert_eq!(strategy, textvn_strategy::IME_STRATEGY_PASSTHROUGH);
        assert_eq!(
            ime_strategy_resolve(&ctx, std::ptr::null(), 1, &mut strategy),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(strategy, textvn_strategy::IME_STRATEGY_PASSTHROUGH);
    }

    #[test]
    fn appdb_verify_is_fail_closed_without_release_public_key() {
        let db = br#"{"appdb_version":1,"entries":[]}"#;
        let signature = [0_u8; 64];
        assert_eq!(
            ime_appdb_verify(db.as_ptr(), db.len(), signature.as_ptr(), signature.len()),
            IME_ERR_INTERNAL
        );
        assert_eq!(
            ime_appdb_verify(std::ptr::null(), 0, signature.as_ptr(), signature.len()),
            IME_ERR_INVALID_ARG
        );
    }

    #[test]
    fn assist_abi_sizes_and_offsets() {
        assert_eq!(size_of::<ime_assist_issue_v1>(), 216);
        assert_eq!(size_of::<ime_assist_result_v1>(), 8 + 8 * 216);
        assert_eq!(
            size_of::<ime_assist_plan_v1>(),
            20 + 2 * IME_MAX_SENTENCE_FIX_CHARS
        );

        assert_eq!(offset_of!(ime_assist_issue_v1, kind), 0);
        assert_eq!(offset_of!(ime_assist_issue_v1, start), 4);
        assert_eq!(offset_of!(ime_assist_issue_v1, end), 8);
        assert_eq!(offset_of!(ime_assist_issue_v1, auto_fixable), 12);
        assert_eq!(offset_of!(ime_assist_issue_v1, confidence), 16);
        assert_eq!(offset_of!(ime_assist_issue_v1, message), 20);
        assert_eq!(offset_of!(ime_assist_issue_v1, suggest_len), 148);
        assert_eq!(offset_of!(ime_assist_issue_v1, suggest), 152);

        assert_eq!(offset_of!(ime_assist_result_v1, abi_version), 0);
        assert_eq!(offset_of!(ime_assist_result_v1, count), 4);
        assert_eq!(offset_of!(ime_assist_result_v1, issues), 8);

        assert_eq!(offset_of!(ime_assist_plan_v1, abi_version), 0);
        assert_eq!(offset_of!(ime_assist_plan_v1, fixed_count), 4);
        assert_eq!(offset_of!(ime_assist_plan_v1, delete_from), 8);
        assert_eq!(offset_of!(ime_assist_plan_v1, delete_chars), 12);
        assert_eq!(offset_of!(ime_assist_plan_v1, delete_glyphs), 16);
        assert_eq!(offset_of!(ime_assist_plan_v1, replacement), 20);
        assert_eq!(
            offset_of!(ime_assist_plan_v1, original_tail),
            20 + IME_MAX_SENTENCE_FIX_CHARS
        );
    }

    #[test]
    fn assist_trigger_kind_classification() {
        assert_eq!(
            ime_assist_trigger_kind('.' as u32),
            IME_ASSIST_TRIGGER_SENTENCE
        );
        assert_eq!(
            ime_assist_trigger_kind('!' as u32),
            IME_ASSIST_TRIGGER_SENTENCE
        );
        assert_eq!(
            ime_assist_trigger_kind('?' as u32),
            IME_ASSIST_TRIGGER_SENTENCE
        );

        assert_eq!(ime_assist_trigger_kind(' ' as u32), IME_ASSIST_TRIGGER_WORD);
        assert_eq!(ime_assist_trigger_kind(',' as u32), IME_ASSIST_TRIGGER_WORD);
        assert_eq!(ime_assist_trigger_kind(';' as u32), IME_ASSIST_TRIGGER_WORD);
        assert_eq!(ime_assist_trigger_kind(':' as u32), IME_ASSIST_TRIGGER_WORD);

        assert_eq!(ime_assist_trigger_kind('a' as u32), IME_ASSIST_TRIGGER_NONE);
        assert_eq!(ime_assist_trigger_kind('z' as u32), IME_ASSIST_TRIGGER_NONE);
        assert_eq!(ime_assist_trigger_kind(0x110000), IME_ASSIST_TRIGGER_NONE);
    }

    #[test]
    fn assist_check_finds_vietnamese_issue() {
        let sample = "Tôi tôi đi học";
        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        let code = ime_assist_check(sample.as_ptr() as *const c_char, sample.len(), &mut res);
        assert_eq!(code, IME_OK);
        assert_eq!(res.count, 1);
        let issue = &res.issues[0];
        assert_eq!(issue.kind, IME_ASSIST_KIND_DUPLICATE_WORD);
        assert_eq!(issue.start, 4);
        assert_eq!(issue.end, 7);
        assert_eq!(issue.auto_fixable, 1);
        assert_eq!(issue.confidence, 100);
        let msg = unsafe { CStr::from_ptr(issue.message.as_ptr()) }
            .to_str()
            .unwrap();
        assert!(msg.contains("Lặp từ"));
        let sugg = unsafe { CStr::from_ptr(issue.suggest.as_ptr()) }
            .to_str()
            .unwrap();
        assert_eq!(sugg, "Tôi");
    }

    #[test]
    fn assist_check_ignores_clean_or_english_text() {
        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        let clean = "Tôi đi học ở trường Bách Khoa.";
        assert_eq!(
            ime_assist_check(clean.as_ptr() as *const c_char, clean.len(), &mut res),
            IME_OK
        );
        assert_eq!(res.count, 0);

        let english = "This is a clean English sentence.";
        assert_eq!(
            ime_assist_check(english.as_ptr() as *const c_char, english.len(), &mut res),
            IME_OK
        );
        assert_eq!(res.count, 0);
    }

    #[test]
    fn assist_check_instance_reads_sentence_buffer() {
        let inst = new_default();
        let _ = key_char(inst, 'T');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'i');
        let _ = key_char(inst, ' ');
        let _ = key_char(inst, 't');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'i');

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(ime_assist_check_instance(inst, &mut res), IME_OK);
        assert_eq!(res.count, 1);
        assert_eq!(res.issues[0].kind, IME_ASSIST_KIND_DUPLICATE_WORD);
        ime_instance_free(inst);
    }

    #[test]
    fn assist_sentence_tail_and_glyph_len() {
        let inst = new_default();
        let _ = key_char(inst, 'x');
        let _ = key_char(inst, 'i');
        let _ = key_char(inst, 'n');
        let _ = key_char(inst, ' ');
        let _ = key_char(inst, 'c');
        let _ = key_char(inst, 'h');
        let _ = key_char(inst, 'a');
        let _ = key_char(inst, 'o');

        let mut out_len = 0usize;
        assert_eq!(
            ime_assist_sentence_tail(inst, 8, std::ptr::null_mut(), 0, &mut out_len),
            IME_OK
        );
        assert_eq!(out_len, "xin chao".len());

        let mut buf = [0 as c_char; 32];
        assert_eq!(
            ime_assist_sentence_tail(inst, 8, buf.as_mut_ptr(), buf.len(), &mut out_len),
            IME_OK
        );
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap();
        assert_eq!(s, "xin chao");

        let word = "được";
        let mut glyphs = 0usize;
        assert_eq!(
            ime_assist_glyph_len(
                inst,
                word.as_ptr() as *const c_char,
                word.len(),
                &mut glyphs
            ),
            IME_OK
        );
        assert_eq!(glyphs, 4);

        ime_instance_free(inst);
    }

    #[test]
    fn assist_rejects_bad_args() {
        let mut res = ime_assist_result_v1 {
            abi_version: 999, // Bad ABI
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        let sample = "tôi tôi";
        assert_eq!(
            ime_assist_check(sample.as_ptr() as *const c_char, sample.len(), &mut res),
            IME_ERR_ABI
        );
        assert_eq!(
            ime_assist_check(
                sample.as_ptr() as *const c_char,
                sample.len(),
                std::ptr::null_mut()
            ),
            IME_ERR_INVALID_ARG
        );

        res.abi_version = IME_ABI_VERSION;
        assert_eq!(
            ime_assist_check_instance(std::ptr::null_mut(), &mut res),
            IME_ERR_INVALID_ARG
        );

        let mut len = 0usize;
        assert_eq!(
            ime_assist_sentence_tail(std::ptr::null(), 10, std::ptr::null_mut(), 0, &mut len),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_assist_glyph_len(
                std::ptr::null(),
                sample.as_ptr() as *const c_char,
                sample.len(),
                &mut len
            ),
            IME_ERR_INVALID_ARG
        );
    }

    fn fresh_assist_result() -> ime_assist_result_v1 {
        ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        }
    }

    /// D29: kiểm quy đổi offset trong câu tiếng Việt có ký tự astral —
    /// offset issue phải đếm theo CHAR Unicode (emoji astral = 2 code unit UTF-16 nhưng 1 char).
    #[test]
    fn assist_check_v1_utf16_offset_is_char_not_utf16() {
        let text = "Xin chào \u{1F600} tôi tôi nhé";
        let units: Vec<u16> = text.encode_utf16().collect();
        let char_len = text.chars().count();
        assert_ne!(char_len, units.len(), "cần chữ astral để lệch 2 đơn vị");

        let mut res = fresh_assist_result();
        let count = ime_assist_check_v1(units.as_ptr(), units.len() as i32, &mut res);
        assert_eq!(count, 1);
        assert_eq!(usize::try_from(count).unwrap(), res.count as usize);
        let issue = &res.issues[0];
        assert_eq!(issue.kind, IME_ASSIST_KIND_DUPLICATE_WORD);
        // "Xin chào 😀 " = 11 chars; từ "tôi" thứ hai đứng tại char 15..18
        // (nếu đếm UTF-16 sẽ là 16..19 → lệch 1 vì emoji chiếm 2 units).
        assert_eq!(issue.start, 15);
        assert_eq!(issue.end, 18);
    }

    /// D29: round-trip UTF-8 → UTF-16 giữ nguyên kết quả so với bản UTF-8.
    #[test]
    fn assist_check_v1_matches_utf8_path() {
        let sample = "Tôi tôi đi học";
        let units: Vec<u16> = sample.encode_utf16().collect();
        let mut via_utf16 = fresh_assist_result();
        assert_eq!(
            ime_assist_check_v1(units.as_ptr(), units.len() as i32, &mut via_utf16),
            1
        );
        let mut via_utf8 = fresh_assist_result();
        assert_eq!(
            ime_assist_check(
                sample.as_ptr() as *const c_char,
                sample.len(),
                &mut via_utf8
            ),
            IME_OK
        );
        assert_eq!(via_utf16.count, via_utf8.count);
        assert_eq!(via_utf16.issues[0].kind, via_utf8.issues[0].kind);
        assert_eq!(via_utf16.issues[0].start, via_utf8.issues[0].start);
    }

    /// Fail-open: input rỗng/null/UTF-16 lệch → 0 issue hoặc mã lỗi caller,
    /// tuyệt đối không panic.
    #[test]
    fn assist_check_v1_fail_open_on_empty_null_and_bad_utf16() {
        // text NULL + len 0 → 0 issue, IME_OK
        let mut res = fresh_assist_result();
        assert_eq!(ime_assist_check_v1(std::ptr::null(), 0, &mut res), IME_OK);
        assert_eq!(res.count, 0);

        // text NULL + len > 0 → lỗi caller (P0-2 §0: NULL là lỗi caller, không fail-open)
        res = fresh_assist_result();
        assert_eq!(
            ime_assist_check_v1(std::ptr::null(), 4, &mut res),
            IME_ERR_INVALID_ARG
        );

        // out NULL → lỗi caller
        let sample: Vec<u16> = "tôi tôi".encode_utf16().collect();
        assert_eq!(
            ime_assist_check_v1(sample.as_ptr(), sample.len() as i32, std::ptr::null_mut()),
            IME_ERR_INVALID_ARG
        );

        // abi lệch → IME_ERR_ABI
        res = fresh_assist_result();
        res.abi_version = 999;
        assert_eq!(
            ime_assist_check_v1(sample.as_ptr(), sample.len() as i32, &mut res),
            IME_ERR_ABI
        );
        assert_eq!(res.count, 0);

        // len âm → fail-open: 0 issue
        res = fresh_assist_result();
        assert_eq!(ime_assist_check_v1(sample.as_ptr(), -1, &mut res), IME_OK);
        assert_eq!(res.count, 0);

        // UTF-16 có unpaired surrogate → fail-open: 0 issue, không panic
        res = fresh_assist_result();
        let broken: [u16; 2] = [0x0041, 0xD800];
        assert_eq!(
            ime_assist_check_v1(broken.as_ptr(), broken.len() as i32, &mut res),
            IME_OK
        );
        assert_eq!(res.count, 0);
    }

    /// Cắt đuôi câu đếm theo CHAR Unicode, giữ nguyên surrogate pair của ký tự astral.
    #[test]
    fn assist_sentence_tail_v1_caps_in_chars_and_keeps_astral() {
        let text = "abc\u{1F600}def"; // 7 chars, 8 UTF-16 units
        let units: Vec<u16> = text.encode_utf16().collect();

        // max 4 chars → "😀 def": emoji (2 units) + "def" (3 units) = 5 units
        let mut out = [0_u16; 16];
        let written = ime_assist_sentence_tail_v1(
            units.as_ptr(),
            units.len() as i32,
            4,
            out.as_mut_ptr(),
            out.len() as i32,
        );
        assert_eq!(written, 5);
        assert_eq!(
            String::from_utf16_lossy(&out[..written as usize]),
            "\u{1F600}def"
        );
        assert_eq!(out[written as usize], 0, "phải NUL kết thúc");

        // max lớn hơn độ dài → nguyên văn toàn bộ (7 chars = 8 units)
        out = [0_u16; 16];
        let written = ime_assist_sentence_tail_v1(
            units.as_ptr(),
            units.len() as i32,
            100,
            out.as_mut_ptr(),
            out.len() as i32,
        );
        assert_eq!(written, 8);
        assert_eq!(String::from_utf16_lossy(&out[..written as usize]), text);

        // out_cap không đủ chỗ cho cả emoji → cắt ranh giới char (bỏ emoji), không chẻ pair
        let mut out4 = [0_u16; 4]; // cap 4: chừa NUL còn 1 unit — emoji cần 2 → chỉ lấy đuôi fits
        let written = ime_assist_sentence_tail_v1(
            units.as_ptr(),
            units.len() as i32,
            7,
            out4.as_mut_ptr(),
            out4.len() as i32,
        );
        assert_eq!(String::from_utf16_lossy(&out4[..written as usize]), "def");

        // input rỗng → 0, NUL an toàn
        let mut rets = [0u16; 4];
        assert_eq!(
            ime_assist_sentence_tail_v1(
                std::ptr::null(),
                0,
                8,
                rets.as_mut_ptr(),
                rets.len() as i32
            ),
            0
        );

        // UTF-16 lệch (surrogate cô đơn) → fail-open: 0
        let broken: [u16; 2] = [0xD800, 0x0041];
        assert_eq!(
            ime_assist_sentence_tail_v1(
                broken.as_ptr(),
                broken.len() as i32,
                8,
                rets.as_mut_ptr(),
                rets.len() as i32
            ),
            0
        );
        assert_eq!(rets[0], 0);

        // lỗi caller: out NULL / cap ≤ 0 → IME_ERR_INVALID_ARG
        let full: Vec<u16> = "xin chào".encode_utf16().collect();
        assert_eq!(
            ime_assist_sentence_tail_v1(
                full.as_ptr(),
                full.len() as i32,
                8,
                std::ptr::null_mut(),
                8
            ),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(
            ime_assist_sentence_tail_v1(full.as_ptr(), full.len() as i32, 8, out.as_mut_ptr(), 0),
            IME_ERR_INVALID_ARG
        );
    }

    unsafe extern "C" fn mock_contains(
        _ctx: *mut c_void,
        word: *const c_char,
        byte_len: u32,
    ) -> i32 {
        let slice = unsafe { std::slice::from_raw_parts(word as *const u8, byte_len as usize) };
        if let Ok(s) = std::str::from_utf8(slice) {
            // Giả lập từ "quẫn" bị coi là sai chính tả
            if s == "quẫn" {
                return 0;
            }
        }
        1
    }

    unsafe extern "C" fn mock_suggest(
        _ctx: *mut c_void,
        word: *const c_char,
        byte_len: u32,
        out: *mut c_char,
        out_len: u32,
        limit: u32,
    ) -> u32 {
        let slice = unsafe { std::slice::from_raw_parts(word as *const u8, byte_len as usize) };
        if let Ok(s) = std::str::from_utf8(slice) {
            if s == "quẫn" && limit >= 1 {
                let sugg = b"qu\xC3\xA2n\0"; // "quân" UTF-8
                if (out_len as usize) >= sugg.len() {
                    let out_slice =
                        unsafe { std::slice::from_raw_parts_mut(out as *mut u8, sugg.len()) };
                    out_slice.copy_from_slice(sugg);
                    return 1;
                }
            }
        }
        0
    }

    #[test]
    fn assist_set_spell_provider_and_check_instance() {
        let inst = new_default();

        assert_eq!(
            ime_assist_set_spell_provider(std::ptr::null_mut(), std::ptr::null()),
            IME_ERR_INVALID_ARG
        );

        let bad_provider = ime_os_spell_v1 {
            abi_version: 999,
            contains: Some(mock_contains),
            suggest: Some(mock_suggest),
            ctx: std::ptr::null_mut(),
        };
        assert_eq!(
            ime_assist_set_spell_provider(inst, &bad_provider),
            IME_ERR_ABI
        );

        let provider = ime_os_spell_v1 {
            abi_version: IME_ABI_VERSION,
            contains: Some(mock_contains),
            suggest: Some(mock_suggest),
            ctx: std::ptr::null_mut(),
        };
        assert_eq!(ime_assist_set_spell_provider(inst, &provider), IME_OK);

        // Gõ "Tôi quẫn "
        let _ = key_char(inst, 'T');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'i');
        let _ = key_char(inst, ' ');
        let _ = key_char(inst, 'q');
        let _ = key_char(inst, 'u');
        let _ = key_char(inst, 'a');
        let _ = key_char(inst, 'a');
        let _ = key_char(inst, 'n');
        let _ = key_char(inst, 'x');
        let _ = key_char(inst, ' ');

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(ime_assist_check_instance(inst, &mut res), IME_OK);
        assert_eq!(res.count, 1);
        assert_eq!(res.issues[0].kind, IME_ASSIST_KIND_SPELLING);
        let sugg = unsafe { CStr::from_ptr(res.issues[0].suggest.as_ptr()) }
            .to_str()
            .unwrap();
        assert_eq!(sugg, "quân");

        assert_eq!(
            ime_assist_set_spell_provider(inst, std::ptr::null()),
            IME_OK
        );
        let mut res2 = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(ime_assist_check_instance(inst, &mut res2), IME_OK);
        assert_eq!(res2.count, 0);

        ime_instance_free(inst);
    }

    unsafe extern "C" fn mock_error_contains(
        _ctx: *mut c_void,
        _word: *const c_char,
        _byte_len: u32,
    ) -> i32 {
        -1 // Mã lỗi / không rõ -> fail-open
    }

    #[test]
    fn assist_spell_provider_fail_open_on_error() {
        let inst = new_default();
        let error_provider = ime_os_spell_v1 {
            abi_version: IME_ABI_VERSION,
            contains: Some(mock_error_contains),
            suggest: None,
            ctx: std::ptr::null_mut(),
        };
        assert_eq!(ime_assist_set_spell_provider(inst, &error_provider), IME_OK);

        let _ = key_char(inst, 'T');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'o');
        let _ = key_char(inst, 'i');
        let _ = key_char(inst, ' ');
        let _ = key_char(inst, 'q');
        let _ = key_char(inst, 'u');
        let _ = key_char(inst, 'a');
        let _ = key_char(inst, 'a');
        let _ = key_char(inst, 'n');
        let _ = key_char(inst, 'x');
        let _ = key_char(inst, ' ');

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        // Mã lỗi -1 trong callback C -> fail-open: coi từ là đúng -> 0 issue
        assert_eq!(ime_assist_check_instance(inst, &mut res), IME_OK);
        assert_eq!(res.count, 0);

        ime_instance_free(inst);
    }

    /// Dựng 1 lỗi thô cho test kế hoạch (span char Unicode).
    fn raw_issue(
        kind: u32,
        start: u32,
        end: u32,
        auto_fixable: bool,
        suggest: &str,
    ) -> ime_assist_issue_v1 {
        let mut issue: ime_assist_issue_v1 = unsafe { std::mem::zeroed() };
        issue.kind = kind;
        issue.start = start;
        issue.end = end;
        issue.auto_fixable = u32::from(auto_fixable);
        issue.confidence = if auto_fixable { 100 } else { 60 };
        copy_str_to_c_buf(suggest, &mut issue.suggest);
        issue.suggest_len = suggest.len() as u32;
        issue
    }

    fn plan_text(plan: &ime_assist_plan_v1) -> String {
        unsafe { CStr::from_ptr(plan.replacement.as_ptr()) }
            .to_str()
            .unwrap()
            .to_string()
    }

    /// R2-131 (chủ repo: "chưa bấm được nút sửa tất cả"): lỗi chính tả là lỗi
    /// chỉ-gợi-ý — Mode 2 tự sửa KHÔNG đụng tới, nhưng hành động người dùng chủ
    /// động (nút/phím tắt/chọn gợi ý) phải sửa được cả câu.
    #[test]
    fn r2_131_explicit_plan_sua_ca_loi_chinh_ta_mode2_thi_khong() {
        let inst = new_default();
        let sentence = "ngành nghề nghành tôi";
        let sentence_c = CString::new(sentence).unwrap();
        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 1,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        res.issues[0] = raw_issue(IME_ASSIST_KIND_SPELLING, 11, 17, false, "ngành");

        let mut plan = ime_assist_plan_v1::zeroed();
        assert_eq!(
            ime_assist_plan_sentence_fix(inst, sentence_c.as_ptr(), &res, &mut plan),
            0,
            "Mode 2 (không cờ) không tự sửa lỗi chỉ-gợi-ý"
        );
        let mut plan = ime_assist_plan_v1::zeroed();
        assert_eq!(
            ime_assist_plan_sentence_fix_ex(
                inst,
                sentence_c.as_ptr(),
                &res,
                IME_ASSIST_PLAN_EXPLICIT,
                &mut plan
            ),
            1
        );
        assert_eq!(plan.delete_from, 11);
        assert_eq!(plan_text(&plan), "ngành tôi");
        ime_instance_free(inst);
    }

    /// R2-131: dấu phẩy cuối câu → dấu chấm (gợi ý của luật là câu hướng dẫn,
    /// không phải chữ thay); phủ định kép cần viết lại ⇒ không có kế hoạch;
    /// lỗi objective + chính tả cùng câu ⇒ sửa hết trong MỘT kế hoạch.
    #[test]
    fn r2_131_explicit_plan_dau_phay_cuoi_va_phu_dinh_kep() {
        let inst = new_default();
        let sentence = "tôi tôi đi họcc,";
        let sentence_c = CString::new(sentence).unwrap();
        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 3,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        res.issues[0] = raw_issue(IME_ASSIST_KIND_DUPLICATE_WORD, 4, 7, true, "tôi");
        res.issues[1] = raw_issue(IME_ASSIST_KIND_SPELLING, 11, 15, false, "học");
        res.issues[2] = raw_issue(
            IME_ASSIST_KIND_COMMA_AT_END,
            15,
            16,
            false,
            "Đổi thành dấu chấm câu",
        );
        let mut plan = ime_assist_plan_v1::zeroed();
        assert_eq!(
            ime_assist_plan_sentence_fix_ex(
                inst,
                sentence_c.as_ptr(),
                &res,
                IME_ASSIST_PLAN_EXPLICIT,
                &mut plan
            ),
            1
        );
        assert_eq!(plan.fixed_count, 3);
        assert_eq!(plan.delete_from, 3);
        assert_eq!(plan_text(&plan), " đi học.");

        res.count = 1;
        res.issues[0] = raw_issue(
            IME_ASSIST_KIND_DOUBLE_NEGATIVE,
            0,
            3,
            false,
            "Cân nhắc viết lại để tránh phủ định kép",
        );
        let mut plan = ime_assist_plan_v1::zeroed();
        assert_eq!(
            ime_assist_plan_sentence_fix_ex(
                inst,
                sentence_c.as_ptr(),
                &res,
                IME_ASSIST_PLAN_EXPLICIT,
                &mut plan
            ),
            0,
            "phủ định kép không có cách sửa máy móc"
        );
        assert_eq!(explicit_fix_text(IME_ASSIST_KIND_SPELLING, ""), None);
        ime_instance_free(inst);
    }

    #[test]
    fn assist_plan_sentence_fix_happy_path() {
        let inst = new_default();
        let sentence = "tôi tôi đi học.";
        let sentence_c = CString::new(sentence).unwrap();

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(
            ime_assist_check(sentence_c.as_ptr(), sentence.len(), &mut res),
            IME_OK
        );
        assert_eq!(res.count, 2);

        let mut plan = ime_assist_plan_v1::zeroed();
        let ok = ime_assist_plan_sentence_fix(inst, sentence_c.as_ptr(), &res, &mut plan);
        assert_eq!(ok, 1);
        assert_eq!(plan.abi_version, IME_ABI_VERSION);
        assert_eq!(plan.fixed_count, 2);
        assert_eq!(plan.delete_from, 0);
        assert_eq!(plan.delete_chars, sentence.chars().count() as u32);
        assert_eq!(plan.delete_glyphs, sentence.chars().count() as u32);

        let rep = unsafe { CStr::from_ptr(plan.replacement.as_ptr()) }
            .to_str()
            .unwrap();
        let tail = unsafe { CStr::from_ptr(plan.original_tail.as_ptr()) }
            .to_str()
            .unwrap();
        assert_eq!(rep, "Tôi đi học.");
        assert_eq!(tail, "tôi tôi đi học.");

        ime_instance_free(inst);
    }

    #[test]
    fn assist_plan_sentence_fix_tail_only_and_null_inst() {
        let inst = new_default();
        let sentence = "Tôi tôi đi học.";
        let sentence_c = CString::new(sentence).unwrap();

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(
            ime_assist_check(sentence_c.as_ptr(), sentence.len(), &mut res),
            IME_OK
        );
        assert_eq!(res.count, 1);

        // Test with valid inst
        let mut plan = ime_assist_plan_v1::zeroed();
        let ok = ime_assist_plan_sentence_fix(inst, sentence_c.as_ptr(), &res, &mut plan);
        assert_eq!(ok, 1);
        assert_eq!(plan.fixed_count, 1);
        assert_eq!(plan.delete_from, 3);
        assert_eq!(plan.delete_chars, 12);
        assert_eq!(plan.delete_glyphs, 12);

        let rep = unsafe { CStr::from_ptr(plan.replacement.as_ptr()) }
            .to_str()
            .unwrap();
        let tail = unsafe { CStr::from_ptr(plan.original_tail.as_ptr()) }
            .to_str()
            .unwrap();
        assert_eq!(rep, " đi học.");
        assert_eq!(tail, " tôi đi học.");

        // Test with NULL inst (fallback to char count)
        let mut plan_no_inst = ime_assist_plan_v1::zeroed();
        let ok2 = ime_assist_plan_sentence_fix(
            std::ptr::null(),
            sentence_c.as_ptr(),
            &res,
            &mut plan_no_inst,
        );
        assert_eq!(ok2, 1);
        assert_eq!(plan_no_inst.delete_glyphs, 12);

        ime_instance_free(inst);
    }

    #[test]
    fn assist_plan_sentence_fix_space_before_punct() {
        let sentence = "đi học .";
        let sentence_c = CString::new(sentence).unwrap();

        let mut res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        assert_eq!(
            ime_assist_check(sentence_c.as_ptr(), sentence.len(), &mut res),
            IME_OK
        );
        assert_eq!(res.count, 2); // lowercase sentence start + space before punct

        let mut plan = ime_assist_plan_v1::zeroed();
        let ok =
            ime_assist_plan_sentence_fix(std::ptr::null(), sentence_c.as_ptr(), &res, &mut plan);
        assert_eq!(ok, 1);
        assert_eq!(plan.fixed_count, 2);
        let rep = unsafe { CStr::from_ptr(plan.replacement.as_ptr()) }
            .to_str()
            .unwrap();
        assert_eq!(rep, "Đi học.");
    }

    #[test]
    fn assist_plan_sentence_fix_no_issues_and_rejects_bad_args() {
        let clean = "Tôi đi học.";
        let clean_c = CString::new(clean).unwrap();

        let mut empty_res = ime_assist_result_v1 {
            abi_version: IME_ABI_VERSION,
            count: 0,
            issues: [unsafe { std::mem::zeroed() }; IME_MAX_ASSIST_ISSUES],
        };
        let mut plan = ime_assist_plan_v1::zeroed();

        // No issues -> 0
        assert_eq!(
            ime_assist_plan_sentence_fix(std::ptr::null(), clean_c.as_ptr(), &empty_res, &mut plan),
            0
        );

        // Null sentence -> 0
        assert_eq!(
            ime_assist_plan_sentence_fix(std::ptr::null(), std::ptr::null(), &empty_res, &mut plan),
            0
        );

        // Null result -> 0
        assert_eq!(
            ime_assist_plan_sentence_fix(
                std::ptr::null(),
                clean_c.as_ptr(),
                std::ptr::null(),
                &mut plan
            ),
            0
        );

        // Null out_plan -> 0
        assert_eq!(
            ime_assist_plan_sentence_fix(
                std::ptr::null(),
                clean_c.as_ptr(),
                &empty_res,
                std::ptr::null_mut()
            ),
            0
        );

        // Bad ABI version -> 0
        empty_res.abi_version = 999;
        assert_eq!(
            ime_assist_plan_sentence_fix(std::ptr::null(), clean_c.as_ptr(), &empty_res, &mut plan),
            0
        );
    }

    /// R-7 (audit vòng 3, 2026-10-23): `plan_issues_from_raw` là **nguồn duy nhất**
    /// của ngữ nghĩa sửa (FFI + Windows). Test khoá các luật lọc để sửa một nơi
    /// là cả 3 OS đổi theo: không auto-fixable · kind lạ · span ngoài câu ·
    /// span == gợi ý · gợi ý chứa ký tự điều khiển · lặp từ nuốt luôn dấu cách
    /// đứng trước.
    #[test]
    fn plan_issues_from_raw_loc_va_ngu_nghia_sua() {
        let sentence = "tôi tôi đi học .";
        let raws = vec![
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_DUPLICATE_WORD,
                start: 4,
                end: 7,
                auto_fixable: true,
                confidence: 90,
                suggest: "tôi".to_string(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_SPELLING,
                start: 8,
                end: 10,
                auto_fixable: false,
                confidence: 50,
                suggest: String::new(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_DIACRITIC_TYPO,
                start: 8,
                end: 10,
                auto_fixable: true,
                confidence: 80,
                suggest: "đi".to_string(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_DIACRITIC_TYPO,
                start: 11,
                end: 14,
                auto_fixable: true,
                confidence: 80,
                suggest: "học\n".to_string(),
            },
            PlanIssueRaw {
                kind: 9_999,
                start: 11,
                end: 14,
                auto_fixable: true,
                confidence: 80,
                suggest: "học".to_string(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_DIACRITIC_TYPO,
                start: 14,
                end: 99,
                auto_fixable: true,
                confidence: 80,
                suggest: ".".to_string(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_SPACE_BEFORE_PUNCT,
                start: 14,
                end: 15,
                auto_fixable: true,
                confidence: 95,
                suggest: String::new(),
            },
        ];

        let issues = plan_issues_from_raw(sentence, &raws);
        assert_eq!(issues.len(), 2, "chỉ lỗi hợp lệ mới vào kế hoạch");
        assert_eq!(issues[0].kind, textvn_assist::IssueKind::DuplicateWord);
        assert_eq!(issues[0].confidence, 90);
        assert!(
            matches!(
                issues[0].fix,
                Some(textvn_assist::Fix::Delete { start: 3, end: 7 })
            ),
            "lặp từ phải nuốt luôn dấu cách trước nó: {:?}",
            issues[0].fix
        );
        assert_eq!(issues[1].kind, textvn_assist::IssueKind::SpaceBeforePunct);
        assert!(
            matches!(
                issues[1].fix,
                Some(textvn_assist::Fix::Delete { start: 14, end: 15 })
            ),
            "xoá đúng dấu cách trước dấu chấm: {:?}",
            issues[1].fix
        );
        // `message` rỗng là chủ ý (kế hoạch sửa không dùng câu thông báo);
        // gợi ý vẫn giữ nguyên để adapter hiển thị nếu cần.
        assert!(issues.iter().all(|i| i.message.is_empty()));
        assert_eq!(issues[0].suggestions, vec!["tôi".to_string()]);
    }

    #[test]
    fn plan_issues_from_raw_double_space_va_space_after_punct() {
        let sentence = "Tôi  đi,học.";
        let raws = vec![
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_DOUBLE_SPACE,
                start: 4,
                end: 5,
                auto_fixable: true,
                confidence: 100,
                suggest: " ".to_string(),
            },
            PlanIssueRaw {
                kind: IME_ASSIST_KIND_SPACE_AFTER_PUNCT,
                start: 7,
                end: 8,
                auto_fixable: true,
                confidence: 100,
                suggest: ", ".to_string(),
            },
        ];
        let issues = plan_issues_from_raw(sentence, &raws);
        assert_eq!(issues.len(), 2);
        let plan = textvn_assist::plan_sentence_fix(sentence, &issues).expect("phai co plan");
        assert_eq!(plan.fixed_count, 2);
        assert_eq!(plan.delete_from, 4);
        assert_eq!(plan.replacement, "đi, học.");
        let head: String = sentence.chars().take(plan.delete_from).collect();
        assert_eq!(format!("{head}{}", plan.replacement), "Tôi đi, học.");

        // Synthetic span bao trùm cả chuỗi dấu cách (như AssistTests.swift trên macOS: 3..5)
        let sentence_full_span = "Tôi  đi học.";
        let raws_full_span = vec![PlanIssueRaw {
            kind: IME_ASSIST_KIND_DOUBLE_SPACE,
            start: 3,
            end: 5,
            auto_fixable: true,
            confidence: 85,
            suggest: " ".to_string(),
        }];
        let issues_full = plan_issues_from_raw(sentence_full_span, &raws_full_span);
        let plan_full =
            textvn_assist::plan_sentence_fix(sentence_full_span, &issues_full).expect("plan");
        assert_eq!(plan_full.fixed_count, 1);
        assert_eq!(plan_full.delete_from, 3);
        assert_eq!(plan_full.original_tail, "  đi học.");
        assert_eq!(plan_full.replacement, " đi học.");
    }

    #[test]
    fn copy_str_to_c_buf_khong_cat_giua_codepoint_utf8() {
        // "đ" (2 byte) + "ể" (3 byte): tổng 5 byte. Buffer 5 byte (max 4 byte)
        // không được cắt giữa "ể" ở byte 4 mà phải lùi về ranh giới byte 2.
        let mut buf = [0 as c_char; 5];
        let n = copy_str_to_c_buf("để", &mut buf);
        assert_eq!(n, 2);
        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(cstr.to_str().unwrap(), "đ");
    }
}
