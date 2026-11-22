// SPDX-License-Identifier: GPL-3.0-or-later
//! Công cụ kiểm Windows release — bản Rust thay `tools/win/check-pe-imports.py`
//! và `tools/win/verify-msix.py` (E59: policy cấm script Python trong repo).
//!
//! - `cargo xtask check-pe-imports <file.exe|dll>...` — chặn binary phụ thuộc
//!   VC++ runtime (R2-18): TIP DLL được nạp vào MỌI tiến trình, máy thiếu
//!   VC++ redist không nạp được → link CRT tĩnh (`+crt-static`).
//! - `cargo xtask verify-msix <file.msix> [--expect-version A.B.C]
//!   [--require-store-identity] [--display-name N] [--expect-name NAME]` —
//!   luật Partner Center ngoài XSD của makeappx (R2-09): Identity/Version
//!   phần đầu ≠ 0, Publisher/DisplayName, mô tả không mojibake, ảnh đúng kích
//!   thước, payload đủ.
//!
//! Điểm gọi: ci-windows.yml, release.yml, tools/release/verify-store-package.sh
//! (chạy trên ubuntu LẪN Windows). MSIX = ZIP: giải nén bằng `bsdtar`
//! (Windows có sẵn C:\Windows\System32\tar.exe đọc được zip; macOS/Linux bsdtar)
//! hoặc `unzip` (Linux runner có sẵn) — không dùng runtime Python.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// check-pe-imports — port tools/win/check-pe-imports.py
// ---------------------------------------------------------------------------

fn u16_at(d: &[u8], off: usize) -> Result<u16, String> {
    d.get(off..off + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| format!("PE: thiếu byte tại 0x{off:X}"))
}

fn u32_at(d: &[u8], off: usize) -> Result<u32, String> {
    d.get(off..off + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| format!("PE: thiếu byte tại 0x{off:X}"))
}

/// Đọc bảng import của một PE (Portable Executable) — parser thuần byte như
/// bản Python cũ: KHÔNG cần dumpbin/Visual Studio.
pub fn pe_imports_bytes(d: &[u8]) -> Result<Vec<String>, String> {
    if d.len() < 0x40 || &d[..2] != b"MZ" {
        return Err("không phải PE (thiếu MZ)".into());
    }
    let pe = u32_at(d, 0x3C)? as usize;
    if d.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err("không phải PE (thiếu chữ ký PE)".into());
    }
    let nsec = u16_at(d, pe + 6)? as usize;
    let opt_size = u16_at(d, pe + 20)? as usize;
    let opt = pe + 24;
    let magic = u16_at(d, opt)?;
    let dd = match magic {
        0x10B => opt + 96,  // PE32
        0x20B => opt + 112, // PE32+
        other => return Err(format!("PE: magic lạ 0x{other:X}")),
    };
    let imp_rva = u32_at(d, dd + 8)? as usize;
    let mut sections: Vec<(usize, usize, usize)> = Vec::with_capacity(nsec);
    let sec = opt + opt_size;
    for i in 0..nsec {
        let off = sec + 40 * i;
        let vsize = u32_at(d, off + 8)? as usize;
        let va = u32_at(d, off + 12)? as usize;
        let rawsize = u32_at(d, off + 16)? as usize;
        let rawptr = u32_at(d, off + 20)? as usize;
        sections.push((va, vsize.max(rawsize), rawptr));
    }
    if imp_rva == 0 {
        return Ok(Vec::new());
    }
    let rva_to_off = |rva: usize| -> Result<usize, String> {
        for (va, size, raw) in &sections {
            if *va <= rva && rva < va + size {
                return Ok(raw + (rva - va));
            }
        }
        Err(format!("RVA 0x{rva:X} ngoài mọi section"))
    };
    let mut names = Vec::new();
    let mut off = rva_to_off(imp_rva)?;
    loop {
        let orig = u32_at(d, off)?;
        let name_rva = u32_at(d, off + 12)?;
        let first = u32_at(d, off + 16)?;
        if orig == 0 && name_rva == 0 && first == 0 {
            break;
        }
        let noff = rva_to_off(name_rva as usize)?;
        let end = d[noff..]
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| "PE: tên DLL không kết thúc bằng \\0".to_string())?;
        names.push(String::from_utf8_lossy(&d[noff..noff + end]).into_owned());
        off += 20;
    }
    Ok(names)
}

/// DLL nào thuộc VC++ runtime? Port regex
/// `^(vcruntime\d+.*|msvcp\d+.*|ucrtbase.*|api-ms-win-crt-.*)\.dll$` IGNORECASE.
fn is_crt_runtime_dll(name: &str) -> bool {
    let lower = name.to_lowercase();
    let Some(stem) = lower.strip_suffix(".dll") else {
        return false;
    };
    let starts_with_digit = |prefix: &str| -> bool {
        stem.strip_prefix(prefix)
            .is_some_and(|rest| rest.chars().next().is_some_and(|c| c.is_ascii_digit()))
    };
    starts_with_digit("vcruntime")
        || starts_with_digit("msvcp")
        || stem.starts_with("ucrtbase")
        || stem.starts_with("api-ms-win-crt-")
}

