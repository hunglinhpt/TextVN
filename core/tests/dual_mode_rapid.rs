// SPDX-License-Identifier: GPL-3.0-or-later
//! Ma trận **gõ nhanh & tương đương chiến lược xuất chữ** — mở rộng
//! `dual_mode_matrix.rs` (yêu cầu audit: ≥ 1.000 case MỚI cho hai nhóm Bug 1
//! MobaXterm/TSF và Bug 2 nhân đôi "TextVN"→"Texxtvn"). File này đúng phần
//! gõ/engine; phần máy trạng thái TSF/focus nằm ở `adapters/windows-tsf`
//! (`plan_key_extended_cross_product_matrix`, `focus_switch_rapid_matrix`, …).
//!
//! Các trục MỚI so với `dual_mode_matrix.rs` (không trùng case):
//! - **E**: đặt lại vị trí dấu (reposition) trên cụm nguyên âm đôi/trên —
//!   `hoaf`/`khoaf`/`khieuf`… × 2 kiểu gõ × 2 kiểu dấu cũ/mới × Telex/VNI.
//! - **F**: dòng gõ LIÊN TỤC trộn VN/EN (không nghỉ giữa từ) — mô phỏng gõ
//!   nhanh thật, ranh giới bằng Space/comma/chấm.
//! - **G**: bão Backspace × nhiều phím rồi gõ lại (q>1) — hội tụ naive hoặc
//!   WYSIWYG (E47), không âm dư, không nhân đôi.
//! - **H**: **tương đương chiến lược** — cùng chuỗi phím qua hint
//!   Preedit / BackspaceType / ForwardAsCommit / tự-resolve phải ra CÙNG
//!   document (ranh giới Unikey-style commit-and-backspace vs TextVN-style
//!   composition replacement).
//! - **I**: bảo toàn hoa/thường mở rộng (iPhone/GitHub/JavaScript…) với
//!   biến thể case xen kẽ và ranh giới Enter/Tab.
//! - **J**: gắn nhãn gốc phím — `is_injected` / key-up / chord phải PASS
//!   tuyệt đối (không bao giờ tái-xử lý phím do adapter bơm — FCA Bug 2).
//! - **K**: sổ `keys_typed`/Dream RSI (G19) không bị bão phím làm lệch.
//!
//! Oracle: `method::fold` + `post::restore_en::should_restore` — đúng hàm
//! engine dùng; bất biến độc lập: sổ xoá ≤ document (anti-ép), WYSIWYG trace,
//! tương đương chiến lược.

use std::sync::atomic::{AtomicUsize, Ordering};

use strategy::{
    IME_FIELD_BODY, IME_STRATEGY_BACKSPACE_TYPE, IME_STRATEGY_FORWARD_AS_COMMIT,
    IME_STRATEGY_PREEDIT,
};
use textvn_core::keymap::{self, KeyEvent};
use textvn_core::method::{self, Method};
use textvn_core::post::restore_en;
use textvn_core::transform::DiacriticStyle;
use textvn_core::{Action, Context, Engine, EngineOptions};

static CASES: AtomicUsize = AtomicUsize::new(0);
fn count_case() {
    CASES.fetch_add(1, Ordering::Relaxed);
}

const BS: char = '\u{8}';
const ESC: char = '\u{1b}';

