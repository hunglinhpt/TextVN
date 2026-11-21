import io

BS = chr(92)
p = 'math/src/context.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')

def find(fn_name):
    return next(i for i, ln in enumerate(lines) if ('fn %s' % fn_name) in ln)

# text_span_reenables_vietnamese
i = find('text_span_reenables_vietnamese')
assert 'feed_text("$' + BS + BS + 'text{' in lines[i + 1], lines[i + 1]
lines[i + 1] = lines[i + 1].replace('Vi' + chr(0x1ec7) + 't}");', 'Vi' + chr(0x1ec7) + 't");')
assert lines[i + 1].rstrip().endswith('Vi' + chr(0x1ec7) + 't");'), lines[i + 1]

# nested_text_and_braces
i = find('nested_text_and_braces')
assert 'feed_text("$' + BS + BS + 'text{a{b}c}");' in lines[i + 1], lines[i + 1]
lines[i + 1] = lines[i + 1].replace('a{b}c}");', 'a{b}c");')

# text_variants: mbox + textbf
for i, ln in enumerate(lines):
    if 'feed_text("$' + BS + BS + 'mbox{abc}");' in ln:
        lines[i] = ln.replace('mbox{abc}");', 'mbox{abc");')
    if 'feed_text("$' + BS + BS + 'textbf{abc}");' in ln:
        lines[i] = ln.replace('textbf{abc}");', 'textbf{abc");')

io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('context tests fixed')
