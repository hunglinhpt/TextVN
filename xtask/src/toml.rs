// SPDX-License-Identifier: GPL-3.0-or-later
//! Parser TOML **tối giản** cho đúng schema `data/tables/*.toml` — cố ý không dùng
//! crate `toml` để `xtask` giữ **0 dependency** (xem `xtask/Cargo.toml`).
//!
//! Schema hỗ trợ (nhiều hơn thì trả `Err` thay vì đoán):
//! - `key = "chuỗi"` / `key = 123` / `key = true|false` / `key = ["a", "b", ...]`
//! - `[bảng]` và `[[mảng_bảng]]`
//! - comment `#` (ngoài chuỗi), khoảng trắng tùy ý.
//!
//! Không hỗ trợ (cố ý — báo lỗi ngay để không sinh ra bảng sai):
//! giá trị nhiều dòng, table lồng nhau, mảng số nguyên, `[[...]]` lồng nhau.

use std::collections::BTreeMap;
use std::fmt;

/// 1 giá trị trong TOML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Result<&str, String> {
        match self {
            Value::Str(s) => Ok(s),
            other => Err(format!("cần chuỗi, thấy {other:?}")),
        }
    }
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            other => Err(format!("cần bool, thấy {other:?}")),
        }
    }
    /// Danh sách chuỗi; chuỗi đơn được coi như danh sách 1 phần tử (tiện cho `from`).
    pub fn as_str_list(&self) -> Result<Vec<String>, String> {
        match self {
            Value::Str(s) => Ok(vec![s.clone()]),
            Value::List(items) => items
                .iter()
                .map(|v| v.as_str().map(str::to_string))
                .collect(),
            other => Err(format!("cần danh sách chuỗi, thấy {other:?}")),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{s}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::List(v) => write!(f, "{v:?}"),
        }
    }
}

/// 1 bảng trong file TOML.
#[derive(Debug, Default, Clone)]
pub struct Table {
    pub name: String,
    pub values: BTreeMap<String, Value>,
}

impl Table {
    pub fn get(&self, key: &str) -> Result<&Value, String> {
        self.values
            .get(key)
            .ok_or_else(|| format!("thiếu khoá `{key}` trong [{}]", self.name))
    }
    pub fn str(&self, key: &str) -> Result<String, String> {
        self.get(key)?.as_str().map(str::to_string)
    }
    pub fn bool(&self, key: &str) -> Result<bool, String> {
        self.get(key)?.as_bool()
    }
    pub fn str_list(&self, key: &str) -> Result<Vec<String>, String> {
        self.get(key)?.as_str_list()
    }
}

/// File đã parse: bảng thường + mảng bảng theo tên + khoá ở đầu file.
#[derive(Debug, Default)]
pub struct Doc {
    pub tables: BTreeMap<String, Table>,
    pub arrays: BTreeMap<String, Vec<Table>>,
    pub root: BTreeMap<String, Value>,
}

impl Doc {
    pub fn array(&self, name: &str) -> Result<&[Table], String> {
        let v = self
            .arrays
            .get(name)
            .ok_or_else(|| format!("thiếu mảng bảng [[{name}]]"))?;
        if v.is_empty() {
            return Err(format!("[[{name}]] rỗng"));
        }
        Ok(v)
    }
    pub fn array_opt(&self, name: &str) -> Vec<&Table> {
        self.arrays
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    pub fn table(&self, name: &str) -> Result<&Table, String> {
        self.tables
            .get(name)
            .ok_or_else(|| format!("thiếu bảng [{name}]"))
    }
    pub fn root_list(&self, key: &str) -> Result<Vec<String>, String> {
        self.root
            .get(key)
            .ok_or_else(|| format!("thiếu khoá gốc `{key}`"))?
            .as_str_list()
    }
}

/// Bỏ comment + trim, **không** đụng `#` nằm trong chuỗi.
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return line[..i].trim(),
            _ => {}
        }
    }
    line.trim()
}

/// Tách `key = value` (dấu `=` đầu tiên ngoài chuỗi).
fn split_kv(line: &str) -> Result<(&str, &str), String> {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '=' if !in_str => return Ok((line[..i].trim(), line[i + 1..].trim())),
            _ => {}
        }
    }
    Err(format!("không tìm thấy `=`: {line}"))
}

fn parse_value(raw: &str) -> Result<Value, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("giá trị rỗng".into());
    }
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }
    if let Some(inner) = s.strip_prefix('[') {
        let inner = inner
            .strip_suffix(']')
            .ok_or_else(|| format!("mảng thiếu `]`: {s}"))?;
        let mut out = Vec::new();
        for part in inner.split(',') {
            let p = part.trim();
            if p.is_empty() {
                continue; // cho phép `["a", "b",]` và `[]`
            }
            out.push(parse_value(p)?);
        }
        return Ok(Value::List(out));
    }
    if let Some(inner) = s.strip_prefix('"') {
        let inner = inner
            .strip_suffix('"')
            .ok_or_else(|| format!("chuỗi thiếu `\"`: {s}"))?;
        return Ok(Value::Str(inner.to_string()));
    }
    if let Ok(i) = s.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    // Hex `0x…` — bảng keycode (`keymap_mac.toml`) viết theo tài liệu Apple.
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16)
            .map(|v| Value::Int(v as i64))
            .map_err(|_| format!("giá trị hex không hợp lệ: {s}"));
    }
    Err(format!("giá trị không hợp lệ: {s}"))
}

