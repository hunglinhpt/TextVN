// SPDX-License-Identifier: GPL-3.0-or-later
//! REG-1 — sổ hồi quy (P0-5 §9b, P0-7 Luồng C): mọi mã lỗi đã ghi (E*, B*/A*… của Windows
//! harness, R* phát hành, R2-* đã phát hành trong CHANGELOG) phải trỏ tới ≥ 1 bằng chứng chống
//! tái diễn trong `docs/specs/regression-index.md`:
//!
//! | Tham chiếu | Ý nghĩa | Kiểm khi chạy gate |
//! |---|---|---|
//! | `test:<tệp>::<hàm>` | test hồi quy (Rust/C/Swift) | tệp tồn tại VÀ còn khai báo hàm đó ⇒ xoá test = đỏ |
//! | `file:<tệp>` | corpus/fixture, hoặc chương trình test cả tệp (C/C++ `main()` chạy bởi ctest/e2e) | tệp tồn tại |
//! | `gate:<tệp>` | gate tự động (xtask/script/workflow) | tệp tồn tại VÀ còn nhắc mã |
//! | `lesson:<tệp>` | bài học harness Windows (`win/*` — không phải hành vi sản phẩm) | tệp còn nhắc mã |
//! | `process:<mô tả>` | mã chỉ-quy-trình (phát hành, thao tác) | mô tả không rỗng |
//! | `todo:REG-2` | CHƯA có bằng chứng — nợ của REG-2 | đếm; vượt `todo-max` = đỏ (chỉ được giảm) |
//!
//! `cargo xtask regression-index` = gate (preflight + repo-hygiene). `--bootstrap` dựng bảng
//! đầu tiên bằng cách tìm test/corpus/gate đang nhắc mã (tên hàm `r2_174_*`, chú thích `E82`…);
//! `--bootstrap --merge` chỉ thêm dòng cho mã mới, giữ nguyên dòng đã có.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const INDEX: &str = "docs/specs/regression-index.md";
const TODO_MAX_TAG: &str = "<!-- regression-index: todo-max = ";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR luôn nằm trong repo")
        .to_path_buf()
}

// ---------------------------------------------------------------------------------------
// Mã lỗi từ các sổ lỗi
// ---------------------------------------------------------------------------------------

/// `(mã, nguồn)` — mã Windows harness mang tiền tố `win/` (sổ đó có E1–E7 riêng, trùng tên với
/// `project-common-errors`).
pub fn collect_codes(root: &Path) -> BTreeMap<String, &'static str> {
    let mut out = BTreeMap::new();
    let read = |p: &str| fs::read_to_string(root.join(p)).unwrap_or_default();
    for (file, prefix, source) in [
        (
            "docs/specs/project-common-errors.md",
            "",
            "project-common-errors",
        ),
        (
            "docs/specs/win-test-common-errors.md",
            "win/",
            "win-test-common-errors",
        ),
        ("docs/release/release-process.md", "", "release-process"),
    ] {
        for line in read(file).lines() {
            if let Some(code) = table_row_code(line) {
                let keep = match source {
                    "project-common-errors" => code.starts_with('E'),
                    "release-process" => code.starts_with('R'),
                    _ => true,
                };
                if keep {
                    out.insert(format!("{prefix}{code}"), source);
                }
            }
        }
    }
    for code in r2_codes(&read("CHANGELOG.md")) {
        out.insert(code, "CHANGELOG");
    }
    out
}

/// Ô đầu của hàng bảng markdown là một mã `[A-Z]{1,2}\d+[a-z]?` (`| E82 |`, `| B12 |`, `| R7 |`).
fn table_row_code(line: &str) -> Option<String> {
    let rest = line.strip_prefix("| ")?;
    let cell = rest.split(" |").next()?.trim();
    let letters = cell.chars().take_while(|c| c.is_ascii_uppercase()).count();
    if letters == 0 || letters > 2 {
        return None;
    }
    let tail = &cell[letters..];
    let digits = tail.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let suffix = &tail[digits..];
    if suffix.len() > 1 || !suffix.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    Some(cell.to_string())
}

