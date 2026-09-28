// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-config` — load/validate `config.v1` (P0-3 §1).
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

/// `config.macros[].when` (P0-3 §1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MacroWhen {
    #[default]
    Always,
    ViOn,
}

/// `config.macros[]` — gõ tắt (P0-3 §1.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroEntry {
    pub trigger: String,
    pub expand: String,
    #[serde(default)]
    pub when: MacroWhen,
}

/// `config.emoji[]` — gõ tắt emoji (P0-3 §1.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiEntry {
    pub trigger: String,
    pub glyph: String,
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
    pub macros: Vec<MacroEntry>,
    pub emoji: Vec<EmojiEntry>,
    /// `english_words[]` — từ tiếng Anh giữ nguyên khi Telex biến thành chuỗi trông
    /// như tiếng Việt (`text` → `tết`). **Mặc định rỗng, không bật sẵn** — `test` → `tết`
    /// là ca mơ hồ (xem `core/src/post/restore_en.rs`; mẫu: `data/stop_en.txt`).
    #[serde(default)]
    pub english_words: Vec<String>,
    /// Bật hội thoại này khi khởi động (UniKey 4.6 RC2 parity). Mặc định true.
    #[serde(default = "default_true")]
    pub show_dialog_on_startup: bool,
    /// Quick Telex (OpenKey): phụ âm đầu gõ đôi → cụm phụ âm
    /// (`cc→ch gg→gi kk→kh nn→ng qq→qu pp→ph tt→th`). Chỉ Telex/Simple Telex; mặc định tắt.
    #[serde(default)]
    pub quick_telex: bool,
}