/// CLI: `xtask check-pe-imports <files>...` — Err khi PE nào phụ thuộc
/// VC++ runtime hoặc không đọc được.
pub fn check_pe_imports_cli(files: &[String]) -> Result<(), String> {
    if files.is_empty() {
        return Err("cách dùng: cargo xtask check-pe-imports <file.exe|file.dll> [...]".into());
    }
    let mut bad = 0usize;
    for path in files {
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                println!("FAIL {path}: {e}");
                bad += 1;
                continue;
            }
        };
        match pe_imports_bytes(&data) {
            Err(e) => {
                println!("FAIL {path}: {e}");
                bad += 1;
            }
            Ok(dlls) => {
                let hits: Vec<&str> = dlls
                    .iter()
                    .map(String::as_str)
                    .filter(|d| is_crt_runtime_dll(d))
                    .collect();
                if hits.is_empty() {
                    println!(
                        "OK   {path}: {} DLL import, không có VC++ runtime",
                        dlls.len()
                    );
                } else {
                    println!(
                        "FAIL {path}: phụ thuộc VC++ runtime {} — cần +crt-static",
                        hits.join(", ")
                    );
                    bad += 1;
                }
            }
        }
    }
    if bad == 0 {
        Ok(())
    } else {
        Err(format!(
            "{bad} file phụ thuộc VC++ runtime hoặc không đọc được PE"
        ))
    }
}

// ---------------------------------------------------------------------------
// verify-msix — port tools/win/verify-msix.py
// ---------------------------------------------------------------------------

const PLACEHOLDER_NAME: &str = "LinhBH.CoM.TextVN";
const EXPECTED_PUBLISHER: &str = "CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545";
const EXPECTED_DISPLAY_NAME: &str = "TextVN";
const EXPECTED_PUBLISHER_DISPLAY: &str = "LinhBH.CoM";
const STORE_LOGO_SIZE: (u32, u32) = (50, 50);
const SPLASH_SIZE: (u32, u32) = (620, 300);

/// Thuộc tính VisualElements → kích thước bắt buộc (scale-100).
fn asset_size(attr: &str) -> Option<(u32, u32)> {
    match attr {
        "Square150x150Logo" => Some((150, 150)),
        "Square44x44Logo" => Some((44, 44)),
        "Wide310x150Logo" => Some((310, 150)),
        "Square71x71Logo" => Some((71, 71)),
        _ => None,
    }
}

/// File runtime tray/cli/TIP đọc từ đĩa — thiếu là lỗi payload.
const REQUIRED_PAYLOAD: &[&str] = &[
    "TextVN.exe",
    "textvn-cli.exe",
    "textvn-tsf.dll",
    "textvn-tsf-x86.dll",
    "resources/textvn.ico",
    "resources/textvn_v.ico",
    "resources/textvn_e.ico",
    "data/appdb.default.json",
];

/// 0.2.27 → 1.2.27.0 — phải khớp tools/win/build-msix.ps1 (R2-01).
fn map_version(product: &str) -> Result<String, String> {
    let parts: Vec<&str> = product.split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(format!("version sản phẩm '{product}' không phải A.B.C"));
    }
    let a: u64 = parts[0].parse().expect("đã kiểm tra digit");
    let b: u64 = parts[1].parse().expect("đã kiểm tra digit");
    let c: u64 = parts[2].parse().expect("đã kiểm tra digit");
    Ok(format!("{}.{}.{}.0", a + 1, b, c))
}

/// Header PNG: width/height IHDR big-endian tại byte 16..24.
fn png_size(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    Some((
        u32::from_be_bytes([data[16], data[17], data[18], data[19]]),
        u32::from_be_bytes([data[20], data[21], data[22], data[23]]),
    ))
}

// ---- XML tối tiểu nhưng STRICT — đủ cho AppxManifest.xml do build sinh ----
//
// Chỉ đọc những gì verify cần: Identity (con trực tiếp của gốc), Properties
// (con trực tiếp), Capability/Application/VisualElements/SplashScreen (bất kỳ
// độ sâu, so khớp bằng LOCAL name — namespace của các thẻ cần đọc luôn khác
// localname trong manifest này). Parser bắt được XML sai well-formed mà
// plutil/CFPropertyList nuốt được (mục đích của check).

struct Xml {
    /// local name sau dấu ':' (không cần bàn tới namespace).
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Xml>,
    /// Text ghép trực tiếp (như `findtext` của ElementTree).
    text: String,
}

