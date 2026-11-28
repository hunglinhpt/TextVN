// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo board` / `cargo xtask board` — bảng điều phối nhiều agent (G21, `coordination/README.md`).
//!
//! Crate **độc lập với build sản phẩm** (BOARD-1, E83): KHÔNG phụ thuộc `textvn-core` hay crate
//! nào khác — chỉ `std` — nên WIP dở của một agent không chặn điều phối của mọi agent nữa
//! (`cargo run -p textvn-board -- status`, hoặc `cargo install --path board` một lần rồi gọi
//! `cargo board status`). `cargo xtask board` vẫn dùng chung thư viện này (G22 — một nguồn).
//!
//! Một tệp DUY NHẤT `coordination/board.jsonl`, **append-only**, mỗi dòng một sự kiện JSON phẳng
//! (mọi giá trị là chuỗi). Được git theo dõi với `merge=union` (`.gitattributes`) ⇒ agent trên
//! máy khác thấy sau khi push và hai bên cùng nối dòng không bao giờ xung đột; agent chung một
//! checkout thấy NGAY. Mọi lệnh ghi giữ **khoá tệp độc quyền của OS** (`File::lock`) trong suốt
//! đọc-kiểm-ghi ⇒ hai agent không thể cùng nhận một task.
//!
//! Trạng thái (ai giữ task, tệp nào đang được giữ chỗ, thư chưa đọc, ai giữ khoá phát hành) được
//! TÍNH từ chuỗi sự kiện — không sửa dòng cũ. Không crate ngoài (chính sách crate này: chỉ `std`).

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Đường dẫn mặc định (tính từ gốc repo — xtask chạy ở gốc như mọi lệnh khác).
pub const BOARD_PATH: &str = "coordination/board.jsonl";
/// Lease mặc định (giờ) — hết hạn mà không `heartbeat` thì task/tệp/khoá coi như bỏ.
pub const CLAIM_LEASE_H: u64 = 8;
pub const RESERVE_LEASE_H: u64 = 4;
pub const RM_LEASE_H: u64 = 6;

pub type Event = BTreeMap<String, String>;

// ---------------------------------------------------------------- JSON phẳng (chuỗi)

pub fn encode(ev: &Event) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in ev.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_str(&mut out, k);
        out.push(':');
        push_str(&mut out, v);
    }
    out.push('}');
    out
}

fn push_str(out: &mut String, s: &str) {
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
}

/// Đọc một object JSON phẳng chỉ có giá trị chuỗi. Dòng không hợp lệ ⇒ `None` (bỏ qua, cảnh báo).
pub fn decode(line: &str) -> Option<Event> {
    let mut it = line.trim().chars().peekable();
    let mut ev = Event::new();
    skip_ws(&mut it);
    if it.next()? != '{' {
        return None;
    }
    skip_ws(&mut it);
    if it.peek() == Some(&'}') {
        it.next();
        return Some(ev);
    }
    loop {
        skip_ws(&mut it);
        let k = read_str(&mut it)?;
        skip_ws(&mut it);
        if it.next()? != ':' {
            return None;
        }
        skip_ws(&mut it);
        let v = read_str(&mut it)?;
        ev.insert(k, v);
        skip_ws(&mut it);
        match it.next()? {
            ',' => continue,
            '}' => break,
            _ => return None,
        }
    }
    skip_ws(&mut it);
    it.next().is_none().then_some(ev)
}

fn skip_ws(it: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while it.peek().is_some_and(|c| c.is_whitespace()) {
        it.next();
    }
}

fn read_str(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    if it.next()? != '"' {
        return None;
    }
    let mut s = String::new();
    loop {
        match it.next()? {
            '"' => return Some(s),
            '\\' => match it.next()? {
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                '/' => s.push('/'),
                'n' => s.push('\n'),
                'r' => s.push('\r'),
                't' => s.push('\t'),
                'b' => s.push('\u{8}'),
                'f' => s.push('\u{c}'),
                'u' => {
                    let hi = read_hex4(it)?;
                    let cp = if (0xD800..0xDC00).contains(&hi) {
                        if it.next()? != '\\' || it.next()? != 'u' {
                            return None;
                        }
                        let lo = read_hex4(it)?;
                        if !(0xDC00..0xE000).contains(&lo) {
                            return None;
                        }
                        0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                    } else {
                        hi
                    };
                    s.push(char::from_u32(cp)?);
                }
                _ => return None,
            },
            c => s.push(c),
        }
    }
}