pub fn parse(input: &str) -> Result<Doc, String> {
    let mut doc = Doc::default();
    // `None` = đang ở khoá gốc; `Some(name)` = trong bảng thường hoặc mảng bảng.
    let mut current: Option<String> = None;

    for (ln, raw_line) in input.lines().enumerate() {
        let line = strip_comment(raw_line);
        if line.is_empty() {
            continue;
        }
        let no_line = ln + 1;
        if let Some(rest) = line.strip_prefix("[[") {
            let name = rest
                .strip_suffix("]]")
                .ok_or_else(|| format!("dòng {no_line}: [[..]] thiếu `]]`"))?
                .trim()
                .to_string();
            if name.is_empty() {
                return Err(format!("dòng {no_line}: tên mảng bảng rỗng"));
            }
            doc.arrays.entry(name.clone()).or_default().push(Table {
                name: name.clone(),
                values: BTreeMap::new(),
            });
            current = Some(name);
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let name = rest
                .strip_suffix(']')
                .ok_or_else(|| format!("dòng {no_line}: [..] thiếu `]`"))?
                .trim()
                .to_string();
            if name.is_empty() {
                return Err(format!("dòng {no_line}: tên bảng rỗng"));
            }
            let entry = doc.tables.entry(name.clone()).or_insert_with(|| Table {
                name: name.clone(),
                values: BTreeMap::new(),
            });
            entry.name = name.clone();
            current = Some(name);
            continue;
        }
        let (k, v) = split_kv(line).map_err(|e| format!("dòng {no_line}: {e}"))?;
        let value = parse_value(v).map_err(|e| format!("dòng {no_line}: {e}"))?;
        match current.as_deref() {
            None => {
                doc.root.insert(k.to_string(), value);
            }
            Some(name) => {
                if let Some(t) = doc.tables.get_mut(name) {
                    t.values.insert(k.to_string(), value);
                } else if let Some(list) = doc.arrays.get_mut(name) {
                    list.last_mut()
                        .ok_or_else(|| format!("dòng {no_line}: mảng bảng rỗng"))?
                        .values
                        .insert(k.to_string(), value);
                } else {
                    return Err(format!("dòng {no_line}: bảng lạ `{name}`"));
                }
            }
        }
    }
    Ok(doc)
}

/// Digest FNV-1a 64 — nhúng vào file sinh ra để phát hiện "đã sửa data nhưng chưa gen".
pub fn digest(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# comment
tones = ["none", "acute"]
name = "vni"          # trailing comment

[method]
display = "VNI"
w_marker = false

[[tone]]
key = "1"
tone = 1

[[tone]]
key = "2"
tone = 2
"#;

    #[test]
    fn parse_bang_doc_day_du() {
        let d = parse(SAMPLE).unwrap();
        assert_eq!(d.root_list("tones").unwrap(), vec!["none", "acute"]);
        assert_eq!(d.root.get("name").unwrap().as_str().unwrap(), "vni");
        let m = d.table("method").unwrap();
        assert_eq!(m.str("display").unwrap(), "VNI");
        assert!(!m.bool("w_marker").unwrap());
        let tones = d.array("tone").unwrap();
        assert_eq!(tones.len(), 2);
        assert_eq!(tones[1].str("key").unwrap(), "2");
    }

    #[test]
    fn comment_trong_chuoi_khong_bi_cat() {
        let d = parse("k = \"a#b\"").unwrap();
        assert_eq!(d.root.get("k").unwrap().as_str().unwrap(), "a#b");
    }

    #[test]
    fn bao_loi_schema_lai() {
        assert!(parse("k = [\"a\"").is_err(), "mảng thiếu ] phải lỗi");
        assert!(parse("k = \"a").is_err(), "chuỗi thiếu dấu phảy phải lỗi");
        assert!(parse("khong_co_dau_bang").is_err(), "thiếu = phải lỗi");
        assert!(parse("k = @@@").is_err(), "giá trị rác phải lỗi");
    }

    #[test]
    fn str_list_nhan_ca_chuoi_don() {
        let d = parse("from = \"A\"").unwrap();
        assert_eq!(
            d.root.get("from").unwrap().as_str_list().unwrap(),
            vec!["A"]
        );
    }

    #[test]
    fn digest_phat_hien_thay_doi_1_byte() {
        assert_ne!(digest(b"abc"), digest(b"abd"));
        assert_eq!(digest(b"abc"), digest(b"abc"));
    }

    #[test]
    fn hex_int_cho_bang_keycode() {
        let d = parse("kvk = 0x31\nvk = 0X20\n").unwrap();
        assert_eq!(d.root.get("kvk").unwrap(), &Value::Int(0x31));
        assert_eq!(d.root.get("vk").unwrap(), &Value::Int(0x20));
        assert!(parse("k = 0xZZ").is_err(), "hex rác phải lỗi");
        assert!(parse("k = 0x1FFFFFFFF").is_err(), "tràn u32 phải lỗi");
    }
}
