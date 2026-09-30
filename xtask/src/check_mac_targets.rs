// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask/src/check_mac_targets.rs` — gate cho `tools/mac/targets/*.json` (MAC-061).
//!
//! Vì sao cần: `tools/mac/targets/` là **nguồn sự thật** cho harness AX (MAC-060).
//! Một `preset` sai tên, `field_role` ngoài whitelist, hay field chỉ có 1 locator
//! (không fallback) sẽ làm harness FAIL trên máy Mac thật mà PR không hề biết —
//! gate này chạy được trên **mọi OS** (CI + máy dev Windows/Linux).
//!
//! Schema: `tools/mac/README.md §1` (giữ nguyên tên khoá với
//! `tools/appcomptest/targets/` — nguyên tắc G15 reuse).
//!
//! `xtask` cố ý **0 dependency** → JSON parser tối thiểu nằm ngay trong file này
//! (đúng tinh thần `toml.rs`: chỉ parse schema mình cần, lỗi phải nói rõ byte nào).

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const TARGETS_DIR: &str = "tools/mac/targets";
const APP_DB: &str = "data/appdb.default.json";

/// 12 app CI của P2-5 §3 — phải có đủ file `<app_id>.json`.
const CI_APPS: [&str; 12] = [
    "textedit",
    "safari",
    "chrome",
    "firefox",
    "spotlight",
    "notes",
    "terminal",
    "vscode",
    "xcode",
    "slack",
    "discord",
    "finder",
];

/// 11 `field_role` hợp lệ — whitelist `P0-3 §2.1` (schema `appdb.v1`).
const FIELD_ROLES: [&str; 11] = [
    "unknown",
    "body",
    "editbox",
    "address_bar",
    "search",
    "combo",
    "candidate",
    "textarea",
    "web",
    "terminal",
    "secure",
];

/// Khoá locator được phép (map sang AX — `tools/mac/README.md §1`).
const LOCATOR_KEYS: [&str; 5] = [
    "control_type",
    "automation_id",
    "name",
    "class_name",
    "name_regex",
];

/// JSON tối thiểu — chỉ 6 kiểu mà schema này cần.
#[derive(Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(o) => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
}

/// Thu thập mọi giá trị của khoá `id` trong cây appdb — đó chính là **tên preset**
/// hợp lệ (`data/appdb.default.json` là mảng `entries[]`, mỗi entry có `id`).
fn collect_preset_ids(v: &Json, out: &mut Vec<String>) {
    match v {
        Json::Obj(o) => {
            for (k, val) in o {
                if k == "id" {
                    if let Json::Str(s) = val {
                        out.push(s.clone());
                    }
                }
                collect_preset_ids(val, out);
            }
        }
        Json::Arr(a) => {
            for x in a {
                collect_preset_ids(x, out);
            }
        }
        _ => {}
    }
}

/// `preset` có phải id thật trong appdb? (40 id — dò tuyến tính đủ nhanh cho gate.)
fn has_preset_id(appdb: &Json, want: &str) -> bool {
    let mut ids: Vec<String> = Vec::new();
    collect_preset_ids(appdb, &mut ids);
    ids.iter().any(|id| id == want)
}

