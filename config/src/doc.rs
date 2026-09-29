// SPDX-License-Identifier: GPL-3.0-or-later
//! `config.json` / `state.json` dạng tài liệu JSON **giữ nguyên** — cho bảng cài đặt.
//!
//! Vì sao không ghi thẳng struct [`Config`]: serde bỏ các khoá chưa định nghĩa
//! (`hotkeys`, `ignore_apps`, … — P0-3 §1.1), nên mỗi lần bấm một checkbox sẽ âm thầm xoá
//! phần người dùng tự thêm. Ở đây mọi thay đổi là **vá một khoá** trên JSON đã đọc:
//! khoá lạ, `macros[]`, `emoji[]`, `english_words[]` đi qua nguyên vẹn.
//!
//! Mọi lần ghi đều nguyên tử (file tạm cùng thư mục → fsync → rename): IME đọc lại file
//! theo mtime ở mỗi phím, đọc trúng file đang ghi dở sẽ mất cấu hình.
//!
//! File hỏng (không phải JSON object / sai schema) không bị ghi đè mất: lần lưu đầu tiên
//! đổi tên nó thành `<tên>.bak` trước.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::macro_text::{self, MacroLineError};
use crate::{parse_config, Config, MacroEntry};

/// Giới hạn kích thước file đọc vào (config thật chỉ vài KB).
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    /// `config.json` — khoá thiếu lấy mặc định của schema; mỗi lần sửa phải còn hợp lệ.
    Config,
    /// `state.json` — trạng thái bật/tắt (`global_enabled`, `apps{}`), không có schema.
    State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocError {
    /// Không đọc/ghi được file.
    Io,
    /// Khoá không thuộc schema / sai kiểu so với mặc định.
    UnknownKey,
    /// Giá trị làm config không còn hợp lệ (enum sai, …).
    BadValue,
}

#[derive(Debug, Clone)]
pub struct SettingsDoc {
    kind: DocKind,
    map: Map<String, Value>,
    /// File có tồn tại nhưng không đọc được thành tài liệu hợp lệ.
    corrupt: bool,
}