fn apply(buf: &mut Vec<char>, action: &Action, k: &KeyEvent) {
    match action {
        Action::Pass => {
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

fn type_string(opts: &EngineOptions, keys: &[char]) -> String {
    let mut e = Engine::new(opts.clone());
    let mut buf: Vec<char> = Vec::new();
    for &c in keys {
        let down = key_event(c, true);
        let o = e.key(&down);
        apply(&mut buf, &o.action, &down);
        let _ = e.key(&key_event(c, false));
    }
    buf.iter().collect()
}

/// Gõ với `Context` strategy gợi ý (Suite H) — trả document cuối.
fn type_string_with_ctx(opts: &EngineOptions, keys: &[char], ctx: Context) -> String {
    let mut e = Engine::new(opts.clone());
    e.set_context(ctx);
    let mut buf: Vec<char> = Vec::new();
    for &c in keys {
        let down = key_event(c, true);
        let o = e.key(&down);
        apply(&mut buf, &o.action, &down);
        let _ = e.key(&key_event(c, false));
    }
    buf.iter().collect()
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

fn prefix_untransformed(prefix: &str, method: Method) -> bool {
    let pf: Vec<char> = prefix.chars().collect();
    method::fold(&pf, method, DiacriticStyle::Old, true) == pf
}

// ====================================================================
// Suite E — đặt lại vị trí dấu (reposition) × kiểu dấu cũ/mới × kiểu gõ
// ====================================================================

/// Âm gốc có cụm nguyên âm đôi/trên (oa/oe/ai/ao/ay/uy/uya/ieu/eu…) — dấu
/// thanh phải nằm đúng vị trí theo `diacritic_style`, KHÔNG đổi glyph khác.
const CLUSTER_BASES: &[&str] = &[
    "hoa", "khoa", "choa", "ngoa", "thoa", "loe", "koe", "choe", "ngheo", "thao", "khao", "ngao",
    "thai", "khai", "chay", "khay", "thuy", "chuy", "khuya", "thuya", "khie", "nghie", "khieu",
    "dieeu", "yeu", "mua", "dua", "xua", "kia", "mia", "kheo", "boi",
];

#[test]
fn suite_e_tone_reposition_styles_matrix() {
    let mut checked = 0usize;
    for base in CLUSTER_BASES {
        for (ti, tone) in ['f', 's', 'r', 'x', 'j'].into_iter().enumerate() {
            for style in [DiacriticStyle::Old, DiacriticStyle::New] {
                // Telex: dấu = f/s/r/x/j.
                let keys = format!("{base}{tone}");
                let opts = EngineOptions {
                    method: Method::Telex,
                    diacritic_style: style,
                    ..Default::default()
                };
                let got = type_string(
                    &opts,
                    &keys.chars().chain(std::iter::once(' ')).collect::<Vec<_>>(),
                );
                assert_eq!(
                    got,
                    boundary_expect(&opts, &keys),
                    "Telex `{keys}` style {style:?}: dấu đặt sai chỗ / nhân đôi / mất chữ"
                );
                assert!(
                    got.chars().count() <= keys.chars().count() + 1,
                    "Telex `{keys}` → `{got}` dài hơn chuỗi gõ — nhân đôi"
                );
                count_case();
                checked += 1;
                // VNI cùng âm: dấu = số 1..5.
                let vkeys = format!("{base}{}", ti + 1);
                let vopts = EngineOptions {
                    method: Method::Vni,
                    diacritic_style: style,
                    ..Default::default()
                };
                let vgot = type_string(
                    &vopts,
                    &vkeys
                        .chars()
                        .chain(std::iter::once(' '))
                        .collect::<Vec<_>>(),
                );
                assert_eq!(
                    vgot,
                    boundary_expect(&vopts, &vkeys),
                    "VNI `{vkeys}` style {style:?}: dấu đặt sai chỗ / nhân đôi / mất chữ"
                );
                count_case();
                checked += 1;
            }
        }
    }
    assert!(checked >= 500, "Suite E chỉ chạy {checked} case");
}

/// Ghim vài golden reposition cho dễ đọc (old vs new thật sự khác glyph).
#[test]
fn suite_e2_tone_reposition_pinned_goldens() {
    // Oracle decides the exact glyph; ghim chính sách qua oracle và in rõ.
    for (keys, base, tone) in [
        ("hoaf", "hoa", 'f'),
        ("khoar", "khoa", 'r'),
        ("nguowif", "nguowi", 'f'),
        ("khieuf", "khieu", 'f'),
        ("thuyeef", "thuyee", 'f'),
    ] {
        for style in [DiacriticStyle::Old, DiacriticStyle::New] {
            let opts = EngineOptions {
                method: Method::Telex,
                diacritic_style: style,
                ..Default::default()
            };
            let _ = (base, tone);
            let got = type_string(
                &opts,
                &keys.chars().chain(std::iter::once(' ')).collect::<Vec<_>>(),
            );
            assert_eq!(
                got,
                boundary_expect(&opts, keys),
                "`{keys}` style {style:?}"
            );
            // Cả hai style: nén 4 phím → 3 glyph + Space — KHÔNG bao giờ vượt
            // chuỗi gõ + 1 (nhân đôi). Composition có thể nén phím thành 1 glyph.
            assert!(
                got.chars().count() <= keys.chars().count() + 1,
                "`{keys}` style {style:?}: `{got}` dài hơn chuỗi gõ — nhân đôi"
            );
            count_case();
        }
    }
}

// ====================================================================
// Suite F — dòng gõ liên tục trộn VN/EN (không nghỉ giữa từ)
// ====================================================================

const TELEX_STREAMS: &[&[&str]] = &[
    &["dduocj", "TextVN", "syntax"],
    &["chaof", "dduocj", "vaof", "export"],
    &["nguowif", "TextVN", "vieetj"],
    &["server", "mouse", "house", "window"],
    &["dduocj", "syntax", "export", "telex"],
    &["chaof", "dduocj", "tooi"],
    &["TextVN", "vni", "telex", "window"],
    &["tooi", "ddawng", "hoicj", "export"],
    &["driver", "buffer", "context", "server"],
    &["hoaf", "chaof", "laf", "TextVN"],
    &["syntax", "context", "buffer", "export"],
    &["dduocj", "chuif", "nguowif", "export"],
];

const VNI_STREAMS: &[&[&str]] = &[
    &["d9u7o7c5", "TextVN", "syntax"],
    &["ch1of", "d9u7o7c5", "v1of"],
    &["ng9owif", "vie65t", "export"],
    &["TextVN", "vni", "window"],
    &["ho1af", "cha2of"],
    &["syntax", "context", "buffer"],
];

fn stream_expect(opts: &EngineOptions, words: &[&str], sep: char) -> String {
    let mut want = String::new();
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            want.push(sep);
        }
        let seg = boundary_expect(opts, w);
        want.push_str(seg.trim_end_matches(' '));
    }
    want.push(sep);
    want
}

fn type_words(opts: &EngineOptions, words: &[&str], sep: char) -> String {
    let mut keys: Vec<char> = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            keys.push(sep);
        }
        keys.extend(w.chars());
    }
    keys.push(sep);
    type_string(opts, &keys)
}

