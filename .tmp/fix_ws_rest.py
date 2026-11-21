import io

BS = chr(92)
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
old = ('            // LaTeX: whitespace ngoai ' + BS + BS + 'text{} khong co nghia — bo qua.\n'
       '            LTok::Char(c) if c.is_whitespace() => {\n'
       '                *pos += 1;\n'
       '                continue;\n'
       '            }')
new = ('            // LaTeX: whitespace giu lai nhu Text(" ") — unicode emitter in ra\n'
       '            // dau cach quanh toan tu (`x2 + y1`); matrix cell tu bo rieng.\n'
       '            LTok::Char(c) if c.is_whitespace() => {\n'
       '                *pos += 1;\n'
       '                nodes.push(MathNode::Text(" ".to_string()));\n'
       '                continue;\n'
       '            }')
assert old in s, 'ws arm not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ws nodes kept')

# ============ converter.rs: sup_str map letters + test update ============
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

# Test unicode_subsup_chain: expectation mới — SO₄²⁻ (nested sup-sub shape)
sub4 = chr(0x2084)
sup2 = chr(0x00b2)
supminus = chr(0x207b)
old_test = ('    #[test]\n'
            '    fn unicode_subsup_chain() {\n'
            '        let node = parse_chem("SO_4^{2-}").unwrap();\n'
            '        let uni = to_unicode(&node);\n'
            "        assert_eq!(uni, \"SO" + chr(0x2084) + "^" + chr(0x00b2) + chr(0x207b) + "\");\n"
            '    }')
new_test = ('    #[test]\n'
            '    fn unicode_subsup_chain() {\n'
            '        let node = parse_chem("SO_4^{2-}").unwrap();\n'
            '        let uni = to_unicode(&node);\n'
            '        // Chem parser: nested Sup(Sub(O,4), 2-) → SO4 sau do sup 2-.\n'
            '        assert_eq!(uni, "SO' + sub4 + sup2 + supminus + '");\n'
            '    }')
assert old_test in s, 'subsup test not found'
s = s.replace(old_test, new_test, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('converter sup fixed')
