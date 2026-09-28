// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask` — công cụ build **duy nhất** được sinh code (P0-1 §3).
//!
//! Lệnh:
//! - `gen-tables`   — đọc `data/tables/*.toml` → sinh `core/src/transform/vowel_table_generated.rs`
//!   và `core/src/method/keys_generated.rs`.
//! - `check-tables` — như trên nhưng **không ghi file**: lệch thì exit 1 (gate CI).
//! - `help`         — in usage.
//!
//! Quy tắc bất di bất dịc: **không sửa tay** file có header `GENERATED`.
//! Đổi bảng → sửa `data/tables/*.toml` rồi chạy `gen-tables`.
//!
//! `cbindgen` (sinh lại `ffi/include/textvn_ffi.h`) **chưa có** trong xtask: cần
//! crate `cbindgen` + nightly. Header hiện do người viết giữ tay và CI kiểm bằng
//! `textvn sizes` + `ffi/tests/abi_invariants.rs` (ghi rõ ở `docs/00-INDEX §5`).

mod gen_win_corpus;
mod toml;
mod win_corpus_cases;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Nguồn: bảng âm Việt (12 âm × 6 dạng).
const VOWELS_TOML: &str = "data/tables/vowels.toml";
/// Nguồn: bảng phím của 4 kiểu gõ.
const METHODS_TOML: [&str; 4] = [
    "data/tables/telex.toml",
    "data/tables/simple_telex.toml",
    "data/tables/vni.toml",
    "data/tables/viqr.toml",
];

const VOWELS_OUT: &str = "core/src/transform/vowel_table_generated.rs";
const KEYS_OUT: &str = "core/src/method/keys_generated.rs";

/// License đóng dấu vào header file sinh ra. Khoá định danh được tạo động vì scanner
/// của REUSE quét chuỗi định danh nằm trong chú thích thành header bản quyền thật
/// (xem `docs/01-AGENT-HANDBOOK.md` S6).
const SPDX_KEY: &str = "SPDX-License-Identifier";
const LICENSE: &str = "GPL-3.0-or-later";

