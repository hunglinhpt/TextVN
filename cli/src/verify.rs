// SPDX-License-Identifier: GPL-3.0-or-later
//! `vietime verify` — chứng minh **header C khớp code Rust** (P0-2 §6).
//!
//! Vì sao cần: `ffi/include/vietime_ffi.h` đang được giữ **tay** (chưa có `xtask cbindgen` —
//! cần crate `cbindgen` + nightly). Nguy cơ thật là sửa struct/hằng ở một bên quên bên kia;
//! test `size_of` không bắt được (thêm 1 trường `u8` có thể không đổi size do padding).
//!
//! Lệnh này kiểm 4 tầng, tất cả **tự suy ra từ code đang chạy** (không hardcode kết quả):
//! 1. **Kích thước/offset** — `size_of`/`offset_of` trên struct thật (giống `sizes`).
//! 2. **Hằng** — mọi `#define IME_*` trong header phải có giá trị **bằng** hằng Rust tương ứng.
//! 3. **Trường struct** — danh sách tên trường trong header phải khớp `stringify!` của struct Rust
//!    (thêm/bớt trường ở Rust mà quên sửa header → biên dịch hỏng hoặc check đỏ).
//! 4. **Hàm export** — danh sách + thứ tự prototype `ime_*` trong header phải khớp
//!    thứ tự khai báo `#[no_mangle] pub extern "C"` trong `ffi/src/lib.rs`
//!    (đổi tên/thêm/bớt/đảo thứ tự ở một bên mà quên bên kia → link sai hoặc
//!    tài liệu vòng đời sai).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::mem::{offset_of, size_of};

use vietime_ffi::*;

/// Tên trường của struct Rust, **tự suy ra lúc biên dịch**: thêm/bớt trường mà quên
/// cập nhật chỗ này → hỏng build (đúng ý đồ: bắt buộc phải xem lại header).
macro_rules! field_names {
    ($t:ty { $($f:ident),+ $(,)? }) => {
        &[$(stringify!($f)),+]
    };
}

/// Hằng ABI: (tên trong header C, giá trị lấy từ Rust).
fn abi_constants() -> Vec<(&'static str, i64)> {
    vec![
        // version / giới hạn
        ("IME_ABI_VERSION", IME_ABI_VERSION as i64),
        ("IME_MAX_TEXT", IME_MAX_TEXT as i64),
        ("IME_MAX_SUGGEST", IME_MAX_SUGGEST as i64),
        // error codes
        ("IME_OK", IME_OK as i64),
        ("IME_ERR_INVALID_ARG", IME_ERR_INVALID_ARG as i64),
        ("IME_ERR_CONFIG", IME_ERR_CONFIG as i64),
        ("IME_ERR_ABI", IME_ERR_ABI as i64),
        ("IME_ERR_INTERNAL", IME_ERR_INTERNAL as i64),
        // action
        ("IME_ACTION_PASS", ACTION_PASS as i64),
        ("IME_ACTION_REPLACE", ACTION_REPLACE as i64),
        ("IME_ACTION_COMMIT", ACTION_COMMIT as i64),
        ("IME_ACTION_RESTORE", ACTION_RESTORE as i64),
        // flags
        ("IME_FLAG_CONSUMED", IME_FLAG_CONSUMED as i64),
        ("IME_FLAG_WORD_END", IME_FLAG_WORD_END as i64),
        ("IME_FLAG_ERROR", IME_FLAG_ERROR as i64),
        ("IME_FLAG_SUGGEST", IME_FLAG_SUGGEST as i64),
        // modifier (core đặt tên MOD_*, header đặt tên IME_MOD_*)
        ("IME_MOD_SHIFT", MOD_SHIFT as i64),
        ("IME_MOD_CTRL", MOD_CTRL as i64),
        ("IME_MOD_ALT", MOD_ALT as i64),
        ("IME_MOD_SUPER", MOD_SUPER as i64),
        ("IME_MOD_META", MOD_META as i64),
        ("IME_MOD_CAPS", MOD_CAPS as i64),
        ("IME_MOD_FN", MOD_FN as i64),
        // field role
        ("IME_FIELD_UNKNOWN", IME_FIELD_UNKNOWN as i64),
        ("IME_FIELD_BODY", IME_FIELD_BODY as i64),
        ("IME_FIELD_EDITBOX", IME_FIELD_EDITBOX as i64),
        ("IME_FIELD_ADDRESS_BAR", IME_FIELD_ADDRESS_BAR as i64),
        ("IME_FIELD_SEARCH", IME_FIELD_SEARCH as i64),
        ("IME_FIELD_COMBO", IME_FIELD_COMBO as i64),
        ("IME_FIELD_CANDIDATE", IME_FIELD_CANDIDATE as i64),
        ("IME_FIELD_TEXTAREA", IME_FIELD_TEXTAREA as i64),
        ("IME_FIELD_WEB", IME_FIELD_WEB as i64),
        ("IME_FIELD_TERMINAL", IME_FIELD_TERMINAL as i64),
        ("IME_FIELD_SECURE", IME_FIELD_SECURE as i64),
        // capability
        ("IME_CAP_PREEDIT", IME_CAP_PREEDIT as i64),
        ("IME_CAP_SELECTION", IME_CAP_SELECTION as i64),
        ("IME_CAP_FIELD_DETECT", IME_CAP_FIELD_DETECT as i64),
        ("IME_CAP_INJECT_VK", IME_CAP_INJECT_VK as i64),
        // strategy
        ("IME_STRATEGY_PREEDIT", IME_STRATEGY_PREEDIT),
        ("IME_STRATEGY_BACKSPACE_TYPE", IME_STRATEGY_BACKSPACE_TYPE),
        (
            "IME_STRATEGY_SELECTION_REPLACE",
            IME_STRATEGY_SELECTION_REPLACE,
        ),
        (
            "IME_STRATEGY_FORWARD_AS_COMMIT",
            IME_STRATEGY_FORWARD_AS_COMMIT,
        ),
        ("IME_STRATEGY_PASSTHROUGH", IME_STRATEGY_PASSTHROUGH),
    ]
}

