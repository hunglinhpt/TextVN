import io

p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            while *pos < toks.len() {
                if inner.len() >= MAX_TOKENS {
                    return err("qu\\u00e1 tr\\u1ea7n node trong \\\\left", *pos);
                }
                match &toks[*pos] {
                    LTok::Command(cmd) if cmd == "right" => {'''
# build old with actual unicode
old = old.encode().decode('unicode_escape')

new = old.replace(
    'match &toks[*pos] {',
    'println!("LEFTLOOP pos={} tok={:?}", *pos, toks.get(*pos));\n                match &toks[*pos] {',
    1,
)
assert old in s, 'anchor not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('debug print added')
