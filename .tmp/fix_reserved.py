import io, glob
# ime_result_v1 literals only — ime_key_v1 keeps _reserved (u8). Distinguish:
# result literal contains 'ime_result_v1 {' marker line above within 3 lines.
files = set(glob.glob('adapters/*/src/*.rs') + glob.glob('ffi/tests/*.rs') + glob.glob('tools/*/src/*.rs') + glob.glob('tray/src/*.rs') + glob.glob('assist/src/*.rs') + glob.glob('cli/src/*.rs'))
total = 0
for p in files:
    lines = io.open(p, encoding='utf-8').read().split('\n')
    out = []
    cur_struct = None
    depth = 0
    changed = 0
    for ln in lines:
        if 'ime_result_v1 {' in ln:
            cur_struct = 'result'
            depth = 0
        elif 'ime_key_v1 {' in ln:
            cur_struct = 'key'
            depth = 0
        if cur_struct:
            depth += ln.count('{') - ln.count('}')
            if '_reserved: 0,' in ln and cur_struct == 'result':
                out.append(ln.replace('_reserved: 0,', 'cursor_left: 0,'))
                changed += 1
                continue
            if depth <= 0 and '}' in ln:
                cur_struct = None
        out.append(ln)
    if changed:
        io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
        print(p, changed)
        total += changed
print('total', total)
