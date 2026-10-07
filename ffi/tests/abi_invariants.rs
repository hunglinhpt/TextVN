// SPDX-License-Identifier: GPL-3.0-or-later
//! Stress test **tương đương fuzz** cho C-ABI, chạy trên **stable** (không cần
//! `cargo-fuzz`/nightly) — cùng bộ invariant với `fuzz/fuzz_targets/ffi_key.rs` và
//! `config_parse.rs` (P0-4 §1 L4).
//!
//! Vì sao vẫn cần: fuzz thật chỉ chạy ở CI nightly; test này chạy **mọi PR** trên 3 OS
//! với PRNG tự cài (xorshift64*) nên invariant bị vi phạm sẽ fail ở `cargo test`.
//!
//! Invariant: xem `fuzz/fuzz_targets/ffi_key.rs` (P0-2 §0/§2) —
//!   1. `insert_len`/`preedit_len`/`delete_count` ≤ `IME_MAX_TEXT` (adapter đọc tràn?).
//!   2. `action` ∈ tập công bố; 3. return code ∈ tập `IME_*`.
//!   4. **Fail-open**: rc ≠ OK ⇒ `action == PASS` (S4).
//!   5. phím `is_injected` luôn PASS (chống loop — P0-3 §6).
//!   6. abi lệch ⇒ `IME_ERR_ABI` + PASS; NULL args ⇒ `IME_ERR_INVALID_ARG`.
//!   7. config sai vẫn tạo instance; FFI và `textvn-config` cùng kết luận.
//!   8. `ime_last_error` không chứa text người dùng (S2).
//!   9. `delete_count` không vượt số ký tự chuỗi phím đã đưa vào document (mô hình độ dài
//!      bảo thủ) — engine không xoá lẹm text có sẵn (CR-01), kể cả với từ > 64 phím.

use std::ffi::CStr;
use std::ptr;

use textvn_ffi::*;

/// xorshift64* — không dependency, đủ để sinh dữ liệu giống fuzz.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn byte(&mut self) -> u8 {
        (self.next_u64() >> 24) as u8
    }
}

fn new_result() -> ime_result_v1 {
    ime_result_v1 {
        abi_version: 0,
        action: 0,
        delete_count: 0,
        insert_len: 0,
        preedit_len: 0,
        _reserved: 0,
        flags: 0,
        insert: [0; IME_MAX_TEXT],
        preedit: [0; IME_MAX_TEXT],
    }
}