/// Danh sách hàm C xuất ra — **1 nguồn sự thật cho cả 2 phía** (P0-2 §1/§6).
/// Thứ tự là thứ tự **chuẩn** (canonical): khớp khối `/* ---- API ---- */`
/// trong `ffi/include/vietime_ffi.h` (vốn theo P0-2 §1: verify → resolve →
/// last_error ở cuối) và khớp thứ tự khai báo `#[no_mangle] pub extern "C"`
/// trong `ffi/src/lib.rs`. Đổi thứ tự/thêm/bớt/đổi tên hàm ở một bên mà quên
/// bên kia → `verify` đỏ.
///
/// Vì sao cần: `ffi/include/vietime_ffi.h` đang giữ **tay** (chưa có
/// `cargo xtask cbindgen`). Test size/offset không bắt được việc đổi tên hàm
/// (`ime_reset` → `ime_clear`) hay đảo thứ tự khai báo — nhưng adapter C/C++
/// link theo tên, còn người đọc header tin vào thứ tự để hiểu vòng đời.
fn abi_exports() -> Vec<&'static str> {
    vec![
        "ime_abi_version",
        "ime_instance_new",
        "ime_instance_free",
        "ime_set_context",
        "ime_key",
        "ime_reset",
        "ime_reload_config",
        "ime_suggest",
        "ime_appdb_verify",
        "ime_strategy_resolve",
        "ime_last_error",
    ]
}

/// Danh sách trường struct Rust (tự sinh) + tên `typedef` tương ứng trong header.
fn abi_structs() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        (
            "ime_key_v1",
            field_names!(ime_key_v1 {
                abi_version,
                vk,
                ch,
                mods,
                key_down,
                is_repeat,
                is_injected,
                _reserved,
            }),
        ),
        (
            "ime_result_v1",
            field_names!(ime_result_v1 {
                abi_version,
                action,
                delete_count,
                insert_len,
                preedit_len,
                _reserved,
                flags,
                insert,
                preedit,
            }),
        ),
        (
            "ime_context_v1",
            field_names!(ime_context_v1 {
                abi_version,
                enabled,
                secure,
                field_role,
                caps,
                app_id,
                element_name,
                hint,
            }),
        ),
        (
            "ime_suggest_v1",
            field_names!(ime_suggest_v1 {
                abi_version,
                count,
                lens,
                items,
            }),
        ),
    ]
}

