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
        // `% 3` thay vì `u32::is_multiple_of`: hàm đó CHƯA ổn định trên rustc
        // 1.85 của Debian trixie (ci-deb build bằng toolchain distro) —
        // clippy mới hơn thì gợi ý đổi, nên miễn trừ tường minh tại chỗ.
        #[allow(clippy::manual_is_multiple_of)]
        if i > 0 && (bytes.len() - i) % 3 == 0 {
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

// ─── OP-010 (vòng 56 — D12): gõ tắt ngày/tháng tự động ───────────────────────
// - `=ngay`   → `dd/mm/yyyy` (hôm nay, theo giờ máy — S1: std::time không I/O)
// - `=thang`  → `mm/yyyy`
// - `=nam`    → `yyyy`
// Kế toán/hợp đồng gõ ngày rất nhiều; macro này định dạng chuẩn dd/mm/yyyy
// Việt Nam. Giờ hệ thống lấy qua std::time::UNIX_EPOCH (0 I/O ngoài clock,
// deterministic trong test qua tham số `days_since_epoch`).

/// Ngày/tháng từ `days_since_epoch` (1970-01-01 = 0) — thuật toán civil-from-days
/// chuẩn (Howard Hinnant), 0 dependency, chính xác leap-year.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Ngày hiện hành (days since epoch) — tách hàm để test xác định.
fn days_since_epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64 / 86_400)
        .unwrap_or(0)
}

/// `=ngay` / `=thang` / `=nam` → ngày/tháng/năm hiện hành dạng Việt Nam.
/// Trả `None` nếu `token` không khớp (caller giữ chuỗi gõ).
pub fn find_date_shorthand(tail: &[char]) -> Option<(usize, String)> {
    const KEYS: [(&str, u8); 3] = [("ngay", 0), ("thang", 1), ("nam", 2)];
    let eq_pos = tail.iter().rposition(|&c| c == '=')?;
    if eq_pos > 0 && (tail[eq_pos - 1].is_alphanumeric() || tail[eq_pos - 1] == '_') {
        return None;
    }
    let token: String = tail[eq_pos + 1..].iter().collect::<String>().to_lowercase();
    let mode = KEYS.iter().find(|(k, _)| *k == token)?.1;
    let (y, m, d) = civil_from_days(days_since_epoch_now());
    let expanded = match mode {
        0 => format!("{d:02}/{m:02}/{y}"),
        1 => format!("{m:02}/{y}"),
        _ => format!("{y}"),
    };
    Some((tail.len() - eq_pos, expanded))
}

#[cfg(test)]
mod date_tests {
    use super::*;

    #[test]
    fn civil_from_days_chuan() {
        // 0 = 1970-01-01; 19723 = 2024-01-01 (năm nhuận); mốc cố định
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        // Leap-year boundary: 2024-02-29 = ngày 19782 từ epoch
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        // 2026-11-06 (ngày build feature) = 20454 + 309 (đầu năm→6/11)
        assert_eq!(civil_from_days(20_763), (2026, 11, 6));
    }

    #[test]
    fn ngay_thang_nam_din_hinh() {
        let tail: Vec<char> = "=ngay".chars().collect();
        let (len, out) = find_date_shorthand(&tail).expect("=ngay khớp");
        assert_eq!(len, 5);
        // dạng dd/mm/yyyy: 10 ký tự, 2 dấu /
        assert_eq!(out.matches('/').count(), 2);
        assert_eq!(out.len(), 10);

        let tail: Vec<char> = "=thang".chars().collect();
        let (_, out) = find_date_shorthand(&tail).expect("=thang khớp");
        assert_eq!(out.matches('/').count(), 1);
        assert!(out.len() == 7, "{out}");

        let tail: Vec<char> = "=nam".chars().collect();
        let (_, out) = find_date_shorthand(&tail).expect("=nam khớp");
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn khong_khop_giugung() {
        // không khớp → None, chuỗi gõ giữ nguyên
        assert!(find_date_shorthand(&"=ngayj".chars().collect::<Vec<_>>()).is_none());
        assert!(find_date_shorthand(&"=ngay1".chars().collect::<Vec<_>>()).is_none());
        // `abc=ngay` — '=' không phải đầu token (trước nó alphanumeric) → None
        assert!(find_date_shorthand(&"abc=ngay".chars().collect::<Vec<_>>()).is_none());
    }

    #[test]
    fn hoa_thuong_deu_khop() {
        let tail: Vec<char> = "=NGAY".chars().collect();
        assert!(find_date_shorthand(&tail).is_some());
        let tail: Vec<char> = "=Thang".chars().collect();
        assert!(find_date_shorthand(&tail).is_some());
    }
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