fn defaults() -> Map<String, Value> {
    match serde_json::to_value(Config::default()) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

impl SettingsDoc {
    /// Tài liệu rỗng (mọi khoá lấy mặc định).
    pub fn new(kind: DocKind) -> Self {
        SettingsDoc {
            kind,
            map: Map::new(),
            corrupt: false,
        }
    }

    /// Parse text. Không hợp lệ → tài liệu rỗng đánh dấu `corrupt`.
    pub fn from_json(text: &str, kind: DocKind) -> Self {
        let parsed = match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(map)) => Some(map),
            _ => None,
        };
        let valid = match (&parsed, kind) {
            (Some(_), DocKind::Config) => parse_config(text).is_ok(),
            (Some(_), DocKind::State) => true,
            (None, _) => false,
        };
        match parsed {
            Some(map) if valid => SettingsDoc {
                kind,
                map,
                corrupt: false,
            },
            _ => SettingsDoc {
                kind,
                map: Map::new(),
                corrupt: true,
            },
        }
    }

    /// Đọc file. File chưa có → tài liệu rỗng, không lỗi (lần đầu chạy).
    pub fn load(path: &Path, kind: DocKind) -> Self {
        let Ok(meta) = std::fs::metadata(path) else {
            return Self::new(kind);
        };
        if !meta.is_file() || meta.len() > MAX_FILE_BYTES {
            let mut doc = Self::new(kind);
            doc.corrupt = meta.is_file();
            return doc;
        }
        match std::fs::read_to_string(path) {
            Ok(text) => Self::from_json(&text, kind),
            Err(_) => {
                let mut doc = Self::new(kind);
                doc.corrupt = true;
                doc
            }
        }
    }

    pub fn kind(&self) -> DocKind {
        self.kind
    }

    /// File gốc hỏng (sẽ được sao lưu thành `.bak` ở lần lưu đầu).
    pub fn was_corrupt(&self) -> bool {
        self.corrupt
    }

    fn value(&self, key: &str) -> Option<Value> {
        if let Some(v) = self.map.get(key) {
            return Some(v.clone());
        }
        match self.kind {
            DocKind::Config => defaults().get(key).cloned(),
            DocKind::State => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.value(key).and_then(|v| v.as_bool())
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.value(key).and_then(|v| v.as_str().map(str::to_string))
    }

    /// Đặt `key = value`; với config: khoá phải có trong schema cùng kiểu và kết quả
    /// phải parse được — nếu không, tài liệu giữ nguyên.
    fn set_value(&mut self, key: &str, value: Value) -> Result<(), DocError> {
        if self.kind == DocKind::Config {
            let same_type = matches!(
                (defaults().get(key), &value),
                (Some(Value::Bool(_)), Value::Bool(_))
                    | (Some(Value::String(_)), Value::String(_))
                    | (Some(Value::Array(_)), Value::Array(_))
            );
            if !same_type {
                return Err(DocError::UnknownKey);
            }
        }
        let old = self.map.insert(key.to_string(), value);
        if self.kind == DocKind::Config && parse_config(&self.to_json()).is_err() {
            match old {
                Some(v) => self.map.insert(key.to_string(), v),
                None => self.map.remove(key),
            };
            return Err(DocError::BadValue);
        }
        Ok(())
    }

    pub fn set_bool(&mut self, key: &str, v: bool) -> Result<(), DocError> {
        self.set_value(key, Value::Bool(v))
    }

    pub fn set_str(&mut self, key: &str, v: &str) -> Result<(), DocError> {
        self.set_value(key, Value::String(v.to_string()))
    }

    /// `macros[]` hiện tại (mục hỏng đã bị loại ở bước parse file).
    pub fn macros(&self) -> Vec<MacroEntry> {
        self.map
            .get("macros")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Bảng gõ tắt dạng text (xem [`macro_text`]).
    pub fn macros_text(&self) -> String {
        macro_text::format(&self.macros())
    }

    /// Thay `macros[]` bằng nội dung ô soạn thảo. Lỗi → không đổi gì, trả dòng lỗi đầu tiên.
    pub fn set_macros_text(&mut self, text: &str) -> Result<(), MacroLineError> {
        let parsed = macro_text::parse(text, &self.macros())?;
        let value = serde_json::to_value(parsed).unwrap_or(Value::Array(Vec::new()));
        // Mục đã qua kiểm tra của `macro_text` luôn hợp lệ với schema.
        let _ = self.set_value("macros", value);
        Ok(())
    }

    /// Nút "Mặc định": mọi tuỳ chọn về mặc định; **giữ** gõ tắt, emoji, từ tiếng Anh và
    /// khoá người dùng tự thêm (UniKey cũng không xoá bảng gõ tắt khi bấm Mặc định).
    pub fn reset_defaults(&mut self) {
        if self.kind != DocKind::Config {
            self.map.clear();
            return;
        }
        for (key, value) in defaults() {
            if matches!(key.as_str(), "macros" | "emoji" | "english_words") {
                continue;
            }
            self.map.insert(key, value);
        }
    }

    /// Ghi đè các trường engine biết bằng `cfg`, giữ nguyên khoá lạ.
    pub fn merge_config(&mut self, cfg: &Config) {
        if let Ok(Value::Object(m)) = serde_json::to_value(cfg) {
            for (k, v) in m {
                self.map.insert(k, v);
            }
        }
    }

    /// JSON đẹp để ghi file. Config luôn có `config_version` (trường bắt buộc — P0-3 §1).
    pub fn to_json(&self) -> String {
        let mut map = self.map.clone();
        if self.kind == DocKind::Config && !map.contains_key("config_version") {
            map.insert("config_version".into(), Value::from(1));
        }
        let mut s = serde_json::to_string_pretty(&Value::Object(map)).unwrap_or_default();
        s.push('\n');
        s
    }

    /// Ghi nguyên tử vào `path` (tạo thư mục cha nếu chưa có).
    pub fn save(&mut self, path: &Path) -> Result<(), DocError> {
        let json = self.to_json();
        if self.kind == DocKind::Config && parse_config(&json).is_err() {
            return Err(DocError::BadValue);
        }
        if self.corrupt && path.is_file() {
            let mut bak = path.as_os_str().to_owned();
            bak.push(".bak");
            std::fs::rename(path, PathBuf::from(bak)).map_err(|_| DocError::Io)?;
        }
        atomic_write(path, json.as_bytes()).map_err(|_| DocError::Io)?;
        self.corrupt = false;
        Ok(())
    }
}

fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
    }
}