fn banner() -> String {
    format!(
        "// {SPDX_KEY}: {LICENSE}\n{}",
        concat!(
            "// == GENERATED FILE — KHÔNG SỬA TAY ==\n",
            "// Nguồn: data/tables/*.toml · sinh bằng `cargo xtask gen-tables` (P0-1 §3).\n",
            "// Đổi bảng: sửa file `.toml` rồi chạy lại `cargo xtask gen-tables`.\n",
            "// `cargo xtask check-tables` (CI) sẽ fail nếu file này lệch với nguồn.\n",
        )
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let result = match cmd {
        "gen-tables" => gen_tables(true),
        "check-tables" => gen_tables(false),
        "gen-win-corpus" => gen_win_corpus::run(true),
        "check-win-corpus" => gen_win_corpus::run(false),
        "help" | "--help" | "-h" => {
            print!("{}", usage());
            Ok(())
        }
        other => Err(format!("lệnh lạ `{other}`\n\n{}", usage())),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> &'static str {
    "xtask — công cụ build của TextVN (P0-1 §3)\n\n\
     Usage:\n  \
     cargo xtask gen-tables       # data/tables/*.toml → core (ghi file)\n  \
     cargo xtask check-tables     # kiểm tra file đã sinh có khớp nguồn (exit 1 nếu lệch)\n  \
     cargo xtask gen-win-corpus   # sinh corpus/win/*.keys (WIN-006)\n  \
     cargo xtask check-win-corpus # kiểm tra corpus/win/*.keys có khớp chuẩn\n  \
     cargo xtask help\n"
}

fn read(rel: &str) -> Result<String, String> {
    // chuẩn hoá line ending TRƯỚC khi parse/digest (FNV-1a trên bytes thô):
    // local Windows hay checkout CRLF, CI Linux LF — digest/embedded hash phải giống nhau.
    fs::read_to_string(rel)
        .map(|s| s.replace("\r\n", "\n"))
        .map_err(|e| format!("không đọc được `{rel}`: {e}"))
}

/// `write = true` → ghi file; `false` → so với file đang có, lệch thì `Err`.
fn emit(rel: &str, content: &str, write: bool) -> Result<(), String> {
    if write {
        if let Some(parent) = Path::new(rel).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir `{}`: {e}", parent.display()))?;
        }
        fs::write(rel, content).map_err(|e| format!("ghi `{rel}`: {e}"))?;
        println!("  gen  {rel} ({} bytes)", content.len());
        Ok(())
    } else {
        let cur = fs::read_to_string(rel).map_err(|e| {
            format!("không đọc được `{rel}` để so sánh: {e} (chạy `cargo xtask gen-tables`)")
        })?;
        if normalize(&cur) == normalize(content) {
            println!("  ok   {rel}");
            Ok(())
        } else {
            Err(format!(
                "`{rel}` LỆCH với data/tables — chạy `cargo xtask gen-tables`"
            ))
        }
    }
}

/// So sánh bỏ khác biệt `\r\n` vs `\n` (Windows checkout) — không báo lệch giả.
fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Đưa code sinh ra qua `rustfmt` để `cargo fmt --check` (gate CI) không đánh đỏ
/// file GENERATED. Không có `rustfmt` → **báo lỗi** (không im lặng ghi file lệch chuẩn).
fn format_rust(src: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!(
                "không chạy được `rustfmt`: {e} (cài component: `rustup component add rustfmt`)"
            )
        })?;
    child
        .stdin
        .take()
        .ok_or("rustfmt: không mở được stdin")?
        .write_all(src.as_bytes())
        .map_err(|e| format!("rustfmt: ghi stdin lỗi: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("rustfmt: chờ kết quả lỗi: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "rustfmt thất bại: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("rustfmt trả UTF-8 lỗi: {e}"))
}

/// Sinh cả 2 file Rust từ data. `write=false` → chỉ kiểm tra (gate CI).
fn gen_tables(write: bool) -> Result<(), String> {
    println!(
        "{} `data/tables/*.toml` → `core/`",
        if write {
            "Đang sinh"
        } else {
            "Đang kiểm tra"
        }
    );
    let vowels_src = read(VOWELS_TOML)?;
    let (vowels, digest_v) = parse_vowels(&vowels_src)?;
    emit(
        VOWELS_OUT,
        &format_rust(&render_vowels(&vowels, digest_v))?,
        write,
    )?;

    let mut methods = Vec::with_capacity(METHODS_TOML.len());
    let mut methods_src = String::new();
    for rel in METHODS_TOML {
        let src = read(rel)?;
        methods_src.push_str(&src);
        methods.push(parse_method(&vowels, &src, rel)?);
    }
    let keys = render_keys(
        &vowels,
        &methods,
        digest_v,
        toml::digest(methods_src.as_bytes()),
    );
    emit(KEYS_OUT, &format_rust(&keys)?, write)?;
    println!("  xong.");
    Ok(())
}

/// Một dòng bảng âm đã kiểm tra (tên + base + 6 dạng).
struct Vowel {
    name: String,
    base: char,
    forms: [char; 6],
}

fn one_char(s: &str) -> Result<char, String> {
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(c), None) => Ok(c),
        _ => Err(format!("`{s}` phải đúng 1 ký tự")),
    }
}

fn parse_vowels(src: &str) -> Result<(Vec<Vowel>, u64), String> {
    let doc = toml::parse(src)?;
    let order = doc.root_list("order")?;
    let rows = doc.array("vowel")?;
    if order.len() != rows.len() {
        return Err(format!(
            "`order` có {} mục nhưng có {} [[vowel]] — phải khớp",
            order.len(),
            rows.len()
        ));
    }
    let tones = doc.root_list("tones")?;
    if tones.len() != 6 {
        return Err(format!(
            "`tones` phải có đúng 6 phần tử, thấy {}",
            tones.len()
        ));
    }
    let mut out = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let name = row.str("name")?;
        if name != order[i] {
            return Err(format!(
                "[[vowel]] thứ {i} tên `{name}` nhưng `order[{i}]` = `{}`",
                order[i]
            ));
        }
        let base =
            one_char(&row.str("base")?).map_err(|e| format!("[[vowel]] {name}: base — {e}"))?;
        let form_strs = row.str_list("forms")?;
        if form_strs.len() != 6 {
            return Err(format!(
                "[[vowel]] {name}: forms phải có 6 dạng, thấy {}",
                form_strs.len()
            ));
        }
        let mut forms = [' '; 6];
        for (j, f) in form_strs.iter().enumerate() {
            forms[j] = one_char(f).map_err(|e| format!("[[vowel]] {name}: forms[{j}] — {e}"))?;
        }
        if forms[0] != base {
            return Err(format!(
                "[[vowel]] {name}: forms[0] phải bằng base ({} ≠ {})",
                forms[0], base
            ));
        }
        out.push(Vowel { name, base, forms });
    }
    // 72 ký tự khác nhau — `locate()` quét tuyến tính bảng này, trùng là tra sai.
    let mut seen = std::collections::HashSet::new();
    for v in &out {
        for f in v.forms {
            if !seen.insert(f) {
                return Err(format!("ký tự `{f}` lặp trong bảng (entry {})", v.name));
            }
        }
    }
    if seen.len() != out.len() * 6 {
        return Err(format!("bảng phải có {} ký tự khác nhau", out.len() * 6));
    }
    Ok((out, toml::digest(src.as_bytes())))
}

