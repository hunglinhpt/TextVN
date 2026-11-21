import io, glob
for p in ['math/src/lexicon.rs', 'core/src/lib.rs']:
    s = io.open(p, encoding='utf-8').read()
    for i, ln in enumerate(s.split('\n'), 1):
        j = 0
        while True:
            j = ln.find('"', j)
            if j < 0: break
            k = ln.find('"', j + 1)
            if k < 0: break
            inner = ln[j+1:k]
            if chr(9) in inner:
                print(p, i, repr(inner))
            j = k + 1
