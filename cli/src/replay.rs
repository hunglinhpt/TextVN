// SPDX-License-Identifier: GPL-3.0-or-later
//! `cli/src/replay.rs` — parser corpus `.keys` + simulator (spec: P0-4, bám sát 100%).
//!
//! Exit: 0 = all pass · 1 = có fail · 2 = lỗi dùng/parse (P0-4 §4).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use textvn_session::compose::{ComposePolicy, Composer, EngineStep, KeyKind};
use textvn_session::{Commands, SessionCommand};
use textvn_win_tsf::compose::is_modifier_vk;

use textvn_ffi::{
    ime_context_v1, ime_instance, ime_instance_free, ime_instance_new, ime_key, ime_key_v1,
    ime_reload_config, ime_reset, ime_result_v1, ime_set_context, IME_ABI_VERSION, IME_OK,
};

// ---- lệnh corpus ----

#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Config(Vec<(String, String)>),
    App { app_id: String, field: String },
    Caps(Vec<String>),
    Enabled(bool),
    Secure(bool),
    Type(Vec<char>),
    Key(String),
    Combo(String),
    Expect(String),
    ExpectPreedit(String),
    ExpectAction(String),
    ExpectCursor(usize),
    Reset,
    EngineNew,
    Injected(Vec<char>),
    Mods { add: bool, name: String },
    Note(String),
}

/// Khả năng mặc định của mỗi adapter trong replay (P0-4 §4).
///
/// Lệnh `:caps` trong corpus luôn ghi đè các giá trị này để một ca có thể mô
/// phỏng capability bị thiếu. Preset theo AppDB chưa được suy diễn ở CLI: nó
/// phải được kiểm thử qua resolver AppDB dùng chung, không được nhân bản ở đây.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AdapterProfile {
    default_caps: u32,
    /// SF-3: mô phỏng adapter bằng CHÍNH `textvn_session::compose::Composer` với chính sách OS
    /// (`TSF`: composition do TIP sở hữu; `LINUX`: preedit IBus/Fcitx5) thay vì áp action trực tiếp.
    compose: Option<ComposePolicy>,
}

fn adapter_profile(adapter: &str) -> AdapterProfile {
    const PREEDIT: u32 = 0x1;
    const SELECTION: u32 = 0x2;
    const FIELD_DETECT: u32 = 0x4;
    const INJECT_VK: u32 = 0x8;

    match adapter {
        "headless" => AdapterProfile {
            default_caps: 0,
            compose: None,
        },
        // TSF supports composition and native ranges. `INJECT_VK` is included
        // for the hook fallback, which shares the Windows corpus profile.
        "win" => AdapterProfile {
            default_caps: PREEDIT | SELECTION | FIELD_DETECT | INJECT_VK,
            compose: None,
        },
        // TSF thật: cả từ nằm trong composition, commit ở ranh giới
        // (`textvn_win_tsf::compose`) — cùng kế hoạch mà DLL áp trong edit session.
        "tsf" => AdapterProfile {
            default_caps: PREEDIT | SELECTION | FIELD_DETECT,
            compose: Some(ComposePolicy::TSF),
        },
        // IBus/Fcitx5: preedit + forward key — cùng `Composer` với chính sách LINUX (SF-3),
        // cổng chord/biến đổi phím như `trace_compose.c` (`runner_step_key`).
        "linux" => AdapterProfile {
            default_caps: PREEDIT | SELECTION | FIELD_DETECT,
            compose: Some(ComposePolicy::LINUX),
        },
        // IMK expose preedit and selection when field-detection integration is available.
        "mac" => AdapterProfile {
            default_caps: PREEDIT | SELECTION | FIELD_DETECT,
            compose: None,
        },
        _ => unreachable!("CLI validates --adapter before replay::run"),
    }
}

#[derive(Debug)]
struct ParseError {
    file: String,
    line: usize,
    msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.msg)
    }
}

/// Cắt comment ('#' ngoài chuỗi) rồi trim.
fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    let mut escape = false;
    for (i, c) in line.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_quote => escape = true,
            '"' => in_quote = !in_quote,
            '#' if !in_quote => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Chuỗi trong ngoặc kép + escape `\" \\ \n` (P0-4 §2.4). Lỗi escape lạ → Err.
fn parse_quoted(s: &str) -> Result<String, String> {
    let s = s.trim();
    let mut chars = s.chars();
    if chars.next() != Some('"') {
        return Err("thiếu ngoặc kép mở".into());
    }
    let mut out = String::new();
    let mut escape = false;
    let mut closed = false;
    for c in chars.by_ref() {
        if escape {
            match c {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => return Err(format!("escape không hợp lệ `\\{other}`")),
            }
            escape = false;
            continue;
        }
        match c {
            '\\' => escape = true,
            '"' => {
                closed = true;
                break;
            }
            other => out.push(other),
        }
    }
    if !closed {
        return Err("thiếu ngoặc kép đóng".into());
    }
    if !chars.clone().all(|c| c.is_whitespace()) {
        return Err("sau chuỗi còn ký tự thừa".into());
    }
    Ok(out)
}

/// Tách token, giữ nguyên chuỗi trong ngoặc kép.
fn split_tokens(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut escape = false;
    for c in s.chars() {
        if escape {
            cur.push('\\');
            cur.push(c);
            escape = false;
            continue;
        }
        match c {
            '\\' if in_quote => escape = true,
            '"' => {
                in_quote = !in_quote;
                cur.push(c);
            }
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
            other => cur.push(other),
        }
    }
    if in_quote {
        return Err("ngoặc kép chưa đóng".into());
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    Ok(tokens)
}

/// Key name chuẩn (P0-4 §2.4, case-sensitive) → (VK, ch).
fn key_by_name(name: &str) -> Option<(u32, u32)> {
    let (vk, ch) = match name {
        "Space" => (0x20, ' ' as u32),
        "Enter" => (0x0D, '\n' as u32),
        "Escape" => (0x1B, 0),
        "Backspace" => (0x08, 0),
        "Tab" => (0x09, '\t' as u32),
        "Delete" => (0x2E, 0),
        "Left" => (0x25, 0),
        "Up" => (0x26, 0),
        "Right" => (0x27, 0),
        "Down" => (0x28, 0),
        "Shift" => (0x10, 0),
        "Ctrl" => (0x11, 0),
        "Alt" => (0x12, 0),
        "Super" => (0x5B, 0),
        "CapsLock" => (0x14, 0),
        _ => {
            // Một combo kết thúc bằng chữ/số (Ctrl+T, Win+L, Alt+Tab...) là
            // shortcut hợp lệ. Giữ case của `ch`, còn VK chuẩn hoá uppercase
            // để mô phỏng virtual-key Windows; khi có Ctrl/Alt/Win engine sẽ
            // bypass nên ký tự không được dùng để biến đổi text.
            if name.len() == 1 && name.as_bytes()[0].is_ascii_alphanumeric() {
                let ch = name.as_bytes()[0] as char;
                let vk = if ch.is_ascii_alphabetic() {
                    ch.to_ascii_uppercase() as u32
                } else {
                    ch as u32
                };
                return Some((vk, ch as u32));
            }
            if let Some(n) = name.strip_prefix('F') {
                if let Ok(i) = n.parse::<u32>() {
                    if (1..=12).contains(&i) {
                        return Some((0x70 + i - 1, 0));
                    }
                }
            }
            return None;
        }
    };
    Some((vk, ch))
}

/// `Ctrl+Shift+Space` → (mods, vk, ch).
fn parse_combo(s: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.len() < 2 {
        return Err(format!("combo `{s}` cần ít nhất 1 modifier"));
    }
    let mut mods = 0u32;
    for p in &parts[..parts.len() - 1] {
        mods |= match *p {
            "Ctrl" => 0x2,
            "Shift" => 0x1,
            "Alt" => 0x4,
            "Super" | "Win" => 0x8,
            other => return Err(format!("modifier lạ `{other}`")),
        };
    }
    let last = parts[parts.len() - 1];
    let (vk, ch) = key_by_name(last).ok_or_else(|| format!("phím lạ `{last}`"))?;
    Ok((mods, vk, ch))
}

/// Bảng config hợp lệ cho `:config` (P0-3 §1.1). Trả **giá trị đã chuẩn hoá** để build JSON.
///
/// `macro` / `emoji` (P0-4 §2.3b bổ sung — ghi `P0-REVIEW-LOG`): mỗi dòng thêm **1 entry**,
/// giá trị phải ở dạng chuỗi có ngoặc kép `"trigger=expand"` (`=` phân tách) — vì `expand`
/// thường chứa dấu cách.
fn config_value(key: &str, value: &str) -> Result<String, String> {
    if key == "macro" || key == "emoji" {
        let inner = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| {
                format!("`{key}` cần giá trị trong ngoặc kép: {key}=\"trigger=text\"")
            })?;
        let Some((trigger, text)) = inner.split_once('=') else {
            return Err(format!("`{key}` cần `trigger=text`, nhận `{inner}`"));
        };
        if trigger.is_empty() || text.is_empty() {
            return Err(format!("`{key}`: trigger và text phải khác rỗng"));
        }
        return Ok(inner.to_string());
    }
    let ok = match key {
        "method" => matches!(value, "telex" | "vni" | "viqr" | "simple_telex"),
        "diacritic_style" => matches!(value, "new" | "old"),
        "macro_trigger" => matches!(value, "tab" | "space"),
        "output_charset" => matches!(
            value,
            "unicode_precomposed" | "unicode_decomposed" | "tcvn3" | "vni_windows"
        ),
        "enabled"
        | "free_marking"
        | "auto_restore_english"
        | "auto_capitalize"
        | "quick_telex"
        | "allow_macro_when_vi_off" => matches!(value, "true" | "false"),
        _ => return Err(format!("config key lạ `{key}`")),
    };
    if ok {
        Ok(value.to_string())
    } else {
        Err(format!("config value không hợp lệ `{key}={value}`"))
    }
}

