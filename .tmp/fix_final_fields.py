import io, glob

# 1. EngineStep literals (tests) — thêm cursor_left: 0
p = 'adapters/windows-tsf/src/compose.rs'
s = io.open(p, encoding='utf-8').read()
lines = s.split('\n')
out = []
in_step = False
n = 0
for ln in lines:
    if 'EngineStep {' in ln and 'pub' not in ln:
        in_step = True
    if in_step and ln.strip().startswith('flags:'):
        out.append(ln)
        indent = ln[:len(ln) - len(ln.lstrip())]
        out.append(indent + 'cursor_left: 0,')
        n += 1
        in_step = False
        continue
    out.append(ln)
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
print('compose EngineStep literals', n)

# 2. Outcome literals trong ffi tests + core tests
count = 0
for p in glob.glob('ffi/src/*.rs') + glob.glob('ffi/tests/*.rs') + glob.glob('core/tests/*.rs') + glob.glob('tray/src/*.rs') + glob.glob('assist/src/*.rs'):
    s = io.open(p, encoding='utf-8').read()
    if 'Outcome {' not in s:
        continue
    lines = s.split('\n')
    out = []
    in_o = False
    ch = 0
    for ln in lines:
        if re_chk := ('Outcome {' in ln and 'pub' not in ln and 'fn ' not in ln):
            in_o = True
        if in_o and ln.strip().startswith('flags:'):
            out.append(ln)
            indent = ln[:len(ln) - len(ln.lstrip())]
            out.append(indent + 'cursor_left: 0,')
            ch += 1
            in_o = False
            continue
        out.append(ln)
    if ch:
        io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
        print(p, ch)
        count += ch
print('Outcome literals', count)

# 3. ffi offset_of test: _reserved -> cursor_left
p = 'ffi/src/lib.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('assert_eq!(offset_of!(ime_result_v1, _reserved), 14);', 'assert_eq!(offset_of!(ime_result_v1, cursor_left), 14);')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ffi offset_of fixed')

# 4. dual_mode_rapid Context còn sót — my earlier fix_ctx missed multi-line contexts where hint not directly after
p = 'core/tests/dual_mode_rapid.rs'
s = io.open(p, encoding='utf-8').read()
old = 'let ctx = Context {\n                    enabled: true,\n'
if old in s:
    s = s.replace(old, old + '                    app_id: String::new(),\n')
    print('ctx 516 fixed')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