// ---------------------------------------------------------------- render vowels

/// `core/src/transform/vowel_table_generated.rs` — bảng 72 âm + hằng index.
fn render_vowels(vowels: &[Vowel], digest: u64) -> String {
    let mut s = banner();
    s.push_str(&format!(
        "//! Nguồn: `{VOWELS_TOML}` (digest FNV-1a 64 = `0x{digest:016x}`).\n\
         //!\n\
         //! Index âm trong bảng là **hợp đồng** với `transform::undo` (`mark_vowel`,\n\
         //! `mark_horn`) — đổi thứ tự trong `.toml` là đổi hành vi gõ.\n\n"
    ));
    s.push_str("use crate::transform::vowel_table::VowelEntry;\n\n");
    for (i, v) in vowels.iter().enumerate() {
        let comment = match v.name.as_str() {
            "A_BREVE" => " // ă",
            "A_CIRC" => " // â",
            "E_CIRC" => " // ê",
            "O_CIRC" => " // ô",
            "O_HOOK" => " // ơ",
            "U_HOOK" => " // ư",
            _ => "",
        };
        s.push_str(&format!("pub const {}: usize = {i};{comment}\n", v.name));
    }
    s.push_str(&format!(
        "\npub const VOWEL_ENTRY_COUNT: usize = {};\n",
        vowels.len()
    ));
    s.push_str(
        "/// Số dạng mỗi âm: 0 = không dấu, 1..=5 = sắc · huyền · hỏi · ngã · nặng.\n\
         pub const TONES_PER_VOWEL: usize = 6;\n\n",
    );
    s.push_str("pub const VOWELS: [VowelEntry; VOWEL_ENTRY_COUNT] = [\n");
    for v in vowels {
        s.push_str("    VowelEntry {\n");
        s.push_str(&format!("        base: '{}',\n", v.base));
        s.push_str(&format!(
            "        forms: [{},],\n",
            v.forms
                .iter()
                .map(|c| format!("'{c}'"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str("    },\n");
    }
    s.push_str("];\n");
    s
}

// ----------------------------------------------------------------- helpers keys

/// Một cặp (key, âm gốc, âm đích) đã tra cứu được index âm.
struct Map {
    key: char,
    from: usize,
    from_name: String,
    to: usize,
    to_name: String,
}

/// Rust char literal an toàn cho mọi ký tự (VIQR có key `'` và `\`).
fn rust_char(c: char) -> String {
    match c {
        '\'' => "'\\''".to_string(),
        '\\' => "'\\\\'".to_string(),
        '\n' => "'\\n'".to_string(),
        '\r' => "'\\r'".to_string(),
        '\t' => "'\\t'".to_string(),
        c if (c as u32) < 0x20 || c as u32 == 0x7f => format!("'\\u{{{:x}}}'", c as u32),
        c => format!("'{c}'"),
    }
}

fn index_of(vowels: &[Vowel], name: &str) -> Result<usize, String> {
    vowels
        .iter()
        .position(|v| v.name == name)
        .ok_or_else(|| format!("tên âm `{name}` không có trong `{VOWELS_TOML}`"))
}

fn key_char(raw: &str, ctx: &str) -> Result<char, String> {
    one_char(raw).map_err(|e| format!("{ctx}: key — {e}"))
}

fn parse_maps(
    doc: &toml::Doc,
    array: &str,
    vowels: &[Vowel],
    ctx: &str,
) -> Result<Vec<Map>, String> {
    let mut out = Vec::new();
    for row in doc.array_opt(array) {
        let key = key_char(&row.str("key")?, ctx)?;
        let from = row.str_list("from")?;
        let to = row.str_list("to")?;
        if from.len() != to.len() || from.is_empty() {
            return Err(format!(
                "{ctx}: `from` ({}) và `to` ({}) phải cùng độ dài, khác rỗng",
                from.len(),
                to.len()
            ));
        }
        for (f, t) in from.iter().zip(to.iter()) {
            out.push(Map {
                key,
                from: index_of(vowels, f)?,
                from_name: f.clone(),
                to: index_of(vowels, t)?,
                to_name: t.clone(),
            });
        }
    }
    Ok(out)
}

fn render_list(maps: &[Map], vowels: &[Vowel], indent: &str) -> String {
    if maps.is_empty() {
        return format!("{indent}[]");
    }
    let mut s = format!("{indent}[\n");
    for m in maps {
        // Dùng TÊN hằng âm (A/A_CIRC…) thay vì số: đọc bảng là hiểu ngay, và import
        // trong file sinh ra luôn được dùng (không có import thừa).
        s.push_str(&format!(
            "{indent}    ({}, {}, {}), // {} → {}\n",
            rust_char(m.key),
            m.from_name,
            m.to_name,
            vowels[m.from].base,
            vowels[m.to].base
        ));
    }
    s.push_str(&format!("{indent}]"));
    s
}

// ----------------------------------------------------------------- render keys

/// Dữ liệu 1 kiểu gõ đã chuẩn hoá từ `.toml`.
struct Method {
    name: String,
    display: String,
    w_marker: bool,
    tone: Vec<char>,
    stroke_key: char,
    stroke_double: bool,
    remove_marks_key: Option<char>,
    tone_remove_key: Option<char>,
    circumflex: Vec<Map>,
    horn: Vec<Map>,
    breve: Vec<Map>,
}

fn parse_method(vowels: &[Vowel], src: &str, rel: &str) -> Result<Method, String> {
    let doc = toml::parse(src)?;
    let m = doc.table("method")?;
    let tone_strs = m.str_list("tone")?;
    if tone_strs.len() != 5 {
        return Err(format!(
            "{rel}: `tone` phải có 5 phím (tone 1..=5), thấy {}",
            tone_strs.len()
        ));
    }
    let mut tone = Vec::with_capacity(5);
    for t in &tone_strs {
        tone.push(key_char(t, &format!("{rel}: tone"))?);
    }
    let remove = m.str("remove_marks_key")?;
    let tone_remove = m.str("tone_remove_key")?;
    Ok(Method {
        name: m.str("name")?,
        display: m.str("display")?,
        w_marker: m.bool("w_marker")?,
        tone,
        stroke_key: key_char(&m.str("stroke_key")?, &format!("{rel}: stroke_key"))?,
        stroke_double: m.bool("stroke_double")?,
        remove_marks_key: if remove.is_empty() {
            None
        } else {
            Some(key_char(&remove, &format!("{rel}: remove_marks_key"))?)
        },
        tone_remove_key: if tone_remove.is_empty() {
            None
        } else {
            Some(key_char(&tone_remove, &format!("{rel}: tone_remove_key"))?)
        },
        circumflex: parse_maps(&doc, "circumflex", vowels, rel)?,
        horn: parse_maps(&doc, "horn", vowels, rel)?,
        breve: parse_maps(&doc, "breve", vowels, rel)?,
    })
}

/// `core/src/method/keys_generated.rs` — bảng phím của 4 kiểu gõ.
fn render_keys(vowels: &[Vowel], methods: &[Method], digest_v: u64, digest_all: u64) -> String {
    let mut s = banner();
    s.push_str(&format!(
        "//! Nguồn: `data/tables/{{telex,simple_telex,vni,viqr}}.toml`\n\
         //!   (digest FNV-1a 64 = `0x{digest_all:016x}`; bảng âm = `0x{digest_v:016x}`).\n\
         //!\n\
         //! Mỗi kiểu gõ là 1 `mod`. Hành vi **thuật toán** (undo marker, cụm `uo`, `iet`…)\n\
         //! vẫn nằm trong `method/telex.rs`, `vni.rs`, `viqr.rs` — bảng ở đây chỉ mô tả phần bảng.\n\n"
    ));
    s.push('\n');

    for m in methods {
        s.push_str(&format!(
            "\n/// Kiểu gõ **{}** — xem `data/tables/{}.toml`.\npub mod {} {{\n",
            m.display, m.name, m.name
        ));
        // Chỉ import đúng những hằng âm mà bảng của kiểu gõ này dùng — file sinh ra
        // phải sạch warning, nếu không `clippy -D warnings` sẽ đỏ.
        let mut used: Vec<&str> = Vec::new();
        for map in m.circumflex.iter().chain(&m.horn).chain(&m.breve) {
            for n in [&map.from_name, &map.to_name] {
                if !used.contains(&n.as_str()) {
                    used.push(n.as_str());
                }
            }
        }
        used.sort_by_key(|n| {
            vowels
                .iter()
                .position(|v| v.name == *n)
                .unwrap_or(usize::MAX)
        });
        if !used.is_empty() {
            s.push_str("    use crate::transform::vowel_table_generated::{\n");
            for n in &used {
                s.push_str(&format!("        {n},\n"));
            }
            s.push_str("    };\n\n");
        }
        s.push_str("    /// Key dấu thanh: index 0 → tone 1 (sắc) … index 4 → tone 5 (nặng).\n");
        s.push_str(&format!(
            "    pub const TONE_KEYS: [char; 5] = [{}];\n",
            m.tone
                .iter()
                .map(|&c| rust_char(c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str("    /// Key dấu thanh → index tone (index 0..=4 → tone 1..=5).\n");
        s.push_str("    pub fn tone_of_key(c: char) -> Option<usize> {\n");
        s.push_str("        TONE_KEYS.iter().position(|&k| k == c).map(|i| i + 1)\n");
        s.push_str("    }\n");
        s.push_str("    /// (key, âm gốc, âm đích) — mũ.\n");
        s.push_str(&format!(
            "    pub const CIRCUMFLEX: [(char, usize, usize); {}] = {};\n",
            m.circumflex.len(),
            render_list(&m.circumflex, vowels, "    ")
        ));
        s.push_str("    /// (key, âm gốc, âm đích) — sừng qua marker.\n");
        s.push_str(&format!(
            "    pub const HORN: [(char, usize, usize); {}] = {};\n",
            m.horn.len(),
            render_list(&m.horn, vowels, "    ")
        ));
        s.push_str("    /// (key, âm gốc, âm đích) — breve qua marker.\n");
        s.push_str(&format!(
            "    pub const BREVE: [(char, usize, usize); {}] = {};\n",
            m.breve.len(),
            render_list(&m.breve, vowels, "    ")
        ));
        s.push_str(&format!(
            "    /// Key tạo `đ`. `STROKE_DOUBLE = true` → gõ **hai lần** key (Telex/VIQR `dd`).\n\
             \x20   pub const STROKE_KEY: char = {};\n\
             \x20   pub const STROKE_DOUBLE: bool = {};\n",
            rust_char(m.stroke_key),
            m.stroke_double
        ));
        match m.remove_marks_key {
            Some(k) => s.push_str(&format!(
                "    /// Key xoá toàn bộ dấu của từ (VNI `0`).\n    pub const REMOVE_MARKS_KEY: Option<char> = Some({});\n",
                rust_char(k)
            )),
            None => s.push_str("    /// Kiểu gõ này không có phím xoá toàn bộ dấu.\n    pub const REMOVE_MARKS_KEY: Option<char> = None;\n"),
        }
        match m.tone_remove_key {
            Some(k) => s.push_str(&format!(
                "    /// Key gỡ **dấu thanh** của từ (Telex `z`); từ chưa có dấu → chữ thường.\n    pub const TONE_REMOVE_KEY: Option<char> = Some({});\n",
                rust_char(k)
            )),
            None => s.push_str("    /// Kiểu gõ này không có phím gỡ riêng dấu thanh.\n    pub const TONE_REMOVE_KEY: Option<char> = None;\n"),
        }
        s.push_str(&format!(
            "    /// `true` = `w` là marker sừng (Telex); `false` = `w` là chữ thường (Simple Telex).\n    pub const W_MARKER: bool = {};\n",
            m.w_marker
        ));
        s.push_str("    /// Key này có phải **marker** (một phần của từ, không phải ranh giới)?\n");
        s.push_str("    pub fn is_marker(c: char) -> bool {\n");
        s.push_str("        tone_of_key(c).is_some()\n");
        s.push_str("            || CIRCUMFLEX.iter().any(|&(k, _, _)| k == c)\n");
        s.push_str("            || HORN.iter().any(|&(k, _, _)| k == c)\n");
        s.push_str("            || BREVE.iter().any(|&(k, _, _)| k == c)\n");
        s.push_str("            || REMOVE_MARKS_KEY == Some(c)\n");
        s.push_str("            || TONE_REMOVE_KEY == Some(c)\n");
        s.push_str(
            "            // `stroke` 1 phím (VNI `9`) là marker; `dd` thì không (chữ cái vốn\n",
        );
        s.push_str("            // đã thuộc từ) — nên chỉ nhận khi STROKE_DOUBLE = false.\n");
        s.push_str("            || (!STROKE_DOUBLE && STROKE_KEY == c)\n");
        s.push_str("            || (W_MARKER && c == 'w')\n");
        s.push_str("    }\n");
        s.push_str("}\n");
    }
    s
}
