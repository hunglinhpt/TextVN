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

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::slice;

use textvn_config::{parse_config, Config};
use textvn_core::{Action, Context, Engine, EngineOptions, KeyEvent, Outcome};
use textvn_strategy::{ResolveInput, Strategy};

/// Hằng modifier — header C gọi là `IME_MOD_*` (`P0-2 §1`).
pub use textvn_core::keymap::{
    MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_FN, MOD_META, MOD_SHIFT, MOD_SUPER,
};
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

/// Opaque instance (P0-2 §1) — 1 instance = 1 thread (§3).
/// Tên `ime_instance` phải khớp header C (P0-2 §6).
#[allow(non_camel_case_types)]
pub struct ime_instance {
    engine: Engine,
    last_error: CString,
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
            Ok(s) => parse_config(s).map_err(|_| ()),
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
    if ctx.field_role > 10 {
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

    match catch_unwind(AssertUnwindSafe(|| inst.engine.key(&key))) {
        Ok(outcome) => {
            fill_result(result, &outcome);
            IME_OK
        }
        Err(_) => {
            // Fail-open: trạng thái treo → reset, phím đi thẳng (S4)
            inst.engine.reset();
            *result = ime_result_v1::zeroed_pass();
            result.abi_version = IME_ABI_VERSION;
            result.action = ACTION_PASS;
            result.flags = IME_FLAG_ERROR;
            set_error(inst, "internal: panic caught");
            IME_OK
        }
    }
}

fn fill_result(result: &mut ime_result_v1, outcome: &Outcome) {
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
    // Cắt ≤ IME_MAX_TEXT (P0-2 §2 — engine tự giữ; đây là lưới an toàn cuối)
    insert.truncate(IME_MAX_TEXT);
    let preedit: Vec<u32> = outcome
        .preedit
        .iter()
        .take(IME_MAX_TEXT)
        .map(|&c| c as u32)
        .collect();

    result.delete_count = delete_count;
    result.insert_len = insert.len() as u16;
    result.preedit_len = preedit.len() as u16;
    result.flags = outcome.flags;
    result.insert[..insert.len()].copy_from_slice(&insert);
    result.preedit[..preedit.len()].copy_from_slice(&preedit);
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
        Ok(s) => parse_config(s),
        Err(_) => Err(textvn_config::ConfigError::Schema),
    };
    match parsed {
        Ok(c) => {
            let opts = options_from_config(&c);
            match catch_unwind(AssertUnwindSafe(|| inst.engine.set_options(opts))) {
                Ok(()) => IME_OK,
                Err(_) => IME_ERR_INTERNAL,
            }
        }
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
    if textvn_appdb::AppDb::parse(text).is_err() {
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
        match textvn_appdb::AppDb::parse(json) {
            Ok(value) => Some(value),
            Err(_) => return IME_ERR_CONFIG,
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
#[no_mangle]
pub extern "C" fn ime_last_error(inst: *const ime_instance) -> *const c_char {
    if inst.is_null() {
        return c"".as_ptr();
    }
    let inst = unsafe { &*inst };
    inst.last_error.as_ptr()
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
}