fn read_hex4(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<u32> {
    let mut v = 0u32;
    for _ in 0..4 {
        v = v * 16 + it.next()?.to_digit(16)?;
    }
    Some(v)
}

// ---------------------------------------------------------------- thời gian

pub fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `YYYY-MM-DDTHH:MM:SSZ` từ giây Unix (thuật toán civil-from-days, không crate ngoài).
pub fn iso_utc(epoch: u64) -> String {
    let days = (epoch / 86_400) as i64;
    let rem = epoch % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

// ---------------------------------------------------------------- trạng thái tính từ sự kiện

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub agent: String,
    pub since: u64,
    pub expires: u64,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    pub agent: String,
    pub path: String,
    pub task: String,
    pub expires: u64,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    /// task → người giữ (kể cả đã hết hạn — `expires` cho biết).
    pub claims: BTreeMap<String, Claim>,
    /// task → trạng thái cuối khi nhả (`done`/`abandoned`/`blocked`) + commit.
    pub finished: BTreeMap<String, (String, String)>,
    pub reservations: Vec<Reservation>,
    pub rm_lock: Option<Claim>,
    /// agent → vị trí (trong `events` đã sắp) của lần `ack` cuối — thư đứng SAU mới là chưa đọc
    /// (so theo vị trí, không theo giây: thư tới cùng giây sau `ack` vẫn hiện).
    pub last_ack: BTreeMap<String, usize>,
    pub events: Vec<Event>,
}

fn field<'a>(ev: &'a Event, k: &str) -> &'a str {
    ev.get(k).map(String::as_str).unwrap_or("")
}

fn epoch_of(ev: &Event) -> u64 {
    field(ev, "epoch").parse().unwrap_or(0)
}

fn lease_secs(ev: &Event, default_h: u64) -> u64 {
    field(ev, "lease_h")
        .parse::<u64>()
        .unwrap_or(default_h)
        .clamp(1, 72)
        * 3600
}

/// Chuẩn hoá đường dẫn tệp để so giữ chỗ: `\` → `/`, bỏ `./` đầu.
pub fn norm_path(p: &str) -> String {
    let s = p.replace('\\', "/");
    s.trim_start_matches("./").to_string()
}

/// `pattern` giữ chỗ cho `path`? Hỗ trợ: đúng tên, thư mục kết thúc `/`, đuôi `*`.
pub fn path_matches(pattern: &str, path: &str) -> bool {
    let (pat, p) = (norm_path(pattern), norm_path(path));
    if let Some(prefix) = pat.strip_suffix('*') {
        p.starts_with(prefix)
    } else if pat.ends_with('/') {
        p.starts_with(&pat)
    } else {
        p == pat
    }
}

pub fn derive(mut events: Vec<Event>) -> State {
    // merge=union có thể xen dòng của hai máy — sắp ổn định theo thời gian (cùng giây giữ thứ tự tệp).
    events.sort_by_key(epoch_of);
    let mut st = State::default();
    for (idx, ev) in events.iter().enumerate() {
        let t = epoch_of(ev);
        let agent = field(ev, "agent").to_string();
        let task = field(ev, "task").to_string();
        match field(ev, "kind") {
            "claim" => {
                st.finished.remove(&task);
                st.claims.insert(
                    task,
                    Claim {
                        agent,
                        since: t,
                        expires: t + lease_secs(ev, CLAIM_LEASE_H),
                        msg: field(ev, "msg").to_string(),
                    },
                );
            }
            "heartbeat" => {
                if let Some(c) = st.claims.get_mut(&task).filter(|c| c.agent == agent) {
                    c.expires = t + lease_secs(ev, CLAIM_LEASE_H);
                }
                for r in st.reservations.iter_mut().filter(|r| r.agent == agent) {
                    r.expires = r.expires.max(t + RESERVE_LEASE_H * 3600);
                }
            }
            "handoff" => {
                if let Some(c) = st.claims.get_mut(&task) {
                    c.agent = field(ev, "to").to_string();
                    c.since = t;
                    c.expires = t + lease_secs(ev, CLAIM_LEASE_H);
                    c.msg = field(ev, "msg").to_string();
                }
            }
            "release" => {
                st.claims.remove(&task);
                st.finished.insert(
                    task.clone(),
                    (
                        field(ev, "status").to_string(),
                        field(ev, "commit").to_string(),
                    ),
                );
                st.reservations
                    .retain(|r| !(r.agent == agent && r.task == task));
            }
            "reserve" => {
                for p in field(ev, "paths").split(';').filter(|p| !p.is_empty()) {
                    st.reservations
                        .retain(|r| !(r.agent == agent && r.path == norm_path(p)));
                    st.reservations.push(Reservation {
                        agent: agent.clone(),
                        path: norm_path(p),
                        task: task.clone(),
                        expires: t + lease_secs(ev, RESERVE_LEASE_H),
                    });
                }
            }
            "unreserve" => {
                let paths: Vec<String> = field(ev, "paths")
                    .split(';')
                    .filter(|p| !p.is_empty())
                    .map(norm_path)
                    .collect();
                st.reservations.retain(|r| {
                    !(r.agent == agent && (paths.is_empty() || paths.contains(&r.path)))
                });
            }
            "rm-lock" => {
                st.rm_lock = Some(Claim {
                    agent,
                    since: t,
                    expires: t + lease_secs(ev, RM_LEASE_H),
                    msg: field(ev, "msg").to_string(),
                });
            }
            "rm-unlock" => {
                if st.rm_lock.as_ref().is_some_and(|l| l.agent == agent) {
                    st.rm_lock = None;
                }
            }
            "ack" => {
                st.last_ack.insert(agent, idx);
            }
            _ => {}
        }
    }
    st.events = events;
    st
}

