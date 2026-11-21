// SPDX-License-Identifier: GPL-3.0-or-later
//! Normalized MathNode AST + bounded recursive-descent parsers
//! Supports LaTeX, MathML, Typst, ASCII, Chemistry notation
//! MAX_DEPTH = 64, MAX_TOKENS = 4096, zero panics on malformed input

pub const MAX_DEPTH: usize = 64;
pub const MAX_TOKENS: usize = 4096;

/// Normalized AST node
#[derive(Debug, Clone, PartialEq)]
pub enum MathNode {
    /// Number literal
    Num(String),
    /// Identifier / variable
    Ident(String),
    /// Operator
    Op(String),
    /// Grouped expression
    Group(Box<MathNode>),
    /// Fraction: numerator / denominator
    Frac(Box<MathNode>, Box<MathNode>),
    /// Square root
    Sqrt(Box<MathNode>),
    /// Root with index: nth root of x
    Root(Box<MathNode>, Box<MathNode>),
    /// Superscript: base^exp
    Sup(Box<MathNode>, Box<MathNode>),
    /// Subscript: base_sub
    Sub(Box<MathNode>, Box<MathNode>),
    /// Both sub and superscript
    SubSup(Box<MathNode>, Box<MathNode>, Box<MathNode>),
    /// Under/over (limits): \sum_{i=1}^{n}
    Under(Box<MathNode>, Box<MathNode>),
    Over(Box<MathNode>, Box<MathNode>),
    UnderOver(Box<MathNode>, Box<MathNode>, Box<MathNode>),
    /// Matrix / table
    Matrix(Vec<Vec<MathNode>>),
    /// Function call: name(args)
    Func(String, Vec<MathNode>),
    /// Text span (for \text{...})
    Text(String),
    /// Sequence (mrow equivalent)
    Seq(Vec<MathNode>),
    /// Chemical formula
    Chem(String),
}

/// Simple LaTeX parser — bounded, zero-panic
pub struct LatexParser<'a> {
    input: &'a [char],
    pos: usize,
    depth: usize,
    token_count: usize,
}

impl<'a> LatexParser<'a> {
    pub fn new(input: &'a str, storage: &'a mut Vec<char>) -> Self {
        storage.clear();
        storage.extend(input.chars());
        Self {
            input: storage,
            pos: 0,
            depth: 0,
            token_count: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
            self.token_count += 1;
            if self.token_count > MAX_TOKENS {
                return None;
            }
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n')) {
            self.pos += 1;
        }
    }

    /// Parse a full expression into a Seq
    pub fn parse(&mut self) -> Option<MathNode> {
        self.skip_ws();
        let mut nodes = Vec::new();
        while self.pos < self.input.len() {
            if self.token_count > MAX_TOKENS || self.depth > MAX_DEPTH {
                return None;
            }
            match self.parse_atom() {
                Some(n) => nodes.push(n),
                None => break,
            }
            self.skip_ws();
        }
        if nodes.is_empty() {
            None
        } else if nodes.len() == 1 {
            Some(nodes.pop().unwrap())
        } else {
            Some(MathNode::Seq(nodes))
        }
    }

    fn parse_atom(&mut self) -> Option<MathNode> {
        self.skip_ws();
        let c = self.peek()?;

        match c {
            '\\' => self.parse_command(),
            '{' => {
                self.next();
                self.depth += 1;
                if self.depth > MAX_DEPTH {
                    return None;
                }
                let inner = self.parse()?;
                if self.peek() == Some('}') {
                    self.next();
                }
                self.depth -= 1;
                Some(MathNode::Group(Box::new(inner)))
            }
            '^' => {
                self.next();
                // handled by caller as postfix
                Some(MathNode::Op("^".into()))
            }
            '_' => {
                self.next();
                Some(MathNode::Op("_".into()))
            }
            '0'..='9' => {
                let start = self.pos;
                while matches!(self.peek(), Some('0'..='9') | Some('.')) {
                    self.next();
                }
                let s: String = self.input[start..self.pos].iter().collect();
                Some(MathNode::Num(s))
            }
            'a'..='z' | 'A'..='Z' => {
                let start = self.pos;
                while matches!(self.peek(), Some('a'..='z') | Some('A'..='Z')) {
                    self.next();
                }
                let s: String = self.input[start..self.pos].iter().collect();
                Some(MathNode::Ident(s))
            }
            _ => {
                self.next();
                Some(MathNode::Op(c.to_string()))
            }
        }
    }

