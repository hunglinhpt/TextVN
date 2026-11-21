import io

BS = chr(92)
CRLF = chr(92) + 'r' + chr(92) + 'n'

# ============ ast.rs: matrix cell bỏ whitespace + chem giữ space ============
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            _ => {
                let before = *pos;
                if let Some(n) = parse_one(toks, pos, st)? {
                    cur_cell.push(n);
                }
                if *pos == before {
                    // parse_one không advance — tự advance để chống treo.
                    *pos += 1;
                }
            }'''
new = '''            LTok::Char(c) if c.is_whitespace() => {
                // Whitespace quanh cell ma trận: bỏ qua (không thành Text cell).
                *pos += 1;
            }
            _ => {
                let before = *pos;
                if let Some(n) = parse_one(toks, pos, st)? {
                    cur_cell.push(n);
                }
                if *pos == before {
                    // parse_one không advance — tự advance để chống treo.
                    *pos += 1;
                }
            }'''
assert old in s, 'matrix ws arm'
s = s.replace(old, new, 1)

# chem: whitespace → Text(" ") giữ khoảng trắng công thức phản ứng
old = '''        } else if c.is_whitespace() {
            *pos += 1;
        } else {
            *pos += 1;
            nodes.push(MathNode::Operator(c.to_string()));
        }
    }
    st.depth -= 1;
    Ok(())
}'''
new = '''        } else if c.is_whitespace() {
            // Giữ khoảng trắng giữa các chất trong phản ứng (H₂SO₄ + 2NaOH → …).
            *pos += 1;
            nodes.push(MathNode::Text(" ".to_string()));
        } else {
            *pos += 1;
            nodes.push(MathNode::Operator(c.to_string()));
        }
    }
    st.depth -= 1;
    Ok(())
}'''
assert old in s, 'chem ws arm'
s = s.replace(old, new, 1)

# coalesce_numbers không được gộp qua Text(" ") — hiện chỉ gộp Number Number ✓ OK.
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ast ws fixed')

# ============ converter.rs: mtd emitter ============
p = 'math/src/converter.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        MathNode::Matrix { env, rows } => {
            let trs: String = rows
                .iter()
                .map(|row| {
                    let tds: String = row.iter().map(mathml_body).collect();
                    format!("<mtr>{}</mtr>", tds)
                })
                .collect();'''
new = '''        MathNode::Matrix { env, rows } => {
            let trs: String = rows
                .iter()
                .map(|row| {
                    let tds: String = row
                        .iter()
                        .map(|cell| format!("<mtd>{}</mtd>", mathml_body(cell)))
                        .collect();
                    format!("<mtr>{}</mtr>", tds)
                })
                .collect();'''
assert old in s, 'mtd emitter'
s = s.replace(old, new, 1)

# test wellformed: count đúng (opens = "<m" matches; closes = "</m")
old = '''            let open_count = mml.matches("<m").count() + mml.matches("<mo").count()
                + mml.matches("<mi").count() + mml.matches("<mn").count();
            let close_count = mml.matches("</m").count();'''
new = '''            // Mọi tag MathML bắt đầu "m" (math/mi/mn/mo/mfrac/…) — "</m" không
            // đụng "<m" vì có dấu "/".
            let open_count = mml.matches("<m").count();
            let close_count = mml.matches("</m").count();'''
assert old in s, 'wellformed count'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('converter fixed')

# ============ serializer.rs tests: marker dài 20 ============
p = 'math/src/clipboard/serializer.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('assert_eq!(&bytes[sf..sf + 19], b"<!--StartFragment-->");',
              'assert_eq!(&bytes[sf..sf + 20], b"<!--StartFragment-->");')
s = s.replace('assert_eq!(&bytes[sf..sf + 19], b"<!--StartFragment-->");',
              'assert_eq!(&bytes[cf.start_fragment..cf.start_fragment + 20], b"<!--StartFragment-->");')
s = s.replace('assert_eq!(&bytes[ef..ef + 19], b"<!--EndFragment-->");',
              'assert_eq!(&bytes[ef..ef + 20], b"<!--EndFragment-->");')
s = s.replace('''        let got = &bytes[cf.start_fragment + 19..cf.end_fragment];
        assert_eq!(got, frag.as_bytes());''',
              '''        let got = &bytes[cf.start_fragment + 20..cf.end_fragment - 20];
        assert_eq!(got, frag.as_bytes());''')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('serializer tests fixed')

# ============ context.rs tests: intent đúng (feed không có '}' cuối) ============
p = 'math/src/context.rs'
s = io.open(p, encoding='utf-8').read()
old = ('''    #[test]
    fn text_span_reenables_vietnamese() {
        let mut fsm = MathContextFsm::new();
        fsm.feed_text("$''' + BS + BS + '''text{ti' + chr(0x1ebf) + 'ng Vi' + chr(0x1ec7) + 't}");
        assert!(fsm.in_math(), "vẫn trong $...$");
        assert!(fsm.in_text_span());
        assert!(!fsm.vietnamese_suspended(), "text-span bat lai tieng Viet");
        fsm.feed('}');
        assert!(fsm.vietnamese_suspended(), "dong text-span treo lai");
        fsm.feed('$');
        assert!(!fsm.in_math());
    }''')
new = ('''    #[test]
    fn text_span_reenables_vietnamese() {
        let mut fsm = MathContextFsm::new();
        // Feed CHƯA đóng \\text{} — kiểm trạng thái BÊN TRONG.
        fsm.feed_text("$''' + BS + BS + '''text{ti' + chr(0x1ebf) + 'ng Vi' + chr(0x1ec7) + 't");
        assert!(fsm.in_math(), "van trong dollar math");
        assert!(fsm.in_text_span());
        assert!(!fsm.vietnamese_suspended(), "text-span bat lai tieng Viet");
        fsm.feed('}');
        assert!(fsm.vietnamese_suspended(), "dong text-span treo lai");
        fsm.feed('$');
        assert!(!fsm.in_math());
    }''')
assert old in s, 'text_span test'
s = s.replace(old, new, 1)

old = ('''    #[test]
    fn nested_text_and_braces() {
        let mut fsm = MathContextFsm::new();
        fsm.feed_text("$''' + BS + BS + '''text{a{b}c}");
        assert!(fsm.in_text_span(), "brace long khong dong som text-span");
        fsm.feed('}');
        assert!(fsm.vietnamese_suspended());
    }''')
new = ('''    #[test]
    fn nested_text_and_braces() {
        let mut fsm = MathContextFsm::new();
        // Feed tới giữa (brace lồng {b} đã đóng, \\text{} vẫn mở).
        fsm.feed_text("$''' + BS + BS + '''text{a{b}c");
        assert!(fsm.in_text_span(), "brace long khong dong som text-span");
        fsm.feed('}');
        assert!(fsm.vietnamese_suspended());
    }''')
assert old in s, 'nested test'
s = s.replace(old, new, 1)

old = ('''        fsm.feed_text("$''' + BS + BS + '''mbox{abc}");
        assert!(fsm.in_text_span());''')
new = ('''        fsm.feed_text("$''' + BS + BS + '''mbox{abc");
        assert!(fsm.in_text_span());''')
assert old in s, 'mbox test'
s = s.replace(old, new, 1)

old = ('''        fsm.feed_text("$''' + BS + BS + '''textbf{abc}");
        assert!(fsm.in_text_span());''')
new = ('''        fsm.feed_text("$''' + BS + BS + '''textbf{abc");
        assert!(fsm.in_text_span());''')
assert old in s, 'textbf test'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('context tests fixed')
