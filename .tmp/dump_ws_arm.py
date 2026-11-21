import io, sys
p = 'math/src/ast.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
for i, ln in enumerate(lines):
    if 'is_whitespace() =>' in ln and 'LTok::Char' in ln:
        for k in range(max(0, i - 3), min(len(lines), i + 5)):
            sys.stdout.buffer.write(('%d %s' % (k + 1, repr(lines[k]))).encode('utf-8') + b'\n')
        sys.stdout.buffer.write(b'---\n')
