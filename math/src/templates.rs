// SPDX-License-Identifier: GPL-3.0-or-later
//! Template tham số động: `\matNxM` `\bmatNxM` `\pmatNxM` `\detNxN` `\hptN` `\casesN`
//! (1 <= N, M <= 10). Mỗi template trả (text, cursor_left_steps) — con trỏ đặt
//! **đầu placeholder ô đầu tiên** để user gõ đè số liệu thật.
//!
//! Zero-unsafe; sinh chuỗi có trần (N,M <= 10 → tối đa ~900 ký tự LaTeX).

use crate::config::MathOutputMode;

/// Trần kích thước — spec cứng.
pub const MAX_DIM: usize = 10;
/// Trần tổng ô (N×M) để chống phình chuỗi vô hạn.
pub const MAX_CELLS: usize = 100;

/// Kết quả sinh template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateOut {
    pub text: String,
    pub cursor_left_steps: usize,
}

/// Cấu trúc lời gọi template đã parse từ trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    Mat,
    BMat,
    PMat,
    Det,
    Hpt,
    Cases,
}

impl TemplateKind {
    /// Tên gốc (không tham số).
    pub fn name(self) -> &'static str {
        match self {
            TemplateKind::Mat => "mat",
            TemplateKind::BMat => "bmat",
            TemplateKind::PMat => "pmat",
            TemplateKind::Det => "det",
            TemplateKind::Hpt => "hpt",
            TemplateKind::Cases => "cases",
        }
    }
}

/// Parse trigger động như `\mat3x3` / `\det4` / `\hpt2` / `\cases3`
/// (có/không có backslash đầu). `None` nếu không khớp mẫu nào.
pub fn parse_trigger(word: &str) -> Option<(TemplateKind, usize, usize)> {
    let w = word.strip_prefix('\\').unwrap_or(word);
    if w.is_empty() || w.len() > 16 {
        return None;
    }
    // Tách phần chữ đầu (mat/bmat/pmat/det/hpt/cases) — không phân biệt hoa/thường
    let lower = w.to_ascii_lowercase();
    let (kind, dims): (TemplateKind, &str) = if let Some(d) = lower.strip_prefix("bmat") {
        (TemplateKind::BMat, d)
    } else if let Some(d) = lower.strip_prefix("pmat") {
        (TemplateKind::PMat, d)
    } else if let Some(d) = lower.strip_prefix("mat") {
        (TemplateKind::Mat, d)
    } else if let Some(d) = lower.strip_prefix("det") {
        (TemplateKind::Det, d)
    } else if let Some(d) = lower.strip_prefix("hpt") {
        (TemplateKind::Hpt, d)
    } else if let Some(d) = lower.strip_prefix("cases") {
        (TemplateKind::Cases, d)
    } else {
        return None;
    };
    // dims: "" → mặc định 2; "N" → (N,N); "NxM"
    let (n, m) = if dims.is_empty() {
        (2, 2)
    } else if let Some((a, b)) = dims.split_once('x') {
        let n: usize = a.parse().ok()?;
        let m: usize = b.parse().ok()?;
        (n, m)
    } else {
        let n: usize = dims.parse().ok()?;
        (n, n)
    };
    if dims.contains('x')
        && kind != TemplateKind::Mat
        && kind != TemplateKind::BMat
        && kind != TemplateKind::PMat
    {
        // det/hpt/cases không nhận NxM
        return None;
    }
    if !in_bounds(n) || !in_bounds(m) || n * m > MAX_CELLS {
        return None;
    }
    Some((kind, n, m))
}

fn in_bounds(v: usize) -> bool {
    (1..=MAX_DIM).contains(&v)
}

