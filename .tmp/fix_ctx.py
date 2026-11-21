import io, glob
files = glob.glob('core/tests/*.rs') + glob.glob('cli/src/*.rs') + glob.glob('tools/**/*.rs', recursive=True)
total = 0
for p in files:
    s = io.open(p, encoding='utf-8').read()
    lines = s.split('\n')
    out = []
    changed = 0
    in_ctx = False
    for ln in lines:
        if 'Context {' in ln and 'pub' not in ln:
            in_ctx = True
        if in_ctx and ln.strip().startswith('hint:'):
            out.append(ln)
            indent = ln[:len(ln) - len(ln.lstrip())]
            out.append(indent + 'app_id: String::new(),')
            changed += 1
            in_ctx = False
            continue
        if in_ctx and ln.strip() == '};':
            in_ctx = False
        out.append(ln)
    if changed:
        io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
        print(p, changed)
        total += changed
print('total', total)
