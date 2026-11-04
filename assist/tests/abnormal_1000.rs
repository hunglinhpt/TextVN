// SPDX-License-Identifier: GPL-3.0-or-later
//! abnormal_1000 — bộ 1000 kịch bản abnormal sinh THỦ TỤC (deterministic,
//! không RNG) cho engine trợ lý + đường dự đoán. Mỗi case qua 4 rào chặn:
//! (1) no-panic — (2) bounded time — (3) fail-open + fix idempotent (áp thêm
//! 2 lần không đổi thêm) — (4) kết quả UTF-8 hợp lệ + span trong biên ký tự.
//!
//! G8: KHÔNG bao giờ in toàn bộ input; báo cáo chỉ chứa case id + mô tả +
//! input rút gọn ≤ 60 ký tự đã escape; panic hook bị nuốt im lặng (message
//! panic của engine có thể nhúng nội dung text người dùng).
//!
//! Thiết kế đầy đủ: tài liệu thiết kế QA vòng 54 (40 seed × 23 mutation =
//! 920 matrix + 80 fixed specials = 1000; độ dài phủ 0/1/2/3/7/255/256/257/
//! 4095/4096/65535/65536).

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};
use textvn_assist::{apply_auto_fixes, check, check_sentence, Issue};
use textvn_core::post::predict::{predict_next_phrase_candidates, predict_prefix_candidates};

// ============================ Mô hình case ============================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// check() (fragment) + check_sentence() — cả hai đường assist.
    Both,
    /// predict_prefix_candidates(input, is_vi, limit).
    PredictPrefix { is_vi: bool },
    /// predict_next_phrase_candidates(input, limit).
    PredictPhrase,
}

struct Case {
    id: String,   // "A0001".."A1000" — STABLE theo vị trí
    desc: String, // không chứa nội dung input
    input: String,
    mode: Mode,
}

// ============================ Bảng 40 seed ============================

const SEEDS: &[&str] = &[
    "Tôi đi học ở trường Đại học Bách Khoa.",
    "anh đã xác nhận lại đã xác nhận lại rồi",
    "vui lòng vui lòng kiểm tra giúp",
    "Em đã gửi report cho anh rồi, check giúp em nhé.",
    "Hạn chót là 32/13/2025 nhé.",
    "Chuyển khoản 1.250.00đ giúp mình.",
    "xin chào. bạn khỏe không.",
    "Tôi không có gì để nói không.",
    "Tôi đã gửi báo cáo cho phòng kế toán vào sáng thứ hai,",
    "It do the job perfectly.",
    "công công công công",
    "Chuyến bay bay thẳng từ Hà Nội vào Đà Nẵng.",
    "Rồi rồi, tôi nhớ rồi.",
    "Họ sống ba chìm bảy nổi.",
    "phát\u{200B}triển nhanh nhé",
    "e\u{200D}m gửi \u{200D}báo cáo",
    "tôi\u{200C}đi\u{200C}học luôn",
    "مرحبا بالعالم và tôi đi học",
    "你好世界 tôi đi học",
    "שלום עולם và tôi",
    "😀🇻🇳👍 tôi đi học 🍜",
    "👩‍👩‍👧‍👦 gia đình 👩‍💻 làm việc",
    "\u{FFFD}tôi đi học\u{FFFD} về",
    "tôi đi học\u{0} về\u{7}\u{8} xong",
    "1.1.1.1.1.1.1.1 xong rồi",
    "xem https://example.com/x?a=1&y=<>() rồi mail a@b.c nhé",
    "Tệp C:\\Users\\vanan\\bao-cao-quy-4.xlsx đã gửi",
    "if (x > 3) { return \"ok\"; } // TODO kiểm tra",
    "phát",
    "techn",
    "ngư",
    "kính gửi",
    "thank you",
    "cộng hòa",
    "anh đã đã đã gửi báo cáo cho phòng kế toán",
    "la la la la hát thật vui",
    "31/02/2026 31/02/2026 31/02/2026 quá nhiều",
    "Không có ai không biết quy định này không.",
    "the the the file and then i went home.",
    "tôi đã đã",
];

