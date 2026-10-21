// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-config` — load/validate `config.v1` (P0-3 §1).
//!
//! Slice 1: subset trường **engine** dùng; các trường UI (hotkeys, macros, updates, …)
//! chấp nhận khi parse (serde bỏ unknown) nhưng chưa hành xử — schema JSON đầy đủ
//! và migration để slice config riêng (P0-3 §1.3).
//!
//! Lỗi trả về **không chứa nội dung file config** (S2 — không ghi text người dùng ra log).

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

pub mod doc;
pub mod macro_text;

pub use doc::{DocError, DocKind, SettingsDoc};

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
    /// Gợi ý hoàn tất từ tiếng Việt bằng Tab (`post/suggest_vn.rs`). Cấu trúc
    /// theo P0-3 §1.1 `"suggest": { "enabled": bool, … }` — chỉ `enabled` được
    /// engine dùng; thiếu khoá = tắt (Tab giữ hành vi cũ).
    #[serde(default)]
    pub suggest: SuggestSettings,
    /// Trợ lý ngữ pháp (Phần 4 — `P4-0-MASTER-PLAN.md` §7). Mặc định **tắt
    /// hoàn toàn** (`mode: off`) theo D20: bật là lựa chọn rõ ràng của người dùng.
    #[serde(default)]
    pub assist: AssistSettings,
}

/// Phần `"suggest"` của `config.v1.json` (P0-3 §1.1). Các trường khác trong
/// spec (`provider`, `ollama`…) là tương lai — serde bỏ qua khoá lạ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SuggestSettings {
    /// Bật Tab hoàn tất từ tiếng Việt. Mặc định `false`.
    pub enabled: bool,
}

/// `assist.mode` (P4-0 §7): mức can thiệp của Trợ lý ngữ pháp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssistMode {
    /// Không kiểm tra, không hiện gì (mặc định — D20 kill-switch).
    #[default]
    Off,
    /// Chỉ gợi ý lỗi (Mode 1 — P4-0 §4.2).
    Highlight,
    /// Tự sửa lỗi objective + gợi ý lỗi mơ hồ (Mode 2 — P4-0 §4.3).
    Autocorrect,
}

impl AssistMode {
    /// Chuỗi trên wire/JSON (khớp schema).
    pub fn as_str(self) -> &'static str {
        match self {
            AssistMode::Off => "off",
            AssistMode::Highlight => "highlight",
            AssistMode::Autocorrect => "autocorrect",
        }
    }
}

/// `assist.auto_fix` (P4-0 §7): chỉ luật **objective** mới được tự sửa (D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssistAutoFix {
    /// Không bao giờ tự sửa (mặc định).
    #[default]
    Off,
    /// Chỉ tự sửa luật objective ("safe").
    Safe,
}

impl AssistAutoFix {
    pub fn as_str(self) -> &'static str {
        match self {
            AssistAutoFix::Off => "off",
            AssistAutoFix::Safe => "safe",
        }
    }
}

/// `assist.language` (P4-0 §7). v1 **chỉ** hỗ trợ `vi` (D9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssistLanguage {
    /// Tiếng Việt (mặc định).
    #[default]
    Vi,
    /// Tiếng Anh — chưa hỗ trợ ở v1 (giữ giá trị để không phá config tương lai).
    En,
    /// Tự nhận diện — chưa hỗ trợ ở v1.
    Auto,
}

impl AssistLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            AssistLanguage::Vi => "vi",
            AssistLanguage::En => "en",
            AssistLanguage::Auto => "auto",
        }
    }
}

/// `assist.display` — **điểm hiển thị** tách khỏi điểm kiểm tra (D11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AssistDisplay {
    /// Hiện gợi ý khi gặp dấu kết thúc câu (`.` `!` `?`).
    pub show_on_sentence_end: bool,
    /// Ngừng gõ bao lâu (ms) thì hiện gợi ý.
    pub idle_ms: u32,
    /// Hiện gợi ý ngay khi gõ dấu phẩy (mặc định tắt — tránh spam UI).
    pub show_on_comma: bool,
}

