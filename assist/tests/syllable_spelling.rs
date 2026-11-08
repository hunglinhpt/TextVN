// SPDX-License-Identifier: GPL-3.0-or-later
//! P4-13 — cổng chất lượng của từ điển dấu thanh dựng sẵn (`SyllableTones`) trên corpus thật.
//!
//! - FPR: câu sạch `corpus/assist/vi_real_clean.ass` không được bị báo `Spelling` quá 0,5 %.
//! - Recall (cận trên): đổi dấu thanh một từ trong câu sạch → luật phải bắt ≥ 95 %.
//! - Gợi ý luôn hợp lệ, khác từ gốc, đúng kiểu hoa; đầu vào bất thường không panic.
//! - Hiệu năng: câu 256 ký tự kiểm trong ngân sách (đo gộp, không flaky).

use textvn_assist::dict::Dictionary;
use textvn_assist::syllables::SyllableTones;
use textvn_assist::{check_sentence_with, IssueKind};

const CLEAN: &str = include_str!("../../corpus/assist/vi_real_clean.ass");
const SYLLABLES: &str = include_str!("../../data/vn_syllables.txt");
const EXTRA: &str = include_str!("../../data/vn_syllables_extra.txt");

const ROWS: [[char; 6]; 12] = [
    ['a', 'à', 'á', 'ả', 'ã', 'ạ'],
    ['ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ'],
    ['â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ'],
    ['e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ'],
    ['ê', 'ề', 'ế', 'ể', 'ễ', 'ệ'],
    ['i', 'ì', 'í', 'ỉ', 'ĩ', 'ị'],
    ['o', 'ò', 'ó', 'ỏ', 'õ', 'ọ'],
    ['ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ'],
    ['ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ'],
    ['u', 'ù', 'ú', 'ủ', 'ũ', 'ụ'],
    ['ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự'],
    ['y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ'],
];

/// (khoá không dấu thanh, thanh) của một chữ thường — cài đặt ĐỘC LẬP với `SyllableTones` để phép đo
/// recall không tự chứng minh (không dùng `contains` của chính từ điển để chọn ca lỗi).
fn key_and_tone(word: &str) -> (String, usize, usize) {
    let (mut key, mut tone, mut marks) = (String::new(), 0usize, 0usize);
    for c in word.chars() {
        match ROWS
            .iter()
            .find_map(|r| r.iter().position(|x| *x == c).map(|t| (r[0], t)))
        {
            Some((base, t)) => {
                key.push(base);
                if t != 0 {
                    tone = t;
                    marks += 1;
                }
            }
            None => key.push(c),
        }
    }
    (key, tone, marks)
}