/// Ghi nguyên tử: file tạm **riêng theo tiến trình** cùng thư mục (bảng cài đặt và IME
/// có thể cùng ghi `state.json`) → fsync → rename.
pub fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    create_private_dir(dir)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "textvn".into());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CNT: AtomicU64 = AtomicU64::new(0);
        let n = CNT.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("textvn-doc-{tag}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn missing_keys_read_schema_defaults() {
        let doc = SettingsDoc::new(DocKind::Config);
        let d = Config::default();
        assert_eq!(doc.get_bool("free_marking"), Some(d.free_marking));
        assert_eq!(doc.get_bool("quick_telex"), Some(false));
        assert_eq!(doc.get_str("method").as_deref(), Some("telex"));
        assert_eq!(doc.get_bool("nope"), None);
    }

    #[test]
    fn patching_one_key_keeps_unknown_keys_and_macros() {
        let src = r#"{"config_version":1,"hotkeys":{"toggle_vi_en":"Ctrl+Shift+Space"},
            "macros":[{"trigger":"cty","expand":"Công ty","when":"vi_on"}],
            "english_words":["text"]}"#;
        let mut doc = SettingsDoc::from_json(src, DocKind::Config);
        assert!(!doc.was_corrupt());
        doc.set_bool("quick_telex", true).unwrap();
        doc.set_str("output_charset", "vni_windows").unwrap();
        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        assert_eq!(out["hotkeys"]["toggle_vi_en"], "Ctrl+Shift+Space");
        assert_eq!(out["macros"][0]["when"], "vi_on");
        assert_eq!(out["english_words"][0], "text");
        assert_eq!(out["quick_telex"], true);
        assert_eq!(out["output_charset"], "vni_windows");
        assert!(parse_config(&doc.to_json()).is_ok());
    }

    #[test]
    fn invalid_values_are_rejected_without_changing_the_doc() {
        let mut doc = SettingsDoc::new(DocKind::Config);
        assert_eq!(doc.set_str("method", "dvorak"), Err(DocError::BadValue));
        assert_eq!(doc.get_str("method").as_deref(), Some("telex"));
        assert_eq!(doc.set_bool("method", true), Err(DocError::UnknownKey));
        assert_eq!(doc.set_bool("no_such_key", true), Err(DocError::UnknownKey));
        assert_eq!(
            doc.set_str("free_marking", "yes"),
            Err(DocError::UnknownKey)
        );
        assert!(parse_config(&doc.to_json()).is_ok());
    }

    #[test]
    fn state_doc_accepts_any_key() {
        let mut doc =
            SettingsDoc::from_json(r#"{"global_enabled":true,"apps":{}}"#, DocKind::State);
        assert_eq!(doc.get_bool("global_enabled"), Some(true));
        doc.set_bool("global_enabled", false).unwrap();
        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        assert_eq!(out["global_enabled"], false);
        assert!(out["apps"].is_object());
        assert!(out.get("config_version").is_none());
    }

    #[test]
    fn macros_text_round_trip_through_doc() {
        let mut doc = SettingsDoc::new(DocKind::Config);
        doc.set_macros_text("vn = Việt Nam\ncty = Công ty").unwrap();
        assert_eq!(doc.macros().len(), 2);
        assert_eq!(doc.macros_text(), "vn = Việt Nam\ncty = Công ty\n");
        let err = doc.set_macros_text("oops").unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(doc.macros().len(), 2, "lỗi không được xoá bảng cũ");
        let cfg = parse_config(&doc.to_json()).unwrap();
        assert_eq!(cfg.macros[0].expand, "Việt Nam");
    }

    #[test]
    fn reset_defaults_keeps_user_tables() {
        let src = r#"{"config_version":1,"method":"vni","quick_telex":true,
            "macros":[{"trigger":"a","expand":"b"}],"emoji":[{"trigger":":x","glyph":"❌"}],
            "custom":1}"#;
        let mut doc = SettingsDoc::from_json(src, DocKind::Config);
        doc.reset_defaults();
        let cfg = parse_config(&doc.to_json()).unwrap();
        assert_eq!(cfg.method, crate::Method::Telex);
        assert!(!cfg.quick_telex);
        assert_eq!(cfg.macros.len(), 1);
        assert_eq!(cfg.emoji.len(), 1);
        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        assert_eq!(out["custom"], 1);
    }

    #[test]
    fn save_is_atomic_creates_dirs_and_backs_up_corrupt_file() {
        let dir = tmpdir("save");
        let path = dir.join("sub").join("config.json");
        let mut doc = SettingsDoc::load(&path, DocKind::Config);
        assert!(!doc.was_corrupt(), "file chưa có không phải là hỏng");
        doc.set_bool("auto_capitalize", false).unwrap();
        doc.save(&path).unwrap();
        let back = SettingsDoc::load(&path, DocKind::Config);
        assert_eq!(back.get_bool("auto_capitalize"), Some(false));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        std::fs::write(&path, "{ hỏng").unwrap();
        let mut doc = SettingsDoc::load(&path, DocKind::Config);
        assert!(doc.was_corrupt());
        doc.set_str("method", "vni").unwrap();
        doc.save(&path).unwrap();
        let mut bak = path.as_os_str().to_owned();
        bak.push(".bak");
        assert_eq!(
            std::fs::read_to_string(PathBuf::from(bak)).unwrap(),
            "{ hỏng"
        );
        assert_eq!(
            SettingsDoc::load(&path, DocKind::Config)
                .get_str("method")
                .as_deref(),
            Some("vni")
        );
        // Không để lại file tạm.
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn schema_invalid_config_counts_as_corrupt() {
        let doc =
            SettingsDoc::from_json(r#"{"config_version":1,"method":"dvorak"}"#, DocKind::Config);
        assert!(doc.was_corrupt());
        let doc = SettingsDoc::from_json("[1,2]", DocKind::State);
        assert!(doc.was_corrupt());
    }
}
