import io

BS = chr(92)
p = 'math/src/context.rs'
s = io.open(p, encoding='utf-8').read()

# '\\' arm: KHÔNG clear prev_dollar ($ và \ độc lập nhau)
old = '''            '\\\\' => {
                self.prev_backslash = true;
                self.prev_dollar = false;
                return;
            }'''
# build with actual backslash char
old = ("            '" + BS + "' => {\n"
       "                self.prev_backslash = true;\n"
       "                self.prev_dollar = false;\n"
       "                return;\n"
       "            }")
new = ("            '" + BS + "' => {\n"
       "                self.prev_backslash = true;\n"
       "                // KHÔNG clear prev_dollar: `$\\text{` phải giữ pending-$\n"
       "                // để mở InlineMath khi lệnh text bắt đầu.\n"
       "                return;\n"
       "            }")
assert old in s, 'backslash arm not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('context backslash fixed')

# ============ ast.rs: whitespace trong parse_seq → Text(" ") ============
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
old = '''            // LaTeX: whitespace ngoài \\text{} không có nghĩa — bỏ qua.
            LTok::Char(c) if c.is_whitespace() => {
                *pos += 1;
                continue;
            }'''
old = old.replace(chr(92) + 'text{}', BS + BS + 'text{}')
new = '''            // LaTeX: whitespace giữ lại như Text(" ") — unicode emitter in ra
            // dấu cách quanh toán tử (`x² + y₁`); matrix cell tự bỏ riêng.
            LTok::Char(c) if c.is_whitespace() => {
                *pos += 1;
                nodes.push(MathNode::Text(" ".to_string()));
                continue;
            }'''
assert old in s, 'ws arm not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ws nodes kept')

# ============ converter.rs: sup_str map letters + tests update ============
p = 'math/src/converter.rs'
s = io.open(p, encoding='utf-8').read()
old = '''fn sup_str(s: &str) -> String {
    s.chars().map(sup_digit).collect()
}'''
new = '''fn sup_str(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'i' => '\\u{2071}',
            'n' => '\\u{207f}',
            'x' => 'ˣ',
            't' => 'ᵗ',
            '+' => '\\u{207a}',
            '-' | '\\u{2212}' => '\\u{207b}',
            other => sup_digit(other),
        })
        .collect()
}'''
# heredoc collapse-safe: build with chr()
old = ('fn sup_str(s: &str) -> String {\n'
       '    s.chars().map(sup_digit).collect()\n'
       '}')
sup_i = chr(0x2071)
sup_n = chr(0x207f)
sup_plus = chr(0x207a)
sup_minus = chr(0x207b)
minus_sign = chr(0x2212)
sup_x = 'ˣ'
sup_t = 'ᵗ'
new = ('fn sup_str(s: &str) -> String {\n'
       '    s.chars()\n'
       '        .map(|c| match c {\n'
       "            'i' => '" + sup_i + "',\n"
       "            'n' => '" + sup_n + "',\n"
       "            'x' => '" + sup_x + "',\n"
       "            't' => '" + sup_t + "',\n"
       "            '+' => '" + sup_plus + "',\n"
       "            '-' | '" + minus_sign + "' => '" + sup_minus + "',\n"
       '            other => sup_digit(other),\n'
       '        })\n'
       '        .collect()\n'
       '}')
assert old in s, 'sup_str not found'
s = s.replace(old, new, 1)

# Test updates
s = s.replace('''    #[test]
    fn unicode_subsup_chain() {
        let node = parse_chem("SO_4^{2-}").unwrap();
        let uni = to_unicode(&node);
        assert_eq!(uni, "SO₄^²⁻");
    }''', '''    #[test]
    fn unicode_subsup_chain() {
        let node = parse_chem("SO_4^{2-}").unwrap();
        let uni = to_unicode(&node);
        assert_eq!(uni, "SO" + chr(0x2084) + chr(0x00b2) + chr(0x207b));''')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('converter sup fixed')
