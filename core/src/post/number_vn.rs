// SPDX-License-Identifier: GPL-3.0-or-later
//! OP-003 (P5) — gõ tắt định dạng số tiền và đọc số thành chữ tiếng Việt
//! cho kế toán / hợp đồng / báo cáo tài chính.
//!
//! Cú pháp kích hoạt qua phím `macro_trigger` (`Tab` / `Space`):
//! - `=<số>d`   → `<số có chấm phân cách> đồng` (vd: `=15450000d` → `15.450.000 đồng`)
//! - `=<số>vnd` → `<số có chấm phân cách> VNĐ`  (vd: `=15450000vnd` → `15.450.000 VNĐ`)
//! - `=<số>bc`  → `<đọc bằng chữ tiếng Việt>`   (vd: `=15000000bc` → `Mười lăm triệu đồng chẵn`)
//!
//! Pure logic (S1): 0 I/O, 0 panic, deterministic. Số > 12 chữ số (hàng trăm
//! tỷ) → `None` (fail-open, giữ nguyên chuỗi gõ — R1-2 không cắt cụt).

const DIGITS: [&str; 10] = [
    "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
];

const UNITS: [&str; 4] = ["", "nghìn", "triệu", "tỷ"];

/// Định dạng chuỗi chữ số nguyên dương với dấu chấm phân cách hàng nghìn
/// (`15450000` → `15.450.000`).
pub fn format_thousands(digits: &str) -> String {
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return "0".to_string();
    }
    let bytes = trimmed.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(b as char);
    }
    out
}

/// Đọc một nhóm 3 chữ số thành chữ. `is_first_group`: nhóm cao nhất —
/// bỏ "không trăm" khi nhóm đầu không có hàng trăm (`15` → "mười lăm",
/// không phải "không trăm mười lăm").
fn read_triple(hundreds: u8, tens: u8, ones: u8, is_first_group: bool) -> Vec<&'static str> {
    let mut words = Vec::new();
    if hundreds == 0 && tens == 0 && ones == 0 {
        return words;
    }
    if !is_first_group || hundreds > 0 {
        words.push(DIGITS[hundreds as usize]);
        words.push("trăm");
    }
    match tens {
        0 => {
            if ones > 0 && (!is_first_group || hundreds > 0) {
                words.push("lẻ");
            }
        }
        1 => words.push("mười"),
        t => {
            words.push(DIGITS[t as usize]);
            words.push("mươi");
        }
    }
    match ones {
        0 => {}
        1 if tens >= 2 => words.push("mốt"),
        4 if tens >= 2 => words.push("tư"),
        5 if tens >= 1 => words.push("lăm"),
        o => words.push(DIGITS[o as usize]),
    }
    words
}

/// Đọc chuỗi chữ số (tối đa 12 chữ số = hàng trăm tỷ) thành chữ tiền tệ
/// tiếng Việt; nhóm toàn 0 ở giữa được bỏ đúng chuẩn ("mười lăm triệu",
/// không "mười lăm triệu không nghìn").
pub fn read_vnd_words(digits: &str) -> Option<String> {
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return Some("Không đồng".to_string());
    }
    if trimmed.len() > 12 || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let padded_len = trimmed.len().div_ceil(3) * 3;
    let pad = padded_len - trimmed.len();
    let mut full = Vec::with_capacity(padded_len);
    full.extend(std::iter::repeat_n(0u8, pad));
    full.extend(trimmed.bytes().map(|b| b - b'0'));

    let num_groups = padded_len / 3;
    let mut parts: Vec<&'static str> = Vec::new();
    for g in 0..num_groups {
        let h = full[g * 3];
        let t = full[g * 3 + 1];
        let o = full[g * 3 + 2];
        let triple = read_triple(h, t, o, g == 0);
        if !triple.is_empty() {
            parts.extend(triple);
            let unit_idx = num_groups - 1 - g;
            if unit_idx > 0 && unit_idx < UNITS.len() {
                parts.push(UNITS[unit_idx]);
            }
        }
    }
    if parts.is_empty() {
        return None; // không đọc được gì — fail-open
    }
    let mut sentence = parts.join(" ");
    // "chẵn" chuẩn kế toán khi số tròn hàng nghìn trở lên.
    let is_even_thousand = trimmed.len() >= 4 && trimmed.ends_with("000");
    if is_even_thousand {
        sentence.push_str(" đồng chẵn");
    } else {
        sentence.push_str(" đồng");
    }
    let mut chars: Vec<char> = sentence.chars().collect();
    if let Some(first) = chars.first_mut() {
        if let Some(up) = first.to_uppercase().next() {
            *first = up;
        }
    }
    Some(chars.into_iter().collect())
}

