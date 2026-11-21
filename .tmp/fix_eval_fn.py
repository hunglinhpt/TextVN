import io

# ============ eval.rs: Func vào OPS stack + RParen pop Func sau arg ============
p = 'math/src/eval.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        match t {
            Tok::Num(_) | Tok::Func(_) => out.push(t),'''
new = '''        match t {
            Tok::Num(_) => out.push(t),
            // Func đẩy lên OPS — sau RParen mới pop ra sau argument
            // (RPN đúng: sqrt(16) → [16, sqrt]).
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
                // Func nằm ngay trên LParen — pop ra SAU argument (sqrt(16)).
                if matches!(ops.last(), Some(Tok::Func(_))) {
                    out.push(ops.pop().unwrap());
                }
            }'''
assert old in s, 'rparen arm'
s = s.replace(old, new, 1)

# detect test: len = "=2+3=" (5)
old = '''        let (len, expr) = detect_balanced_expr("t\\u1ed5ng =2+3=").unwrap();
        assert_eq!(expr, "2+3");
        assert_eq!(len, " =2+3=".len());'''
old = ('    let (len, expr) = detect_balanced_expr("t'
       + chr(0x1ed5) + 'ng =2+3=").unwrap();\n'
       '        assert_eq!(expr, "2+3");\n'
       '        assert_eq!(len, " =2+3=".len());')
new = ('    let (len, expr) = detect_balanced_expr("t'
       + chr(0x1ed5) + 'ng =2+3=").unwrap();\n'
       '        assert_eq!(expr, "2+3");\n'
       '        // len = từ \'=\' mở tới \'=\' đóng (kể cả hai dấu).\n'
       '        assert_eq!(len, "=2+3=".len());')
assert old in s, 'detect test'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('eval fixed')

# ============ ast.rs: xml_text KHÔNG consume Close ============
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        Some(XTok::Close(_)) => {
            // thẻ rỗng <mi></mi>
            *pos += 1;
            Ok(String::new())
        }'''
new = '''        Some(XTok::Close(_)) => {
            // thẻ rỗng <mi></mi> — KHÔNG consume Close (caller expect_close xử lý)
            Ok(String::new())
        }'''
assert old in s, 'xml_text close arm'
s = s.replace(old, new, 1)

# chem: auto sub/sup CHỈ sau element Text (không phải space/operator)
old = '''            // số ngay sau element = subscript tự nhiên (H2O)
            if let Some(MathNode::Text(_)) = nodes.last() {'''
new = '''            // số ngay sau ELEMENT (Text ≠ space) = subscript tự nhiên (H2O)
            if let Some(MathNode::Text(t)) = nodes.last() {
                if t != " " {
                    let base = nodes.pop().unwrap();
                    nodes.push(MathNode::Sub {
                        body: Box::new(base),
                        exp: Box::new(MathNode::Text(num)),
                    });
                    continue;
                }
            }
            if false {
                let base = nodes.pop().unwrap();
                nodes.push(MathNode::Sub {
                    body: Box::new(base),
                    exp: Box::new(MathNode::Text(num)),
                });
            }
            if false {'''
assert old in s, 'chem digit arm'
s = s.replace(old, new, 1)
# dọn nhánh dead if false — rewrite arm hoàn toàn
old = '''            if false {
                let base = nodes.pop().unwrap();
                nodes.push(MathNode::Sub {
                    body: Box::new(base),
                    exp: Box::new(MathNode::Text(num)),
                });
            }
            if false {
                let base = nodes.pop().unwrap();
                nodes.push(MathNode::Sub {
                    body: Box::new(base),
                    exp: Box::new(MathNode::Text(num)),
                });
            } else {
                nodes.push(MathNode::Number(num));
            }'''
new = '''            nodes.push(MathNode::Number(num));'''
if old in s:
    s = s.replace(old, new, 1)
    print('chem digit arm cleaned')
else:
    # fallback: tìm pattern thực tế
    i = s.find('if false {')
    print('WARN fallback needed, if false at', i)

# '+'/'-' sau element → Sup; sau space/operator → Operator
old = '''        } else if c == '+' || c == '-' || c == chr(0x2212) {
            *pos += 1;
            let ch = c.to_string();
            if let Some(MathNode::Text(_)) = nodes.last() {
                let base = nodes.pop().unwrap();
                nodes.push(MathNode::Sup {
                    body: Box::new(base),
                    exp: Box::new(MathNode::Text(ch)),
                });
            } else {
                nodes.push(MathNode::Operator(ch));
            }
        } else if c == chr(0x00b7) || c == '.' {'''
old = old.replace('chr(0x2212)', "'" + chr(0x2212) + "'")
old = old.replace('chr(0x00b7)', "'" + chr(0x00b7) + "'")
new = '''        } else if c == '+' || c == '-' || c == ''' + "'" + chr(0x2212) + """' {
            *pos += 1;
            let ch = c.to_string();
            // Charge SAU ELEMENT (Text ≠ space) → superscript ion (Na+, Fe3-).
            let is_after_element = matches!(nodes.last(), Some(MathNode::Text(t)) if t != " ");
            if is_after_element {
                let base = nodes.pop().unwrap();
                nodes.push(MathNode::Sup {
                    body: Box::new(base),
                    exp: Box::new(MathNode::Text(ch)),
                });
            } else {
                nodes.push(MathNode::Operator(ch));
            }
        } else if c == ''' + "'" + chr(0x00b7) + """' || c == '.' {"""
assert old in s, 'chem charge arm'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('chem arms fixed')
