// SPDX-License-Identifier: GPL-3.0-or-later
//! Weighted confidence formula classifier with strict anti-false-positive filters
//! Score 0.0-1.0; trigger conversion ONLY if score >= 0.75

use super::MAX_PAYLOAD;

/// Classify a clipboard string as a formula. Returns confidence 0.0-1.0
pub fn classify(input: &str) -> f32 {
    // Hard rejects — immediate 0.0
    if is_hard_reject(input) {
        return 0.0;
    }

    // Size cap
    if input.len() > MAX_PAYLOAD {
        return 0.0;
    }

    let mut score: f32 = 0.0;
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    if len == 0 {
        return 0.0;
    }

    // LaTeX delimiters/commands (strong signal)
    let latex_commands = count_occurrences(
        input,
        &[
            "\\frac",
            "\\sqrt",
            "\\sum",
            "\\int",
            "\\alpha",
            "\\beta",
            "\\gamma",
            "\\pi",
            "\\sigma",
            "\\theta",
            "\\omega",
            "\\partial",
            "\\nabla",
            "\\infty",
            "\\pm",
            "\\times",
            "\\leq",
            "\\geq",
            "\\neq",
            "\\approx",
            "\\begin{",
            "\\end{",
            "\\left",
            "\\right",
            "\\text{",
        ],
    );
    score += (latex_commands as f32 * 0.15).min(0.45);
    // E57: chuỗi LaTeX nguyên khối (trim bắt đầu bằng `\` + ít nhất 1 command
    // đã biết) gần như chắc chắn là công thức — floor 0.85 (trước đây `rac{a}{b}`
    // chỉ đạt 0.2 < ngưỡng 0.75 → Smart Paste không bao giờ nhận).
    if latex_commands >= 1 && input.trim_start().starts_with('\\') {
        score = score.max(0.85);
    }

    // Math delimiters
    if input.contains('$') {
        score += 0.1;
    }
    if input.contains("\\[") || input.contains("\\]") {
        score += 0.15;
    }

    // Unicode math symbols — 0.06/ký tự, cap 0.35 (E57: chuỗi như `∑ᵢ₌₁ⁿ i²`
    // trước đây chỉ đếm 2/6 ký tự vì dải sub/superscript hẹp).
    let unicode_math = chars.iter().filter(|c| is_unicode_math(**c)).count();
    score += (unicode_math as f32 * 0.06).min(0.35);

    // Superscript/subscript notation
    if input.contains('^') || input.contains('_') {
        score += 0.1;
    }

    // MathML: khối <math xmlns=…>…</math> khép kín là công thức chắc chắn (E57:
    // trước đây chỉ +0.5 → đúng định dạng MathML vẫn bị chặn dưới ngưỡng 0.75).
    if input.contains("<math") && input.contains("Math/MathML") {
        let boost = if input.contains("</math>") { 0.85 } else { 0.5 };
        score = score.max(boost);
    }

    // Balanced braces (math structure indicator)
    let open = input.matches('{').count();
    let close = input.matches('}').count();
    if open == close && open > 0 {
        score += 0.05;
    }

    // Penalize prose (high stopword density)
    let stopwords = count_occurrences(
        input,
        &[
            " the ", " and ", " of ", " to ", " in ", " is ", " for ", " với ", " của ", " và ",
            " là ", " trong ",
        ],
    );
    if stopwords > 0 && latex_commands == 0 {
        score -= (stopwords as f32 * 0.1).min(0.3);
    }

    score.clamp(0.0, 1.0)
}

/// Hard reject rules — return true if input must NEVER be treated as formula
pub fn is_hard_reject(input: &str) -> bool {
    let trimmed = input.trim();

    // Empty
    if trimmed.is_empty() {
        return true;
    }

    // Windows paths: C:\, D:\, \Users\
    if trimmed.len() >= 3 {
        let bytes = trimmed.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'\\'
        {
            return true;
        }
    }
    if trimmed.starts_with("\\Users\\") || trimmed.starts_with("\\\\") {
        return true;
    }

    // URLs
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
    {
        return true;
    }

    // Currency: $ followed immediately by digits
    if is_currency_span(trimmed) {
        return true;
    }

    // JSON
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        // Check if it looks like valid JSON (has quotes and colons)
        if trimmed.contains('"') && trimmed.contains(':') {
            return true;
        }
    }

    // Code blocks
    for keyword in ["fn ", "def ", "class ", "import ", "SELECT ", "FROM "] {
        if trimmed.contains(keyword) && trimmed.lines().count() > 2 {
            return true;
        }
    }

    // Non-MathML XML/HTML
    trimmed.starts_with('<') && !trimmed.contains("<math")
}