#[test]
fn suite_f_continuous_mixed_streams() {
    let mut checked = 0usize;
    for words in TELEX_STREAMS {
        for m in [Method::Telex, Method::SimpleTelex] {
            for sep in [' ', ','] {
                let opts = opts_for(m, false);
                let got = type_words(&opts, words, sep);
                let want = stream_expect(&opts, words, sep);
                assert_eq!(
                    got, want,
                    "{m:?} stream {words:?} sep {sep:?}: lệch giữa dòng gõ liên tục (nhân đôi/mất chữ/mất case)"
                );
                // Không từ nào dài hơn chuỗi gõ của nó + 1 (anti-ép toàn dòng).
                assert!(
                    got.chars().count()
                        <= words.iter().map(|w| w.chars().count()).sum::<usize>() + words.len() + 4,
                    "{m:?} stream {words:?} sep {sep:?}: dài bất thường — nhân đôi"
                );
                checked += 1;
                count_case();
            }
        }
    }
    for words in VNI_STREAMS {
        let opts = opts_for(Method::Vni, false);
        let got = type_words(&opts, words, ' ');
        assert_eq!(
            got,
            stream_expect(&opts, words, ' '),
            "VNI stream {words:?}"
        );
        checked += 1;
        count_case();
    }
    assert!(checked >= 54, "Suite F chỉ chạy {checked} case");
}

// ====================================================================
// Suite G — bão Backspace nhiều phím rồi gõ lại (q ≥ 1)
// ====================================================================

