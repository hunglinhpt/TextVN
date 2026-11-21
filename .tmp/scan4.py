import io, sys
BS = chr(92)
out = []
for p in ['math/src/lexicon.rs', 'math/src/chords.rs', 'math/src/context.rs', 'math/src/templates.rs', 'core/src/lib.rs']:
    s = io.open(p, encoding='utf-8').read()
    for i, ln in enumerate(s.split(chr(10)), 1):
        j = 0
        while True:
            j = ln.find(BS, j)
            if j < 0: break
            nxt = ln[j+1:j+2]
            if nxt in ('t', 'n', 'r') and j+2 < len(ln) and ln[j+2:j+3].islower():
                out.append('%s:%d: ...%s...' % (p, i, ln[max(0,j-15):j+25]))
            j += 1
sys.stdout.buffer.write(chr(10).join(out).encode('utf-8', 'replace'))
sys.stdout.buffer.write(('count=%d' % len(out)).encode())