/// Parser JSON tối giản (đủ cho schema targets + appdb).
struct P<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn parse(text: &'a str) -> Result<Json, String> {
        let mut p = P {
            b: text.as_bytes(),
            i: 0,
        };
        p.ws();
        let v = p.value()?;
        p.ws();
        if p.i != p.b.len() {
            return Err(format!("còn ký tự thừa ở byte {}", p.i));
        }
        Ok(v)
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => {
                self.lit("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.lit("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.lit("null")?;
                Ok(Json::Null)
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            Some(c) => Err(format!(
                "ký tự `{}` không mong đợi ở byte {}",
                c as char, self.i
            )),
            None => Err("hết input".to_string()),
        }
    }

    fn lit(&mut self, s: &str) -> Result<(), String> {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            Ok(())
        } else {
            Err(format!("literal `{s}` sai ở byte {}", self.i))
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.i += 1; // '{'
        let mut o: Vec<(String, Json)> = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(o));
        }
        loop {
            self.ws();
            let key = self.string()?;
            if o.iter().any(|(k, _)| *k == key) {
                return Err(format!("khoá `{key}` bị lặp"));
            }
            self.ws();
            if self.peek() != Some(b':') {
                return Err(format!("thiếu `:` sau khoá `{key}`"));
            }
            self.i += 1;
            self.ws();
            let v = self.value()?;
            o.push((key, v));
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(o));
                }
                _ => return Err(format!("thiếu `,` hoặc `}}` ở byte {}", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.i += 1; // '['
        let mut a: Vec<Json> = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(a));
        }
        loop {
            self.ws();
            a.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(a));
                }
                _ => return Err(format!("thiếu `,` hoặc `]` ở byte {}", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err(format!("cần `\"` ở byte {}", self.i));
        }
        self.i += 1;
        let mut out = String::new();
        loop {
            let c = self.peek().ok_or("chuỗi chưa đóng")?;
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let e = self.peek().ok_or("escape chưa xong")?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let end = self.i + 4;
                            let hex = self
                                .b
                                .get(self.i..end)
                                .ok_or("`\\u` cần đúng 4 chữ số hex")?;
                            let s = std::str::from_utf8(hex)
                                .map_err(|_| "`\\u` hex không phải UTF-8".to_string())?;
                            let n = u32::from_str_radix(s, 16)
                                .map_err(|_| format!("`\\u{s}` không phải hex"))?;
                            self.i = end;
                            // Surrogate chỉ đúng khi đi cặp; gate này chỉ tra *khoá* nên
                            // thay bằng U+FFFD là đủ (không cần khôi phục cặp surrogate).
                            out.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                        other => return Err(format!("escape `\\{}` không hợp lệ", other as char)),
                    }
                }
                _ => {
                    self.i += 1;
                    let start = self.i - 1;
                    let mut end = self.i;
                    while end < self.b.len() && self.b[end] != b'"' && self.b[end] != b'\\' {
                        end += 1;
                    }
                    let chunk = std::str::from_utf8(&self.b[start..end])
                        .map_err(|_| format!("UTF-8 lỗi ở byte {start}"))?;
                    out.push_str(chunk);
                    self.i = end;
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-')
        {
            self.i += 1;
        }
        let s = std::str::from_utf8(&self.b[start..self.i])
            .map_err(|_| format!("số không hợp lệ ở byte {start}"))?;
        s.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("số `{s}` không parse được"))
    }
}

