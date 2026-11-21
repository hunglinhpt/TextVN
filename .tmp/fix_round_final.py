import io

BS = chr(92)

# ============ context.rs: $ toggle + escaped $ + nested brace depth ============
p = 'math/src/context.rs'
s = io.open(p, encoding='utf-8').read()

# 1. escaped: $ literal KHÔNG arm prev_dollar
old = '''        if escaped_next {
            self.prev_backslash = false;
            // `\\{`, `\\}`, `\\$` là literal — KHÔNG đổi ngữ cảnh.
            // `\\[`, `\\]`, `\\(`, `\\)` là delimiter thật → tiếp tục xử lý thường.
            if c == '{' || c == '}' || c == '$' {
                self.prev_dollar = c == '$';
                return;
            }
        }'''
new = '''        if escaped_next {
            self.prev_backslash = false;
            // `\\{`, `\\}`, `\\$` là literal — KHÔNG đổi ngữ cảnh, KHÔNG arm prev_dollar.
            // `\\[`, `\\]`, `\\(`, `\\)` là delimiter thật → tiếp tục xử lý thường.
            if c == '{' || c == '}' || c == '$' {
                self.prev_dollar = false;
                return;
            }
        }'''
assert old in s, 'escaped block'
s = s.replace(old, new, 1)

# 2. $ toggle: đóng inline khi đang trong inline
old = '''            '$' => {
                if self.prev_dollar {
                    // `$$` — toggle block math
                    self.prev_dollar = false;
                    if self.in_math && self.current() == SpanKind::BlockMath {
                        self.pop();
                    } else if self.in_math && self.current() == SpanKind::InlineMath {
                        // `$a$$b$` lệch cú pháp — đóng inline trước rồi mở block.
                        self.pop();
                        self.push(SpanKind::BlockMath);
                    } else {
                        self.push(SpanKind::BlockMath);
                    }
                } else {
                    self.prev_dollar = true;
                    // Chưa đóng: `$` có thể là mở inline (đợi ký tự kế).
                }
                return;
            }'''
new = '''            '$' => {
                if self.prev_dollar {
                    // `$$` — toggle block math
                    self.prev_dollar = false;
                    if self.in_math && self.current() == SpanKind::BlockMath {
                        self.pop();
                    } else if self.in_math && self.current() == SpanKind::InlineMath {
                        // `$a$$b$` lệch cú pháp — đóng inline trước rồi mở block.
                        self.pop();
                        self.push(SpanKind::BlockMath);
                    } else {
                        self.push(SpanKind::BlockMath);
                    }
                } else if self.in_math && self.current() == SpanKind::InlineMath {
                    // `$` thứ hai của $...$ — ĐÓNG inline (không mở span mới).
                    self.pop();
                } else {
                    self.prev_dollar = true;
                    // Chưa đóng: `$` có thể là mở inline (đợi ký tự kế).
                }
                return;
            }'''
assert old in s, 'dollar block'
s = s.replace(old, new, 1)

# 3. nested brace depth: '{' tăng depth, '}' chỉ pop TextInMath khi depth == 0
old = '''        if c == '{' {
            // `{` thường trong math — tăng brace depth (không đổi span).
            self.prev_dollar = false;
            return;
        }
        if c == '}' {
            self.prev_dollar = false;
            // `}` đóng `\\text{...}`: span TextInMath nằm trên stack.
            if self.depth > 0 && self.current() == SpanKind::TextInMath {
                self.pop();
            }
            return;
        }'''
new = '''        if c == '{' {
            // `{` thường trong math — tăng brace depth (không đổi span).
            self.brace_depth_text += 1;
            self.prev_dollar = false;
            return;
        }
        if c == '}' {
            self.prev_dollar = false;
            // `}`: brace lồng trước → đóng brace đó; không brace lồng → đóng \\text{}.
            if self.brace_depth_text > 0 {
                self.brace_depth_text -= 1;
            } else if self.depth > 0 && self.current() == SpanKind::TextInMath {
                self.pop();
            }
            return;
        }'''
assert old in s, 'brace block'
s = s.replace(old, new, 1)

# 4. Fix test text_variants: bên trong \textrm{ tiếng Việt BẬT lại → NOT suspended
old = '''    #[test]
    fn text_variants() {
        assert!(suspended_after("$x \\\\textrm{"));
        let mut fsm = MathContextFsm::new();
        fsm.feed_text("$\\\\mbox{abc}");
        assert!(fsm.in_text_span());
        let mut fsm = MathContextFsm::new();
        fsm.feed_text("$\\\\textbf{abc}");
        assert!(fsm.in_text_span());
    }'''
# heredoc collapse safe: build programmatically
old = ('    #[test]\n'
       '    fn text_variants() {\n'
       '        assert!(suspended_after("$x ' + BS + BS + 'textrm{"));\n'
       '        let mut fsm = MathContextFsm::new();\n'
       '        fsm.feed_text("$' + BS + BS + 'mbox{abc}");\n'
       '        assert!(fsm.in_text_span());\n'
       '        let mut fsm = MathContextFsm::new();\n'
       '        fsm.feed_text("$' + BS + BS + 'textbf{abc}");\n'
       '        assert!(fsm.in_text_span());\n'
       '    }')
new = ('    #[test]\n'
       '    fn text_variants() {\n'
       '        // Trong ' + BS + BS + 'textrm{ tiếng Việt BẬT lại → KHÔNG suspended.\n'
       '        assert!(!suspended_after("$x ' + BS + BS + 'textrm{"));\n'
       '        let mut fsm = MathContextFsm::new();\n'
       '        fsm.feed_text("$' + BS + BS + 'mbox{abc}");\n'
       '        assert!(fsm.in_text_span());\n'
       '        let mut fsm = MathContextFsm::new();\n'
       '        fsm.feed_text("$' + BS + BS + 'textbf{abc}");\n'
       '        assert!(fsm.in_text_span());\n'
       '    }')
assert old in s, 'text_variants test'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('context fixed')

# ============ config.rs: validator strip backslash trước valid check ============
p = 'math/src/config.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        for (k, _) in &self.custom_keybindings {
            if !valid_trigger_token(k) {
                return Err(ConfigError::InvalidTrigger);
            }
            let key = k.strip_prefix('\\\\').unwrap_or(k);'''
# build without heredoc-collapse issues
old = ('''        for (k, _) in &self.custom_keybindings {
            if !valid_trigger_token(k) {
                return Err(ConfigError::InvalidTrigger);
            }
            let key = k.strip_prefix('''' + BS + '''').unwrap_or(k);''')
new = ('''        for (k, _) in &self.custom_keybindings {
            // Chấp nhận cả dạng có/không `\\` đầu — so sánh ở dạng stripped.
            let key = k.strip_prefix('''' + BS + '''').unwrap_or(k);
            if !valid_trigger_token(key) {
                return Err(ConfigError::InvalidTrigger);
            }''')
assert old in s, 'validator block'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('config validator fixed')
