// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-168 / R2-169 / R2-170 — đối chiếu engine với **oracle x-unikey 1.0.4** (bản UniKey
//! cho Linux, `ukengine` build thật làm harness — `docs/specs/oracle-unikey.md` §2) và các
//! ca người dùng báo ở Ngó Sen (ngosen/ngosen#613, #958, #959).
//!
//! Mỗi dòng ghi rõ kết quả x-unikey (`uk`) và lý do khi TextVN cố ý khác:
//! - R2-168: chữ số dính liền sau từ biến đổi không có thật → trả phím (`Sose2026`) — như
//!   x-unikey bật `autoNonVnRestore`; âm tiết thật vẫn giữ (`cos2` → `có2`, như x-unikey mặc định).
//! - R2-169: vần `oo` chỉ đi với âm cuối `ng`/`c` (luật ghép vần `validate::oo_coda_ok`) ⇒
//!   `booots` → `boots` như x-unikey; mọi từ `oòng/oóc` thật vẫn gõ được (`goòng moóc soóc
//!   Thoòng voọc`, R2-63, OpenKey#41) — kể cả từ KHÔNG có trong từ điển âm tiết.
//! - R2-170: âm đầu `dz` (`dzô`, `dzũng`) — x-unikey gõ được.

use textvn_core::{keymap, Action, Engine, EngineOptions, KeyEvent, FLAG_CONSUMED};

fn type_stream(e: &mut Engine, keys: &str) -> String {
    let mut buf: Vec<char> = Vec::new();
    for c in keys.chars() {
        let k = match c {
            ' ' => KeyEvent {
                vk: keymap::vk::SPACE,
                ch: ' ' as u32,
                key_down: true,
                ..Default::default()
            },
            _ => KeyEvent::char_down(c),
        };
        let out = e.key(&k);
        match out.action {
            Action::Pass => {
                if out.flags & FLAG_CONSUMED == 0 {
                    buf.push(c);
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
                buf.truncate(buf.len().saturating_sub(delete_count as usize));
                buf.extend(insert);
            }
            Action::Commit { insert } => buf.extend(insert),
        }
    }
    buf.into_iter().collect()
}

fn run(keys: &str) -> String {
    let mut e = Engine::new(EngineOptions {
        auto_capitalize: false,
        ..Default::default()
    });
    type_stream(&mut e, keys)
}

/// (phím Telex, kỳ vọng TextVN, kết quả x-unikey 1.0.4 mặc định — để đối chiếu khi đọc)
const CASES: &[(&str, &str, &str)] = &[
    // R2-168 — chữ số dính liền
    (
        "Sose2026 ",
        "Sose2026 ",
        "Sóe2026 (autoNonVnRestore: Sose2026)",
    ),
    ("Wifi6 ", "Wifi6 ", "Wifi6"),
    ("cos2 ", "có2 ", "có2"),
    ("lowps12 ", "lớp12 ", "lớp12"),
    ("50000dd ", "50000đ ", "50000đ"),
    // R2-169 — hoàn tác rồi đánh dấu thanh
    ("booots ", "boots ", "boots"),
    ("gooongf ", "goòng ", "goongf (TextVN giữ từ thật R2-63)"),
    ("mooocs ", "moóc ", "moocs (TextVN giữ từ thật R2-63)"),
    ("sooocs ", "soóc ", "soocs (TextVN giữ từ thật R2-63)"),
    ("booongs ", "boóng ", "boongs"),
    ("Thooongf ", "Thoòng ", "Thoongf (OpenKey#41 muốn Thoòng)"),
    ("vooocj ", "voọc ", "voocj (OpenKey#41 muốn voọc)"),
    ("xooong ", "xoong ", "xoong"),
    ("ddd ", "dd ", "dd"),
    ("aaaa ", "aaa ", "aaa"),
    // R2-170 — âm đầu dz
    ("dzoo ", "dzô ", "dzô"),
    ("dzuxng ", "dzũng ", "dzũng"),
    ("Dzuxng ", "Dzũng ", "Dzũng"),
    ("dzaayj ", "dzậy ", "dzậy"),
    // Không đổi so với trước (khớp x-unikey) — chốt không hồi quy
    ("dduoxwcj ", "được ", "được"),
    ("looxi ", "lỗi ", "lỗi"),
    ("cowf ", "cờ ", "cờ"),
    ("tieengs vieetj ", "tiếng việt ", "tiếng việt"),
    ("thuowr ", "thuở ", "thuở"),
    (
        "huow ",
        "huơ ",
        "hươ (x-unikey sai; huơ đúng chính tả — OpenKey#229)",
    ),
    ("khuow ", "khuơ ", "khươ (OpenKey#229)"),
    ("quowrn ", "quởn ", "quởn (OpenKey#86)"),
    ("hoongf ", "hồng ", "hồng (OpenKey#114)"),
    ("ddutws ", "đứt ", "đứt (OpenKey#63)"),
    ("SDDT ", "SĐT ", "SĐT (OpenKey#48)"),
    ("[Toongr ", "[Tổng ", "oTongr (OpenKey#85)"),
    ("(hinhf ", "(hình ", "(hình (OpenKey#26)"),
    ("huychj ", "huỵch ", "huỵch (OpenKey#275)"),
    ("Next ", "Next ", "Nẽt (OpenKey#119)"),
    ("official ", "official ", "oficial (OpenKey#141)"),
    ("test1.c ", "test1.c ", "tét1.c (ibus-bamboo#429)"),
    ("JWWT ", "JWWT ", "JWWT (ibus-bamboo#267)"),
];

#[test]
fn r2_168_170_matches_unikey_oracle_or_documented_policy() {
    let mut bad = Vec::new();
    for &(keys, want, uk) in CASES {
        let got = run(keys);
        if got != want {
            bad.push(format!(
                "{keys:?}: got {got:?}, want {want:?} (x-unikey: {uk})"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "lệch oracle/chính sách:\n{}",
        bad.join("\n")
    );
}

/// R2-168 chỉ áp cho chữ số DÍNH LIỀN: cách một dấu cách thì từ đã đóng, giữ nguyên.
#[test]
fn r2_168_digit_after_space_keeps_word() {
    assert_eq!(run("Sose 2026 "), "Sóe 2026 ");
}

/// R2-168/169 tôn trọng công tắc `auto_restore_english` (tắt = kiểu UniKey cổ điển).
#[test]
fn r2_168_169_respect_auto_restore_switch() {
    let classic = || EngineOptions {
        auto_capitalize: false,
        auto_restore_english: false,
        ..Default::default()
    };
    let mut e = Engine::new(classic());
    assert_eq!(type_stream(&mut e, "Sose2026 "), "Sóe2026 ");
    let mut e = Engine::new(classic());
    assert_eq!(type_stream(&mut e, "booots "), "boót ");
}
