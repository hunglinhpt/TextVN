import io, sys

p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()

# 1. Remove duplicated si_table! { header
dup = 'si_table! {\nsi_table! {'
if dup in s:
    s = s.replace(dup, 'si_table! {', 1)
    print('dup header fixed')

# 2. Fix single-backslash escapes inside the si region only
start = s.find('macro_rules! si_row')
end = s.find('/// Alias ASCII')
assert start > 0 and end > start, (start, end)
region = s[start:end]

# double every single backslash inside string literals of this region:
# occurrences like "\deca" "\mu " "\Omega" "\mathrm{" "\n..."
out = []
i = 0
fixed = 0
while i < len(region):
    c = region[i]
    if c == '"':
        # inside a string literal until next unescaped quote
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
                    # already a valid escape like \n \t \" \0 -> keep as is
                    out.append(ch)
                    i += 1
                else:
                    # single backslash + letter -> double it
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
region2 = ''.join(out)
s = s[:start] + region2 + s[end:]
print('single escapes fixed:', fixed)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('done')
