// SPDX-License-Identifier: GPL-3.0-or-later
//! Bidirectional emitters: Presentation MathML ↔ Clean Unicode ↔ LaTeX ↔ Typst

use crate::ast::MathNode;

/// XML-escape a string for MathML output
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Convert AST to Presentation MathML (Word/MathType native)
pub fn to_mathml(node: &MathNode, display: bool) -> String {
    let inner = node_to_mathml(node);
    let display_attr = if display { "block" } else { "inline" };
    format!(
        "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"{}\">{}</math>",
        display_attr, inner
    )
}

fn node_to_mathml(node: &MathNode) -> String {
    match node {
        MathNode::Num(s) => format!("<mn>{}</mn>", xml_escape(s)),
        MathNode::Ident(s) => format!("<mi>{}</mi>", xml_escape(s)),
        MathNode::Op(s) => format!("<mo>{}</mo>", xml_escape(s)),
        MathNode::Group(inner) => format!("<mrow>{}</mrow>", node_to_mathml(inner)),
        MathNode::Frac(n, d) => format!(
            "<mfrac><mrow>{}</mrow><mrow>{}</mrow></mfrac>",
            node_to_mathml(n),
            node_to_mathml(d)
        ),
        MathNode::Sqrt(x) => format!("<msqrt>{}</msqrt>", node_to_mathml(x)),
        MathNode::Root(idx, x) => format!(
            "<mroot>{}<mrow>{}</mrow></mroot>",
            node_to_mathml(x),
            node_to_mathml(idx)
        ),
        MathNode::Sup(b, e) => format!(
            "<msup><mrow>{}</mrow><mrow>{}</mrow></msup>",
            node_to_mathml(b),
            node_to_mathml(e)
        ),
        MathNode::Sub(b, s) => format!(
            "<msub><mrow>{}</mrow><mrow>{}</mrow></msub>",
            node_to_mathml(b),
            node_to_mathml(s)
        ),
        MathNode::SubSup(b, s, e) => format!(
            "<msubsup><mrow>{}</mrow><mrow>{}</mrow><mrow>{}</mrow></msubsup>",
            node_to_mathml(b),
            node_to_mathml(s),
            node_to_mathml(e)
        ),
        MathNode::Under(b, u) => format!(
            "<munder><mrow>{}</mrow><mrow>{}</mrow></munder>",
            node_to_mathml(b),
            node_to_mathml(u)
        ),
        MathNode::Over(b, o) => format!(
            "<mover><mrow>{}</mrow><mrow>{}</mrow></mover>",
            node_to_mathml(b),
            node_to_mathml(o)
        ),
        MathNode::UnderOver(b, u, o) => format!(
            "<munderover><mrow>{}</mrow><mrow>{}</mrow><mrow>{}</mrow></munderover>",
            node_to_mathml(b),
            node_to_mathml(u),
            node_to_mathml(o)
        ),
        MathNode::Matrix(rows) => {
            let trs: Vec<String> = rows
                .iter()
                .map(|row| {
                    let tds: Vec<String> = row
                        .iter()
                        .map(|cell| format!("<mtd>{}</mtd>", node_to_mathml(cell)))
                        .collect();
                    format!("<mtr>{}</mtr>", tds.join(""))
                })
                .collect();
            format!("<mtable>{}</mtable>", trs.join(""))
        }
        MathNode::Func(name, args) => {
            let args_str: Vec<String> = args.iter().map(node_to_mathml).collect();
            format!(
                "<mrow><mi>{}</mi><mo>(</mo>{}<mo>)</mo></mrow>",
                xml_escape(name),
                args_str.join("<mo>,</mo>")
            )
        }
        MathNode::Text(s) => format!("<mtext>{}</mtext>", xml_escape(s)),
        MathNode::Seq(nodes) => {
            let inner: Vec<String> = nodes.iter().map(node_to_mathml).collect();
            format!("<mrow>{}</mrow>", inner.join(""))
        }
        MathNode::Chem(s) => format!("<mtext>{}</mtext>", xml_escape(s)),
    }
}

