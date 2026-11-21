import io, sys

BS = chr(92)
p = 'math/src/context.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
needle = "            '" + BS + BS + "' => {"
idx = next(i for i, ln in enumerate(lines) if ln == needle)
assert 'prev_backslash = true' in lines[idx + 1]
assert 'prev_dollar = false' in lines[idx + 2]
lines[idx + 2] = ('                // KHONG clear prev_dollar: `$' + BS + 'text{` phai giu pending-$\n'
                  '                // de mo InlineMath khi lenh text bat dau.')
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('backslash arm fixed at line', idx + 1)
