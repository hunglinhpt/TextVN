import io

BS = chr(92)
p = 'math/src/config.rs'
s = io.open(p, encoding='utf-8').read()
old = ('''        for (k, _) in &self.custom_keybindings {
            if !valid_trigger_token(k) {
                return Err(ConfigError::InvalidTrigger);
            }
            let key = k.strip_prefix('''' + BS + BS + '''').unwrap_or(k);''')
new = ('''        for (k, _) in &self.custom_keybindings {
            // Chấp nhận cả dạng có/không backslash đầu — so khớp ở dạng stripped.
            let key = k.strip_prefix('''' + BS + BS + '''').unwrap_or(k);
            if !valid_trigger_token(key) {
                return Err(ConfigError::InvalidTrigger);
            }''')
assert old in s, 'validator block still missing'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('validator fixed')
