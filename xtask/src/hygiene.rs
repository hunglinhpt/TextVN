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
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Gốc repo (xtask nằm ở `<root>/xtask`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR luôn nằm trong repo")
        .to_path_buf()
}

/// `git ls-files --cached --others --exclude-standard [pattern]` — quét CẢ
/// file mới chưa track (không bị .gitignore), port nguyên tắc R2-112: gate
/// local phải thấy file secret/key vừa tạo chưa `git add`.
fn git_list(pattern: Option<&str>) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    cmd.arg("ls-files")
        .arg("--cached")
        .arg("--others")
        .arg("--exclude-standard");
    if let Some(p) = pattern {
        cmd.arg(p);
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
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

// ---------------------------------------------------------------------------
// hygiene-docs — port check_doc_links.py
// ---------------------------------------------------------------------------

/// Link markdown nội bộ dạng `](target)` — thay regex `\]\(([^)\s]+)\)`:
/// sau `](` lấy tối đa các ký tự không phải `)` và không phải khoảng trắng.
/// Regex gốc yêu cầu ≥ 1 ký tự nên `]()` không tính.
fn doc_link_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b')' && !bytes[end].is_ascii_whitespace() {
                end += 1;
            }
            if end > start {
                out.push(text[start..end].to_string());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Gom mọi file `docs/**/*.md` (đệ quy, có cả `docs/a.md` như `**` của
/// pathlib) — sắp xếp theo đường dẫn như `sorted()` của Python.
fn collect_md(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "md") {
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
        let text = fs::read_to_string(&md).unwrap_or_else(|_| String::new());
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
fn find_private_key_header(text: &str) -> Option<String> {
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
        // backtracking: dừng dần ở mỗi vị trí class xem có "PRIVATE KEY..." không
        let run = &text[run_start..j];
        let mut search_pos = run_start;
        while let Some(off) = text[search_pos..j]
            .find("PRIVATE KEY")
            .map(|p| search_pos + p)
        {
            let mut end = off + "PRIVATE KEY".len();
            if text[end..].starts_with(" BLOCK") {
                end += " BLOCK".len();
            }
            if text[end..].starts_with("-----") {
                return Some(text[pos..end + 5].to_string());
            }
            search_pos = off + 1;
            if run.is_empty() || search_pos >= j {
                break;
            }
        }
        i = pos + BEGIN.len();
    }
    None
}

/// Token GitHub: `(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}` hoặc
/// `github_pat_[A-Za-z0-9_]{30,}`.
fn find_github_token(text: &str) -> Option<String> {
    for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
        let mut i = 0;
        while let Some(pos) = text[i..].find(prefix).map(|p| i + p) {
            let start = pos + prefix.len();
            let mut end = start;
            let bytes = text.as_bytes();
            while end < bytes.len() && bytes[end].is_ascii_alphanumeric() {
                end += 1;
            }
            if end - start >= 30 {
                return Some(text[pos..end].to_string());
            }
            i = pos + 1;
        }
    }
    let mut i = 0;
    while let Some(pos) = text[i..].find("github_pat_").map(|p| i + p) {
        let start = pos + "github_pat_".len();
        let mut end = start;
        let bytes = text.as_bytes();
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        if end - start >= 30 {
            return Some(text[pos..end].to_string());
        }
        i = pos + 1;
    }
    None
}

/// AWS Access Key: `\bAKIA[0-9A-Z]{16}\b` — ranh giới từ hai bên.
fn find_aws_key(text: &str) -> Option<String> {
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
            return Some(text[pos..end].to_string());
        }
        i = pos + 1;
    }
    None
}