fn assert_sane(rc: i32, out: &ime_result_v1, ctx: &str) {
    assert!(
        rc == IME_OK
            || rc == IME_ERR_INVALID_ARG
            || rc == IME_ERR_CONFIG
            || rc == IME_ERR_ABI
            || rc == IME_ERR_INTERNAL,
        "{ctx}: return code lạ {rc}"
    );
    assert!(
        out.insert_len as usize <= IME_MAX_TEXT,
        "{ctx}: insert_len {} > IME_MAX_TEXT (adapter đọc tràn)",
        out.insert_len
    );
    assert!(
        out.preedit_len as usize <= IME_MAX_TEXT,
        "{ctx}: preedit_len {} > IME_MAX_TEXT",
        out.preedit_len
    );
    assert!(
        out.delete_count as usize <= IME_MAX_TEXT,
        "{ctx}: delete_count {} quá lớn",
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
    if rc != IME_OK {
        assert_eq!(
            out.action, ACTION_PASS,
            "{ctx}: rc={rc} mà action={} — nuốt phím khi lỗi (vi phạm S4)",
            out.action
        );
    }
}

const VK_BACK: u32 = 0x08;
const VK_TAB: u32 = 0x09;
const VK_RETURN: u32 = 0x0D;
const VK_SPACE: u32 = 0x20;

/// Mô hình độ dài document (con trỏ ở cuối) — giống hệt `fuzz/fuzz_targets/ffi_key.rs`.
/// Đếm DƯ ở mọi chỗ không chắc (chord, ký tự điều khiển); chỉ Backspace đi thẳng trừ 1.
fn apply_doc(doc_len: &mut usize, key: &ime_key_v1, out: &ime_result_v1, ctx: &str) {
    if out.action == ACTION_PASS {
        if key.vk == VK_BACK {
            *doc_len = doc_len.saturating_sub(1);
        } else if (key.ch != 0 && char::from_u32(key.ch).is_some())
            || matches!(key.vk, VK_SPACE | VK_RETURN | VK_TAB)
        {
            *doc_len += 1;
        }
        return;
    }
    let d = out.delete_count as usize;
    assert!(
        d <= *doc_len,
        "{ctx}: delete_count {d} lẹm vào text có sẵn (chuỗi phím mới đưa vào {doc_len})"
    );
    *doc_len = *doc_len - d + out.insert_len as usize;
}

#[test]
fn abi_invariants_giu_duoi_input_ngau_nhien() {
    // 200 vòng × 64 phím = 12 800 lần gọi `ime_key` (nhanh, chạy mọi PR).
    let mut rng = Rng::new(0x5EED_1234_ABCD_0001);
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
    for round in 0..200u32 {
        // config: byte đầu chọn 3 đường (NULL default / "{}" / rỗng)
        let mut inst: *mut ime_instance = ptr::null_mut();
        let choice = rng.byte() % 3;
        let (ptr_cfg, len_cfg) = match choice {
            0 => (ptr::null(), 0usize),
            1 => (b"{}".as_ptr(), 2usize),
            _ => (b"".as_ptr(), 0usize),
        };
        let rc_new = ime_instance_new(ptr_cfg, len_cfg, &mut inst);
        assert!(
            !inst.is_null(),
            "round {round}: phải luôn tạo được instance"
        );
        assert!(
            rc_new == IME_OK || rc_new == IME_ERR_CONFIG,
            "round {round}: rc_new={rc_new} lạ"
        );

        let ctx = ime_context_v1 {
            abi_version: IME_ABI_VERSION,
            enabled: (rng.byte() % 2) as u32,
            secure: (rng.byte() % 2) as u32,
            field_role: u32::from(rng.byte() % 11), // 0..=10 hợp lệ
            caps: u32::from_le_bytes([rng.byte(); 4]),
            app_id: ptr::null(),
            element_name: ptr::null(),
            hint: 0,
        };
        let rc_ctx = ime_set_context(inst, &ctx);
        assert!(
            rc_ctx == IME_OK || rc_ctx == IME_ERR_ABI || rc_ctx == IME_ERR_INTERNAL,
            "round {round}: rc_ctx={rc_ctx} lạ"
        );

        let mut out = new_result();
        let mut doc_len = 0usize;
        for step in 0..64 {
            let flags = rng.byte();
            last_key = ime_key_v1 {
                abi_version: IME_ABI_VERSION,
                vk: u32::from(rng.byte()),
                ch: u32::from(rng.byte()),
                mods: u32::from(rng.byte()),
                key_down: 1,
                is_repeat: flags & 1,
                is_injected: (flags >> 1) & 1,
                _reserved: 0,
            };
            let rc = ime_key(inst, &last_key, &mut out);
            assert_sane(rc, &out, &format!("round {round} step {step}"));
            if last_key.is_injected == 1 {
                assert_eq!(
                    out.action, ACTION_PASS,
                    "round {round} step {step}: phím injected phải PASS"
                );
            }
            apply_doc(
                &mut doc_len,
                &last_key,
                &out,
                &format!("round {round} step {step}"),
            );
        }

        // abi lệch + NULL args (6)
        let bad = ime_key_v1 {
            abi_version: IME_ABI_VERSION.wrapping_add(1),
            ..last_key
        };
        assert_eq!(ime_key(inst, &bad, &mut out), IME_ERR_ABI);
        assert_eq!(out.action, ACTION_PASS);
        assert_eq!(
            ime_key(ptr::null_mut(), &last_key, &mut out),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(ime_key(inst, ptr::null(), &mut out), IME_ERR_INVALID_ARG);
        assert_eq!(
            ime_key(inst, &last_key, ptr::null_mut()),
            IME_ERR_INVALID_ARG
        );
        assert_eq!(ime_set_context(ptr::null_mut(), &ctx), IME_ERR_INVALID_ARG);
        assert_eq!(ime_reset(ptr::null_mut()), IME_ERR_INVALID_ARG);
        assert!(!ime_last_error(ptr::null()).is_null());

        ime_instance_free(inst);
    }
}

/// Chuỗi phím DÀI có "giữ phím" (giống chế độ lặp của fuzz `ffi_key`): chữ Telex
/// chiếm đa số để từ vượt 64 phím, xen Backspace/Esc/Space/Tab/Enter, context ngẫu
/// nhiên (mọi strategy). Bất biến 1–4 + 9 trên mỗi phím.
#[test]
fn tu_dai_va_giu_phim_khong_xoa_lem_text_co_san() {
    const LETTERS: &[u8] = b"aoeuiydnghtpsfrxjwzqAS";
    const OTHERS: &[u32] = &[VK_BACK, 0x1B, VK_SPACE, VK_TAB, VK_RETURN];
    let mut rng = Rng::new(0xC0DE_0001_D0C5_0042);
    for round in 0..60u32 {
        let mut inst: *mut ime_instance = ptr::null_mut();
        assert_eq!(ime_instance_new(ptr::null(), 0, &mut inst), IME_OK);
        let ctx = ime_context_v1 {
            abi_version: IME_ABI_VERSION,
            enabled: 1,
            secure: 0,
            field_role: u32::from(rng.byte() % 11),
            caps: u32::from(rng.byte() & 0x0F),
            app_id: ptr::null(),
            element_name: ptr::null(),
            hint: 0,
        };
        assert_eq!(ime_set_context(inst, &ctx), IME_OK);
        let mut out = new_result();
        let mut doc_len = 0usize;
        let mut sent = 0usize;
        while sent < 512 {
            let r = rng.byte();
            let key = if r.is_multiple_of(8) {
                let vk = OTHERS[usize::from(rng.byte()) % OTHERS.len()];
                ime_key_v1 {
                    abi_version: IME_ABI_VERSION,
                    vk,
                    ch: if vk == VK_SPACE { u32::from(b' ') } else { 0 },
                    mods: 0,
                    key_down: 1,
                    is_repeat: 0,
                    is_injected: 0,
                    _reserved: 0,
                }
            } else {
                let c = LETTERS[usize::from(rng.byte()) % LETTERS.len()];
                ime_key_v1 {
                    abi_version: IME_ABI_VERSION,
                    vk: 0,
                    ch: u32::from(c),
                    mods: 0,
                    key_down: 1,
                    is_repeat: 0,
                    is_injected: 0,
                    _reserved: 0,
                }
            };
            // 1/4 số phím được "giữ" 2..=90 lần.
            let times = if r % 4 == 1 {
                2 + usize::from(rng.byte()) % 89
            } else {
                1
            };
            for i in 0..times.min(512 - sent) {
                let k = ime_key_v1 {
                    is_repeat: u8::from(i > 0),
                    ..key
                };
                let rc = ime_key(inst, &k, &mut out);
                let at = format!("round {round} key {sent}");
                assert_sane(rc, &out, &at);
                assert_eq!(rc, IME_OK, "{at}: engine báo lỗi trên chuỗi phím hợp lệ");
                apply_doc(&mut doc_len, &k, &out, &at);
                sent += 1;
            }
        }
        ime_instance_free(inst);
    }
}

/// Input config ngẫu nhiên: FFI và `textvn-config` phải **cùng kết luận**,
/// và config sai vẫn phải ra instance (fail-open) + lỗi không echo text (S2).
#[test]
fn config_paths_fail_open_va_khong_echo_text() {
    let mut rng = Rng::new(0xC0FF_EE00_1234_5678);
    // mẫu config: hợp lệ, sai schema, sai kiểu, rỗng, cắt dở, JSON vỡ
    let fixed: &[&str] = &[
        "",
        "{}",
        "{\"config_version\":1}",
        "{\"config_version\":1,\"method\":\"vni\"}",
        "{\"config_version\":1,\"method\":\"telex\",\"auto_capitalize\":true}",
        "{\"config_version\":2}",
        "{\"config_version\":1,\"method\":\"dvorak\"}",
        "{\"config_version\":1,\"auto_capitalize\":\"khong-phai-bool\"}",
        "{\"config_version\":1,\"macros\":[{\"trigger\":\"cty\",\"expand\":\"Cong ty\"}]}",
        "{\"config_version\":1,\"macros\":[{\"trigger\":\"\",\"expand\":\"\"}]}",
        "[1,2,3]",
        "null",
        "{\"config_version\":1",
    ];
    let mut cases: Vec<String> = fixed.iter().map(|s| s.to_string()).collect();
    // cắt dở chuỗi hợp lệ ở mọi vị trí (bắt lỗi "thiếu }" ở cuối file)
    let valid = "{\"config_version\":1,\"method\":\"vni\",\"macro_trigger\":\"space\"}";
    for cut in 1..valid.len() {
        cases.push(valid[..cut].to_string());
    }
    // rác ngẫu nhiên (chủ yếu sai schema)
    for _ in 0..200 {
        let len = (rng.byte() % 40) as usize;
        let s: String = (0..len)
            .map(|_| {
                let pool = b"{}[]\",:0123456789abcdefghijklmnopqrstuvwxyz_ \n\t\\";
                pool[(rng.byte() as usize) % pool.len()] as char
            })
            .collect();
        cases.push(s);
    }

    for s in &cases {
        let parsed_ok = textvn_config::parse_config(s).is_ok();

        let mut inst: *mut ime_instance = ptr::null_mut();
        let rc_new = ime_instance_new(s.as_ptr(), s.len(), &mut inst);
        assert!(!inst.is_null(), "config phải luôn ra instance: {s:?}");
        assert!(
            rc_new == IME_OK || rc_new == IME_ERR_CONFIG,
            "rc_new={rc_new} lạ cho {s:?}"
        );
        assert_eq!(
            rc_new == IME_OK,
            parsed_ok,
            "FFI và textvn-config lệch kết luận cho {s:?}"
        );

        if rc_new == IME_ERR_CONFIG {
            let err = unsafe { CStr::from_ptr(ime_last_error(inst)) }
                .to_string_lossy()
                .to_string();
            // S2: `ime_last_error` chỉ chứa **kind** lỗi, không chứa nội dung config.
            // Kiểm tra theo hình dạng (bền với mọi input), không so khớp chuỗi —
            // input hợp lệ vẫn có thể trùng từ với thông báo lỗi (false positive).
            assert!(!err.is_empty(), "lỗi phải có kind");
            assert!(
                err.len() <= 64,
                "last_error quá dài — nghi echo input: {err}"
            );
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

        // reload: cùng bytes → cùng kết luận; len=0 là no-op
        assert_eq!(
            ime_reload_config(inst, s.as_ptr(), s.len()) == IME_OK,
            parsed_ok,
            "ime_reload_config lệch với parse_config cho {s:?}"
        );
        assert_eq!(ime_reload_config(inst, ptr::null(), 0), IME_OK);

        // sau mọi đường config, engine vẫn gõ được (không treo state)
        let mut out = new_result();
        for ch in "duocj ".chars() {
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
            let rc = ime_key(inst, &key, &mut out);
            assert!(
                rc == IME_OK || rc == IME_ERR_INTERNAL,
                "ime_key sau reload trả {rc} lạ cho {s:?}"
            );
            assert_sane(rc, &out, &format!("sau reload {s:?}"));
        }

        // suggest (feature tắt) — không item
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
            "ime_suggest trả {rc_sug} lạ"
        );
        if rc_sug != IME_OK {
            assert_eq!(sug.count, 0);
        }
        assert!((sug.count as usize) <= IME_MAX_SUGGEST);
        for i in 0..sug.count as usize {
            assert!((sug.lens[i] as usize) <= IME_MAX_TEXT);
        }

        ime_instance_free(inst);
    }
}