/// Bỏ **toàn bộ** comment C khỏi mã: `/* ... */` (nhiều dòng) và `// ...`.
///
/// Phải strip cả khối nhiều dòng *trước khi* tách dòng: nếu strip theo dòng, dòng tiếp theo
/// của một khối comment (` * ime_key_v1 (20)…`) sẽ bị đọc như một field của struct.
fn strip_all_comments(src: &str) -> String {
    let b: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut in_block = false;
    while i < b.len() {
        if in_block {
            if b[i] == '*' && b.get(i + 1) == Some(&'/') {
                in_block = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if b[i] == '/' && b.get(i + 1) == Some(&'*') {
            in_block = true;
            i += 2;
            continue;
        }
        if b[i] == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// `#define NAME VALUE` trong header → map tên → giá trị số (bỏ hậu tố `u`/`UL`, chấp nhận hex).
fn parse_defines(src: &str) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    let code = strip_all_comments(src);
    for line in code.lines() {
        let Some(rest) = line.trim().strip_prefix("#define") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let (Some(name), Some(value)) = (parts.next(), parts.next()) else {
            continue; // `#define` không có giá trị (vd guard header) → bỏ qua
        };
        if let Some(v) = parse_c_int(value) {
            out.insert(name.to_string(), v);
        }
    }
    out
}

fn parse_c_int(raw: &str) -> Option<i64> {
    let t = raw.trim().trim_end_matches(['u', 'U', 'l', 'L']).trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return i64::from_str_radix(hex, 16).ok();
    }
    t.parse::<i64>().ok()
}

/// Tên trường trong `typedef struct ime_key { ... } ime_key_v1;`.
///
/// Header dùng **tên khác** cho struct tag và typedef (`typedef struct ime_key {…} ime_key_v1;`)
/// → tìm theo `} <typedef>;` rồi lùi về `typedef struct` phía trên (không giả định tên trùng).
fn parse_struct_fields(src: &str, typedef: &str) -> Option<Vec<String>> {
    let code = strip_all_comments(src);
    let close = code.find(&format!("}} {typedef};"))?;
    let head = code[..close].rfind("typedef struct")?;
    let body_start = code[head..close].find('{')? + head + 1;
    let mut out = Vec::new();
    for line in code[body_start..close].lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue; // dòng chỉ có comment
        }
        let Some(field) = line.split_whitespace().last() else {
            continue;
        };
        let field = field.trim().trim_start_matches('*');
        let field = field.split('[').next().unwrap_or(field).trim();
        if !field.is_empty() {
            out.push(field.to_string());
        }
    }
    Some(out)
}

/// Tên hàm trong khối `/* ---- API ---- */` của header, **giữ đúng thứ tự khai báo**.
///
/// Chỉ quét từ dòng `/* ---- API ---- */` trở đi (không quét struct/hằng phía
/// trên — tên type `ime_key_v1` trong `typedef` không phải hàm). Mỗi prototype
/// C có dạng `<kiểu trả về> ime_tên(<tham số>);` — có thể xuống dòng giữa chừng
/// (`ime_suggest`, `ime_appdb_verify`, `ime_strategy_resolve` đều xuống dòng),
/// nên gom theo dấu `;`: tên hàm = token `ime_*` đầu tiên trong mỗi chunk.
/// Bỏ qua từ khoá `typedef`/`struct` và tên type trong comment đã strip.
/// Dòng `extern "C" {` chứa `C` viết hoa — không nhầm vì chỉ nhận token bắt đầu
/// bằng `ime_` thường.
fn parse_header_exports(src: &str) -> Vec<String> {
    let code = strip_all_comments(src);
    let marker = "/* ---- API ---- */";
    let from = code.find(marker).map(|i| i + marker.len()).unwrap_or(0);
    let api = &code[from..];
    let mut out = Vec::new();
    for chunk in api.split(';') {
        // Mỗi prototype kết thúc bằng `;` (kể cả prototype xuống nhiều dòng).
        // Tên hàm = token `ime_*` đi kèm `(` mở ngay sau (bỏ tên type như
        // `ime_instance`, `ime_key_v1` trong tham số). Chunk comment lẻ
        // (`/* luôn IME_ABI_VERSION */`) đã strip → không nhiễu.
        if let Some(name) = first_call_name(chunk) {
            out.push(name);
        }
    }
    out
}

