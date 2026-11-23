// SPDX-License-Identifier: GPL-3.0-or-later
//! Các gate hygiene của repo — bản Rust thay thế `.github/scripts/*.py` (E59:
//! policy "chỉ Rust + C/C++/Swift" cấm script Python trong repo; 5 script
//! hygiene dưới đây chuyển vào xtask, `.py` bị xoá và gate `no-python` chặn
//! tái diễn).
//!
//! Mỗi hàm = 1 lệnh `cargo xtask <cmd>`, dùng chung bởi:
//!   - `.github/workflows/repo-hygiene.yml` (CI), và
//!   - `xtask/src/preflight.rs` (gate local trước commit, luật E13).
//!
//! Port 1:1 ngữ nghĩa của script Python cũ — KHÔNG thêm rule mới trong bản
//! chuyển: hành vi từng gate giữ nguyên để lịch sử CI sóng đôi được.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Gốc repo (xtask nằm ở `<root>/xtask`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR luôn nằm trong repo")
        .to_path_buf()
}

/// `git ls-files -z --cached --others --exclude-standard [-- patterns...]` —
/// quét CẢ file mới chưa track (không bị .gitignore), port nguyên tắc R2-112.
/// Dùng `-z` để vô hiệu hoá `core.quotepath` (tránh bọc ngoặc/octal với tên
/// file tiếng Việt / có khoảng trắng) và khử trùng lặp qua `BTreeSet`.
fn git_list(patterns: &[&str]) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    cmd.arg("ls-files")
        .arg("-z")
        .arg("--cached")
        .arg("--others")
        .arg("--exclude-standard");
    if !patterns.is_empty() {
        cmd.arg("--");
        for p in patterns {
            cmd.arg(p);
        }
    }
    let out = cmd
        .current_dir(repo_root())
        .output()
        .map_err(|e| format!("chạy `git ls-files`: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "`git ls-files` exit {}: {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let unique: BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    Ok(unique.into_iter().collect())
}

// ---------------------------------------------------------------------------
// hygiene-docs — port check_doc_links.py
// ---------------------------------------------------------------------------

/// Link markdown nội bộ dạng `](target)` — thay regex `\]\(([^)\s]+)\)`:
/// sau `](` lấy tối đa các ký tự không phải `)` và không phải khoảng trắng,
/// và BẮT BUỘC kết thúc ngay bằng `)` (khớp 1:1 regex Python).
fn doc_link_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            let start = i + 2;
            let mut end = start;
            for (rel, ch) in text[start..].char_indices() {
                if ch == ')' || ch.is_whitespace() || ch == '\x0B' {
                    end = start + rel;
                    break;
                }
                end = start + rel + ch.len_utf8();
            }
            if end > start && bytes.get(end) == Some(&b')') {
                out.push(text[start..end].to_string());
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Gom mọi file `docs/**/*.md` (đệ quy, có cả `docs/a.md` như `**` của
/// pathlib) — không đi theo symlink thư mục (tránh vòng lặp vô hạn).
fn collect_md(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            let p = entry.path();
            if ft.is_dir() {
                walk(&p, out);
            } else if ft.is_file() && p.extension().is_some_and(|e| e == "md") {
                out.push(p);
            }
        }
    }
    let mut files = vec![root.join("PLAN.md"), root.join("README.md")];
    let mut docs = Vec::new();
    walk(&root.join("docs"), &mut docs);
    docs.sort();
    files.extend(docs);
    files
}

/// Gate: link markdown tương đối không hỏng (00-WORKFLOW §12).
pub fn docs_links() -> Result<(), String> {
    let root = repo_root();
    let mut checked = 0usize;
    let mut scanned = 0usize;
    let mut broken = Vec::new();
    for md in collect_md(&root) {
        if !md.is_file() {
            continue;
        }
        scanned += 1;
        let bytes = fs::read(&md).map_err(|e| format!("không đọc được `{}`: {e}", md.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        for target in doc_link_targets(&text) {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with('#')
                || target.starts_with("mailto:")
                || target.starts_with("tel:")
            {
                continue;
            }
            let path = target.split('#').next().unwrap_or("");
            if path.is_empty() {
                continue;
            }
            checked += 1;
            if !md.parent().map(|p| p.join(path).exists()).unwrap_or(false) {
                let rel = md
                    .strip_prefix(&root)
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| md.to_string_lossy().to_string());
                broken.push(format!("{rel}: {target}"));
            }
        }
    }
    println!("check_doc_links: {checked} relative link trong {scanned} file md");
    if broken.is_empty() {
        println!("check_doc_links: OK");
        Ok(())
    } else {
        let mut msg = String::from("BROKEN relative links:");
        for line in &broken {
            msg.push_str(&format!("\n  {line}"));
        }
        Err(msg)
    }
}

// ---------------------------------------------------------------------------
// hygiene-secrets — port check_no_secrets.py
// ---------------------------------------------------------------------------

const SECRET_EXTS: &[&str] = &[
    ".key",
    ".pem",
    ".p12",
    ".pfx",
    ".pkcs12",
    ".keystore",
    ".sec",
    ".credentials",
    ".secret",
];

const SECRET_NAME_PREFIXES: &[&str] = &[".env", "id_rsa", "id_ed25519", "textvn_signing_ed25519"];

/// Đầu ra `-----BEGIN <chữ in hoa/số/giữa cách>PRIVATE KEY[ BLOCK]-----`.
/// Với regex Python, `[A-Z0-9 ]*` greedy chạy qua chính cụm "PRIVATE KEY"
/// (các ký tự đó cũng thuộc class) rồi backtracking — ở đây tìm trong toàn
/// dải class một vị trí "PRIVATE KEY" (+ tuỳ chọn " BLOCK") theo sau là
/// `-----`: tồn tại bất kỳ đâu trong dải là hit (đủ cho gate exist/not-exist).
fn find_private_key_headers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    const BEGIN: &[u8] = b"-----BEGIN ";
    let mut i = 0;
    while let Some(pos) = text[i..].find("-----BEGIN ").map(|p| i + p) {
        let mut j = pos + BEGIN.len();
        let run_start = j;
        while j < bytes.len()
            && (bytes[j].is_ascii_uppercase() || bytes[j].is_ascii_digit() || bytes[j] == b' ')
        {
            j += 1;
        }
        let mut matched_end = None;
        let mut search_pos = run_start;
        while search_pos < j {
            let Some(off) = text[search_pos..j]
                .find("PRIVATE KEY")
                .map(|p| search_pos + p)
            else {
                break;
            };
            let mut end = off + "PRIVATE KEY".len();
            if text[end..].starts_with(" BLOCK") {
                end += " BLOCK".len();
            }
            if text[end..].starts_with("-----") {
                out.push(text[pos..end + 5].to_string());
                matched_end = Some(end + 5);
                break;
            }
            search_pos = off + 1;
        }
        i = matched_end.unwrap_or(pos + BEGIN.len());
    }
    out
}

/// Token GitHub: `(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}` hoặc
/// `github_pat_[A-Za-z0-9_]{30,}`.
fn find_github_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
        let mut i = 0;
        while let Some(pos) = text[i..].find(prefix).map(|p| i + p) {
            let start = pos + prefix.len();
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_alphanumeric() {
                end += 1;
            }
            if end - start >= 30 {
                out.push(text[pos..end].to_string());
                i = end;
            } else {
                i = pos + 1;
            }
        }
    }
    let mut i = 0;
    while let Some(pos) = text[i..].find("github_pat_").map(|p| i + p) {
        let start = pos + "github_pat_".len();
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        if end - start >= 30 {
            out.push(text[pos..end].to_string());
            i = end;
        } else {
            i = pos + 1;
        }
    }
    out
}