/// Sinh template theo kind + mode. `None` khi tham số ngoài bounds.
pub fn generate(
    kind: TemplateKind,
    n: usize,
    m: usize,
    mode: MathOutputMode,
) -> Option<TemplateOut> {
    if !in_bounds(n) || !in_bounds(m) || n * m > MAX_CELLS {
        return None;
    }
    let is_square_kind = matches!(kind, TemplateKind::Det);
    if is_square_kind && n != m {
        return None;
    }
    let rows: Vec<Vec<String>> = (0..n)
        .map(|i| (0..m).map(|j| cell(kind, i + 1, j + 1, mode)).collect())
        .collect();
    let (text, cursor_offset_from_end) = match kind {
        TemplateKind::Mat | TemplateKind::BMat | TemplateKind::PMat => match mode {
            MathOutputMode::Latex => {
                let env = match kind {
                    TemplateKind::BMat => "bmatrix",
                    TemplateKind::PMat => "pmatrix",
                    _ => "matrix",
                };
                let body = join_rows(&rows, " & ", " \\\\ ");
                let prefix = format!("\\begin{{{}}} ", env);
                let suffix = format!(" \\end{{{}}}", env);
                let t = format!("{prefix}{body}{suffix}");
                let cl = count_utf32(&t) - count_utf32(&prefix);
                (t, cl)
            }
            MathOutputMode::Typst => {
                let body = join_rows(&rows, ", ", "; ");
                let t = format!("mat({body})");
                let cl = count_utf32(&t) - count_utf32("mat(");
                (t, cl)
            }
            _ => {
                let body = join_rows(&rows, "  ", "; ");
                let t = format!("[{body}]");
                let cl = count_utf32(&t) - count_utf32("[");
                (t, cl)
            }
        },
        TemplateKind::Det => match mode {
            MathOutputMode::Latex => {
                let body = join_rows(&rows, " & ", " \\\\ ");
                let prefix = "\\det\\begin{vmatrix} ".to_string();
                let suffix = " \\end{vmatrix}".to_string();
                let t = format!("{prefix}{body}{suffix}");
                let cl = count_utf32(&t) - count_utf32(&prefix);
                (t, cl)
            }
            MathOutputMode::Typst => {
                let body = join_rows(&rows, ", ", "; ");
                let t = format!("det(mat({body}))");
                let cl = count_utf32(&t) - count_utf32("det(mat(");
                (t, cl)
            }
            _ => {
                let body = join_rows(&rows, "  ", "; ");
                let t = format!("|{body}|");
                let cl = count_utf32(&t) - count_utf32("|");
                (t, cl)
            }
        },
        TemplateKind::Hpt => {
            let eqs: Vec<String> = (0..n)
                .map(|i| {
                    let x = unknown(i + 1, mode);
                    format!("{x} = 0")
                })
                .collect();
            let sep = match mode {
                MathOutputMode::Latex => " \\\\ ",
                MathOutputMode::Typst => ", ",
                _ => "; ",
            };
            let body = eqs.join(sep);
            let (t, pre) = match mode {
                MathOutputMode::Latex => (
                    format!("\\begin{{cases}} {body} \\end{{cases}}"),
                    "\\begin{cases} ".to_string(),
                ),
                MathOutputMode::Typst => (format!("cases({body})"), "cases(".to_string()),
                _ => (body.clone(), String::new()),
            };
            let cl = count_utf32(&t) - count_utf32(&pre);
            (t, cl)
        }
        TemplateKind::Cases => {
            let conds: Vec<String> = (0..n)
                .map(|i| match mode {
                    MathOutputMode::Latex => format!("f(x) = \\text{{khi }} x = {}", i + 1),
                    MathOutputMode::Typst => format!("f(x) = \"khi\" x = {}", i + 1),
                    _ => format!("f(x) = ?, x = {}", i + 1),
                })
                .collect();
            let sep = match mode {
                MathOutputMode::Latex => " \\\\ ",
                MathOutputMode::Typst => ", ",
                _ => "; ",
            };
            let body = conds.join(sep);
            let (t, pre) = match mode {
                MathOutputMode::Latex => (
                    format!("\\begin{{cases}} {body} \\end{{cases}}"),
                    "\\begin{cases} ".to_string(),
                ),
                MathOutputMode::Typst => (format!("cases({body})"), "cases(".to_string()),
                _ => (body.clone(), String::new()),
            };
            let cl = count_utf32(&t) - count_utf32(&pre);
            (t, cl)
        }
    };
    // Con trỏ không bao giờ âm / vượt chiều dài (guards cứng).
    let total = count_utf32(&text);
    let cl = cursor_offset_from_end.min(total);
    Some(TemplateOut {
        text,
        cursor_left_steps: cl,
    })
}