// ==================== Bảng phân rã (NFD cục bộ) =======================

const DECOMP: &[(char, &str)] = &[
    ('à', "a\u{0300}"),
    ('á', "a\u{0301}"),
    ('ả', "a\u{0309}"),
    ('ã', "a\u{0303}"),
    ('ạ', "a\u{0323}"),
    ('ă', "a\u{0306}"),
    ('â', "a\u{0302}"),
    ('ằ', "a\u{0306}\u{0300}"),
    ('ắ', "a\u{0306}\u{0301}"),
    ('ẳ', "a\u{0306}\u{0309}"),
    ('ẵ', "a\u{0306}\u{0303}"),
    ('ặ', "a\u{0306}\u{0323}"),
    ('ầ', "a\u{0302}\u{0300}"),
    ('ấ', "a\u{0302}\u{0301}"),
    ('ẩ', "a\u{0302}\u{0309}"),
    ('ẫ', "a\u{0302}\u{0303}"),
    ('ậ', "a\u{0302}\u{0323}"),
    ('è', "e\u{0300}"),
    ('é', "e\u{0301}"),
    ('ẻ', "e\u{0309}"),
    ('ẽ', "e\u{0303}"),
    ('ẹ', "e\u{0323}"),
    ('ê', "e\u{0302}"),
    ('ề', "e\u{0302}\u{0300}"),
    ('ế', "e\u{0302}\u{0301}"),
    ('ể', "e\u{0302}\u{0309}"),
    ('ễ', "e\u{0302}\u{0303}"),
    ('ệ', "e\u{0302}\u{0323}"),
    ('ì', "i\u{0300}"),
    ('í', "i\u{0301}"),
    ('ỉ', "i\u{0309}"),
    ('ĩ', "i\u{0303}"),
    ('ị', "i\u{0323}"),
    ('ò', "o\u{0300}"),
    ('ó', "o\u{0301}"),
    ('ỏ', "o\u{0309}"),
    ('õ', "o\u{0303}"),
    ('ọ', "o\u{0323}"),
    ('ô', "o\u{0302}"),
    ('ồ', "o\u{0302}\u{0300}"),
    ('ố', "o\u{0302}\u{0301}"),
    ('ổ', "o\u{0302}\u{0309}"),
    ('ỗ', "o\u{0302}\u{0303}"),
    ('ộ', "o\u{0302}\u{0323}"),
    ('ơ', "o\u{031B}"),
    ('ờ', "o\u{031B}\u{0300}"),
    ('ớ', "o\u{031B}\u{0301}"),
    ('ở', "o\u{031B}\u{0309}"),
    ('ỡ', "o\u{031B}\u{0303}"),
    ('ợ', "o\u{031B}\u{0323}"),
    ('ù', "u\u{0300}"),
    ('ú', "u\u{0301}"),
    ('ủ', "u\u{0309}"),
    ('ũ', "u\u{0303}"),
    ('ụ', "u\u{0323}"),
    ('ư', "u\u{031B}"),
    ('ừ', "u\u{031B}\u{0300}"),
    ('ứ', "u\u{031B}\u{0301}"),
    ('ử', "u\u{031B}\u{0309}"),
    ('ữ', "u\u{031B}\u{0303}"),
    ('ự', "u\u{031B}\u{0323}"),
    ('ỳ', "y\u{0300}"),
    ('ý', "y\u{0301}"),
    ('ỷ', "y\u{0309}"),
    ('ỹ', "y\u{0303}"),
    ('ỵ', "y\u{0323}"),
    ('Đ', "D\u{0110}"),
];

fn decomp_char(c: char) -> Option<&'static str> {
    DECOMP.iter().find(|(p, _)| *p == c).map(|(_, d)| *d)
}

fn is_combining(c: char) -> bool {
    (0x0300..=0x036F).contains(&(c as u32)) // gồm cả horn U+031B
}

// ============================ Helpers chuỗi ============================