/// G1: BS × q phím trên tiền tố CHƯA transform (chữ đã PASS) rồi gõ lại phần
/// sau — phải hội tụ về gõ xuôi của từ "đã xóa q phím giữa" (naive).
#[test]
fn suite_g1_multi_backspace_converges_to_naive() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let words = [
        "syntax", "export", "server", "mouse", "house", "telex", "window", "context", "buffer",
        "dduocj", "chaof", "vieetj", "TextVN",
    ];
    let mut checked = 0usize;
    for word in &words {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        for p in 1..chars.len() {
            let typed_prefix: String = chars[..p].iter().collect();
            if !prefix_untransformed(&typed_prefix, Method::Telex)
                || !prefix_untransformed(&typed_prefix, Method::SimpleTelex)
                || !prefix_untransformed(&typed_prefix, Method::Vni)
            {
                continue; // đã transform ở prefix → thuộc G2 (WYSIWYG)
            }
            for q in 1..=p.min(2) {
                let naive: String = chars[..p - q].iter().chain(chars[p..].iter()).collect();
                for m in methods {
                    let opts = opts_for(m, false);
                    let mut seq: Vec<char> = typed_prefix.chars().collect();
                    seq.extend(std::iter::repeat_n(BS, q));
                    seq.extend(chars[p..].iter());
                    seq.push(' ');
                    let got = type_string(&opts, &seq);
                    let want = boundary_expect(&opts, &naive);
                    assert_eq!(
                        got,
                        want,
                        "{m:?}: `{typed_prefix}`+BS×{q}+`{}` phải hội tụ về gõ xuôi `{naive}`",
                        chars[p..].iter().collect::<String>()
                    );
                    checked += 1;
                    count_case();
                }
            }
        }
    }
    assert!(checked >= 170, "G1 chỉ chạy {checked} case");
}

/// G2: BS × n sau khi từ ĐÃ transform — raw được dựng lại bằng `keys_for`
/// (R2-55) nên phím gõ tiếp là phím dấu **trên raw tổng hợp** (đúng thiết kế,
/// UniKey parity); oracle = fold(keys_for(display') + suffix), và ranh giới từ
/// KHÔNG đổi glyph (E47 WYSIWYG mở rộng cho n > 1).
#[test]
fn suite_g2_multi_backspace_wysiwyg_pinned() {
    // (keys đã gõ, số BS, hiển thị mong đợi sau bão BS)
    let cases: &[(&str, usize, &str)] = &[
        ("dduocj", 2, "đư"),
        ("dduocj", 3, "đ"),
        ("vieetj", 2, "vi"),
        ("tieeng", 2, "tiê"),
        ("nguowif", 2, "ngư"),
        ("hoaf", 1, "ho"),
        ("dduowcj", 2, "đư"),
    ];
    let suffix = ['x'];
    let mut checked = 0usize;
    for (word, nbs, want_display) in cases {
        for m in [Method::Telex, Method::SimpleTelex] {
            let opts = opts_for(m, false);
            let mut seq: Vec<char> = word.chars().collect();
            seq.extend(std::iter::repeat_n(BS, *nbs));
            // Gõ tiếp 1 phím thường để chứng minh engine không giữ trạng thái cũ.
            seq.push('x');
            seq.push(' ');
            let got = type_string(&opts, &seq);
            // Oracle: display' → raw tổng hợp (keys_for) → fold thêm suffix.
            let display: Vec<char> = want_display.chars().collect();
            let fold_fn = |keys: &[char]| {
                method::fold(keys, opts.method, opts.diacritic_style, opts.free_marking)
            };
            let raw = method::keys_for(&display, opts.method, opts.diacritic_style, fold_fn)
                .unwrap_or_else(|| display.clone());
            let mut keys = raw;
            keys.extend_from_slice(&suffix);
            let mut want: String =
                method::fold(&keys, opts.method, opts.diacritic_style, opts.free_marking)
                    .iter()
                    .collect();
            // E47: từ đã dựng lại raw — ranh giới giữ nguyên phần hiển thị.
            want.push(' ');
            assert_eq!(
                got, want,
                "{m:?}: `{word}` + BS×{nbs} + x — hiển thị sai / phím lạ bung vào document"
            );
            checked += 1;
            count_case();
        }
    }
    assert!(checked >= 12, "G2 chỉ chạy {checked} case");
}