fn clean_sentences() -> Vec<String> {
    CLEAN
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| {
            l.split('\t')
                .next()
                .unwrap_or("")
                .trim_start_matches('~')
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn spelling_spans(text: &str) -> Vec<(usize, usize)> {
    check_sentence_with(text, SyllableTones::builtin())
        .into_iter()
        .filter(|i| i.kind == IssueKind::Spelling)
        .map(|i| (i.span.start, i.span.end))
        .collect()
}

#[test]
fn clean_corpus_fpr_at_most_half_percent() {
    let sentences = clean_sentences();
    assert!(
        sentences.len() > 900,
        "corpus phải đủ lớn: {}",
        sentences.len()
    );
    let flagged: Vec<&String> = sentences
        .iter()
        .filter(|s| !spelling_spans(s).is_empty())
        .collect();
    let limit = sentences.len() / 200; // 0,5 %
    assert!(
        flagged.len() <= limit,
        "FPR {}/{} vượt 0,5 %: {:?}",
        flagged.len(),
        sentences.len(),
        flagged.iter().take(5).collect::<Vec<_>>()
    );
}

/// Đổi dấu thanh của MỘT từ trong câu sạch sao cho thành âm tiết không hợp lệ; luật phải báo đúng span.
#[test]
fn injected_tone_errors_are_caught() {
    let mut valid: std::collections::HashSet<(String, usize)> = std::collections::HashSet::new();
    for line in SYLLABLES.lines().chain(EXTRA.lines()) {
        let w = line.trim().to_lowercase();
        if w.is_empty() || w.starts_with('#') || w.contains(char::is_whitespace) {
            continue;
        }
        let (k, t, m) = key_and_tone(&w);
        if m <= 1 {
            valid.insert((k, t));
        }
    }
    let family_known = |k: &str| (0..6).any(|t| valid.contains(&(k.to_string(), t)));
    let (mut total, mut hit) = (0usize, 0usize);
    let mut seed = 20_261_108u64;
    let mut next = || {
        // LCG deterministic (không rand) để test ổn định.
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (seed >> 33) as usize
    };
    for sentence in (0..3).flat_map(|_| clean_sentences()) {
        let chars: Vec<char> = sentence.chars().collect();
        // Mọi vị trí chữ có dấu thanh trong từ ≥ 3 ký tự — chọn ngẫu nhiên một.
        let mut spots: Vec<(usize, usize, usize, usize)> = Vec::new(); // (word_start, word_end, idx, row)
        let mut i = 0;
        while i < chars.len() {
            if !chars[i].is_alphabetic() {
                i += 1;
                continue;
            }
            let s = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            if i - s < 3 || chars[s..i].iter().any(|c| c.is_uppercase()) {
                continue;
            }
            for (k, c) in chars.iter().enumerate().take(i).skip(s) {
                if let Some(row) = ROWS.iter().position(|r| r.contains(c)) {
                    spots.push((s, i, k, row));
                }
            }
        }
        if spots.is_empty() {
            continue;
        }
        let (ws, we, k, row) = spots[next() % spots.len()];
        let tone = next() % 6;
        let mut mutated = chars.clone();
        mutated[k] = ROWS[row][tone];
        let word: String = mutated[ws..we].iter().collect();
        let original: String = chars[ws..we].iter().collect();
        // Chỉ tính ca thực sự là lỗi dấu thanh trên khoá đã biết, xác định ĐỘC LẬP bằng dữ liệu thô.
        let (key, tone_w, marks) = key_and_tone(&word);
        if word == original || marks != 1 || !family_known(&key) || valid.contains(&(key, tone_w)) {
            continue;
        }
        total += 1;
        let text: String = mutated.iter().collect();
        if spelling_spans(&text).contains(&(ws, we)) {
            hit += 1;
        }
    }
    assert!(total > 300, "đủ mẫu: {total}");
    let rate = hit as f64 / total as f64;
    assert!(
        rate >= 0.95,
        "recall cận trên {hit}/{total} = {rate:.3} < 0,95"
    );
}

#[test]
fn suggestions_are_valid_distinct_and_case_adapted() {
    let d = SyllableTones::builtin();
    for (input, expect) in [("chềt", "chết"), ("Chềt", "Chết")] {
        let text = format!("Hôm nay {input} rồi");
        let issues: Vec<_> = check_sentence_with(&text, d)
            .into_iter()
            .filter(|i| i.kind == IssueKind::Spelling)
            .collect();
        assert_eq!(issues.len(), 1, "{input}: {issues:?}");
        assert_eq!(
            issues[0].suggestions.first().map(String::as_str),
            Some(expect)
        );
        assert!(
            !issues[0].auto_fixable,
            "chính tả chỉ gợi ý — không tự sửa (A1/D12)"
        );
    }
    // Chữ HOA toàn bộ = viết tắt/tên riêng (cổng "chữ hoa bên trong") → im lặng, không đoán.
    assert!(spelling_spans("Hôm nay CHỀT rồi").is_empty());
    for w in ["bẵt", "tầt", "hõc", "chềt"] {
        for s in d.suggest(w, 5) {
            assert!(d.contains(&s) && s != w, "{w} → {s}");
        }
    }
}

#[test]
fn abnormal_inputs_never_panic_and_stay_silent_on_unknowns() {
    let d = SyllableTones::builtin();
    let long_word = "a".repeat(50_000);
    let many_marks = "ạ̀́̃".repeat(500);
    for text in [
        "",
        " ",
        "\u{0}\u{0}",
        "🙂🙂🙂 tôi đi",
        "\u{202e}tôi đi học",
        "Qua\u{0302}n Ha\u{0300}",
        long_word.as_str(),
        many_marks.as_str(),
        "xyzáb quẩnn",
        "ǆǈǋ ǅ",
    ] {
        let _ = check_sentence_with(text, d);
    }
    // Từ lạ không thuộc nhóm âm tiết nào → im lặng (không phải lỗi dấu thanh).
    assert!(spelling_spans("Tôi thích café và ôtô").is_empty());
}

#[test]
fn per_sentence_cost_is_small() {
    let text = "Hôm nay tôi đi học ở trường và gặp lại bạn cũ sau nhiều năm xa cách, \
                chúng tôi nói chuyện rất lâu về những kỷ niệm ngày xưa. Trời hôm nay đẹp lắm.";
    assert!(text.chars().count() <= 256);
    let d = SyllableTones::builtin();
    let _ = check_sentence_with(text, d); // khởi động bảng
    let t = std::time::Instant::now();
    for _ in 0..200 {
        let _ = check_sentence_with(text, d);
    }
    let per = t.elapsed() / 200;
    // Ngân sách sản phẩm 50 ms/câu (D36); debug build trên CI chậm → trần rộng 25 ms.
    assert!(per.as_millis() < 25, "{per:?}/câu");
}
