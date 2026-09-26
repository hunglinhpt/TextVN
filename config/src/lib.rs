// SPDX-License-Identifier: GPL-3.0-or-later
//! `vietime-config` — load/validate `config.v1` (P0-3 §1).
//!
//! Slice 1: subset trường **engine** dùng; các trường UI (hotkeys, macros, updates, …)
//! chấp nhận khi parse (serde bỏ unknown) nhưng chưa hành xử — schema JSON đầy đủ
//! và migration để slice config riêng (P0-3 §1.3).
//!
//! Lỗi trả về **không chứa nội dung file config** (S2 — không ghi text người dùng ra log).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    #[default]
    Telex,
    Vni,
    Viqr,
    SimpleTelex,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Telex => "telex",
            Method::Vni => "vni",
            Method::Viqr => "viqr",
            Method::SimpleTelex => "simple_telex",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiacriticStyle {
    /// `hoà` (mới — dấu ở âm cuối của cụm âm)
    #[default]
    New,
    /// `hòa` (cũ — dấu ở âm đầu của cụm âm)
    Old,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MacroTrigger {
    #[default]
    Tab,
    Space,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputCharset {
    #[default]
    UnicodePrecomposed,
    UnicodeDecomposed,
    Tcvn3,
    VniWindows,
}

/// Subset `config.v1.json` mà engine quan tâm (P0-3 §1.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub config_version: u32,
    pub enabled: bool,
    pub method: Method,
    pub diacritic_style: DiacriticStyle,
    pub free_marking: bool,
    pub auto_restore_english: bool,
    pub auto_capitalize: bool,
    pub macro_trigger: MacroTrigger,
    pub allow_macro_when_vi_off: bool,
    pub output_charset: OutputCharset,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            config_version: 1,
            enabled: true,
            method: Method::Telex,
            diacritic_style: DiacriticStyle::New,
            free_marking: true,
            auto_restore_english: true,
            auto_capitalize: true,
            macro_trigger: MacroTrigger::Tab,
            allow_macro_when_vi_off: false,
            output_charset: OutputCharset::UnicodePrecomposed,
        }
    }
}

/// Lỗi config — message dùng cho `ime_last_error()` chỉ chứa **kind**, không chứa nội dung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// JSON hỏng / sai kiểu trường / giá trị enum không hợp lệ
    Schema,
    /// `config_version` không phải 1 (migration ở slice config đầy đủ)
    Version,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Schema => write!(f, "config: invalid schema"),
            ConfigError::Version => write!(f, "config: unsupported config_version"),
        }
    }
}

/// Parse + validate (P0-3 §1.1). Không OK → engine chạy default + `IME_ERR_CONFIG` (P0-2 §5).
pub fn parse_config(json: &str) -> Result<Config, ConfigError> {
    let cfg: Config = serde_json::from_str(json).map_err(|_| ConfigError::Schema)?;
    if cfg.config_version != 1 {
        return Err(ConfigError::Version);
    }
    Ok(cfg)
}

/// JSON của config mặc định — dùng cho khởi tạo engine / `config init` (sau).
pub fn default_json() -> String {
    serde_json::to_string(&Config::default()).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ví dụ đầy đủ P0-3 §1.2 — chứa hotkeys/macros/emoji/… (chưa định nghĩa vẫn phải parse được).
    const FULL_EXAMPLE: &str = r#"{
      "config_version": 1,
      "enabled": true,
      "method": "telex",
      "diacritic_style": "new",
      "free_marking": true,
      "auto_restore_english": true,
      "auto_capitalize": true,
      "macro_trigger": "tab",
      "allow_macro_when_vi_off": false,
      "output_charset": "unicode_precomposed",
      "hotkeys": { "toggle_vi_en": "Ctrl+Shift+Space", "toggle_method": "Ctrl+Shift+K",
                   "restore_last": "Escape", "open_settings": "Ctrl+Shift+O" },
      "macros": [ { "trigger": "cty", "expand": "Công ty TNHH", "when": "always" } ],
      "emoji":  [ { "trigger": ":smile", "glyph": "😊" } ],
      "ignore_apps": [ { "match": "exe", "value": "code.exe", "mode": "disable_vi" } ],
      "app_overrides": { "chrome.exe": { "method": "telex" } },
      "secure_fields": "always_pass",
      "diagnostics": { "log_level": "warn", "export_repro": false },
      "updates": { "channel": "stable", "auto": true },
      "suggest": { "enabled": false, "provider": "local_ngram", "ollama": null }
    }"#;

    #[test]
    fn full_example_parses() {
        let cfg = parse_config(FULL_EXAMPLE).expect("example P0-3 §1.2 phải parse được");
        assert_eq!(cfg.method, Method::Telex);
        assert_eq!(cfg.diacritic_style, DiacriticStyle::New);
        assert_eq!(cfg.config_version, 1);
    }

    #[test]
    fn defaults_match_p0_3_table() {
        let cfg = parse_config("{}").unwrap();
        assert_eq!(cfg, Config::default());
        assert!(cfg.enabled && cfg.free_marking && cfg.auto_restore_english && cfg.auto_capitalize);
        assert!(!cfg.allow_macro_when_vi_off);
    }

    #[test]
    fn invalid_method_is_schema_error() {
        assert_eq!(
            parse_config(r#"{"method":"dvorak"}"#),
            Err(ConfigError::Schema)
        );
        assert_eq!(parse_config("{oops"), Err(ConfigError::Schema));
    }

    #[test]
    fn wrong_version_rejected() {
        assert_eq!(
            parse_config(r#"{"config_version":2}"#),
            Err(ConfigError::Version)
        );
    }

    #[test]
    fn error_messages_contain_no_config_content() {
        assert_eq!(ConfigError::Schema.to_string(), "config: invalid schema");
        assert_eq!(
            ConfigError::Version.to_string(),
            "config: unsupported config_version"
        );
    }
}
