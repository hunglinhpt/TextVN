import io, sys
p = 'math/src/ast.rs'
s = io.open(p, encoding='utf-8').read()
BS = chr(92)
i = s.find('qu' + chr(0x00e1) + ' tr' + chr(0x1ea7) + 'n node trong ' + BS + BS + 'left')
sys.stdout.buffer.write(repr(s[i - 260:i + 700]).encode('utf-8'))
