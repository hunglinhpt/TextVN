import io

p = 'math/src/chords.rs'
s = io.open(p, encoding='utf-8').read()
BS = chr(92)
old = ('        let latex = expand_lexicon_word("' + BS + BS + 'alpha", MathOutputMode::Latex, |_| None)\n'
       '            .unwrap()\n'
       '            .inserted_text();\n'
       '        assert_eq!(latex, "' + BS + BS + 'alpha");\n'
       '        let typst = expand_lexicon_word("' + BS + BS + 'alpha", MathOutputMode::Typst, |_| None)\n'
       '            .unwrap()\n'
       '            .inserted_text();\n'
       '        assert_eq!(typst, "alpha");')
new = ('        let action_l = expand_lexicon_word("' + BS + BS + 'alpha", MathOutputMode::Latex, |_| None).unwrap();\n'
       '        assert_eq!(action_l.inserted_text(), "' + BS + BS + 'alpha");\n'
       '        let action_t = expand_lexicon_word("' + BS + BS + 'alpha", MathOutputMode::Typst, |_| None).unwrap();\n'
       '        assert_eq!(action_t.inserted_text(), "alpha");')
assert old in s, 'chords E0716 pattern not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('chords fixed')

p = 'math/src/lexicon.rs'
s = io.open(p, encoding='utf-8').read()
old = 'std::ptr::eq(owner as *const _, entry as *const _)'
new = 'std::ptr::eq(owner as *const MathEntry, entry as *const MathEntry)'
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('lexicon fixed')