/// Thư chưa đọc của `agent` (gửi riêng hoặc `*`, không tính thư chính mình gửi).
pub fn inbox<'a>(st: &'a State, agent: &str) -> Vec<&'a Event> {
    let after = st.last_ack.get(agent).map(|i| i + 1).unwrap_or(0);
    st.events
        .iter()
        .skip(after)
        .filter(|e| {
            matches!(
                field(e, "kind"),
                "notify" | "handoff" | "review-request" | "rm-lock" | "rm-unlock"
            )
        })
        .filter(|e| field(e, "agent") != agent)
        .filter(|e| {
            let to = field(e, "to");
            to == agent || to == "*" || to.is_empty()
        })
        .collect()
}

/// Tệp nào trong `paths` đang được agent KHÁC giữ chỗ (lease còn hạn) tại `now`.
pub fn guard_conflicts(
    st: &State,
    agent: &str,
    paths: &[String],
    now: u64,
) -> Vec<(String, Reservation)> {
    let mut out = Vec::new();
    for p in paths {
        for r in &st.reservations {
            if r.agent != agent && r.expires > now && path_matches(&r.path, p) {
                out.push((norm_path(p), r.clone()));
            }
        }
    }
    out
}

/// Kiểm một lệnh ghi theo trạng thái hiện tại; `Ok(event)` = được phép ghi.
pub fn validate(st: &State, mut ev: Event, now: u64) -> Result<Event, String> {
    let agent = field(&ev, "agent").to_string();
    let task = field(&ev, "task").to_string();
    if agent.is_empty() {
        return Err("thiếu --agent (hoặc biến TEXTVN_AGENT)".into());
    }
    match field(&ev, "kind") {
        "claim" => {
            if task.is_empty() {
                return Err("claim cần mã task".into());
            }
            if let Some(c) = st.claims.get(&task) {
                if c.agent != agent && c.expires > now {
                    return Err(format!(
                        "task {task} đang do `{}` giữ tới {} — nhắn `board notify --to {}` để xin giao lại",
                        c.agent,
                        iso_utc(c.expires),
                        c.agent
                    ));
                }
            }
        }
        "heartbeat" | "release" | "handoff" => {
            let Some(c) = st.claims.get(&task) else {
                return Err(format!("task {task} chưa ai nhận"));
            };
            if c.agent != agent {
                return Err(format!(
                    "task {task} do `{}` giữ, không phải `{agent}`",
                    c.agent
                ));
            }
            if field(&ev, "kind") == "handoff" && field(&ev, "to").is_empty() {
                return Err("handoff cần --to <agent>".into());
            }
            if field(&ev, "kind") == "release"
                && !matches!(field(&ev, "status"), "done" | "abandoned" | "blocked")
            {
                return Err("release cần --status done|abandoned|blocked".into());
            }
        }
        "reserve" => {
            let paths: Vec<String> = field(&ev, "paths")
                .split(';')
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect();
            if paths.is_empty() {
                return Err("reserve cần ít nhất một đường dẫn".into());
            }
            let clash = guard_conflicts(st, &agent, &paths, now);
            if let Some((p, r)) = clash.first() {
                return Err(format!(
                    "`{p}` đang được `{}` giữ chỗ (task {}) tới {}",
                    r.agent,
                    r.task,
                    iso_utc(r.expires)
                ));
            }
        }
        "rm-lock" => {
            if let Some(l) = &st.rm_lock {
                if l.agent != agent && l.expires > now {
                    return Err(format!(
                        "khoá phát hành đang do `{}` giữ tới {} ({})",
                        l.agent,
                        iso_utc(l.expires),
                        l.msg
                    ));
                }
            }
        }
        "notify" | "review-request" => {
            if field(&ev, "msg").is_empty() {
                return Err("cần --msg".into());
            }
        }
        "unreserve" | "ack" | "rm-unlock" | "note" | "review-done" => {}
        other => return Err(format!("loại sự kiện lạ `{other}`")),
    }
    ev.insert("epoch".into(), now.to_string());
    ev.insert("ts".into(), iso_utc(now));
    Ok(ev)
}

