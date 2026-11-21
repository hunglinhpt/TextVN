import io

p = 'math/src/ast.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
# Chỉ sửa occurrence ĐẦU (parse_seq) — occurrence 2 là matrix cell (giữ skip).
count = 0
for i, ln in enumerate(lines):
    if 'LTok::Char(c) if c.is_whitespace() => {' in ln:
        count += 1
        if count == 1:
            assert 'continue;' in lines[i + 2], lines[i + 2]
            lines[i + 1] = '                // Whitespace giu lai nhu Text(" ") — unicode emitter in dau cach'
            lines[i + 1] += ' quanh toan tu.'
            lines[i + 2] = '                *pos += 1;'
            lines.insert(i + 3, '                nodes.push(MathNode::Text(" ".to_string()));')
            lines.insert(i + 4, '                continue;')
            break
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('ws arm fixed')
