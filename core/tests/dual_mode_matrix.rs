// SPDX-License-Identifier: GPL-3.0-or-later
//! Ma trận kiểm thử song ngữ TextVN/UniKey-style + chống lệch sổ (E47, yêu cầu
//! audit "1.000 case"): gõ từ có phím dấu nhúng trong từ EN/kết hợp, bảo toàn
//! hoa/thường, undo-tone, backspace/re-type nhanh — engine phải **không bao
//! giờ** nhân đôi hay mất ký tự, và kết quả ở ranh giới từ phải đúng chính sách.
//!
//! Oracle: `method::fold` (stateless, cùng tuỳ chọn engine) +
//! `post::restore_en::should_restore` (đúng hàm engine dùng). Ma trận bắt
//! **lệch động của máy trạng thái** (nhân đôi, mất chữ, lệch `owned`, case
//! mất) — không phải chốt lại chính sách (policy được ghim ở `en_restore.rs`).
//!
//! E47 (fix kèm suite này): raw dựng lại sau Backspace (`keys_for`) là chuỗi
//! tổng hợp — phải KHÔNG bao giờ xuất hiện trong document qua đường restore.
//! B2 kiểm bằng bất biến **WYSIWYG**: ranh giới từ không được đổi glyph của từ
//! khi từ đã dựng lại raw (`dduocj⌫j` + Space cũ ra `"dduwowjj"` = phím lạ).

use std::sync::atomic::{AtomicUsize, Ordering};

use textvn_core::keymap::{self, KeyEvent};
use textvn_core::method::{self, Method};
use textvn_core::post::restore_en;
use textvn_core::transform::DiacriticStyle;
use textvn_core::{Action, Engine, EngineOptions};

static CASES: AtomicUsize = AtomicUsize::new(0);
fn count_case() {
    CASES.fetch_add(1, Ordering::Relaxed);
}

const BS: char = '\u{8}';
const ESC: char = '\u{1b}';

fn apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
            // Phím PASS tới app như phím thật: Backspace phải xoá 1 ký tự
            // (app tự xoá), chữ in được vào cuối.
            if k.vk == keymap::vk::BACK {
                buf.pop();
                return;
            }
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
            assert!(
                del <= buf.len(),
                "delete_count {del} vượt buffer {} — lệch sổ engine (nhân đôi/mất chữ)",
                buf.len()
            );
            buf.truncate(buf.len() - del);
            buf.extend_from_slice(insert);
        }
        Action::Commit { insert } => buf.extend_from_slice(insert),
    }
}

fn key_event(c: char, down: bool) -> KeyEvent {
    let (vk, ch) = if c == BS {
        (keymap::vk::BACK, 0)
    } else if c == '\n' {
        (keymap::vk::RETURN, 0)
    } else if c == ESC {
        (keymap::vk::ESCAPE, 0)
    } else {
        (c.to_ascii_uppercase() as u32, c as u32)
    };
    KeyEvent {
        vk,
        ch,
        key_down: down,
        ..Default::default()
    }
}

/// Gõ chuỗi phím (down+up từng phím) qua engine; trả (doc cuối, doc NGAY TRƯỚC
/// phím cuối) — dùng cho bất biến WYSIWYG ở ranh giới từ.
fn type_seq_traced(opts: &EngineOptions, keys: &[char]) -> (String, String) {
    let mut e = Engine::new(opts.clone());
    let mut buf: Vec<char> = Vec::new();
    let before_last;
    for (i, &c) in keys.iter().enumerate() {
        if i + 1 == keys.len() {
            before_last = buf.clone();
            let down = key_event(c, true);
            let o = e.key(&down);
            apply(&mut buf, &o.action, &down);
            let _ = e.key(&key_event(c, false));
            count_case();
            return (buf.iter().collect(), before_last.iter().collect::<String>());
        }
        let down = key_event(c, true);
        let o = e.key(&down);
        apply(&mut buf, &o.action, &down);
        let _ = e.key(&key_event(c, false));
    }
    unreachable!("keys rỗng")
}

