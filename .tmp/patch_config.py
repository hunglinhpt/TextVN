import io

# 1. config/Cargo.toml — thêm textvn-math
p = 'config/Cargo.toml'
s = io.open(p, encoding='utf-8').read()
old = '[dependencies]' + chr(10) + 'serde.workspace = true' + chr(10) + 'serde_json.workspace = true'
new = old + chr(10) + '# P11 — cấu hình Math Engine: reuse MathConfig của crate math (G15, không copy).' + chr(10) + 'textvn-math = { path = "../math" }'
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('config Cargo ok')

# 2. config/src/lib.rs — field math + validate
p = 'config/src/lib.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    /// User Style Profile (P7 — vòng 54): dữ liệu học cá nhân (từ hay dùng +
    /// cặp sai→đúng đã chấp nhận). OPT-IN tắt mặc định (G8); cap chặt
    /// (200 từ / 50 cặp / ≤16KB — `sanitize` chạy sau parse).
    #[serde(default)]
    pub style_profile: StyleProfileSettings,
}'''
new = old[:-2] + '''
    /// P11 — Math Engine (`config.math`). Mặc định TẮT (hành vi gõ cũ giữ
    /// nguyên 100%); reuse [`textvn_math::MathConfig`] — serde default đủ cho
    /// config cũ không có section `math` (tương thích ngược P0-3).
    #[serde(default)]
    pub math: textvn_math::MathConfig,
}'''
assert old in s
s = s.replace(old, new, 1)

# validate: math section trong parse_config (sau validate_assist)
old = '''    let mut cfg: Config = serde_json::from_str(json).map_err(|_| ConfigError::Schema)?;
    cfg.validate_assist()?;'''
new = '''    let mut cfg: Config = serde_json::from_str(json).map_err(|_| ConfigError::Schema)?;
    cfg.validate_assist()?;
    cfg.validate_math()?;'''
assert old in s
s = s.replace(old, new, 1)

# validate_math method — thêm vào impl Config sau validate_assist
old = '''        if a.per_app.keys().any(|app| app.trim().is_empty()) {
            return Err(ConfigError::Schema);
        }
        Ok(())
    }
}'''
new = '''        if a.per_app.keys().any(|app| app.trim().is_empty()) {
            return Err(ConfigError::Schema);
        }
        Ok(())
    }

    /// Ràng buộc khối `math` (P11): leader 1 ký tự ASCII in được, timeout
    /// trong dải, hotkey không đè phím bảo vệ, trigger tùy chỉnh hợp lệ.
    /// Lỗi trả về chỉ là kind (S2 — không nội dung config).
    pub fn validate_math(&self) -> Result<(), ConfigError> {
        if self.math.validate().is_err() {
            return Err(ConfigError::Schema);
        }
        Ok(())
    }
}'''
assert old in s
s = s.replace(old, new, 1)

# 3. tests cho math trong config crate — thêm vào mod tests
old = '''    /// Ví dụ đầy đủ P0-3 §1.2 — chứa hotkeys/macros/emoji/… (chưa định nghĩa vẫn phải parse được).
    const FULL_EXAMPLE: &str = r#"{'''
new = '''    #[test]
    fn math_section_defaults_backward_compatible() {
        // Config cũ KHÔNG có section math → parse được, math tắt (hành vi cũ).
        let cfg = parse_config(r#"{"config_version":1}"#).unwrap();
        assert!(!cfg.math.enabled);
        assert_eq!(cfg.math.leader_key, '`');
    }

    #[test]
    fn math_section_parses_and_validates() {
        let cfg = parse_config(
            r#"{"config_version":1,"math":{"enabled":true,"leader_key":"\\\\","output_mode":"latex"}}"#,
        )
        .unwrap();
        assert!(cfg.math.enabled);
        assert_eq!(cfg.math.leader_key, '\\\\');
        assert_eq!(cfg.math.output_mode, textvn_math::MathOutputMode::Latex);
    }

    #[test]
    fn math_section_rejects_bad_leader() {
        // leader 2 byte (é) → Schema lỗi
        assert!(parse_config(
            r#"{"config_version":1,"math":{"leader_key":"é"}}"#
        )
        .is_err());
        // timeout ngoài dải → Schema lỗi
        assert!(parse_config(
            r#"{"config_version":1,"math":{"chord_timeout_ms":10}}"#
        )
        .is_err());
        // hotkey đè Ctrl+V bảo vệ → Schema lỗi
        assert!(parse_config(
            r#"{"config_version":1,"math":{"hotkeys":[{"combo":"Ctrl+V","action":"smart_paste"}]}}"#
        )
        .is_err());
    }

    /// Ví dụ đầy đủ P0-3 §1.2 — chứa hotkeys/macros/emoji/… (chưa định nghĩa vẫn phải parse được).
    const FULL_EXAMPLE: &str = r#"{'''
assert old in s
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('config lib.rs patched')