/// `english_words=w1,w2` — danh sách từ EN giữ nguyên (opt-in, P0-3 §1.1; mẫu
/// `data/stop_en.txt`). Giá trị rỗng = danh sách rỗng (tắt). Từ chứa dấu phẩy/dấu
/// nháy kép thì không biểu diễn được trong DSL → báo lỗi tường minh thay vì cắt sai.
fn config_english_words(value: &str) -> Result<String, String> {
    for w in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if !w.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(format!(
                "`english_words` chỉ nhận chữ cái ASCII, từng từ cách nhau bằng `,` (lỗi ở `{w}`)"
            ));
        }
    }
    Ok(value.to_string())
}

/// `field_role` chuỗi → số FFI (bảng 1-1 P0-3 §2.1 — giữ inline để cli không phụ thuộc strategy).
fn field_role_id(s: &str) -> Option<u32> {
    Some(match s {
        "unknown" => 0,
        "body" => 1,
        "editbox" => 2,
        "address_bar" => 3,
        "search" => 4,
        "combo" => 5,
        "candidate" => 6,
        "textarea" => 7,
        "web" => 8,
        "terminal" => 9,
        "secure" => 10,
        _ => return None,
    })
}

/// Parse 1 dòng lệnh → `Cmd`. Trả `Ok(None)` nếu dòng rỗng/comment.
fn parse_line(line: &str) -> Result<Option<Cmd>, String> {
    let stripped = strip_comment(line).trim();
    if stripped.is_empty() {
        return Ok(None);
    }
    let Some(rest) = stripped.strip_prefix(':') else {
        return Err("lệnh phải bắt đầu bằng `:`".into());
    };
    let tokens = split_tokens(rest)?;
    let (name, args) = tokens.split_first().ok_or("thiếu tên lệnh")?;

    let cmd = match name.as_str() {
        "config" => {
            let mut pairs = Vec::new();
            for a in args {
                let Some((k, v)) = a.split_once('=') else {
                    return Err(format!("`:config` cần k=v, nhận `{a}`"));
                };
                let v = if k == "english_words" {
                    config_english_words(v)?
                } else {
                    config_value(k, v)?
                };
                pairs.push((k.to_string(), v));
            }
            if pairs.is_empty() {
                return Err("`:config` cần ít nhất 1 k=v".into());
            }
            Cmd::Config(pairs)
        }
        "app" => {
            if args.len() != 2 || !args[1].starts_with("field=") {
                return Err("cú pháp: `:app <app_id> field=<role>`".into());
            }
            let role = &args[1][6..];
            if field_role_id(role).is_none() {
                return Err(format!("role lạ `{role}`"));
            }
            Cmd::App {
                app_id: args[0].clone(),
                field: role.to_string(),
            }
        }
        "caps" => {
            let mut flags = Vec::new();
            for a in args.iter().flat_map(|s| s.split(',')) {
                if a.is_empty() {
                    continue;
                }
                if !matches!(a, "preedit" | "selection" | "field_detect" | "inject_vk") {
                    return Err(format!("caps lạ `{a}`"));
                }
                flags.push(a.to_string());
            }
            if flags.is_empty() {
                return Err("`:caps` cần ít nhất 1 flag".into());
            }
            Cmd::Caps(flags)
        }
        "enabled" | "secure" => {
            let Some(v) = args.first() else {
                return Err(format!("`:{name}` cần on|off"));
            };
            let b = match v.as_str() {
                "on" => true,
                "off" => false,
                other => return Err(format!("`:{name}` cần on|off, nhận `{other}`")),
            };
            if name == "enabled" {
                Cmd::Enabled(b)
            } else {
                Cmd::Secure(b)
            }
        }
        "type" | "injected" => {
            let Some(s) = args.first() else {
                return Err(format!("`:{name}` cần chuỗi trong ngoặc kép"));
            };
            let text = parse_quoted(s)?;
            let chars: Vec<char> = text.chars().collect();
            if name == "type" {
                Cmd::Type(chars)
            } else {
                Cmd::Injected(chars)
            }
        }
        "key" => {
            let Some(k) = args.first() else {
                return Err("`:key` cần tên phím (P0-4 §2.4)".into());
            };
            if key_by_name(k).is_none() {
                return Err(format!("tên phím lạ `{k}`"));
            }
            Cmd::Key(k.clone())
        }
        "combo" => {
            let Some(s) = args.first() else {
                return Err("`:combo` cần chuỗi phím tắt".into());
            };
            parse_combo(s)?;
            Cmd::Combo(s.clone())
        }
        "expect" => {
            let Some(s) = args.first() else {
                return Err("`:expect` cần chuỗi".into());
            };
            Cmd::Expect(parse_quoted(s)?)
        }
        "expect_preedit" => {
            let Some(s) = args.first() else {
                return Err("`:expect_preedit` cần chuỗi".into());
            };
            Cmd::ExpectPreedit(parse_quoted(s)?)
        }
        "expect_action" => {
            let Some(s) = args.first() else {
                return Err("`:expect_action` cần PASS|REPLACE|COMMIT|RESTORE".into());
            };
            if !matches!(s.as_str(), "PASS" | "REPLACE" | "COMMIT" | "RESTORE") {
                return Err(format!("action lạ `{s}`"));
            }
            Cmd::ExpectAction(s.clone())
        }
        "expect_cursor" => {
            let Some(s) = args.first() else {
                return Err("`:expect_cursor` cần số".into());
            };
            let n: usize = s.parse().map_err(|_| format!("expect_cursor sai `{s}`"))?;
            Cmd::ExpectCursor(n)
        }
        "reset" => {
            if !args.is_empty() {
                return Err("`:reset` không nhận tham số".into());
            }
            Cmd::Reset
        }
        "engine_new" => {
            if !args.is_empty() {
                return Err("`:engine_new` không nhận tham số".into());
            }
            Cmd::EngineNew
        }
        "mods" => {
            let Some(s) = args.first() else {
                return Err("`:mods` cần +Shift / -Shift".into());
            };
            let (add, name) = match s.split_at(1) {
                ("+", rest) => (true, rest),
                ("-", rest) => (false, rest),
                _ => return Err(format!("`:mods` cần +/-, nhận `{s}`")),
            };
            if !matches!(name, "Shift" | "Ctrl" | "Alt" | "Super" | "CapsLock") {
                return Err(format!("modifier lạ `{name}`"));
            }
            Cmd::Mods {
                add,
                name: name.to_string(),
            }
        }
        "note" => {
            let Some(s) = args.first() else {
                return Err("`:note` cần chuỗi".into());
            };
            Cmd::Note(parse_quoted(s)?)
        }
        other => return Err(format!("lệnh lạ `:{other}`")),
    };
    Ok(Some(cmd))
}

// ---- nạp corpus ----

struct Case {
    id: String,
    cmds: Vec<(usize, Cmd)>,
    source: Vec<String>,
}

fn collect_files(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in paths {
        let path = Path::new(p);
        if !path.exists() {
            return Err(format!("path không tồn tại: {p}"));
        }
        if path.is_file() {
            out.push(path.to_path_buf());
        } else {
            walk(path, &mut out)?;
        }
    }
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("không đọc được {}: {e}", dir.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| format!("read_dir lỗi: {e}"))?;
        let p = entry.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some("keys") {
            out.push(p);
        }
    }
    Ok(())
}