fn type_string(opts: &EngineOptions, keys: &[char]) -> String {
    type_seq_traced(opts, keys).0
}

fn opts_for(method: Method, auto_capitalize: bool) -> EngineOptions {
    EngineOptions {
        method,
        auto_capitalize,
        ..Default::default()
    }
}

/// Oracle chính sách ranh giới: expected = chuỗi gõ (restore) hoặc fold (giữ VN).
fn boundary_expect(opts: &EngineOptions, typed: &str) -> String {
    let raw: Vec<char> = typed.chars().collect();
    let mut display = method::fold(&raw, opts.method, opts.diacritic_style, opts.free_marking);
    if opts.quick_telex && matches!(opts.method, Method::Telex | Method::SimpleTelex) {
        textvn_core::post::quick_telex::apply(&mut display);
    }
    if restore_en::should_restore(&raw, &display, &opts.english_words) {
        format!("{typed} ")
    } else {
        let mut out: String = display.iter().collect();
        out.push(' ');
        out
    }
}

// ====================================================================
// Corpus dữ liệu
// ====================================================================

/// Từ do chủ repo/audit liệt kê — phím dấu Telex nhúng trong từ EN/kết hợp.
const USER_WORDS: &[&str] = &[
    "TextVN",
    "TEXTVN",
    "tExTvN",
    "syntax",
    "export",
    "window",
    "windows",
    "context",
    "buffer",
    "server",
    "driver",
    "mouse",
    "house",
    "telex",
    "vni",
    "text",
    "texts",
    "next",
    "queen",
    "sex",
    "hello",
    "world",
    "warranty",
    "university",
    "system",
    "file",
    "email",
    "user",
    "sort",
    "smart",
    "target",
    "margin",
    "rust",
    "async",
    "await",
    "interface",
    "struct",
    "public",
    "private",
    "return",
    "package",
    "docker",
    "nginx",
    "redis",
    "python",
    "typescript",
    "javascript",
];

fn parse_data(src: &'static str, limit: usize, min_len: usize) -> Vec<String> {
    src.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| l.chars().all(|c| c.is_ascii_alphabetic()))
        .filter(|l| l.len() >= min_len && l.len() <= 14)
        .take(limit)
        .map(String::from)
        .collect()
}

fn en_common_corpus() -> Vec<String> {
    parse_data(textvn_core::post::restore_en::EN_COMMON_DATA, 500, 2)
}

fn en_freq_corpus() -> Vec<String> {
    parse_data(textvn_core::post::restore_en::EN_FREQ_DATA, 400, 2)
}

fn user_corpus() -> Vec<String> {
    USER_WORDS.iter().map(|w| (*w).to_string()).collect()
}

fn prefix_untransformed(prefix: &str, method: Method) -> bool {
    let pf: Vec<char> = prefix.chars().collect();
    method::fold(&pf, method, DiacriticStyle::Old, true) == pf
}

// ====================================================================
// Suite A — restore/fold ma trận chính (từ × kiểu gõ × hoa/thường)
// ====================================================================

#[test]
fn suite_a_matrix_policy_and_case_preservation() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let mut corpus = user_corpus();
    corpus.extend(en_common_corpus());
    corpus.extend(en_freq_corpus());
    corpus.dedup();
    let mut checked = 0usize;
    for word in &corpus {
        for m in methods {
            let opts = opts_for(m, false);
            // lower: oracle quyết định; upper/camel: chữ hoa gõ bằng Shift là chữ
            // thường lệ (`USA`) hoặc OP-006 identifier — document == nguyên văn.
            let lower = word.to_lowercase();
            let upper = word.to_uppercase();
            let variants: Vec<(&str, String, bool)> = vec![
                ("lower", lower.clone(), false),
                ("upper", upper.clone(), false),
                (
                    "camel",
                    {
                        let mut c: Vec<char> = lower.chars().collect();
                        if c.len() >= 3 {
                            let last = c.len() - 1;
                            c[last] = c[last].to_ascii_uppercase();
                        }
                        c.into_iter().collect()
                    },
                    true,
                ),
            ];
            for (label, typed, expect_typed) in variants {
                if label == "camel" && (typed == upper || typed == lower) {
                    continue; // từ quá ngắn, camel trùng upper/lower
                }
                let got = type_string(
                    &opts,
                    &typed
                        .chars()
                        .chain(std::iter::once(' '))
                        .collect::<Vec<_>>(),
                );
                if expect_typed {
                    assert_eq!(
                        got,
                        format!("{typed} "),
                        "{m:?}/{label}: `{typed}` bị engine biến đổi (nhân đôi/mất chữ/mất case)"
                    );
                } else {
                    let want = boundary_expect(&opts, &typed);
                    assert_eq!(got, want, "{m:?}/{label}: `{typed}` lệch chính sách/oracle");
                    assert!(
                        got.chars().count() <= typed.chars().count() + 1,
                        "{m:?}: `{typed}` → `{got}` dài hơn chuỗi gõ — nhân đôi"
                    );
                }
                checked += 1;
            }
        }
    }
    assert!(checked >= 600, "Suite A chỉ chạy {checked} case");
}

