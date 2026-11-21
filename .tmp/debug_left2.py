import io, sys
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
i = s.find('while *pos < toks.len() {')
j = s.find('match &toks[*pos] {', i)
sys.stdout.buffer.write(repr(s[i:j + 60]).encode('utf-8'))