// ---------------------------------------------------------------- I/O + CLI

fn board_path() -> PathBuf {
    std::env::var_os("TEXTVN_BOARD")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(BOARD_PATH))
}

fn parse_all(text: &str) -> Vec<Event> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match decode(line) {
            Some(ev) => out.push(ev),
            None => eprintln!("board: bỏ qua dòng {} không hợp lệ", i + 1),
        }
    }
    out
}

fn read_state() -> Result<State, String> {
    let path = board_path();
    let text = match std::fs::File::open(&path) {
        Ok(mut f) => {
            f.lock_shared()
                .map_err(|e| format!("khoá đọc {}: {e}", path.display()))?;
            let mut s = String::new();
            f.read_to_string(&mut s).map_err(|e| e.to_string())?;
            s
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("mở {}: {e}", path.display())),
    };
    Ok(derive(parse_all(&text)))
}

/// Đọc–kiểm–ghi dưới MỘT khoá độc quyền (không có cửa sổ tranh chấp giữa hai agent).
fn append(ev: Event) -> Result<Event, String> {
    let path = board_path();
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut f = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|e| format!("mở {}: {e}", path.display()))?;
    f.lock()
        .map_err(|e| format!("khoá ghi {}: {e}", path.display()))?;
    let mut text = String::new();
    f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    f.read_to_string(&mut text).map_err(|e| e.to_string())?;
    let st = derive(parse_all(&text));
    let ev = validate(&st, ev, now_epoch())?;
    let mut line = encode(&ev);
    line.push('\n');
    if !text.is_empty() && !text.ends_with('\n') {
        line.insert(0, '\n');
    }
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    f.flush().map_err(|e| e.to_string())?;
    Ok(ev)
}

struct Args {
    positional: Vec<String>,
    flags: BTreeMap<String, String>,
}

fn parse_args(args: &[String]) -> Args {
    let mut positional = Vec::new();
    let mut flags = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(name) = a.strip_prefix("--") {
            if name.is_empty() {
                positional.extend(args[i + 1..].iter().cloned());
                break;
            }
            let value = args.get(i + 1).filter(|v| !v.starts_with("--")).cloned();
            if value.is_some() {
                i += 1;
            }
            flags.insert(name.to_string(), value.unwrap_or_else(|| "1".into()));
        } else {
            positional.push(a.clone());
        }
        i += 1;
    }
    Args { positional, flags }
}

fn agent_of(a: &Args) -> String {
    a.flags
        .get("agent")
        .cloned()
        .or_else(|| std::env::var("TEXTVN_AGENT").ok())
        .unwrap_or_default()
}

fn ev(kind: &str, agent: &str) -> Event {
    let mut e = Event::new();
    e.insert("kind".into(), kind.into());
    e.insert("agent".into(), agent.into());
    e
}

fn copy_flags(e: &mut Event, a: &Args, keys: &[&str]) {
    for k in keys {
        if let Some(v) = a.flags.get(*k) {
            let key = if *k == "lease" { "lease_h" } else { k };
            e.insert(key.into(), v.clone());
        }
    }
}

