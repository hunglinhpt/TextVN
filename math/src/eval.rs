// SPDX-License-Identifier: GPL-3.0-or-later
//! Pure-Rust bounded Shunting-Yard inline calculator
//! Triggered by (= <expr> =), supports + - * / ^ %, sqrt, sin, cos, tan, ln, log10, abs, pi, e
//! Hard caps: MAX_DEPTH = 64, MAX_TOKENS = 4096, graceful None on overflow/NaN/div-by-zero

pub const MAX_DEPTH: usize = 64;
pub const MAX_TOKENS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Num(f64),
    Op(char),
    LParen,
    RParen,
    Func(&'static str),
}

/// Tokenize input. Returns None on too many tokens or invalid char.
fn tokenize(s: &str) -> Option<Vec<Tok>> {
    let mut toks = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > MAX_TOKENS {
        return None;
    }
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let n: f64 = num_str.parse().ok()?;
            if !n.is_finite() {
                return None;
            }
            toks.push(Tok::Num(n));
        } else if "+-*/^%".contains(c) {
            toks.push(Tok::Op(c));
            i += 1;
        } else if c == '(' {
            toks.push(Tok::LParen);
            i += 1;
        } else if c == ')' {
            toks.push(Tok::RParen);
            i += 1;
        } else if c.is_ascii_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            let func = match name.as_str() {
                "sqrt" => "sqrt",
                "sin" => "sin",
                "cos" => "cos",
                "tan" => "tan",
                "ln" => "ln",
                "log" | "log10" => "log10",
                "abs" => "abs",
                "pi" => "pi",
                "e" => "e",
                _ => return None,
            };
            toks.push(Tok::Func(func));
        } else {
            return None;
        }
    }
    if toks.len() > MAX_TOKENS {
        return None;
    }
    Some(toks)
}

fn precedence(op: char) -> u8 {
    match op {
        '+' | '-' => 1,
        '*' | '/' | '%' => 2,
        '^' => 3,
        // Unary plus/minus: precedence cao nhất — áp TRƯỚC mọi toán tử nhị phân
        // (`-5+3` = -2), nhưng khi gặp '^' thì KHÔNG pop (break) để `-2^2` = -(2^2) = -4.
        'u' => 4,
        _ => 0,
    }
}

/// Pop unary 'u' từ op stack: đổi dấu đỉnh output. `None` nếu thiếu operand.
fn apply_unary(output: &mut Vec<f64>) -> Option<()> {
    let a = output.pop()?;
    output.push(-a);
    Some(())
}

fn apply_op(op: char, a: f64, b: f64) -> Option<f64> {
    let r = match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => {
            if b == 0.0 {
                return None;
            }
            a / b
        }
        '%' => {
            if b == 0.0 {
                return None;
            }
            a % b
        }
        '^' => a.powf(b),
        _ => return None,
    };
    if r.is_finite() {
        Some(r)
    } else {
        None
    }
}

fn apply_func(func: &str, x: f64) -> Option<f64> {
    let r = match func {
        "sqrt" => {
            if x < 0.0 {
                return None;
            }
            x.sqrt()
        }
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "ln" => {
            if x <= 0.0 {
                return None;
            }
            x.ln()
        }
        "log10" => {
            if x <= 0.0 {
                return None;
            }
            x.log10()
        }
        "abs" => x.abs(),
        "pi" => std::f64::consts::PI,
        "e" => std::f64::consts::E,
        _ => return None,
    };
    if r.is_finite() {
        Some(r)
    } else {
        None
    }
}