/// AWS Access Key: `\bAKIA[0-9A-Z]{16}\b` — ranh giới từ hai bên.
fn find_aws_keys(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    const W: fn(u8) -> bool = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while let Some(pos) = text[i..].find("AKIA").map(|p| i + p) {
        let before_ok = pos == 0 || !W(bytes[pos - 1]);
        let start = pos + 4;
        let end = start + 16;
        let run_ok = end <= bytes.len()
            && bytes[start..end]
                .iter()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
        let after_ok = end >= bytes.len() || !W(bytes[end]);
        if before_ok && run_ok && after_ok {
            out.push(text[pos..end].to_string());
            i = end;
        } else {
            i = pos + 1;
        }
    }
    out
}

/// Đọc tối đa 1KB đầu để lọc file nhị phân (chứa byte NUL) mà không nạp cả file
/// lớn vào RAM; chỉ đọc phần còn lại nếu là file văn bản.
fn read_text_file_if_non_binary(full: &Path) -> Result<Option<Vec<u8>>, String> {
    let mut f = File::open(full).map_err(|e| format!("không mở được `{}`: {e}", full.display()))?;
    let mut head = [0u8; 1024];
    let n = f
        .read(&mut head)
        .map_err(|e| format!("không đọc được `{}`: {e}", full.display()))?;
    if head[..n].contains(&0) {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(n);
    bytes.extend_from_slice(&head[..n]);
    f.read_to_end(&mut bytes)
        .map_err(|e| format!("không đọc được `{}`: {e}", full.display()))?;
    Ok(Some(bytes))
}

/// Gate: secret / token / private key không được commit (R2-112: quét cả file
/// chưa track; bỏ qua `graphify-out/` là cache graph đã kiểm soát).
pub fn no_secrets() -> Result<(), String> {
    let root = repo_root();
    let tracked: Vec<String> = git_list(&[])?
        .into_iter()
        .filter(|f| !f.starts_with("graphify-out/"))
        .collect();
    if tracked.is_empty() {
        return Err("check_no_secrets: Khong lay duoc danh sach git tracked files".to_string());
    }

    let mut errors = Vec::new();
    let mut scanned = 0usize;
    for rel in &tracked {
        let p = PathBuf::from(rel);
        let filename = p
            .file_name()
            .map(|f| f.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ext = p
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();
        if SECRET_EXTS.contains(&ext.as_str()) {
            errors.push(format!("File duoi cam bi track vao git: {rel}"));
            continue;
        }
        if SECRET_NAME_PREFIXES
            .iter()
            .any(|pref| filename.starts_with(pref))
        {
            errors.push(format!("File mang ten nhay cam bi track vao git: {rel}"));
            continue;
        }
        let full = root.join(&p);
        if !full.is_file() {
            continue;
        }
        let Some(bytes) = read_text_file_if_non_binary(&full)? else {
            continue;
        };
        scanned += 1;
        // Port đọc nội dung lossy, giống `errors="replace"` của bản Python.
        let content = String::from_utf8_lossy(&bytes);
        // Chỉ miễn trừ chính file định nghĩa detector & unit-test mẫu.
        if rel == "xtask/src/hygiene.rs" {
            continue;
        }
        for (rule, hits) in [
            ("Private Key Header", find_private_key_headers(&content)),
            ("GitHub Token", find_github_tokens(&content)),
            ("AWS Access Key", find_aws_keys(&content)),
        ] {
            for text in hits {
                let head15: String = text.chars().take(15).collect();
                errors.push(format!("{rel}: Phat hien {rule} (doan khop: {head15}...)"));
            }
        }
    }
    println!("check_no_secrets: Da quet {scanned} files text trong repo");
    if errors.is_empty() {
        println!("check_no_secrets: OK (khong co private key hoac token nao bi ro ri)");
        Ok(())
    } else {
        let mut msg = String::from("Phat hien nguy co ro ri security/private key:");
        for e in &errors {
            msg.push_str(&format!("\n  [CANH BAO] {e}"));
        }
        Err(msg)
    }
}

// ---------------------------------------------------------------------------
// hygiene-apis — port check_no_injection_apis.py (Farch-3)
// ---------------------------------------------------------------------------

const BANNED_APIS: &[&str] = &[
    // thread/process injection
    "CreateRemoteThread",
    "RtlCreateUserThread",
    "QueueUserAPC",
    "SetThreadContext",
    "Wow64SetContextThread",
    "NtSetContextThread",
    // đọc/ghi bộ nhớ process khác
    "WriteProcessMemory",
    "VirtualAllocEx",
    "NtWriteVirtualMemory",
    "process_vm_writev",
    "PTRACE_ATTACH",
];

const INJECTION_CODE_SUFFIXES: &[&str] = &[
    ".rs", ".c", ".h", ".cpp", ".hpp", ".cc", ".py", ".ps1", ".sh", ".iss",
];

/// Tìm `word` với ranh giới từ (port `\bWORD\b`): ký tự hai bên không thuộc
/// `[A-Za-z0-9_]` (hoặc đầu/cuối chuỗi).
fn contains_word(line: &str, word: &str) -> bool {
    let bytes = line.as_bytes();
    const W: fn(u8) -> bool = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while let Some(pos) = line[i..].find(word).map(|p| i + p) {
        let start_ok = pos == 0 || !W(bytes[pos - 1]);
        let end = pos + word.len();
        let end_ok = end >= bytes.len() || !W(bytes[end]);
        if start_ok && end_ok {
            return true;
        }
        i = pos + 1;
    }
    false
}

/// `\bSetWindowsHookEx\w*\s*\(` — nhận cả `SetWindowsHookExW(`.
fn contains_set_hook_call(line: &str) -> bool {
    let bytes = line.as_bytes();
    const W: fn(u8) -> bool = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    const WORD: &[u8] = b"SetWindowsHookEx";
    let mut i = 0;
    while let Some(pos) = line[i..].find("SetWindowsHookEx").map(|p| i + p) {
        let start_ok = pos == 0 || !W(bytes[pos - 1]);
        let mut j = pos + WORD.len();
        while j < bytes.len() && W(bytes[j]) {
            j += 1;
        }
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if start_ok && j < bytes.len() && bytes[j] == b'(' {
            return true;
        }
        i = pos + 1;
    }
    false
}

fn scan_injection_line(rel: &str, lineno: usize, line: &str, violations: &mut Vec<String>) {
    for name in BANNED_APIS {
        if line.contains(name) {
            violations.push(format!("{rel}:{lineno}: {name}"));
        }
    }
    let non_ll_hook = contains_word(line, "WH_KEYBOARD") || contains_word(line, "WH_MOUSE");
    let ll_ok = contains_word(line, "WH_KEYBOARD_LL") || contains_word(line, "WH_MOUSE_LL");
    if non_ll_hook {
        violations.push(format!(
            "{rel}:{lineno}: hook non-LL (chi WH_KEYBOARD_LL/WH_MOUSE_LL duoc phep)"
        ));
    } else if contains_set_hook_call(line) && !ll_ok {
        violations.push(format!(
            "{rel}:{lineno}: SetWindowsHookEx khong ghi ro LL hook tren cung dong"
        ));
    }
}

/// Gate: API inject/giống malware bị cấm (antivirus-false-positive.md §8).
/// `spikes/` = probe không phát hành (hook-spike, WIN-005) — guard áp cho
/// code phát hành, copy vào `adapters/` vẫn bị chặn tại đó.
pub fn no_injection_apis() -> Result<(), String> {
    let root = repo_root();
    let files = git_list(&[])?;

    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for rel in &files {
        if rel == "xtask/src/hygiene.rs" || rel.starts_with("spikes/") {
            continue;
        }
        let ext = Path::new(rel)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
            .unwrap_or_default();
        if !INJECTION_CODE_SUFFIXES.contains(&ext.as_str()) {
            continue;
        }
        let path = root.join(rel);
        if !path.is_file() {
            continue;
        }
        scanned += 1;
        // Đọc byte thô + lossy UTF-8 (khớp `errors="replace"` của Python) — không bỏ qua file non-UTF-8.
        let bytes = fs::read(&path).map_err(|e| format!("không đọc được `{rel}`: {e}"))?;
        let text = String::from_utf8_lossy(&bytes);
        for (lineno, line) in text.lines().enumerate() {
            scan_injection_line(rel, lineno + 1, line, &mut violations);
        }
    }
    if violations.is_empty() {
        println!("check_no_injection_apis: OK ({scanned} file code, khong co API bi cam)");
        Ok(())
    } else {
        let mut msg =
            String::from("API inject/giong malware bi cam (antivirus-false-positive.md §8):");
        for v in &violations {
            msg.push_str(&format!("\n  {v}"));
        }
        Err(msg)
    }
}

// ---------------------------------------------------------------------------
// iss-tabs — port check_iss_tabs.py (B12/B13)
// ---------------------------------------------------------------------------

fn find_tab_lines(bytes: &[u8]) -> Vec<usize> {
    bytes
        .split(|&b| b == b'\n')
        .enumerate()
        .filter_map(|(idx, line)| line.contains(&b'\t').then_some(idx + 1))
        .collect()
}

/// Gate: `.iss` không chứa TAB — escape `\t` từng bị ghi thành TAB thật phá
/// pattern/đường dẫn `{app}\textvn-tsf.dll` trong [UninstallDelete]; ISCC
/// compile vẫn XANH nên gate này là lớp duy nhất bắt được.
pub fn iss_tabs() -> Result<(), String> {
    let root = repo_root();
    let files = git_list(&["*.iss"])?;
    let mut bad = Vec::new();
    let mut scanned = 0usize;
    for path in &files {
        let full = root.join(path);
        if !full.is_file() {
            continue;
        }
        scanned += 1;
        let bytes = fs::read(&full).map_err(|e| format!("không đọc được `{path}`: {e}"))?;
        for lineno in find_tab_lines(&bytes) {
            bad.push(format!("{path}:{lineno}"));
        }
    }
    if bad.is_empty() {
        println!("check_iss_tabs: OK ({scanned} file .iss khong co TAB)");
        Ok(())
    } else {
        let mut msg = String::from(".iss chua ky tu TAB (escape \\t bi ghi sai):");
        for loc in &bad {
            msg.push_str(&format!("\n  {loc}"));
        }
        Err(msg)
    }
}

// ---------------------------------------------------------------------------
// ascii-ps1 — port check_ps1_ascii.py (G7/A5)
// ---------------------------------------------------------------------------

/// Gate: `.ps1` ASCII-only — PowerShell 5.1 đọc script không BOM theo ANSI
/// codepage của máy, ký tự ngoài ASCII bị mojibake và có thể phá cú pháp
/// (sự cố 2026-10-03: comment tiếng Việt trong test-portable.ps1).
pub fn ascii_ps1() -> Result<(), String> {
    let root = repo_root();
    let files = git_list(&["*.ps1"])?;
    let mut bad = Vec::new();
    let mut scanned = 0usize;
    for path in &files {
        let full = root.join(path);
        if !full.is_file() {
            continue;
        }
        scanned += 1;
        let data = fs::read(&full).map_err(|e| format!("không đọc được `{path}`: {e}"))?;
        if data.iter().any(|b| *b > 0x7F) {
            bad.push(path.clone());
        }
    }
    if bad.is_empty() {
        println!("check_ps1_ascii: OK ({scanned} file .ps1 ASCII-only)");
        Ok(())
    } else {
        let mut msg = String::from(".ps1 phải ASCII-only (G7/A5):");
        for p in &bad {
            msg.push_str(&format!("\n  {p}"));
        }
        Err(msg)
    }
}

const PYTHON_EXTS: &[&str] = &["py", "pyw", "pyi", "ipynb"];

fn is_python_path(rel: &str) -> bool {
    Path::new(rel)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .is_some_and(|e| PYTHON_EXTS.contains(&e.as_str()))
}

fn has_python_shebang(head: &[u8]) -> bool {
    if head.contains(&0) || !head.starts_with(b"#!") {
        return false;
    }
    let first_line = head.split(|&b| b == b'\n').next().unwrap_or(&[]);
    let line = String::from_utf8_lossy(first_line).to_ascii_lowercase();
    line.contains("python")
}

/// E59: repo KHÔNG được chứa file Python (policy ngôn ngữ AGENTS.md — chỉ Rust +
/// C/C++/Swift native). Quét cả file track lẫn file mới chưa track (không bị
/// gitignore), không phân biệt hoa/thường (.py/.pyw/.pyi/.ipynb) và kiểm
/// shebang `#!/...python` ở script không đuôi.
pub fn no_python() -> Result<(), String> {
    let root = repo_root();
    let mut hits = Vec::new();
    for rel in git_list(&[])? {
        if is_python_path(&rel) {
            hits.push(rel);
            continue;
        }
        if rel.starts_with("graphify-out/") || Path::new(&rel).extension().is_some() {
            continue;
        }
        let full = root.join(&rel);
        if !full.is_file() {
            continue;
        }
        let Ok(mut f) = File::open(&full) else {
            continue;
        };
        let mut head = [0u8; 256];
        if let Ok(n) = f.read(&mut head) {
            if has_python_shebang(&head[..n]) {
                hits.push(format!("{rel} (shebang python)"));
            }
        }
    }
    if hits.is_empty() {
        println!("check_no_python: OK (không file Python nào trong repo)");
        Ok(())
    } else {
        let mut msg = String::from(
            "E59: repo không được chứa file Python (AGENTS.md language policy). \
             Port sang cargo xtask (xem ledger E59 - docs/specs/project-common-errors.md):",
        );
        for p in &hits {
            msg.push_str(&format!("\n  {p}"));
        }
        Err(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_links_bat_ca_dang_target() {
        let t = "xem [a](docs/x.md) và [b](docs/y.md#phan-muc) va [c](#gan-day) va [d]() va [e](https://vt.vn)";
        let links = doc_link_targets(t);
        assert_eq!(
            links,
            vec![
                "docs/x.md",
                "docs/y.md#phan-muc",
                "#gan-day",
                "https://vt.vn"
            ]
        );
        // `]()` (target rỗng) không tính — regex gốc yêu cầu ≥1 ký tự.
        assert!(!links.iter().any(|l| l.is_empty()));
    }

    #[test]
    fn doc_links_yeu_cau_dong_ngoac_va_bo_khoang_trang() {
        assert!(doc_link_targets("[note](see section 2)").is_empty());
        assert!(doc_link_targets("arr[0](x, y)").is_empty());
        assert!(doc_link_targets("[a](unclosed").is_empty());
        assert!(doc_link_targets("[a](foo\u{00A0}bar)").is_empty());
        assert_eq!(
            doc_link_targets("[a](foo [b](bar.md))"),
            vec!["bar.md".to_string()]
        );
    }

    #[test]
    fn private_key_bat_du_phien_ban() {
        for header in ["RSA", "OPENSSH", "EC", ""] {
            let text = format!("-----BEGIN {header} PRIVATE KEY-----\nabc");
            assert!(
                !find_private_key_headers(&text).is_empty(),
                "phải bắt '-----BEGIN {header} PRIVATE KEY-----'"
            );
        }
        assert!(
            !find_private_key_headers("-----BEGIN OPENSSH PRIVATE KEY BLOCK-----\nx").is_empty()
        );
        assert!(find_private_key_headers("-----BEGIN CERTIFICATE-----").is_empty());
        assert!(find_private_key_headers("khong co gi").is_empty());
        let two = "-----BEGIN RSA PRIVATE KEY-----\n-----BEGIN EC PRIVATE KEY-----";
        assert_eq!(find_private_key_headers(two).len(), 2);
    }

    #[test]
    fn github_token_do_dai_30() {
        let tok = format!("ghp_{}1", "a".repeat(30));
        assert_eq!(find_github_tokens(&format!("token = {tok}")).len(), 1);
        assert!(find_github_tokens("ghp_too_short").is_empty());
        let pat = format!("github_pat_{}x", "a".repeat(31));
        assert_eq!(find_github_tokens(&pat).len(), 1);
        assert!(find_github_tokens("github_pat_ngan").is_empty());
        assert_eq!(find_github_tokens(&format!("{tok} {pat}")).len(), 2);
    }

    #[test]
    fn aws_key_ranh_gioi_tu() {
        assert_eq!(find_aws_keys("AKIAABCDEFGHIJKL0123").len(), 1);
        // quá ngắn
        assert!(find_aws_keys("AKIAABCDEFGHIJKL012").is_empty());
        // dính ký tự từ → vi phạm \b
        assert!(find_aws_keys("xAKIAABCDEFGHIJKL0123").is_empty());
        assert!(find_aws_keys("AKIAABCDEFGHIJKL0123z").is_empty());
        // \b hai bên: dấu chấm là non-word
        assert_eq!(find_aws_keys("AKIAABCDEFGHIJKL0123.").len(), 1);
        assert_eq!(find_aws_keys(".AKIAABCDEFGHIJKL0123").len(), 1);
        assert_eq!(
            find_aws_keys("AKIAABCDEFGHIJKL0123 AKIA0123456789ABCDEF").len(),
            2
        );
    }

    #[test]
    fn hooked_word_boundaries() {
        assert!(contains_word(
            "SetWindowsHookEx(13, WH_KEYBOARD, f, h)",
            "WH_KEYBOARD"
        ));
        // WH_KEYBOARD_LL: sau D là '_' → không phải word-boundary "WH_KEYBOARD"
        assert!(!contains_word("WH_KEYBOARD_LL", "WH_KEYBOARD"));
        assert!(!contains_word("MYWH_KEYBOARD", "WH_KEYBOARD"));
        assert!(contains_word("(*ptr).WH_MOUSE)", "WH_MOUSE"));
    }

    #[test]
    fn set_hook_call() {
        assert!(contains_set_hook_call("h = SetWindowsHookEx(13, 1, 2, 3);"));
        assert!(contains_set_hook_call(
            "h = SetWindowsHookExW ( 13, 1, 2, 3);"
        ));
        assert!(!contains_set_hook_call("let s = \"SetWindowsHookExX\";"));
        // \w* ăn ký tự từ sau tên → phải có '(' sau đó
        assert!(!contains_set_hook_call("SetWindowsHookExA"));
        // không gọi hàm — chỉ định nghĩa chuỗi
        assert!(!contains_set_hook_call("// SetWindowsHookExKalice"));
    }

    #[test]
    fn injection_ll_boundary_va_non_utf8() {
        let mut v = Vec::new();
        scan_injection_line(
            "a.rs",
            1,
            "SetWindowsHookExW(id, cb, h, 0) // FAKE_WH_KEYBOARD_LL_2",
            &mut v,
        );
        assert_eq!(v.len(), 1, "FAKE_WH_KEYBOARD_LL_2 không được miễn trừ");
        v.clear();
        scan_injection_line(
            "a.rs",
            2,
            "SetWindowsHookExW(WH_KEYBOARD_LL, cb, h, 0)",
            &mut v,
        );
        assert!(v.is_empty());
        // Byte không hợp lệ UTF-8 kèm CreateRemoteThread vẫn phải bị bắt qua lossy decode.
        let raw = b"// \xff comment\nCreateRemoteThread(h);\n";
        let lossy = String::from_utf8_lossy(raw);
        for (i, ln) in lossy.lines().enumerate() {
            scan_injection_line("b.c", i + 1, ln, &mut v);
        }
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn iss_tabs_bat_ca_non_utf8() {
        let raw = b"; \xff ansi comment\nName: \"{app}\tfoo\"\n";
        assert_eq!(find_tab_lines(raw), vec![2]);
    }

    #[test]
    fn no_python_bat_hoa_thuong_va_shebang() {
        assert!(is_python_path("a.py"));
        assert!(is_python_path("A.PY"));
        assert!(is_python_path("tools/x.pyw"));
        assert!(is_python_path("stubs/a.pyi"));
        assert!(is_python_path("nb/demo.ipynb"));
        assert!(!is_python_path("a.rs"));
        assert!(has_python_shebang(b"#!/usr/bin/env python3\nprint(1)\n"));
        assert!(has_python_shebang(b"#!/usr/bin/python\n"));
        assert!(!has_python_shebang(b"#!/usr/bin/env bash\necho python\n"));
        assert!(!has_python_shebang(b"#!/usr/bin/env python3\x00bin"));
    }
}