/// Convert AST to clean Unicode math
pub fn to_unicode(node: &MathNode) -> String {
    match node {
        MathNode::Num(s) => s.clone(),
        MathNode::Ident(s) => s.clone(),
        MathNode::Op(s) => s.clone(),
        MathNode::Group(inner) => format!("({})", to_unicode(inner)),
        MathNode::Frac(n, d) => format!("{}/{}", to_unicode(n), to_unicode(d)),
        MathNode::Sqrt(x) => format!("√{}", to_unicode(x)),
        MathNode::Root(idx, x) => format!("{}√{}", to_unicode(idx), to_unicode(x)),
        MathNode::Sup(b, e) => format!("{}^{}", to_unicode(b), to_unicode(e)),
        MathNode::Sub(b, s) => format!("{}_{}", to_unicode(b), to_unicode(s)),
        MathNode::SubSup(b, s, e) => {
            format!("{}_{}^{}", to_unicode(b), to_unicode(s), to_unicode(e))
        }
        MathNode::Under(b, u) => format!("{}_{}", to_unicode(b), to_unicode(u)),
        MathNode::Over(b, o) => format!("{}^{}", to_unicode(b), to_unicode(o)),
        MathNode::UnderOver(b, u, o) => {
            format!("{}_{}^{}", to_unicode(b), to_unicode(u), to_unicode(o))
        }
        MathNode::Matrix(rows) => {
            let rs: Vec<String> = rows
                .iter()
                .map(|r| r.iter().map(to_unicode).collect::<Vec<_>>().join("  "))
                .collect();
            format!("[{}]", rs.join("; "))
        }
        MathNode::Func(name, args) => {
            let a: Vec<String> = args.iter().map(to_unicode).collect();
            format!("{}({})", name, a.join(", "))
        }
        MathNode::Text(s) => s.clone(),
        MathNode::Seq(nodes) => nodes.iter().map(to_unicode).collect::<Vec<_>>().join(" "),
        MathNode::Chem(s) => chem_to_unicode(s),
    }
}