/// Evaluate expression using Shunting-Yard. Returns None on any error.
pub fn eval(expr: &str) -> Option<f64> {
    let toks = tokenize(expr)?;
    if toks.is_empty() {
        return None;
    }

    let mut output: Vec<f64> = Vec::new();
    let mut ops: Vec<Tok> = Vec::new();
    let mut depth = 0usize;

    let mut expect_operand = true;

    for tok in &toks {
        match tok {
            Tok::Num(n) => {
                if !expect_operand {
                    return None; // two numbers in a row
                }
                output.push(*n);
                expect_operand = false;
            }
            Tok::Func(f) => {
                // Hằng số không đối số: pi / e — đẩy giá trị ngay (E57: trước đây
                // được đẩy vào op stack rồi pop() đòi operand → eval("pi") = None).
                if *f == "pi" || *f == "e" {
                    if !expect_operand {
                        return None; // "2pi" không hợp lệ (cần toán tử)
                    }
                    output.push(apply_func(f, 0.0)?);
                    expect_operand = false;
                } else {
                    ops.push(Tok::Func(f));
                    expect_operand = true;
                }
            }
            Tok::Op(c) => {
                if expect_operand {
                    // Unary minus/plus
                    if *c == '-' {
                        // Treat as 0 - x: push special marker
                        ops.push(Tok::Op('u')); // unary
                        expect_operand = true;
                        continue;
                    } else if *c == '+' {
                        expect_operand = true;
                        continue;
                    } else {
                        return None;
                    }
                }
                // '^' kết hợp PHẢI (right-assoc: 2^3^2 = 2^(3^2) = 512, E57):
                // pop khi precedence(top) > current, hoặc == nhưng KHÔNG phải '^'.
                let right_assoc = *c == '^';
                while let Some(top) = ops.last() {
                    match top {
                        Tok::Op(top_op)
                            if precedence(*top_op) > precedence(*c)
                                || (!right_assoc && precedence(*top_op) == precedence(*c)) =>
                        {
                            let top = ops.pop().unwrap();
                            if let Tok::Op('u') = top {
                                apply_unary(&mut output)?;
                            } else if let Tok::Op(o) = top {
                                let b = output.pop()?;
                                let a = output.pop()?;
                                output.push(apply_op(o, a, b)?);
                            }
                        }
                        // `-2^2` = -(2^2): 'u' giữ trên stack khi push '^' (E57)
                        Tok::Op('u') if *c == '^' => break,
                        Tok::Func(_) => {
                            let f = ops.pop().unwrap();
                            if let Tok::Func(name) = f {
                                let x = output.pop()?;
                                output.push(apply_func(name, x)?);
                            }
                        }
                        _ => break,
                    }
                }
                ops.push(Tok::Op(*c));
                expect_operand = true;
            }
            Tok::LParen => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return None;
                }
                ops.push(Tok::LParen);
                expect_operand = true;
            }
            Tok::RParen => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                while let Some(top) = ops.pop() {
                    match top {
                        Tok::LParen => break,
                        Tok::Op('u') => {
                            apply_unary(&mut output)?;
                        }
                        Tok::Op(o) => {
                            let b = output.pop()?;
                            let a = output.pop()?;
                            output.push(apply_op(o, a, b)?);
                        }
                        Tok::Func(name) => {
                            let x = output.pop()?;
                            output.push(apply_func(name, x)?);
                        }
                        _ => return None,
                    }
                }
                expect_operand = false;
            }
        }
    }

    while let Some(top) = ops.pop() {
        match top {
            Tok::Op('u') => {
                let a = output.pop()?;
                output.push(-a);
            }
            Tok::Op(o) => {
                let b = output.pop()?;
                let a = output.pop()?;
                output.push(apply_op(o, a, b)?);
            }
            Tok::Func(name) => {
                let x = output.pop()?;
                output.push(apply_func(name, x)?);
            }
            Tok::LParen => return None, // unmatched
            _ => {}
        }
    }

    if output.len() == 1 {
        let r = output[0];
        if r.is_finite() {
            Some(r)
        } else {
            None
        }
    } else {
        None
    }
}

/// Kiểm tra chuỗi bên trong biểu thức inline eval (`=expr=` đang mở, đã bỏ dấu `=` đầu)
/// có còn khả năng hợp lệ thành biểu thức toán học hay không. Nếu người dùng chỉ gõ dấu `=`
/// trong văn xuôi (`Ta có x = 5, nên kết quả...`), trả `false` ngay ở ký tự phá vỡ (`,`,
/// chữ Việt, từ không phải tên hàm toán học...) để thoát eval tức thì và không làm chết Telex.
pub fn is_valid_eval_prefix(inner: &str) -> bool {
    const ALLOWED_IDENTS: &[&str] = &[
        "sqrt", "sin", "cos", "tan", "ln", "log", "log10", "abs", "pi", "e",
    ];
    let bytes = inner.as_bytes();
    let mut i = 0;
    let mut last_was_operand = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b' ' {
            i += 1;
            continue;
        }
        if b.is_ascii_digit() || b == b'.' {
            if last_was_operand {
                // Hai số/toán hạng tách bởi space mà không có toán tử (`= 5 6` hoặc `= 5 a`)
                // — không phải biểu thức eval hợp lệ.
                return false;
            }
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            last_was_operand = true;
            continue;
        }
        if matches!(b, b'+' | b'-' | b'*' | b'/' | b'^' | b'%' | b'(') {
            last_was_operand = false;
            i += 1;
            continue;
        }
        if b == b')' {
            last_was_operand = true;
            i += 1;
            continue;
        }
        if b.is_ascii_alphabetic() {
            if last_was_operand {
                // Ví dụ `= 5 n` ("x = 5 nên") — chữ cái sau số tách bằng space không phải eval.
                return false;
            }
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let ident = &inner[start..i];
            let at_end = i == bytes.len();
            let ok = if at_end {
                // Đang gõ dở ở cuối buffer: chấp nhận tiền tố của tên hàm/hằng số hợp lệ.
                ALLOWED_IDENTS
                    .iter()
                    .any(|&name| name.starts_with(ident) || name.eq_ignore_ascii_case(ident))
                    || ALLOWED_IDENTS.iter().any(|&name| {
                        name.len() >= ident.len() && name[..ident.len()].eq_ignore_ascii_case(ident)
                    })
            } else {
                // Đã kết thúc định danh (sau là space/toán tử/`(`): phải khớp chính xác hàm/hằng số.
                ALLOWED_IDENTS
                    .iter()
                    .any(|&name| name.eq_ignore_ascii_case(ident))
            };
            if !ok {
                return false;
            }
            last_was_operand = true;
            continue;
        }
        // Bất kỳ ký tự nào khác (`,`, `\n`, `\t`, `;`, chữ có dấu Unicode...) đều phá vỡ eval.
        return false;
    }
    true
}