impl Default for AssistDisplay {
    fn default() -> Self {
        AssistDisplay {
            show_on_sentence_end: true,
            idle_ms: 700,
            show_on_comma: false,
        }
    }
}

/// `assist.trigger_keys` hợp lệ (P4-0 §4.1) — cùng danh sách với schema.
pub const ASSIST_TRIGGER_KEYS: [&str; 7] = ["space", ",", ".", "!", "?", ";", ":"];

/// `assist.trigger_keys` mặc định (P4-0 §7).
pub fn default_assist_trigger_keys() -> Vec<String> {
    vec!["space".to_string(), ",".to_string(), ".".to_string()]
}

/// `assist` — Trợ lý ngữ pháp (P4-0 §7). Mọi trường đều có mặc định an toàn:
/// thiếu khoá = tắt, không tự sửa, không hiện gì.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AssistSettings {
    /// Mức can thiệp (mặc định `off`).
    pub mode: AssistMode,
    /// Tự sửa lỗi objective (mặc định `off` — P4-0 §4.3).
    pub auto_fix: AssistAutoFix,
    /// Ngôn ngữ kiểm tra (v1 chỉ `vi`).
    pub language: AssistLanguage,
    /// Phím kiểm tra im lặng (mặc định space, `,`, `.`).
    pub trigger_keys: Vec<String>,
    /// Điểm hiển thị (D11).
    pub display: AssistDisplay,
    /// Độ dài tối đa của một câu được kiểm tra (ký tự Unicode).
    pub max_sentence_chars: u32,
    /// Bật/tắt riêng theo app (`app_id` lowercase → bool). Thiếu = dùng chung.
    pub per_app: BTreeMap<String, bool>,
}

impl Default for AssistSettings {
    fn default() -> Self {
        AssistSettings {
            mode: AssistMode::Off,
            auto_fix: AssistAutoFix::Off,
            language: AssistLanguage::Vi,
            trigger_keys: default_assist_trigger_keys(),
            display: AssistDisplay::default(),
            max_sentence_chars: 500,
            per_app: BTreeMap::new(),
        }
    }
}

impl AssistSettings {
    /// Trợ lý có đang bật ở mức nào đó không (kill-switch đọc cái này).
    pub fn is_on(&self) -> bool {
        self.mode != AssistMode::Off
    }

    /// App này có bị tắt Trợ lý ngữ pháp riêng không (`per_app`).
    pub fn is_app_disabled(&self, app_id: &str) -> bool {
        self.per_app
            .get(&app_id.to_lowercase())
            .is_some_and(|enabled| !enabled)
    }
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
            diacritic_style: DiacriticStyle::Old,
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
            suggest: SuggestSettings::default(),
            assist: AssistSettings::default(),
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
    let cfg: Config = serde_json::from_str(json).map_err(|_| ConfigError::Schema)?;
    cfg.validate_assist()?;
    Ok(cfg)
}

impl Config {
    /// Ràng buộc của khối `assist` mà serde **không** diễn đạt được (P4-0 §7).
    ///
    /// Vì sao kiểm ở đây chứ không "kệ, adapter tự bỏ qua": schema JSON hứa
    /// `enum` cho `trigger_keys`, nếu parser tha giá trị lạ thì editor báo đỏ
    /// trong khi chạy vẫn nhận — hai nguồn sự thật lệch nhau.
    ///
    /// Lỗi trả về **chỉ là kind**, không chứa nội dung config (S2).
    pub fn validate_assist(&self) -> Result<(), ConfigError> {
        let a = &self.assist;
        // Câu rỗng không kiểm tra được gì; 0 là cấu hình sai rõ ràng.
        if a.max_sentence_chars == 0 {
            return Err(ConfigError::Schema);
        }
        // Danh sách rỗng = không có điểm kiểm tra nào → `mode` khác off là vô nghĩa.
        if a.trigger_keys.is_empty() {
            return Err(ConfigError::Schema);
        }
        if a.trigger_keys
            .iter()
            .any(|key| !ASSIST_TRIGGER_KEYS.contains(&key.as_str()))
        {
            return Err(ConfigError::Schema);
        }
        if a.per_app.keys().any(|app| app.trim().is_empty()) {
            return Err(ConfigError::Schema);
        }
        Ok(())
    }
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
      "diacritic_style": "old",
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
        assert_eq!(cfg.diacritic_style, DiacriticStyle::Old);
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