fn print_event(e: &Event) {
    let kind = field(e, "kind");
    let mut line = format!("{}  {:<14} {:<10}", field(e, "ts"), field(e, "agent"), kind);
    for k in ["task", "to", "status", "commit", "paths"] {
        if !field(e, k).is_empty() {
            line.push_str(&format!(" {k}={}", field(e, k)));
        }
    }
    if !field(e, "msg").is_empty() {
        line.push_str(&format!(" — {}", field(e, "msg")));
    }
    println!("{line}");
}

fn print_status(st: &State, now: u64) {
    println!("== Khoá phát hành (G20)");
    match &st.rm_lock {
        Some(l) if l.expires > now => {
            println!("  `{}` giữ tới {} — {}", l.agent, iso_utc(l.expires), l.msg)
        }
        _ => println!("  (trống)"),
    }
    println!("== Task đang nhận");
    if st.claims.is_empty() {
        println!("  (không có)");
    }
    for (task, c) in &st.claims {
        let stale = if c.expires <= now {
            " [HẾT HẠN — nhận lại được]"
        } else {
            ""
        };
        println!(
            "  {task:<10} `{}` từ {} tới {}{stale}{}",
            c.agent,
            iso_utc(c.since),
            iso_utc(c.expires),
            if c.msg.is_empty() {
                String::new()
            } else {
                format!(" — {}", c.msg)
            }
        );
    }
    println!("== Tệp đang giữ chỗ");
    let live: Vec<_> = st.reservations.iter().filter(|r| r.expires > now).collect();
    if live.is_empty() {
        println!("  (không có)");
    }
    for r in live {
        println!(
            "  {:<48} `{}` task {} tới {}",
            r.path,
            r.agent,
            r.task,
            iso_utc(r.expires)
        );
    }
    println!("== Task đã nhả gần đây");
    for (task, (status, commit)) in st.finished.iter().rev().take(10) {
        println!("  {task:<10} {status} {commit}");
    }
}

/// BOARD-1b — tham số của binary `cargo-board`. Cargo gọi lệnh con ngoài theo quy ước
/// `cargo-board board <lệnh> …` (argv[1] = tên lệnh con), nên `cargo board status` từng báo
/// "lệnh con lạ `board`". Bỏ đúng MỘT `board` đứng đầu; chạy thẳng `cargo-board status` hay
/// `cargo run -p textvn-board -- status` giữ nguyên.
pub fn cargo_subcommand_args(argv_after_exe: Vec<String>) -> Vec<String> {
    let mut a = argv_after_exe;
    if a.first().map(String::as_str) == Some("board") {
        a.remove(0);
    }
    a
}