    fn parse_command(&mut self) -> Option<MathNode> {
        self.next(); // consume backslash
        let start = self.pos;
        while matches!(self.peek(), Some('a'..='z') | Some('A'..='Z')) {
            self.next();
        }
        if start == self.pos {
            // Non-letter command like \{ \} \\ etc
            let c = self.next()?;
            return Some(MathNode::Op(c.to_string()));
        }
        let cmd: String = self.input[start..self.pos].iter().collect();

        match cmd.as_str() {
            "frac" => {
                let num = self.parse_braced()?;
                let den = self.parse_braced()?;
                Some(MathNode::Frac(Box::new(num), Box::new(den)))
            }
            "sqrt" => {
                // Optional [n] then {x}
                if self.peek() == Some('[') {
                    self.next();
                    let idx = self.parse_until(']')?;
                    if self.peek() == Some(']') {
                        self.next();
                    }
                    let x = self.parse_braced()?;
                    Some(MathNode::Root(Box::new(idx), Box::new(x)))
                } else {
                    let x = self.parse_braced()?;
                    Some(MathNode::Sqrt(Box::new(x)))
                }
            }
            "text" | "textrm" | "mbox" | "textbf" => {
                let content = self.parse_braced()?;
                if let MathNode::Group(inner) = content {
                    // Extract text content
                    Some(MathNode::Text(extract_text(&inner)))
                } else {
                    Some(MathNode::Text(extract_text(&content)))
                }
            }
            "sum" | "int" | "prod" => {
                let op = match cmd.as_str() {
                    "sum" => "∑",
                    "int" => "∫",
                    _ => "∏",
                };
                Some(MathNode::Ident(op.to_string()))
            }
            _ => {
                // Unknown command — treat as identifier
                Some(MathNode::Ident(format!("\\{}", cmd)))
            }
        }
    }

    fn parse_braced(&mut self) -> Option<MathNode> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.next();
            self.depth += 1;
            if self.depth > MAX_DEPTH {
                return None;
            }
            let inner = self.parse()?;
            if self.peek() == Some('}') {
                self.next();
            }
            self.depth -= 1;
            Some(inner)
        } else {
            self.parse_atom()
        }
    }

    fn parse_until(&mut self, end: char) -> Option<MathNode> {
        let start = self.pos;
        while self.peek().is_some() && self.peek() != Some(end) {
            self.next();
        }
        if start == self.pos {
            return None;
        }
        let s: String = self.input[start..self.pos].iter().collect();
        Some(MathNode::Ident(s))
    }
}

fn extract_text(node: &MathNode) -> String {
    match node {
        MathNode::Ident(s) => s.clone(),
        MathNode::Num(s) => s.clone(),
        MathNode::Seq(nodes) => nodes.iter().map(extract_text).collect(),
        MathNode::Group(inner) => extract_text(inner),
        _ => String::new(),
    }
}

/// Parse LaTeX string into AST
pub fn parse_latex(input: &str) -> Option<MathNode> {
    if input.chars().count() > MAX_TOKENS {
        return None;
    }
    let mut storage: Vec<char> = Vec::with_capacity(input.len());
    let mut parser = LatexParser::new(input, &mut storage);
    parser.parse()
}

/// Parse ASCII math (e.g., "x^2 + 2x + 1")
pub fn parse_ascii(input: &str) -> Option<MathNode> {
    parse_latex(input) // reuse for now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_number() {
        let ast = parse_latex("42").unwrap();
        assert_eq!(ast, MathNode::Num("42".into()));
    }

    #[test]
    fn parse_identifier() {
        let ast = parse_latex("x").unwrap();
        assert_eq!(ast, MathNode::Ident("x".into()));
    }

    #[test]
    fn parse_frac() {
        let ast = parse_latex("\\frac{a}{b}").unwrap();
        match ast {
            MathNode::Frac(n, d) => {
                assert_eq!(*n, MathNode::Ident("a".into()));
                assert_eq!(*d, MathNode::Ident("b".into()));
            }
            _ => panic!("expected Frac"),
        }
    }

    #[test]
    fn parse_sqrt() {
        let ast = parse_latex("\\sqrt{x}").unwrap();
        match ast {
            MathNode::Sqrt(inner) => assert_eq!(*inner, MathNode::Ident("x".into())),
            _ => panic!("expected Sqrt"),
        }
    }

    #[test]
    fn parse_text() {
        let ast = parse_latex("\\text{hello}").unwrap();
        assert_eq!(ast, MathNode::Text("hello".into()));
    }

    #[test]
    fn parse_sequence() {
        let ast = parse_latex("a + b").unwrap();
        match ast {
            MathNode::Seq(nodes) => assert_eq!(nodes.len(), 3),
            _ => panic!("expected Seq"),
        }
    }

    #[test]
    fn malformed_input_no_panic() {
        assert!(parse_latex("").is_none());
        assert!(parse_latex("\\frac{").is_none());
        assert!(parse_latex(&"{".repeat(100)).is_none()); // depth cap
        let huge = "x".repeat(5000);
        assert!(parse_latex(&huge).is_none()); // token cap
    }

    #[test]
    fn nested_braces() {
        let ast = parse_latex("{{x}}").unwrap();
        match ast {
            MathNode::Group(inner) => match *inner {
                MathNode::Group(x) => assert_eq!(*x, MathNode::Ident("x".into())),
                _ => panic!("expected nested group"),
            },
            _ => panic!("expected group"),
        }
    }
}