/// Mọi `R2-<số>[chữ]` trong văn bản (CHANGELOG = các lỗi review đã sửa và phát hành).
fn r2_codes(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let b = text.as_bytes();
    let mut i = 0;
    while let Some(off) = text[i..].find("R2-") {
        let start = i + off;
        let mut j = start + 3;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > start + 3 && (start == 0 || !b[start - 1].is_ascii_alphanumeric()) {
            if j < b.len()
                && b[j].is_ascii_lowercase()
                && !(j + 1 < b.len() && b[j + 1].is_ascii_alphanumeric())
            {
                j += 1;
            }
            if j >= b.len() || !b[j].is_ascii_alphanumeric() {
                out.insert(text[start..j].to_string());
            }
        }
        i = start + 3;
    }
    out
}

/// `code` xuất hiện như một từ riêng (`E8` không khớp `E82`/`HE8`/`E8a`).
pub fn mentions(text: &str, code: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0;
    while let Some(off) = text[i..].find(code) {
        let s = i + off;
        let e = s + code.len();
        let before_ok = s == 0 || !(b[s - 1].is_ascii_alphanumeric() || b[s - 1] == b'-');
        // `R1-2` (bất biến số 2) không phải `R1`; `R2-174` vẫn là một mã (do `code` chứa `-`).
        let after_ok = e >= b.len()
            || !(b[e].is_ascii_alphanumeric()
                || (b[e] == b'-' && e + 1 < b.len() && b[e + 1].is_ascii_digit()));
        if before_ok && after_ok {
            return true;
        }
        i = s + 1;
    }
    false
}

/// Tên hàm test mang mã (`r2_174_space…`, `e82_toggle`, `test_r2_26`): mã viết thường, `-` → `_`,
/// đứng ở đầu hoặc sau `_`, theo sau là `_` hoặc hết tên.
fn name_mentions(fn_name: &str, code: &str) -> bool {
    let token = code.to_ascii_lowercase().replace('-', "_");
    let n = fn_name.to_ascii_lowercase();
    let mut i = 0;
    while let Some(off) = n[i..].find(&token) {
        let s = i + off;
        let e = s + token.len();
        let before_ok = s == 0 || n.as_bytes()[s - 1] == b'_';
        let after_ok = e == n.len() || n.as_bytes()[e] == b'_';
        if before_ok && after_ok {
            return true;
        }
        i = s + 1;
    }
    false
}

// ---------------------------------------------------------------------------------------
// Bằng chứng trong repo
// ---------------------------------------------------------------------------------------

struct TestFn {
    file: String,
    name: String,
    /// Tên + khối chú thích/thuộc tính NGAY trên hàm + thân hàm — nơi test ghi mã lỗi nó chặn
    /// (không lấy cửa sổ dòng cố định: sẽ dính thân test liền trước).
    context: String,
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for ent in rd.flatten() {
        let p = ent.path();
        let name = ent.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if matches!(
                name.as_str(),
                "target" | ".git" | "graphify-out" | "dist" | "node_modules" | ".build" | "build"
            ) || name.starts_with("build-")
            {
                continue;
            }
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Tập tệp git-tracked (`git ls-files`) — CHỈ file đã vào git index mới được làm bằng chứng.
/// E88: `--bootstrap` quét toàn cây kể cả file untracked (WIP chưa commit của agent khác) ⇒ doc
/// sinh ra trỏ tới test không tồn tại trên main ⇒ CI đỏ. Fail-closed: không chạy được git ⇒ lỗi
/// thay vì bỏ lọc (bỏ lọc chính là tái diễn E88).
fn tracked_files(root: &Path) -> Result<HashSet<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| {
            format!("E88: không chạy được `git ls-files` ({e}) — bootstrap chỉ chạy trong repo git")
        })?;
    if !out.status.success() {
        return Err("E88: `git ls-files` thất bại — bootstrap chỉ chạy trong repo git".into());
    }
    Ok(out
        .stdout
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .collect())
}

