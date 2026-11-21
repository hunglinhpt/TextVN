import io

p = 'math/src/converter.rs'
s = io.open(p, encoding='utf-8').read()

# 1. sub_str letters
old = ('fn sub_str(s: &str) -> String {\n'
       '    s.chars().map(sub_digit).collect()\n'
       '}')
sub_map = {
    'a': chr(0x2090), 'e': chr(0x2091), 'h': chr(0x2095), 'i': chr(0x1d62),
    'j': chr(0x2c7c), 'k': chr(0x2096), 'l': chr(0x2097), 'm': chr(0x2098),
    'n': chr(0x2099), 'o': chr(0x2092), 'p': chr(0x209a), 'r': chr(0x1d63),
    's': chr(0x209b), 't': chr(0x209c), 'u': chr(0x1d64), 'v': chr(0x1d65),
    'x': chr(0x2093),
}
lines = ['fn sub_str(s: &str) -> String {',
         '    s.chars()',
         '        .map(|c| match c {']
for k, v in sub_map.items():
    lines.append("            '" + k + "' => '" + v + "',")
lines.append('            other => sub_digit(other),')
lines.append('        })')
lines.append('        .collect()')
lines.append('}')
new = '\n'.join(lines)
assert old in s, 'sub_str not found'
s = s.replace(old, new, 1)

# 2. sup_str thêm letters
sup_add = {
    'a': chr(0x1d43), 'b': chr(0x1d47), 'd': chr(0x1d48), 'e': chr(0x1d49),
    'o': chr(0x1d52), 'p': chr(0x1d56), 'r': chr(0x02b3), 's': chr(0x02e2),
    'v': chr(0x1d5b), 'm': chr(0x1d50), 'k': chr(0x1d4f), 'h': chr(0x02b0),
    'l': chr(0x02e1), 'g': chr(0x1d4a),
}
old = ("            'i' => '" + chr(0x2071) + "',")
lines_ins = []
for k, v in sup_add.items():
    lines_ins.append("            '" + k + "' => '" + v + "',")
assert old in s, 'sup_str i arm not found'
s = s.replace(old, '\n'.join(lines_ins) + '\n' + old, 1)

# 3. Sup{body: Sub} merge — không in dấu ^ giữa sub và sup
old = '''        MathNode::Sup { body, exp } => {
            unicode_body(body, out);
            out.push_str(&sup_str(&to_unicode(exp)));
        }'''
new = '''        MathNode::Sup { body, exp } => {
            // Sup sau Sub (x_a^b): gộp thành xₐᵇ — không in dấu ^ giữa.
            if let MathNode::Sub {
                body: base2,
                exp: exp2,
            } = body.as_ref()
            {
                unicode_body(base2, out);
                out.push_str(&sub_str(&to_unicode(exp2)));
                out.push_str(&sup_str(&to_unicode(exp)));
            } else {
                unicode_body(body, out);
                out.push_str(&sup_str(&to_unicode(exp)));
            }
        }'''
assert old in s, 'sup branch not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('unicode maps + merge fixed')