fn load_cases(paths: &[String], filter: Option<&str>) -> (Vec<Case>, Vec<ParseError>) {
    let mut cases = Vec::new();
    let mut errors = Vec::new();
    let files = match collect_files(paths) {
        Ok(f) => f,
        Err(msg) => {
            errors.push(ParseError {
                file: "<cli>".into(),
                line: 0,
                msg,
            });
            return (cases, errors);
        }
    };
    for file in files {
        let id = file.display().to_string();
        let text = match fs::read_to_string(&file) {
            Ok(t) => t,
            Err(e) => {
                errors.push(ParseError {
                    file: id,
                    line: 0,
                    msg: format!("đọc file lỗi: {e}"),
                });
                continue;
            }
        };
        let mut cmds = Vec::new();
        for (i, line) in text.lines().enumerate() {
            match parse_line(line) {
                Ok(Some(cmd)) => cmds.push((i + 1, cmd)),
                Ok(None) => {}
                Err(msg) => errors.push(ParseError {
                    file: id.clone(),
                    line: i + 1,
                    msg,
                }),
            }
        }
        if let Some(f) = filter {
            if !id.contains(f) {
                continue;
            }
        }
        // Case không kiểm gì thì luôn PASS — che mất hồi quy (gõ lại corpus mà quên
        // dòng `:expect` vẫn xanh). Bắt buộc ít nhất một khẳng định.
        let asserts = cmds.iter().any(|(_, c)| {
            matches!(
                c,
                Cmd::Expect(_)
                    | Cmd::ExpectPreedit(_)
                    | Cmd::ExpectAction(_)
                    | Cmd::ExpectCursor(_)
            )
        });
        if !asserts {
            errors.push(ParseError {
                file: id.clone(),
                line: 0,
                msg: "case không có `:expect*` nào — luôn PASS, không kiểm được gì".into(),
            });
            continue;
        }
        cases.push(Case {
            id,
            cmds,
            source: text.lines().map(String::from).collect(),
        });
    }
    (cases, errors)
}

// ---- simulator (P0-4 §3) ----

fn zeroed_result() -> ime_result_v1 {
    unsafe { std::mem::zeroed::<ime_result_v1>() }
}

fn decode_utf32(arr: &[u32], len: u16) -> String {
    arr[..len as usize]
        .iter()
        .filter_map(|&u| char::from_u32(u))
        .collect()
}

fn action_name(action: u32) -> &'static str {
    match action {
        0 => "PASS",
        1 => "REPLACE",
        2 => "COMMIT",
        3 => "RESTORE",
        _ => "?",
    }
}

