// SPDX-License-Identifier: GPL-3.0-or-later
//! L4 fuzz — `ime_key` trên **ABI thật** (P0-4 §1 L4: "fuzz `ffi_key`").
//!
//! Không chỉ "không panic": target assert các bất biến mà adapter dựa vào
//! (P0-2 §0/§2). Một vi phạm = bug chặn phím hoặc đọc tràn buffer ở adapter.
//!
//! 1. `insert_len`/`preedit_len` ≤ `IME_MAX_TEXT` — adapter đọc `out.insert[..insert_len]`.
//! 2. `action` luôn thuộc tập đã công bố trong header C.
//! 3. Return code luôn thuộc tập `IME_*` đã công bố (không trả số lạ).
//! 4. **Fail-open**: mọi đường lỗi/ABI-lệch vẫn trả `action = PASS` (P0-2 §0.3, S4) —
//!    nếu lỗi mà nuốt phím thì bàn phím người dùng hỏng.
//! 5. `delete_count` ≤ `IME_MAX_TEXT` (buffer engine tự quản).

#![no_main]

use libfuzzer_sys::fuzz_target;
use vietime_ffi::*;

/// Chia `data` thành từng byte → trường của `ime_key_v1`.
struct ByteCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn next(&mut self) -> Option<u8> {
        let b = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }
    fn done(&self) -> bool {
        self.pos >= self.data.len()
    }
}

fn assert_result_sane(rc: i32, out: &ime_result_v1, ctx: &str) {
    assert!(
        rc == IME_OK
            || rc == IME_ERR_INVALID_ARG
            || rc == IME_ERR_CONFIG
            || rc == IME_ERR_ABI
            || rc == IME_ERR_INTERNAL,
        "{ctx}: ime_key trả mã lạ {rc}"
    );
    assert!(
        out.insert_len as usize <= IME_MAX_TEXT,
        "{ctx}: insert_len {} > IME_MAX_TEXT — adapter đọc tràn",
        out.insert_len
    );
    assert!(
        out.preedit_len as usize <= IME_MAX_TEXT,
        "{ctx}: preedit_len {} > IME_MAX_TEXT",
        out.preedit_len
    );
    assert!(
        out.delete_count as usize <= IME_MAX_TEXT,
        "{ctx}: delete_count {} vượt text buffer",
        out.delete_count
    );
    assert!(
        out.action == ACTION_PASS
            || out.action == ACTION_REPLACE
            || out.action == ACTION_COMMIT
            || out.action == ACTION_RESTORE,
        "{ctx}: action {} lạ",
        out.action
    );
    assert_eq!(
        out.abi_version, IME_ABI_VERSION,
        "{ctx}: abi_version phải set"
    );
    // Fail-open: không phải OK thì phím phải đi thẳng (PASS), không nuốt.
    if rc != IME_OK {
        assert_eq!(
            out.action, ACTION_PASS,
            "{ctx}: rc={rc} mà action={} — vi phạm fail-open (S4)",
            out.action
        );
    }
}

