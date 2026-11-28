// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-9 (P0-6 §5b, G22) — `cargo xtask gen-ui` / `check-ui`: MỘT nguồn chữ UI
//! (`data/ui/strings.toml`) sinh mã cho 3 lớp UI native:
//!
//! - `tray/src/ui_strings_generated.rs`            (Windows — Win32)
//! - `adapters/linux-settings/src/ui_strings_generated.h` (Linux — GTK, macro để ghép chuỗi lúc biên dịch)
//! - `adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift` (macOS — SwiftUI/AppKit)
//!
//! `check-ui` (preflight + ci-shared) không ghi file: bản sinh lệch nguồn ⇒ exit 1. Thêm gate
//! **không chép tay**: mã viết tay của 3 lớp UI không được chứa lại literal `"<chữ>"` của bất kỳ
//! mục nào — chữ chung chỉ sống ở nguồn TOML (thay test parity `include_str!` cũ).

use std::path::Path;

use crate::toml::{self, Value};

const SRC: &str = "data/ui/strings.toml";
const OUT_RUST: &str = "tray/src/ui_strings_generated.rs";
const OUT_C: &str = "adapters/linux-settings/src/ui_strings_generated.h";
const OUT_SWIFT: &str = "adapters/macos-app/Sources/TextVNAppLib/UiStringsGenerated.swift";

/// Thư mục mã UI viết tay (quét gate "không chép tay") + đuôi tệp.
const HAND_WRITTEN: [(&str, &[&str]); 3] = [
    ("tray/src", &["rs"]),
    ("adapters/linux-settings/src", &["c", "h"]),
    ("adapters/macos-app/Sources", &["swift"]),
];

/// Mô tả trợ lý phải vừa MỘT dòng (yêu cầu chủ repo 0.2.76).
const ONE_LINE_IDS: [&str; 1] = ["assist_mode_hints"];
const ONE_LINE_MAX: usize = 55;