impl Xml {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.split(':').next_back() == Some(name))
            .map(|(_, v)| v.as_str())
    }
    fn child(&self, name: &str) -> Option<&Xml> {
        self.children.iter().find(|c| c.name == name)
    }
    fn descendants<'a>(&'a self, out: &mut Vec<&'a Xml>) {
        for c in &self.children {
            out.push(c);
            c.descendants(out);
        }
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn local_name(tag: &str) -> String {
    tag.split(':').next_back().unwrap_or(tag).to_string()
}

fn valid_xml_name(tag: &str) -> bool {
    !tag.is_empty()
        && !tag.starts_with('-')
        && !tag.starts_with('.')
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':'))
}

// NOTE: build-msix.ps1 sinh manifest chỉ chứa 5 thực thể chuẩn dưới đây.
fn decode_entities(s: &str) -> String {
    // NOTE: build-msix.ps1 sinh manifest chi dung 5 thuc the chuan. Thu tu
    // `&amp;` dung dau de khong bi thuc the khac an truoc.
    let known = [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        match known
            .iter()
            .find_map(|(ent, val)| tail.strip_prefix(ent).map(|stripped| (*val, stripped)))
        {
            Some((val, stripped)) => {
                out.push_str(val);
                rest = stripped;
            }
            None => {
                // thuc the la hoac ky tu & tho — giu nguyen.
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn parse_xml(src: &str) -> Result<Xml, String> {
    let src = src.trim_start_matches(char::from_u32(0xFEFF).unwrap());
    let bytes = src.as_bytes();
    let mut i = 0usize;

    // Bỏ qua khoảng trắng / XML decl `<?...?>` / comment `<!--...-->`.
    // Nghiêm ngặt hơn plutil: comment chứa "--" là XML sai well-formed → Err
    // (plutil/CFPropertyList nuốt được; parser strict phải bắt — R3 F3-11).
    fn skip_misc(bytes: &[u8], i: &mut usize) -> Result<(), String> {
        loop {
            while *i < bytes.len() && (bytes[*i] as char).is_ascii_whitespace() {
                *i += 1;
            }
            if bytes[*i..].starts_with(b"<?") {
                if let Some(k) = find(&bytes[*i..], b"?>") {
                    // `?>` bắt đầu tại i+k và dài 2 → vượt qua hết decl.
                    *i += k + 2;
                    continue;
                }
                return Err("XML: `<?` không có `?>`".into());
            }
            if bytes[*i..].starts_with(b"<!--") {
                let body = &bytes[*i + 4..];
                if let Some(k) = find(body, b"-->") {
                    if body[..k].windows(2).any(|w| w == b"--") {
                        return Err("XML: comment chứa \"--\" (không well-formed)".into());
                    }
                    // "<!--" (4) + thân k + "-->" (3)
                    *i += k + 7;
                    continue;
                }
                return Err("XML: comment không có \"-->\"".into());
            }
            return Ok(());
        }
    }

    // root ảo đỡ stack; con đầu tiên của nó là phần tử gốc thật.
    let mut stack: Vec<Xml> = vec![Xml {
        name: String::new(),
        attrs: Vec::new(),
        children: Vec::new(),
        text: String::new(),
    }];
    loop {
        skip_misc(bytes, &mut i)?;
        if i >= bytes.len() {
            break;
        }
        if bytes[i] != b'<' {
            return Err(format!(
                "XML: kỳ vọng '<' tại offset {i} — text nằm ngoài thẻ"
            ));
        }
        let close = find(&bytes[i..], b">")
            .map(|k| i + k)
            .ok_or("XML: thẻ không có '>'")?;
        let inner_raw = String::from_utf8_lossy(&bytes[i + 1..close]).into_owned();
        let self_closing = inner_raw.ends_with('/');
        let inner = inner_raw
            .strip_suffix('/')
            .unwrap_or(inner_raw.as_str())
            .trim();
        let (name_raw, attr_src) = inner.split_once(char::is_whitespace).unwrap_or((inner, ""));
        if let Some(closing) = name_raw.strip_prefix('/') {
            let name = local_name(closing);
            let Some(opened) = stack.pop() else {
                return Err(format!("XML: đóng </{name}> thừa"));
            };
            if opened.name != name {
                return Err(format!(
                    "XML: đóng </{name}> nhưng đang mở <{}>",
                    opened.name
                ));
            }
            // text nội dung đã hứng ở vòng push bên dưới
            let Some(parent) = stack.last_mut() else {
                return Err(format!("XML: đóng </{name}> thừa"));
            };
            parent.children.push(opened);
            i = close + 1;
            continue;
        }
        if name_raw.starts_with('?') || name_raw.starts_with('!') {
            // `<?...?>` đã bị skip_misc ăn; `<!...>` (DOCTYPE/CDATA) → để
            // valid_xml_name bắt lỗi vì manifest không dùng.
            i = close + 1;
            continue;
        }
        if !valid_xml_name(name_raw) {
            return Err(format!("XML: tên thẻ lạ '{name_raw}'"));
        }
        // thuộc tính key="value"
        let mut attrs = Vec::new();
        let mut p = 0usize;
        let ab = attr_src.as_bytes();
        while p < ab.len() {
            while p < ab.len() && (ab[p] as char).is_ascii_whitespace() {
                p += 1;
            }
            if p >= ab.len() {
                break;
            }
            let eq = attr_src[p..]
                .find('=')
                .map(|k| p + k)
                .ok_or(format!("XML: thuộc tính thiếu '=' gần '{attr_src}'"))?;
            let key = attr_src[p..eq].trim().to_string();
            let quote = *ab
                .get(eq + 1)
                .ok_or(format!("XML: thuộc tính {key} thiếu nháy giá trị"))?;
            if quote != b'"' && quote != b'\'' {
                return Err(format!(
                    "XML: giá trị thuộc tính {key} không nằm trong nháy"
                ));
            }
            let vpos = eq + 2;
            let vend = attr_src[vpos..]
                .find(quote as char)
                .map(|k| vpos + k)
                .ok_or(format!("XML: thuộc tính {key} không đóng nháy"))?;
            attrs.push((local_name(&key), decode_entities(&attr_src[vpos..vend])));
            p = vend + 1;
        }
        let mut node = Xml {
            name: local_name(name_raw),
            attrs,
            children: Vec::new(),
            text: String::new(),
        };
        i = close + 1;
        if self_closing {
            stack.last_mut().expect("root ảo").children.push(node);
        } else {
            // hứng text tới '<' kế tiếp làm nội dung — như itertext chỉ 1 mức.
            let start_text = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let raw_text = String::from_utf8_lossy(&bytes[start_text..i])
                .trim()
                .to_string();
            node.text = decode_entities(&raw_text);
            stack.push(node);
        }
    }
    if stack.len() != 1 {
        return Err(format!("XML: {} thẻ chưa đóng", stack.len() - 1));
    }
    stack
        .pop()
        .expect("checked len")
        .children
        .into_iter()
        .next()
        .ok_or_else(|| "XML: không có phần tử gốc".to_string())
}

/// Manifest có chuỗi mojibake (UTF-8 bị mã hoá 2 lần — bug 0.2.27)?
fn manifest_mojibake(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        let c = chars[i];
        if (c == '\u{00C3}' || c == '\u{00C2}')
            && chars
                .get(i + 1)
                .is_some_and(|n| ('\u{0080}'..='\u{00BF}').contains(n))
        {
            return true;
        }
    }
    text.contains("á»") || text.contains("áº") || text.contains("Æ°")
}

/// Mô tả còn giữ dấu tiếng Việt? — port regex `[ăâđêôơưẠ-ỹ]` IGNORECASE.
fn contains_vietnamese(text: &str) -> bool {
    text.chars().any(|c| {
        let lc = c.to_lowercase().next().unwrap_or(c);
        matches!(lc, 'ă' | 'â' | 'đ' | 'ê' | 'ô' | 'ơ' | 'ư')
            || ('\u{1EA0}'..='\u{1EF9}').contains(&lc)
    })
}

/// Nội dung MSIX đã giải nén — index theo path lowercase, đổi `\` → `/`.
struct MsixDir {
    root: PathBuf,
    index: BTreeMap<String, String>,
}

impl MsixDir {
    fn member(&self, rel: &str) -> Option<&String> {
        self.index.get(&rel.replace('\\', "/").to_lowercase())
    }
    fn read(&self, rel: &str) -> std::io::Result<Vec<u8>> {
        fs::read(self.root.join(rel))
    }
}

fn walk_index(base: &Path, dir: &Path, index: &mut BTreeMap<String, String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_index(base, &p, index);
        } else if let Ok(rel) = p.strip_prefix(base) {
            let rel = rel.to_string_lossy().replace('\\', "/");
            index.insert(rel.to_lowercase(), rel);
        }
    }
}

/// MSIX là ZIP. Thứ tự thử: `tar` (Windows System32 = bsdtar libarchive đọc
/// zip; macOS mặc định bsdtar) → `bsdtar` (Linux cài libarchive-tools) →
/// `unzip` (ubuntu runner có sẵn). Không dùng runtime Python.
fn extract_msix(msix: &Path) -> Result<MsixDir, String> {
    let dest = std::env::temp_dir().join("textvn-verify-msix");
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("dọn {}: {e}", dest.display()))?;
    }
    fs::create_dir_all(&dest).map_err(|e| format!("tạo {}: {e}", dest.display()))?;
    let msix_abs = msix
        .canonicalize()
        .map_err(|e| format!("không thấy {}: {e}", msix.display()))?;

    let mut errs: Vec<String> = Vec::new();
    let mut extracted = false;
    // (chương trình, đôi số build args): để đối số theo đúng trình tự.
    type Extractor = (&'static str, &'static [&'static str]);
    let attempts: &[Extractor] = if std::env::consts::OS == "windows" {
        &[
            ("tar", &["-xf"]),
            ("bsdtar", &["-xf"]),
            ("unzip", &["-q", "-o", "-d"]),
        ]
    } else {
        &[
            ("bsdtar", &["-xf"]),
            ("tar", &["-xf"]),
            ("unzip", &["-q", "-o", "-d"]),
        ]
    };
    for &(prog, lead) in attempts {
        let mut cmd = Command::new(prog);
        cmd.args(lead).arg(&msix_abs);
        if prog == "unzip" {
            cmd.arg(&dest);
        } else {
            cmd.arg("-C").arg(&dest);
        }
        match cmd.output() {
            Ok(o) if o.status.success() => {
                extracted = true;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                errs.push(format!("{prog}: không có trên máy"));
            }
            Ok(o) => errs.push(format!(
                "{prog} exit {}: {}",
                o.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&o.stderr)
            )),
            Err(e) => errs.push(format!("{prog}: {e}")),
        }
    }
    if !extracted {
        return Err(format!(
            "giải nén MSIX không thành công — {} (cần bsdtar/tar hoặc unzip)",
            errs.join("; ")
        ));
    }
    let mut index = BTreeMap::new();
    walk_index(&dest, &dest, &mut index);
    if index.is_empty() {
        return Err("giải nén ra 0 file — không phải MSIX?".into());
    }
    Ok(MsixDir { root: dest, index })
}

