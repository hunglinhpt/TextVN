import io, sys

BS = chr(92)
p = 'math/src/context.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
# Tìm dòng "'\\' => {" (match arm backslash)
idx = None
for i, ln in enumerate(lines):
    if ln.strip() == "'" + BS + "' => {":
        idx = i
        break
if idx is None:
    sys.stdout.buffer.write(b'not found; dump arms:\n')
    for i, ln in enumerate(lines):
        if "prev_backslash = true" in ln:
            sys.stdout.buffer.write(('%d %s' % (i + 1, repr(lines[i - 1]))).encode() + b'\n')
            sys.stdout.buffer.write(('%d %s' % (i + 1, repr(ln))).encode() + b'\n')
            sys.stdout.buffer.write(('%d %s' % (i + 2, repr(lines[i + 1]))).encode() + b'\n')
    raise SystemExit(1)
# lines[idx] là "'\\' => {" — idx+1 prev_backslash=true, idx+2 prev_dollar=false
assert 'prev_backslash = true' in lines[idx + 1]
assert 'prev_dollar = false' in lines[idx + 2]
lines[idx + 2] = ('                // KHONG clear prev_dollar: `$' + BS + 'text{` phai giu pending-$\n'
                  '                // de mo InlineMath khi lenh text bat dau.')
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('backslash arm fixed at line', idx + 1)
