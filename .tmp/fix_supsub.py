import io

p = 'math/src/clipboard/classifier.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    const SUPSUB: |char| bool = |c: char| {
        matches!(c, '\\u{2070}'..='\\u{209f}' | '\\u{00b9}' | '\\u{00b2}' | '\\u{00b3}' | '\\u{2071}' | '\\u{fb00}'..='\\u{fb06}') || c == 'ⁿ' || c == 'ⁱ' || c == 'ˣ' || c == 'ᵗ'
    };
    let supsub_hits = s.chars().filter(SUPSUB).count();'''

# Build the replacement with actual unicode chars (file is UTF-8)
sup_chars = ''.join(chr(c) for c in range(0x2070, 0x2080))  # ⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾ⁿ
sub_chars = ''.join(chr(c) for c in range(0x2080, 0x2090))  # ₀₁₂₃₄₅₆₇₈₉
extra = '¹²³ⁿⁱˣᵗ'

new = ("    const SUPSUB_CHARS: &str = \"" + sup_chars + sub_chars + extra + "\";\n"
       "    let supsub_hits = s.chars().filter(|c| SUPSUB_CHARS.contains(*c)).count();")

assert old in s, 'SUPSUB block not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('SUPSUB fixed')