/// Tên hàm test trong một dòng: Rust `fn x(` (sau `#[test]`), C `void test_x(`, Swift `func testX(`.
fn test_fn_name(line: &str, lang: &str, after_test_attr: bool) -> Option<String> {
    let t = line.trim_start();
    let ident = |s: &str| -> Option<String> {
        let id: String = s
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        (!id.is_empty()).then_some(id)
    };
    match lang {
        "rs" if after_test_attr => {
            let s = t.strip_prefix("pub ").unwrap_or(t);
            let s = s.strip_prefix("async ").unwrap_or(s);
            ident(s.strip_prefix("fn ")?)
        }
        "c" => {
            let s = t.strip_prefix("static ").unwrap_or(t);
            let s = s.strip_prefix("void ").or_else(|| s.strip_prefix("int "))?;
            let id = ident(s)?;
            (id.starts_with("test_") && s[id.len()..].starts_with('(')).then_some(id)
        }
        "swift" => {
            let id = ident(t.strip_prefix("func ")?)?;
            id.starts_with("test").then_some(id)
        }
        _ => None,
    }
}

fn collect_tests(root: &Path, files: &[PathBuf]) -> Vec<TestFn> {
    let mut out = Vec::new();
    for p in files {
        let r = rel(root, p);
        let lang = match p.extension().and_then(|e| e.to_str()) {
            Some("rs") => "rs",
            Some("c") if r.contains("/tests/") || r.contains("/test/") => "c",
            Some("swift") if r.contains("Tests/") => "swift",
            _ => continue,
        };
        if r.starts_with("spikes/") || r.ends_with("xtask/src/regression_index.rs") {
            continue; // mã thăm dò / chính gate này (test của nó nhắc mã mẫu) — không phải bằng chứng
        }
        let Ok(text) = fs::read_to_string(p) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        let mut after_attr = false;
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            if lang == "rs" && (t.starts_with("#[test]") || t.starts_with("#[tokio::test")) {
                after_attr = true;
                continue;
            }
            if let Some(name) = test_fn_name(line, lang, after_attr) {
                // Khối ngay trên: chỉ dòng chú thích/thuộc tính liền kề.
                let mut from = i;
                while from > 0 {
                    let t = lines[from - 1].trim_start();
                    if t.starts_with("//")
                        || t.starts_with("#[")
                        || t.starts_with("/*")
                        || t.starts_with('*')
                    {
                        from -= 1;
                    } else {
                        break;
                    }
                }
                // Thân: tới dòng `}` cùng thụt lề với khai báo (tối đa 120 dòng).
                let indent = line.len() - line.trim_start().len();
                let mut to = i + 1;
                while to < lines.len() && to < i + 120 {
                    let l = lines[to];
                    let li = l.len() - l.trim_start().len();
                    to += 1;
                    if li == indent && l.trim_start().starts_with('}') {
                        break;
                    }
                }
                let context = format!(
                    "{name}\n{}\n{}",
                    lines[from..i].join("\n"),
                    lines[i + 1..to].join("\n")
                );
                // Đầu tệp (`//!`, chú thích mở đầu) mô tả cả tệp test — mã nêu ở đó gắn với hàm test
                // ĐẦU TIÊN của tệp (`readme_numbers.rs`: "//! E40 — số liệu README…").
                // Chỉ tệp test thuần (`tests/`, `Tests/`) — đầu tệp `src/*.rs` mô tả mã, không phải test.
                let test_file = r.contains("/tests/") || r.contains("Tests/");
                let first_in_file = test_file && !out.iter().any(|t: &TestFn| t.file == r);
                let header = if first_in_file {
                    lines
                        .iter()
                        .take_while(|l| {
                            let t = l.trim_start();
                            t.is_empty()
                                || t.starts_with("//")
                                || t.starts_with("/*")
                                || t.starts_with('*')
                        })
                        .copied()
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                };
                out.push(TestFn {
                    file: r.clone(),
                    name,
                    context: format!("{context}\n{header}"),
                });
                after_attr = false;
            } else if after_attr && !(t.starts_with("#[") || t.starts_with("//") || t.is_empty()) {
                after_attr = false;
            }
        }
    }
    out
}