fn default_true() -> bool {
    true
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
            macros: Vec::new(),
            emoji: Vec::new(),
            english_words: Vec::new(),
            show_dialog_on_startup: true,
            quick_telex: false,
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

/// Probe riêng: `config_version` là trường **bắt buộc** (P0-3 §1).
///
/// Vì sao không kiểm trong `Config` được: struct có `#[serde(default)]` nên serde lấp cả
/// `config_version` bằng `Default::default() = 1` — nghĩa là **mọi** JSON kể cả `{}` hay file
/// không phải config đều "hợp lệ", và `textvn config validate` báo OK nhầm.
#[derive(serde::Deserialize)]
struct VersionProbe {
    config_version: u32,
}

/// Parse + validate (P0-3 §1.1). Không OK → engine chạy default + `IME_ERR_CONFIG` (P0-2 §5).
///
/// Quy tắc: `config_version` **phải có** và phải bằng 1; các trường còn lại thì tuỳ chọn
/// (nhận giá trị mặc định) — khoá ngoài schema bị serde bỏ qua (P0-3 §1.1 ghi rõ).
pub fn parse_config(json: &str) -> Result<Config, ConfigError> {
    let probe: VersionProbe = serde_json::from_str(json).map_err(|_| ConfigError::Schema)?;
    if probe.config_version != 1 {
        return Err(ConfigError::Version);
    }
    serde_json::from_str(json).map_err(|_| ConfigError::Schema)
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
        // macros[] / emoji[] (P0-3 §1.1)
        assert_eq!(cfg.macros.len(), 1);
        assert_eq!(cfg.macros[0].trigger, "cty");
        assert_eq!(cfg.macros[0].expand, "Công ty TNHH");
        assert_eq!(cfg.macros[0].when, MacroWhen::Always);
        assert_eq!(cfg.emoji.len(), 1);
        assert_eq!(cfg.emoji[0].trigger, ":smile");
        assert_eq!(cfg.emoji[0].glyph, "😊");
    }

    #[test]
    fn macro_when_vi_on_and_default() {
        let cfg = parse_config(
            r#"{"config_version":1,"macros":[{"trigger":"vn","expand":"Việt Nam","when":"vi_on"},
                          {"trigger":"cty","expand":"Công ty"}]}"#,
        )
        .unwrap();
        assert_eq!(cfg.macros[0].when, MacroWhen::ViOn);
        assert_eq!(cfg.macros[1].when, MacroWhen::Always); // mặc định
    }

    #[test]
    fn invalid_macro_when_is_schema_error() {
        assert_eq!(
            parse_config(
                r#"{"config_version":1,"macros":[{"trigger":"x","expand":"y","when":"sometimes"}]}"#
            ),
            Err(ConfigError::Schema)
        );
        assert_eq!(
            parse_config(r#"{"config_version":1,"macros":[{"trigger":"x"}]}"#),
            Err(ConfigError::Schema)
        );
    }

    #[test]
    fn defaults_match_p0_3_table() {
        // Config rỗng vẫn phải khai báo `config_version` (bắt buộc theo P0-3 §1) —
        // mọi trường khác mới lấy mặc định.
        let cfg = parse_config(r#"{"config_version":1}"#).unwrap();
        assert_eq!(cfg, Config::default());
        assert!(cfg.enabled && cfg.free_marking && cfg.auto_restore_english && cfg.auto_capitalize);
        assert!(!cfg.allow_macro_when_vi_off);
    }

    #[test]
    fn thieu_config_version_la_loi_schema() {
        // Lỗ hổng thật đã gặp: `#[serde(default)]` lấp cả `config_version` → mọi JSON
        // (kể cả file không phải config) từng bị coi là hợp lệ.
        for bad in [
            "{}",
            r#"{"method":"telex"}"#,
            r#"{"schema":"textvn-bench.v1","measurements":[]}"#,
            r#"{"appdb_version":1,"entries":[]}"#,
            "[]",
            "null",
            "42",
        ] {
            assert_eq!(
                parse_config(bad),
                Err(ConfigError::Schema),
                "`{bad}` phải bị từ chối"
            );
        }
    }

    #[test]
    fn json_khong_phai_config_bi_bat() {
        // Bằng chứng thực tế: file baseline perf trong repo từng validate "OK" — giờ phải lỗi.
        let bench_like = r#"{"schema":"textvn-bench.v1","platform":"win","measurements":[]}"#;
        assert_eq!(parse_config(bench_like), Err(ConfigError::Schema));
    }

    #[test]
    fn invalid_method_is_schema_error() {
        assert_eq!(
            parse_config(r#"{"config_version":1,"method":"dvorak"}"#),
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
        assert_eq!(
            parse_config(r#"{"config_version":0}"#),
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

    /// Schema `schemas/config.v1.schema.json` phải khớp **code** — nếu không, editor
    /// gạch đỏ chỗ user vẫn gõ được (parser thiếu) hoặc ngược lại.
    ///
    /// Không thêm crate `jsonschema` (nặng) chỉ để kiểm: các điểm dễ lệch nhất là
    /// (1) tên trường, (2) enum, (3) trường bắt buộc — 3 thứ này kiểm tay được chính xác.
    const SCHEMA: &str = include_str!("../../schemas/config.v1.schema.json");

    fn schema_props() -> serde_json::Value {
        let v: serde_json::Value =
            serde_json::from_str(SCHEMA).expect("schema phải là JSON hợp lệ");
        v["properties"].clone()
    }

    #[test]
    fn schema_va_code_co_cung_bo_truong() {
        // 1) Mọi trường serde biết đều có trong schema (thiếu → user không được gạch đỏ).
        let cfg_json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&Config::default()).unwrap()).unwrap();
        let props = schema_props();
        for (key, _) in cfg_json.as_object().expect("config serialize ra object") {
            assert!(
                props.get(key).is_some(),
                "trường `{key}` có trong code nhưng THIẾU trong schemas/config.v1.schema.json"
            );
        }
        // 2) Mọi trường trong schema đều được parser biết (thừa → schema hứa hão).
        //    `config_version` là ngoại lệ duy nhất: nó là probe bắt buộc, không nằm trong
        //    `Config` (xem `VersionProbe`).
        let allowed_unknown = [
            // Trường UI chấp nhận khi parse (P0-3 §1.1: serde bỏ unknown) — schema ghi
            // để người dùng biết chúng tồn tại, engine chưa dùng.
            "config_version",
            "hotkeys",
            "ignore_apps",
            "app_overrides",
            "secure_fields",
            "diagnostics",
            "updates",
            "suggest",
        ];
        for (key, _) in props.as_object().expect("properties là object") {
            let known = cfg_json.get(key).is_some() || allowed_unknown.contains(&key.as_str());
            assert!(
                known,
                "trường `{key}` có trong schema nhưng parser không biết và cũng không thuộc nhóm 'chấp nhận khi parse'"
            );
        }
    }

    #[test]
    fn schema_enum_khop_ham_as_str() {
        let props = schema_props();
        let enum_of = |field: &str| -> Vec<String> {
            props[field]["enum"]
                .as_array()
                .unwrap_or_else(|| panic!("`{field}` phải có enum"))
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        };
        assert_eq!(
            enum_of("method"),
            vec!["telex", "vni", "viqr", "simple_telex"]
        );
        for m in [
            Method::Telex,
            Method::Vni,
            Method::Viqr,
            Method::SimpleTelex,
        ] {
            assert!(enum_of("method").contains(&m.as_str().to_string()));
        }
        assert_eq!(enum_of("diacritic_style"), vec!["new", "old"]);
        assert_eq!(enum_of("macro_trigger"), vec!["tab", "space"]);
        assert_eq!(
            enum_of("output_charset"),
            vec![
                "unicode_precomposed",
                "unicode_decomposed",
                "tcvn3",
                "vni_windows"
            ]
        );
    }

    #[test]
    fn schema_yeu_cau_config_version_dung_nhu_parser() {
        let v: serde_json::Value = serde_json::from_str(SCHEMA).unwrap();
        let required: Vec<&str> = v["required"]
            .as_array()
            .expect("required phải là array")
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["config_version"]);
        // và parser thực sự từ chối khi thiếu
        assert_eq!(parse_config("{}"), Err(ConfigError::Schema));
    }

    #[test]
    fn schema_ghi_dung_mac_dinh_cua_code() {
        let props = schema_props();
        let cfg = Config::default();
        let j = serde_json::to_value(&cfg).unwrap();
        for (key, expected) in [
            ("enabled", cfg.enabled),
            ("free_marking", cfg.free_marking),
            ("auto_restore_english", cfg.auto_restore_english),
            ("auto_capitalize", cfg.auto_capitalize),
            ("allow_macro_when_vi_off", cfg.allow_macro_when_vi_off),
        ] {
            let from_schema = props[key]["default"]
                .as_bool()
                .expect("default phải là bool");
            assert_eq!(
                from_schema, expected,
                "mặc định `{key}` lệch giữa schema và code"
            );
            assert_eq!(j[key], serde_json::Value::Bool(expected));
        }
        for (key, code) in [
            ("method", cfg.method.as_str()),
            (
                "macro_trigger",
                match cfg.macro_trigger {
                    MacroTrigger::Tab => "tab",
                    MacroTrigger::Space => "space",
                },
            ),
        ] {
            let d = props[key]["default"]
                .as_str()
                .expect("default phải là chuỗi");
            let c = match code {
                "simple_telex" => "telex",
                other => other,
            };
            assert_eq!(d, c, "mặc định `{key}` lệch giữa schema và code");
        }
    }
}