/// Format kết quả eval ra text gọn: số nguyên không có `.0`, loại bỏ nhiễu dấu phẩy động
/// IEEE-754 (`0.1+0.2` → `"0.3"`, `sin(pi/6)` → `"0.5"`, `cos(pi/2)` → `"0"`, `-0` → `"0"`);
/// NaN/Inf không bao giờ tới đây (eval đã guard), nhưng vẫn chặn an toàn.
pub fn format_value(v: f64) -> String {
    if !v.is_finite() {
        return String::from("NaN");
    }
    let clean = if v.abs() < 1e-12 { 0.0 } else { v };
    let rounded_int = clean.round();
    if clean.abs() < 1e15 && (clean - rounded_int).abs() <= 1e-12 * clean.abs().max(1.0) {
        return format!("{}", rounded_int as i64);
    }
    if clean.abs() >= 1e-6 && clean.abs() < 1e12 {
        // Làm tròn 12 chữ số có nghĩa để xoá nhiễu `0.30000000000000004` / `0.49999999999999994`.
        let mag = clean.abs().log10().floor() as i32;
        let dec = (11 - mag).clamp(0, 12) as usize;
        let s = format!("{clean:.dec$}");
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed == "-0" || trimmed.is_empty() {
            return String::from("0");
        }
        return trimmed.to_string();
    }
    format!("{clean}")
}

/// Check if input is wrapped in (= ... =) delimiters and evaluate inner expression
pub fn eval_delimited(input: &str) -> Option<f64> {
    let trimmed = input.trim();
    if trimmed.len() >= 6 && trimmed.starts_with("(= ") && trimmed.ends_with(" =)") {
        eval(&trimmed[3..trimmed.len() - 2])
    } else if trimmed.len() >= 4 && trimmed.starts_with("= ") && trimmed.ends_with(" =") {
        eval(&trimmed[2..trimmed.len() - 2])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        assert_eq!(eval("2+3"), Some(5.0));
        assert_eq!(eval("10-4"), Some(6.0));
        assert_eq!(eval("3*4"), Some(12.0));
        assert_eq!(eval("15/3"), Some(5.0));
        assert_eq!(eval("2^10"), Some(1024.0));
        assert_eq!(eval("17%5"), Some(2.0));
    }

    #[test]
    fn precedence() {
        assert_eq!(eval("2+3*4"), Some(14.0));
        assert_eq!(eval("(2+3)*4"), Some(20.0));
        assert_eq!(eval("2^3^2"), Some(512.0)); // right-assoc
    }

    #[test]
    fn functions() {
        assert_eq!(eval("sqrt(16)"), Some(4.0));
        assert!((eval("sin(0)").unwrap() - 0.0).abs() < 1e-10);
        assert!((eval("cos(0)").unwrap() - 1.0).abs() < 1e-10);
        assert!((eval("ln(e)").unwrap() - 1.0).abs() < 1e-10);
        assert_eq!(eval("abs(-5)"), Some(5.0));
        assert!((eval("pi").unwrap() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn div_by_zero_returns_none() {
        assert_eq!(eval("1/0"), None);
        assert_eq!(eval("1%0"), None);
    }

    #[test]
    fn malformed_returns_none() {
        assert_eq!(eval(""), None);
        assert_eq!(eval("2+"), None);
        assert_eq!(eval("+*3"), None);
        assert_eq!(eval("(2+3"), None);
        assert_eq!(eval("2..5+1"), None);
        assert_eq!(eval("hello"), None);
    }

    #[test]
    fn overflow_returns_none() {
        assert_eq!(eval("1e308*10"), None); // parse fails or inf
        assert_eq!(eval("999999999999999999999999999999999999999^999"), None);
    }

    #[test]
    fn negative_numbers() {
        assert_eq!(eval("-5+3"), Some(-2.0));
        assert_eq!(eval("3*-2"), Some(-6.0));
        assert_eq!(eval("-(2+3)"), Some(-5.0));
    }

    #[test]
    fn delimited_eval() {
        assert_eq!(eval_delimited("(= 2+3 =)"), Some(5.0));
        assert_eq!(eval_delimited("= 10*5 ="), Some(50.0));
        assert_eq!(eval_delimited("2+3"), None); // no delimiters
    }

    #[test]
    fn depth_cap() {
        let deep = format!("{}1{}", "(".repeat(100), ")".repeat(100));
        assert_eq!(eval(&deep), None); // exceeds MAX_DEPTH
    }
}
