import io, sys

p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()

# Dùng anchor ASCII thuần để tránh lỗi encoding
old = '''            let mut inner: Vec<MathNode> = Vec::new();
            let mut close = ")".to_string();
            while *pos < toks.len() {'''
new = '''            let mut inner: Vec<MathNode> = Vec::new();
            let mut close = ")".to_string();
            println!("LEFTARM start pos={}", *pos);
            while *pos < toks.len() {
                println!("LEFTLOOP pos={} tok={:?}", *pos, toks.get(*pos));'''
assert old in s, 'anchor missing'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ok')