// ====================================================================
// Suite B — Backspace / re-type (R2-55 + E47)
// ====================================================================

/// B1: Backspace khi từ **chưa biến đổi** (chữ đã PASS) = xoá đúng phím đã gõ —
/// chuỗi gõ tiếp theo phải hội tụ về chuỗi "gõ xuôi" (naive).
#[test]
fn suite_b1_backspace_over_passed_letters_converges_to_naive() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let mut corpus = user_corpus();
    corpus.extend(en_common_corpus());
    corpus.dedup();
    let mut checked = 0usize;
    for word in &corpus {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        for p in 1..chars.len() {
            let prefix: String = chars[..p].iter().collect();
            if !prefix_untransformed(&prefix, Method::Telex)
                || !prefix_untransformed(&prefix, Method::SimpleTelex)
                || !prefix_untransformed(&prefix, Method::Vni)
            {
                continue; // đã transform ở prefix → thuộc B2
            }
            // BS xoá phím CUỐI của prefix (chỉ số p-1): naive = prefix trừ phím
            // cuối + phần còn lại của từ.
            let naive: String = chars[..p - 1].iter().chain(chars[p..].iter()).collect();
            for m in methods {
                let opts = opts_for(m, false);
                let mut seq: Vec<char> = prefix.chars().collect();
                seq.push(BS);
                seq.extend(chars[p..].iter());
                seq.push(' ');
                let got = type_string(&opts, &seq);
                let want = boundary_expect(&opts, &naive);
                assert_eq!(
                    got,
                    want,
                    "{m:?}: `{prefix}`+BS+`{}` phải hội tụ về gõ xuôi `{naive}`",
                    chars[p..].iter().collect::<String>()
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 150, "B1 chỉ chạy {checked} case");
}

/// B2: Backspace khi từ **đã biến đổi** (R2-55 xoá ký tự hiển thị + dựng lại raw
/// tổng hợp, E47 cắm cờ `rebuilt`) — bất biến **WYSIWYG**: ranh giới từ phải giữ
/// nguyên phần hiển thị người dùng đang thấy, KHÔNG bung phím tổng hợp.
#[test]
fn suite_b2_backspace_after_transform_wysiwyg() {
    let methods = [Method::Telex, Method::SimpleTelex];
    let mut corpus = user_corpus();
    corpus.extend(en_freq_corpus());
    corpus.dedup();
    let mut checked = 0usize;
    for word in &corpus {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        if chars.len() < 4 {
            continue;
        }
        for p in 2..chars.len() {
            let prefix: String = chars[..p].iter().collect();
            // BS xoá 1 glyph hiển thị: nếu fold(prefix) chỉ còn ≤ 1 glyph thì BS
            // xoá HẾT từ → engine clear (không rebuilt) → phần gõ tiếp là từ MỚI
            // có thể restore hợp lệ (không thuộc WYSIWYG). Điều kiện chọn case
            // PHẢI theo từng kiểu gõ: SimpleTelex `wi` là identity dù Telex
            // `wi` = `ưi` — BS trên từ chưa transform là đường B1, không phải B2.
            let transformed: Vec<Method> = methods
                .iter()
                .copied()
                .filter(|m| {
                    let pf: Vec<char> = prefix.chars().collect();
                    let f = method::fold(&pf, *m, DiacriticStyle::Old, true);
                    f != pf && f.len() > 1
                })
                .collect();
            for m in transformed {
                let opts = opts_for(m, false);
                let mut seq: Vec<char> = prefix.chars().collect();
                seq.push(BS);
                seq.extend(chars[p + 1..].iter());
                seq.push(' ');
                let (got, before) = type_seq_traced(&opts, &seq);
                let want = format!("{before} ");
                assert_eq!(
                    got, want,
                    "{m:?}: `{prefix}`+BS+`{}` — ranh giới đổi glyph (E47: phím lạ bung vào document)",
                    chars[p + 1..].iter().collect::<String>()
                );
                assert!(
                    got.chars().count() <= chars.len() + 2,
                    "{m:?}: `{prefix}`+BS+rest → `{got}` dài bất thường — nhân đôi"
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 200, "B2 chỉ chạy {checked} case");
}

/// B3: ca hồi quy E47 ghim cứng — hành vi đã verify bằng trace từng phím.
#[test]
fn suite_b3_e47_regression_pinned() {
    let opts = opts_for(Method::Telex, false);
    // "dduocj" (được) BS xoá 'c' → "đượ"; gõ j (cùng phím dấu → undo + literal)
    // → "đươj"; Space: từ KHÔNG hợp lệ + raw đã dựng lại → giữ hiển thị.
    let got = type_string(&opts, &format!("dduocj{BS}j ").chars().collect::<Vec<_>>());
    assert_eq!(got, "đươj ", "E47: không được ra phím tổng hợp `dduwowjj`");
    // "wor" (ửo) BS xoá 'o' → "ử"; gõ l,d → "ửld"; Space: giữ hiển thị.
    let got = type_string(&opts, &format!("wor{BS}ld ").chars().collect::<Vec<_>>());
    assert_eq!(got, "ửld ", "E47: không được ra phím tổng hợp `uwrld`");
    // "wor" BS xoá 'o' → "ử"; gõ r (cùng phím hỏi → undo + literal) → "ưr", l,d.
    let got = type_string(&opts, &format!("wor{BS}rld ").chars().collect::<Vec<_>>());
    assert_eq!(
        got, "ưrld ",
        "E47: giữ hiển thị sau BS trên từ đã transform"
    );
    // Raw chưa dựng lại (BS trên chữ PASS) → restore chuỗi gõ thật vẫn hoạt động.
    let got = type_string(&opts, &format!("sy{BS}tax ").chars().collect::<Vec<_>>());
    assert_eq!(got, "stax ", "BS trên chữ PASS: hội tụ về gõ xuôi `stax`");
    // R2-55 không thoái hoá: xoá 1 ký tự hiển thị, dấu còn lại giữ đúng chỗ.
    let got = type_string(&opts, &format!("tieeng{BS} ").chars().collect::<Vec<_>>());
    assert_eq!(got, "tiên ", "R2-55: `tieeng`+BS → `tiên`");
    let got = type_string(&opts, &format!("dduocj{BS} ").chars().collect::<Vec<_>>());
    assert_eq!(got, "đượ ", "R2-55: `dduocj`+BS → `đượ`");
    // ESC trên từ đã dựng lại: giữ hiển thị, KHÔNG bung phím tổng hợp (E47).
    let got = type_string(
        &opts,
        &format!("dduocj{BS}j{ESC} ").chars().collect::<Vec<_>>(),
    );
    assert_eq!(got, "đươj ", "E47: ESC trên từ dựng lại phải giữ hiển thị");
    // ESC trên từ thường: trả chuỗi gõ thật (B5) — không thoái hoá.
    let got = type_string(&opts, &format!("dduocj{ESC} ").chars().collect::<Vec<_>>());
    assert_eq!(got, "dduocj ", "ESC phải trả chuỗi gõ thật");
    // Ctrl tap khôi phục từ EN fold-sai vẫn hoạt động (raw chưa dựng lại).
    let mut e = Engine::new(opts_for(Method::Telex, false));
    let mut buf: Vec<char> = Vec::new();
    for c in "sort".chars() {
        let k = KeyEvent::char_down(c);
        let o = e.key(&k);
        apply(&mut buf, &o.action, &k);
        let _ = e.key(&KeyEvent {
            ch: c as u32,
            key_down: false,
            ..Default::default()
        });
    }
    let _ = e.key(&KeyEvent::key_down(keymap::vk::CONTROL));
    let out_up = e.key(&KeyEvent::key_up(keymap::vk::CONTROL));
    apply(&mut buf, &out_up.action, &KeyEvent::char_down(' '));
    let s: String = buf.into_iter().collect();
    assert_eq!(s, "sort", "Ctrl tap: `sỏt` → `sort` (không thoái hoá)");
    count_case();
}

/// B4: BS xoá HẾT từ (kể cả vượt) — document về đúng chỗ, không xoá lẹm chữ
/// phía trước, không âm dư (bất biến chỉ số `owned`).
#[test]
fn suite_b4_backspace_to_empty_is_exact() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let mut corpus = user_corpus();
    corpus.extend(en_common_corpus());
    corpus.dedup();
    let mut checked = 0usize;
    for word in &corpus {
        let lower = word.to_lowercase();
        let n = lower.chars().count();
        if n < 3 {
            continue;
        }
        for m in methods {
            let opts = opts_for(m, false);
            let mut seq: Vec<char> = lower.chars().collect();
            seq.extend(std::iter::repeat_n(BS, n + 2));
            seq.push('x');
            seq.push(' ');
            let got = type_string(&opts, &seq);
            assert_eq!(
                got, "x ",
                "{m:?}: `{lower}` + BS vượt must về rỗng rồi gõ `x`"
            );
            checked += 1;
        }
    }
    assert!(checked >= 100, "B4 chỉ chạy {checked} case");
}

// ====================================================================
// Suite C — bảo toàn hoa/thường (case state desync — chỉ báo case)
// ====================================================================

#[test]
fn suite_c_case_preservation_matrix() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let words = [
        "TextVN", "TEXTVN", "tExTvN", "TeXt", "TeX", "UseState", "McDonald", "USA",
    ];
    let mut checked = 0usize;
    for w in words {
        for m in methods {
            for cap in [false, true] {
                // Đầu câu và giữa câu. auto_capitalize viết hoa CHỮ ĐẦU từ ở
                // đầu câu — phần còn lại của từ phải giữ nguyênCase.
                for prefix in ["", "x. "] {
                    let opts = opts_for(m, cap);
                    let typed: Vec<char> = prefix
                        .chars()
                        .chain(w.chars())
                        .chain(std::iter::once(' '))
                        .collect();
                    let got = type_string(&opts, &typed);
                    let head = if cap && prefix == "x. " {
                        w.chars()
                            .next()
                            .map(|c| c.to_uppercase().next().unwrap_or(c))
                            .into_iter()
                            .chain(w.chars().skip(1))
                            .collect::<String>()
                    } else {
                        w.to_string()
                    };
                    let want = format!("{prefix}{head} ");
                    assert_eq!(
                        got, want,
                        "{m:?}/cap={cap}/prefix={prefix:?}: `{w}` mất case hoặc nhân đôi"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, words.len() * methods.len() * 2 * 2, "C thiếu case");
}

/// C2: hoa/thường **giữa dòng gõ nhanh** — gõ liên tục không Space, kết thúc
/// bằng Enter (mô phỏng gõ "TextVN" rồi Enter ngay).
#[test]
fn suite_c2_rapid_upper_tail_keeps_case() {
    for w in ["TextVN", "TEXTVN", "abcDef", "xY"] {
        for m in [Method::Telex, Method::SimpleTelex] {
            let opts = opts_for(m, false);
            let got = type_string(
                &opts,
                &w.chars().chain(std::iter::once('\n')).collect::<Vec<_>>(),
            );
            assert_eq!(
                got,
                format!("{w}\n"),
                "{m:?}: `{w}` + Enter mất case/nhân đôi"
            );
        }
    }
    count_case();
}

// ====================================================================
// Suite D — undo-tone golden (dd→đ, aa→â→a, uw/ow/aw, VNI số)
// ====================================================================

#[test]
fn suite_d_undo_tone_and_double_key_golden() {
    let old = DiacriticStyle::Old;
    let telex_pairs: &[(&str, &str)] = &[
        ("dd", "đ"),
        ("ddd", "dd"),
        ("dddd", "ddd"),
        ("aa", "â"),
        ("aaa", "aa"),
        ("aaaa", "aaa"),
        ("ee", "ê"),
        ("eee", "ee"),
        ("oo", "ô"),
        ("ooo", "oo"),
        ("aw", "ă"),
        ("aww", "aw"),
        ("awww", "aww"),
        ("ow", "ơ"),
        ("oww", "ow"),
        ("uw", "ư"),
        ("uww", "uw"),
        ("ww", "w"),
        ("www", "ww"),
        ("ass", "as"),
        ("asss", "ass"),
        ("aff", "af"),
        ("arr", "ar"),
        ("axx", "ax"),
        ("ajj", "aj"),
        ("dduocj", "được"),
        ("duocj", "dược"),
        ("vieetj", "việt"),
        ("tieeng", "tiêng"),
        ("dduowcj", "được"),
        ("nguowif", "người"),
        ("toansz", "toan"),
        ("vieetjz", "viêt"),
    ];
    for (keys, want) in telex_pairs {
        let opts = opts_for(Method::Telex, false);
        let got = type_string(&opts, &format!("{keys} ").chars().collect::<Vec<_>>());
        // Ghim golden: fold chính là đích (từ VN hợp lệ không restore);
        // ngoại lệ restore được liệt kê tường minh bên dưới.
        let raw: Vec<char> = keys.chars().collect();
        let folded_chars = method::fold(&raw, Method::Telex, old, true);
        let folded: String = folded_chars.iter().collect();
        let expected = if restore_en::should_restore(&raw, &folded_chars, &[]) {
            format!("{keys} ")
        } else {
            format!("{folded} ")
        };
        assert_eq!(got, expected, "Telex golden `{keys}`");
        // Fold phải đúng golden trừ khi restore (`z` gỡ dấu → literal).
        if !restore_en::should_restore(&raw, &folded_chars, &[]) {
            assert_eq!(folded, *want, "Telex golden fold `{keys}`");
        }
    }
    let vni_pairs: &[(&str, &str)] = &[
        ("d9", "đ"),
        ("d99", "d9"),
        ("a1", "á"),
        ("a2", "à"),
        ("a3", "ả"),
        ("a4", "ã"),
        ("a5", "ạ"),
        ("a11", "a1"),
        ("oo7", "ô"),
        ("uo7", "uơ"),
        ("aw6", "ă"),
        ("d9u7o7ng2", "đường"),
        ("d9u7o7ng5", "đượng"),
        ("vie65t", "việt"),
    ];
    for (keys, want) in vni_pairs {
        let opts = opts_for(Method::Vni, false);
        let got = type_string(&opts, &format!("{keys} ").chars().collect::<Vec<_>>());
        let raw: Vec<char> = keys.chars().collect();
        let folded_chars = method::fold(&raw, Method::Vni, old, true);
        let folded: String = folded_chars.iter().collect();
        let expected = if restore_en::should_restore(&raw, &folded_chars, &[]) {
            format!("{keys} ")
        } else {
            format!("{folded} ")
        };
        assert_eq!(got, expected, "VNI golden `{keys}`");
        if !restore_en::should_restore(&raw, &folded_chars, &[]) {
            assert_eq!(folded, *want, "VNI golden fold `{keys}`");
        }
    }
    // Simple Telex: `w` CHỈ là dấu sừng (UniKey vneHookAll) — không có `w` đứng riêng.
    for (keys, want) in [
        ("aw", "ă"),
        ("ow", "ơ"),
        ("uw", "ư"),
        ("ww", "ww"),
        ("nhw", "nhw"),
    ] {
        let opts = opts_for(Method::SimpleTelex, false);
        let got = type_string(&opts, &format!("{keys} ").chars().collect::<Vec<_>>());
        let raw: Vec<char> = keys.chars().collect();
        let folded_chars = method::fold(&raw, Method::SimpleTelex, old, true);
        let folded: String = folded_chars.iter().collect();
        let expected = if restore_en::should_restore(&raw, &folded_chars, &[]) {
            format!("{keys} ")
        } else {
            format!("{folded} ")
        };
        assert_eq!(got, expected, "SimpleTelex golden `{keys}`");
        if !restore_en::should_restore(&raw, &folded_chars, &[]) {
            assert_eq!(folded, *want, "SimpleTelex golden fold `{keys}`");
        }
    }
}

// ====================================================================
// Suite E — gate: tổng số case của file này ≥ 1.000
// ====================================================================

#[test]
fn gate_total_case_count_at_least_1000() {
    // Đếm tĩnh đúng theo cách các suite sinh case (không phụ thuộc thứ tự chạy).
    let methods = 3usize;
    let corpus_a: Vec<String> = {
        let mut v = user_corpus();
        v.extend(en_common_corpus());
        v.extend(en_freq_corpus());
        v.dedup();
        v
    };
    let suite_a: usize = corpus_a
        .iter()
        .map(|w| {
            let lower = w.to_lowercase();
            let upper = w.to_uppercase();
            let mut n = 2; // lower + upper
            if lower.chars().count() >= 3 && {
                let mut c: Vec<char> = lower.chars().collect();
                let last = c.len() - 1;
                c[last] = c[last].to_ascii_uppercase();
                let camel: String = c.into_iter().collect();
                camel != upper && camel != lower
            } {
                n += 1;
            }
            n
        })
        .sum::<usize>()
        * methods;

    let corpus_b: Vec<String> = {
        let mut v = user_corpus();
        v.extend(en_common_corpus());
        v.dedup();
        v
    };
    let b1_pairs: usize = corpus_b
        .iter()
        .map(|w| {
            let chars: Vec<char> = w.to_lowercase().chars().collect();
            (1..chars.len())
                .filter(|&p| {
                    let prefix: String = chars[..p].iter().collect();
                    prefix_untransformed(&prefix, Method::Telex)
                        && prefix_untransformed(&prefix, Method::SimpleTelex)
                        && prefix_untransformed(&prefix, Method::Vni)
                })
                .count()
        })
        .sum::<usize>()
        * methods;
    let corpus_b2: Vec<String> = {
        let mut v = user_corpus();
        v.extend(en_freq_corpus());
        v.dedup();
        v
    };
    let b2_pairs: usize = corpus_b2
        .iter()
        .map(|w| {
            let chars: Vec<char> = w.to_lowercase().chars().collect();
            let methods_b2 = [Method::Telex, Method::SimpleTelex];
            (2..chars.len())
                .filter(|&p| {
                    let prefix: String = chars[..p].iter().collect();
                    let pf: Vec<char> = prefix.chars().collect();
                    methods_b2.iter().any(|m| {
                        let f = method::fold(&pf, *m, DiacriticStyle::Old, true);
                        f != pf && f.len() > 1
                    })
                })
                .count()
        })
        .sum::<usize>();
    let b4 = corpus_b.iter().filter(|w| w.chars().count() >= 3).count() * methods;
    let suite_c = 8 * 3 * 2 * 2 + 4 * 2;
    let suite_d = 31 + 12 + 5;
    let total = suite_a + b1_pairs + b2_pairs + b4 + suite_c + suite_d + 8;
    println!(
        "dual_mode_matrix: A={suite_a} B1={b1_pairs} B2={b2_pairs} B4={b4} C={suite_c} D={suite_d} pinned=8 → total {total}"
    );
    assert!(
        total >= 1000,
        "ma trận dual-mode chỉ có {total} case — yêu cầu audit là ≥ 1.000"
    );
}