/// Tệp gate (xtask, script, workflow) — mã được nhắc ở đó = gate tự động chặn lỗi đó.
fn is_gate_file(r: &str) -> bool {
    (r.starts_with("xtask/src/") && r.ends_with(".rs") && !r.ends_with("regression_index.rs"))
        || (r.starts_with("scripts/") && r.ends_with(".sh"))
        || (r.starts_with("tools/") && (r.ends_with(".ps1") || r.ends_with(".sh")))
        // Test bộ cài/portable Windows (`test-installer.ps1`, `test-portable.ps1`) chạy trên CI.
        || (r.starts_with("installer/windows/tests/") && r.ends_with(".ps1"))
        || (r.starts_with(".github/workflows/") && r.ends_with(".yml"))
        || r == ".gitattributes"
}

fn is_corpus_file(r: &str) -> bool {
    (r.starts_with("corpus/") && !r.ends_with(".md")) || is_program_test(r)
}

/// Chương trình test cả tệp (C/C++ `main()` của ctest/e2e Linux) — bằng chứng dạng `file:`.
fn is_program_test(r: &str) -> bool {
    r.starts_with("adapters/")
        && r.contains("/tests/")
        && (r.ends_with(".c") || r.ends_with(".cpp"))
}

// ---------------------------------------------------------------------------------------
// Bảng chỉ mục
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Ref {
    Test { file: String, func: String },
    File(String),
    Gate(String),
    Lesson(String),
    Process(String),
    Todo,
}

impl Ref {
    fn parse(s: &str) -> Result<Ref, String> {
        let s = s.trim().trim_matches('`');
        let (kind, val) = s
            .split_once(':')
            .ok_or_else(|| format!("tham chiếu thiếu loại: `{s}`"))?;
        let val = val.trim();
        Ok(match kind {
            "test" => {
                let (file, func) = val
                    .rsplit_once("::")
                    .ok_or_else(|| format!("`test:` cần <tệp>::<hàm>: `{s}`"))?;
                Ref::Test {
                    file: file.into(),
                    func: func.into(),
                }
            }
            "file" => Ref::File(val.into()),
            "gate" => Ref::Gate(val.into()),
            "lesson" => Ref::Lesson(val.into()),
            "process" => Ref::Process(val.into()),
            "todo" if val == "REG-2" => Ref::Todo,
            _ => {
                return Err(format!(
                    "loại tham chiếu lạ `{kind}` (test/file/gate/lesson/process/todo:REG-2)"
                ))
            }
        })
    }
    fn render(&self) -> String {
        match self {
            Ref::Test { file, func } => format!("`test:{file}::{func}`"),
            Ref::File(f) => format!("`file:{f}`"),
            Ref::Gate(f) => format!("`gate:{f}`"),
            Ref::Lesson(f) => format!("`lesson:{f}`"),
            Ref::Process(t) => format!("`process:{t}`"),
            Ref::Todo => "`todo:REG-2`".into(),
        }
    }
}

pub struct Index {
    pub todo_max: Option<usize>,
    pub rows: BTreeMap<String, (String, Vec<Ref>)>,
}

pub fn parse_index(text: &str) -> Result<Index, String> {
    let todo_max = text.lines().find_map(|l| {
        let rest = l.trim().strip_prefix(TODO_MAX_TAG)?;
        rest.trim_end_matches("-->").trim().parse().ok()
    });
    let mut rows = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let Some(rest) = line.strip_prefix("| ") else {
            continue;
        };
        let cells: Vec<&str> = rest.split(" | ").collect();
        if cells.len() < 3 {
            continue;
        }
        let code = cells[0].trim().trim_matches('`');
        if code == "Mã" || code.starts_with("---") {
            continue;
        }
        let refs_cell = cells[2].trim().trim_end_matches('|').trim();
        let refs = refs_cell
            .split(" · ")
            .filter(|s| !s.trim().is_empty())
            .map(Ref::parse)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{INDEX}:{}: {e}", n + 1))?;
        if rows
            .insert(code.to_string(), (cells[1].trim().to_string(), refs))
            .is_some()
        {
            return Err(format!("{INDEX}:{}: mã `{code}` lặp", n + 1));
        }
    }
    Ok(Index { todo_max, rows })
}