    /// `suggest.enabled` (0.2.33): thiếu khoá = tắt; `true` bật; khoá lạ trong
    /// `suggest` (`provider`, `ollama` — spec P0-3) bị bỏ qua, không lỗi schema.
    #[test]
    fn suggest_defaults_off_and_parses_enabled() {
        let cfg = parse_config(r#"{"config_version":1}"#).unwrap();
        assert!(!cfg.suggest.enabled, "thiếu khoá suggest = tắt");
        let cfg = parse_config(
            r#"{"config_version":1,"suggest":{"enabled":true,"provider":"local_ngram"}}"#,
        )
        .unwrap();
        assert!(cfg.suggest.enabled, "enabled=true phải bật, khoá lạ bỏ qua");
        assert!(!Config::default().suggest.enabled);
    }

    /// `assist` (P4-0 §7): mặc định **tắt hoàn toàn**, đúng bảng spec.
    #[test]
    fn assist_defaults_match_p4_0_section_7() {
        let cfg = parse_config(r#"{"config_version":1}"#).unwrap();
        let a = &cfg.assist;
        assert_eq!(a, &Config::default().assist);
        assert_eq!(a.mode, AssistMode::Off);
        assert_eq!(a.auto_fix, AssistAutoFix::Off);
        assert_eq!(a.language, AssistLanguage::Vi);
        assert_eq!(a.trigger_keys, vec!["space", ",", "."]);
        assert!(a.display.show_on_sentence_end);
        assert_eq!(a.display.idle_ms, 700);
        assert!(!a.display.show_on_comma);
        assert_eq!(a.max_sentence_chars, 500);
        assert!(a.per_app.is_empty());
        assert!(!a.is_on(), "mặc định phải là off (D20)");
        assert!(!a.is_app_disabled("notepad"), "thiếu khoá = dùng chung");
    }

    /// Khối `assist` đầy đủ đúng ví dụ P4-0 §7 phải parse được, kể cả `per_app`.
    #[test]
    fn assist_full_block_parses() {
        let cfg = parse_config(
            r#"{"config_version":1,"assist":{
                "mode":"highlight",
                "auto_fix":"safe",
                "language":"vi",
                "trigger_keys":["space",",",".","!","?",";",":"],
                "display":{"show_on_sentence_end":false,"idle_ms":1200,"show_on_comma":true},
                "max_sentence_chars":300,
                "per_app":{"notepad":true,"code.exe":false}
            }}"#,
        )
        .unwrap();
        let a = &cfg.assist;
        assert_eq!(a.mode, AssistMode::Highlight);
        assert!(a.is_on());
        assert_eq!(a.auto_fix, AssistAutoFix::Safe);
        assert_eq!(a.language, AssistLanguage::Vi);
        assert_eq!(a.trigger_keys.len(), 7);
        assert!(!a.display.show_on_sentence_end);
        assert_eq!(a.display.idle_ms, 1200);
        assert!(a.display.show_on_comma);
        assert_eq!(a.max_sentence_chars, 300);
        assert_eq!(a.per_app.get("code.exe"), Some(&false));
        // `per_app` so khớp không phân biệt hoa thường (app_id luôn lowercase).
        assert!(a.is_app_disabled("Code.EXE"));
        assert!(!a.is_app_disabled("notepad"));
        // `autocorrect` cũng là giá trị hợp lệ.
        let cfg = parse_config(r#"{"config_version":1,"assist":{"mode":"autocorrect"}}"#).unwrap();
        assert_eq!(cfg.assist.mode, AssistMode::Autocorrect);
    }

    /// Giá trị `assist.*` sai → `Schema` (không đoán, không âm thầm sửa).
    #[test]
    fn invalid_assist_values_are_schema_errors() {
        for bad in [
            r#"{"config_version":1,"assist":{"mode":"fix_everything"}}"#,
            r#"{"config_version":1,"assist":{"auto_fix":"aggressive"}}"#,
            r#"{"config_version":1,"assist":{"language":"fr"}}"#,
            r#"{"config_version":1,"assist":{"mode":"highlight","trigger_keys":["enter"]}}"#,
            r#"{"config_version":1,"assist":{"mode":"highlight","trigger_keys":[]}}"#,
            r#"{"config_version":1,"assist":{"max_sentence_chars":0}}"#,
            r#"{"config_version":1,"assist":{"per_app":{"":true}}}"#,
            r#"{"config_version":1,"assist":{"display":{"idle_ms":"soon"}}}"#,
        ] {
            assert_eq!(parse_config(bad), Err(ConfigError::Schema), "`{bad}`");
        }
    }

    /// `default_json()` đi vòng parse → `Config` giữ nguyên (config mẫu ghi ra đĩa
    /// phải đọc lại được, kể cả khối `assist` mới).
    #[test]
    fn default_json_round_trips_with_assist() {
        let cfg = parse_config(&default_json()).unwrap();
        assert_eq!(cfg, Config::default());
        let value: serde_json::Value = serde_json::from_str(&default_json()).unwrap();
        assert_eq!(value["assist"]["mode"], "off");
        assert_eq!(value["assist"]["display"]["idle_ms"], 700);
        assert_eq!(value["assist"]["trigger_keys"][0], "space");
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

    /// Khối `assist` trong schema phải khớp **code**: đủ trường, đúng enum, đúng
    /// mặc định (P4-0 §7) — nếu lệch, editor báo đỏ chỗ user vẫn gõ được.
    #[test]
    fn schema_khop_khoi_assist() {
        let props = schema_props();
        let assist = &props["assist"];
        let aprops = assist["properties"]
            .as_object()
            .expect("assist.properties phải là object");
        let code =
            serde_json::to_value(AssistSettings::default()).expect("assist serialize ra object");
        for key in code.as_object().expect("object").keys() {
            assert!(
                aprops.contains_key(key),
                "trường `assist.{key}` có trong code nhưng thiếu trong schema"
            );
        }

        let enum_of = |value: &serde_json::Value| -> Vec<String> {
            value
                .as_array()
                .unwrap_or_else(|| panic!("phải là array: {value}"))
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        };
        assert_eq!(
            enum_of(&aprops["mode"]["enum"]),
            vec!["off", "highlight", "autocorrect"]
        );
        assert_eq!(enum_of(&aprops["auto_fix"]["enum"]), vec!["off", "safe"]);
        assert_eq!(
            enum_of(&aprops["language"]["enum"]),
            vec!["vi", "en", "auto"]
        );
        assert_eq!(
            enum_of(&aprops["trigger_keys"]["items"]["enum"]),
            ASSIST_TRIGGER_KEYS
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );

        // Mặc định: code và schema phải trùng (kể cả mặc định lồng trong object).
        assert_eq!(aprops["mode"]["default"], "off");
        assert_eq!(aprops["auto_fix"]["default"], "off");
        assert_eq!(aprops["language"]["default"], "vi");
        assert_eq!(aprops["max_sentence_chars"]["default"], 500);
        assert_eq!(aprops["trigger_keys"]["default"][0], "space");
        assert_eq!(
            aprops["trigger_keys"]["default"].as_array().map(Vec::len),
            Some(3)
        );
        let d = &aprops["display"]["properties"];
        let dflt = &aprops["display"]["default"];
        assert_eq!(
            d["show_on_sentence_end"]["default"],
            dflt["show_on_sentence_end"]
        );
        assert_eq!(d["idle_ms"]["default"], dflt["idle_ms"]);
        assert_eq!(d["show_on_comma"]["default"], dflt["show_on_comma"]);
        assert_eq!(dflt["show_on_sentence_end"], true);
        assert_eq!(dflt["idle_ms"], 700);
        assert_eq!(dflt["show_on_comma"], false);
        assert_eq!(aprops["per_app"]["default"], serde_json::json!({}));
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
