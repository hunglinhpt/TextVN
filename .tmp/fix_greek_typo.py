import io
p='math/src/chords.rs'
s=io.open(p,encoding='utf-8').read()
pairs = [
    ('\u03c8', '\u03c8', 'psi'),
]
# dùng trực tiếp ký tự thật
psi = chr(0x3c8)
lam = chr(0x39b)
Psi = chr(0x3a8)
s = s.replace('("' + psi + '", "\\' + psi + '")', '("' + psi + '", "\\psi")')
s = s.replace('"' + psi + '", "\\' + psi + '"', '"' + psi + '", "\\psi"')
s = s.replace('"\\' + lam + 'ambda"', '"\\Lambda"')
s = s.replace('"\\' + Psi + '"', '"\\Psi"')
io.open(p,'w',encoding='utf-8',newline='\n').write(s)
print('fixed greek typos')
