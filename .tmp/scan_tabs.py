import io, glob
for p in glob.glob('math/src/**/*.rs', recursive=True):
    s = io.open(p, encoding='utf-8').read()
    for i, ln in enumerate(s.split('\n'), 1):
        # chuỗi có tab literal bên trong (giữa hai dấu ")
        q = ln.split('"')
        for k in range(1, len(q), 2):
            if '\t' in q[k] or '\n' in q[k]:
                print(p, i, repr(q[k]))