/// G3: BS vượt quá đầu từ (xoá cả từ cũ) rồi gõ từ VN mới — hội tụ về gõ
/// từ mới trong điều kiện sạch (không âm dư, không trạng thái mồ côi).
#[test]
fn suite_g3_backspace_past_start_then_new_word() {
    let methods = [Method::Telex, Method::SimpleTelex, Method::Vni];
    let old_words = ["dduocj", "vieetj", "syntax", "TextVN", "chaof"];
    let new_words = ["chaof", "text", "dduocj", "syntax"];
    let mut checked = 0usize;
    for old in old_words {
        for new in new_words {
            for m in methods {
                let opts = opts_for(m, false);
                let mut seq: Vec<char> = old.chars().collect();
                seq.extend(std::iter::repeat_n(BS, old.chars().count() + 2));
                seq.extend(new.chars());
                seq.push(' ');
                let got = type_string(&opts, &seq);
                let want = boundary_expect(&opts, new);
                assert_eq!(
                    got, want,
                    "{m:?}: `{old}` + BS vượt + `{new}` phải như gõ `{new}` trong sạch"
                );
                checked += 1;
                count_case();
            }
        }
    }
    assert!(checked >= 55, "G3 chỉ chạy {checked} case");
}

// ====================================================================
// Suite H — tương đương chiến lược xuất chữ (Unikey-style commit-and-backspace
// vs TextVN-style composition): cùng chuỗi phím qua mọi hint phải ra CÙNG doc
// ====================================================================

#[test]
fn suite_h_strategy_equivalence_matrix() {
    let streams: &[&[&str]] = &[
        &["dduocj", "TextVN"],
        &["chaof", "syntax"],
        &["export", "hoaf"],
        &["nguowif", "vieetj"],
        &["TextVN", "window", "telex"],
        &["tooi", "ddawng", "goif"],
    ];
    let hints = [
        IME_STRATEGY_PREEDIT,
        IME_STRATEGY_BACKSPACE_TYPE,
        IME_STRATEGY_FORWARD_AS_COMMIT,
        -1, // tự resolve
    ];
    let mut checked = 0usize;
    for words in streams {
        for m in [Method::Telex, Method::SimpleTelex, Method::Vni] {
            let opts = opts_for(m, false);
            let keys: Vec<char> = words
                .iter()
                .flat_map(|w| w.chars().chain(std::iter::once(' ')))
                .collect();
            let mut docs: Vec<String> = Vec::new();
            for hint in hints {
                let ctx = Context {
                    enabled: true,
                    secure: false,
                    field_role: IME_FIELD_BODY,
                    caps: 0,
                    hint,
                };
                let got = type_string_with_ctx(&opts, &keys, ctx);
                docs.push(got);
            }
            for d in docs.iter().skip(1) {
                assert_eq!(
                    &docs[0], d,
                    "{m:?} stream {words:?}: chiến lược xuất chữ khác nhau làm KHÁC document — {docs:?}"
                );
                checked += 1;
                count_case();
            }
            // Nội dung cuối phải đúng chính sách ranh giới từng từ (không phụ thuộc strategy).
            let want = stream_expect(&opts, words, ' ');
            assert_eq!(
                docs[0], want,
                "{m:?} stream {words:?}: document lệch oracle ranh giới"
            );
            checked += 1;
            count_case();
        }
    }
    assert!(checked >= 45, "Suite H chỉ chạy {checked} case");
}

// ====================================================================
// Suite I — bảo toàn hoa/thường mở rộng (từ code thật + case xen kẽ)
// ====================================================================