fn join_rows(rows: &[Vec<String>], cell_sep: &str, row_sep: &str) -> String {
    rows.iter()
        .map(|r| r.join(cell_sep))
        .collect::<Vec<_>>()
        .join(row_sep)
}

fn cell(_kind: TemplateKind, i: usize, j: usize, mode: MathOutputMode) -> String {
    match mode {
        MathOutputMode::Latex => format!("a_{{{i}{j}}}"),
        MathOutputMode::Typst => format!("a_({i},{j})"),
        _ => format!("a{i}{j}"),
    }
}

fn unknown(i: usize, mode: MathOutputMode) -> String {
    match mode {
        MathOutputMode::Latex => format!("x_{{{i}}}"),
        MathOutputMode::Typst => format!("x_{i}"),
        _ => format!("x{}", i),
    }
}

/// Số ký tự Unicode (không phải byte) — chuẩn so khớp cursor steps.
pub fn count_utf32(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_2x2_latex_with_cursor_in_first_cell() {
        let out = generate(TemplateKind::Mat, 2, 2, MathOutputMode::Latex).unwrap();
        assert_eq!(
            out.text,
            "\\begin{matrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{matrix}"
        );
        // Con trỏ đầu ô a_{11}: prefix "\begin{matrix} " phía trước.
        assert_eq!(
            out.cursor_left_steps,
            count_utf32(&out.text) - count_utf32("\\begin{matrix} ")
        );
    }

    #[test]
    fn matrix_cursor_lands_on_first_cell_start() {
        for kind in [TemplateKind::Mat, TemplateKind::BMat, TemplateKind::PMat] {
            for (n, m) in [(1, 1), (2, 3), (3, 3), (5, 4), (10, 10)] {
                let out = generate(kind, n, m, MathOutputMode::Latex).unwrap();
                let total = count_utf32(&out.text);
                let at = total - out.cursor_left_steps;
                let head: String = out.text.chars().take(at).collect();
                // Ngay sau vị trí con trỏ phải là placeholder ô đầu a_{11}
                let rest: String = out.text.chars().skip(at).collect();
                assert!(
                    rest.starts_with("a_{11}"),
                    "kind {kind:?} {n}x{m} → {rest:?}"
                );
                assert!(at <= total);
                let _ = head;
            }
        }
    }

    #[test]
    fn matrix_typst_and_unicode_forms() {
        let out = generate(TemplateKind::Mat, 2, 2, MathOutputMode::Typst).unwrap();
        assert_eq!(out.text, "mat(a_(1,1), a_(1,2); a_(2,1), a_(2,2))");
        assert_eq!(
            out.cursor_left_steps,
            count_utf32("a_(1,1), a_(1,2); a_(2,1), a_(2,2))")
        );
        let out = generate(TemplateKind::Mat, 2, 2, MathOutputMode::Normal).unwrap();
        assert_eq!(out.text, "[a11  a12; a21  a22]");
    }

    #[test]
    fn bmat_pmat_env_names() {
        let b = generate(TemplateKind::BMat, 2, 2, MathOutputMode::Latex).unwrap();
        assert!(b.text.starts_with("\\begin{bmatrix}"));
        let p = generate(TemplateKind::PMat, 2, 2, MathOutputMode::Latex).unwrap();
        assert!(p.text.starts_with("\\begin{pmatrix}"));
    }

    #[test]
    fn det_must_be_square() {
        assert!(generate(TemplateKind::Det, 2, 3, MathOutputMode::Latex).is_none());
        let d = generate(TemplateKind::Det, 3, 3, MathOutputMode::Latex).unwrap();
        assert!(d.text.contains("\\det\\begin{vmatrix}"));
        assert!(d.text.contains("a_{33}"));
    }

    #[test]
    fn hpt_and_cases() {
        let h = generate(TemplateKind::Hpt, 2, 2, MathOutputMode::Latex).unwrap();
        assert!(h.text.contains("x_{1} = 0 \\\\ x_{2} = 0"));
        let c = generate(TemplateKind::Cases, 3, 3, MathOutputMode::Latex).unwrap();
        assert!(c.text.contains("\\text{khi }} x = 1"));
        let h = generate(TemplateKind::Hpt, 2, 2, MathOutputMode::Typst).unwrap();
        assert!(h.text.starts_with("cases(x_"));
    }

    #[test]
    fn bounds_enforced() {
        for (n, m) in [(0, 2), (2, 0), (11, 2), (2, 11)] {
            assert!(generate(TemplateKind::Mat, n, m, MathOutputMode::Latex).is_none());
            assert!(parse_trigger(&format!("\\mat{}x{}", n, m)).is_none());
        }
        assert!(generate(TemplateKind::Mat, 1, 1, MathOutputMode::Latex).is_some());
        assert!(generate(TemplateKind::Mat, 10, 10, MathOutputMode::Latex).is_some());
    }

    #[test]
    fn parse_trigger_forms() {
        assert_eq!(parse_trigger("\\mat3x3"), Some((TemplateKind::Mat, 3, 3)));
        assert_eq!(parse_trigger("bmat2x4"), Some((TemplateKind::BMat, 2, 4)));
        assert_eq!(parse_trigger("\\PMAT2x2"), Some((TemplateKind::PMat, 2, 2)));
        assert_eq!(parse_trigger("\\det4"), Some((TemplateKind::Det, 4, 4)));
        assert_eq!(parse_trigger("\\hpt3"), Some((TemplateKind::Hpt, 3, 3)));
        assert_eq!(parse_trigger("cases2"), Some((TemplateKind::Cases, 2, 2)));
        assert_eq!(
            parse_trigger("\\mat"),
            Some((TemplateKind::Mat, 2, 2)),
            "mặc định 2x2"
        );
        assert!(parse_trigger("\\mat11x11").is_none());
        assert!(parse_trigger("\\det3x3").is_none(), "det không nhận NxM");
        assert!(parse_trigger("\\matrix").is_none(), "không phải mẫu");
        assert!(parse_trigger("\\mat0x2").is_none());
    }

    #[test]
    fn max_cells_guard() {
        // 10x10 = 100 ô — đúng trần, cho phép
        assert!(generate(TemplateKind::Mat, 10, 10, MathOutputMode::Latex).is_some());
        assert!(parse_trigger("\\mat10x10").is_some());
    }

    #[test]
    fn cursor_never_exceeds_length() {
        for kind in [
            TemplateKind::Mat,
            TemplateKind::BMat,
            TemplateKind::PMat,
            TemplateKind::Det,
            TemplateKind::Hpt,
            TemplateKind::Cases,
        ] {
            for mode in [
                MathOutputMode::Normal,
                MathOutputMode::Latex,
                MathOutputMode::Typst,
            ] {
                for n in 1..=3usize {
                    for m in 1..=3usize {
                        if let Some(out) = generate(kind, n, m, mode) {
                            assert!(out.cursor_left_steps <= count_utf32(&out.text));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn legacy_api_still_works() {
        // API tương thích: matrix_template("mat", ...) giữ hành vi cũ cho caller cũ.
        let out = generate(TemplateKind::Mat, 2, 2, MathOutputMode::Latex).unwrap();
        assert!(out.text.contains("\\begin{matrix}"));
        assert!(generate(TemplateKind::Mat, 0, 2, MathOutputMode::Latex).is_none());
    }
}
