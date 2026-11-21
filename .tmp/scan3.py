import io
s = io.open('math/src/lexicon.rs', encoding='utf-8').read()
BS = chr(92)
lines = s.split(chr(10))
for i, ln in enumerate(lines, 1):
    if '"'+BS+'tera"' in ln or '"'+BS+'t"' in ln or BS+'textrm' in ln or BS+'text{' in ln:
        print(i, ln.strip()[:120])