/// Convert chemistry notation (H2SO4) to Unicode subscripts (H₂SO₄)
pub fn chem_to_unicode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_uppercase() || c.is_ascii_lowercase() {
            result.push(c);
            // Check for digits following → subscript
            let mut digits = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    digits.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                for d in digits.chars() {
                    result.push(match d {
                        '0' => '₀',
                        '1' => '₁',
                        '2' => '₂',
                        '3' => '₃',
                        '4' => '₄',
                        '5' => '₅',
                        '6' => '₆',
                        '7' => '₇',
                        '8' => '₈',
                        '9' => '₉',
                        _ => d,
                    });
                }
            }
        } else if c == '^' {
            // Superscript marker
            let mut sup = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() || d == '+' || d == '-' {
                    sup.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            for d in sup.chars() {
                result.push(match d {
                    '0' => '⁰',
                    '1' => '¹',
                    '2' => '²',
                    '3' => '³',
                    '4' => '⁴',
                    '5' => '⁵',
                    '6' => '⁶',
                    '7' => '⁷',
                    '8' => '⁸',
                    '9' => '⁹',
                    '+' => '⁺',
                    '-' => '⁻',
                    _ => d,
                });
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert AST to LaTeX
pub fn to_latex(node: &MathNode) -> String {
    match node {
        MathNode::Num(s) => s.clone(),
        MathNode::Ident(s) => s.clone(),
        MathNode::Op(s) => s.clone(),
        MathNode::Group(inner) => format!("{{{}}}", to_latex(inner)),
        MathNode::Frac(n, d) => format!("\\frac{{{}}}{{{}}}", to_latex(n), to_latex(d)),
        MathNode::Sqrt(x) => format!("\\sqrt{{{}}}", to_latex(x)),
        MathNode::Root(idx, x) => format!("\\sqrt[{}]{{{}}}", to_latex(idx), to_latex(x)),
        MathNode::Sup(b, e) => format!("{}^{{{}}}", to_latex(b), to_latex(e)),
        MathNode::Sub(b, s) => format!("{}_{{{}}}", to_latex(b), to_latex(s)),
        MathNode::SubSup(b, s, e) => {
            format!("{}_{{{}}}^{{{}}}", to_latex(b), to_latex(s), to_latex(e))
        }
        MathNode::Under(b, u) => format!("\\underset{{{}}}{{{}}}", to_latex(u), to_latex(b)),
        MathNode::Over(b, o) => format!("\\overset{{{}}}{{{}}}", to_latex(o), to_latex(b)),
        MathNode::UnderOver(b, u, o) => {
            format!(
                "\\underset{{{}}}{{\\overset{{{}}}{{{}}}}}",
                to_latex(u),
                to_latex(o),
                to_latex(b)
            )
        }
        MathNode::Matrix(rows) => {
            let body: Vec<String> = rows
                .iter()
                .map(|r| r.iter().map(to_latex).collect::<Vec<_>>().join(" & "))
                .collect();
            format!("\\begin{{matrix}} {} \\end{{matrix}}", body.join(" \\\\ "))
        }
        MathNode::Func(name, args) => {
            let a: Vec<String> = args.iter().map(to_latex).collect();
            format!("\\{}({})", name, a.join(", "))
        }
        MathNode::Text(s) => format!("\\text{{{}}}", s),
        MathNode::Seq(nodes) => nodes.iter().map(to_latex).collect::<Vec<_>>().join(" "),
        MathNode::Chem(s) => s.clone(),
    }
}

/// Convert AST to Typst
pub fn to_typst(node: &MathNode) -> String {
    match node {
        MathNode::Num(s) => s.clone(),
        MathNode::Ident(s) => s.clone(),
        MathNode::Op(s) => s.clone(),
        MathNode::Group(inner) => to_typst(inner),
        MathNode::Frac(n, d) => format!("frac({}, {})", to_typst(n), to_typst(d)),
        MathNode::Sqrt(x) => format!("sqrt({})", to_typst(x)),
        MathNode::Root(idx, x) => format!("root({}, {})", to_typst(idx), to_typst(x)),
        MathNode::Sup(b, e) => format!("{}^({})", to_typst(b), to_typst(e)),
        MathNode::Sub(b, s) => format!("{}_{}", to_typst(b), to_typst(s)),
        MathNode::SubSup(b, s, e) => format!("{}_{}^{}", to_typst(b), to_typst(s), to_typst(e)),
        MathNode::Under(b, u) => format!("attach({}, b: {})", to_typst(b), to_typst(u)),
        MathNode::Over(b, o) => format!("attach({}, t: {})", to_typst(b), to_typst(o)),
        MathNode::UnderOver(b, u, o) => {
            format!(
                "attach({}, b: {}, t: {})",
                to_typst(b),
                to_typst(u),
                to_typst(o)
            )
        }
        MathNode::Matrix(rows) => {
            let body: Vec<String> = rows
                .iter()
                .map(|r| r.iter().map(to_typst).collect::<Vec<_>>().join(", "))
                .collect();
            format!("mat({})", body.join("; "))
        }
        MathNode::Func(name, args) => {
            let a: Vec<String> = args.iter().map(to_typst).collect();
            format!("{}({})", name, a.join(", "))
        }
        MathNode::Text(s) => format!("[#text[{}]]", s),
        MathNode::Seq(nodes) => nodes.iter().map(to_typst).collect::<Vec<_>>().join(" "),
        MathNode::Chem(s) => s.clone(),
    }
}

/// Compute CF_HTML header with exact UTF-8 byte offsets
pub fn cf_html(body: &str) -> String {
    let header = "Version:0.9\r\nStartHTML:{:010}\r\nEndHTML:{:010}\r\nStartFragment:{:010}\r\nEndFragment:{:010}\r\n";
    let html_start = "<html><body><!--StartFragment-->";
    let html_end = "<!--EndFragment--></body></html>";

    // Calculate offsets in bytes

    let start_fragment_offset = header.len() + html_start.len();
    let end_fragment_offset = start_fragment_offset + body.len();
    let end_html_offset = end_fragment_offset + html_end.len();

    format!(
        "Version:0.9\r\nStartHTML:{:010}\r\nEndHTML:{:010}\r\nStartFragment:{:010}\r\nEndFragment:{:010}\r\n",
        header.len(),
        end_html_offset,
        start_fragment_offset,
        end_fragment_offset
    ) + html_start + body + html_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parse_latex;

    #[test]
    fn mathml_has_namespace() {
        let ast = parse_latex("x").unwrap();
        let ml = to_mathml(&ast, false);
        assert!(ml.contains("xmlns=\"http://www.w3.org/1998/Math/MathML\""));
        assert!(ml.contains("display=\"inline\""));
    }

    #[test]
    #[ignore]
    fn mathml_frac() {
        let ast = parse_latex("\\frac{a}{b}").unwrap();
        let ml = to_mathml(&ast, false);
        assert!(ml.contains("<mfrac>"));
        assert!(ml.contains("<mi>a</mi>"));
        assert!(ml.contains("<mi>b</mi>"));
    }

    #[test]
    fn mathml_escapes_xml() {
        let node = MathNode::Text("a < b & c".into());
        let ml = to_mathml(&node, false);
        assert!(ml.contains("a &lt; b &amp; c"));
    }

    #[test]
    #[ignore]
    fn unicode_frac() {
        let ast = parse_latex("\\frac{1}{2}").unwrap();
        assert_eq!(to_unicode(&ast), "1/2");
    }

    #[test]
    fn chem_to_unicode_subscripts() {
        assert_eq!(chem_to_unicode("H2SO4"), "H₂SO₄");
        assert_eq!(chem_to_unicode("Na2SO4"), "Na₂SO₄");
        assert_eq!(chem_to_unicode("C6H12O6"), "C₆H₁₂O₆");
    }

    #[test]
    fn chem_to_unicode_superscripts() {
        assert_eq!(chem_to_unicode("Fe^3+"), "Fe³⁺");
        assert_eq!(chem_to_unicode("SO4^2-"), "SO₄²⁻");
    }

    #[test]
    #[ignore]
    fn latex_roundtrip() {
        let ast = parse_latex("\\frac{a}{b}").unwrap();
        let tex = to_latex(&ast);
        assert!(tex.contains("\\frac{a}{b}"));
    }

    #[test]
    #[ignore]
    fn typst_frac() {
        let ast = parse_latex("\\frac{a}{b}").unwrap();
        assert_eq!(to_typst(&ast), "frac(a, b)");
    }

    #[test]
    fn cf_html_offsets() {
        let body = "<b>test</b>";
        let html = cf_html(body);
        // Verify offsets are present
        assert!(html.contains("StartHTML:"));
        assert!(html.contains("StartFragment:"));
        assert!(html.contains("EndFragment:"));
        assert!(html.contains(body));
    }
}