pub fn cli(args: &[String]) -> Result<(), String> {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    let a = parse_args(args.get(1..).unwrap_or(&[]));
    let agent = agent_of(&a);
    let now = now_epoch();
    let task = a.positional.first().cloned().unwrap_or_default();
    let write = |mut e: Event| -> Result<(), String> {
        if !task.is_empty() && !e.contains_key("task") {
            e.insert("task".into(), task.clone());
        }
        let written = append(e)?;
        print_event(&written);
        Ok(())
    };
    match sub {
        "status" => {
            print_status(&read_state()?, now);
            if !agent.is_empty() {
                let st = read_state()?;
                let unread = inbox(&st, &agent);
                println!(
                    "== Hộp thư `{agent}`: {} chưa đọc (`board inbox`)",
                    unread.len()
                );
            }
            Ok(())
        }
        "log" => {
            let st = read_state()?;
            let n: usize = a.flags.get("n").and_then(|v| v.parse().ok()).unwrap_or(30);
            let skip = st.events.len().saturating_sub(n);
            st.events.iter().skip(skip).for_each(print_event);
            Ok(())
        }
        "inbox" => {
            let st = read_state()?;
            let unread = inbox(&st, &agent);
            if unread.is_empty() {
                println!("(không có thư mới)");
            }
            unread.into_iter().for_each(print_event);
            Ok(())
        }
        "claim" | "heartbeat" | "release" | "handoff" => {
            let mut e = ev(sub, &agent);
            copy_flags(&mut e, &a, &["lease", "msg", "status", "commit", "to"]);
            write(e)
        }
        "reserve" | "unreserve" => {
            let mut e = ev(sub, &agent);
            let paths: Vec<String> = a.positional.iter().map(|p| norm_path(p)).collect();
            e.insert("paths".into(), paths.join(";"));
            if let Some(t) = a.flags.get("task") {
                e.insert("task".into(), t.clone());
            }
            copy_flags(&mut e, &a, &["lease", "msg"]);
            let written = append(e)?;
            print_event(&written);
            Ok(())
        }
        "guard" => {
            let st = read_state()?;
            let clash = guard_conflicts(&st, &agent, &a.positional, now);
            for (p, r) in &clash {
                eprintln!(
                    "GUARD: `{p}` đang được `{}` giữ chỗ (task {}) tới {}",
                    r.agent,
                    r.task,
                    iso_utc(r.expires)
                );
            }
            if clash.is_empty() {
                println!("guard: OK ({} tệp)", a.positional.len());
                Ok(())
            } else {
                Err(format!(
                    "{} tệp đang bị agent khác giữ chỗ — KHÔNG commit",
                    clash.len()
                ))
            }
        }
        "notify" | "review-request" | "review-done" | "note" => {
            let mut e = ev(sub, &agent);
            copy_flags(&mut e, &a, &["to", "msg", "commit"]);
            if let Some(t) = a.flags.get("task") {
                e.insert("task".into(), t.clone());
            }
            let written = append(e)?;
            print_event(&written);
            Ok(())
        }
        "ack" | "rm-lock" | "rm-unlock" => {
            let mut e = ev(sub, &agent);
            copy_flags(&mut e, &a, &["msg", "lease"]);
            if sub != "ack" {
                e.insert("to".into(), "*".into());
            }
            let written = append(e)?;
            print_event(&written);
            Ok(())
        }
        "help" | "--help" => {
            print!("{}", usage());
            Ok(())
        }
        other => Err(format!("board: lệnh con lạ `{other}`\n\n{}", usage())),
    }
}

