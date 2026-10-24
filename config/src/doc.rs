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

/// Kiểu leaf của schema: bool↔bool, chuỗi↔chuỗi, số↔số, mảng↔mảng. Giá trị khác
/// kiểu mặc định (vd `set_bool` cho leaf số) phải bị từ chối — cùng ràng buộc bản cũ.
fn same_kind(default: &Value, value: &Value) -> bool {
    matches!(
        (default, value),
        (Value::Bool(_), Value::Bool(_))
            | (Value::String(_), Value::String(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::Array(_), Value::Array(_))
    )
}

/// Trị mặc định của schema tại đường lồng dấu chấm (`assist.display.idle_ms`).
/// `Some` chỉ khi **mọi** thành phần xuất hiện trong cây `Config::default()` —
/// khoá lạ ở bất kỳ cấp nào → `None` (giữ phía an toàn, không nới lỏng).
fn default_at(segs: &[&str]) -> Option<Value> {
    let mut cur = Value::Object(defaults());
    for seg in segs {
        cur = cur.get(seg)?.clone();
    }
    Some(cur)
}

/// Đi xuống `current` theo `segs` với `default` là nhánh mặc định tương ứng cùng cấp:
/// chỗ file đã có → dùng giá trị file (kể cả leaf); chỗ file thiếu → **toàn bộ phần
/// còn lại** của đường lấy từ cây mặc định của schema. `default` luôn là nhánh đúng
/// cấp (được chọn qua `default_at`), nhưng vẫn trả an toàn nếu trái ngẫu nhiên.
fn lookup_nested(current: &Value, default: &Value, segs: &[&str]) -> Option<Value> {
    let (&first, rest) = segs.split_first()?;
    let sub = default.get(first)?;
    match current.get(first) {
        Some(v) if rest.is_empty() => Some(v.clone()),
        Some(v) => lookup_nested(v, sub, rest),
        None if rest.is_empty() => Some(sub.clone()),
        // Trong file không có đoạn này → phần còn lại lấy trọn từ nhánh mặc định.
        None => {
            let mut cur = sub;
            for seg in rest {
                cur = cur.get(seg)?;
            }
            Some(cur.clone())
        }
    }
}

/// Ghi `value` vào đúng vị trí `segs` của `map`; object trung gian thiếu thì tạo mới,
/// có nhưng sai loại (file hỏng) thì thay — cả thay đổi được `parse_config` kiểm lại
/// trước khi công bố nên file hợp lệ chỉ ghi hợp lệ. Khoá khác **không** bị đụng tới.
fn insert_nested(map: &mut Map<String, Value>, segs: &[&str], value: Value) {
    let (first, rest) = segs
        .split_first()
        .expect("split('/') tạo ít nhất một thành phần");
    if rest.is_empty() {
        map.insert(first.to_string(), value);
    } else {
        let entry = map
            .entry(first.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        insert_nested(
            entry.as_object_mut().expect("đã kiểm là object"),
            rest,
            value,
        );
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

    /// Giá trị JSON đầy đủ tại `key` — đường phẳng và cả **lồng** (`assist.display.idle_ms`).
    /// Config: mọi thành phần của đường phải thuộc schema (khoá lạ → `None`); chỗ thiếu
    /// trong file lấy mặc định của `Config::default()`. Trả nguyên khối object với khoá
    /// không phải leaf (`get_value("assist")`) — còn `get_bool/get_str/get_u32` là dạng
    /// typed nên với object ở giữa sẽ trả `None` (giữ hành vi cũ).
    pub fn get_value(&self, key: &str) -> Option<Value> {
        self.value(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.value(key).and_then(|v| v.as_bool())
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.value(key).and_then(|v| v.as_str().map(str::to_string))
    }

    /// Số nguyên 32 bit không dấu (`assist.display.idle_ms`, `assist.max_sentence_chars`).
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.value(key)
            .and_then(|v| v.as_u64().and_then(|n| u32::try_from(n).ok()))
    }

    fn value(&self, key: &str) -> Option<Value> {
        match self.kind {
            DocKind::State => self.map.get(key).cloned(),
            DocKind::Config => {
                let segs: Vec<&str> = key.split('.').collect();
                lookup_nested(
                    &Value::Object(self.map.clone()),
                    &Value::Object(defaults()),
                    &segs,
                )
            }
        }
    }

    /// Đặt `key = value` — `key` là đường phẳng hoặc **lồng** (`"assist.mode"`) theo cây
    /// `Config::default()`. Config: khoá phải có trong schema với cùng kiểu leaf và kết
    /// quả phải parse được — nếu không, tài liệu giữ nguyên. `state.json` là JSON tự do
    /// (không schema), giữ nguyên hành vi cũ: chép thẳng khoá, không kiểm gì.
    fn set_value(&mut self, key: &str, value: Value) -> Result<(), DocError> {
        if self.kind == DocKind::State {
            self.map.insert(key.to_string(), value);
            return Ok(());
        }
        let segs: Vec<&str> = key.split('.').collect();
        let leaf = default_at(&segs).ok_or(DocError::UnknownKey)?;
        if !same_kind(&leaf, &value) {
            return Err(DocError::UnknownKey);
        }
        // Chọn thử trên bản sao: toàn bộ thay đổi công bố chỉ khi kết quả vẫn parse được.
        let mut trial = self.map.clone();
        insert_nested(&mut trial, &segs, value);
        let probe = SettingsDoc {
            kind: self.kind,
            map: trial,
            corrupt: self.corrupt,
        };
        if self.kind == DocKind::Config && parse_config(&probe.to_json()).is_err() {
            return Err(DocError::BadValue);
        }
        self.map = probe.map;
        Ok(())
    }

    pub fn set_bool(&mut self, key: &str, v: bool) -> Result<(), DocError> {
        self.set_value(key, Value::Bool(v))
    }

    pub fn set_str(&mut self, key: &str, v: &str) -> Result<(), DocError> {
        self.set_value(key, Value::String(v.to_string()))
    }

    /// Đặt leaf số nguyên không dấu (phẳng hoặc lồng) — `assist.display.idle_ms`,
    /// `assist.max_sentence_chars`. Ràng buộc giữa `parse_config`
    /// (`max_sentence_chars >= 1`, `config_version == 1`) vẫn giữ nguyên.
    pub fn set_u32(&mut self, key: &str, v: u32) -> Result<(), DocError> {
        self.set_value(key, Value::Number(serde_json::Number::from(v)))
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

    /// `english_words[]` — từ điển EN của người dùng (0.2.13).
    pub fn english_words(&self) -> Vec<String> {
        self.map
            .get("english_words")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Danh sách từ EN dạng text (mỗi dòng một từ) cho ô soạn thảo GTK/Swift.
    pub fn english_words_text(&self) -> String {
        let mut out = self.english_words().join("\n");
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }

    /// Chuẩn hoá + thay `english_words[]`: trim, lowercase, chỉ chữ cái ASCII,
    /// bỏ dòng trống/`#`, khử trùng lặp giữ thứ tự — CÙNG quy tắc với
    /// `tray::settings_dialog::normalize_word_list` (Windows) để ba nền tảng
    /// lưu file y hệt nhau.
    pub fn set_english_words_text(&mut self, text: &str) -> Result<(), crate::DocError> {
        let words = normalize_english_words(text);
        let value = serde_json::to_value(&words).map_err(|_| crate::DocError::BadValue)?;
        self.set_value("english_words", value)
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

/// Ô soạn từ điển EN → `english_words[]`: trim, lowercase, chỉ chữ cái ASCII, bỏ dòng
/// trống/`#`, khử trùng lặp giữ thứ tự. Nguồn DUY NHẤT cho Windows (tray), Linux (GTK qua
/// C API) và macOS — ba nền tảng phải lưu file y hệt nhau.
pub fn normalize_english_words(text: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for line in text.replace("\r\n", "\n").lines() {
        let w = line.trim().to_lowercase();
        if w.is_empty() || w.starts_with('#') || !w.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        if !words.contains(&w) {
            words.push(w);
        }
    }
    words
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

    /// Đường lồng qua `SettingsDoc`: đọc giá trị mặc định khi file thiếu, ghi đúng vào
    /// object tương ứng (tạo mới nếu chưa có), không phá các khoá khác (P4-0 §7).
    #[test]
    fn nested_assist_paths_round_trip() {
        // File cũ KHÔNG có khối `assist` — đọc phải ra mặc định của schema.
        let mut doc = SettingsDoc::from_json(
            r#"{"config_version":1,"method":"vni","macros":[{"trigger":"a","expand":"b","when":"vi_on"}]}"#,
            DocKind::Config,
        );
        assert_eq!(doc.get_str("assist.mode").as_deref(), Some("highlight"));
        assert_eq!(
            doc.get_bool("assist.display.show_on_sentence_end"),
            Some(true)
        );
        assert_eq!(doc.get_u32("assist.display.idle_ms"), Some(700));

        // Ghi đúng vào từng khoá con dù object chưa tồn tại.
        doc.set_str("assist.mode", "highlight").unwrap();
        doc.set_u32("assist.display.idle_ms", 250).unwrap();
        assert_eq!(doc.get_str("assist.mode").as_deref(), Some("highlight"));
        assert_eq!(doc.get_u32("assist.display.idle_ms"), Some(250));

        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        // Vào đúng object, đúng cấp: `assist.mode`, `assist.display.idle_ms`.
        assert_eq!(out["assist"]["mode"], "highlight");
        assert_eq!(out["assist"]["display"]["idle_ms"], 250);
        // Khoá khác không bị đụng tới.
        assert_eq!(out["method"], "vni");
        assert_eq!(out["macros"][0]["expand"], "b");
        assert!(parse_config(&doc.to_json()).is_ok());
    }

    /// Khoá lồng: sai kiểu, sai đường, khoá lạ → UnknownKey; leaf object qua
    /// getter typed vẫn `None` (giữ hành vi với `get_*("assist")`).
    #[test]
    fn nested_unknown_and_type_errors() {
        let mut doc = SettingsDoc::new(DocKind::Config);
        // Khoá lồng chưa có bất kỳ cấp nào phải từ chối bằng UnknownKey, không tạo rác.
        assert_eq!(doc.set_bool("bo_gu.sai", true), Err(DocError::UnknownKey));
        // Sai kiểu leaf so với schema.
        assert_eq!(doc.set_bool("assist.mode", true), Err(DocError::UnknownKey));
        assert_eq!(doc.set_u32("assist.auto_fix", 1), Err(DocError::UnknownKey));
        // Đi qua leaf đủ dài (`mode` là chuỗi, không chứa `nope`).
        assert_eq!(
            doc.set_str("assist.mode.nope", "x"),
            Err(DocError::UnknownKey)
        );
        assert_eq!(doc.get_str("assist.mode.nope"), None);
        assert_eq!(doc.get_bool("assist"), None);
        assert_eq!(doc.get_str("assist"), None);
        assert_eq!(doc.set_bool("assist", true), Err(DocError::UnknownKey));
        assert_eq!(doc.get_u32("assist.bogus"), None);
        // Cả thay đổi sai này đều phải bị từ chối — file thực chỉ còn mặc định.
        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        assert!(
            out.get("assist").is_none(),
            "không được ghi khoá lạ vào file"
        );
        assert_eq!(
            out.as_object().unwrap().len(),
            1,
            "chỉ có config_version được thêm"
        );
    }

    /// Tương thích ngược: JSON có khối `assist` thiếu key con → lấy mặc định;
    /// ghi từng khoá không làm mất key người dùng tự thêm **trong** khối `assist`.
    #[test]
    fn nested_patch_preserves_partial_block_and_extras() {
        let mut doc = SettingsDoc::from_json(
            r#"{"config_version":1,"assist":{"per_app":{"code.exe":false},"custom":9}}"#,
            DocKind::Config,
        );
        assert_eq!(doc.get_str("assist.mode").as_deref(), Some("highlight"));
        assert_eq!(doc.get_u32("assist.display.idle_ms"), Some(700));
        doc.set_u32("assist.display.idle_ms", 900).unwrap();
        let out: Value = serde_json::from_str(&doc.to_json()).unwrap();
        assert_eq!(out["assist"]["display"]["idle_ms"], 900);
        // Trừ leaf vừa patch, key con thiếu KHÔNG được nặn vào file — đọc qua get lấy
        // mặc định từ `defaults()`; file ghi tối thiểu như bản cũ (chỉ patch đúng khoá).
        assert!(out["assist"]["display"]
            .get("show_on_sentence_end")
            .is_none());
        assert_eq!(
            out["assist"]["per_app"]["code.exe"], false,
            "per_app phải còn"
        );
        assert_eq!(out["assist"]["custom"], 9, "khoá lạ trong assist phải còn");
        assert!(
            out.get("display").is_none(),
            "không được lồng nhầm cấp trên"
        );
        assert!(parse_config(&doc.to_json()).is_ok());
    }

    /// Giá trị lồng vi phạm schema → BadValue (kèm rollback) qua `parse_config`
    /// chứ không phải kiểm tra kiểu leaf.
    #[test]
    fn nested_schema_violations_rejected() {
        let mut doc = SettingsDoc::new(DocKind::Config);
        assert_eq!(
            doc.set_u32("assist.max_sentence_chars", 0),
            Err(DocError::BadValue)
        );
        assert_eq!(doc.get_u32("assist.max_sentence_chars"), Some(500));
        assert_eq!(
            doc.set_str("assist.mode", "fix_everything"),
            Err(DocError::BadValue)
        );
        assert_eq!(doc.get_str("assist.mode").as_deref(), Some("highlight"));
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
        // Không để lại file tạm. Trên Windows, `.tmp` sau rename có thể còn
        // hiện trong namespace một nhịp khi Defender/indexer giữ handle
        // (audit R2 finding 3 — flake 1/3 lần) → retry trước khi fail.
        let mut leftovers_empty = false;
        for _ in 0..3 {
            leftovers_empty = std::fs::read_dir(path.parent().unwrap())
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
                .count()
                == 0;
            if leftovers_empty {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(leftovers_empty, "atomic_write không được để lại .tmp");
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
