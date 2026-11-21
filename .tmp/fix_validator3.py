import io

p = 'math/src/config.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')
# Tìm index dòng chứa 'for (k, _) in &self.custom_keybindings'
idx = next(i for i, ln in enumerate(lines) if 'for (k, _) in &self.custom_keybindings' in ln)
BS = chr(92)
# Thay 5 dòng (idx..idx+4) bằng phiên bản strip-trước-valid
new_block = [
    '        for (k, _) in &self.custom_keybindings {',
    '            // Chấp nhận cả dạng có/không backslash đầu — so khớp ở dạng stripped.',
    "            let key = k.strip_prefix('" + BS + BS + "').unwrap_or(k);",
    '            if !valid_trigger_token(key) {',
    '                return Err(ConfigError::InvalidTrigger);',
    '            }',
]
assert lines[idx + 1].strip().startswith('if !valid_trigger_token'), lines[idx + 1]
assert 'strip_prefix' in lines[idx + 4], lines[idx + 4]
lines[idx:idx + 5] = new_block
io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
print('validator fixed by index')