/// Check if string is a currency span like $50 - $100
fn is_currency_span(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '$' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            return true;
        }
    }
    false
}

fn count_occurrences(haystack: &str, needles: &[&str]) -> usize {
    needles.iter().map(|n| haystack.matches(n).count()).sum()
}

fn is_unicode_math(c: char) -> bool {
    matches!(c,
        '∑' | '∫' | '∏' | '√' | '∞' | 'π' | 'θ' | 'λ' | 'µ' | 'σ' | 'ω' | 'Ω' |
        'α' | 'β' | 'γ' | 'δ' | 'ε' | 'φ' | 'ρ' | 'Δ' | '∇' | '∂' | '∀' | '∃' |
        '∈' | '∉' | '⊂' | '∪' | '∩' | '∅' | '±' | '×' | '÷' | '·' | '≠' | '≈' |
        '≤' | '≥' | '∝' | '∠' | '⊥' | '∥' | '⇒' | '⇔' | '∧' | '∨' | '¬' | '⊕' | '⊗' |
        // E57: mở đủ dải Unicode sub/superscript — charset cũ bỏ sót `ᵢ` `₌`
        // `ⁿ` `²` `³` (U+00B2/B3/B9, U+2070–U+209F) nên công thức Unicode
        // mật độ cao vẫn bị chấm điểm thấp.
        '₀'..='₟' | '\u{2070}'..='\u{209F}' | '\u{00B2}' | '\u{00B3}' | '\u{00B9}' | '\u{1D62}'..='\u{1D6A}' |
        '⏥' | '⌖' | 'Γ' | 'Θ' | 'Λ' | 'Ξ' | 'Φ' | 'Ψ' | 'Σ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_windows_path() {
        assert_eq!(classify("C:\\Users\\alpha"), 0.0);
        assert_eq!(classify("\\Users\\test"), 0.0);
    }

    #[test]
    fn rejects_urls() {
        assert_eq!(classify("https://example.com"), 0.0);
        assert_eq!(classify("http://math.com"), 0.0);
        assert_eq!(classify("mailto:test@test.com"), 0.0);
    }

    #[test]
    fn rejects_currency() {
        assert_eq!(classify("$50 - $100"), 0.0);
        assert_eq!(classify("$1,200.50"), 0.0);
        assert_eq!(classify("$10"), 0.0);
    }

    #[test]
    fn rejects_json() {
        assert_eq!(classify(r#"{"key": "value"}"#), 0.0);
        assert_eq!(classify(r#"[1, 2, 3]"#), 0.0);
    }

    #[test]
    fn rejects_code() {
        assert_eq!(classify("fn main() {\n  let x = 1;\n}"), 0.0);
        assert_eq!(classify("SELECT * FROM users WHERE id = 1"), 0.0);
    }

    #[test]
    fn rejects_html() {
        assert_eq!(classify("<html><body>hello</body></html>"), 0.0);
    }

    #[test]
    fn accepts_latex_frac() {
        let score = classify("\\frac{a}{b}");
        assert!(score >= 0.75, "score = {}", score);
    }

    #[test]
    fn accepts_mathml() {
        let score = classify(
            "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mrow><mi>x</mi></mrow></math>",
        );
        assert!(score >= 0.75, "score = {}", score);
    }

    #[test]
    fn accepts_unicode_math() {
        let score = classify("∑ᵢ₌₁ⁿ i² = n(n+1)(2n+1)/6");
        assert!(score > 0.3, "score = {}", score);
    }

    #[test]
    fn rejects_prose() {
        let score =
            classify("Đây là một câu văn bản tiếng Việt bình thường không có công thức toán học");
        assert!(score < 0.5, "score = {}", score);
    }

    #[test]
    fn rejects_oversized() {
        let huge = "x".repeat(MAX_PAYLOAD + 1);
        assert_eq!(classify(&huge), 0.0);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(classify(""), 0.0);
        assert_eq!(classify("   "), 0.0);
    }
}
