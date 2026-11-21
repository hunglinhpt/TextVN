import io

# context.rs fixes
p = 'math/src/context.rs'
s = io.open(p, encoding='utf-8').read()

BS = chr(92)
textb = BS + BS + 'text{}'

# 1. format-string braces must be doubled
s = s.replace('assert!(fsm.in_math(), "vẫn trong $...$");', 'assert!(fsm.in_math(), "van trong dollar math");')
s = s.replace('assert!(!fsm.vietnamese_suspended(), "' + textb + ' bật lại tiếng Việt");',
              'assert!(!fsm.vietnamese_suspended(), "text-span bat lai tieng Viet");')
s = s.replace('assert!(fsm.vietnamese_suspended(), "đóng ' + textb + ' → treo lại");',
              'assert!(fsm.vietnamese_suspended(), "dong text-span treo lai");')
s = s.replace('assert!(fsm.in_text_span(), "brace lồng không đóng sớm ' + textb + '");',
              'assert!(fsm.in_text_span(), "brace long khong dong som text-span");')

# 2. TEXT_CMDS iteration: destructure by value
s = s.replace('''        for (pat, kind) in TEXT_CMDS {
            if self.suffix_is(pat.0) {
                self.push(kind);''',
              '''        for &(pat, kind) in TEXT_CMDS {
            if self.suffix_is(pat) {
                self.push(kind);''')

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('context fixed')

# lexicon.rs: unused import SUPSUB in supsub_tests + ALL_ENTRIES flattening for SI
p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('use super::{lookup_exact, SUPSUB};', 'use super::{lookup_exact};')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('lexicon supsub import fixed')

# templates.rs: unused kind param in cell()
p = 'math/src/templates.rs'
s = io.open(p, encoding='utf-8').read()
s = s.replace('fn cell(kind: TemplateKind, i: usize, j: usize, mode: MathOutputMode) -> String {',
              'fn cell(_kind: TemplateKind, i: usize, j: usize, mode: MathOutputMode) -> String {')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('templates fixed')
