import io, sys
p = 'math/src/config.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
for i, ln in enumerate(lines[320:340], 321):
    sys.stdout.buffer.write(('%d %s' % (i, repr(ln))).encode('utf-8') + b'\n')
