// SPDX-License-Identifier: GPL-3.0-or-later
//! `cli/src/replay.rs` — parser corpus `.keys` + simulator (spec: P0-4, bám sát 100%).
//!
//! Exit: 0 = all pass · 1 = có fail · 2 = lỗi dùng/parse (P0-4 §4).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use vietime_ffi::{
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

/// Bảng config scalar hợp lệ cho `:config` (P0-3 §1.1).
fn config_value_ok(key: &str, value: &str) -> Result<(), String> {
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
        | "allow_macro_when_vi_off" => matches!(value, "true" | "false"),
        _ => return Err(format!("config key lạ `{key}`")),
    };
    if ok {
        Ok(())
    } else {
        Err(format!("config value không hợp lệ `{key}={value}`"))
    }
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
                config_value_ok(k, v)?;
                pairs.push((k.to_string(), v.to_string()));
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
            if !matches!(name, "Shift" | "Ctrl" | "Alt" | "Super") {
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
}

struct SimFailure {
    line: usize,
    msg: String,
    expected: Option<String>,
    actual: Option<String>,
}

impl Sim {
    fn new() -> Sim {
        let mut inst: *mut ime_instance = std::ptr::null_mut();
        let rc = ime_instance_new(std::ptr::null(), 0, &mut inst);
        assert_eq!(rc, IME_OK, "ime_instance_new default phải OK");
        Sim {
            inst,
            settings: Vec::new(),
            field: 0,
            caps: 0,
            enabled: true,
            secure: false,
            buf: Vec::new(),
            cursor: 0,
            preedit: String::new(),
            held_mods: 0,
            pending_action: None,
            last_action: "PASS".into(),
            failures: Vec::new(),
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

    /// Gộp `:config k=v` vào settings rồi reload (JSON build thủ công — value đã validate).
    fn apply_config(&mut self, pairs: &[(String, String)], line: usize) {
        for (k, v) in pairs {
            match self.settings.iter_mut().find(|(ek, _)| ek == k) {
                Some(slot) => slot.1 = v.clone(),
                None => self.settings.push((k.clone(), v.clone())),
            }
        }
        let mut json = String::from("{\"config_version\":1");
        for (k, v) in &self.settings {
            json.push(',');
            json.push('"');
            json.push_str(k);
            json.push_str("\":");
            if v == "true" || v == "false" {
                json.push_str(v);
            } else {
                json.push('"');
                json.push_str(v);
                json.push('"');
            }
        }
        json.push('}');
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
        for down in [1u8, 0u8] {
            let k = ime_key_v1 {
                abi_version: IME_ABI_VERSION,
                vk,
                ch,
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
                return;
            }
            self.preedit = decode_utf32(&out.preedit, out.preedit_len);
            if down == 0 {
                continue;
            }
            if !injected {
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

fn build_json(settings: &[(String, String)]) -> String {
    let mut json = String::from("{\"config_version\":1");
    for (k, v) in settings {
        json.push(',');
        json.push('"');
        json.push_str(k);
        json.push_str("\":");
        if v == "true" || v == "false" {
            json.push_str(v);
        } else {
            json.push('"');
            json.push_str(v);
            json.push('"');
        }
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

fn mod_bit(name: &str) -> u32 {
    match name {
        "Shift" => 0x1,
        "Ctrl" => 0x2,
        "Alt" => 0x4,
        "Super" => 0x8,
        _ => 0,
    }
}

/// Chạy 1 case → (pass?, failures, ms).
fn run_case(case: &Case) -> (bool, Vec<SimFailure>, u128) {
    let start = Instant::now();
    let mut sim = Sim::new();
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
            }
            Cmd::EngineNew => {
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
    ime_instance_free(sim.inst);
    (
        sim.failures.is_empty(),
        sim.failures,
        start.elapsed().as_millis(),
    )
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

/// Entrypoint replay (P0-4 §4). Exit: 0/1/2.
pub fn run(paths: &[String], adapter: &str, json: bool, filter: Option<&str>) -> i32 {
    // TODO slice sau: profile win/mac/linux set caps/preset mặc định (P0-4 §4).
    // Hiện tất cả profile chạy cùng engine headless — giữ giá trị để corpus dùng `--adapter win`.
    let _ = adapter;

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
        let (ok, failures, ms) = run_case(case);
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
        assert!(key_by_name("Return").is_none()); // chuẩn là Enter
    }

    #[test]
    fn empty_and_comment_lines_ok() {
        assert!(parse_line("").unwrap().is_none());
        assert!(parse_line("   # comment").unwrap().is_none());
        assert!(parse_line(":note \"giữ chỗ\"").unwrap().is_some());
    }
}
