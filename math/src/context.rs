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
    tail: [char; 16],
    tail_len: usize,
    esc_pending: bool,
    just_opened_dollar: bool,
    closing_block_dollar: bool,
    text_brace_depth: usize,
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
            tail: ['\0'; 16],
            tail_len: 0,
            esc_pending: false,
            just_opened_dollar: false,
            closing_block_dollar: false,
            text_brace_depth: 0,
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
        self.in_math && !self.in_text_span()
    }

    pub fn in_text_span(&self) -> bool {
        self.current() == SpanKind::TextInMath
    }

    pub fn push(&mut self, kind: SpanKind) {
        if self.depth < MAX_STACK {
            self.stack[self.depth] = kind;
            self.depth += 1;
            if matches!(
                kind,
                SpanKind::Math | SpanKind::InlineMath | SpanKind::BlockMath
            ) {
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
        for prefix in [
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
            "usize", "isize",
        ] {
            if lower == prefix {
                return true;
            }
        }
        token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
            && token.contains('_')
            && token.chars().next().is_some_and(|c| c.is_ascii_uppercase())
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn in_math(&self) -> bool {
        self.in_math
    }

    fn push_tail(&mut self, c: char) {
        if self.tail_len < self.tail.len() {
            self.tail[self.tail_len] = c;
            self.tail_len += 1;
        } else {
            self.tail.copy_within(1.., 0);
            self.tail[self.tail.len() - 1] = c;
        }
    }

    fn tail_ends_with_cmd(&self, cmd: &str) -> bool {
        let cmd_len = cmd.chars().count();
        if self.tail_len < cmd_len {
            return false;
        }
        let start = self.tail_len - cmd_len;
        let matches = self.tail[start..self.tail_len]
            .iter()
            .copied()
            .eq(cmd.chars());
        // Tránh khớp `\\text` (backslash kép là xuống dòng LaTeX).
        let unescaped = start == 0 || self.tail[start - 1] != '\\';
        matches && unescaped
    }

    fn tail_closing_brace_cmd(&self, prefix: &str) -> bool {
        // Khớp `<prefix>{<env>` ở đuôi tail khi `c == '}'`, với `<env>` chỉ gồm chữ cái/số/`*`.
        let brace_pos = self.tail[..self.tail_len].iter().rposition(|&ch| ch == '{');
        let Some(bp) = brace_pos else {
            return false;
        };
        let env = &self.tail[bp + 1..self.tail_len];
        if env.is_empty()
            || !env
                .iter()
                .all(|ch| ch.is_ascii_alphanumeric() || *ch == '*')
        {
            return false;
        }
        let pref_len = prefix.chars().count();
        if bp < pref_len {
            return false;
        }
        let start = bp - pref_len;
        let matches = self.tail[start..bp].iter().copied().eq(prefix.chars());
        let unescaped = start == 0 || self.tail[start - 1] != '\\';
        matches && unescaped
    }

    /// Feed a text character for delimiter tracking (`$...$`, `$$...$$`, `\(...\)`, `\[...\]`,
    /// `\begin{...}...\end{...}`, `\text{...}`, và `\$`/`\{`/`\}` escape).
    pub fn feed_text_char(&mut self, c: char) {
        if self.esc_pending {
            self.esc_pending = false;
            self.just_opened_dollar = false;
            self.closing_block_dollar = false;
            match c {
                '(' if !self.in_math() => self.enter_math(false),
                ')' if self.depth > 0 && self.current() == SpanKind::InlineMath => self.exit_math(),
                '[' if !self.in_math() => self.enter_math(true),
                ']' if self.depth > 0 && self.current() == SpanKind::BlockMath => self.exit_math(),
                _ => {}
            }
            self.push_tail(c);
            return;
        }

        if c == '\\' {
            self.just_opened_dollar = false;
            self.closing_block_dollar = false;
            self.esc_pending = true;
            self.push_tail(c);
            return;
        }

        if self.in_text_span() {
            if c == '{' {
                self.text_brace_depth += 1;
            } else if c == '}' {
                self.text_brace_depth = self.text_brace_depth.saturating_sub(1);
                if self.text_brace_depth == 0 {
                    self.pop();
                }
            }
            self.push_tail(c);
            return;
        }

        if c == '$' {
            if self.just_opened_dollar && self.depth > 0 && self.current() == SpanKind::InlineMath {
                self.just_opened_dollar = false;
                self.stack[self.depth - 1] = SpanKind::BlockMath;
                self.in_math = true;
            } else if self.depth > 0 && self.current() == SpanKind::BlockMath {
                if self.closing_block_dollar {
                    self.closing_block_dollar = false;
                    self.exit_math();
                } else {
                    self.closing_block_dollar = true;
                }
            } else if self.in_math() {
                self.exit_math();
            } else {
                self.enter_math(false);
                self.just_opened_dollar = true;
            }
            self.push_tail(c);
            return;
        }

        self.just_opened_dollar = false;
        self.closing_block_dollar = false;

        if c == '{'
            && self.in_math()
            && (self.tail_ends_with_cmd("\\text")
                || self.tail_ends_with_cmd("\\textrm")
                || self.tail_ends_with_cmd("\\mbox")
                || self.tail_ends_with_cmd("\\textbf"))
        {
            self.enter_text_in_math();
            self.text_brace_depth = 1;
        } else if c == '}' {
            if self.tail_closing_brace_cmd("\\begin") && !self.in_math() {
                self.enter_math(true);
            } else if self.tail_closing_brace_cmd("\\end")
                && self.depth > 0
                && self.current() == SpanKind::BlockMath
            {
                self.exit_math();
            }
        }

        self.push_tail(c);
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
        assert!(!fsm.vietnamese_suspended());
        fsm.enter_math(false);
        assert!(fsm.in_math());
        assert!(fsm.vietnamese_suspended());
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
        assert!(fsm.vietnamese_suspended());
        fsm.feed_text_char('x');
        fsm.feed_text_char('$');
        assert!(!fsm.in_math());
    }

    #[test]
    fn escaped_block_paren_and_text_spans() {
        let mut fsm = MathContextFsm::new();
        fsm.feed_text("\\$50");
        assert!(!fsm.in_math());
        fsm.feed_text("$$x^2$$");
        assert!(!fsm.in_math());
        fsm.feed_text("\\(a+b\\)");
        assert!(!fsm.in_math());
        fsm.feed_text("\\[x_1\\]");
        assert!(!fsm.in_math());
        fsm.feed_text("$x + \\text{với ");
        assert!(fsm.in_text_span());
        assert!(!fsm.vietnamese_suspended());
        fsm.feed_text("} y$");
        assert!(!fsm.in_math());
        fsm.feed_text("\\begin{equation}a_{11}\\end{equation}");
        assert!(!fsm.in_math());
    }
}
