import io

BS = chr(92)
p = 'math/src/context.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')

def find(fn_name):
    return next(i for i, ln in enumerate(lines) if ('fn %s' % fn_name) in ln)

# text_span: tìm dòng feed_text trong 5 dòng sau fn
i = find('text_span_reenables_vietnamese')
j = next(k for k in range(i, i + 8) if 'feed_text("$' + BS + BS + 'text{' in lines[k])
lines[j] = lines[j].replace('Vi' + chr(0x1ec7) + 't}");', 'Vi' + chr(0x1ec7) + 't");')

# nested
i = find('nested_text_and_braces')
j = next(k for k in range(i, i + 8) if 'feed_text("$' + BS + BS + 'text{a{b}c}");' in lines[k])
lines[j] = lines[j].replace('a{b}c}");', 'a{b}c");')

# mbox + textbf
for k, ln in enumerate(lines):
    if 'feed_text("$' + BS + BS + 'mbox{abc}");' in ln:
        lines[k] = ln.replace('mbox{abc}");', 'mbox{abc");')
    if 'feed_text("$' + BS + BS + 'textbf{abc}");' in ln:
        lines[k] = ln.replace('textbf{abc}");', 'textbf{abc");')

io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('context tests fixed v2')