const SPDX_KEY: &str = "SPDX-License-Identifier";
const LICENSE: &str = "GPL-3.0-or-later";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiValue {
    One(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiString {
    pub id: String,
    pub value: UiValue,
}

impl UiString {
    fn texts(&self) -> Vec<&str> {
        match &self.value {
            UiValue::One(s) => vec![s.as_str()],
            UiValue::List(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

fn check_text(id: &str, s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err(format!("`{id}`: chữ rỗng"));
    }
    if let Some(c) = s
        .chars()
        .find(|c| *c == '"' || *c == '\\' || c.is_control())
    {
        return Err(format!(
            "`{id}`: ký tự {c:?} không được phép (escape khác nhau giữa Rust/C/Swift)"
        ));
    }
    Ok(())
}

/// Đọc + kiểm nguồn: id snake_case, mục đơn không trùng, danh sách liền 0..n-1, chữ hợp lệ.
pub fn parse(input: &str) -> Result<Vec<UiString>, String> {
    let doc = toml::parse(input)?;
    let mut out: Vec<UiString> = Vec::new();
    let mut lists: Vec<(String, Vec<(i64, String)>)> = Vec::new();
    for t in doc.array("string")? {
        for k in t.values.keys() {
            if !matches!(k.as_str(), "id" | "index" | "vi") {
                return Err(format!("[[string]] khoá lạ `{k}`"));
            }
        }
        let id = t.str("id")?;
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            || id.starts_with(|c: char| c.is_ascii_digit())
        {
            return Err(format!("id `{id}` phải là snake_case ASCII"));
        }
        let vi = t.str("vi")?;
        check_text(&id, &vi)?;
        match t.values.get("index") {
            None => {
                if out.iter().any(|u| u.id == id) || lists.iter().any(|(l, _)| *l == id) {
                    return Err(format!("id `{id}` trùng"));
                }
                out.push(UiString {
                    id,
                    value: UiValue::One(vi),
                });
            }
            Some(Value::Int(i)) => {
                if out
                    .iter()
                    .any(|u| u.id == id && matches!(u.value, UiValue::One(_)))
                {
                    return Err(format!("id `{id}` vừa là mục đơn vừa là danh sách"));
                }
                match lists.iter_mut().find(|(l, _)| *l == id) {
                    Some((_, items)) => items.push((*i, vi)),
                    None => {
                        // Giữ chỗ theo thứ tự xuất hiện; điền thật sau khi gom đủ phần tử.
                        out.push(UiString {
                            id: id.clone(),
                            value: UiValue::List(Vec::new()),
                        });
                        lists.push((id, vec![(*i, vi)]));
                    }
                }
            }
            Some(other) => return Err(format!("`{id}`: index phải là số nguyên, thấy {other:?}")),
        }
    }
    for (id, mut items) in lists {
        items.sort_by_key(|(i, _)| *i);
        for (want, (got, _)) in items.iter().enumerate() {
            if *got != want as i64 {
                return Err(format!(
                    "danh sách `{id}`: index phải liền 0..{} (thiếu/trùng ở {want})",
                    items.len() - 1
                ));
            }
        }
        let v: Vec<String> = items.into_iter().map(|(_, s)| s).collect();
        let slot = out.iter_mut().find(|u| u.id == id).expect("giữ chỗ ở trên");
        slot.value = UiValue::List(v);
    }
    for id in ONE_LINE_IDS {
        if let Some(u) = out.iter().find(|u| u.id == id) {
            for s in u.texts() {
                if s.chars().count() > ONE_LINE_MAX {
                    return Err(format!(
                        "`{id}`: `{s}` dài {} > {ONE_LINE_MAX} ký tự — phải vừa MỘT dòng",
                        s.chars().count()
                    ));
                }
            }
        }
    }
    Ok(out)
}

fn upper(id: &str) -> String {
    id.to_ascii_uppercase()
}

fn camel(id: &str) -> String {
    let mut out = String::new();
    for (i, part) in id.split('_').enumerate() {
        if i == 0 {
            out.push_str(part);
        } else {
            let mut cs = part.chars();
            if let Some(f) = cs.next() {
                out.push(f.to_ascii_uppercase());
                out.push_str(cs.as_str());
            }
        }
    }
    out
}

fn header(line_comment: &str, digest: u64) -> String {
    format!(
        "{c} {SPDX_KEY}: {LICENSE}\n\
         {c} == GENERATED FILE — KHÔNG SỬA TAY ==\n\
         {c} Nguồn: {SRC} (digest FNV-1a 64 = 0x{digest:016x}) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).\n\
         {c} Đổi chữ: sửa {SRC} rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.\n",
        c = line_comment
    )
}

pub fn render_rust(items: &[UiString], digest: u64) -> String {
    let mut s = header("//", digest);
    s.push_str(
        "//! Chữ UI dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.\n",
    );
    s.push_str("#![allow(dead_code)]\n");
    for u in items {
        s.push('\n');
        match &u.value {
            UiValue::One(v) => {
                let single = format!("pub const {}: &str = \"{v}\";", upper(&u.id));
                if single.chars().count() > 100 {
                    s.push_str(&format!(
                        "pub const {}: &str =\n    \"{v}\";\n",
                        upper(&u.id)
                    ));
                } else {
                    s.push_str(&single);
                    s.push('\n');
                }
            }
            UiValue::List(v) => {
                let inline = v
                    .iter()
                    .map(|x| format!("\"{x}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                let single = format!(
                    "pub const {}: [&str; {}] = [{inline}];",
                    upper(&u.id),
                    v.len()
                );
                if single.chars().count() <= 100 {
                    s.push_str(&single);
                    s.push('\n');
                } else {
                    s.push_str(&format!(
                        "pub const {}: [&str; {}] = [\n",
                        upper(&u.id),
                        v.len()
                    ));
                    for x in v {
                        s.push_str(&format!("    \"{x}\",\n"));
                    }
                    s.push_str("];\n");
                }
            }
        }
    }
    s
}

pub fn render_c(items: &[UiString], digest: u64) -> String {
    let mut s = header("//", digest);
    s.push_str(
        "/* Chữ UI dùng chung Windows/Linux/macOS (G22). Macro chuỗi để ghép lúc biên dịch\n \
         * (vd `TV_UI_ABOUT_TAGLINE \"\\n\\n\" TV_UI_ABOUT_DETAILS_0`) — không thêm tệp .c vào build. */\n",
    );
    s.push_str("#ifndef TEXTVN_UI_STRINGS_GENERATED_H\n#define TEXTVN_UI_STRINGS_GENERATED_H\n");
    for u in items {
        s.push('\n');
        let name = format!("TV_UI_{}", upper(&u.id));
        match &u.value {
            UiValue::One(v) => s.push_str(&format!("#define {name} \"{v}\"\n")),
            UiValue::List(v) => {
                s.push_str(&format!("#define {name}_COUNT {}\n", v.len()));
                for (i, x) in v.iter().enumerate() {
                    s.push_str(&format!("#define {name}_{i} \"{x}\"\n"));
                }
            }
        }
    }
    s.push_str("\n#endif /* TEXTVN_UI_STRINGS_GENERATED_H */\n");
    s
}

pub fn render_swift(items: &[UiString], digest: u64) -> String {
    let mut s = header("//", digest);
    s.push_str(
        "\n/// Chữ UI dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.\n",
    );
    s.push_str("enum UiStrings {\n");
    for (n, u) in items.iter().enumerate() {
        if n > 0 {
            s.push('\n');
        }
        match &u.value {
            UiValue::One(v) => s.push_str(&format!("    static let {} = \"{v}\"\n", camel(&u.id))),
            UiValue::List(v) => {
                s.push_str(&format!("    static let {}: [String] = [\n", camel(&u.id)));
                for x in v {
                    s.push_str(&format!("        \"{x}\",\n"));
                }
                s.push_str("    ]\n");
            }
        }
    }
    s.push_str("}\n");
    s
}

fn is_generated(rel: &str) -> bool {
    rel == OUT_RUST || rel == OUT_C || rel == OUT_SWIFT
}

fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut chunk_start = 0;
    while i < b.len() {
        // Chuỗi thường `"..."` (giữ escape `\"` và `\\`).
        if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 2;
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Chuỗi thô Rust `r#"..."#` (chỉ khi đứng riêng, không nằm cuối identifier).
        if b[i] == b'r' && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) {
            let mut k = 0;
            while i + 1 + k < b.len() && b[i + 1 + k] == b'#' {
                k += 1;
            }
            if i + 1 + k < b.len() && b[i + 1 + k] == b'"' {
                i += 2 + k;
                while i < b.len() {
                    if b[i] == b'"'
                        && i + k < b.len()
                        && b[i + 1..=i + k].iter().all(|c| *c == b'#')
                    {
                        i += 1 + k;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }
        // Chú thích dòng `// ...`.
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            out.push_str(&src[chunk_start..i]);
            i += 2;
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            chunk_start = i;
            continue;
        }
        // Chú thích khối `/* ... */`.
        if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            out.push_str(&src[chunk_start..i]);
            out.push(' ');
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(b.len());
            chunk_start = i;
            continue;
        }
        i += 1;
    }
    out.push_str(&src[chunk_start..]);
    out
}

/// Gate "không chép tay": trả `tệp: "chữ"` cho mỗi literal trùng nguồn trong mã UI viết tay
/// (bỏ qua chú thích `//`/`/* */` và nhãn đơn rất ngắn < 4 ký tự không khoảng trắng như "VNI"/"Tắt").
pub fn hand_copies(root: &Path, items: &[UiString]) -> Result<Vec<String>, String> {
    let mut bad = Vec::new();
    for (dir, exts) in HAND_WRITTEN {
        let mut stack = vec![root.join(dir)];
        while let Some(d) = stack.pop() {
            let rd = std::fs::read_dir(&d).map_err(|e| format!("đọc {}: {e}", d.display()))?;
            for e in rd {
                let p = e.map_err(|e| e.to_string())?.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let Some(ext) = p.extension().and_then(|x| x.to_str()) else {
                    continue;
                };
                if !exts.contains(&ext) {
                    continue;
                }
                let rel = p
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                if is_generated(&rel) {
                    continue;
                }
                let raw = std::fs::read_to_string(&p).map_err(|e| format!("đọc {rel}: {e}"))?;
                let text = strip_comments(&raw);
                for u in items {
                    for t in u.texts() {
                        if !t.contains(' ') && t.chars().count() < 4 {
                            continue;
                        }
                        if text.contains(&format!("\"{t}\"")) {
                            bad.push(format!("{rel}: \"{t}\" (id `{}`)", u.id));
                        }
                    }
                }
            }
        }
    }
    bad.sort();
    Ok(bad)
}

pub fn run(write: bool) -> Result<(), String> {
    let root = crate::assist_tools::repo_root();
    let src = std::fs::read_to_string(root.join(SRC))
        .map_err(|e| format!("đọc {SRC}: {e}"))?
        .replace("\r\n", "\n");
    let items = parse(&src)?;
    let digest = toml::digest(src.as_bytes());
    let outs = [
        (OUT_RUST, render_rust(&items, digest)),
        (OUT_C, render_c(&items, digest)),
        (OUT_SWIFT, render_swift(&items, digest)),
    ];
    let mut stale = Vec::new();
    for (rel, want) in &outs {
        let path = root.join(rel);
        let have = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        if have == *want {
            println!("  ok   {rel}");
        } else if write {
            std::fs::write(&path, want).map_err(|e| format!("ghi {rel}: {e}"))?;
            println!("  gen  {rel}");
        } else {
            stale.push(*rel);
        }
    }
    if !stale.is_empty() {
        return Err(format!(
            "check-ui: {} tệp lệch {SRC} — chạy `cargo xtask gen-ui` rồi commit: {}",
            stale.len(),
            stale.join(", ")
        ));
    }
    let bad = hand_copies(&root, &items)?;
    if !bad.is_empty() {
        return Err(format!(
            "SF-9/G22: chữ UI dùng chung bị chép tay ({} chỗ) — dùng hằng sinh mã \
             (Rust `ui_strings_generated::*`, C `TV_UI_*`, Swift `UiStrings.*`):\n  {}",
            bad.len(),
            bad.join("\n  ")
        ));
    }
    let n: usize = items.iter().map(|u| u.texts().len()).sum();
    println!(
        "{}: {} mục / {n} chuỗi, 3 bản sinh khớp nguồn, 0 bản chép tay",
        if write { "gen-ui" } else { "check-ui" },
        items.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[string]]
id = "about_tagline"
vi = "Bộ gõ, đa nền tảng"

[[string]]
id = "assist_mode_hints"
index = 1
vi = "Hai"

[[string]]
id = "assist_mode_hints"
index = 0
vi = "Một"
"#;

    #[test]
    fn sf9_parse_gom_danh_sach_theo_index_va_giu_thu_tu() {
        let items = parse(SAMPLE).unwrap();
        assert_eq!(items[0].value, UiValue::One("Bộ gõ, đa nền tảng".into()));
        assert_eq!(items[1].id, "assist_mode_hints");
        assert_eq!(
            items[1].value,
            UiValue::List(vec!["Một".into(), "Hai".into()])
        );
    }

    #[test]
    fn sf9_parse_tu_choi_nguon_sai() {
        let dup = "[[string]]\nid = \"a\"\nvi = \"x\"\n[[string]]\nid = \"a\"\nvi = \"y\"\n";
        assert!(parse(dup).unwrap_err().contains("trùng"));
        let gap = "[[string]]\nid = \"l\"\nindex = 0\nvi = \"x\"\n[[string]]\nid = \"l\"\nindex = 2\nvi = \"y\"\n";
        assert!(parse(gap).unwrap_err().contains("liền"));
        let camel_id = "[[string]]\nid = \"aB\"\nvi = \"x\"\n";
        assert!(parse(camel_id).unwrap_err().contains("snake_case"));
        let quote = "[[string]]\nid = \"q\"\nvi = \"a\\b\"\n";
        assert!(parse(quote).unwrap_err().contains("không được phép"));
        let long = format!(
            "[[string]]\nid = \"assist_mode_hints\"\nindex = 0\nvi = \"{}\"\n",
            "x".repeat(ONE_LINE_MAX + 1)
        );
        assert!(parse(&long).unwrap_err().contains("MỘT dòng"));
        let odd_key = "[[string]]\nid = \"a\"\nen = \"x\"\nvi = \"y\"\n";
        assert!(parse(odd_key).unwrap_err().contains("khoá lạ"));
    }

    #[test]
    fn sf9_ba_ban_sinh_cung_noi_dung() {
        let items = parse(SAMPLE).unwrap();
        let rs = render_rust(&items, 1);
        let c = render_c(&items, 1);
        let sw = render_swift(&items, 1);
        assert!(rs.contains("pub const ABOUT_TAGLINE: &str = \"Bộ gõ, đa nền tảng\";"));
        assert!(rs.contains("pub const ASSIST_MODE_HINTS: [&str; 2] = [\"Một\", \"Hai\"];"));
        assert!(c.contains("#define TV_UI_ABOUT_TAGLINE \"Bộ gõ, đa nền tảng\""));
        assert!(c.contains(
            "#define TV_UI_ASSIST_MODE_HINTS_COUNT 2\n#define TV_UI_ASSIST_MODE_HINTS_0 \"Một\""
        ));
        assert!(sw.contains("    static let aboutTagline = \"Bộ gõ, đa nền tảng\""));
        assert!(sw.contains("    static let assistModeHints: [String] = [\n        \"Một\",\n        \"Hai\",\n    ]"));
    }

    /// Thay test `about_text_matches_every_platform` / `assist_mode_hints_match_every_platform`
    /// cũ: nguồn thật sinh đúng 3 bản ĐANG commit + 0 bản chép tay trong mã UI 3 OS.
    #[test]
    fn sf9_repo_ban_sinh_khop_nguon_va_khong_chep_tay() {
        run(false).unwrap();
    }

    #[test]
    fn sf9_gate_bat_ban_chep_tay() {
        let dir = std::env::temp_dir().join(format!("textvn-sf9-{}", std::process::id()));
        let tray = dir.join("tray/src");
        std::fs::create_dir_all(&tray).unwrap();
        std::fs::create_dir_all(dir.join("adapters/linux-settings/src")).unwrap();
        std::fs::create_dir_all(dir.join("adapters/macos-app/Sources")).unwrap();
        std::fs::write(
            tray.join("x.rs"),
            "const T: &str = \"Bộ gõ, đa nền tảng\";\n",
        )
        .unwrap();
        // Bản sinh được phép chứa chữ.
        std::fs::write(tray.join("ui_strings_generated.rs"), "\"Một\"").unwrap();
        let items = parse(SAMPLE).unwrap();
        let bad = hand_copies(&dir, &items).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(
            bad,
            vec!["tray/src/x.rs: \"Bộ gõ, đa nền tảng\" (id `about_tagline`)".to_string()]
        );
    }

    #[test]
    fn sf9_hand_copies_bo_qua_chu_thich() {
        let dir = std::env::temp_dir().join(format!("textvn-sf9-cmt-{}", std::process::id()));
        let tray = dir.join("tray/src");
        std::fs::create_dir_all(&tray).unwrap();
        std::fs::create_dir_all(dir.join("adapters/linux-settings/src")).unwrap();
        std::fs::create_dir_all(dir.join("adapters/macos-app/Sources")).unwrap();
        std::fs::write(
            tray.join("comments_only.rs"),
            "// \"Bộ gõ, đa nền tảng\"\n/* \"Bộ gõ, đa nền tảng\" */\nconst URL: &str = \"https://x//y\";\n",
        )
        .unwrap();
        let items = parse(SAMPLE).unwrap();
        let bad = hand_copies(&dir, &items).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            bad.is_empty(),
            "chú thích không được tính là chép tay: {bad:?}"
        );
    }
}