/// Lỗi của một tham chiếu (None = hợp lệ).
fn check_ref(root: &Path, code: &str, r: &Ref) -> Option<String> {
    let read = |f: &str| fs::read_to_string(root.join(f)).ok();
    match r {
        Ref::Test { file, func } => match read(file) {
            None => Some(format!("{code}: test `{file}` không tồn tại")),
            Some(t) => {
                let declared = t.lines().any(|l| {
                    let lang = if file.ends_with(".rs") {
                        "rs"
                    } else if file.ends_with(".c") {
                        "c"
                    } else {
                        "swift"
                    };
                    test_fn_name(l, lang, true).as_deref() == Some(func.as_str())
                });
                (!declared)
                    .then(|| format!("{code}: test `{file}::{func}` không còn (bị xoá/đổi tên?)"))
            }
        },
        Ref::File(f) => {
            (!root.join(f).is_file()).then(|| format!("{code}: tệp `{f}` không tồn tại"))
        }
        Ref::Gate(f) | Ref::Lesson(f) => match read(f) {
            None => Some(format!("{code}: tệp `{f}` không tồn tại")),
            Some(t) => {
                let bare = code.strip_prefix("win/").unwrap_or(code);
                (!mentions(&t, bare))
                    .then(|| format!("{code}: `{f}` không còn nhắc mã (gate bị gỡ?)"))
            }
        },
        Ref::Process(t) => t
            .trim()
            .is_empty()
            .then(|| format!("{code}: `process:` rỗng")),
        Ref::Todo => None,
    }
}

// ---------------------------------------------------------------------------------------
// Dựng bảng (bootstrap)
// ---------------------------------------------------------------------------------------

fn discover(
    root: &Path,
    codes: &BTreeMap<String, &'static str>,
) -> Result<BTreeMap<String, Vec<Ref>>, String> {
    let tracked = tracked_files(root)?;
    let mut all = Vec::new();
    walk(root, &mut all);
    // E88: chỉ file git-tracked — WIP untracked của agent khác không phải bằng chứng sống trên main.
    let mut files: Vec<PathBuf> = all
        .into_iter()
        .filter(|p| tracked.contains(&rel(root, p)))
        .collect();
    files.sort();
    let tests = collect_tests(root, &files);
    let texts: Vec<(String, String)> = files
        .iter()
        .filter_map(|p| {
            let r = rel(root, p);
            (is_gate_file(&r) || is_corpus_file(&r))
                .then(|| fs::read_to_string(p).ok().map(|t| (r, t)))
                .flatten()
        })
        .collect();
    let mut out = BTreeMap::new();
    for (code, source) in codes {
        let bare = code.strip_prefix("win/").unwrap_or(code);
        let mut refs = Vec::new();
        // Mã phát hành (R*) là sự cố quy trình — bằng chứng là gate/process, không phải test
        // (`R1` trong thân test thường là "round 1"/"rule 1").
        let release = *source == "release-process";
        // Mã một chữ số (`E1`…`E9`, `R1`…`R8`) quá chung để tự tìm (`E1 BB 99` là byte UTF-8, `E1`
        // là nhãn luật assist) — để REG-2 phân loại tay.
        let generic =
            bare.chars().filter(|c| c.is_ascii_digit()).count() < 2 && !bare.contains('-');
        if !code.starts_with("win/") && !generic {
            for t in tests.iter().filter(|_| !release) {
                if refs.len() >= 3 {
                    break;
                }
                if name_mentions(&t.name, bare) || mentions(&t.context, bare) {
                    refs.push(Ref::Test {
                        file: t.file.clone(),
                        func: t.name.clone(),
                    });
                }
            }
            let mut corpus = 0;
            let mut gates = 0;
            for (r, t) in &texts {
                let file_name = r.rsplit('/').next().unwrap_or(r);
                let hit = mentions(t, bare)
                    || name_mentions(file_name.split('.').next().unwrap_or(""), bare);
                if !hit {
                    continue;
                }
                if is_corpus_file(r) && corpus < 2 {
                    refs.push(Ref::File(r.clone()));
                    corpus += 1;
                } else if is_gate_file(r) && gates < 2 && mentions(t, bare) {
                    refs.push(Ref::Gate(r.clone()));
                    gates += 1;
                }
            }
        }
        if refs.is_empty() {
            refs.push(if code.starts_with("win/") {
                Ref::Lesson("docs/specs/win-test-common-errors.md".into())
            } else if *source == "release-process" {
                Ref::Process(
                    "sự cố phát hành — xử lý theo bảng R ở docs/release/release-process.md".into(),
                )
            } else {
                Ref::Todo
            });
        }
        out.insert(code.clone(), refs);
    }
    Ok(out)
}