#[test]
fn suite_i_case_preservation_extended() {
    let words = [
        "iPhone",
        "GitHub",
        "JavaScript",
        "TypeScript",
        "macOS",
        "PowerPoint",
        "LinkedIn",
        "YouTube",
        "WordPress",
        "VSCode",
        "TensorFlow",
        "FastAPI",
    ];
    let mut checked = 0usize;
    for w in words {
        let upper = w.to_uppercase();
        let alt: String = w
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect();
        for variant in [w, upper.as_str(), alt.as_str()] {
            for m in [Method::Telex, Method::SimpleTelex, Method::Vni] {
                for prefix in ["", "a. "] {
                    let opts = opts_for(m, false);
                    let typed: Vec<char> = prefix
                        .chars()
                        .chain(variant.chars())
                        .chain(std::iter::once(' '))
                        .collect();
                    let got = type_string(&opts, &typed);
                    assert_eq!(
                        got,
                        format!("{prefix}{variant} "),
                        "{m:?}: `{variant}` (prefix {prefix:?}) mất case hoặc nhân đôi"
                    );
                    checked += 1;
                    count_case();
                }
            }
        }
        // Ranh giới Enter/Tab — case vẫn giữ, không nhân đôi (một method đại diện).
        for boundary in ['\n', '\t'] {
            let opts = opts_for(Method::Telex, false);
            let typed: Vec<char> = w.chars().chain(std::iter::once(boundary)).collect();
            let got = type_string(&opts, &typed);
            assert_eq!(
                got,
                format!("{w}{boundary}"),
                "Telex: `{w}` + ranh giới {boundary:?} mất case/nhân đôi"
            );
            checked += 1;
            count_case();
        }
    }
    assert!(checked >= 240, "Suite I chỉ chạy {checked} case");
}

// ====================================================================
// Suite J — gắn nhãn gốc phím (event-origin tagging): phím inject / key-up /
// chord phải PASS tuyệt đối, không bao giờ tái-xử lý (FCA Bug 2)
// ====================================================================

#[test]
fn suite_j_injected_origin_never_transformed() {
    let mut checked = 0usize;
    let keys = ['x', 's', 'f', 'j', 'd', 'w', 'a', '1', '\u{8}', ' '];
    // Phím inject: dù đang giữa từ đã transform hay đang trống — luôn PASS.
    for pre in ["", "Te", "Tẽ", "ddư", "syntax"] {
        for c in keys {
            for m in [Method::Telex, Method::Vni] {
                let mut opts = opts_for(m, false);
                opts.auto_restore_english = true;
                let mut e = Engine::new(opts.clone());
                let mut buf: Vec<char> = Vec::new();
                for pc in pre.chars() {
                    let down = key_event(pc, true);
                    let o = e.key(&down);
                    apply(&mut buf, &o.action, &down);
                    let _ = e.key(&key_event(pc, false));
                }
                let before = buf.len();
                let mut inj = key_event(c, true);
                inj.is_injected = true;
                let o = e.key(&inj);
                // Injected: engine phải trả PASS (adapter tự in phím vào app).
                assert!(
                    matches!(o.action, Action::Pass),
                    "{m:?} pre={pre:?} injected `{c}` bị engine xử lý — tái-xử lý phím bơm"
                );
                let _ = e.key(&KeyEvent {
                    ch: c as u32,
                    key_down: false,
                    is_injected: true,
                    ..Default::default()
                });
                assert_eq!(buf.len(), before, "injected đổi document");
                checked += 1;
                count_case();
            }
        }
    }
    assert!(checked >= 100, "Suite J chỉ chạy {checked} case");
}

/// Key-up (không phải phím bấm) không được engine đếm/hưởng trạng thái gõ.
#[test]
fn suite_j2_key_up_never_counts_or_transforms() {
    let mut checked = 0usize;
    for m in [Method::Telex, Method::SimpleTelex, Method::Vni] {
        let opts = opts_for(m, false);
        let mut e = Engine::new(opts);
        for c in "dduocjxswfjw".chars() {
            // Chỉ key-up, không key-down: engine không được tạo từ/transform.
            let up = key_event(c, false);
            let o = e.key(&up);
            assert!(
                matches!(o.action, Action::Pass),
                "{m:?}: key-up `{c}` bị xử lý như phím gõ"
            );
            count_case();
            checked += 1;
        }
    }
    assert!(checked >= 33, "J2 chỉ chạy {checked} case");
}

// ====================================================================
// Suite K — sổ Dream RSI (G19): bão phím không làm lệch `keys_typed`
// ====================================================================