fn printable_of(vk: u32, ch: u32) -> Option<char> {
    if ch != 0 {
        return char::from_u32(ch);
    }
    match vk {
        0x20 => Some(' '),
        0x0D => Some('\n'),
        0x09 => Some('\t'),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceEntry {
    step: usize,
    cmd: &'static str,
    text: String,
    n: u32,
}

fn canonical_case_rel(id: &str) -> String {
    let mut s = id.replace('\\', "/");
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string();
    }
    let is_abs = s.starts_with('/') || (s.len() >= 2 && s.as_bytes()[1] == b':');
    let rel = if is_abs {
        if let Some(pos) = s.find("/corpus/") {
            &s[pos + "/corpus/".len()..]
        } else {
            Path::new(id)
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("case")
        }
    } else if let Some(rest) = s.strip_prefix("corpus/") {
        rest
    } else {
        s.as_str()
    };
    rel.strip_suffix(".keys").unwrap_or(rel).to_string()
}

fn format_trace_jsonl(case_rel: &str, entries: &[TraceEntry]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let escaped_case = json_str(case_rel);
    for e in entries {
        let _ = writeln!(
            out,
            "{{\"case\":{escaped_case},\"step\":{},\"cmd\":{},\"text\":{},\"n\":{}}}",
            e.step,
            json_str(e.cmd),
            json_str(&e.text),
            e.n
        );
    }
    out
}

struct Sim {
    inst: *mut ime_instance,
    settings: Vec<(String, String)>,
    field: u32,
    caps: u32,
    enabled: bool,
    secure: bool,
    buf: Vec<char>,
    cursor: usize,
    preedit: String,
    held_mods: u32,
    pending_action: Option<(usize, String)>,
    last_action: String,
    failures: Vec<SimFailure>,
    /// `Some` khi mô phỏng TSF: text composition đang mở (nằm trong `buf` tại
    /// `comp_start`, caret ở cuối composition).
    tsf: Option<TsfComposition>,
    /// Số ký tự composition TSF đang nằm trong `buf` tại `TsfComposition::start`.
    comp_len: usize,
    /// SF-1c: ghi vết chuẩn hoá các lệnh adapter áp khi `--trace` bật.
    trace: Option<Vec<TraceEntry>>,
    /// Chỉ số phím hiện tại (0-based theo từng sự kiện `send_event`).
    key_step: usize,
}

/// Composition TSF mô phỏng: chữ đang soạn nằm trong `buf` tại `start`; model + lệnh do
/// `textvn_session::compose::Composer` (SF-3) — vết TSF là CHÍNH lệnh của session.
struct TsfComposition {
    start: usize,
    composer: Composer,
}

impl TsfComposition {
    fn new(policy: ComposePolicy) -> Self {
        Self {
            start: 0,
            composer: Composer::new(policy),
        }
    }
}

#[derive(Debug)]
struct SimFailure {
    line: usize,
    msg: String,
    expected: Option<String>,
    actual: Option<String>,
}

impl Sim {
    fn new(profile: AdapterProfile, record_trace: bool) -> Sim {
        let mut inst: *mut ime_instance = std::ptr::null_mut();
        let rc = ime_instance_new(std::ptr::null(), 0, &mut inst);
        assert_eq!(rc, IME_OK, "ime_instance_new default phải OK");
        let mut sim = Sim {
            inst,
            settings: Vec::new(),
            field: 0,
            caps: profile.default_caps,
            enabled: true,
            secure: false,
            buf: Vec::new(),
            cursor: 0,
            preedit: String::new(),
            held_mods: 0,
            pending_action: None,
            last_action: "PASS".into(),
            failures: Vec::new(),
            tsf: profile.compose.map(TsfComposition::new),
            comp_len: 0,
            trace: record_trace.then(Vec::new),
            key_step: 0,
        };
        // Context mặc định cũng phải mang capability profile: corpus không có
        // `:app`/`:caps` vẫn cần mô phỏng adapter đã chọn.
        sim.push_ctx(0);
        sim
    }

    fn emit_trace(&mut self, cmd: &'static str, text: String, n: u32) {
        if let Some(t) = self.trace.as_mut() {
            t.push(TraceEntry {
                step: self.key_step,
                cmd,
                text,
                n,
            });
        }
    }

    fn fail(&mut self, line: usize, msg: String) {
        self.failures.push(SimFailure {
            line,
            msg,
            expected: None,
            actual: None,
        });
    }

    fn fail_cmp(&mut self, line: usize, msg: String, expected: String, actual: String) {
        self.failures.push(SimFailure {
            line,
            msg,
            expected: Some(expected),
            actual: Some(actual),
        });
    }

    fn push_ctx(&mut self, line: usize) {
        let ctx = ime_context_v1 {
            abi_version: IME_ABI_VERSION,
            enabled: self.enabled as u32,
            secure: self.secure as u32,
            field_role: self.field,
            caps: self.caps,
            app_id: std::ptr::null(),
            element_name: std::ptr::null(),
            hint: -1,
        };
        let rc = ime_set_context(self.inst, &ctx);
        if rc != IME_OK {
            self.fail(line, format!("ime_set_context rc={rc}"));
        }
    }

    /// Gộp `:config k=v` vào settings rồi reload (JSON build ở `build_json` — value đã validate).
    fn apply_config(&mut self, pairs: &[(String, String)], line: usize) {
        for (k, v) in pairs {
            // `macro`/`emoji` lặp được (mỗi dòng 1 entry) — không gộp đè
            if k == "macro" || k == "emoji" {
                self.settings.push((k.clone(), v.clone()));
                continue;
            }
            match self.settings.iter_mut().find(|(ek, _)| ek == k) {
                Some(slot) => slot.1 = v.clone(),
                None => self.settings.push((k.clone(), v.clone())),
            }
        }
        let json = build_json(&self.settings);
        let rc = ime_reload_config(self.inst, json.as_ptr(), json.len());
        if rc != IME_OK {
            self.fail(
                line,
                format!("ime_reload_config rc={rc} (config giữ nguyên theo P0-3 §1.3)"),
            );
        }
    }

    /// Gửi 1 sự kiện key (down rồi up — P0-4 §2.2), cập nhật buffer/preedit theo §3.
    fn send_event(&mut self, vk: u32, ch: u32, mods: u32, injected: bool, line: usize) {
        // IBus/Fcitx5 gửi engine phím đã chuẩn hoá (`lc_key_to_ime`) ở key-down đường thường;
        // phím modifier, phím bị cổng chặn và mọi key-up đi nguyên văn.
        let linux_down =
            (self.linux_mode() && !is_modifier_vk(vk) && !self.gated(vk, ch, mods, injected))
                .then(|| linux_key_to_engine(vk, ch, mods));
        for down in [1u8, 0u8] {
            let (kvk, kch) = match linux_down {
                Some((_, evk, ech)) if down == 1 => (evk, ech),
                _ => (vk, ch),
            };
            let k = ime_key_v1 {
                abi_version: IME_ABI_VERSION,
                vk: kvk,
                ch: kch,
                mods,
                key_down: down,
                is_repeat: 0,
                is_injected: injected as u8,
                _reserved: 0,
            };
            let mut out = zeroed_result();
            let rc = ime_key(self.inst, &k, &mut out);
            if rc != IME_OK {
                self.fail(line, format!("ime_key rc={rc}"));
                self.key_step += 1;
                return;
            }
            self.preedit = decode_utf32(&out.preedit, out.preedit_len);
            if down == 0 {
                // Ctrl tap trả RESTORE ngay ở key-up (`sort` → `sỏt` → `sort`).
                if out.action != 0 {
                    if self.tsf.is_some() {
                        self.apply_tsf(vk, ch, mods, injected, &out, line, down);
                    } else {
                        self.apply_action(&out, line);
                    }
                } else if out.flags & textvn_ffi::IME_FLAG_WORD_END != 0 {
                    self.tsf_end();
                }
                continue;
            }
            if self.tsf.is_some() {
                self.apply_tsf(vk, ch, mods, injected, &out, line, down);
            } else if !injected {
                if out.action == 0 {
                    self.apply_pass(vk, ch);
                } else {
                    self.apply_action(&out, line);
                }
            }
            self.last_action = action_name(out.action).to_string();
            if let Some((eline, want)) = self.pending_action.take() {
                if want != self.last_action {
                    self.fail_cmp(
                        eline,
                        "expect_action không khớp".into(),
                        want,
                        self.last_action.clone(),
                    );
                }
            }
        }
        self.key_step += 1;
    }

    /// Đang mô phỏng IBus/Fcitx5 (`Composer` chính sách LINUX)?
    fn linux_mode(&self) -> bool {
        self.tsf
            .as_ref()
            .is_some_and(|c| c.composer.policy() == ComposePolicy::LINUX)
    }

    /// Phím bị cổng adapter chặn (commit nguyên văn rồi app nhận phím): phím bơm, chord, ô secure,
    /// tắt tiếng Việt (trừ khi còn gõ tắt `allow_macro_when_vi_off`). TSF coi mọi Ctrl/Alt/Win là
    /// chord; Linux chừa Ctrl+Space (trigger dự đoán) và tính cả Meta (`trace_compose.c`).
    fn gated(&self, vk: u32, ch: u32, mods: u32, injected: bool) -> bool {
        let macro_only = !self.enabled && self.setting_is_true("allow_macro_when_vi_off");
        let chord = if self.linux_mode() {
            mods & 0x1E != 0 && !linux_ctrl_space(vk, ch, mods)
        } else {
            mods & 0xE != 0
        };
        injected || chord || self.secure || !(self.enabled || macro_only)
    }

    fn setting_is_true(&self, key: &str) -> bool {
        self.settings.iter().any(|(k, v)| k == key && v == "true")
    }

    /// Đóng composition TSF (text giữ nguyên trong buffer): `Composer::end` commit TRƯỚC khi ẩn.
    fn tsf_end(&mut self) {
        let mut cmds = Commands::new();
        if let Some(comp) = self.tsf.as_mut() {
            comp.composer.end(&mut cmds);
        }
        self.apply_session_cmds(cmds, 0);
    }

    /// Áp lệnh session vào tài liệu mô phỏng (composition ở `start`) và ghi vết chuẩn hoá.
    /// `DeleteBefore` xoá trước composition; `SetPreedit`/`Commit` thay vùng composition.
    fn apply_session_cmds(&mut self, cmds: Commands, line: usize) {
        for cmd in cmds.into_vec() {
            let Some(comp) = self.tsf.as_mut() else {
                return;
            };
            match cmd {
                SessionCommand::DeleteBefore { chars } => {
                    let n = usize::from(chars);
                    if n > comp.start {
                        self.fail(
                            line,
                            format!("TSF delete_before({n}) vượt text trước composition"),
                        );
                        return;
                    }
                    self.buf.drain(comp.start - n..comp.start);
                    comp.start -= n;
                    self.emit_trace("delete_before", String::new(), u32::from(chars));
                }
                SessionCommand::SetPreedit { text, .. } => {
                    let new: Vec<char> = text.chars().collect();
                    let end = (comp.start + self.comp_len).min(self.buf.len());
                    self.buf.splice(comp.start..end, new.iter().copied());
                    self.comp_len = new.len();
                    if !new.is_empty() || end > comp.start {
                        self.cursor = comp.start + new.len();
                    }
                    self.emit_trace("set_preedit", text, new.len() as u32);
                }
                SessionCommand::Commit { text } => {
                    let new: Vec<char> = text.chars().collect();
                    let end = (comp.start + self.comp_len).min(self.buf.len());
                    self.buf.splice(comp.start..end, new.iter().copied());
                    self.cursor = comp.start + new.len();
                    comp.start = self.cursor;
                    self.comp_len = 0;
                    self.emit_trace("commit", text, 0);
                }
                SessionCommand::EatKey => self.emit_trace("eat_key", String::new(), 0),
                SessionCommand::ForwardKey => self.emit_trace("forward_key", String::new(), 0),
                _ => {}
            }
        }
    }

    /// Mô phỏng TSF adapter: cùng thứ tự quyết định như `key_event.rs` và cùng
    /// kế hoạch `plan_key` như `edit_session.rs`.
    #[allow(clippy::too_many_arguments)]
    fn apply_tsf(
        &mut self,
        vk: u32,
        ch: u32,
        mods: u32,
        injected: bool,
        out: &ime_result_v1,
        line: usize,
        down: u8,
    ) {
        if is_modifier_vk(vk) && out.action == 0 {
            if down == 1 {
                self.emit_trace("forward_key", String::new(), 0);
            }
            return;
        }
        // Chord / phím bơm (VK_PACKET) / field bị gate / tắt VN (trừ khi còn gõ tắt —
        // `allow_macro_when_vi_off`): commit rồi app nhận phím. Linux (`runner_finish_word`)
        // còn reset engine ở key-down.
        if self.gated(vk, ch, mods, injected) {
            self.tsf_end();
            if down == 1 && self.linux_mode() {
                ime_reset(self.inst);
            }
            if down == 1 {
                self.emit_trace("forward_key", String::new(), 0);
            }
            if !injected && down == 1 {
                self.apply_pass(vk, ch);
            }
            return;
        }
        let Some(comp) = self.tsf.as_mut() else {
            return;
        };
        let linux = comp.composer.policy() == ComposePolicy::LINUX;
        let key = if linux && down == 0 {
            KeyKind::Other // key-up non-PASS (Ctrl tap RESTORE): `LC_KEY_OTHER`
        } else if linux {
            linux_key_to_engine(vk, ch, mods).0
        } else {
            KeyKind::classify(vk, ch)
        };
        if !comp.composer.is_composing() {
            comp.start = self.cursor;
            self.comp_len = 0;
        }
        let mut cmds = Commands::new();
        let plan = comp
            .composer
            .key(key, &EngineStep::from_result(out), down == 1, &mut cmds);
        self.apply_session_cmds(cmds, line);
        if plan.reset_engine {
            ime_reset(self.inst);
        }
        if !plan.eaten && down == 1 {
            self.apply_pass(vk, ch);
        }
    }

    /// PASS: phím đi thẳng cho "app" — chuẩn hóa theo P0-4 §3
    /// (Backspace xoá trước cursor, Delete xoá tại cursor, ký tự in được chèn vào cursor).
    /// Giữ im ở rìa (giống bàn phím thật, không fail).
    fn apply_pass(&mut self, vk: u32, ch: u32) {
        match vk {
            0x08 => {
                // Backspace
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buf.remove(self.cursor);
                }
            }
            0x2E => {
                // Delete
                if self.cursor < self.buf.len() {
                    self.buf.remove(self.cursor);
                }
            }
            0x25 => self.cursor = self.cursor.saturating_sub(1), // Left
            0x27 => self.cursor = (self.cursor + 1).min(self.buf.len()), // Right
            _ => {
                if let Some(c) = printable_of(vk, ch) {
                    self.buf.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
        }
    }

    /// Áp action vào buffer mô phỏng — đúng semantics P0-4 §3.
    fn apply_action(&mut self, out: &ime_result_v1, line: usize) {
        let insert = decode_utf32(&out.insert, out.insert_len);
        match out.action {
            1 | 3 => {
                // REPLACE / RESTORE: xóa delete_count ký tự TRƯỚC cursor, chèn insert
                let del = out.delete_count as usize;
                if del > self.cursor {
                    self.fail_cmp(
                        line,
                        format!(
                            "delete_count({del}) vượt cursor({}) — đừng mask bug",
                            self.cursor
                        ),
                        "hợp lệ".into(),
                        format!("delete={del}, cursor={}", self.cursor),
                    );
                    return;
                }
                let start = self.cursor - del;
                self.buf.drain(start..self.cursor);
                self.buf.splice(start..start, insert.chars());
                self.cursor = start + insert.chars().count();
            }
            2 => {
                // COMMIT — chèn không xóa
                self.buf.splice(self.cursor..self.cursor, insert.chars());
                self.cursor += insert.chars().count();
            }
            0 => {} // PASS đã xử lý riêng trong send_event
            _ => self.fail(line, format!("action không hợp lệ {}", out.action)),
        }
    }
}

impl Drop for Sim {
    fn drop(&mut self) {
        ime_instance_free(self.inst);
    }
}

/// Chuỗi JSON an toàn (escape `"` `\` và ký tự điều khiển) — text người dùng trong macro/emoji.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// JSON `config.v1` từ `:config` — `macro`/`emoji` (lặp được) gom vào `macros[]`/`emoji[]`.
fn build_json(settings: &[(String, String)]) -> String {
    let mut json = String::from("{\"config_version\":1");
    let mut macros: Vec<String> = Vec::new();
    let mut emoji: Vec<String> = Vec::new();
    let mut english_words = String::new();
    for (k, v) in settings {
        if k == "macro" || k == "emoji" {
            let Some((trigger, text)) = v.split_once('=') else {
                continue; // đã validate ở parser
            };
            if k == "macro" {
                macros.push(format!(
                    "{{\"trigger\":{},\"expand\":{},\"when\":\"always\"}}",
                    json_str(trigger),
                    json_str(text)
                ));
            } else {
                emoji.push(format!(
                    "{{\"trigger\":{},\"glyph\":{}}}",
                    json_str(trigger),
                    json_str(text)
                ));
            }
            continue;
        }
        // `english_words=w1,w2` → phần tử bên trong ngoặc (không dấu phẩy thừa —
        // `["a",]` là JSON hỏng, đó là lý do `rc=-2` lúc đầu). Lần sau **thay thế** lần trước.
        if k == "english_words" {
            let inner = v
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(json_str)
                .collect::<Vec<_>>()
                .join(",");
            english_words = inner;
            continue;
        }
        json.push(',');
        json.push_str(&json_str(k));
        json.push(':');
        if v == "true" || v == "false" {
            json.push_str(v);
        } else {
            json.push_str(&json_str(v));
        }
    }
    if !macros.is_empty() {
        json.push_str(",\"macros\":[");
        json.push_str(&macros.join(","));
        json.push(']');
    }
    if !emoji.is_empty() {
        json.push_str(",\"emoji\":[");
        json.push_str(&emoji.join(","));
        json.push(']');
    }
    // `:config english_words=...` — lần xuất hiện sau thắng; ghi ra đúng 1 mảng.
    if !english_words.is_empty() {
        json.push_str(",\"english_words\":[");
        json.push_str(&english_words);
        json.push(']');
    }
    json.push('}');
    json
}

fn caps_bits(flags: &[String]) -> u32 {
    let mut bits = 0;
    for f in flags {
        bits |= match f.as_str() {
            "preedit" => 0x1,
            "selection" => 0x2,
            "field_detect" => 0x4,
            "inject_vk" => 0x8,
            _ => 0,
        };
    }
    bits
}

/// Ctrl+Space của adapter Linux (`is_ctrl_space`/`lc_key_classify_with_mods`: Space theo vk HOẶC ch).
fn linux_ctrl_space(vk: u32, ch: u32, mods: u32) -> bool {
    (vk == 0x20 || ch == ' ' as u32) && (mods & !0x20) == 0x2
}

/// `lc_key_classify_with_mods` + `lc_key_to_ime`: loại phím và (vk, ch) gửi engine của IBus/Fcitx5.
fn linux_key_to_engine(vk: u32, ch: u32, mods: u32) -> (KeyKind, u32, u32) {
    let key = if linux_ctrl_space(vk, ch, mods) {
        KeyKind::CtrlSpace
    } else {
        KeyKind::classify(vk, ch)
    };
    let evk = match key {
        KeyKind::Backspace => 0x08,
        KeyKind::Tab => 0x09,
        KeyKind::Enter => 0x0D,
        KeyKind::Escape => 0x1B,
        KeyKind::CtrlSpace => 0x20,
        KeyKind::Char(' ') => 0x20,
        _ => vk,
    };
    (key, evk, key.engine_ch())
}

fn mod_bit(name: &str) -> u32 {
    match name {
        "Shift" => 0x1,
        "Ctrl" => 0x2,
        "Alt" => 0x4,
        "Super" => 0x8,
        // Trạng thái Caps Lock bật (IME_MOD_CAPS) — `:mods +CapsLock`.
        "CapsLock" => 0x20,
        _ => 0,
    }
}

struct CaseKeySpec<'a> {
    vk: u32,
    ch: u32,
    mods: u32,
    injected: bool,
    step: usize,
    bundle: &'a str,
}

fn push_key_json(keys: &mut Vec<String>, events: &mut Vec<String>, spec: &CaseKeySpec<'_>) {
    let _ = spec.bundle;
    for down in [1u8, 0u8] {
        let obj = format!(
            "{{\"step\":{},\"vk\":{},\"ch\":{},\"mods\":{},\"down\":{},\"injected\":{}}}",
            spec.step, spec.vk, spec.ch, spec.mods, down, spec.injected
        );
        keys.push(obj);
        events.push(format!(
            "{{\"kind\":\"key\",\"step\":{},\"vk\":{},\"ch\":{},\"mods\":{},\"down\":{},\"injected\":{}}}",
            spec.step, spec.vk, spec.ch, spec.mods, down, spec.injected
        ));
    }
}

/// SF-1c / G22 (`--emit-keys <dir>`): xuất cấu hình, ngữ cảnh và chuỗi phím đã phân giải
/// thành JSON để adapter macOS (SF-1e Swift) / Linux (SF-1d C) dùng lại thay vì viết lại
/// parser `.keys`.
fn emit_case_keys_json(case: &Case, case_rel: &str, profile: AdapterProfile) -> String {
    let mut settings: Vec<(String, String)> = Vec::new();
    let mut bundle = String::new();
    let mut field_role = 0u32;
    let mut caps = profile.default_caps;
    let mut enabled = true;
    let mut secure = false;
    let mut held_mods = 0u32;
    let mut step = 0usize;
    let mut started = false;

    let mut init_config_json = String::from("{\"config_version\":1}");
    let mut init_app = (bundle.clone(), field_role, caps, secure, enabled);
    let mut keys: Vec<String> = Vec::new();
    let mut resets: Vec<String> = Vec::new();
    let mut events: Vec<String> = Vec::new();

    for (_, cmd) in &case.cmds {
        match cmd {
            Cmd::Config(pairs) => {
                for (k, v) in pairs {
                    if k == "macro" || k == "emoji" {
                        settings.push((k.clone(), v.clone()));
                        continue;
                    }
                    match settings.iter_mut().find(|(ek, _)| ek == k) {
                        Some(slot) => slot.1 = v.clone(),
                        None => settings.push((k.clone(), v.clone())),
                    }
                }
                let cfg = build_json(&settings);
                if !started {
                    init_config_json = cfg.clone();
                } else {
                    events.push(format!(
                        "{{\"kind\":\"config\",\"step\":{},\"config_json\":{}}}",
                        step,
                        json_str(&cfg)
                    ));
                }
            }
            Cmd::App { app_id, field } => {
                bundle = app_id.clone();
                field_role = field_role_id(field).unwrap_or(0);
                if !started {
                    init_app = (bundle.clone(), field_role, caps, secure, enabled);
                } else {
                    events.push(format!(
                        "{{\"kind\":\"context\",\"step\":{},\"bundle\":{},\"field_role\":{},\"caps\":{},\"secure\":{},\"enabled\":{}}}",
                        step,
                        json_str(&bundle),
                        field_role,
                        caps,
                        secure,
                        enabled
                    ));
                }
            }
            Cmd::Caps(flags) => {
                caps = caps_bits(flags);
                if !started {
                    init_app = (bundle.clone(), field_role, caps, secure, enabled);
                } else {
                    events.push(format!(
                        "{{\"kind\":\"context\",\"step\":{},\"bundle\":{},\"field_role\":{},\"caps\":{},\"secure\":{},\"enabled\":{}}}",
                        step,
                        json_str(&bundle),
                        field_role,
                        caps,
                        secure,
                        enabled
                    ));
                }
            }
            Cmd::Enabled(b) => {
                enabled = *b;
                if !started {
                    init_app = (bundle.clone(), field_role, caps, secure, enabled);
                } else {
                    events.push(format!(
                        "{{\"kind\":\"context\",\"step\":{},\"bundle\":{},\"field_role\":{},\"caps\":{},\"secure\":{},\"enabled\":{}}}",
                        step,
                        json_str(&bundle),
                        field_role,
                        caps,
                        secure,
                        enabled
                    ));
                }
            }
            Cmd::Secure(b) => {
                secure = *b;
                if !started {
                    init_app = (bundle.clone(), field_role, caps, secure, enabled);
                } else {
                    events.push(format!(
                        "{{\"kind\":\"context\",\"step\":{},\"bundle\":{},\"field_role\":{},\"caps\":{},\"secure\":{},\"enabled\":{}}}",
                        step,
                        json_str(&bundle),
                        field_role,
                        caps,
                        secure,
                        enabled
                    ));
                }
            }
            Cmd::Mods { add, name } => {
                let bit = mod_bit(name);
                if *add {
                    held_mods |= bit;
                } else {
                    held_mods &= !bit;
                }
            }
            Cmd::Type(chars) => {
                started = true;
                for &c in chars {
                    push_key_json(
                        &mut keys,
                        &mut events,
                        &CaseKeySpec {
                            vk: 0,
                            ch: c as u32,
                            mods: held_mods,
                            injected: false,
                            step,
                            bundle: &bundle,
                        },
                    );
                    step += 1;
                }
            }
            Cmd::Injected(chars) => {
                started = true;
                for &c in chars {
                    push_key_json(
                        &mut keys,
                        &mut events,
                        &CaseKeySpec {
                            vk: 0,
                            ch: c as u32,
                            mods: held_mods,
                            injected: true,
                            step,
                            bundle: &bundle,
                        },
                    );
                    step += 1;
                }
            }
            Cmd::Key(name) => {
                started = true;
                let (vk, ch) = key_by_name(name).expect("đã validate ở parse");
                push_key_json(
                    &mut keys,
                    &mut events,
                    &CaseKeySpec {
                        vk,
                        ch,
                        mods: held_mods,
                        injected: false,
                        step,
                        bundle: &bundle,
                    },
                );
                step += 1;
            }
            Cmd::Combo(s) => {
                started = true;
                let (mods, vk, ch) = parse_combo(s).expect("đã validate ở parse");
                push_key_json(
                    &mut keys,
                    &mut events,
                    &CaseKeySpec {
                        vk,
                        ch,
                        mods,
                        injected: false,
                        step,
                        bundle: &bundle,
                    },
                );
                step += 1;
            }
            Cmd::Reset => {
                started = true;
                resets.push(step.to_string());
                events.push(format!("{{\"kind\":\"reset\",\"step\":{step}}}"));
            }
            Cmd::EngineNew => {
                started = true;
                events.push(format!("{{\"kind\":\"engine_new\",\"step\":{step}}}"));
            }
            Cmd::Expect(s) => {
                started = true;
                events.push(format!(
                    "{{\"kind\":\"expect\",\"step\":{},\"text\":{}}}",
                    step,
                    json_str(s)
                ));
            }
            Cmd::ExpectPreedit(s) => {
                started = true;
                events.push(format!(
                    "{{\"kind\":\"expect_preedit\",\"step\":{},\"text\":{}}}",
                    step,
                    json_str(s)
                ));
            }
            Cmd::ExpectCursor(n) => {
                started = true;
                events.push(format!(
                    "{{\"kind\":\"expect_cursor\",\"step\":{},\"n\":{n}}}",
                    step
                ));
            }
            Cmd::ExpectAction(_) | Cmd::Note(_) => {}
        }
    }

    format!(
        "{{\"case\":{},\"config_json\":{},\"app\":{{\"bundle\":{},\"field_role\":{},\"caps\":{},\"secure\":{},\"enabled\":{}}},\"keys\":[{}],\"resets\":[{}],\"events\":[{}]}}\n",
        json_str(case_rel),
        json_str(&init_config_json),
        json_str(&init_app.0),
        init_app.1,
        init_app.2,
        init_app.3,
        init_app.4,
        keys.join(","),
        resets.join(","),
        events.join(",")
    )
}

fn write_case_artifact(dir: &str, case_rel: &str, ext: &str, content: &str) -> Result<(), String> {
    let mut parts: Vec<&str> = case_rel.split('/').filter(|s| !s.is_empty()).collect();
    let Some(last) = parts.pop() else {
        return Err(format!("tên case rỗng: `{case_rel}`"));
    };
    let mut path = PathBuf::from(dir);
    for seg in parts {
        path.push(seg);
    }
    if let Err(e) = fs::create_dir_all(&path) {
        return Err(format!("tạo thư mục `{}` lỗi: {e}", path.display()));
    }
    path.push(format!("{last}.{ext}"));
    fs::write(&path, content).map_err(|e| format!("ghi `{}` lỗi: {e}", path.display()))
}

/// Chạy 1 case → (pass?, failures, ms, trace).
fn run_case(
    case: &Case,
    profile: AdapterProfile,
    record_trace: bool,
) -> (bool, Vec<SimFailure>, u128, Option<Vec<TraceEntry>>) {
    let start = Instant::now();
    let mut sim = Sim::new(profile, record_trace);
    for &(line, ref cmd) in &case.cmds {
        match cmd {
            Cmd::Config(pairs) => sim.apply_config(pairs, line),
            Cmd::App { field, .. } => {
                sim.field = field_role_id(field).unwrap_or(0);
                sim.push_ctx(line);
            }
            Cmd::Caps(flags) => {
                sim.caps = caps_bits(flags);
                sim.push_ctx(line);
            }
            Cmd::Enabled(b) => {
                sim.enabled = *b;
                sim.push_ctx(line);
            }
            Cmd::Secure(b) => {
                sim.secure = *b;
                sim.push_ctx(line);
            }
            Cmd::Type(chars) => {
                let m = sim.held_mods;
                for &c in chars {
                    sim.send_event(0, c as u32, m, false, line);
                }
            }
            Cmd::Key(name) => {
                let (vk, ch) = key_by_name(name).expect("đã validate ở parse");
                let m = sim.held_mods;
                sim.send_event(vk, ch, m, false, line);
            }
            Cmd::Combo(s) => {
                let (mods, vk, ch) = parse_combo(s).expect("đã validate ở parse");
                sim.send_event(vk, ch, mods, false, line);
            }
            Cmd::Injected(chars) => {
                let m = sim.held_mods;
                for &c in chars {
                    sim.send_event(0, c as u32, m, true, line);
                }
            }
            Cmd::Expect(s) => {
                let got: String = sim.buf.iter().collect();
                if got != *s {
                    sim.fail_cmp(line, ":expect không khớp".into(), s.clone(), got);
                }
            }
            Cmd::ExpectPreedit(s) => {
                if sim.preedit != *s {
                    sim.fail_cmp(
                        line,
                        ":expect_preedit không khớp".into(),
                        s.clone(),
                        sim.preedit.clone(),
                    );
                }
            }
            Cmd::ExpectAction(a) => {
                sim.pending_action = Some((line, a.clone()));
            }
            Cmd::ExpectCursor(n) => {
                if sim.cursor != *n {
                    sim.fail_cmp(
                        line,
                        ":expect_cursor không khớp".into(),
                        n.to_string(),
                        sim.cursor.to_string(),
                    );
                }
            }
            Cmd::Reset => {
                ime_reset(sim.inst);
                sim.preedit.clear();
                // TSF: focus đổi ⇒ composition được commit (OnSetFocus).
                sim.tsf_end();
            }
            Cmd::EngineNew => {
                // IBus/Fcitx5 (`EV_ENGINE_NEW`): chốt từ đang soạn trước khi thay engine.
                if sim.linux_mode() {
                    sim.tsf_end();
                }
                ime_instance_free(sim.inst);
                let mut inst: *mut ime_instance = std::ptr::null_mut();
                let rc = ime_instance_new(std::ptr::null(), 0, &mut inst);
                if rc != IME_OK {
                    sim.fail(line, format!("ime_instance_new rc={rc}"));
                }
                sim.inst = inst;
                if !sim.settings.is_empty() {
                    let json = build_json(&sim.settings);
                    let rc = ime_reload_config(sim.inst, json.as_ptr(), json.len());
                    if rc != IME_OK {
                        sim.fail(line, format!("ime_reload_config (engine_new) rc={rc}"));
                    }
                }
                sim.push_ctx(line);
            }
            Cmd::Mods { add, name } => {
                let bit = mod_bit(name);
                if *add {
                    sim.held_mods |= bit;
                } else {
                    sim.held_mods &= !bit;
                }
            }
            Cmd::Note(_) => {}
        }
    }
    // `expect_action` còn treo ở cuối file → so với action CUỐI CÙNG đã chạy
    // (ví dụ P0-4 §2.3 đặt `:expect_action PASS` sau `:type` — cho phép cả 2 vị trí).
    if let Some((eline, want)) = sim.pending_action.take() {
        if want != sim.last_action {
            sim.fail_cmp(
                eline,
                "expect_action không khớp (retro cuối file)".into(),
                want,
                sim.last_action.clone(),
            );
        }
    }
    let failures = std::mem::take(&mut sim.failures);
    let ok = failures.is_empty();
    let trace = sim.trace.take();
    (ok, failures, start.elapsed().as_millis(), trace)
}

// ---- báo cáo ----

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// In fail chi tiết: diff rõ ràng + context 3 dòng quanh (P0-4 §3).
fn print_failures(case: &Case, failures: &[SimFailure]) {
    for f in failures {
        println!("  {}:{}: {}", case.id, f.line, f.msg);
        if let (Some(exp), Some(act)) = (&f.expected, &f.actual) {
            println!("    expected: \"{exp}\"");
            println!("    actual:   \"{act}\"");
        }
        let lo = f.line.saturating_sub(2);
        let hi = (f.line + 2).min(case.source.len());
        for i in lo..=hi {
            if i == 0 || i > case.source.len() {
                continue;
            }
            let marker = if i == f.line { ">" } else { " " };
            println!("  {marker} {:4} | {}", i, case.source[i - 1]);
        }
    }
}

/// Entrypoint replay (P0-4 §4) + ghi vết SF-1c (`--trace`, `--emit-keys`). Exit: 0/1/2.
pub fn run(
    paths: &[String],
    adapter: &str,
    json: bool,
    filter: Option<&str>,
    trace_dir: Option<&str>,
    emit_keys_dir: Option<&str>,
) -> i32 {
    let profile = adapter_profile(adapter);

    let (cases, errors) = load_cases(paths, filter);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("parse error: {e}");
        }
        return 2;
    }
    if cases.is_empty() {
        eprintln!("error: không tìm thấy case `.keys` nào");
        return 2;
    }

    let mut failed = 0usize;
    let mut json_items: Vec<String> = Vec::new();
    for case in &cases {
        let case_rel = canonical_case_rel(&case.id);
        let (ok, failures, ms, trace) = run_case(case, profile, trace_dir.is_some());
        if let (Some(dir), Some(entries)) = (trace_dir, trace.as_deref()) {
            let body = format_trace_jsonl(&case_rel, entries);
            if let Err(e) = write_case_artifact(dir, &case_rel, "jsonl", &body) {
                eprintln!("error: {e}");
                return 2;
            }
        }
        if let Some(dir) = emit_keys_dir {
            let body = emit_case_keys_json(case, &case_rel, profile);
            if let Err(e) = write_case_artifact(dir, &case_rel, "json", &body) {
                eprintln!("error: {e}");
                return 2;
            }
        }
        let (first_exp, first_act) = failures
            .first()
            .map(|f| {
                (
                    f.expected.clone().unwrap_or_default(),
                    f.actual.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_default();
        if !ok {
            failed += 1;
        }
        if json {
            json_items.push(format!(
                "{{\"id\":\"{}\",\"status\":\"{}\",\"expected\":\"{}\",\"actual\":\"{}\",\"ms\":{}}}",
                json_escape(&case.id),
                if ok { "pass" } else { "fail" },
                json_escape(&first_exp),
                json_escape(&first_act),
                ms
            ));
        } else if ok {
            println!("PASS  {} ({} ms)", case.id, ms);
        } else {
            println!("FAIL  {} ({} ms)", case.id, ms);
            print_failures(case, &failures);
        }
    }

    if json {
        println!("[{}]", json_items.join(","));
    } else {
        println!("{} passed, {} failed", cases.len() - failed, failed);
    }

    if failed > 0 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_example_from_p0_4() {
        // Ví dụ P0-4 §2.3 — phải parse được ngay
        let c = parse_line(":config method=telex diacritic_style=new")
            .unwrap()
            .unwrap();
        assert_eq!(
            c,
            Cmd::Config(vec![
                ("method".into(), "telex".into()),
                ("diacritic_style".into(), "new".into())
            ])
        );
        assert!(matches!(
            parse_line(":app chrome.exe field=address_bar")
                .unwrap()
                .unwrap(),
            Cmd::App { .. }
        ));
        assert_eq!(
            parse_line(":type \"duocj\"").unwrap().unwrap(),
            Cmd::Type("duocj".chars().collect())
        );
        assert_eq!(
            parse_line(":expect \"được\"").unwrap().unwrap(),
            Cmd::Expect("được".into())
        );
    }

    #[test]
    fn parse_inline_comment_and_escapes() {
        let c = parse_line(":caps field_detect,inject_vk,selection     # adapter có UIA")
            .unwrap()
            .unwrap();
        assert_eq!(
            c,
            Cmd::Caps(vec![
                "field_detect".into(),
                "inject_vk".into(),
                "selection".into()
            ])
        );
        assert_eq!(
            parse_line(":type \"a\\\"b\\\\c\\n\"").unwrap().unwrap(),
            Cmd::Type("a\"b\\c\n".chars().collect())
        );
        // # trong chuỗi không phải comment
        assert_eq!(
            parse_line(":expect \"a#b\"").unwrap().unwrap(),
            Cmd::Expect("a#b".into())
        );
    }

    #[test]
    fn parse_errors_are_strict_exit2() {
        assert!(parse_line("type duocj").is_err()); // thiếu :
        assert!(parse_line(":unknownthing").is_err());
        assert!(parse_line(":app chrome.exe").is_err()); // thiếu field=
        assert!(parse_line(":app x field=unknownrole").is_err());
        assert!(parse_line(":config typo_key=1").is_err()); // config key lạ
        assert!(parse_line(":config method=dvorak").is_err()); // value lạ
        assert!(parse_line(":key NotAKey").is_err());
        assert!(parse_line(":expect_action SOMETIMES").is_err());
        assert!(parse_line(":type \"unterminated").is_err());
        assert!(parse_line(":type \"bad \\q escape\"").is_err());
    }

    #[test]
    fn combo_and_mods_parsing() {
        let (mods, vk, ch) = parse_combo("Ctrl+Shift+Space").unwrap();
        assert_eq!(mods, 0x3);
        assert_eq!(vk, 0x20);
        assert_eq!(ch, ' ' as u32);
        assert_eq!(
            parse_combo("Ctrl+T").unwrap(),
            (0x2, 'T' as u32, 'T' as u32)
        );
        assert_eq!(parse_combo("Win+L").unwrap(), (0x8, 'L' as u32, 'L' as u32));
        assert!(parse_combo("Space").is_err()); // thiếu modifier
        assert!(parse_combo("Hyper+Space").is_err());
        assert_eq!(
            parse_line(":mods +Shift").unwrap().unwrap(),
            Cmd::Mods {
                add: true,
                name: "Shift".into()
            }
        );
        assert_eq!(
            parse_line(":mods -Ctrl").unwrap().unwrap(),
            Cmd::Mods {
                add: false,
                name: "Ctrl".into()
            }
        );
    }

    #[test]
    fn key_names_from_spec() {
        for n in [
            "Space",
            "Enter",
            "Escape",
            "Backspace",
            "Tab",
            "Delete",
            "Left",
            "Right",
            "Up",
            "Down",
            "Shift",
            "Ctrl",
            "Alt",
            "Super",
            "CapsLock",
            "F1",
            "F12",
        ] {
            assert!(key_by_name(n).is_some(), "{n} phải hợp lệ");
        }
        assert!(key_by_name("f13").is_none());
        assert_eq!(key_by_name("a"), Some(('A' as u32, 'a' as u32)));
        assert!(key_by_name("Return").is_none()); // chuẩn là Enter
    }

    #[test]
    fn case_without_any_expectation_is_rejected() {
        let dir = std::env::temp_dir().join(format!("textvn-replay-noexp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("vacuous.keys");
        fs::write(&file, ":config method=telex\n:type \"dduocj\"\n").unwrap();
        let (cases, errors) = load_cases(&[file.display().to_string()], None);
        assert!(cases.is_empty());
        assert_eq!(errors.len(), 1, "{errors:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_and_comment_lines_ok() {
        assert!(parse_line("").unwrap().is_none());
        assert!(parse_line("   # comment").unwrap().is_none());
        assert!(parse_line(":note \"giữ chỗ\"").unwrap().is_some());
    }

    #[test]
    fn config_p1_keys_va_json_khop_schema() {
        // P1-6: method mới + `macro`/`emoji` lặp, giá trị quoted chứa dấu cách
        let c = parse_line(
            ":config method=simple_telex macro_trigger=space macro=\"cty=Công ty TNHH\" emoji=\":smile=😀\"",
        )
        .unwrap()
        .unwrap();
        let settings = match c {
            Cmd::Config(s) => s,
            other => panic!("mong Cmd::Config, nhận {other:?}"),
        };
        assert_eq!(settings[0], ("method".into(), "simple_telex".into()));
        assert_eq!(settings[2], ("macro".into(), "cty=Công ty TNHH".into()));

        let json = build_json(&settings);
        assert!(json.contains("\"method\":\"simple_telex\""), "{json}");
        assert!(json.contains("\"macro_trigger\":\"space\""), "{json}");
        assert!(
            json.contains(
                r#""macros":[{"trigger":"cty","expand":"Công ty TNHH","when":"always"}]"#
            ),
            "{json}"
        );
        assert!(
            json.contains(r#""emoji":[{"trigger":":smile","glyph":"😀"}]"#),
            "{json}"
        );

        // phải parse lại được bằng crate config thật (schema khớp, không chỉ là chuỗi)
        let cfg = textvn_config::parse_config(&json).expect("JSON build ra phải hợp lệ");
        assert_eq!(cfg.macros.len(), 1);
        assert_eq!(cfg.macros[0].expand, "Công ty TNHH");
        assert_eq!(cfg.emoji[0].glyph, "😀");
    }

    #[test]
    fn english_words_vao_json_dung_schema() {
        let json = build_json(&[
            ("method".into(), "telex".into()),
            ("english_words".into(), "text,nest".into()),
        ]);
        assert!(
            json.contains(r#""english_words":["text","nest"]"#),
            "json sai schema: {json}"
        );
        // Dấu phẩy thừa (`["a",]`) là JSON hỏng → rc=-2, nên test này chặn đúng lỗi đó.
        let cfg = textvn_config::parse_config(&json).expect("JSON phải parse được");
        assert_eq!(
            cfg.english_words,
            vec!["text".to_string(), "nest".to_string()]
        );
    }

    #[test]
    fn english_words_gi_a_re_ban() {
        assert!(config_english_words("text, nest ").is_ok());
        assert!(config_english_words("").is_ok(), "rỗng = tắt, hợp lệ");
        assert!(
            config_english_words("a-b").is_err(),
            "từ lạ phải báo lỗi tường minh"
        );
        assert!(
            config_english_words("tôi").is_err(),
            "chỉ nhận chữ cái ASCII"
        );
    }

    #[test]
    fn config_p1_bool_flags() {
        for line in [
            ":config auto_capitalize=false",
            ":config auto_restore_english=false",
            ":config allow_macro_when_vi_off=true",
            ":config method=viqr",
            ":config method=vni",
        ] {
            assert!(
                parse_line(line).unwrap().is_some(),
                "phải chấp nhận `{line}`"
            );
        }
        assert!(parse_line(":config auto_capitalize=maybe").is_err());
        assert!(parse_line(":config macro=thiếu-nháy").is_err());
        // `=` cuối → text rỗng → schema config từ chối.
        assert!(parse_line(":config macro=\"khongcoadau=\"").is_err());
    }

    fn parse_test_case(id: &str, lines: &[&str]) -> Case {
        let mut cmds = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if let Some(cmd) = parse_line(line).unwrap() {
                cmds.push((i + 1, cmd));
            }
        }
        Case {
            id: id.into(),
            cmds,
            source: lines.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn sf_1c_canonical_case_rel_chuan_hoa_moi_he_dieu_hanh() {
        assert_eq!(
            canonical_case_rel("corpus\\shared\\telex_basic_01.keys"),
            "shared/telex_basic_01"
        );
        assert_eq!(
            canonical_case_rel("./corpus/win/bug_B1_edge.keys"),
            "win/bug_B1_edge"
        );
        assert_eq!(
            canonical_case_rel("C:\\work\\TextVN\\corpus\\mac\\b11.keys"),
            "mac/b11"
        );
        assert_eq!(canonical_case_rel("/tmp/my_case.keys"), "my_case");
    }

    #[test]
    fn sf_1c_trace_tsf_tat_dinh_va_bat_bien_lenh() {
        let case = parse_test_case(
            "corpus/shared/sample.keys",
            &[
                ":config method=telex",
                ":type \"dduocj\"",
                ":key Space",
                ":type \"a\"",
                ":reset",
                ":type \"tunx\"",
                ":key Ctrl",
                ":secure on",
                ":type \"b\"",
                ":expect \"được atũnb\"",
            ],
        );
        let profile = adapter_profile("tsf");
        let (ok1, f1, _, t1) = run_case(&case, profile, true);
        let (ok2, f2, _, t2) = run_case(&case, profile, true);
        assert!(ok1 && f1.is_empty(), "{f1:?}");
        assert!(ok2 && f2.is_empty(), "{f2:?}");
        let j1 = format_trace_jsonl("shared/sample", t1.as_deref().unwrap());
        let j2 = format_trace_jsonl("shared/sample", t2.as_deref().unwrap());
        assert_eq!(j1, j2, "vết SF-1c phải tất định");
        assert!(!j1.is_empty());

        let mut per_step: std::collections::BTreeMap<u64, Vec<(String, String, u64)>> =
            std::collections::BTreeMap::new();
        for line in j1.lines() {
            let v: serde_json::Value = serde_json::from_str(line).expect("mỗi dòng là JSON hợp lệ");
            assert_eq!(v["case"], "shared/sample");
            let cmd = v["cmd"].as_str().unwrap().to_string();
            assert!(
                matches!(
                    cmd.as_str(),
                    "delete_before" | "commit" | "set_preedit" | "forward_key" | "eat_key"
                ),
                "lệnh ngoài từ vựng SF-1c: {cmd}"
            );
            let step = v["step"].as_u64().unwrap();
            let text = v["text"].as_str().unwrap().to_string();
            let n = v["n"].as_u64().unwrap();
            per_step.entry(step).or_default().push((cmd, text, n));
        }

        // Bất biến P0-6 §3 trên từng step phím (0..14: dduocj + Space + a + tunx + Ctrl + b).
        let mut saw_commit_then_hide = false;
        for step in 0..14u64 {
            let cmds = &per_step[&step];
            let dels = cmds.iter().filter(|(c, _, _)| c == "delete_before").count();
            let commits = cmds.iter().filter(|(c, _, _)| c == "commit").count();
            let key_disp = cmds
                .iter()
                .filter(|(c, _, _)| c == "forward_key" || c == "eat_key")
                .count();
            assert!(dels <= 1, "step {step}: quá 1 delete_before: {cmds:?}");
            assert!(commits <= 1, "step {step}: quá 1 commit: {cmds:?}");
            assert_eq!(key_disp, 1, "step {step}: cần đúng 1 eat/forward: {cmds:?}");
            if let (Some(c_idx), Some(h_idx)) = (
                cmds.iter().position(|(c, _, _)| c == "commit"),
                cmds.iter()
                    .position(|(c, t, _)| c == "set_preedit" && t.is_empty()),
            ) {
                assert!(
                    c_idx < h_idx,
                    "step {step}: commit phải đứng TRƯỚC ẩn preedit: {cmds:?}"
                );
                saw_commit_then_hide = true;
            }
        }
        assert!(
            saw_commit_then_hide,
            "test phải chạm nhánh commit -> ẩn preedit trên cùng step"
        );
        // Step 6 (Space kết thúc "được" trong `apply_tsf`) bắt buộc phát `commit` TRƯỚC `set_preedit("", 0)`.
        assert!(
            per_step[&6]
                .iter()
                .any(|(c, t, _)| c == "commit" && t == "được "),
            "step 6 phải commit 'được ' khi kết thúc composition: {:?}",
            per_step[&6]
        );
    }

    #[test]
    fn sf_1c_emit_keys_xuat_json_hop_le_va_tat_dinh() {
        let case = parse_test_case(
            "corpus/shared/sample.keys",
            &[
                ":config method=vni",
                ":app TextEdit field=body",
                ":caps preedit,selection",
                ":type \"tooi\"",
                ":reset",
                ":enabled off",
                ":type \"ab\"",
                ":expect \"tôiab\"",
            ],
        );
        let j1 = emit_case_keys_json(&case, "shared/sample", adapter_profile("mac"));
        let j2 = emit_case_keys_json(&case, "shared/sample", adapter_profile("mac"));
        assert_eq!(j1, j2);
        let v: serde_json::Value = serde_json::from_str(&j1).expect("emit-keys JSON phải hợp lệ");
        assert_eq!(v["case"], "shared/sample");
        let cfg_str = v["config_json"].as_str().unwrap();
        let cfg_v: serde_json::Value = serde_json::from_str(cfg_str).unwrap();
        assert_eq!(cfg_v["method"], "vni");
        assert_eq!(v["app"]["bundle"], "TextEdit");
        assert_eq!(v["app"]["field_role"], 1);
        assert_eq!(v["app"]["caps"], 3);
        assert_eq!(v["resets"], serde_json::json!([4]));
        assert_eq!(v["keys"].as_array().unwrap().len(), 12);
        let events = v["events"].as_array().unwrap();
        assert!(events
            .iter()
            .any(|e| e["kind"] == "context" && e["enabled"] == false));
    }
}
