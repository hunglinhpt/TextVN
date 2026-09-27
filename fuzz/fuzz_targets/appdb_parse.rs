// SPDX-License-Identifier: GPL-3.0-or-later
//! L4 fuzz — parser preset `appdb.v1` (P0-4 §1 L4: "fuzz `appdb_parse`").
//!
//! Preset là **input không tin được** (tải về từ server, có thể hỏng/cũ). Bất biến:
//! 1. `AppDb::parse` không panic với bất kỳ UTF-8 nào (kể cả rác).
//! 2. appdb hợp lệ → strategy id luôn trong tập đã công bố (0..=4).
//!
//! Target này cố ý không link `textvn-ffi`: libFuzzer MSVC thêm `/include:main`
//! và không thể link cùng crate có artifact `cdylib`. Đường FFI được kiểm bởi
//! `ffi/tests/abi_invariants.rs` trên stable; `ffi_key`/`config_parse` fuzz ở
//! runner Unix với feature `ffi-fuzz`.

#![no_main]

use libfuzzer_sys::fuzz_target;

const STRATEGY_MAX_ID: i64 = 4; // xem `IME_STRATEGY_*` trong header C

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };

    let parsed = textvn_appdb::AppDb::parse(s);

    // ---- 1/2: không panic; nếu hợp lệ thì strategy vẫn thuộc ABI công bố ----
    if let Ok(db) = &parsed {
        for role in 0..=10u32 {
            let st = db.strategy_for("chrome.exe", role);
            if let Some(st) = st {
                assert!(
                    (0..=STRATEGY_MAX_ID).contains(&st.id()),
                    "strategy id {} ngoài tập đã công bố",
                    st.id()
                );
            }
        }
    }
});