/// `cap` = `todo-max` cũ khi `--merge`: ratchet chỉ được GIẢM (mã mới không được đẩy nợ lên —
/// gate sẽ đỏ tới khi mã đó có bằng chứng thật).
fn render_index(
    codes: &BTreeMap<String, &'static str>,
    rows: &BTreeMap<String, (String, Vec<Ref>)>,
    cap: Option<usize>,
) -> String {
    let count = rows
        .values()
        .filter(|(_, r)| r.contains(&Ref::Todo))
        .count();
    let todo = cap.map_or(count, |c| c.min(count));
    let mut s = String::new();
    s.push_str("# Sổ hồi quy — mã lỗi đã sửa ↔ bằng chứng chống tái diễn (REG-1)\n\n");
    s.push_str("> Gate: `cargo xtask regression-index` (preflight + `repo-hygiene`). Mỗi mã trong `project-common-errors.md` (E*),\n");
    s.push_str("> `win-test-common-errors.md` (`win/…`), `release-process.md` (R*) và mọi `R2-*` trong `CHANGELOG.md` phải có dòng ở đây với\n");
    s.push_str("> ≥ 1 bằng chứng: `test:<tệp>::<hàm>` (xoá/đổi tên test ⇒ đỏ), `file:<corpus>`, `gate:<tệp xtask/script/workflow còn nhắc mã>`,\n");
    s.push_str("> `lesson:` (chỉ `win/*` — bài học harness, không phải hành vi sản phẩm), `process:<mô tả>` (mã chỉ-quy-trình),\n");
    s.push_str("> hoặc `todo:REG-2` (nợ — số dòng todo không được vượt `todo-max`, REG-2 giảm dần). Nhiều bằng chứng cách nhau ` · `.\n");
    s.push_str("> Mã mới: `cargo xtask regression-index --bootstrap --merge` thêm dòng (tự tìm test nhắc mã), rồi sửa tay cho đúng.\n\n");
    s.push_str(&format!("{TODO_MAX_TAG}{todo} -->\n\n"));
    s.push_str("| Mã | Nguồn | Bằng chứng |\n|---|---|---|\n");
    for (code, (source, refs)) in rows {
        let src = codes.get(code).copied().unwrap_or(source.as_str());
        let r: Vec<String> = refs.iter().map(Ref::render).collect();
        s.push_str(&format!("| {code} | {src} | {} |\n", r.join(" · ")));
    }
    s
}

// ---------------------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------------------