pub fn usage() -> &'static str {
    "cargo board <lệnh> [--agent <id>]   (hoặc đặt TEXTVN_AGENT) — G21, coordination/README.md\n  \
     (= `cargo xtask board …` khi workspace build được; = `cargo run -p textvn-board -- …` khi WIP hỏng)\n  \
     status                       khoá phát hành · task đang nhận · tệp giữ chỗ · số thư chưa đọc\n  \
     inbox | ack                  xem thư gửi mình/`*` chưa đọc | đánh dấu đã đọc tới giờ\n  \
     claim <TASK> [--lease 8] [--msg ..]      nhận task (từ chối nếu agent khác giữ, lease còn hạn)\n  \
     heartbeat <TASK>             gia hạn lease task + tệp giữ chỗ (mỗi ≤ 4 giờ khi đang làm)\n  \
     handoff <TASK> --to <agent> --msg ..     giao lại task (người nhận thấy trong inbox)\n  \
     release <TASK> --status done|abandoned|blocked [--commit <sha>] [--msg ..]\n  \
     reserve <path...> --task <TASK> [--lease 4]   giữ chỗ tệp/thư mục (`dir/`, `prefix*`)\n  \
     unreserve [path...]          bỏ giữ chỗ (không đường dẫn = bỏ hết của mình)\n  \
     guard <path...>              exit 1 nếu tệp đang bị agent khác giữ chỗ (chạy trước commit)\n  \
     notify --to <agent|*> --msg .. [--task T]  gửi thông báo · review-request / review-done tương tự\n  \
     rm-lock --msg vX.Y.Z | rm-unlock          khoá/nhả quyền chạy quy trình phát hành (một agent)\n  \
     log [--n 30]                 30 sự kiện cuối\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_1b_cargo_external_subcommand_argv() {
        let v = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        // `cargo board status --agent a` ⇒ cargo-board nhận ["board","status","--agent","a"].
        assert_eq!(
            cargo_subcommand_args(v(&["board", "status", "--agent", "a"])),
            v(&["status", "--agent", "a"])
        );
        // Gọi thẳng binary / `cargo run -p textvn-board -- status`: giữ nguyên.
        assert_eq!(cargo_subcommand_args(v(&["status"])), v(&["status"]));
        assert_eq!(cargo_subcommand_args(v(&[])), v(&[]));
        // Chỉ bỏ MỘT `board` đứng đầu.
        assert_eq!(cargo_subcommand_args(v(&["board", "board"])), v(&["board"]));
    }

    fn e(pairs: &[(&str, &str)]) -> Event {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn json_roundtrip_with_vietnamese_quotes_and_control_chars() {
        let ev = e(&[
            ("agent", "claude-opus-c4c5"),
            ("msg", "Đã xong \"SF-1a\" \\ path C:\\x\nline2\ttab"),
            ("kind", "note"),
        ]);
        let line = encode(&ev);
        assert!(!line.contains('\n'), "một sự kiện = một dòng");
        assert_eq!(decode(&line).unwrap(), ev);
        assert_eq!(
            decode(r#"{"msg":"\u0110\u1eaf\ud83d\ude00"}"#).unwrap()["msg"],
            "Đắ😀"
        );
        assert!(decode("{\"a\":1}").is_none(), "chỉ nhận giá trị chuỗi");
        assert!(decode("{\"a\":\"b\"} rác").is_none());
        assert_eq!(decode("{}").unwrap(), Event::new());
    }

    #[test]
    fn iso_utc_known_values() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_utc(1_798_416_000), "2026-12-28T00:00:00Z");
    }

    fn claim(agent: &str, task: &str, t: u64) -> Event {
        e(&[
            ("kind", "claim"),
            ("agent", agent),
            ("task", task),
            ("epoch", &t.to_string()),
        ])
    }

    #[test]
    fn claim_is_exclusive_until_lease_expires() {
        let st = derive(vec![claim("a", "SF-1a", 1000)]);
        let try_b = e(&[("kind", "claim"), ("agent", "b"), ("task", "SF-1a")]);
        assert!(validate(&st, try_b.clone(), 1000 + 3600).is_err());
        assert!(
            validate(&st, try_b, 1000 + CLAIM_LEASE_H * 3600 + 1).is_ok(),
            "hết lease ⇒ nhận lại được"
        );
        let again = e(&[("kind", "claim"), ("agent", "a"), ("task", "SF-1a")]);
        assert!(
            validate(&st, again, 2000).is_ok(),
            "chính chủ nhận lại = gia hạn"
        );
    }

    #[test]
    fn heartbeat_extends_and_handoff_transfers_ownership() {
        let hb = e(&[
            ("kind", "heartbeat"),
            ("agent", "a"),
            ("task", "T"),
            ("epoch", "20000"),
        ]);
        let st = derive(vec![claim("a", "T", 1000), hb]);
        assert_eq!(st.claims["T"].expires, 20_000 + CLAIM_LEASE_H * 3600);
        let ho = e(&[
            ("kind", "handoff"),
            ("agent", "a"),
            ("task", "T"),
            ("to", "b"),
            ("epoch", "21000"),
        ]);
        let st = derive(vec![claim("a", "T", 1000), ho]);
        assert_eq!(st.claims["T"].agent, "b");
        let b_hb = e(&[("kind", "heartbeat"), ("agent", "b"), ("task", "T")]);
        assert!(validate(&st, b_hb, 22_000).is_ok());
        let a_hb = e(&[("kind", "heartbeat"), ("agent", "a"), ("task", "T")]);
        assert!(
            validate(&st, a_hb, 22_000).is_err(),
            "đã giao thì không còn quyền"
        );
    }

    #[test]
    fn release_requires_owner_and_status() {
        let st = derive(vec![claim("a", "T", 1000)]);
        let bad = e(&[("kind", "release"), ("agent", "a"), ("task", "T")]);
        assert!(validate(&st, bad, 2000).is_err());
        let other = e(&[
            ("kind", "release"),
            ("agent", "b"),
            ("task", "T"),
            ("status", "done"),
        ]);
        assert!(validate(&st, other, 2000).is_err());
        let ok = e(&[
            ("kind", "release"),
            ("agent", "a"),
            ("task", "T"),
            ("status", "done"),
            ("commit", "abc123"),
            ("epoch", "3000"),
        ]);
        let st = derive(vec![claim("a", "T", 1000), ok]);
        assert!(st.claims.is_empty());
        assert_eq!(st.finished["T"], ("done".to_string(), "abc123".to_string()));
    }

    #[test]
    fn reservations_guard_other_agents_only() {
        let res = e(&[
            ("kind", "reserve"),
            ("agent", "a"),
            ("task", "SF-2"),
            (
                "paths",
                "session/;adapters\\linux-common/src/compose.c;docs/specs/x*",
            ),
            ("epoch", "1000"),
        ]);
        let st = derive(vec![res]);
        let paths = vec![
            "session/src/lib.rs".to_string(),
            "./adapters/linux-common/src/compose.c".to_string(),
            "docs/specs/xyz.md".to_string(),
            "README.md".to_string(),
        ];
        assert_eq!(guard_conflicts(&st, "b", &paths, 2000).len(), 3);
        assert!(
            guard_conflicts(&st, "a", &paths, 2000).is_empty(),
            "chính chủ không bị chặn"
        );
        assert!(
            guard_conflicts(&st, "b", &paths, 1000 + RESERVE_LEASE_H * 3600 + 1).is_empty(),
            "hết lease ⇒ không chặn"
        );
        let clash = e(&[
            ("kind", "reserve"),
            ("agent", "b"),
            ("paths", "session/src/lib.rs"),
        ]);
        assert!(validate(&st, clash, 2000).is_err());
        let un = e(&[
            ("kind", "unreserve"),
            ("agent", "a"),
            ("paths", ""),
            ("epoch", "1500"),
        ]);
        let st2 = derive(vec![
            e(&[
                ("kind", "reserve"),
                ("agent", "a"),
                ("paths", "session/"),
                ("epoch", "1000"),
            ]),
            un,
        ]);
        assert!(
            st2.reservations.is_empty(),
            "unreserve không đường dẫn = bỏ hết"
        );
    }

    #[test]
    fn inbox_shows_unread_direct_and_broadcast_messages() {
        let n1 = e(&[
            ("kind", "notify"),
            ("agent", "a"),
            ("to", "b"),
            ("msg", "SF-1a xong"),
            ("epoch", "100"),
        ]);
        let n2 = e(&[
            ("kind", "notify"),
            ("agent", "c"),
            ("to", "*"),
            ("msg", "đừng sửa ffi.h"),
            ("epoch", "200"),
        ]);
        let n3 = e(&[
            ("kind", "notify"),
            ("agent", "a"),
            ("to", "c"),
            ("msg", "riêng c"),
            ("epoch", "300"),
        ]);
        let ack = e(&[("kind", "ack"), ("agent", "b"), ("epoch", "150")]);
        let st = derive(vec![n1, n2, n3, ack]);
        let unread: Vec<&str> = inbox(&st, "b").iter().map(|e| field(e, "msg")).collect();
        assert_eq!(unread, vec!["đừng sửa ffi.h"]);
        assert!(inbox(&st, "c").iter().all(|e| field(e, "agent") != "c"));
    }

    #[test]
    fn release_manager_lock_is_exclusive() {
        let lock = e(&[
            ("kind", "rm-lock"),
            ("agent", "a"),
            ("msg", "v0.2.94"),
            ("epoch", "1000"),
        ]);
        let st = derive(vec![lock.clone()]);
        let b = e(&[("kind", "rm-lock"), ("agent", "b"), ("msg", "v0.2.95")]);
        assert!(validate(&st, b.clone(), 2000).is_err());
        let unlock = e(&[("kind", "rm-unlock"), ("agent", "a"), ("epoch", "3000")]);
        let st = derive(vec![lock, unlock]);
        assert!(st.rm_lock.is_none());
        assert!(validate(&st, b, 4000).is_ok());
    }

    #[test]
    fn union_merged_lines_are_ordered_by_epoch() {
        let text = format!(
            "{}\n{}\n",
            encode(&claim("b", "T", 2000)),
            encode(&claim("a", "T", 1000))
        );
        let st = derive(parse_all(&text));
        assert_eq!(
            st.claims["T"].agent, "b",
            "sự kiện mới hơn thắng dù đứng trước"
        );
    }

    #[test]
    fn append_round_trip_on_temp_board() {
        let dir = std::env::temp_dir().join(format!("textvn-board-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("board.jsonl");
        // Không dùng biến môi trường (test song song): gọi phần lõi trực tiếp.
        std::fs::create_dir_all(&dir).unwrap();
        let st = derive(Vec::new());
        let ev = validate(
            &st,
            e(&[("kind", "claim"), ("agent", "a"), ("task", "SF-1a")]),
            now_epoch(),
        )
        .unwrap();
        std::fs::write(&path, format!("{}\n", encode(&ev))).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let st = derive(parse_all(&text));
        assert_eq!(st.claims["SF-1a"].agent, "a");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
