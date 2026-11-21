import io

# ============ eval.rs ============
p = 'math/src/eval.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        match t {
            Tok::Num(_) | Tok::Func(_) => out.push(t),'''
new = '''        match t {
            Tok::Num(_) => out.push(t),
            // Func day len OPS — sau RParen moi pop ra sau argument
            // (RPN dung: sqrt(16) -> [16, sqrt]).
            Tok::Func(_) => ops.push(t),'''
assert old in s, 'shunting func arm'
s = s.replace(old, new, 1)
old = '''            Tok::RParen => {
                loop {
                    let top = ops.pop()?;
                    steps += 1;
                    if steps > MAX_STEPS {
                        return None;
                    }
                    if top == Tok::LParen {
                        break;
                    }
                    out.push(top);
                }
            }'''
new = '''            Tok::RParen => {
                loop {
                    let top = ops.pop()?;
                    steps += 1;
                    if steps > MAX_STEPS {
                        return None;
                    }
                    if top == Tok::LParen {
                        break;
                    }
                    out.push(top);
                }
                // Func nam ngay tren LParen — pop ra SAU argument (sqrt(16)).
                if matches!(ops.last(), Some(Tok::Func(_))) {
                    out.push(ops.pop().unwrap());
                }
            }'''
assert old in s, 'rparen arm'
s = s.replace(old, new, 1)

old = ('    let (len, expr) = detect_balanced_expr("t'
       + chr(0x1ed5) + 'ng =2+3=").unwrap();\n'
       '        assert_eq!(expr, "2+3");\n'
       '        assert_eq!(len, " =2+3=".len());')
new = ('    let (len, expr) = detect_balanced_expr("t'
       + chr(0x1ed5) + 'ng =2+3=").unwrap();\n'
       '        assert_eq!(expr, "2+3");\n'
       '        // len = tu \'=\' mo toi \'=\' dong (ke ca hai dau).\n'
       '        assert_eq!(len, "=2+3=".len());')
assert old in s, 'detect test'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('eval fixed')

# ============ ast.rs: xml_text KHONG consume Close ============
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        Some(XTok::Close(_)) => {
            // the rong <mi></mi>
            *pos += 1;
            Ok(String::new())
        }'''
if old not in s:
    # thử với comment tiếng Việt gốc
    old = '''        Some(XTok::Close(_)) => {
            // thẻ rỗng <mi></mi>
            *pos += 1;
            Ok(String::new())
        }'''
new = '''        Some(XTok::Close(_)) => {
            // the rong <mi></mi> — KHONG consume Close (caller expect_close xu ly)
            Ok(String::new())
        }'''
assert old in s, 'xml_text close arm'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('xml_text fixed')