/// Gate: secret / token / private key không được commit (R2-112: quét cả file
/// chưa track; bỏ qua `graphify-out/` là cache graph đã kiểm soát).
pub fn no_secrets() -> Result<(), String> {
    let root = repo_root();
    let tracked: BTreeSet<String> = git_list(None)?
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
        let full = root.join(p);
        if !full.is_file() {
            continue;
        }
        // Đọc 1 lần: byte NUL trong 1KB đầu = file nhị phân → bỏ quét nội dung.
        let Ok(bytes) = fs::read(&full) else {
            continue;
        };
        let head = &bytes[..bytes.len().min(1024)];
        if head.contains(&0) {
            continue;
        }
        scanned += 1;
        // Port đọc nội dung lossy, giống `errors="replace"` của bản Python.
        let content = String::from_utf8_lossy(&bytes).into_owned();
        // Chính file chứa các detector (literal pattern) và workflow khai báo
        // chúng phải miễn nhiễm — như bản Python cũ tự loại trừ chính nó.
        if rel == "xtask/src/hygiene.rs" || rel == ".github/workflows/repo-hygiene.yml" {
            continue;
        }
        for (rule, hit) in [
            ("Private Key Header", find_private_key_header(&content)),
            ("GitHub Token", find_github_token(&content)),
            ("AWS Access Key", find_aws_key(&content)),
        ] {
            if let Some(text) = hit {
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

/// Gate: API inject/giống malware bị cấm (antivirus-false-positive.md §8).
/// `spikes/` = probe không phát hành (hook-spike, WIN-005) — guard áp cho
/// code phát hành, copy vào `adapters/` vẫn bị chặn tại đó.
pub fn no_injection_apis() -> Result<(), String> {
    let root = repo_root();
    let out = Command::new("git")
        .arg("ls-files")
        .arg("-z")
        .arg("--cached")
        .arg("--others")
        .arg("--exclude-standard")
        .current_dir(&root)
        .output()
        .map_err(|e| format!("chạy `git ls-files`: {e}"))?;
    let listed = String::from_utf8_lossy(&out.stdout);

    let mut violations = Vec::new();
    let mut scanned = 0usize;
    for rel in listed.split('\0') {
        let rel = rel.trim();
        if rel.is_empty() || rel == "xtask/src/hygiene.rs" {
            continue;
        }
        if rel.starts_with("spikes/") {
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
        let text = fs::read_to_string(&path).unwrap_or_default();
        for (lineno, line) in text.lines().enumerate() {
            let lineno = lineno + 1;
            for name in BANNED_APIS {
                if line.contains(name) {
                    violations.push(format!("{rel}:{lineno}: {name}"));
                }
            }
            let non_ll_hook = contains_word(line, "WH_KEYBOARD") || contains_word(line, "WH_MOUSE");
            let ll_ok = line.contains("WH_KEYBOARD_LL") || line.contains("WH_MOUSE_LL");
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

/// Gate: `.iss` không chứa TAB — escape `\t` từng bị ghi thành TAB thật phá
/// pattern/đường dẫn `{app}\textvn-tsf.dll` trong [UninstallDelete]; ISCC
/// compile vẫn XANH nên gate này là lớp duy nhất bắt được.
pub fn iss_tabs() -> Result<(), String> {
    let files = git_list(Some("*.iss"))?;
    let mut bad = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).unwrap_or_default();
        for (lineno, line) in text.lines().enumerate() {
            if line.contains('\t') {
                bad.push(format!("{path}:{}", lineno + 1));
            }
        }
    }
    if bad.is_empty() {
        println!(
            "check_iss_tabs: OK ({} file .iss khong co TAB)",
            files.len()
        );
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
    let files = git_list(Some("*.ps1"))?;
    let mut bad = Vec::new();
    for path in &files {
        let Ok(data) = fs::read(path) else {
            continue;
        };
        if data.iter().any(|b| *b > 0x7F) {
            bad.push(path.clone());
        }
    }
    if bad.is_empty() {
        println!("check_ps1_ascii: OK ({} file .ps1 ASCII-only)", files.len());
        Ok(())
    } else {
        let mut msg = String::from(".ps1 phải ASCII-only (G7/A5):");
        for p in &bad {
            msg.push_str(&format!("\n  {p}"));
        }
        Err(msg)
    }
}

/// E59: repo KHÔNG được chứa file Python (policy ngôn ngữ AGENTS.md — chỉ Rust +
/// C/C++/Swift native). Quét cả file track lẫn file mới chưa track (không bị
/// gitignore) — chặn tái diễn việc agent thêm .py như 18 file lịch sử (E59).
/// Fixture/exception: KHÔNG có — mọi nhu cầu đo lường/sinh dữ liệu đã port
/// sang `cargo xtask` (assist-*, gen-*, check-*).
pub fn no_python() -> Result<(), String> {
    let files = git_list(Some("*.py"))?;
    if files.is_empty() {
        println!("check_no_python: OK (không file .py nào trong repo)");
        Ok(())
    } else {
        let mut msg = String::from(
            "E59: repo không được chứa file Python (AGENTS.md language policy). \
             Port sang cargo xtask (xem ledger E59 - docs/specs/project-common-errors.md):",
        );
        for p in &files {
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
    fn private_key_bat_du_phien_ban() {
        for header in ["RSA", "OPENSSH", "EC", ""] {
            let text = format!("-----BEGIN {header} PRIVATE KEY-----\nabc");
            assert!(
                find_private_key_header(&text).is_some(),
                "phải bắt '-----BEGIN {header} PRIVATE KEY-----'"
            );
        }
        assert!(find_private_key_header("-----BEGIN OPENSSH PRIVATE KEY BLOCK-----\nx").is_some());
        assert!(find_private_key_header("-----BEGIN CERTIFICATE-----").is_none());
        assert!(find_private_key_header("khong co gi").is_none());
    }

    #[test]
    fn github_token_do_dai_30() {
        let tok = format!("ghp_{}1", "a".repeat(30));
        assert!(find_github_token(&format!("token = {tok}")).is_some());
        assert!(find_github_token("ghp_too_short").is_none());
        let pat = format!("github_pat_{}x", "a".repeat(31));
        assert!(find_github_token(&pat).is_some());
        assert!(find_github_token("github_pat_ngan").is_none());
    }

    #[test]
    fn aws_key_ranh_gioi_tu() {
        assert!(find_aws_key("AKIAABCDEFGHIJKL0123").is_some());
        // quá ngắn
        assert!(find_aws_key("AKIAABCDEFGHIJKL012").is_none());
        // dính ký tự từ → vi phạm \b
        assert!(find_aws_key("xAKIAABCDEFGHIJKL0123").is_none());
        assert!(find_aws_key("AKIAABCDEFGHIJKL0123z").is_none());
        // \b hai bên: dấu chấm là non-word
        assert!(find_aws_key("AKIAABCDEFGHIJKL0123.").is_some());
        assert!(find_aws_key(".AKIAABCDEFGHIJKL0123").is_some());
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
}