/// Kiểm 1 file targets (thuần — không đụng filesystem để dễ test). Trả về danh sách
/// vấn đề, rỗng = hợp lệ.
fn validate_doc(file: &str, text: &str, appdb: &Json) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    let doc = match P::parse(text) {
        Ok(d) => d,
        Err(e) => {
            out.push(format!("{file}: JSON không parse được — {e}"));
            return out;
        }
    };

    // app_id phải khớp tên file (ax-driver tra theo app_id).
    let stem = file.strip_suffix(".json").unwrap_or(file);
    match doc.get("app_id").and_then(Json::as_str) {
        Some(id) if id == stem => {}
        Some(id) => out.push(format!(
            "{file}: `app_id` = `{id}` không khớp tên file `{stem}`"
        )),
        None => out.push(format!("{file}: thiếu `app_id`")),
    }

    // So đúng giá trị số: `*n as i64` cắt thập phân → `1.9` từng lọt (review R3 minor 10).
    let version_ok = matches!(doc.get("targets_version"), Some(Json::Num(n)) if *n == 1.0);
    if !version_ok {
        out.push(format!("{file}: `targets_version` phải = 1"));
    }

    // match.any[].bundle_id — macOS định danh app bằng bundle id (không có path DLL).
    match doc
        .get("match")
        .and_then(|m| m.get("any"))
        .and_then(Json::as_arr)
    {
        Some(list) if !list.is_empty() => {
            for (i, m) in list.iter().enumerate() {
                match m.get("bundle_id").and_then(Json::as_str) {
                    Some(b) if b.contains('.') && !b.starts_with('.') && !b.ends_with('.') => {}
                    _ => out.push(format!(
                        "{file}: match.any[{i}].bundle_id thiếu/dạng sai (vd `com.apple.TextEdit`)"
                    )),
                }
            }
        }
        _ => out.push(format!("{file}: thiếu `match.any` (mảng không rỗng)")),
    }

    // launch — ax-driver dùng để tự mở app khi test.
    let launch = doc.get("launch");
    match launch.and_then(|l| l.get("kind")).and_then(Json::as_str) {
        Some("app") => {}
        _ => out.push(format!("{file}: `launch.kind` phải = \"app\"")),
    }
    match launch.and_then(|l| l.get("paths")).and_then(Json::as_arr) {
        Some(p) if p.iter().any(|x| x.as_str().is_some_and(|s| s.ends_with(".app"))) => {}
        _ => out.push(format!(
            "{file}: `launch.paths` phải có ≥1 đường dẫn `.app` (Finder = /System/Library/CoreServices/Finder.app)"
        )),
    }
    match launch
        .and_then(|l| l.get("ready_role"))
        .and_then(Json::as_str)
    {
        Some(r) if r.starts_with("AX") => {}
        _ => out.push(format!(
            "{file}: `launch.ready_role` phải là AX role (`AX…`)"
        )),
    }

    // fields — mỗi field: role whitelist + preset tôn tại + ≥2 locator AX.
    match doc.get("fields").and_then(Json::as_arr) {
        Some(fields) if !fields.is_empty() => {
            for (fi, f) in fields.iter().enumerate() {
                let role = f.get("field_role").and_then(Json::as_str).unwrap_or("");
                if !FIELD_ROLES.contains(&role) {
                    out.push(format!(
                        "{file}: fields[{fi}].field_role `{role}` không thuộc whitelist P0-3 §2.1"
                    ));
                }
                match f.get("preset").and_then(Json::as_str) {
                    Some(p) if !p.is_empty() && has_preset_id(appdb, p) => {}
                    Some(p) => out.push(format!(
                        "{file}: fields[{fi}].preset `{p}` không tồn tại trong {APP_DB}"
                    )),
                    None => out.push(format!("{file}: fields[{fi}] thiếu `preset`")),
                }
                match f.get("locators").and_then(Json::as_arr) {
                    Some(locs) if locs.len() >= 2 => {
                        for (li, loc) in locs.iter().enumerate() {
                            if !LOCATOR_KEYS.iter().any(|k| loc.get(k).is_some()) {
                                out.push(format!(
                                    "{file}: fields[{fi}].locators[{li}] không có khoá locator nào ({LOCATOR_KEYS:?})"
                                ));
                            }
                            match loc.get("control_type").and_then(Json::as_str) {
                                Some(ct) if ct.starts_with("AX") => {}
                                Some(ct) => out.push(format!(
                                    "{file}: fields[{fi}].locators[{li}].control_type `{ct}` không phải AX role (Windows role lọt sang — xem bảng map ở tools/mac/README.md §1)"
                                )),
                                None => out.push(format!(
                                    "{file}: fields[{fi}].locators[{li}] thiếu `control_type`"
                                )),
                            }
                        }
                    }
                    Some(locs) => out.push(format!(
                        "{file}: fields[{fi}] mới có {} locator — cần ≥2 (MAC-061: 2 locator/field)",
                        locs.len()
                    )),
                    None => out.push(format!("{file}: fields[{fi}] thiếu `locators`")),
                }
            }
        }
        _ => out.push(format!("{file}: thiếu `fields` (mảng không rỗng)")),
    }

    out
}

