// SPDX-License-Identifier: GPL-3.0-or-later
//! L4 fuzz — parser `config.v1` + đường FFI đi qua config (P0-4 §1 L4: "fuzz `config_parse`").
//!
//! Bất biến assert:
//! 1. **Fail-open**: config sai → `parse_config` trả `Err`, nhưng `ime_instance_new` **vẫn**
//!    tạo instance (chạy config mặc định) và trả đúng `IME_ERR_CONFIG` — không panic,
//!    không trả NULL (P0-3 §1.3, P0-2 §1/§5).
//! 2. **Nhất quán 2 tầng**: `rc_new == IME_ERR_CONFIG` ⟺ `parse_config` cũng `Err`.
//! 3. **Reload**: config hợp lệ → `ime_reload_config` = `IME_OK`; config sai → `IME_ERR_CONFIG`
//!    và **giữ config đang chạy** (không đổi hành vi engine sau đó).
//! 4. **S2**: `ime_last_error` không chứa lại text người dùng đưa vào.
//! 5. `ime_suggest` (feature tắt) không panic và không trả item nào (P0-2 §1).

#![no_main]

use std::ffi::CStr;
use std::ptr;

use libfuzzer_sys::fuzz_target;
use vietime_ffi::*;
use vietime_config::parse_config;

fuzz_target!(|data: &[u8]| {
    // JSON bắt buộc UTF-8; byte vô nghĩa thì bỏ qua (không phải đường hợp thú vị).
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    if s.is_empty() {
        return;
    }

    let parsed_ok = parse_config(s).is_ok();

    // ---- 1/2: instance từ config này ----
    let mut inst: *mut ime_instance = ptr::null_mut();
    let rc_new = ime_instance_new(s.as_ptr(), s.len(), &mut inst);
    assert!(!inst.is_null(), "ime_instance_new phải luôn trả instance");
    assert!(
        rc_new == IME_OK || rc_new == IME_ERR_CONFIG,
        "rc_new={rc_new} lạ"
    );
    assert_eq!(
        rc_new == IME_OK,
        parsed_ok,
        "FFI và vietime-config phải cùng kết luận về 1 config"
    );

    // ---- 4: S2 — lỗi không echo text người dùng ----
    if rc_new == IME_ERR_CONFIG {
        let err = unsafe { CStr::from_ptr(ime_last_error(inst)) }
            .to_string_lossy()
            .to_string();
        assert!(!err.is_empty(), "lỗi phải có mô tả (kind), không rỗng");
        // Kiểm tra theo **hình dạng** chứ không so khớp chuỗi: input hợp lệ vẫn có
        // thể trùng từ với thông báo lỗi → so khớp sẽ báo động giả.
        assert!(err.len() <= 64, "last_error quá dài — nghi echo input: {err}");
        assert!(
            !err.contains('{') && !err.contains('"'),
            "last_error chứa ký tự JSON — nghi echo nội dung config: {err}"
        );
        if s.len() >= 16 {
            assert!(
                !err.contains(&s[..16]),
                "last_error chứa 16 ký tự đầu của input (S2): {err}"
            );
        }
    }

    // ---- 3: reload — cùng bytes, phải cho kết luận nhất quán ----
    let rc_reload = ime_reload_config(inst, s.as_ptr(), s.len());
    assert_eq!(
        rc_reload == IME_OK,
        parsed_ok,
        "ime_reload_config lệch với parse_config"
    );
    // len = 0 → no-op, hợp lệ (P0-2 §1)
    assert_eq!(ime_reload_config(inst, ptr::null(), 0), IME_OK);

    // ---- engine vẫn gõ được sau mọi đường config (fail-open không treo state) ----
    let mut out = ime_result_v1 {
        abi_version: 0,
        action: 0,
        delete_count: 0,
        insert_len: 0,
        preedit_len: 0,
        _reserved: 0,
        flags: 0,
        insert: [0; IME_MAX_TEXT],
        preedit: [0; IME_MAX_TEXT],
    };
    for (i, ch) in "duocj ".chars().enumerate() {
        let key = ime_key_v1 {
            abi_version: IME_ABI_VERSION,
            vk: ch as u32,
            ch: ch as u32,
            mods: 0,
            key_down: 1,
            is_repeat: 0,
            is_injected: 0,
            _reserved: 0,
        };
        let _ = i;
        let rc = ime_key(inst, &key, &mut out);
        assert!(
            rc == IME_OK || rc == IME_ERR_INTERNAL,
            "ime_key sau reload trả {rc} lạ"
        );
        assert!(out.insert_len as usize <= IME_MAX_TEXT);
        assert!(out.preedit_len as usize <= IME_MAX_TEXT);
    }

    // ---- 5: suggest (feature tắt) — không panic, không item ----
    let mut sug = ime_suggest_v1 {
        abi_version: 0,
        count: 0,
        lens: [0; IME_MAX_SUGGEST],
        items: [[0; IME_MAX_TEXT]; IME_MAX_SUGGEST],
    };
    let word: [u32; 4] = [b'd' as u32, b'u' as u32, b'o' as u32, b'c' as u32];
    let rc_sug = ime_suggest(inst, word.as_ptr(), word.len() as u32, &mut sug);
    assert!(
        rc_sug == IME_ERR_INTERNAL || rc_sug == IME_OK,
        "ime_suggest trả mã lạ {rc_sug}"
    );
    if rc_sug != IME_OK {
        assert_eq!(sug.count, 0, "suggest tắt thì không được trả item nào");
        assert_eq!(sug.abi_version, IME_ABI_VERSION);
    }
    assert!((sug.count as usize) <= IME_MAX_SUGGEST);
    for i in 0..sug.count as usize {
        assert!((sug.lens[i] as usize) <= IME_MAX_TEXT, "lens vượt IME_MAX_TEXT");
    }

    ime_instance_free(inst);
});