/// Toàn bộ luật Store + payload — port `check()` của verify-msix.py.
fn verify_full(
    dir: &MsixDir,
    want_ver: Option<&str>,
    require_identity: bool,
    expect_display: &str,
    expect_name: Option<&str>,
) -> Vec<String> {
    let mut fails = Vec::new();
    let Some(manifest) = dir.member("AppxManifest.xml") else {
        return vec!["thiếu AppxManifest.xml".into()];
    };
    let raw = match dir.read(manifest) {
        Ok(r) => r,
        Err(e) => return vec![format!("không đọc được AppxManifest.xml: {e}")],
    };
    let text = match String::from_utf8(raw) {
        Ok(t) => t,
        Err(_) => return vec!["AppxManifest.xml không phải UTF-8".into()],
    };
    let root = match parse_xml(&text) {
        Ok(r) => r,
        Err(e) => return vec![format!("AppxManifest.xml XML lỗi: {e}")],
    };
    let Some(ident) = root.child("Identity") else {
        return vec!["thiếu <Identity>".into()];
    };
    let name = ident.attr("Name").unwrap_or("");
    let publisher = ident.attr("Publisher").unwrap_or("");
    let version = ident.attr("Version").unwrap_or("");

    if !(3..=50).contains(&name.len())
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
    {
        fails.push(format!(
            "Identity/Name '{name}' không hợp lệ (3-50 ký tự A-Za-z0-9.-)"
        ));
    }
    if require_identity && name == PLACEHOLDER_NAME {
        fails.push(format!(
            "Identity/Name vẫn là placeholder '{PLACEHOLDER_NAME}' — đặt MSIX_IDENTITY_NAME"
        ));
    }
    if let Some(want) = expect_name {
        if name != want {
            fails.push(format!(
                "Identity/Name '{name}' ≠ Product identity của sản phẩm Store '{want}'"
            ));
        }
    }
    if publisher != EXPECTED_PUBLISHER {
        fails.push(format!(
            "Identity/Publisher '{publisher}' ≠ Partner Center '{EXPECTED_PUBLISHER}'"
        ));
    }
    let parts: Vec<&str> = version.split('.').collect();
    // 4 phần, mỗi phần là số 0..=65535 (Validate `Identity/Version`).
    let ns: Option<Vec<u64>> = (parts.len() == 4)
        .then(|| {
            parts
                .iter()
                .map(|p| p.parse::<u64>().ok().filter(|n| *n <= 65535))
                .collect::<Option<Vec<u64>>>()
        })
        .flatten();
    match ns {
        None => fails.push(format!(
            "Identity/Version '{version}' phải là 4 số 0..65535"
        )),
        Some(ns) => {
            if ns[0] == 0 {
                fails.push(format!(
                    "Identity/Version '{version}': Partner Center từ chối phần đầu = 0"
                ));
            }
            if ns[3] != 0 {
                fails.push(format!(
                    "Identity/Version '{version}': phần thứ 4 Store dành riêng, phải = 0"
                ));
            }
            if let Some(want) = want_ver {
                match map_version(want) {
                    Ok(mapped) if version != mapped => fails.push(format!(
                        "Identity/Version '{version}' ≠ '{mapped}' (map từ {want})"
                    )),
                    Ok(_) => {}
                    Err(e) => fails.push(e),
                }
            }
        }
    }
    if ident.attr("ProcessorArchitecture") != Some("x64") {
        fails.push("Identity/ProcessorArchitecture phải là x64".into());
    }

    let props = root.child("Properties");
    let text_of = |tag: &str| -> String {
        props
            .and_then(|p| p.child(tag))
            .map(|n| n.text.clone())
            .unwrap_or_default()
    };
    let disp = text_of("DisplayName");
    let pub_disp = text_of("PublisherDisplayName");
    let desc = text_of("Description");
    // <f:Logo>Assets\storelogo.png</f:Logo> — là phần tử TEXT (giống findtext
    // của ElementTree), không phải thuộc tính.
    let logo = props
        .and_then(|p| p.child("Logo"))
        .map(|n| n.text.clone())
        .unwrap_or_default();

    if disp != expect_display {
        fails.push(format!(
            "DisplayName '{disp}' ≠ tên đã reserve '{expect_display}'"
        ));
    }
    if pub_disp != EXPECTED_PUBLISHER_DISPLAY {
        fails.push(format!(
            "PublisherDisplayName '{pub_disp}' ≠ '{EXPECTED_PUBLISHER_DISPLAY}'"
        ));
    }
    if manifest_mojibake(&text) {
        fails.push(
            "manifest có chuỗi mojibake (UTF-8 mã hoá 2 lần) — template đọc sai encoding".into(),
        );
    }
    if !contains_vietnamese(&desc) {
        fails.push(format!("Description không còn dấu tiếng Việt: '{desc}'"));
    }

    let mut all = Vec::new();
    root.descendants(&mut all);
    if !all
        .iter()
        .any(|n| n.name == "Capability" && n.attr("Name") == Some("runFullTrust"))
    {
        fails.push("thiếu rescap:Capability runFullTrust".into());
    }

    // Ảnh: mọi thuộc tính *Logo / SplashScreen tham chiếu phải tồn tại + đúng cỡ.
    type LogoRef = (String, String, Option<(u32, u32)>);
    let mut refs: Vec<LogoRef> = vec![("Logo".into(), logo, Some(STORE_LOGO_SIZE))];
    for node in &all {
        for (k, v) in &node.attrs {
            if let Some(size) = asset_size(k) {
                refs.push((k.clone(), v.clone(), Some(size)));
            }
        }
        if node.name == "SplashScreen" {
            refs.push((
                "SplashScreen".into(),
                node.attr("Image").map(String::from).unwrap_or_default(),
                Some(SPLASH_SIZE),
            ));
        }
    }
    for (attr, rel, size) in refs {
        let Some(member) = dir.member(&rel) else {
            fails.push(format!("{attr} trỏ '{rel}' không có trong gói"));
            continue;
        };
        let data = dir.read(member).unwrap_or_default();
        match png_size(&data) {
            None => fails.push(format!("{attr} '{rel}' không phải PNG")),
            Some((w, h)) => {
                if let Some((ew, eh)) = size {
                    if (w, h) != (ew, eh) {
                        fails.push(format!("{attr} '{rel}' là {w}x{h}, cần {ew}x{eh}"));
                    }
                }
            }
        }
    }

    for app in all.iter().filter(|n| n.name == "Application") {
        let exe = app.attr("Executable").map(String::from).unwrap_or_default();
        if dir.member(&exe).is_none() {
            fails.push(format!("Application Executable '{exe}' không có trong gói"));
        }
    }

    for rel in REQUIRED_PAYLOAD {
        if dir.member(rel).is_none() {
            fails.push(format!("payload thiếu {rel}"));
        }
    }
    if !dir
        .index
        .keys()
        .any(|k| k.starts_with("data/tables/") && k.ends_with(".toml"))
    {
        fails.push("payload thiếu data/tables/*.toml (engine đọc bảng từ đĩa)".into());
    }
    fails
}

