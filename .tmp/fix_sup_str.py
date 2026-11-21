import io

p = 'math/src/converter.rs'
s = io.open(p, encoding='utf-8').read()
old = ('fn sup_str(s: &str) -> String {\n'
       '    s.chars().map(sup_digit).collect()\n'
       '}')
sup_i = chr(0x2071)
sup_n = chr(0x207f)
sup_plus = chr(0x207a)
sup_minus = chr(0x207b)
minus_sign = chr(0x2212)
new = ('fn sup_str(s: &str) -> String {\n'
       '    s.chars()\n'
       '        .map(|c| match c {\n'
       "            'i' => '" + sup_i + "',\n"
       "            'n' => '" + sup_n + "',\n"
       "            'x' => '" + chr(0x02e3) + "',\n"
       "            't' => '" + chr(0x1d57) + "',\n"
       "            '+' => '" + sup_plus + "',\n"
       "            '-' | '" + minus_sign + "' => '" + sup_minus + "',\n"
       '            other => sup_digit(other),\n'
       '        })\n'
       '        .collect()\n'
       '}')
assert old in s, 'sup_str not found'
s = s.replace(old, new, 1)

# Test unicode_subsup_chain: nested Sup(Sub(O,4), 2-) → SO₄²⁻
sub4 = chr(0x2084)
sup2 = chr(0x00b2)
supminus = chr(0x207b)
caret = '^'
old_test = ('    fn unicode_subsup_chain() {\n'
            '        let node = parse_chem("SO_4^{2-}").unwrap();\n'
            '        let uni = to_unicode(&node);\n'
            '        assert_eq!(uni, "SO' + sub4 + caret + sup2 + supminus + '");\n'
            '    }')
new_test = ('    fn unicode_subsup_chain() {\n'
            '        let node = parse_chem("SO_4^{2-}").unwrap();\n'
            '        let uni = to_unicode(&node);\n'
            '        // Chem parser: nested Sup(Sub(O,4), 2-) -> SO4 roi sup 2-.\n'
            '        assert_eq!(uni, "SO' + sub4 + sup2 + supminus + '");\n'
            '    }')
assert old_test in s, 'subsup test not found'
s = s.replace(old_test, new_test, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('sup_str + subsup test fixed')

# ============ ast tests: filter ws Text(" ") — update indices ====
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    #[test]
    fn latex_greek_symbols() {
        let n = tex("\\\\alpha + \\\\beta");
        match n {
            MathNode::Row(items) => {
                assert_eq!(items[0], MathNode::Operator("α".into()));
                assert_eq!(items[2], MathNode::Operator("β".into()));
            }
            other => panic!("got {other:?}"),
        }
    }'''
BS = chr(92)
old = ('''    #[test]
    fn latex_greek_symbols() {
        let n = tex("''' + BS + BS + '''alpha + ''' + BS + BS + '''beta");
        match n {
            MathNode::Row(items) => {
                assert_eq!(items[0], MathNode::Operator("α".into()));
                assert_eq!(items[2], MathNode::Operator("β".into()));
            }
            other => panic!("got {other:?}"),
        }
    }''')
new = '''    /// Nodes có nghĩa (bỏ Text(" ") whitespace — LaTeX semantics).
    fn atoms(n: &MathNode) -> Vec<&MathNode> {
        match n {
            MathNode::Row(items) => {
                items.iter().filter(|x| **x != MathNode::Text(" ".into())).collect()
            }
            other => vec![other],
        }
    }

    #[test]
    fn latex_greek_symbols() {
        let n = tex("\\\\alpha + \\\\beta");
        let a = atoms(&n);
        assert_eq!(a[0], &MathNode::Operator("α".into()));
        assert_eq!(a[2], &MathNode::Operator("β".into()));
    }'''
assert old in s, 'greek test not found'
s = s.replace(old, new, 1)

old = '''    #[test]
    fn latex_numbers_coalesce() {
        let n = tex("12.5 + 3");
        match n {
            MathNode::Row(items) => {
                assert_eq!(items[0], MathNode::Number("12.5".into()));
                assert_eq!(items[2], MathNode::Number("3".into()));
            }
            other => panic!("got {other:?}"),
        }
    }'''
new = '''    #[test]
    fn latex_numbers_coalesce() {
        let n = tex("12.5 + 3");
        let a = atoms(&n);
        assert_eq!(a[0], &MathNode::Number("12.5".into()));
        assert_eq!(a[2], &MathNode::Number("3".into()));
    }'''
assert old in s, 'coalesce test not found'
s = s.replace(old, new, 1)

old = '''        let n = tex("x^2 + y_1");
        match n {
            MathNode::Row(items) => {
                assert!(matches!(items[0], MathNode::Sup { .. }));
                assert!(matches!(items[2], MathNode::Sub { .. }));
            }
            other => panic!("got {other:?}"),
        }
    }'''
new = '''        let n = tex("x^2 + y_1");
        let a = atoms(&n);
        assert!(matches!(a[0], MathNode::Sup { .. }));
        assert!(matches!(a[2], MathNode::Sub { .. }));
    }'''
assert old in s, 'supsub test not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ast tests updated')

# ============ serializer tests: marker-aware offsets ====
p = 'math/src/clipboard/serializer.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        assert_eq!(&bytes[ef - frag.len()..ef], frag.as_bytes());'''
new = '''        assert_eq!(&bytes[sf + 20..ef - 20], frag.as_bytes());'''
assert old in s, 'frag body assert'
s = s.replace(old, new, 1)
old = '''        let got = &bytes[cf.start_fragment + 19..cf.end_fragment];
        assert_eq!(got, frag.as_bytes());'''
new = '''        let got = &bytes[cf.start_fragment + 20..cf.end_fragment - 20];
        assert_eq!(got, frag.as_bytes());'''
assert old in s, 'multibyte got assert'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('serializer tests fixed')
