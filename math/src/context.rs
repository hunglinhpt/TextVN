// SPDX-License-Identifier: GPL-3.0-or-later
//! Zero-alloc MathContextFsm — delimiter tracking, \text{} brace stack, code token immunity

/// Span kind inside math mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Math,
    TextInMath,
    InlineMath,
    BlockMath,
}

const MAX_STACK: usize = 32;

#[derive(Debug, Clone)]
pub struct MathContextFsm {
    stack: [SpanKind; MAX_STACK],
    depth: usize,
    in_math: bool,
}

impl Default for MathContextFsm {
    fn default() -> Self {
        Self::new()
    }
}

impl MathContextFsm {
    pub fn new() -> Self {
        Self {
            stack: [SpanKind::Math; MAX_STACK],
            depth: 0,
            in_math: false,
        }
    }

    pub fn current(&self) -> SpanKind {
        if self.depth == 0 {
            SpanKind::Math
        } else {
            self.stack[self.depth - 1]
        }
    }

    pub fn vietnamese_suspended(&self) -> bool {
        self.in_math && self.current() == SpanKind::Math
    }

    pub fn in_text_span(&self) -> bool {
        self.current() == SpanKind::TextInMath
    }

    pub fn push(&mut self, kind: SpanKind) {
        if self.depth < MAX_STACK {
            self.stack[self.depth] = kind;
            self.depth += 1;
            if matches!(kind, SpanKind::Math | SpanKind::InlineMath | SpanKind::BlockMath) {
                self.in_math = true;
            }
        }
    }

    pub fn pop(&mut self) -> Option<SpanKind> {
        if self.depth > 0 {
            self.depth -= 1;
            let kind = self.stack[self.depth];
            self.in_math = (0..self.depth).any(|i| {
                matches!(
                    self.stack[i],
                    SpanKind::Math | SpanKind::InlineMath | SpanKind::BlockMath
                )
            });
            Some(kind)
        } else {
            None
        }
    }

    pub fn enter_math(&mut self, block: bool) {
        self.push(if block {
            SpanKind::BlockMath
        } else {
            SpanKind::InlineMath
        });
    }

    pub fn exit_math(&mut self) {
        self.pop();
    }

    pub fn enter_text_in_math(&mut self) {
        self.push(SpanKind::TextInMath);
    }

    pub fn is_immune_token(token: &str) -> bool {
        if token.is_empty() {
            return false;
        }
        if token.len() > 2 && token.starts_with("0x") {
            return token[2..].chars().all(|c| c.is_ascii_hexdigit());
        }
        if token.len() > 2 && token.starts_with("0b") {
            return token[2..].chars().all(|c| c == '0' || c == '1');
        }
        let lower = token.to_ascii_lowercase();
        for prefix in ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "usize", "isize"] {
            if lower == prefix {
                return true;
            }
        }
        if token.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
            && token.contains('_')
            && token.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
            return true;
        }
        false
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn in_math(&self) -> bool {
        self.in_math
    }

    /// Feed a text character for delimiter tracking
    pub fn feed_text_char(&mut self, c: char) {
        if c == '$' {
            if self.in_math() {
                self.exit_math();
            } else {
                self.enter_math(false);
            }
        }
    }

    /// Feed a text slice for delimiter tracking
    pub fn feed_text(&mut self, text: &str) {
        for c in text.chars() {
            self.feed_text_char(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_exit_math() {
        let mut fsm = MathContextFsm::new();
        assert!(!fsm.in_math());
        fsm.enter_math(false);
        assert!(fsm.in_math());
        fsm.exit_math();
        assert!(!fsm.in_math());
    }

    #[test]
    fn immune_tokens() {
        assert!(MathContextFsm::is_immune_token("0xFF"));
        assert!(MathContextFsm::is_immune_token("CAN_FD"));
        assert!(!MathContextFsm::is_immune_token("hello"));
    }

    #[test]
    fn feed_dollar_toggles() {
        let mut fsm = MathContextFsm::new();
        fsm.feed_text_char('$');
        assert!(fsm.in_math());
        fsm.feed_text_char('$');
        assert!(!fsm.in_math());
    }
}