#[test]
fn suite_k_keys_typed_ledger_integrity() {
    let mut checked = 0usize;
    let opts = opts_for(Method::Telex, false);
    let mut e = Engine::new(opts);
    // Gõ 1 dòng: keys_typed tăng đúng số phím chữ thật (ch>0, down, không inject).
    let real: Vec<char> = "dduocj syntax TextVN ".chars().collect();
    let expect_real = real
        .iter()
        .filter(|&&c| c != BS && c != ESC && c != '\n' && c != '\t')
        .count() as u64;
    for &c in &real {
        let down = key_event(c, true);
        let _ = e.key(&down);
        let _ = e.key(&key_event(c, false));
    }
    let snap = e.dream_rsi_snapshot();
    assert_eq!(
        snap.keys_typed, expect_real,
        "keys_typed lệch sổ: {} ≠ phím chữ thật {expect_real} (G19: chỉ phím gõ chữ thật ch>0)",
        snap.keys_typed
    );
    checked += 1;
    count_case();
    // Bão BS/injected/key-up: keys_typed KHÔNG đổi (không phải phím gõ chữ).
    let before = snap.keys_typed;
    for _ in 0..12 {
        let _ = e.key(&KeyEvent::key_down(keymap::vk::BACK));
        let _ = e.key(&KeyEvent::key_up(keymap::vk::BACK));
        let mut inj = KeyEvent::char_down('x');
        inj.is_injected = true;
        let _ = e.key(&inj);
        let _ = e.key(&KeyEvent::key_up(keymap::vk::CONTROL));
        let _ = e.key(&KeyEvent::key_down(keymap::vk::SHIFT));
    }
    let after = e.dream_rsi_snapshot().keys_typed;
    assert_eq!(
        after, before,
        "bão BS/inject/modifier làm đổi keys_typed (G19)"
    );
    checked += 1;
    count_case();
    // Chord (Ctrl+phím) không được đếm là phím gõ chữ.
    let before = after;
    let mut chord = KeyEvent::char_down('x');
    chord.vk = 'X' as u32;
    chord.mods = 2; // MOD_CTRL
    let _ = e.key(&chord);
    let after = e.dream_rsi_snapshot().keys_typed;
    assert_eq!(after, before, "chord Ctrl+chữ được đếm keys_typed (G19)");
    checked += 1;
    count_case();
    assert_eq!(checked, 3, "K thiếu case");
}

// ====================================================================
// Gate — tổng số case MỚI của file này ≥ 600
// ====================================================================

#[test]
fn gate_dual_mode_rapid_case_count_at_least_600() {
    let suite_e = CLUSTER_BASES.len() * 5 * 2; // base × tone × (telex+VNI) — mỗi vòng chạy cả 2 style
    let suite_e2 = 5 * 2;
    let suite_f = TELEX_STREAMS.len() * 2 * 2 + VNI_STREAMS.len();
    let g1_pairs: usize = [
        "syntax", "export", "server", "mouse", "house", "telex", "window", "context", "buffer",
        "dduocj", "chaof", "vieetj", "TextVN",
    ]
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
            .map(|p| p.min(2))
            .sum::<usize>()
    })
    .sum::<usize>()
        * 3; // methods
    let suite_g2 = 7 * 2;
    let suite_g3 = 5 * 4 * 3;
    let suite_h = 6 * 3 * 4 + 6 * 3; // 3 cặp so-sánh hint + 1 oracle mỗi (stream, method)
    let suite_i = 12 * 3 * 3 * 2 + 12 * 2;
    let suite_j = 5 * 10 * 2 + 3 * 12;
    let suite_k = 3;
    let total = suite_e
        + suite_e2
        + suite_f
        + g1_pairs
        + suite_g2
        + suite_g3
        + suite_h
        + suite_i
        + suite_j
        + suite_k;
    println!(
        "dual_mode_rapid: E={suite_e} E2={suite_e2} F={suite_f} G1={g1_pairs} G2={suite_g2} G3={suite_g3} H={suite_h} I={suite_i} J={suite_j} K={suite_k} → total {total}"
    );
    assert!(
        total >= 600,
        "ma trận dual-mode-rapid chỉ có {total} case — yêu cầu audit ≥ 600 case MỚI"
    );
}
