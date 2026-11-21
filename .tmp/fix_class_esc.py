import io

p = 'math/src/clipboard/classifier.rs'
s = io.open(p, encoding='utf-8').read()

start = s.find('fn math_signals')
end = s.find('fn has_chem_pattern')
assert 0 < start < end
region = s[start:end]

out = []
i = 0
fixed = 0
while i < len(region):
    c = region[i]
    if c == '"':
        out.append(c)
        i += 1
        while i < len(region):
            ch = region[i]
            if ch == '\\' and i + 1 < len(region):
                nxt = region[i + 1]
                if nxt == '\\':
                    out.append('\\\\')
                    i += 2
                elif nxt in ('n', 't', '0', '"'):
                    # escape hợp lệ như \n \t \" — GIỮ NGUYÊN (trừ trường hợp
                    # \frac...: \t + chữ thường = lệnh LaTeX thật!)
                    if nxt in ('t', 'n', 'r') and i + 2 < len(region) and region[i + 2].islower():
                        # \frac \sqrt \sum... — phải là \\frac
                        out.append('\\\\')
                        i += 1
                        fixed += 1
                    else:
                        out.append(ch)
                        i += 1
                else:
                    out.append('\\\\')
                    i += 1
                    fixed += 1
            elif ch == '"':
                out.append(ch)
                i += 1
                break
            else:
                out.append(ch)
                i += 1
    else:
        out.append(c)
        i += 1

s = s[:start] + ''.join(out) + s[end:]
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('classifier escapes fixed:', fixed)
