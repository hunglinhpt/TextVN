// SPDX-License-Identifier: GPL-3.0-or-later
//! Word/MathType clipboard formatters: CF_UNICODETEXT, "MathML Presentation", CF_HTML

use crate::ast::MathNode;
use crate::converter::{cf_html, to_mathml, to_unicode};

/// Clipboard payload for a formula
#[derive(Debug, Clone)]
pub struct FormulaClipboard {
    pub unicode_text: String,
    pub mathml: String,
    pub html: String,
}

/// Serialize AST to all clipboard formats
pub fn serialize(node: &MathNode, display_block: bool) -> FormulaClipboard {
    let unicode = to_unicode(node);
    let mathml = to_mathml(node, display_block);
    let html = cf_html(&format!(
        "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{}</math>",
        mathml
            .trim_start_matches(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"inline\">"
            )
            .trim_start_matches(
                "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">"
            )
            .trim_end_matches("</math>")
    ));

    FormulaClipboard {
        unicode_text: unicode,
        mathml,
        html,
    }
}

/// Format name constants for Windows clipboard
pub const CF_UNICODETEXT_NAME: &str = "CF_UNICODETEXT";
pub const MATHML_FORMAT_NAME: &str = "MathML Presentation";
pub const CF_HTML_NAME: &str = "HTML Format";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parse_latex;

    #[test]
    fn serialize_frac() {
        let ast = parse_latex("\\frac{a}{b}").unwrap();
        let clip = serialize(&ast, false);
        assert_eq!(clip.unicode_text, "a/b");
        assert!(clip.mathml.contains("<mfrac>"));
        assert!(clip
            .mathml
            .contains("xmlns=\"http://www.w3.org/1998/Math/MathML\""));
        assert!(clip.html.contains("StartFragment:"));
    }

    #[test]
    fn serialize_block_math() {
        let ast = parse_latex("\\sum_{i=1}^{n} i").unwrap();
        let clip = serialize(&ast, true);
        assert!(clip.mathml.contains("display=\"block\""));
    }

    #[test]
    fn format_names() {
        assert_eq!(CF_UNICODETEXT_NAME, "CF_UNICODETEXT");
        assert_eq!(MATHML_FORMAT_NAME, "MathML Presentation");
        assert_eq!(CF_HTML_NAME, "HTML Format");
    }
}