fn take_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// take hoặc pad "x" đến đúng n ký tự (biên 255/256/257/4095/4096/65535/65536).
fn fit_to(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    while out.chars().count() < n {
        out.push('x');
    }
    out
}

// ==================== 23 mutation (thứ tự cố định) ====================

fn m01_raw(s: &str) -> String {
    s.to_string()
}

fn m02_decomp(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match decomp_char(c) {
            Some(d) => out.push_str(d),
            None => out.push(c),
        }
    }
    out
}

/// Decompose rồi NHÂN ĐÔI dấu thanh cuối của âm đầu mỗi từ ("ô" → ô + 2 circumflex).
fn m03_combining_du(s: &str) -> String {
    let decomp = m02_decomp(s);
    let mut out = String::with_capacity(decomp.len() + 16);
    let mut word: Vec<char> = Vec::new();
    fn flush(word: &mut Vec<char>, out: &mut String) {
        if let Some(pos) = word.iter().rposition(|c| is_combining(*c)) {
            let mark = word[pos];
            for c in word.drain(..=pos) {
                out.push(c);
            }
            out.push(mark); // dấu THỪA — nhân đôi
            out.extend(word.drain(..));
        } else {
            out.extend(word.drain(..));
        }
    }
    for c in decomp.chars() {
        if c.is_alphanumeric() || is_combining(c) {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Trộn NFC/NFD: ký tự ở vị trí char chẵn bị phân rã, lẻ giữ nguyên.
fn m04_mixed_nf(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for (i, c) in s.chars().enumerate() {
        match (i % 2 == 0, decomp_char(c)) {
            (true, Some(d)) => out.push_str(d),
            _ => out.push(c),
        }
    }
    out
}

/// ZWSP sau ký tự đầu / ZWNJ giữa / ZWSP cuối mỗi từ; ZWJ giữa 2 từ liền kề.
fn m05_zero_width(s: &str) -> String {
    let mut out = String::new();
    let mut prev_alpha = false;
    for w in s.split_whitespace() {
        if prev_alpha {
            out.push('\u{200D}');
        }
        let chars: Vec<char> = w.chars().collect();
        let alpha = chars.iter().any(|c| c.is_alphabetic());
        if alpha && chars.len() >= 2 {
            let mid = chars.len() / 2;
            for (i, c) in chars.iter().enumerate() {
                if i == 1 {
                    out.push('\u{200B}');
                }
                if i == mid {
                    out.push('\u{200C}');
                }
                out.push(*c);
            }
            out.push('\u{200B}');
        } else {
            out.extend(chars.iter());
        }
        out.push(' ');
        prev_alpha = alpha;
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Bidi override: RLO … PDF + RLM sau mỗi dấu cách.
fn m06_bidi(s: &str) -> String {
    let mut out = String::from("\u{202E}");
    for c in s.chars() {
        out.push(c);
        if c == ' ' {
            out.push('\u{200F}');
        }
    }
    out.push('\u{202C}');
    out
}

/// Variation selector: FE00 sau ký tự đầu từ; FE0F sau mỗi char ≥ U+1F000.
fn m07_variation(s: &str) -> String {
    let mut out = String::new();
    let mut at_word_start = true;
    for c in s.chars() {
        if c.is_alphabetic() {
            out.push(c);
            if at_word_start {
                out.push('\u{FE00}');
                at_word_start = false;
            }
        } else {
            out.push(c);
            if (c as u32) >= 0x1F000 {
                out.push('\u{FE0F}');
            }
            at_word_start = true;
        }
    }
    out
}

/// Xen CJK mỗi 4, Arabic mỗi 9, Hebrew mỗi 13 ký tự.
fn m08_script(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for (i, c) in s.chars().enumerate() {
        out.push(c);
        if (i + 1) % 4 == 0 {
            out.push('漢');
        }
        if (i + 1) % 9 == 0 {
            out.push('ب');
        }
        if (i + 1) % 13 == 0 {
            out.push('א');
        }
    }
    out
}

fn m09_newline_weave(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return format!("{s}\n");
    }
    let mut out = words.join("\n");
    out.push_str("\r\n");
    out
}

fn m10_cr_only(s: &str) -> String {
    s.replace(' ', "\r")
}

fn m11_mixed_endings(s: &str) -> String {
    const E: [&str; 4] = ["\r\n", "\n", "\r", "\t"];
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return format!("{s}\r\n");
    }
    let mut out = String::new();
    for (i, w) in words.iter().enumerate() {
        out.push_str(w);
        if i + 1 < words.len() {
            out.push_str(E[i % 4]);
        }
    }
    out.push_str("\r\n");
    out
}

fn m12_space_flood(s: &str) -> String {
    format!(
        "{}{}{}",
        " ".repeat(16),
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(&" ".repeat(40)),
        " ".repeat(8)
    )
}

fn m13_bom_wrap(s: &str) -> String {
    format!("\u{FEFF}{s}\u{FEFF}")
}

/// Chỉ giữ punct/emoji/ký-hiệu; fallback composite khi kết quả trống thuần space.
fn m14_punct_emoji_only(s: &str) -> String {
    let kept: String = s.chars().filter(|c| !c.is_alphanumeric()).collect();
    if kept
        .chars()
        .any(|c| c.is_ascii_punctuation() || matches!(c, '…' | '“' | '”' | '·'))
    {
        // Cả nhánh kept & non-empty-kept trả nguyên vẹn: punct thuần là input
        // hợp lệ; emoji/zero-width/không-gian cũng vậy (chỉ khác không có ASCII
        // punct) — gộp khối cho clippy (semantics không đổi).
        kept
    } else {
        format!("{s} ...!!!??? 🇻🇳😀👏")
    }
}

/// Unit = 64 ký tự đầu (+1 space), lặp 200 → ~≤13k chars (lặp từ dày).
fn m15_repeat_x200(s: &str) -> String {
    let unit = take_chars(s, 64);
    let mut out = String::with_capacity((unit.len() + 1) * 200);
    for _ in 0..200 {
        out.push_str(&unit);
        out.push(' ');
    }
    out
}

/// 8 ký tự đầu lặp 500 liền (~4k chars).
fn m16_unit8_x500(s: &str) -> String {
    take_chars(s, 8).repeat(500)
}

fn m17_whitelist_repeat(s: &str) -> String {
    format!("{s} công công công công bay bay bay bay")
}

fn m18_dot_chain(s: &str) -> String {
    format!("{s} {}", "1.".repeat(100))
}

fn m19_money_date_scale(s: &str) -> String {
    format!(
        "{s} 9.999.999.999.999đ 1.25.000đ 1.250.00đ 32/13/2025 31/02/2026 00/00/0000 15/13/9999"
    )
}

/// Chen emoji sau mỗi từ + đuôi decor (lỗi thật sát decor).
fn m20_decor(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return format!("{s}🚀⏰📁🎉");
    }
    format!("{} ⏰📁🎉", words.join("🚀 "))
}

fn m21_one_char(s: &str) -> String {
    let mut out = take_chars(s, 1);
    if out.is_empty() {
        out.push('a');
    }
    out
}

fn m22_to_256(s: &str) -> String {
    fit_to(s, 256)
}
fn m23_to_4096(s: &str) -> String {
    fit_to(s, 4096)
}

type Mut = fn(&str) -> String;

const MUTATIONS: &[(&str, Mut)] = &[
    ("raw", m01_raw),
    ("decomp", m02_decomp),
    ("combining_du", m03_combining_du),
    ("mixed_nf", m04_mixed_nf),
    ("zero_width", m05_zero_width),
    ("bidi", m06_bidi),
    ("variation_sel", m07_variation),
    ("script_xen", m08_script),
    ("newline_weave", m09_newline_weave),
    ("cr_only", m10_cr_only),
    ("mixed_endings", m11_mixed_endings),
    ("space_flood", m12_space_flood),
    ("bom_wrap", m13_bom_wrap),
    ("punct_emoji_only", m14_punct_emoji_only),
    ("repeat_x200", m15_repeat_x200),
    ("unit8_x500", m16_unit8_x500),
    ("whitelist_repeat", m17_whitelist_repeat),
    ("dot_chain", m18_dot_chain),
    ("money_date_scale", m19_money_date_scale),
    ("decor_saturate", m20_decor),
    ("to_1_char", m21_one_char),
    ("to_256", m22_to_256),
    ("to_4096", m23_to_4096),
];

// ============================ build_cases ============================

fn build_cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::with_capacity(1000);
    // ---- Ma trận 40 × 23 = 920: A0001..A0920 (seed-major, cố định) ----
    for (si, seed) in SEEDS.iter().enumerate() {
        for (mi, (mname, m)) in MUTATIONS.iter().enumerate() {
            cases.push(Case {
                id: format!("A{:04}", si * MUTATIONS.len() + mi + 1),
                desc: format!("S{:02} x {mname}", si + 1),
                input: m(seed),
                mode: Mode::Both,
            });
        }
    }
    debug_assert_eq!(cases.len(), 920);
    // ---- 80 fixed specials: A0921..A1000 ----
    let mut n = 920usize;
    let mut sp = |cases: &mut Vec<Case>, desc: &str, input: String, mode: Mode| {
        n += 1;
        cases.push(Case {
            id: format!("A{n:04}"),
            desc: desc.to_string(),
            input,
            mode,
        });
    };
    // P1 — predict prefix (16)
    let unit_ctx = "tôi đã gửi báo cáo cho anh ";
    let ctx_base: String = unit_ctx.repeat(40);
    sp(
        &mut cases,
        "pred/prefix phat vi",
        "phát".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix techn en",
        "techn".into(),
        Mode::PredictPrefix { is_vi: false },
    );
    sp(
        &mut cases,
        "pred/prefix 255 p",
        "p".repeat(255),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix 256 t en",
        "t".repeat(256),
        Mode::PredictPrefix { is_vi: false },
    );
    sp(
        &mut cases,
        "pred/prefix 4096 x",
        "x".repeat(4096),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix HOA PHAT",
        "PHÁT".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix hoa lan",
        "pHát".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix emoji",
        "😀😀phát".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix decomp",
        "cafe\u{0301}".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix rong",
        String::new(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix 1 ky tu",
        "a".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix du tu ghep",
        "phát triển".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix am tiet go",
        "ngư".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix khong dau",
        "phat".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix so en",
        "1.25".into(),
        Mode::PredictPrefix { is_vi: false },
    );
    sp(
        &mut cases,
        "pred/prefix BOM",
        "\u{FEFF}phát".into(),
        Mode::PredictPrefix { is_vi: true },
    );
    // P2 — predict phrase (16)
    let phrase64 = format!(
        "{}anh chị vui lòng kiểm tra lại",
        "x".repeat(64 - "anh chị vui lòng kiểm tra lại".chars().count())
    );
    sp(
        &mut cases,
        "pred/phrase kinh gui",
        "kính gửi".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx64",
        phrase64,
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx255",
        take_chars(&ctx_base, 255),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx256 bien",
        take_chars(&ctx_base, 256),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx1000",
        take_chars(&ctx_base, 1000),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase newline cuoi",
        "kính gửi\n".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase crlf cuoi",
        "kính gửi\r\n".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase cr cuoi",
        "kính gửi\r".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase rong",
        String::new(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase chi space",
        "   ".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase cong hoa",
        "cộng hòa".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase hoa dau",
        "Cộng hoà".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase thank you",
        "thank you".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase emoji",
        "😀🎉👍".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase date",
        "31/02/2026".into(),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase 1 ky tu",
        "a".into(),
        Mode::PredictPhrase,
    );
    // L-ladder 33 case
    let seeds3 = [0usize, 1, 25, 38]; // S01 S02 S26 S39
    let seeds_fit = [0usize, 3, 36, 39]; // S01 S04 S37 S40
    let seeds_big = [1usize, 34, 39]; // S02 S35 S40
    let unit_big = |i: usize| take_chars(SEEDS[i], 64);
    sp(&mut cases, "len/0", String::new(), Mode::Both);
    sp(&mut cases, "len/1", "a".into(), Mode::Both);
    sp(&mut cases, "len/2", "ab".into(), Mode::Both);
    for i in seeds3 {
        sp(&mut cases, "len/3", take_chars(SEEDS[i], 3), Mode::Both);
    }
    for i in seeds3 {
        sp(&mut cases, "len/7", take_chars(SEEDS[i], 7), Mode::Both);
    }
    for i in seeds_fit {
        sp(&mut cases, "len/255", fit_to(SEEDS[i], 255), Mode::Both);
    }
    for i in seeds_fit {
        sp(&mut cases, "len/256", fit_to(SEEDS[i], 256), Mode::Both);
    }
    for i in seeds_fit {
        sp(&mut cases, "len/257", fit_to(SEEDS[i], 257), Mode::Both);
    }
    for i in seeds_big {
        sp(&mut cases, "len/4095", fit_to(SEEDS[i], 4095), Mode::Both);
    }
    for i in seeds_big {
        sp(&mut cases, "len/4096", fit_to(SEEDS[i], 4096), Mode::Both);
    }
    // 65535/65536 chỉ 2 seed (S35/S40) — giữ đúng 1000 case tổng.
    for i in [34usize, 39] {
        let mut s = String::new();
        while s.chars().count() < 65535 {
            s.push_str(&unit_big(i));
        }
        sp(&mut cases, "len/65535", take_chars(&s, 65535), Mode::Both);
        sp(
            &mut cases,
            "len/65536",
            take_chars(&format!("{s}{s}"), 65536),
            Mode::Both,
        );
    }
    // Misc biên 8 case
    sp(&mut cases, "misc/space", " ".into(), Mode::Both);
    sp(&mut cases, "misc/tab", "\t".into(), Mode::Both);
    sp(&mut cases, "misc/cr", "\r".into(), Mode::Both);
    sp(&mut cases, "misc/bom", "\u{FEFF}".into(), Mode::Both);
    sp(
        &mut cases,
        "misc/bom x2",
        "\u{FEFF}\u{FEFF}".into(),
        Mode::Both,
    );
    sp(&mut cases, "misc/emoji 1 cap", "😀".into(), Mode::Both);
    sp(
        &mut cases,
        "misc/replacement x3",
        "\u{FFFD}\u{FFFD}\u{FFFD}".into(),
        Mode::Both,
    );
    sp(&mut cases, "misc/nul", "\u{0}".into(), Mode::Both);
    // Predict-ladder 7 case
    sp(
        &mut cases,
        "pred/prefix 64 p",
        "p".repeat(64),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/prefix 254 cau vi",
        take_chars(&ctx_base, 254),
        Mode::PredictPrefix { is_vi: true },
    );
    sp(
        &mut cases,
        "pred/phrase ctx63",
        take_chars(&ctx_base, 63),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx64 bien tail",
        take_chars(&ctx_base, 64),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase ctx65",
        take_chars(&ctx_base, 65),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase crl giua",
        format!("kính gửi\r\n{}", take_chars(&ctx_base, 88)),
        Mode::PredictPhrase,
    );
    sp(
        &mut cases,
        "pred/phrase bom dau",
        format!("\u{FEFF}{}", take_chars(&ctx_base, 99)),
        Mode::PredictPhrase,
    );
    cases
}

// ==================== G8: rút gọn + escape an toàn ====================

/// Rút gọn ≤ max ký tự + escape ký tự điều khiển. KHÔNG bao giờ in toàn bộ.
fn clip(s: &str, max: usize) -> String {
    let mut out = String::new();
    for c in s.chars().take(max) {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7F => {
                out.push_str(&format!("\\u{{{:04X}}}", c as u32))
            }
            c => out.push(c),
        }
    }
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

// ==================== Ngân sách thời gian ====================

/// Spec: p99 ≤ 10ms/256-ký tự, dài hơn scale TUYẾN TÍNH; hệ số an toàn ×2
/// (tiền lệ v53: 65k đo < 2.5s → trần 5.12s đủ chặn O(n²) quay lại, dư cho CI).
/// Predict: bất biến < 50µs/call → trần phẳng 50ms (headroom 1000×).
fn budget_for(chars: usize, mode: Mode, mult: u64) -> Duration {
    let ms: u64 = match mode {
        Mode::Both => {
            let blocks = (chars.div_ceil(256).max(1)) as u64;
            20 * blocks
        }
        Mode::PredictPrefix { .. } | Mode::PredictPhrase => 50,
    };
    Duration::from_millis(ms * mult.max(1))
}

// ========================= 4 rào chặn / case =========================

fn assert_utf8(s: &str) -> Result<(), String> {
    // Rust String bất biến UTF-8 — kiểm tra này cố ý hiện diện để khóa ý định.
    std::str::from_utf8(s.as_bytes())
        .map(|_| ())
        .map_err(|e| format!("UTF-8 hỏng: {e}"))
}

/// Span đếm theo ký tự Unicode — phải nằm trong biên số ký tự của input.
fn validate_spans(input: &str, issues: &[Issue]) -> Result<(), String> {
    let n = input.chars().count();
    for (idx, is) in issues.iter().enumerate() {
        if is.span.start > is.span.end || is.span.end > n {
            return Err(format!(
                "issue #{idx} kind={} span {}..{} NGOAI BIEN {n} ky tu",
                is.kind.as_str(),
                is.span.start,
                is.span.end
            ));
        }
        if is.confidence > 100 {
            return Err(format!("issue #{idx} confidence {} > 100", is.confidence));
        }
    }
    Ok(())
}

/// 1 case qua đủ rào chặn 3 + 4 (rào 1 = catch_unwind ở runner, rào 2 = elapsed).
fn run_case(c: &Case) -> Result<(), String> {
    match c.mode {
        Mode::Both => {
            for path in ["fragment", "sentence"] {
                let issues = if path == "fragment" {
                    check(&c.input)
                } else {
                    check_sentence(&c.input)
                };
                validate_spans(&c.input, &issues)?;
                if issues.is_empty() {
                    continue; // fail-open: không lỗi → không áp fix
                }
                // Lần 1
                let fixed1 = apply_auto_fixes(&c.input, &issues);
                assert_utf8(&fixed1)?;
                // Lần 2
                let issues2 = check(&fixed1);
                validate_spans(&fixed1, &issues2)?;
                let fixed2 = apply_auto_fixes(&fixed1, &issues2);
                assert_utf8(&fixed2)?;
                // Lần 3 — phải ỔN ĐỊNH: áp thêm không đổi thêm (idempotent)
                let issues3 = check(&fixed2);
                validate_spans(&fixed2, &issues3)?;
                let fixed3 = apply_auto_fixes(&fixed2, &issues3);
                assert_utf8(&fixed3)?;
                if fixed3 != fixed2 {
                    return Err(format!(
                        "fix KHONG HOP TU (path={path}): {} -> {} -> {} ky tu",
                        c.input.chars().count(),
                        fixed1.chars().count(),
                        fixed2.chars().count(),
                    ));
                }
            }
            Ok(())
        }
        Mode::PredictPrefix { is_vi } => {
            for limit in [1usize, 5, 10] {
                let cand = predict_prefix_candidates(&c.input, is_vi, limit);
                if cand.len() > limit {
                    return Err(format!(
                        "prefix: tra {}/{} ung vien vi pham limit",
                        cand.len(),
                        limit
                    ));
                }
                for s in &cand {
                    assert_utf8(s)?;
                    if s.is_empty() {
                        return Err("prefix: ung vien rong".into());
                    }
                    if s.chars().count() > 64 {
                        // R1-2: chèn ≤ MAX_TEXT = 64 — ứng viên phải nằm trong trần
                        return Err(format!(
                            "prefix: ung vien {} ky tu > 64 (R1-2)",
                            s.chars().count()
                        ));
                    }
                }
            }
            Ok(())
        }
        Mode::PredictPhrase => {
            for limit in [1usize, 3, 5] {
                let cand = predict_next_phrase_candidates(&c.input, limit);
                if cand.len() > limit {
                    return Err(format!(
                        "phrase: tra {}/{} ung vien vi pham limit",
                        cand.len(),
                        limit
                    ));
                }
                for s in &cand {
                    assert_utf8(s)?;
                    if s.chars().count() > 64 {
                        return Err(format!(
                            "phrase: ung vien {} ky tu > 64 (R1-2)",
                            s.chars().count()
                        ));
                    }
                }
            }
            Ok(())
        }
    }
}

// ============================== Runner ==============================

#[test]
fn a1000_tat_ca_case_qua_4_rao_chan() {
    let cases = build_cases();
    assert_eq!(
        cases.len(),
        1000,
        "ma tran phai du 1000 case, co {}",
        cases.len()
    );

    // Debug build chậm ~10-30× release (tiền lệ: A0798 release 3.5ms / debug
    // 353ms) — hệ số mặc định theo build type, env vẫn override được.
    let debug_default: u64 = if cfg!(debug_assertions) { 16 } else { 1 };
    let mult: u64 = std::env::var("TEXTVN_ABNORMAL_BUDGET_MULT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(debug_default);
    let only = std::env::var("TEXTVN_ABNORMAL_ONLY").ok();

    // G8: nuốt panic output toàn cục trong suốt runner — message panic của
    // engine có thể nhúng nội dung text người dùng. Khôi phục TRƯỚC khi tự
    // panic để báo cáo của chính suite luôn in được.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let t_suite = Instant::now();
    let mut failure: Option<String> = None;
    let mut ran = 0usize;

    for c in cases
        .iter()
        .filter(|c| only.as_ref().is_none_or(|o| c.id.contains(o.as_str())))
    {
        let n = c.input.chars().count();
        let budget = budget_for(n, c.mode, mult);
        let t0 = Instant::now();
        let res = catch_unwind(AssertUnwindSafe(|| run_case(c)));
        let elapsed = t0.elapsed();
        let short = clip(&c.input, 60);
        let msg = match res {
            Err(_) => Some(format!(
                "[{}] {} PANIC engine | len={n} ky tu | input={short}",
                c.id, c.desc
            )),
            Ok(Err(e)) => Some(format!(
                "[{}] {} FAIL: {e} | len={n} ky tu | input={short}",
                c.id, c.desc
            )),
            Ok(Ok(())) if elapsed > budget => Some(format!(
                "[{}] {} VUOT NGAN SACH {elapsed:?} > {budget:?} | len={n} | input={short}",
                c.id, c.desc
            )),
            Ok(Ok(())) => None,
        };
        if let Some(m) = msg {
            failure = Some(m);
            break; // fail-fast — repro bằng TEXTVN_ABNORMAL_ONLY=<id>
        }
        ran += 1;
    }

    let total = t_suite.elapsed();
    std::panic::set_hook(prev_hook);

    if let Some(m) = failure {
        panic!("{m}");
    }
    if only.is_none() {
        assert_eq!(ran, 1000, "phai chay du 1000 case");
        assert!(
            total < Duration::from_secs(60),
            "toan suite 1000 case ton {total:?} > 60s"
        );
    }
}

/// Meta-test: builder pure, deterministic, id stable A0001..A1000.
#[test]
fn a1000_builder_deterministic_va_id_stable() {
    let a = build_cases();
    let b = build_cases();
    assert_eq!(a.len(), 1000);
    assert_eq!(a[0].id, "A0001");
    assert_eq!(a[920].id, "A0921");
    assert_eq!(a[999].id, "A1000");
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            x.id,
            format!("A{:04}", i + 1),
            "id phai STABLE tai vi tri {i}"
        );
        assert_eq!(x.id, y.id);
        assert_eq!(x.desc, y.desc);
        assert_eq!(x.input, y.input, "input phai deterministic tai {}", x.id);
        assert_eq!(x.mode, y.mode);
    }
}