/// CLI: `xtask verify-msix <msix> [flags]` — Err khi có lỗi, mỗi lỗi một dòng
/// `FAIL ...` như bản Python.
pub fn verify_msix_cli(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("cách dùng: cargo xtask verify-msix <file.msix> [--expect-version A.B.C] [--require-store-identity] [--display-name N] [--expect-name NAME]".into());
    }
    let mut it = args.iter();
    let Some(msix) = it.next() else {
        return Err("chưa truyền file .msix".into());
    };
    let mut want_ver: Option<String> = None;
    let mut require_identity = false;
    let mut display_name: Option<String> = None;
    let mut expect_name: Option<String> = None;
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--expect-version" => want_ver = it.next().cloned(),
            "--require-store-identity" => require_identity = true,
            "--display-name" => display_name = it.next().cloned(),
            "--expect-name" => expect_name = it.next().cloned(),
            other => return Err(format!("đối số lạ '{other}'")),
        }
    }
    // R2-95: cắt khoảng trắng và rỗng → mặc định, ĐÚNG như build-msix.ps1 —
    // repo variable dán kèm dấu cách từng làm verify báo lệch tên với gói
    // vừa build.
    let display = display_name
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .or_else(|| {
            std::env::var("MSIX_DISPLAY_NAME")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .unwrap_or_else(|| EXPECTED_DISPLAY_NAME.to_string());
    let dir = extract_msix(Path::new(msix))?;
    let fails = verify_full(
        &dir,
        want_ver.as_deref(),
        require_identity,
        &display,
        expect_name.as_deref().map(str::trim),
    );
    if fails.is_empty() {
        println!("OK   {msix}: identity/version/tên hiển thị/encoding/ảnh/payload đạt luật Store");
        Ok(())
    } else {
        let mut msg = String::new();
        for f in &fails {
            msg.push_str(&format!("FAIL {f}\n"));
            println!("FAIL {f}");
        }
        Err(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crt_runtime_dll_tung_nhom() {
        assert!(is_crt_runtime_dll("VCRUNTIME140.dll"));
        assert!(is_crt_runtime_dll("vcruntime140_1.dll"));
        assert!(is_crt_runtime_dll("MSVCP140.DLL"));
        assert!(is_crt_runtime_dll("ucrtbase.dll"));
        assert!(is_crt_runtime_dll("api-ms-win-crt-stdio-l1-1-0.dll"));
        assert!(!is_crt_runtime_dll("USER32.dll"));
        assert!(!is_crt_runtime_dll("KERNEL32"));
        assert!(!is_crt_runtime_dll("vcruntime.dll")); // thiếu chữ số
        assert!(!is_crt_runtime_dll("api-ms-win-security.dll"));
    }

    #[test]
    fn map_version_khop_build_msix() {
        assert_eq!(map_version("0.2.27").unwrap(), "1.2.27.0");
        assert_eq!(map_version("1.9.99").unwrap(), "2.9.99.0");
        assert!(map_version("0.2").is_err());
        assert!(map_version("0.2.27.1").is_err());
        assert!(map_version("a.b.c").is_err());
    }

    #[test]
    fn png_kich_thuoc_tu_header() {
        let mut png = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
        png.extend_from_slice(&[0u8; 16]); // tới offset 24
        png[16..20].copy_from_slice(&310u32.to_be_bytes());
        png[20..24].copy_from_slice(&150u32.to_be_bytes());
        assert_eq!(png_size(&png), Some((310, 150)));
        assert_eq!(png_size(&png[..20]), None);
        assert_eq!(png_size(b"khong phai png"), None);
    }

    fn mini_manifest(version: &str, desc: &str) -> String {
        let xml = String::new()
            + r#"<?xml version="1.0" encoding="utf-8"?>
<Pkg xmlns:f="http://schemas.microsoft.com/appx/manifest/foundation/windows10" xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10" xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">
  <f:Identity Name="23651Linhi.TextVN" Publisher="CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545" Version="VVV" ProcessorArchitecture="x64"/>
  <f:Properties>
    <f:DisplayName>TextVN</f:DisplayName>
    <f:PublisherDisplayName>LinhBH.CoM</f:PublisherDisplayName>
    <f:Logo>Assets\storelogo.png</f:Logo>
    <f:Description>DDD</f:Description>
  </f:Properties>
  <f:Applications>
    <f:Application Id="TextVN" Executable="TextVN.exe">
      <uap:VisualElements Square150x150Logo="Assets\s150.png" Square44x44Logo="Assets\s44.png" Wide310x150Logo="Assets\w310.png" Square71x71Logo="Assets\s71.png"/>
    </f:Application>
  </f:Applications>
  <f:Capabilities>
    <rescap:Capability Name="runFullTrust"/>
  </f:Capabilities>
</Pkg>
"#;
        xml.replace("VVV", version).replace("DDD", desc)
    }

    fn fake_png(w: u32, h: u32) -> Vec<u8> {
        let mut png = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
        png.extend_from_slice(&[0u8; 16]);
        png[16..20].copy_from_slice(&w.to_be_bytes());
        png[20..24].copy_from_slice(&h.to_be_bytes());
        png
    }

    /// Dựng một "gói MSIX đã giải nén" trong tempdir để test verify_full.
    fn fake_msix_dir(manifest: String) -> MsixDir {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = std::sync::atomic::AtomicUsize::fetch_add(
            &SEQ,
            1,
            std::sync::atomic::Ordering::Relaxed,
        );
        let root = std::env::temp_dir().join(format!("textvn-verify-msix-test-{n}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Assets")).expect("mkdir assets");
        fs::write(root.join("AppxManifest.xml"), manifest).expect("ghi manifest");
        for (name, w, h) in [
            ("storelogo.png", 50, 50),
            ("s150.png", 150, 150),
            ("s44.png", 44, 44),
            ("w310.png", 310, 150),
            ("s71.png", 71, 71),
        ] {
            fs::write(root.join("Assets").join(name), fake_png(w, h)).expect("ghi png");
        }
        for rel in [
            "TextVN.exe",
            "textvn-cli.exe",
            "textvn-tsf.dll",
            "textvn-tsf-x86.dll",
        ] {
            fs::write(root.join(rel), b"MZ").expect("ghi payload");
        }
        fs::create_dir_all(root.join("resources")).expect("mkdir resources");
        for rel in ["textvn.ico", "textvn_v.ico", "textvn_e.ico"] {
            fs::write(root.join("resources").join(rel), b"icon").expect("ghi ico");
        }
        fs::create_dir_all(root.join("data")).expect("mkdir data");
        fs::write(root.join("data/appdb.default.json"), b"{}").expect("ghi appdb");
        fs::create_dir_all(root.join("data/tables")).expect("mkdir tables");
        fs::write(root.join("data/tables/vowels.toml"), b"[a]").expect("ghi toml");
        let mut index = BTreeMap::new();
        walk_index(&root, &root, &mut index);
        MsixDir { root, index }
    }

    #[test]
    fn verify_msix_full_happy_path() {
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, Some("0.2.85"), false, "TextVN", None);
        assert!(fails.is_empty(), "kỳ vọng không lỗi, được: {fails:?}");
    }

    #[test]
    fn verify_msix_bat_loi_tung_luat() {
        // Version sai mapping
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, Some("0.2.99"), false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("map từ 0.2.99")));

        // thiếu dấu tiếng Việt trong Description
        let men = mini_manifest("1.2.85.0", "may go tieng viet");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("không còn dấu tiếng Việt")));

        // mojibake
        let men = mini_manifest(
            "1.2.85.0",
            "Tr\u{00C3}\u{00A9}nh g\u{00C3}\u{00B5} mojibake",
        );
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("mojibake")));

        // phiên bản 4 số sai: đầu = 0
        let men = mini_manifest("0.2.85.0", "Trình gõ tiếng Việt an toàn");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("từ chối phần đầu = 0")));

        // Arch sai
        let men =
            mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn").replace("x64\"", "arm64\"");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("ProcessorArchitecture")));

        // expect-name sai
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let dir = fake_msix_dir(men);
        let fails = verify_full(&dir, None, false, "TextVN", Some("Khac.Name"));
        assert!(fails.iter().any(|f| f.contains("Khac.Name")));
    }

    #[test]
    fn xml_parser_manifest_that() {
        let xml = parse_xml(&mini_manifest("1.2.85.0", "Trình gõ tiếng Việt")).expect("parse");
        let ident = xml.child("Identity").expect("Identity");
        assert_eq!(ident.attr("Name"), Some("23651Linhi.TextVN"));
        assert_eq!(ident.attr("Version"), Some("1.2.85.0"));
        let props = xml.child("Properties").expect("Properties");
        assert_eq!(props.child("DisplayName").unwrap().text, "TextVN");
        assert_eq!(
            props.child("Description").unwrap().text,
            "Trình gõ tiếng Việt"
        );
        let mut all = Vec::new();
        xml.descendants(&mut all);
        assert!(all
            .iter()
            .any(|n| n.name == "Capability" && n.attr("Name") == Some("runFullTrust")));
        // Visual elements chứa đủ 4 thuộc tính Logo
        let ve = all.iter().find(|n| n.name == "VisualElements").expect("VE");
        assert_eq!(ve.attrs.len(), 4);
    }

    #[test]
    fn xml_parser_bat_xml_sai_well_formed() {
        assert!(parse_xml("<a><b></b>").is_err()); // chưa đóng
        assert!(parse_xml("<a></b></a>").is_err()); // đóng sai cặp
        assert!(parse_xml("<a attr=b></a>").is_err()); // attr không nháy
        assert!(parse_xml("van ban tu do").is_err()); // không có thẻ
        assert!(parse_xml("<a><!-- co -- ben trong --></a>").is_err()); // "--" trong comment
    }

    #[test]
    fn mojibake_va_dau_tieng_viet() {
        assert!(manifest_mojibake("Tr\u{00C3}\u{00A9}nh"));
        assert!(manifest_mojibake("x\u{00E1}\u{00BB}y"));
        assert!(manifest_mojibake("x\u{00E1}\u{00BA}y"));
        assert!(manifest_mojibake("x\u{00C6}\u{00B0}y"));
        assert!(!manifest_mojibake("Trình gõ tiếng Việt sạch"));
        assert!(contains_vietnamese("Bộ gõ tiếng Việt"));
        assert!(contains_vietnamese("tr\u{1EA3}n chữ thường"));
        assert!(!contains_vietnamese("plain english only"));
    }
}
