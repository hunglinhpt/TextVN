import io, sys

p = 'math/src/ast.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')

# --- digit arm: chỉ auto-sub sau element Text != " " ---
i = next(i for i, ln in enumerate(lines) if '// số ngay sau element = subscript tự nhiên (H2O)' in ln)
# dòng kế tiếp: if let Some(MathNode::Text(_)) = nodes.last() {
assert 'if let Some(MathNode::Text(_)) = nodes.last() {' in lines[i + 1], lines[i + 1]
# tìm block: if let ... { pop; push Sub } else { push Number }
j = i + 1
depth = 0
end = None
for k in range(j, min(j + 30, len(lines))):
    depth += lines[k].count('{') - lines[k].count('}')
    if depth == 0 and lines[k].strip() == '}':
        end = k
        break
assert end, 'block end not found'
new_block = [
    '            // số ngay sau ELEMENT (Text != space) = subscript tu nhien (H2O)',
    '            let is_after_element = matches!(nodes.last(), Some(MathNode::Text(t)) if t != " ");',
    '            if is_after_element {',
    '                let base = nodes.pop().unwrap();',
    '                nodes.push(MathNode::Sub {',
    '                    body: Box::new(base),',
    '                    exp: Box::new(MathNode::Text(num)),',
    '                });',
    '            } else {',
    '                nodes.push(MathNode::Number(num));',
    '            }',
]
lines[i:end + 1] = new_block
print('digit arm fixed at', i + 1)

# --- charge arm: '+'/'-' sau element -> Sup, else Operator ---
i = next(i for i, ln in enumerate(lines) if "} else if c == '+' || c == '-' || c ==" in ln and 'Sup' in ln or "} else if c == '+' || c == '-' || c ==" in ln)
# xác định đúng arm có Sup
while True:
    blk = '\n'.join(lines[i:i + 20])
    if 'MathNode::Sup' in blk:
        break
    i = next(k for k in enumerate([]) ) if False else next((k for k in range(i + 1, len(lines)) if "} else if c == '+' || c == '-' || c ==" in lines[k]))
# tìm hết block
depth = 0
end = None
for k in range(i, min(i + 30, len(lines))):
    depth += lines[k].count('{') - lines[k].count('}')
    if depth == 0 and lines[k].strip().rstrip(';') == '}':
        end = k
        break
assert end, 'charge block end not found'
minus = chr(0x2212)
dotc = chr(0x00b7)
new_block = [
    "        } else if c == '+' || c == '-' || c == '" + minus + "' {",
    '            *pos += 1;',
    '            let ch = c.to_string();',
    '            // Charge SAU ELEMENT (Text != space) -> superscript ion (Na+, Fe3-).',
    '            let is_after_element = matches!(nodes.last(), Some(MathNode::Text(t)) if t != " ");',
    '            if is_after_element {',
    '                let base = nodes.pop().unwrap();',
    '                nodes.push(MathNode::Sup {',
    '                    body: Box::new(base),',
    '                    exp: Box::new(MathNode::Text(ch)),',
    '                });',
    '            } else {',
    '                nodes.push(MathNode::Operator(ch));',
    '            }',
    "        } else if c == '" + dotc + "' || c == '.' {",
]
lines[i:end + 1] = new_block
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('charge arm fixed at', i + 1)