pub fn cli(args: &[String]) -> Result<(), String> {
    let root = repo_root();
    let codes = collect_codes(&root);
    let path = root.join(INDEX);
    let bootstrap = args.iter().any(|a| a == "--bootstrap");
    let merge = args.iter().any(|a| a == "--merge");
    if bootstrap {
        let found = discover(&root, &codes)?;
        let mut rows: BTreeMap<String, (String, Vec<Ref>)> = BTreeMap::new();
        let mut cap = None;
        if merge {
            let text = fs::read_to_string(&path).map_err(|e| format!("{INDEX}: {e}"))?;
            let old = parse_index(&text)?;
            cap = old.todo_max;
            rows = old.rows;
        } else if path.exists() && !args.iter().any(|a| a == "--force") {
            return Err(format!("{INDEX} đã có — dùng `--bootstrap --merge` (thêm mã mới) hoặc `--force` (dựng lại)"));
        }
        let mut added = 0;
        for (code, refs) in found {
            match rows.entry(code.clone()) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    v.insert((codes[&code].to_string(), refs));
                    added += 1;
                }
                // `--merge` nâng cấp dòng còn nợ khi nay đã tìm được bằng chứng thật; dòng sửa tay giữ nguyên.
                std::collections::btree_map::Entry::Occupied(mut o)
                    if o.get().1 == vec![Ref::Todo] && !refs.contains(&Ref::Todo) =>
                {
                    o.get_mut().1 = refs;
                    added += 1;
                }
                _ => {}
            }
        }
        fs::write(&path, render_index(&codes, &rows, cap)).map_err(|e| format!("{INDEX}: {e}"))?;
        println!(
            "regression-index: ghi {INDEX} ({added} dòng mới, {} dòng)",
            rows.len()
        );
    }
    check(&root, &codes, &path)
}