/// Kiểm tra đuôi `tail` có khớp mẫu `=<số>(d|vnd|bc)` không; trả về
/// `(số ký tự cần xoá kể cả `=`, chuỗi mở rộng)`.
///
/// `=` phải ở đầu từ (trước nó không phải chữ/số/gạch dưới) để không đụng
/// `a=15d`, `x_2=3bc`; hậu tố phân biệt hoa/thường (`VND`/`VnD` vẫn ăn).
pub fn find_number_shorthand(tail: &[char]) -> Option<(usize, String)> {
    let eq_pos = tail.iter().rposition(|&c| c == '=')?;
    if eq_pos > 0 && (tail[eq_pos - 1].is_alphanumeric() || tail[eq_pos - 1] == '_') {
        return None;
    }
    let token: String = tail[eq_pos + 1..].iter().collect();
    let lower = token.to_ascii_lowercase();
    // (clippy::question_mark) chuỗi hậu tố suy biến qua `?` thay vì return-None
    // rải rác: `vnd` ưu tiên trước `d` (vì `d` là tiền tố của `vnd`).
    let matched = if let Some(d) = lower.strip_suffix("vnd") {
        Some((d, "vnd"))
    } else if let Some(d) = lower.strip_suffix("bc") {
        Some((d, "bc"))
    } else {
        lower.strip_suffix('d').map(|d| (d, "d"))
    };
    let (digits, mode) = matched?;
    if digits.is_empty() || digits.len() > 12 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let expanded = match mode {
        "d" => format!("{} đồng", format_thousands(digits)),
        "vnd" => format!("{} VNĐ", format_thousands(digits)),
        _ => read_vnd_words(digits)?,
    };
    Some((tail.len() - eq_pos, expanded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_thousands_chuan() {
        assert_eq!(format_thousands("15450000"), "15.450.000");
        assert_eq!(format_thousands("125000"), "125.000");
        assert_eq!(format_thousands("999"), "999");
        assert_eq!(format_thousands("000"), "0");
        assert_eq!(format_thousands("001234"), "1.234");
    }

    #[test]
    fn doc_so_chuan_ke_toan() {
        assert_eq!(
            read_vnd_words("15000000").as_deref(),
            Some("Mười lăm triệu đồng chẵn")
        );
        assert_eq!(
            read_vnd_words("125000").as_deref(),
            Some("Một trăm hai mươi lăm nghìn đồng chẵn")
        );
        assert_eq!(
            read_vnd_words("1500000").as_deref(),
            Some("Một triệu năm trăm nghìn đồng chẵn")
        );
        // Lẻ: 1.500.001 không tròn → không "chẵn"; nhóm có hàng trăm = 0 nhưng
        // nhóm không rỗng đọc "không trăm lẻ …" (chuẩn kế toán: 1.001 =
        // "một nghìn không trăm lẻ một").
        assert_eq!(
            read_vnd_words("1500001").as_deref(),
            Some("Một triệu năm trăm nghìn không trăm lẻ một đồng")
        );
        // Nhóm giữa toàn 0 bị bỏ đúng ("một tỷ không trăm lẻ một", không có
        // "không trăm không nghìn").
        assert_eq!(
            read_vnd_words("1000000001").as_deref(),
            Some("Một tỷ không trăm lẻ một đồng")
        );
        // Mười một / hai mươi mốt / bốn mươi tư / lăm.
        assert_eq!(read_vnd_words("11").as_deref(), Some("Mười một đồng"));
        assert_eq!(read_vnd_words("21").as_deref(), Some("Hai mươi mốt đồng"));
        assert_eq!(read_vnd_words("44").as_deref(), Some("Bốn mươi tư đồng"));
        assert_eq!(read_vnd_words("25").as_deref(), Some("Hai mươi lăm đồng"));
        // 105 = một trăm lẻ năm; 100 = một trăm đồng.
        assert_eq!(
            read_vnd_words("105").as_deref(),
            Some("Một trăm lẻ năm đồng")
        );
        assert_eq!(read_vnd_words("100").as_deref(), Some("Một trăm đồng"));
        // 0 → "Không đồng" (trim đầu tiên; toàn số 0 cũng vậy); vượt 12 chữ
        // số / có ký tự lạ → None (fail-open).
        assert_eq!(read_vnd_words("0").as_deref(), Some("Không đồng"));
        assert_eq!(read_vnd_words("000").as_deref(), Some("Không đồng"));
        assert_eq!(read_vnd_words("1234567890123"), None);
        assert_eq!(read_vnd_words("12a4"), None);
    }

    #[test]
    fn find_shorthand_mau_va_loai_tru() {
        let t = |s: &str| s.chars().collect::<Vec<char>>();
        assert_eq!(
            find_number_shorthand(&t("=15450000d")),
            Some((10, "15.450.000 đồng".to_string()))
        );
        assert_eq!(
            find_number_shorthand(&t("=15450000vnd")),
            Some((12, "15.450.000 VNĐ".to_string()))
        );
        assert_eq!(
            find_number_shorthand(&t("=15000000bc")),
            Some((11, "Mười lăm triệu đồng chẵn".to_string()))
        );
        assert_eq!(
            find_number_shorthand(&t("=125000BC")),
            Some((9, "Một trăm hai mươi lăm nghìn đồng chẵn".to_string()))
        );
        // `=` sau chữ/số/gạch dưới → không phải cú pháp số tiền.
        assert_eq!(find_number_shorthand(&t("a=15d")), None);
        assert_eq!(find_number_shorthand(&t("x_2=3bc")), None);
        // Thiếu số / quá dài / ký tự lạ → None.
        assert_eq!(find_number_shorthand(&t("=d")), None);
        assert_eq!(find_number_shorthand(&t("=1234567890123d")), None);
        assert_eq!(find_number_shorthand(&t("=12a4d")), None);
        // Không có `=` → None.
        assert_eq!(find_number_shorthand(&t("xin chào")), None);
    }
}