fuzz_target!(|data: &[u8]| {
    // Cần ≥ 2 byte: byte 0 chọn config, byte 1 khởi tạo context. libFuzzer sinh
    // input rất ngắn — không được assert rằng "đã gõ phím nào" (sẽ báo động giả).
    if data.len() < 2 {
        return;
    }

    // ---- instance: config lấy từ byte đầu (2 = NULL → default) ----
    let mut inst: *mut ime_instance = std::ptr::null_mut();
    let cfg_choice = data[0] % 3;
    let cfg_bytes: &[u8] = if cfg_choice == 1 { b"{}" } else { b"" };
    let rc_new = ime_instance_new(
        if cfg_choice == 2 {
            std::ptr::null()
        } else {
            cfg_bytes.as_ptr()
        },
        if cfg_choice == 2 { 0 } else { cfg_bytes.len() },
        &mut inst,
    );
    assert!(!inst.is_null(), "ime_instance_new phải luôn trả instance");
    assert!(
        rc_new == IME_OK || rc_new == IME_ERR_CONFIG,
        "rc_new={rc_new} không hợp lệ cho config mặc định/rỗng"
    );
    if rc_new == IME_ERR_CONFIG {
        assert!(
            !ime_last_error(inst).is_null(),
            "last_error phải non-NULL khi có lỗi"
        );
    }

    // ---- context: field_role hợp lệ, secure/enabled ngẫu nhiên ----
    let mut cur = ByteCursor::new(&data[1..]);
    let role = cur.next().unwrap_or(0) % 11; // 0..=10 hợp lệ (P0-2 §2)
    let ctx = ime_context_v1 {
        abi_version: IME_ABI_VERSION,
        enabled: (cur.next().unwrap_or(1) % 2) as u32,
        secure: (cur.next().unwrap_or(0) % 2) as u32,
        field_role: role as u32,
        caps: u32::from_le_bytes([cur.next().unwrap_or(0); 4]),
        app_id: std::ptr::null(),
        element_name: std::ptr::null(),
        hint: 0,
    };
    let rc_ctx = ime_set_context(inst, &ctx);
    assert!(
        rc_ctx == IME_OK || rc_ctx == IME_ERR_ABI || rc_ctx == IME_ERR_INTERNAL,
        "ime_set_context trả mã lạ {rc_ctx}"
    );

    // ---- chuỗi phím: mỗi vòng đọc byte cho vk/ch/mods/flags ----
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
    let mut last_key = ime_key_v1 {
        abi_version: IME_ABI_VERSION,
        vk: 0,
        ch: 0,
        mods: 0,
        key_down: 1,
        is_repeat: 0,
        is_injected: 0,
        _reserved: 0,
    };
    for _ in 0..64 {
        if cur.done() {
            break;
        }
        let b4 = cur.next().unwrap_or(0);
        last_key = ime_key_v1 {
            abi_version: IME_ABI_VERSION,
            vk: u32::from(cur.next().unwrap_or(0)),
            ch: u32::from(cur.next().unwrap_or(0)),
            mods: u32::from(cur.next().unwrap_or(0)),
            key_down: 1,
            is_repeat: b4 & 1,
            is_injected: (b4 >> 1) & 1,
            _reserved: 0,
        };
        let rc = ime_key(inst, &last_key, &mut out);
        assert_result_sane(rc, &out, "ime_key");
        if last_key.is_injected == 1 {
            assert_eq!(
                out.action, ACTION_PASS,
                "phím injected phải PASS, nhận action {}",
                out.action
            );
        }
    }
    // ---- đường ABI-lệch: abi sai → ERR_ABI + PASS, engine giữ nguyên state ----
    let bad = ime_key_v1 {
        abi_version: IME_ABI_VERSION.wrapping_add(1),
        ..last_key
    };
    let rc_bad = ime_key(inst, &bad, &mut out);
    assert_eq!(rc_bad, IME_ERR_ABI, "abi lệch phải trả IME_ERR_ABI");
    assert_eq!(out.action, ACTION_PASS, "abi lệch → phím đi thẳng");

    // ---- NULL args: không crash, trả ERR_INVALID_ARG (P0-2 §1) ----
    assert_eq!(
        ime_key(std::ptr::null_mut(), &last_key, &mut out),
        IME_ERR_INVALID_ARG
    );
    assert_eq!(
        ime_key(inst, std::ptr::null(), &mut out),
        IME_ERR_INVALID_ARG
    );
    assert_eq!(
        ime_key(inst, &last_key, std::ptr::null_mut()),
        IME_ERR_INVALID_ARG
    );
    assert_eq!(
        ime_set_context(std::ptr::null_mut(), &ctx),
        IME_ERR_INVALID_ARG
    );
    assert_eq!(ime_reset(std::ptr::null_mut()), IME_ERR_INVALID_ARG);
    assert!(!ime_last_error(std::ptr::null()).is_null());

    ime_instance_free(inst);
});