fn check(root: &Path, codes: &BTreeMap<String, &'static str>, path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("{INDEX}: {e} — chạy `cargo xtask regression-index --bootstrap`"))?;
    let idx = parse_index(&text)?;
    let mut errs = Vec::new();
    for code in codes.keys() {
        if !idx.rows.contains_key(code) {
            errs.push(format!("{code}: mã mới chưa có trong {INDEX} — `cargo xtask regression-index --bootstrap --merge` rồi ghi bằng chứng"));
        }
    }
    let (mut tests, mut gates, mut files, mut lessons, mut procs, mut todo) = (0, 0, 0, 0, 0, 0);
    for (code, (_, refs)) in &idx.rows {
        if refs.is_empty() {
            errs.push(format!("{code}: không có bằng chứng nào"));
        }
        if refs.contains(&Ref::Todo) && refs.len() > 1 {
            errs.push(format!(
                "{code}: `todo:REG-2` đi cùng bằng chứng khác — bỏ todo"
            ));
        }
        if refs.iter().any(|r| matches!(r, Ref::Lesson(_))) && !code.starts_with("win/") {
            errs.push(format!("{code}: `lesson:` chỉ dành cho mã `win/*`"));
        }
        for r in refs {
            match r {
                Ref::Test { .. } => tests += 1,
                Ref::Gate(_) => gates += 1,
                Ref::File(_) => files += 1,
                Ref::Lesson(_) => lessons += 1,
                Ref::Process(_) => procs += 1,
                Ref::Todo => todo += 1,
            }
            if let Some(e) = check_ref(root, code, r) {
                errs.push(e);
            }
        }
    }
    let stale: Vec<&String> = idx
        .rows
        .keys()
        .filter(|c| !codes.contains_key(*c))
        .collect();
    match idx.todo_max {
        None => errs.push(format!("{INDEX}: thiếu dòng `{TODO_MAX_TAG}N -->`")),
        Some(max) if todo > max => errs.push(format!(
            "todo:REG-2 = {todo} > todo-max {max} — mã mới phải có bằng chứng thật (không thêm nợ)"
        )),
        _ => {}
    }
    println!(
        "regression-index: {} mã · test {tests} · gate {gates} · corpus {files} · lesson {lessons} · process {procs} · todo:REG-2 {todo}/{}{}",
        idx.rows.len(),
        idx.todo_max.unwrap_or(0),
        if stale.is_empty() {
            String::new()
        } else {
            format!(" · {} dòng mã không còn trong sổ lỗi (giữ được, xem lại)", stale.len())
        }
    );
    if errs.is_empty() {
        Ok(())
    } else {
        for e in &errs {
            eprintln!("  {e}");
        }
        Err(format!("regression-index: {} lỗi", errs.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentions_ranh_gioi_tu() {
        assert!(mentions("chặn E8 tái diễn", "E8"));
        assert!(!mentions("E82", "E8"));
        assert!(!mentions("HE8", "E8"));
        assert!(!mentions("R2-174", "R2-17"));
        assert!(mentions("(R2-174)", "R2-174"));
        assert!(!mentions("bất biến R1-2", "R1"));
        assert!(mentions("R1: pipe", "R1"));
        assert!(!mentions("xR2-174", "R2-174"));
    }

    #[test]
    fn ten_ham_mang_ma() {
        assert!(name_mentions("r2_174_space_complete", "R2-174"));
        assert!(name_mentions("test_e82_toggle", "E82"));
        assert!(!name_mentions("r2_1740_x", "R2-174"));
        assert!(!name_mentions("e820", "E82"));
    }

    #[test]
    fn ma_tu_bang_va_changelog() {
        assert_eq!(
            table_row_code("| E82 | **x** | y |").as_deref(),
            Some("E82")
        );
        assert_eq!(table_row_code("| B12 | a |").as_deref(), Some("B12"));
        assert_eq!(table_row_code("| R2b | a |").as_deref(), Some("R2b"));
        assert_eq!(table_row_code("| # | Lỗi |"), None);
        assert_eq!(table_row_code("| Mã | x |"), None);
        let r = r2_codes("sửa R2-174 và R2-120c, không phải XR2-9 hay R2-");
        assert!(r.contains("R2-174") && r.contains("R2-120c"));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn parse_va_kiem_tham_chieu() {
        let t = "<!-- regression-index: todo-max = 1 -->\n| Mã | Nguồn | Bằng chứng |\n|---|---|---|\n\
                 | E1 | project-common-errors | `test:xtask/src/regression_index.rs::mentions_ranh_gioi_tu` · `todo:REG-2` |\n\
                 | E2 | project-common-errors | `test:xtask/src/regression_index.rs::ham_da_xoa` |\n";
        let idx = parse_index(t).unwrap();
        assert_eq!(idx.todo_max, Some(1));
        let root = repo_root();
        let (_, e1) = &idx.rows["E1"];
        assert!(
            check_ref(&root, "E1", &e1[0]).is_none(),
            "test đang tồn tại phải hợp lệ"
        );
        let (_, e2) = &idx.rows["E2"];
        // Đột biến "xoá test của một mã" ⇒ tham chiếu không còn khai báo ⇒ đỏ.
        assert!(check_ref(&root, "E2", &e2[0])
            .unwrap()
            .contains("không còn"));
        assert!(Ref::parse("`foo:bar`").is_err());
        assert!(Ref::parse("`todo:REG-3`").is_err());
    }

    #[test]
    fn e88_bootstrap_chi_dem_file_git_tracked() {
        // Tái hiện E88: file untracked (WIP chưa commit của agent khác) không được làm bằng chứng.
        if Command::new("git").arg("--version").output().is_err() {
            eprintln!("bỏ qua: không có git trong PATH");
            return;
        }
        let dir = std::env::temp_dir().join(format!("textvn-regidx-e88-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tracked.rs"), "// foo\n").unwrap();
        fs::write(dir.join("untracked_wip.rs"), "// wip của agent khác\n").unwrap();
        let run = |a: &[&str]| {
            let o = Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(a)
                .output()
                .unwrap();
            assert!(o.status.success(), "git {:?} thất bại", a);
        };
        run(&["init", "-q"]);
        run(&["add", "tracked.rs"]);
        let set = tracked_files(&dir).unwrap();
        assert!(
            set.contains("tracked.rs"),
            "file đã `git add` phải được đếm làm bằng chứng"
        );
        assert!(
            !set.contains("untracked_wip.rs"),
            "E88: file untracked phải bị loại khỏi bằng chứng"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