/// Entry point `cargo run -p xtask -- check-mac-targets`.
pub fn run() -> Result<(), String> {
    let dir = Path::new(TARGETS_DIR);
    let appdb_text =
        fs::read_to_string(APP_DB).map_err(|e| format!("không đọc được `{APP_DB}`: {e}"))?;
    let appdb = P::parse(&appdb_text).map_err(|e| format!("`{APP_DB}` không parse được: {e}"))?;

    let entries =
        fs::read_dir(dir).map_err(|e| format!("không đọc được `{TARGETS_DIR}` (MAC-061): {e}"))?;
    let mut files: Vec<String> = Vec::new();
    for e in entries.flatten() {
        if let Some(n) = e.file_name().to_str() {
            if n.ends_with(".json") {
                files.push(n.to_string());
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("`{TARGETS_DIR}` không có file `.json` nào"));
    }

    let mut problems: Vec<String> = Vec::new();
    for f in &files {
        let path = dir.join(f);
        let text =
            fs::read_to_string(&path).map_err(|e| format!("đọc `{}`: {e}", path.display()))?;
        problems.extend(validate_doc(f, &text, &appdb));
    }

    // Đủ 12 app CI (P2-5 §3) — thiếu app nào gọi tên app đó.
    for app in CI_APPS {
        let want = format!("{app}.json");
        if !files.iter().any(|f| f == &want) {
            problems.push(format!(
                "{TARGETS_DIR}/{want} thiếu — P2-5 §3 yêu cầu đủ 12 app CI"
            ));
        }
    }

    if !problems.is_empty() {
        let mut msg = String::from("targets macOS không hợp lệ:\n");
        for p in &problems {
            let _ = writeln!(msg, "  - {p}");
        }
        return Err(msg);
    }

    println!(
        "  ok   {TARGETS_DIR}: {} file · 12 app CI đủ · ≥2 locator AX/field · preset khớp {APP_DB}",
        files.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// appdb tối giản cho test — **đúng shape thật**: mảng `entries[]`, mỗi entry có `id`.
    fn appdb() -> Json {
        P::parse(
            r#"{ "appdb_version": 1, "updated_at": "2026-09-29", "entries": [
                 { "id": "generic", "field_role": "editbox" },
                 { "id": "mac.textedit", "field_role": "body" },
                 { "id": "browser.url", "field_role": "address_bar" } ] }"#,
        )
        .expect("appdb test phải parse được")
    }

    fn good() -> String {
        r#"{
  "targets_version": 1,
  "app_id": "textedit",
  "match": { "any": [ { "bundle_id": "com.apple.TextEdit" } ] },
  "launch": {
    "kind": "app",
    "paths": [ "/System/Applications/TextEdit.app" ],
    "args": [],
    "ready_role": "AXTextArea"
  },
  "fields": [
    { "field_role": "body", "preset": "mac.textedit",
      "locators": [ { "control_type": "AXTextArea" },
                    { "control_type": "AXScrollArea" },
                    { "control_type": "AXGroup" } ] }
  ]
}"#
        .to_string()
    }

    /// Đổi 1 chuỗi con trong doc tốt (các test lỗi chỉ cần sửa 1 chỗ).
    fn mutated(find: &str, repl: &str) -> String {
        good().replace(find, repl)
    }

    #[test]
    fn parser_nhan_dung_schema() {
        let v = P::parse(&good()).expect("doc tốt phải parse được");
        assert_eq!(v.get("app_id").and_then(Json::as_str), Some("textedit"));
        assert_eq!(
            v.get("fields").and_then(|f| f.as_arr()).map(|a| a.len()),
            Some(1)
        );
    }

    #[test]
    fn parser_bao_loi_ro_vi_tri() {
        for bad in [
            r#"{ "a": }"#,             // thiếu value
            r#"{ "a": 1, }"#,          // dấu phẩy thừa
            r#"{ "a": 1 "b": 2 }"#,    // thiếu dấu phẩy
            r#"{ "a": "chuưa dong }"#, // chuỗi chưa đóng
            r#"{ "a": 1 } trailing"#,  // ký tự thừa
            r#"{ "a": 1, "a": 2 }"#,   // khoá lặp
            r#"{ "a": "\u00zz" }"#,    // \u không phải hex
        ] {
            assert!(P::parse(bad).is_err(), "phải báo lỗi: {bad}");
        }
    }

    #[test]
    fn validate_targets_version_phai_dung_1() {
        for bad in ["1.9", "1.5", "2", "0"] {
            let doc = mutated(
                "\"targets_version\": 1",
                &format!("\"targets_version\": {bad}"),
            );
            let errs = validate_doc("textedit.json", &doc, &appdb());
            assert!(
                errs.iter().any(|e| e.contains("targets_version")),
                "phải báo targets_version = {bad}: {errs:?}"
            );
        }
        let ok = mutated("\"targets_version\": 1", "\"targets_version\": 1.0");
        assert_eq!(
            validate_doc("textedit.json", &ok, &appdb()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn validate_chap_nhan_doc_tot() {
        assert_eq!(
            validate_doc("textedit.json", &good(), &appdb()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn validate_bao_field_chi_co_1_locator() {
        let doc = mutated(
            r#"{ "control_type": "AXTextArea" },
                    { "control_type": "AXScrollArea" },
                    { "control_type": "AXGroup" }"#,
            r#"{ "control_type": "AXTextArea" }"#,
        );
        let errs = validate_doc("textedit.json", &doc, &appdb());
        assert!(
            errs.iter().any(|e| e.contains("cần ≥2")),
            "phải báo thiếu locator: {errs:?}"
        );
    }

    #[test]
    fn validate_bao_control_type_windows() {
        // Locator mang `Edit` (UIA ControlType của Windows) lọt sang → phải bị chặn.
        let doc = mutated(
            r#""control_type": "AXTextArea""#,
            r#""control_type": "Edit""#,
        );
        let errs = validate_doc("textedit.json", &doc, &appdb());
        assert!(
            errs.iter().any(|e| e.contains("không phải AX role")),
            "phải báo role Windows lọt sang: {errs:?}"
        );
    }

    #[test]
    fn validate_bao_preset_khong_ton_tai() {
        let doc = mutated("mac.textedit", "mac.textedit.khong_co");
        let errs = validate_doc("textedit.json", &doc, &appdb());
        assert!(
            errs.iter()
                .any(|e| e.contains("preset") && e.contains("không tồn tại")),
            "phải báo preset ảo: {errs:?}"
        );
    }

    #[test]
    fn validate_bao_app_id_khong_khop_ten_file() {
        let errs = validate_doc(
            "textedit.json",
            &mutated("\"textedit\"", "\"safari\""),
            &appdb(),
        );
        assert!(
            errs.iter().any(|e| e.contains("không khớp tên file")),
            "phải báo app_id lệch: {errs:?}"
        );
    }

    #[test]
    fn validate_bao_field_role_ngoai_whitelist() {
        let doc = mutated(r#""field_role": "body""#, r#""field_role": "textbox""#);
        let errs = validate_doc("textedit.json", &doc, &appdb());
        assert!(
            errs.iter().any(|e| e.contains("whitelist")),
            "phải báo role ngoài whitelist: {errs:?}"
        );
    }

    #[test]
    fn validate_bao_bundle_id_thieu() {
        let doc = mutated(
            r#""bundle_id": "com.apple.TextEdit""#,
            r#""exe": "TextEdit""#,
        );
        let errs = validate_doc("textedit.json", &doc, &appdb());
        assert!(
            errs.iter().any(|e| e.contains("bundle_id")),
            "macOS phải bắt buộc bundle_id: {errs:?}"
        );
    }
}