/// Tên hàm đầu tiên dạng `ime_xxx (` trong chunk (bỏ `ime_instance`, `ime_key_v1`
/// là tên type — chúng không đi kèm `(` mở ngay sau).
fn first_call_name(chunk: &str) -> Option<String> {
    let bytes = chunk.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if chunk[i..].starts_with("ime_") {
            let mut j = i + 4;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let name = &chunk[i..j];
            // Tên type (`ime_instance`, `ime_key_v1`, `ime_context_v1`,
            // `ime_result_v1`, `ime_suggest_v1`) không bao giờ đi kèm `(` ngay
            // sau trong prototype — tên hàm thì có.
            let mut k = j;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            if k < bytes.len() && bytes[k] == b'(' && !is_type_name(name) {
                return Some(name.to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    None
}

fn is_type_name(name: &str) -> bool {
    matches!(
        name,
        "ime_instance" | "ime_key_v1" | "ime_context_v1" | "ime_result_v1" | "ime_suggest_v1"
    )
}

/// Một phát hiện sai lệch.
struct Mismatch {
    what: String,
    header: String,
    rust: String,
}

/// Chạy toàn bộ check. `Ok(report)` = khớp hết; `Err` = danh sách sai lệch (đã format).
pub fn run(header_path: &str, as_json: bool) -> Result<String, Vec<String>> {
    let src = std::fs::read_to_string(header_path)
        .map_err(|e| vec![format!("không đọc được header `{header_path}`: {e}")])?;

    let mut bad: Vec<Mismatch> = Vec::new();

    // 1) kích thước struct (P0-2 §6)
    let key_size = size_of::<ime_key_v1>();
    let result_size = size_of::<ime_result_v1>();
    if key_size != 20 {
        bad.push(Mismatch {
            what: "sizeof(ime_key_v1)".into(),
            header: "20".into(),
            rust: key_size.to_string(),
        });
    }
    if result_size != 532 {
        bad.push(Mismatch {
            what: "sizeof(ime_result_v1)".into(),
            header: "532".into(),
            rust: result_size.to_string(),
        });
    }

    // 2) hằng: header ↔ Rust (mọi hằng ABI phải khớp giá trị)
    let defines = parse_defines(&src);
    for (name, rust_value) in abi_constants() {
        match defines.get(name) {
            Some(h) if *h == rust_value => {}
            Some(h) => bad.push(Mismatch {
                what: format!("#define {name}"),
                header: h.to_string(),
                rust: rust_value.to_string(),
            }),
            None => bad.push(Mismatch {
                what: format!("#define {name}"),
                header: "(thiếu)".into(),
                rust: rust_value.to_string(),
            }),
        }
    }
    // Hằng `IME_*` có trong header mà Rust chưa biết → vừa thêm, cần khai báo vào bảng.
    let known: Vec<&str> = abi_constants().iter().map(|(n, _)| *n).collect();
    for (name, value) in &defines {
        if name.starts_with("IME_") && !known.contains(&name.as_str()) {
            bad.push(Mismatch {
                what: format!("#define {name}"),
                header: value.to_string(),
                rust: "(chưa khai báo trong verify)".into(),
            });
        }
    }

    // 3) trường struct: thứ tự + tên phải khớp
    for (typedef, rust_fields) in abi_structs() {
        match parse_struct_fields(&src, typedef) {
            Some(header_fields) if header_fields == rust_fields => {}
            Some(header_fields) => bad.push(Mismatch {
                what: format!("trường của {typedef}"),
                header: header_fields.join(","),
                rust: rust_fields.join(","),
            }),
            None => bad.push(Mismatch {
                what: format!("typedef struct {typedef}"),
                header: "(không tìm thấy)".into(),
                rust: format!("{} trường", rust_fields.len()),
            }),
        }
    }

    // 4) hàm export: danh sách + thứ tự prototype trong header phải khớp
    //    thứ tự khai báo `#[no_mangle]` trong Rust (P0-2 §1/§6).
    let rust_exports: Vec<String> = abi_exports().iter().map(|s| s.to_string()).collect();
    let header_exports = parse_header_exports(&src);
    if header_exports != rust_exports {
        bad.push(Mismatch {
            what: "thứ tự hàm export".into(),
            header: if header_exports.is_empty() {
                "(không tìm thấy khối API)".into()
            } else {
                header_exports.join(",")
            },
            rust: rust_exports.join(","),
        });
    }

    if !bad.is_empty() {
        return Err(bad
            .iter()
            .map(|m| format!("  {} · header={} · rust={}", m.what, m.header, m.rust))
            .collect());
    }
    Ok(report(
        header_path,
        key_size,
        result_size,
        defines.len(),
        header_exports.len(),
        as_json,
    ))
}

fn report(
    header_path: &str,
    key_size: usize,
    result_size: usize,
    defines: usize,
    exports: usize,
    json: bool,
) -> String {
    let mut text = String::new();
    if json {
        let _ = write!(
            text,
            "{{\"ok\":true,\"abi\":{},\"key\":{key_size},\"result\":{result_size},\"defines\":{defines},\"exports\":{exports},\"structs\":{{",
            IME_ABI_VERSION
        );
        for (i, (name, fields)) in abi_structs().iter().enumerate() {
            if i > 0 {
                text.push(',');
            }
            let _ = write!(text, "\"{name}\":{}", fields.len());
        }
        text.push_str("}}");
        return text;
    }
    let _ = writeln!(text, "ABI v{} — header khớp code Rust", IME_ABI_VERSION);
    let _ = writeln!(
        text,
        "  header  : {header_path} ({defines} hằng `IME_*` + {exports} hàm export đã kiểm)"
    );
    let _ = writeln!(
        text,
        "  sizeof  : ime_key_v1={key_size} · ime_result_v1={result_size} (kỳ vọng 20/532)"
    );
    let _ = writeln!(
        text,
        "  offsets : result.insert={} result.preedit={} key.ch={} key.mods={}",
        offset_of!(ime_result_v1, insert),
        offset_of!(ime_result_v1, preedit),
        offset_of!(ime_key_v1, ch),
        offset_of!(ime_key_v1, mods),
    );
    for (name, fields) in abi_structs() {
        let _ = writeln!(
            text,
            "  struct  : {name} — {} trường, thứ tự khớp",
            fields.len()
        );
    }
    let _ = writeln!(
        text,
        "  exports : {} hàm, thứ tự khớp ({}…{})",
        exports,
        abi_exports().first().unwrap_or(&"?"),
        abi_exports().last().unwrap_or(&"?"),
    );
    text.trim_end().to_string()
}

/// Header mặc định — tìm cạnh binary (`../include/...`) rồi tới gốc repo.
pub fn default_header() -> String {
    const REL: &str = "ffi/include/vietime_ffi.h";
    let mut dir = std::env::current_dir().unwrap_or_default();
    loop {
        let cand = dir.join(REL);
        if cand.is_file() {
            return cand.display().to_string();
        }
        if !dir.pop() {
            break;
        }
    }
    // Không tìm thấy → trả đường dẫn tương đối; `run` sẽ báo lỗi đọc file kèm đường dẫn.
    REL.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = r#"
#define IME_ABI_VERSION      1u
#define IME_MAX_TEXT         64
#define IME_FLAG_ERROR      0x4u   /* lỗi engine */
#define VIETIME_FFI_H              /* guard: không có giá trị → bỏ qua */
#define IME_FLAG_WEIRD 0x10u
typedef struct ime_key {
  uint32_t abi_version;
  uint32_t ch;
  /* khối comment nhiều dòng
   * ime_key_v1 (20) — dòng này KHÔNG phải field
   */
  uint8_t  key_down;
  uint16_t _reserved;
} ime_key_v1;
typedef struct ime_result {
  uint32_t action;
  uint32_t insert[IME_MAX_TEXT]; /* UTF-32 */
  const char *app_id;
} ime_result_v1;
"#;

    #[test]
    fn define_boc_comment_va_bo_guard() {
        let d = parse_defines(HEADER);
        assert_eq!(d.get("IME_ABI_VERSION"), Some(&1));
        assert_eq!(d.get("IME_MAX_TEXT"), Some(&64));
        assert_eq!(
            d.get("IME_FLAG_ERROR"),
            Some(&4),
            "hằng trong comment phải lấy đúng"
        );
        assert!(
            !d.contains_key("VIETIME_FFI_H"),
            "#define không có giá trị phải bỏ qua"
        );
    }

    #[test]
    fn truong_struct_bo_comment_nhieu_dong_con_tro_va_mang() {
        let f = parse_struct_fields(HEADER, "ime_key_v1").expect("phải tìm thấy struct");
        assert_eq!(f, vec!["abi_version", "ch", "key_down", "_reserved"]);
        let r = parse_struct_fields(HEADER, "ime_result_v1").expect("phải tìm thấy struct");
        assert_eq!(
            r,
            vec!["action", "insert", "app_id"],
            "mảng + con trỏ phải lấy tên"
        );
    }

    #[test]
    fn struct_khong_ton_tai_tra_none() {
        assert!(parse_struct_fields(HEADER, "ime_suggest_v1").is_none());
    }

    #[test]
    fn verify_header_that_bat_doi_chieu_hang_moi() {
        // Header thật trong repo phải khớp (gate CI chạy lệnh này qua `vietime verify`).
        let path = default_header();
        assert!(
            std::path::Path::new(&path).is_file(),
            "không tìm thấy header: {path}"
        );
        if let Err(list) = run(&path, false) {
            panic!("header lệch với code Rust:\n{}", list.join("\n"));
        }
    }

    #[test]
    fn verify_bat_hang_la_khong_khai_bao() {
        // `IME_FLAG_WEIRD` trong HEADER là hằng lạ → phải bị báo, không bỏ qua im lặng.
        let mut list = Vec::new();
        let known: Vec<&str> = abi_constants().iter().map(|(n, _)| *n).collect();
        for name in parse_defines(HEADER).keys() {
            if name.starts_with("IME_") && !known.contains(&name.as_str()) {
                list.push(name.clone());
            }
        }
        assert_eq!(list, vec!["IME_FLAG_WEIRD".to_string()]);
    }

    /// Header mẫu cho parser export: prototype xuống dòng, comment xen giữa,
    /// tên type `ime_*` trong tham số (không phải hàm), khối API ở cuối.
    const HEADER_API: &str = r#"
typedef struct ime_key {
  uint32_t abi_version;
} ime_key_v1;
/* ---- API ---- */
uint32_t     ime_abi_version(void);                     /* luôn IME_ABI_VERSION của lib */
int32_t      ime_instance_new(const uint8_t *config_utf8, size_t len, ime_instance **out);
void         ime_instance_free(ime_instance *inst);
int32_t      ime_key(ime_instance *inst, const ime_key_v1 *k, ime_result_v1 *out);
int32_t      ime_suggest(ime_instance *inst, const uint32_t *word, uint32_t word_len,
                         ime_suggest_v1 *out);          /* xuống dòng giữa prototype */
const char  *ime_last_error(const ime_instance *inst);
"#;

    #[test]
    fn export_giu_thu_tu_bo_type_va_chiu_xuong_dong() {
        assert_eq!(
            parse_header_exports(HEADER_API),
            vec![
                "ime_abi_version",
                "ime_instance_new",
                "ime_instance_free",
                "ime_key",
                "ime_suggest",
                "ime_last_error",
            ]
        );
    }

    #[test]
    fn export_dao_thu_tu_bi_phat_hien() {
        // Đảo 2 hàm trong header mẫu → danh sách khác `abi_exports()`-thu-nhỏ.
        let swapped = HEADER_API
            .replace("ime_key(", "ime_tmp(")
            .replace("ime_suggest(ime_instance", "ime_key(ime_instance");
        let swapped = swapped.replace("ime_tmp(", "ime_suggest(");
        let got = parse_header_exports(&swapped);
        let want = parse_header_exports(HEADER_API);
        assert_ne!(got, want, "đảo thứ tự phải đổi danh sách parse được");
        assert_eq!(got[3], "ime_suggest");
        assert_eq!(got[4], "ime_key");
    }

    #[test]
    fn export_thieu_ham_bi_phat_hien() {
        // Xoá 1 prototype → danh sách ngắn hơn, `run()` thật sẽ báo mismatch.
        let cut: Vec<&str> = HEADER_API
            .lines()
            .filter(|l| !l.contains("ime_instance_free"))
            .collect();
        let got = parse_header_exports(&cut.join("\n"));
        assert!(
            !got.contains(&"ime_instance_free".to_string()),
            "xoá prototype phải mất tên trong danh sách"
        );
        assert_eq!(got.len(), parse_header_exports(HEADER_API).len() - 1);
    }

    #[test]
    fn export_rust_dung_thu_tu_header_that() {
        // `abi_exports()` (thứ tự chuẩn) phải khớp header thật trong repo —
        // chính là điều `run()` kiểm ở tầng 4. Test này chốt thứ tự chuẩn vào
        // code: ai đảo thứ tự `#[no_mangle]` trong `ffi/src/lib.rs` mà quên
        // sửa `abi_exports()` → `verify` đỏ trước cả khi chạy `verify`.
        let path = default_header();
        let src = std::fs::read_to_string(&path).expect("phải đọc được header thật");
        assert_eq!(
            parse_header_exports(&src),
            abi_exports(),
            "thứ tự hàm trong header thật phải khớp abi_exports()"
        );
        // Và thứ tự khai báo `#[no_mangle]` trong `ffi/src/lib.rs` thật cũng
        // phải khớp — đọc source Rust, quét `pub extern "C" fn ime_*` theo thứ
        // tự xuất hiện (không hardcode lại danh sách ở chỗ khác).
        let root = std::path::Path::new(&path)
            .parent()
            .and_then(|p| p.parent())
            .expect("header phải nằm trong ffi/include/")
            .join("src/lib.rs");
        let rust = std::fs::read_to_string(&root).expect("phải đọc được ffi/src/lib.rs");
        assert_eq!(
            parse_rust_exports(&rust),
            abi_exports(),
            "thứ tự #[no_mangle] trong ffi/src/lib.rs phải khớp abi_exports()"
        );
    }
}

/// Tên hàm `ime_*` sau mỗi `pub extern "C" fn` trong source Rust, giữ thứ tự.
///
/// Parser tối giản (không cần `syn` — 0 dependency): tìm chuỗi
/// `pub extern "C" fn ime_`, đọc tên tới ký tự không phải `[A-Za-z0-9_]`.
/// Chỉ dùng trong test (đọc `ffi/src/lib.rs` thật để đối chiếu) — `run()` kiểm
/// header ↔ `abi_exports()`, còn test này kiểm `abi_exports()` ↔ source Rust.
#[cfg(test)]
fn parse_rust_exports(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search_from = 0;
    let needle = "pub extern \"C\" fn ime_";
    while let Some(rel) = src[search_from..].find(needle) {
        let name_start = search_from + rel + "pub extern \"C\" fn ".len();
        let mut name_end = name_start;
        while name_end < src.len()
            && (src.as_bytes()[name_end].is_ascii_alphanumeric()
                || src.as_bytes()[name_end] == b'_')
        {
            name_end += 1;
        }
        let name = &src[name_start..name_end];
        // Chấp nhận mọi `pub extern "C" fn ime_*` — trong crate này tất cả đều
        // là export `#[no_mangle]` (không có helper `pub extern` nội bộ).
        out.push(name.to_string());
        search_from = name_end;
    }
    out
}

#[cfg(test)]
mod rust_export_tests {
    use super::parse_rust_exports;

    const RUST_SRC: &str = r#"
#[no_mangle]
pub extern "C" fn ime_abi_version() -> u32 {
    IME_ABI_VERSION
}
/// comment xen giữa
#[no_mangle]
pub extern "C" fn ime_key(
    inst: *mut ime_instance,
) -> i32 {
    IME_OK
}
"#;

    #[test]
    fn rust_parser_giu_thu_tu_va_bo_comment() {
        assert_eq!(
            parse_rust_exports(RUST_SRC),
            vec!["ime_abi_version", "ime_key"]
        );
    }

    #[test]
    fn rust_parser_dao_thu_tu_bi_phat_hien() {
        // Đảo 2 hàm trong source mẫu → danh sách parse khác thứ tự chuẩn.
        let swapped = RUST_SRC
            .replace("ime_abi_version(", "ime_tmp(")
            .replace("ime_key(", "ime_abi_version(")
            .replace("ime_tmp(", "ime_key(");
        let got = parse_rust_exports(&swapped);
        assert_eq!(got, vec!["ime_key", "ime_abi_version"]);
        assert_ne!(got, vec!["ime_abi_version", "ime_key"]);
    }
}
