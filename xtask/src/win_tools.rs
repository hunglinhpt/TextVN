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
//! (chạy trên ubuntu LẪN Windows). MSIX = ZIP: đọc trực tiếp trong bộ nhớ bằng
//! parser ZIP + RFC 1951 Inflate thuần std (không phụ thuộc `tar`/`bsdtar`/`unzip`,
//! không tạo thư mục tạm).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::png::{crc32, inflate_raw_bounded};

// ---------------------------------------------------------------------------
// check-pe-imports — port tools/win/check-pe-imports.py
// ---------------------------------------------------------------------------

fn u16_at(d: &[u8], off: usize) -> Result<u16, String> {
    let end = off
        .checked_add(2)
        .ok_or_else(|| format!("PE: tràn offset tại 0x{off:X}"))?;
    d.get(off..end)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| format!("PE: thiếu byte tại 0x{off:X}"))
}

fn u32_at(d: &[u8], off: usize) -> Result<u32, String> {
    let end = off
        .checked_add(4)
        .ok_or_else(|| format!("PE: tràn offset tại 0x{off:X}"))?;
    d.get(off..end)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| format!("PE: thiếu byte tại 0x{off:X}"))
}

fn u64_at(d: &[u8], off: usize) -> Result<u64, String> {
    let end = off
        .checked_add(8)
        .ok_or_else(|| format!("ZIP: tràn offset tại 0x{off:X}"))?;
    d.get(off..end)
        .map(|s| u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]))
        .ok_or_else(|| format!("ZIP: thiếu byte tại 0x{off:X}"))
}

/// ARM-W1 — trường `Machine` của COFF header (`IMAGE_FILE_MACHINE_*`): 0x8664 x64, 0x14C x86,
/// 0xAA64 ARM64 (cả ảnh ARM64X — phân biệt bằng bảng CHPE, ngoài phạm vi kiểm này).
pub fn pe_machine(d: &[u8]) -> Result<u16, String> {
    if d.len() < 0x40 || &d[..2] != b"MZ" {
        return Err("không phải PE (thiếu MZ)".into());
    }
    let pe = u32_at(d, 0x3C)? as usize;
    if d.get(pe..pe.saturating_add(4)) != Some(b"PE\0\0") {
        return Err("không phải PE (thiếu chữ ký PE)".into());
    }
    u16_at(d, pe.checked_add(4).ok_or("PE: tràn Machine")?)
}

/// Tên kiến trúc ↔ mã `Machine` cho `check-pe-imports --machine`.
pub fn machine_code(name: &str) -> Option<u16> {
    match name {
        "x64" | "x86_64" | "amd64" => Some(0x8664),
        "x86" | "i686" => Some(0x14C),
        "arm64" | "aarch64" => Some(0xAA64),
        _ => None,
    }
}

/// Đọc bảng import (và delay-load import nếu có) của một PE (Portable Executable)
/// — parser thuần byte với kiểm tra biên/tràn đầy đủ, KHÔNG cần dumpbin/Visual Studio.
pub fn pe_imports_bytes(d: &[u8]) -> Result<Vec<String>, String> {
    if d.len() < 0x40 || &d[..2] != b"MZ" {
        return Err("không phải PE (thiếu MZ)".into());
    }
    let pe = u32_at(d, 0x3C)? as usize;
    let pe_sig_end = pe
        .checked_add(4)
        .ok_or_else(|| "PE: tràn offset chữ ký PE".to_string())?;
    if d.get(pe..pe_sig_end) != Some(b"PE\0\0") {
        return Err("không phải PE (thiếu chữ ký PE)".into());
    }
    let nsec = u16_at(d, pe.checked_add(6).ok_or("PE: tràn nsec")?)? as usize;
    let opt_size = u16_at(d, pe.checked_add(20).ok_or("PE: tràn opt_size")?)? as usize;
    let opt = pe.checked_add(24).ok_or("PE: tràn optional header")?;
    let magic = u16_at(d, opt)?;
    let dd_rel: usize = match magic {
        0x10B => 96,  // PE32
        0x20B => 112, // PE32+
        other => return Err(format!("PE: magic lạ 0x{other:X}")),
    };
    let dd = opt.checked_add(dd_rel).ok_or("PE: tràn data directory")?;
    let sec = opt.checked_add(opt_size).ok_or("PE: tràn section table")?;
    let sec_bytes = nsec
        .checked_mul(40)
        .and_then(|n| sec.checked_add(n))
        .ok_or("PE: tràn bảng section")?;
    if sec_bytes > d.len() {
        return Err("PE: bảng section vượt quá kích thước file".into());
    }
    let mut sections: Vec<(usize, usize, usize)> = Vec::with_capacity(nsec);
    for i in 0..nsec {
        let off = sec + 40 * i;
        let vsize = u32_at(d, off + 8)? as usize;
        let va = u32_at(d, off + 12)? as usize;
        let rawsize = u32_at(d, off + 16)? as usize;
        let rawptr = u32_at(d, off + 20)? as usize;
        // Chỉ ánh xạ vùng thực sự có trong file (bỏ qua .bss rawsize == 0 và phần đuôi ảo).
        if rawsize == 0 {
            continue;
        }
        let span = if vsize == 0 {
            rawsize
        } else {
            vsize.min(rawsize)
        };
        sections.push((va, span, rawptr));
    }

    // Nếu OptionalHeader nhỏ hơn NumberOfRvaAndSizes hoặc không có DataDirectory[1] → 0 import.
    if opt_size < dd_rel {
        return Ok(Vec::new());
    }
    let num_dd = u32_at(d, dd - 4)? as usize;
    if num_dd <= 1 || opt_size < dd_rel + 16 {
        return Ok(Vec::new());
    }
    let imp_rva = u32_at(d, dd + 8)? as usize;
    let imp_size = u32_at(d, dd + 12)? as usize;

    let rva_to_off = |rva: usize| -> Result<usize, String> {
        for &(va, span, raw) in &sections {
            if rva >= va {
                let delta = rva - va;
                if delta < span {
                    let file_off = raw
                        .checked_add(delta)
                        .ok_or_else(|| format!("PE: tràn file offset từ RVA 0x{rva:X}"))?;
                    if file_off < d.len() {
                        return Ok(file_off);
                    }
                    return Err(format!(
                        "PE: file offset 0x{file_off:X} vượt quá kích thước file"
                    ));
                }
            }
        }
        Err(format!("RVA 0x{rva:X} ngoài mọi section có dữ liệu"))
    };

    let read_cstr = |rva: usize| -> Result<String, String> {
        let noff = rva_to_off(rva)?;
        let tail = d
            .get(noff..)
            .ok_or_else(|| format!("PE: offset tên DLL 0x{noff:X} ngoài file"))?;
        let end = tail
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| "PE: tên DLL không kết thúc bằng \\0".to_string())?;
        Ok(String::from_utf8_lossy(&tail[..end]).into_owned())
    };

    let mut names = Vec::new();
    if imp_rva != 0 && imp_size != 0 {
        let mut off = rva_to_off(imp_rva)?;
        loop {
            let orig = u32_at(d, off)?;
            let ts = u32_at(d, off.checked_add(4).ok_or("PE: tràn descriptor")?)?;
            let fwd = u32_at(d, off.checked_add(8).ok_or("PE: tràn descriptor")?)?;
            let name_rva = u32_at(d, off.checked_add(12).ok_or("PE: tràn descriptor")?)?;
            let first = u32_at(d, off.checked_add(16).ok_or("PE: tràn descriptor")?)?;
            if (orig == 0 && ts == 0 && fwd == 0 && name_rva == 0 && first == 0)
                || (orig == 0 && name_rva == 0 && first == 0)
            {
                break;
            }
            if name_rva == 0 {
                return Err("PE: import descriptor có Name RVA = 0".into());
            }
            names.push(read_cstr(name_rva as usize)?);
            off = off.checked_add(20).ok_or("PE: tràn offset import table")?;
        }
    }

    // Kiểm thêm Delay-Load Import Directory (DataDirectory[13]) để chặn `/DELAYLOAD:vcruntime*.dll`.
    if num_dd > 13 && opt_size >= dd_rel + 14 * 8 {
        let delay_rva = u32_at(d, dd + 13 * 8)? as usize;
        let delay_size = u32_at(d, dd + 13 * 8 + 4)? as usize;
        if delay_rva != 0 && delay_size != 0 {
            let pe32_image_base = if magic == 0x10B {
                u32_at(d, opt.checked_add(28).ok_or("PE: tràn ImageBase")?)? as usize
            } else {
                0
            };
            let mut off = rva_to_off(delay_rva)?;
            loop {
                let attrs = u32_at(d, off)?;
                let name_rva = u32_at(d, off.checked_add(4).ok_or("PE: tràn delay descriptor")?)?;
                let iat = u32_at(d, off.checked_add(12).ok_or("PE: tràn delay descriptor")?)?;
                let int = u32_at(d, off.checked_add(16).ok_or("PE: tràn delay descriptor")?)?;
                if name_rva == 0 && iat == 0 && int == 0 {
                    break;
                }
                if name_rva == 0 {
                    return Err("PE: delay-load descriptor có Name RVA = 0".into());
                }
                let resolved_rva = if (attrs & 1) != 0 {
                    name_rva as usize
                } else if magic == 0x10B {
                    (name_rva as usize)
                        .checked_sub(pe32_image_base)
                        .ok_or("PE: delay-load VA nhỏ hơn ImageBase")?
                } else {
                    return Err(
                        "PE: PE32+ delay-load descriptor thiếu cờ RVA (grAttrs & 1 == 0)".into(),
                    );
                };
                names.push(read_cstr(resolved_rva)?);
                off = off.checked_add(32).ok_or("PE: tràn offset delay table")?;
            }
        }
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

/// E66: `d2d1.dll`, `dwrite.dll`, `dwmapi.dll` PHẢI nạp lười lúc chạy
/// (`LoadLibraryExW(..., LOAD_LIBRARY_SEARCH_SYSTEM32)`) — không bao giờ link tĩnh
/// vào PE (đặc biệt `textvn-tsf.dll` nạp vào mọi tiến trình Win10/11 Enterprise/VDI/sandbox).
fn is_forbidden_eager_graphics_dll(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "d2d1.dll" | "dwrite.dll" | "dwmapi.dll"
    )
}

/// CLI: `xtask check-pe-imports <files>...` — Err khi PE nào phụ thuộc
/// VC++ runtime, link tĩnh DLL đồ họa nặng (E66), hoặc không đọc được.
pub fn check_pe_imports_cli(args: &[String]) -> Result<(), String> {
    // `--machine <x64|x86|arm64>` (ARM-W1): mọi tệp phải đúng kiến trúc — chặn bản ARM64 lẫn
    // DLL x64 (hoặc ngược lại) khi đóng gói nhiều kiến trúc.
    let mut want_machine: Option<u16> = None;
    let mut files: Vec<&String> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--machine" {
            let name = it.next().ok_or("--machine cần giá trị x64|x86|arm64")?;
            want_machine = Some(machine_code(name).ok_or_else(|| format!("--machine lạ: {name}"))?);
        } else {
            files.push(a);
        }
    }
    if files.is_empty() {
        return Err("cách dùng: cargo xtask check-pe-imports [--machine x64|x86|arm64] <file.exe|file.dll> [...]".into());
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
        if let Some(want) = want_machine {
            match pe_machine(&data) {
                Ok(m) if m == want => println!("OK   {path}: Machine 0x{m:04X}"),
                Ok(m) => {
                    println!("FAIL {path}: Machine 0x{m:04X}, cần 0x{want:04X}");
                    bad += 1;
                    continue;
                }
                Err(e) => {
                    println!("FAIL {path}: {e}");
                    bad += 1;
                    continue;
                }
            }
        }
        match pe_imports_bytes(&data) {
            Err(e) => {
                println!("FAIL {path}: {e}");
                bad += 1;
            }
            Ok(dlls) => {
                let crt_hits: Vec<&str> = dlls
                    .iter()
                    .map(String::as_str)
                    .filter(|d| is_crt_runtime_dll(d))
                    .collect();
                let gfx_hits: Vec<&str> = dlls
                    .iter()
                    .map(String::as_str)
                    .filter(|d| is_forbidden_eager_graphics_dll(d))
                    .collect();
                if crt_hits.is_empty() && gfx_hits.is_empty() {
                    println!(
                        "OK   {path}: {} DLL import, không có VC++ runtime hay link tĩnh d2d1/dwrite/dwmapi",
                        dlls.len()
                    );
                } else {
                    if !crt_hits.is_empty() {
                        println!(
                            "FAIL {path}: phụ thuộc VC++ runtime {} — cần +crt-static",
                            crt_hits.join(", ")
                        );
                    }
                    if !gfx_hits.is_empty() {
                        println!(
                            "FAIL {path}: link tĩnh {} (E66) — phải nạp lười qua LoadLibraryExW(System32)",
                            gfx_hits.join(", ")
                        );
                    }
                    bad += 1;
                }
            }
        }
    }
    if bad == 0 {
        Ok(())
    } else {
        Err(format!(
            "{bad} file vi phạm kiểm tra PE imports (VC++ runtime / eager graphics DLL / lỗi đọc PE)"
        ))
    }
}

// ---------------------------------------------------------------------------
// verify-msix — port tools/win/verify-msix.py
// ---------------------------------------------------------------------------

const NS_FOUNDATION: &str = "http://schemas.microsoft.com/appx/manifest/foundation/windows10";
const NS_UAP: &str = "http://schemas.microsoft.com/appx/manifest/uap/windows10";
const NS_RESCAP: &str =
    "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities";

const PLACEHOLDER_NAME: &str = "LinhBH.CoM.TextVN";
const EXPECTED_PUBLISHER: &str = "CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545";
const EXPECTED_DISPLAY_NAME: &str = "TextVN";
const EXPECTED_PUBLISHER_DISPLAY: &str = "LinhBH.CoM";
const STORE_LOGO_SIZE: (u32, u32) = (50, 50);
const SPLASH_SIZE: (u32, u32) = (620, 300);

/// Thuộc tính VisualElements / DefaultTile → kích thước bắt buộc (scale-100).
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

/// Header PNG: signature 8 byte + chunk đầu phải là `IHDR` + width/height BE tại byte 16..24.
fn png_size(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" || &data[12..16] != b"IHDR" {
        return None;
    }
    Some((
        u32::from_be_bytes([data[16], data[17], data[18], data[19]]),
        u32::from_be_bytes([data[20], data[21], data[22], data[23]]),
    ))
}

// ---------------------------------------------------------------------------
// XML 1.0 strict + namespace-aware parser (dùng chung cho verify-msix & linux_tools)
// ---------------------------------------------------------------------------

pub(crate) struct Xml {
    /// Local name sau dấu ':' (ví dụ `Identity`, `Capability`, `exec`).
    pub(crate) name: String,
    /// Namespace URI đã giải (rỗng nếu không có default xmlns và không có prefix).
    pub(crate) ns: String,
    /// Qualified name gốc (để khớp chính xác thẻ đóng).
    qname: String,
    /// Thuộc tính không phải `xmlns`/`xmlns:*`, giữ nguyên qname (thuộc tính
    /// không có prefix không kế thừa default xmlns theo chuẩn XML Namespaces).
    pub(crate) attrs: Vec<(String, String)>,
    pub(crate) children: Vec<Xml>,
    /// Toàn bộ nội dung text trực tiếp của phần tử (KHÔNG `.trim()` để giữ kiểm
    /// khoảng trắng đầu/cuối R2-95 như `ElementTree.findtext`).
    pub(crate) text: String,
}

impl Xml {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
    pub(crate) fn child(&self, name: &str) -> Option<&Xml> {
        self.children.iter().find(|c| c.name == name)
    }
    fn child_ns(&self, ns: &str, name: &str) -> Option<&Xml> {
        self.children.iter().find(|c| c.ns == ns && c.name == name)
    }
    pub(crate) fn descendants<'a>(&'a self, out: &mut Vec<&'a Xml>) {
        for c in &self.children {
            out.push(c);
            c.descendants(out);
        }
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn is_xml_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || !c.is_ascii()
}

fn is_xml_name_char(c: char) -> bool {
    is_xml_name_start(c) || c.is_ascii_digit() || matches!(c, '-' | '.' | ':')
}

/// Kiểm tra QName XML 1.0 hợp lệ (tối đa 1 dấu `:`, không mở/đóng bằng `:`,
/// không bắt đầu bằng chữ số/`-`/`.`).
fn valid_xml_name(tag: &str) -> bool {
    if tag.is_empty() {
        return false;
    }
    let mut parts = tag.split(':');
    let first = parts.next().unwrap_or("");
    let second = parts.next();
    if parts.next().is_some() {
        return false; // > 1 dấu ':'
    }
    let valid_ncname = |s: &str| -> bool {
        let mut chars = s.chars();
        let Some(c0) = chars.next() else {
            return false;
        };
        is_xml_name_start(c0) && chars.all(|c| is_xml_name_char(c) && c != ':')
    };
    match second {
        None => valid_ncname(first),
        Some(local) => valid_ncname(first) && valid_ncname(local),
    }
}

fn split_qname(qname: &str) -> (Option<&str>, &str) {
    match qname.split_once(':') {
        Some((pref, local)) => (Some(pref), local),
        None => (None, qname),
    }
}

fn is_valid_xml_char(cp: u32) -> bool {
    matches!(
        cp,
        0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF
    )
}

/// Giải mã 5 thực thể chuẩn XML + numeric char reference (`&#N;` / `&#xH;`).
/// Trả `Err` với `&` trần hoặc thực thể không hợp lệ (đảm bảo well-formed XML 1.0).
fn decode_entities(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos + 1..];
        let Some(semi) = tail.find(';') else {
            return Err("XML: ký tự '&' không kết thúc bằng ';'".into());
        };
        let ent = &tail[..semi];
        match ent {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            _ if ent.starts_with('#') => {
                let num = &ent[1..];
                let cp = if let Some(hex) = num.strip_prefix('x') {
                    if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(format!("XML: thực thể số không hợp lệ '&{ent};'"));
                    }
                    u32::from_str_radix(hex, 16)
                } else {
                    if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(format!("XML: thực thể số không hợp lệ '&{ent};'"));
                    }
                    num.parse::<u32>()
                }
                .map_err(|_| format!("XML: thực thể số không hợp lệ '&{ent};'"))?;
                if !is_valid_xml_char(cp) {
                    return Err(format!("XML: codepoint không hợp lệ XML 1.0 '&{ent};'"));
                }
                let ch = char::from_u32(cp)
                    .ok_or_else(|| format!("XML: codepoint không hợp lệ '&{ent};'"))?;
                out.push(ch);
            }
            _ => return Err(format!("XML: thực thể không xác định '&{ent};'")),
        }
        rest = &tail[semi + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Tìm dấu `>` đóng thẻ có tính đến chuỗi trích dẫn `'...'` / `"..."` (XML 1.0 cho phép `>` trong attr).
fn find_tag_end(bytes: &[u8], start: usize) -> Result<usize, String> {
    let mut i = start + 1;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == q {
                    quote = None;
                } else if b == b'<' {
                    return Err("XML: ký tự '<' không được phép trong giá trị thuộc tính".into());
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'>' {
                    return Ok(i);
                } else if b == b'<' {
                    return Err("XML: gặp '<' bên trong thẻ chưa đóng".into());
                }
            }
        }
        i += 1;
    }
    Err("XML: thẻ không có '>'".into())
}

fn resolve_ns(
    ns_stack: &[BTreeMap<String, String>],
    prefix: Option<&str>,
) -> Result<String, String> {
    let key = prefix.unwrap_or("");
    for frame in ns_stack.iter().rev() {
        if let Some(uri) = frame.get(key) {
            return Ok(uri.clone());
        }
    }
    match prefix {
        None => Ok(String::new()),
        Some(p) => Err(format!("XML: namespace prefix '{p}' chưa được khai báo")),
    }
}

pub(crate) fn parse_xml(src: &str) -> Result<Xml, String> {
    let src = src.trim_start_matches('\u{FEFF}');
    let bytes = src.as_bytes();
    let mut i = 0usize;

    let mut base_ns = BTreeMap::new();
    base_ns.insert(
        "xml".to_string(),
        "http://www.w3.org/XML/1998/namespace".to_string(),
    );
    let mut ns_stack: Vec<BTreeMap<String, String>> = vec![base_ns];

    // Root ảo đỡ stack; sau khi parse xong phải có ĐÚNG 1 phần tử gốc.
    let mut stack: Vec<Xml> = vec![Xml {
        name: String::new(),
        ns: String::new(),
        qname: String::new(),
        attrs: Vec::new(),
        children: Vec::new(),
        text: String::new(),
    }];

    while i < bytes.len() {
        if bytes[i] != b'<' {
            let start_text = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let chunk = &src[start_text..i];
            if chunk.contains("]]>") {
                return Err("XML: chuỗi ']]>' không được phép ngoài CDATA".into());
            }
            if stack.len() == 1 {
                if !chunk.chars().all(|c| c.is_ascii_whitespace()) {
                    return Err(format!(
                        "XML: kỳ vọng '<' tại offset {start_text} — text nằm ngoài thẻ"
                    ));
                }
            } else {
                let decoded = decode_entities(chunk)?;
                stack.last_mut().expect("non-empty").text.push_str(&decoded);
            }
            continue;
        }

        // Processing instruction / XML declaration `<?...?>`
        if bytes[i..].starts_with(b"<?") {
            let Some(k) = find(&bytes[i + 2..], b"?>") else {
                return Err("XML: `<?` không có `?>`".into());
            };
            let target_body = src[i + 2..i + 2 + k].trim();
            if target_body.is_empty() {
                return Err("XML: processing instruction rỗng `<??>`".into());
            }
            i += 2 + k + 2;
            continue;
        }

        // Comment `<!--...-->` (cấm `--` và `-` ngay trước `-->` theo XML 1.0 §2.5)
        if bytes[i..].starts_with(b"<!--") {
            let body = &bytes[i + 4..];
            let Some(k) = find(body, b"-->") else {
                return Err("XML: comment không có \"-->\"".into());
            };
            if body[..k].windows(2).any(|w| w == b"--") || body[..k].ends_with(b"-") {
                return Err("XML: comment chứa \"--\" (không well-formed)".into());
            }
            i += 4 + k + 3;
            continue;
        }

        // CDATA `<![CDATA[...]]>` — chỉ hợp lệ bên trong thẻ mở.
        if bytes[i..].starts_with(b"<![CDATA[") {
            if stack.len() == 1 {
                return Err("XML: CDATA nằm ngoài phần tử gốc".into());
            }
            let body = &bytes[i + 9..];
            let Some(k) = find(body, b"]]>") else {
                return Err("XML: CDATA không có \"]]>\" đóng".into());
            };
            stack
                .last_mut()
                .expect("non-empty")
                .text
                .push_str(&src[i + 9..i + 9 + k]);
            i += 9 + k + 3;
            continue;
        }

        // Các cấu trúc `<!...>` khác (DOCTYPE / rác) không được phép trong manifest/component.
        if bytes[i..].starts_with(b"<!") {
            return Err(format!(
                "XML: cấu trúc `<!...>` không được hỗ trợ tại offset {i}"
            ));
        }

        let close = find_tag_end(bytes, i)?;
        let raw_inner = &src[i + 1..close];
        i = close + 1;

        // Thẻ đóng `</QName>`
        if let Some(after_slash) = raw_inner.strip_prefix('/') {
            let qname = after_slash.trim_end_matches(|c: char| c.is_ascii_whitespace());
            if qname.is_empty()
                || qname.chars().any(|c| c.is_ascii_whitespace())
                || qname.ends_with('/')
                || !valid_xml_name(qname)
            {
                return Err(format!("XML: thẻ đóng không hợp lệ '</{after_slash}>'"));
            }
            if stack.len() <= 1 {
                return Err(format!("XML: đóng </{qname}> thừa"));
            }
            let opened = stack.pop().expect("checked len");
            ns_stack.pop();
            if opened.qname != qname {
                return Err(format!(
                    "XML: đóng </{qname}> nhưng đang mở <{}>",
                    opened.qname
                ));
            }
            stack.last_mut().expect("root ảo").children.push(opened);
            continue;
        }

        // Thẻ mở `<QName ...>` hoặc tự đóng `<QName .../>`
        let (self_closing, body) = match raw_inner.strip_suffix('/') {
            Some(b) => (true, b),
            None => (false, raw_inner),
        };
        if body.is_empty() || body.as_bytes()[0].is_ascii_whitespace() {
            return Err("XML: tên thẻ mở rỗng hoặc có khoảng trắng đầu".into());
        }
        let name_end = body
            .find(|c: char| c.is_ascii_whitespace())
            .unwrap_or(body.len());
        let qname = &body[..name_end];
        if !valid_xml_name(qname) {
            return Err(format!("XML: tên thẻ lạ '{qname}'"));
        }

        // Parse thuộc tính: bắt buộc có khoảng trắng giữa các thuộc tính, cho phép S? '=' S?.
        let attr_src = &body[name_end..];
        let ab = attr_src.as_bytes();
        let mut p = 0usize;
        let mut seen_keys = BTreeSet::new();
        let mut raw_attrs: Vec<(String, String)> = Vec::new();
        let mut ns_frame = BTreeMap::new();

        while p < ab.len() {
            let ws_start = p;
            while p < ab.len() && ab[p].is_ascii_whitespace() {
                p += 1;
            }
            if p >= ab.len() {
                break;
            }
            if p == ws_start {
                return Err(format!(
                    "XML: thiếu khoảng trắng giữa các thuộc tính gần '{}'",
                    &attr_src[p..]
                ));
            }
            let kstart = p;
            while p < ab.len() && !ab[p].is_ascii_whitespace() && ab[p] != b'=' {
                p += 1;
            }
            let key = &attr_src[kstart..p];
            if !valid_xml_name(key) {
                return Err(format!("XML: tên thuộc tính không hợp lệ '{key}'"));
            }
            if !seen_keys.insert(key.to_string()) {
                return Err(format!("XML: trùng thuộc tính '{key}' trên <{qname}>"));
            }
            while p < ab.len() && ab[p].is_ascii_whitespace() {
                p += 1;
            }
            if ab.get(p) != Some(&b'=') {
                return Err(format!("XML: thuộc tính '{key}' thiếu '='"));
            }
            p += 1;
            while p < ab.len() && ab[p].is_ascii_whitespace() {
                p += 1;
            }
            let Some(&quote) = ab.get(p) else {
                return Err(format!("XML: thuộc tính '{key}' thiếu giá trị"));
            };
            if quote != b'"' && quote != b'\'' {
                return Err(format!(
                    "XML: giá trị thuộc tính '{key}' không nằm trong nháy"
                ));
            }
            p += 1;
            let vstart = p;
            let Some(vrel) = attr_src[vstart..].find(quote as char) else {
                return Err(format!("XML: thuộc tính '{key}' không đóng nháy"));
            };
            let val = decode_entities(&attr_src[vstart..vstart + vrel])?;
            p = vstart + vrel + 1;

            if key == "xmlns" {
                ns_frame.insert(String::new(), val);
            } else if let Some(pref) = key.strip_prefix("xmlns:") {
                if val.is_empty() {
                    return Err(format!(
                        "XML: không được gán namespace rỗng cho prefix '{pref}'"
                    ));
                }
                ns_frame.insert(pref.to_string(), val);
            } else {
                raw_attrs.push((key.to_string(), val));
            }
        }

        ns_stack.push(ns_frame);
        let (pref, local) = split_qname(qname);
        let ns = resolve_ns(&ns_stack, pref)?;
        // Kiểm tra prefix của thuộc tính có prefix (nếu có) đều đã được khai báo.
        for (k, _) in &raw_attrs {
            if let (Some(ap), _) = split_qname(k) {
                let _ = resolve_ns(&ns_stack, Some(ap))?;
            }
        }

        let node = Xml {
            name: local.to_string(),
            ns,
            qname: qname.to_string(),
            attrs: raw_attrs,
            children: Vec::new(),
            text: String::new(),
        };
        if self_closing {
            ns_stack.pop();
            stack.last_mut().expect("root ảo").children.push(node);
        } else {
            stack.push(node);
        }
    }

    if stack.len() != 1 {
        return Err(format!("XML: {} thẻ chưa đóng", stack.len() - 1));
    }
    let mut roots = stack.pop().expect("checked len").children;
    if roots.is_empty() {
        return Err("XML: không có phần tử gốc".into());
    }
    if roots.len() > 1 {
        return Err(format!(
            "XML: nhiều hơn 1 phần tử gốc ({} phần tử)",
            roots.len()
        ));
    }
    Ok(roots.remove(0))
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

// ---------------------------------------------------------------------------
// Trình đọc ZIP/MSIX thuần std trong bộ nhớ (Stored + Deflate + ZIP64 + Data Descriptor)
// ---------------------------------------------------------------------------

struct MsixArchive {
    /// Key = path lowercase chuẩn hoá `/` → dữ liệu đã giải nén.
    entries: BTreeMap<String, Vec<u8>>,
    /// Tên entry giữ nguyên hoa/thường (chuẩn hoá `\` → `/`) để kiểm chính xác `AppxManifest.xml`.
    exact_names: BTreeSet<String>,
}

impl MsixArchive {
    fn has_exact(&self, exact: &str) -> bool {
        self.exact_names.contains(exact)
    }
    fn get(&self, rel: &str) -> Option<&[u8]> {
        self.entries
            .get(&rel.replace('\\', "/").to_lowercase())
            .map(Vec::as_slice)
    }
}

fn find_eocd(data: &[u8]) -> Result<usize, String> {
    if data.len() < 22 {
        return Err("MSIX/ZIP: file quá nhỏ (< 22 bytes)".into());
    }
    let min_off = data.len().saturating_sub(22 + 65_535);
    let mut pos = data.len() - 22;
    loop {
        if &data[pos..pos + 4] == b"PK\x05\x06" {
            let comment_len = u16::from_le_bytes([data[pos + 20], data[pos + 21]]) as usize;
            if pos + 22 + comment_len == data.len() {
                return Ok(pos);
            }
        }
        if pos == min_off {
            break;
        }
        pos -= 1;
    }
    Err("MSIX/ZIP: không tìm thấy End of Central Directory (PK\\x05\\x06)".into())
}

type Zip64Sizes = (Option<u64>, Option<u64>, Option<u64>);

fn parse_zip64_extra(
    extra: &[u8],
    need_uncomp: bool,
    need_comp: bool,
    need_local: bool,
) -> Result<Zip64Sizes, String> {
    let mut i = 0usize;
    while i + 4 <= extra.len() {
        let header_id = u16::from_le_bytes([extra[i], extra[i + 1]]);
        let data_size = u16::from_le_bytes([extra[i + 2], extra[i + 3]]) as usize;
        i += 4;
        if i + data_size > extra.len() {
            break;
        }
        if header_id == 0x0001 {
            let block = &extra[i..i + data_size];
            let mut off = 0usize;
            let mut uncomp = None;
            let mut comp = None;
            let mut local = None;
            if need_uncomp {
                uncomp = Some(u64_at(block, off)?);
                off += 8;
            }
            if need_comp {
                comp = Some(u64_at(block, off)?);
                off += 8;
            }
            if need_local {
                local = Some(u64_at(block, off)?);
            }
            return Ok((uncomp, comp, local));
        }
        i += data_size;
    }
    if need_uncomp || need_comp || need_local {
        return Err("MSIX/ZIP64: thiếu hoặc cắt cụt ZIP64 extra field (0x0001)".into());
    }
    Ok((None, None, None))
}

fn read_msix_bytes(data: &[u8]) -> Result<MsixArchive, String> {
    const MAX_ARCHIVE_UNCOMP: usize = 512 * 1024 * 1024;
    let eocd_off = find_eocd(data)?;
    let mut cd_entries = u16_at(data, eocd_off + 10)? as u64;
    let mut cd_size = u32_at(data, eocd_off + 12)? as u64;
    let mut cd_offset = u32_at(data, eocd_off + 16)? as u64;

    // Hỗ trợ ZIP64 EOCD Locator (`PK\x06\x07`, 20 bytes ngay trước EOCD).
    let need_z64_cd = cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF;
    if cd_entries == 0xFFFF || need_z64_cd {
        if eocd_off >= 20 && &data[eocd_off - 20..eocd_off - 16] == b"PK\x06\x07" {
            let z64_off = usize::try_from(u64_at(data, eocd_off - 12)?)
                .map_err(|_| "MSIX/ZIP64: tràn offset ZIP64 EOCD")?;
            if z64_off.checked_add(56).is_none_or(|end| end > data.len())
                || &data[z64_off..z64_off + 4] != b"PK\x06\x06"
            {
                return Err("MSIX/ZIP64: thiếu hoặc cắt cụt ZIP64 EOCD (PK\\x06\\x06)".into());
            }
            cd_entries = u64_at(data, z64_off + 32)?;
            cd_size = u64_at(data, z64_off + 40)?;
            cd_offset = u64_at(data, z64_off + 48)?;
        } else if need_z64_cd {
            return Err(
                "MSIX/ZIP64: EOCD báo sentinel 0xFFFFFFFF nhưng thiếu ZIP64 Locator".into(),
            );
        }
    }

    let cd_start = usize::try_from(cd_offset).map_err(|_| "ZIP: tràn cd_offset")?;
    let cd_len = usize::try_from(cd_size).map_err(|_| "ZIP: tràn cd_size")?;
    let cd_end = cd_start
        .checked_add(cd_len)
        .filter(|&end| end <= data.len())
        .ok_or("MSIX/ZIP: Central Directory vượt quá kích thước file")?;

    let mut entries = BTreeMap::new();
    let mut exact_names = BTreeSet::new();
    let mut total_uncomp: usize = 0;
    let mut pos = cd_start;
    for _ in 0..cd_entries {
        if pos + 46 > cd_end || &data[pos..pos + 4] != b"PK\x01\x02" {
            return Err(format!(
                "MSIX/ZIP: Central Directory header lỗi tại offset 0x{pos:X}"
            ));
        }
        let method = u16_at(data, pos + 10)?;
        let expected_crc = u32_at(data, pos + 16)?;
        let comp32 = u32_at(data, pos + 20)?;
        let uncomp32 = u32_at(data, pos + 24)?;
        let name_len = u16_at(data, pos + 28)? as usize;
        let extra_len = u16_at(data, pos + 30)? as usize;
        let comment_len = u16_at(data, pos + 32)? as usize;
        let local32 = u32_at(data, pos + 42)?;

        let name_start = pos + 46;
        let extra_start = name_start + name_len;
        let next_pos = extra_start + extra_len + comment_len;
        if next_pos > cd_end {
            return Err("MSIX/ZIP: entry Central Directory bị cắt cụt".into());
        }
        let raw_name = String::from_utf8_lossy(&data[name_start..extra_start]).replace('\\', "/");
        let extra = &data[extra_start..extra_start + extra_len];
        pos = next_pos;

        if raw_name.ends_with('/') {
            continue;
        }
        let lower = raw_name.to_lowercase();
        if entries.contains_key(&lower) {
            return Err(format!(
                "MSIX/ZIP: trùng entry (không phân biệt hoa/thường): '{raw_name}'"
            ));
        }
        let (z_uncomp, z_comp, z_local) =
            if uncomp32 == 0xFFFF_FFFF || comp32 == 0xFFFF_FFFF || local32 == 0xFFFF_FFFF {
                parse_zip64_extra(
                    extra,
                    uncomp32 == 0xFFFF_FFFF,
                    comp32 == 0xFFFF_FFFF,
                    local32 == 0xFFFF_FFFF,
                )?
            } else {
                (None, None, None)
            };
        let comp_size =
            usize::try_from(z_comp.unwrap_or(comp32 as u64)).map_err(|_| "ZIP: tràn comp_size")?;
        let uncomp_size = usize::try_from(z_uncomp.unwrap_or(uncomp32 as u64))
            .map_err(|_| "ZIP: tràn uncomp_size")?;
        let local_off = usize::try_from(z_local.unwrap_or(local32 as u64))
            .map_err(|_| "ZIP: tràn local_off")?;
        total_uncomp = total_uncomp
            .checked_add(uncomp_size)
            .filter(|&t| t <= MAX_ARCHIVE_UNCOMP)
            .ok_or("MSIX/ZIP: tổng kích thước giải nén vượt giới hạn an toàn")?;

        if local_off.checked_add(30).is_none_or(|e| e > data.len())
            || &data[local_off..local_off + 4] != b"PK\x03\x04"
        {
            return Err(format!(
                "MSIX/ZIP: thiếu Local File Header cho '{raw_name}' tại 0x{local_off:X}"
            ));
        }
        let lh_name_len = u16_at(data, local_off + 26)? as usize;
        let lh_extra_len = u16_at(data, local_off + 28)? as usize;
        let data_start = local_off
            .checked_add(30)
            .and_then(|o| o.checked_add(lh_name_len))
            .and_then(|o| o.checked_add(lh_extra_len))
            .ok_or("ZIP: tràn data_start")?;
        let data_end = data_start
            .checked_add(comp_size)
            .filter(|&e| e <= data.len())
            .ok_or_else(|| format!("MSIX/ZIP: dữ liệu '{raw_name}' vượt cuối file"))?;
        let comp_bytes = &data[data_start..data_end];

        let payload = match method {
            0 => {
                if comp_size != uncomp_size {
                    return Err(format!(
                        "MSIX/ZIP: '{raw_name}' Stored nhưng comp({comp_size}) != uncomp({uncomp_size})"
                    ));
                }
                comp_bytes.to_vec()
            }
            8 => inflate_raw_bounded(comp_bytes, uncomp_size)
                .map_err(|e| format!("MSIX/ZIP: lỗi giải nén '{raw_name}': {e}"))?,
            other => {
                return Err(format!(
                    "MSIX/ZIP: '{raw_name}' dùng phương thức nén {other} không hỗ trợ (chỉ 0=Stored, 8=Deflate)"
                ));
            }
        };
        if payload.len() != uncomp_size {
            return Err(format!(
                "MSIX/ZIP: '{raw_name}' giải nén ra {} bytes, kỳ vọng {uncomp_size}",
                payload.len()
            ));
        }
        let got_crc = crc32(&payload);
        if got_crc != expected_crc {
            return Err(format!(
                "MSIX/ZIP: '{raw_name}' sai CRC-32 (got 0x{got_crc:08X} != want 0x{expected_crc:08X})"
            ));
        }
        exact_names.insert(raw_name);
        entries.insert(lower, payload);
    }

    if entries.is_empty() {
        return Err("MSIX/ZIP: gói rỗng (0 file)".into());
    }
    Ok(MsixArchive {
        entries,
        exact_names,
    })
}

fn read_msix_file(msix: &Path) -> Result<MsixArchive, String> {
    let data = fs::read(msix).map_err(|e| format!("không đọc được {}: {e}", msix.display()))?;
    read_msix_bytes(&data)
}

/// Toàn bộ luật Store + payload — port 1:1 `check()` của verify-msix.py.
fn verify_full(
    dir: &MsixArchive,
    want_ver: Option<&str>,
    require_identity: bool,
    expect_display: &str,
    expect_name: Option<&str>,
) -> Vec<String> {
    let mut fails = Vec::new();
    // Khớp chính xác hoa/thường "AppxManifest.xml" (giống `if "AppxManifest.xml" not in names`).
    if !dir.has_exact("AppxManifest.xml") {
        return vec!["thiếu AppxManifest.xml".into()];
    }
    let Some(raw) = dir.get("AppxManifest.xml") else {
        return vec!["thiếu AppxManifest.xml".into()];
    };
    let text = match String::from_utf8(raw.to_vec()) {
        Ok(t) => t,
        Err(_) => return vec!["AppxManifest.xml không phải UTF-8".into()],
    };
    let root = match parse_xml(&text) {
        Ok(r) => r,
        Err(e) => return vec![format!("AppxManifest.xml XML lỗi: {e}")],
    };
    let Some(ident) = root.child_ns(NS_FOUNDATION, "Identity") else {
        return vec!["thiếu <Identity> (namespace foundation/windows10)".into()];
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

    // 4 phần thuần chữ số 0..=65535 (từ chối dấu '+' như "+1.2.85.0").
    let parts: Vec<&str> = version.split('.').collect();
    let ns: Option<Vec<u64>> = (parts.len() == 4)
        .then(|| {
            parts
                .iter()
                .map(|p| {
                    if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                        None
                    } else {
                        p.parse::<u64>().ok().filter(|n| *n <= 65535)
                    }
                })
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
        }
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
    if ident.attr("ProcessorArchitecture") != Some("x64") {
        fails.push("Identity/ProcessorArchitecture phải là x64".into());
    }

    let props = root.child_ns(NS_FOUNDATION, "Properties");
    let text_of = |tag: &str| -> String {
        props
            .and_then(|p| p.child_ns(NS_FOUNDATION, tag))
            .map(|n| n.text.clone())
            .unwrap_or_default()
    };
    let disp = text_of("DisplayName");
    let pub_disp = text_of("PublisherDisplayName");
    let desc = text_of("Description");
    let logo = text_of("Logo");

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
    if !all.iter().any(|n| {
        n.ns == NS_RESCAP && n.name == "Capability" && n.attr("Name") == Some("runFullTrust")
    }) {
        fails.push("thiếu rescap:Capability runFullTrust".into());
    }

    // Ảnh: mọi thuộc tính *Logo / uap:SplashScreen tham chiếu phải tồn tại + đúng cỡ.
    type LogoRef = (String, String, Option<(u32, u32)>);
    let mut refs: Vec<LogoRef> = vec![("Logo".into(), logo, Some(STORE_LOGO_SIZE))];
    for node in &all {
        for (k, v) in &node.attrs {
            if let Some(size) = asset_size(k) {
                refs.push((k.clone(), v.clone(), Some(size)));
            }
        }
        if node.ns == NS_UAP && node.name == "SplashScreen" {
            refs.push((
                "SplashScreen".into(),
                node.attr("Image").map(String::from).unwrap_or_default(),
                Some(SPLASH_SIZE),
            ));
        }
    }
    for (attr, rel, size) in refs {
        let Some(data) = dir.get(&rel) else {
            fails.push(format!("{attr} trỏ '{rel}' không có trong gói"));
            continue;
        };
        match png_size(data) {
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

    for app in all
        .iter()
        .filter(|n| n.ns == NS_FOUNDATION && n.name == "Application")
    {
        let exe = app.attr("Executable").map(String::from).unwrap_or_default();
        if dir.get(&exe).is_none() {
            fails.push(format!("Application Executable '{exe}' không có trong gói"));
        }
    }

    for rel in REQUIRED_PAYLOAD {
        if dir.get(rel).is_none() {
            fails.push(format!("payload thiếu {rel}"));
        }
    }
    if !dir
        .entries
        .keys()
        .any(|k| k.starts_with("data/tables/") && k.ends_with(".toml"))
    {
        fails.push("payload thiếu data/tables/*.toml (engine đọc bảng từ đĩa)".into());
    }
    fails
}

struct VerifyMsixOpts {
    msix: String,
    want_ver: Option<String>,
    require_identity: bool,
    display_name: Option<String>,
    expect_name: Option<String>,
}

fn parse_verify_msix_args(args: &[String]) -> Result<VerifyMsixOpts, String> {
    if args.is_empty() {
        return Err("cách dùng: cargo xtask verify-msix <file.msix> [--expect-version A.B.C] [--require-store-identity] [--display-name N] [--expect-name NAME]".into());
    }
    let mut msix: Option<String> = None;
    let mut want_ver: Option<String> = None;
    let mut require_identity = false;
    let mut display_name: Option<String> = None;
    let mut expect_name: Option<String> = None;

    let take_val = |flag: &str,
                    inline: Option<&str>,
                    it: &mut std::slice::Iter<'_, String>|
     -> Result<String, String> {
        if let Some(v) = inline {
            return Ok(v.to_string());
        }
        let next = it
            .next()
            .ok_or_else(|| format!("thiếu giá trị sau `{flag}`"))?;
        if matches!(
            next.as_str(),
            "--expect-version" | "--require-store-identity" | "--display-name" | "--expect-name"
        ) || next.starts_with("--expect-version=")
            || next.starts_with("--display-name=")
            || next.starts_with("--expect-name=")
        {
            return Err(format!("thiếu giá trị sau `{flag}` (gặp cờ `{next}`)"));
        }
        Ok(next.clone())
    };

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--require-store-identity" {
            require_identity = true;
        } else if arg == "--expect-version" {
            want_ver = Some(take_val("--expect-version", None, &mut it)?);
        } else if let Some(v) = arg.strip_prefix("--expect-version=") {
            want_ver = Some(take_val("--expect-version", Some(v), &mut it)?);
        } else if arg == "--display-name" {
            display_name = Some(take_val("--display-name", None, &mut it)?);
        } else if let Some(v) = arg.strip_prefix("--display-name=") {
            display_name = Some(take_val("--display-name", Some(v), &mut it)?);
        } else if arg == "--expect-name" {
            expect_name = Some(take_val("--expect-name", None, &mut it)?);
        } else if let Some(v) = arg.strip_prefix("--expect-name=") {
            expect_name = Some(take_val("--expect-name", Some(v), &mut it)?);
        } else if arg.starts_with('-') {
            return Err(format!("đối số lạ '{arg}'"));
        } else if msix.is_none() {
            msix = Some(arg.clone());
        } else {
            return Err(format!("thừa đối số vị trí '{arg}'"));
        }
    }
    let Some(msix) = msix else {
        return Err("chưa truyền file .msix".into());
    };
    Ok(VerifyMsixOpts {
        msix,
        want_ver,
        require_identity,
        display_name,
        expect_name,
    })
}

/// CLI: `xtask verify-msix <msix> [flags]` — Err khi có lỗi, mỗi lỗi một dòng
/// `FAIL ...` như bản Python.
pub fn verify_msix_cli(args: &[String]) -> Result<(), String> {
    let opts = parse_verify_msix_args(args)?;
    // R2-95: cắt khoảng trắng và rỗng → mặc định, ĐÚNG như build-msix.ps1.
    let display = opts
        .display_name
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .or_else(|| {
            std::env::var("MSIX_DISPLAY_NAME")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .unwrap_or_else(|| EXPECTED_DISPLAY_NAME.to_string());
    let expect_name = opts
        .expect_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let dir = read_msix_file(Path::new(&opts.msix))?;
    let fails = verify_full(
        &dir,
        opts.want_ver.as_deref(),
        opts.require_identity,
        &display,
        expect_name,
    );
    if fails.is_empty() {
        println!(
            "OK   {}: identity/version/tên hiển thị/encoding/ảnh/payload đạt luật Store",
            opts.msix
        );
        Ok(())
    } else {
        for f in &fails {
            println!("FAIL {f}");
        }
        Err(format!("{} lỗi kiểm tra MSIX ({})", fails.len(), opts.msix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::deflate_fixed;

    /// Dựng file ZIP hợp lệ (hỗ trợ cả method 0=Stored và 8=Deflate) trong bộ nhớ để test.
    fn build_zip(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut cd = Vec::new();
        for &(name, data, use_deflate) in files {
            let local_off = buf.len() as u32;
            let crc = crc32(data);
            let (method, payload): (u16, Vec<u8>) = if use_deflate {
                (8, deflate_fixed(data))
            } else {
                (0, data.to_vec())
            };
            let name_b = name.as_bytes();
            // Local file header
            buf.extend_from_slice(b"PK\x03\x04");
            buf.extend_from_slice(&20u16.to_le_bytes()); // version needed
            buf.extend_from_slice(&0u16.to_le_bytes()); // flags
            buf.extend_from_slice(&method.to_le_bytes());
            buf.extend_from_slice(&0u16.to_le_bytes()); // time
            buf.extend_from_slice(&0u16.to_le_bytes()); // date
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
            buf.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
            buf.extend_from_slice(&0u16.to_le_bytes()); // extra len
            buf.extend_from_slice(name_b);
            buf.extend_from_slice(&payload);

            // Central directory header
            cd.extend_from_slice(b"PK\x01\x02");
            cd.extend_from_slice(&20u16.to_le_bytes());
            cd.extend_from_slice(&20u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&method.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&crc.to_le_bytes());
            cd.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            cd.extend_from_slice(&(data.len() as u32).to_le_bytes());
            cd.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u32.to_le_bytes());
            cd.extend_from_slice(&local_off.to_le_bytes());
            cd.extend_from_slice(name_b);
        }
        let cd_off = buf.len() as u32;
        buf.extend_from_slice(&cd);
        // EOCD
        buf.extend_from_slice(b"PK\x05\x06");
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(files.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(files.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(cd.len() as u32).to_le_bytes());
        buf.extend_from_slice(&cd_off.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf
    }

    /// Dựng PE32 hoặc PE32+ tối giản có bảng import thật để kiểm `pe_imports_bytes`.
    fn build_synthetic_pe(pe32_plus: bool, dlls: &[&str]) -> Vec<u8> {
        let pe_off = 0x80usize;
        let opt_off = pe_off + 24;
        let dd_rel = if pe32_plus { 112 } else { 96 };
        let opt_size = dd_rel + 16 * 8;
        let sec_off = opt_off + opt_size;
        let raw_off = 0x400usize;
        let va_base = 0x1000usize;

        let mut sec_data = Vec::new();
        let desc_table_len = (dlls.len() + 1) * 20;
        sec_data.resize(desc_table_len, 0);
        for (idx, dll) in dlls.iter().enumerate() {
            let name_rva = (va_base + sec_data.len()) as u32;
            sec_data.extend_from_slice(dll.as_bytes());
            sec_data.push(0);
            let d_off = idx * 20;
            sec_data[d_off + 12..d_off + 16].copy_from_slice(&name_rva.to_le_bytes());
            sec_data[d_off + 16..d_off + 20].copy_from_slice(&1u32.to_le_bytes());
        }

        let mut pe = vec![0u8; raw_off + sec_data.len()];
        pe[0..2].copy_from_slice(b"MZ");
        pe[0x3C..0x40].copy_from_slice(&(pe_off as u32).to_le_bytes());
        pe[pe_off..pe_off + 4].copy_from_slice(b"PE\0\0");
        pe[pe_off + 6..pe_off + 8].copy_from_slice(&1u16.to_le_bytes()); // nsec=1
        pe[pe_off + 20..pe_off + 22].copy_from_slice(&(opt_size as u16).to_le_bytes());
        let magic: u16 = if pe32_plus { 0x20B } else { 0x10B };
        pe[opt_off..opt_off + 2].copy_from_slice(&magic.to_le_bytes());
        let dd_off = opt_off + dd_rel;
        pe[dd_off - 4..dd_off].copy_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes
        if !dlls.is_empty() {
            pe[dd_off + 8..dd_off + 12].copy_from_slice(&(va_base as u32).to_le_bytes());
            pe[dd_off + 12..dd_off + 16].copy_from_slice(&(desc_table_len as u32).to_le_bytes());
        }
        // Section header
        pe[sec_off..sec_off + 6].copy_from_slice(b".rdata");
        pe[sec_off + 8..sec_off + 12].copy_from_slice(&(sec_data.len() as u32).to_le_bytes());
        pe[sec_off + 12..sec_off + 16].copy_from_slice(&(va_base as u32).to_le_bytes());
        pe[sec_off + 16..sec_off + 20].copy_from_slice(&(sec_data.len() as u32).to_le_bytes());
        pe[sec_off + 20..sec_off + 24].copy_from_slice(&(raw_off as u32).to_le_bytes());
        pe[raw_off..raw_off + sec_data.len()].copy_from_slice(&sec_data);
        pe
    }

    /// ARM-W1: đọc đúng trường `Machine` của COFF header; tên kiến trúc ↔ mã.
    #[test]
    fn arm_w1_pe_machine_doc_dung_kien_truc() {
        let mut pe = build_synthetic_pe(true, &["KERNEL32.dll"]);
        let off = u32::from_le_bytes(pe[0x3C..0x40].try_into().unwrap()) as usize + 4;
        for (name, code) in [("arm64", 0xAA64u16), ("x64", 0x8664), ("x86", 0x14C)] {
            pe[off..off + 2].copy_from_slice(&code.to_le_bytes());
            assert_eq!(pe_machine(&pe).unwrap(), code);
            assert_eq!(machine_code(name), Some(code));
        }
        assert_eq!(machine_code("aarch64"), Some(0xAA64));
        assert_eq!(machine_code("mips"), None);
        assert!(pe_machine(b"not a pe").is_err());
    }

    #[test]
    fn pe_parser_pe32_pe32plus_empty_and_malformed() {
        let pe64 = build_synthetic_pe(true, &["KERNEL32.dll", "USER32.dll"]);
        assert_eq!(
            pe_imports_bytes(&pe64).unwrap(),
            vec!["KERNEL32.dll", "USER32.dll"]
        );
        let pe32 = build_synthetic_pe(false, &["VCRUNTIME140.dll"]);
        let imps = pe_imports_bytes(&pe32).unwrap();
        assert_eq!(imps, vec!["VCRUNTIME140.dll"]);
        assert!(is_crt_runtime_dll(&imps[0]));
        let empty = build_synthetic_pe(true, &[]);
        assert!(pe_imports_bytes(&empty).unwrap().is_empty());

        // File bị cắt cụt (RVA trỏ vượt EOF) phải trả Err chứ KHÔNG panic `d[noff..]`.
        let mut trunc = build_synthetic_pe(true, &["KERNEL32.dll"]);
        trunc.truncate(0x418);
        assert!(pe_imports_bytes(&trunc).is_err());

        // Section có rawsize=0 (.bss) không được ánh xạ.
        let mut bss = build_synthetic_pe(true, &["KERNEL32.dll"]);
        let sec_off = 0x80 + 24 + 112 + 16 * 8;
        bss[sec_off + 16..sec_off + 20].copy_from_slice(&0u32.to_le_bytes());
        assert!(pe_imports_bytes(&bss).is_err());

        // PE32 delay-load import dạng VA (grAttrs=0, MSVC x86 cũ): rva = va - ImageBase.
        let mut pe32_va = build_synthetic_pe(false, &[]);
        let opt_off = 0x80 + 24;
        let image_base: u32 = 0x0040_0000;
        pe32_va[opt_off + 28..opt_off + 32].copy_from_slice(&image_base.to_le_bytes());
        let va_base: u32 = 0x1000;
        let raw_off = 0x400usize;
        let mut sec = vec![0u8; 64]; // 2 descriptor * 32 bytes (1 thật + 1 kết thúc toàn 0)
        let name_rva = va_base + sec.len() as u32;
        sec.extend_from_slice(b"VCRUNTIME140.dll\0");
        // grAttrs=0 (VA), szName = image_base + name_rva
        sec[0..4].copy_from_slice(&0u32.to_le_bytes());
        sec[4..8].copy_from_slice(&(image_base + name_rva).to_le_bytes());
        let dd_off = opt_off + 96;
        // DataDirectory[13] = Delay Import Descriptor
        pe32_va[dd_off + 13 * 8..dd_off + 13 * 8 + 4].copy_from_slice(&va_base.to_le_bytes());
        pe32_va[dd_off + 13 * 8 + 4..dd_off + 13 * 8 + 8].copy_from_slice(&64u32.to_le_bytes());
        let sec_off32 = opt_off + 96 + 16 * 8;
        pe32_va[sec_off32 + 8..sec_off32 + 12].copy_from_slice(&(sec.len() as u32).to_le_bytes());
        pe32_va[sec_off32 + 16..sec_off32 + 20].copy_from_slice(&(sec.len() as u32).to_le_bytes());
        pe32_va.resize(raw_off + sec.len(), 0);
        pe32_va[raw_off..raw_off + sec.len()].copy_from_slice(&sec);
        assert_eq!(
            pe_imports_bytes(&pe32_va).unwrap(),
            vec!["VCRUNTIME140.dll"]
        );
    }

    #[test]
    fn crt_runtime_dll_tung_nhom() {
        assert!(is_crt_runtime_dll("VCRUNTIME140.dll"));
        assert!(is_crt_runtime_dll("vcruntime140_1.dll"));
        assert!(is_crt_runtime_dll("MSVCP140.DLL"));
        assert!(is_crt_runtime_dll("ucrtbase.dll"));
        assert!(is_crt_runtime_dll("api-ms-win-crt-stdio-l1-1-0.dll"));
        assert!(!is_crt_runtime_dll("USER32.dll"));
        assert!(!is_crt_runtime_dll("KERNEL32"));
        assert!(!is_crt_runtime_dll("vcruntime.dll"));
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
        png.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
        png.extend_from_slice(&310u32.to_be_bytes());
        png.extend_from_slice(&150u32.to_be_bytes());
        assert_eq!(png_size(&png), Some((310, 150)));
        // Sai chunk đầu (không phải IHDR) → None.
        let mut bad_chunk = png.clone();
        bad_chunk[12..16].copy_from_slice(b"FAKE");
        assert_eq!(png_size(&bad_chunk), None);
        assert_eq!(png_size(&png[..20]), None);
        assert_eq!(png_size(b"khong phai png"), None);
    }

    fn mini_manifest(version: &str, desc: &str) -> String {
        let xml = String::new()
            + r#"<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10" xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10" xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">
  <Identity Name="23651Linhi.TextVN" Publisher="CN=1A703CAB-3E18-4E4D-8FD8-E1D54FC67545" Version="VVV" ProcessorArchitecture="x64"/>
  <Properties>
    <DisplayName>TextVN</DisplayName>
    <PublisherDisplayName>LinhBH.CoM</PublisherDisplayName>
    <Logo>Assets\storelogo.png</Logo>
    <Description>DDD</Description>
  </Properties>
  <Applications>
    <Application Id="TextVN" Executable="TextVN.exe">
      <uap:VisualElements Square150x150Logo="Assets\s150.png" Square44x44Logo="Assets\s44.png" Wide310x150Logo="Assets\w310.png" Square71x71Logo="Assets\s71.png"/>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust"/>
  </Capabilities>
</Package>
"#;
        xml.replace("VVV", version).replace("DDD", desc)
    }

    fn fake_png(w: u32, h: u32) -> Vec<u8> {
        let mut png = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
        png.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
        png.extend_from_slice(&w.to_be_bytes());
        png.extend_from_slice(&h.to_be_bytes());
        png
    }

    fn fake_msix_zip_bytes(manifest_name: &str, manifest: &str) -> Vec<u8> {
        let p50 = fake_png(50, 50);
        let p150 = fake_png(150, 150);
        let p44 = fake_png(44, 44);
        let p310 = fake_png(310, 150);
        let p71 = fake_png(71, 71);
        build_zip(&[
            (manifest_name, manifest.as_bytes(), true),
            ("Assets/storelogo.png", &p50, false),
            ("Assets/s150.png", &p150, true),
            ("Assets/s44.png", &p44, false),
            ("Assets/w310.png", &p310, true),
            ("Assets/s71.png", &p71, false),
            ("TextVN.exe", b"MZ", false),
            ("textvn-cli.exe", b"MZ", true),
            ("textvn-tsf.dll", b"MZ", false),
            ("textvn-tsf-x86.dll", b"MZ", true),
            ("resources/textvn.ico", b"icon", false),
            ("resources/textvn_v.ico", b"icon", true),
            ("resources/textvn_e.ico", b"icon", false),
            ("data/appdb.default.json", b"{}", true),
            ("data/tables/vowels.toml", b"[a]", false),
        ])
    }

    fn fake_msix_archive(manifest: &str) -> MsixArchive {
        read_msix_bytes(&fake_msix_zip_bytes("AppxManifest.xml", manifest)).expect("zip")
    }

    #[test]
    fn verify_msix_full_happy_path_and_cli_e2e() {
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let zip = fake_msix_zip_bytes("AppxManifest.xml", &men);
        let dir = read_msix_bytes(&zip).expect("read zip");
        let fails = verify_full(&dir, Some("0.2.85"), false, "TextVN", None);
        assert!(fails.is_empty(), "kỳ vọng không lỗi, được: {fails:?}");

        // E2E qua file thật + verify_msix_cli (hỗ trợ cả --flag=value và --expect-name rỗng).
        let tmp =
            std::env::temp_dir().join(format!("textvn-verify-e2e-{}.msix", std::process::id()));
        fs::write(&tmp, &zip).expect("ghi msix");
        let res = verify_msix_cli(&[
            "--expect-version=0.2.85".into(),
            tmp.to_string_lossy().into_owned(),
            "--expect-name".into(),
            "   ".into(),
        ]);
        let _ = fs::remove_file(&tmp);
        assert!(res.is_ok(), "verify_msix_cli phải pass: {res:?}");
    }

    #[test]
    fn verify_msix_bat_loi_tung_luat() {
        // Version sai mapping
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let dir = fake_msix_archive(&men);
        let fails = verify_full(&dir, Some("0.2.99"), false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("map từ 0.2.99")));

        // AppxManifest.xml sai hoa/thường phải bị chặn (exact-case).
        let lc_dir = read_msix_bytes(&fake_msix_zip_bytes("appxmanifest.xml", &men)).expect("zip");
        let fails = verify_full(&lc_dir, None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("thiếu AppxManifest.xml")));

        // R2-95: DisplayName có khoảng trắng đầu/cuối KHÔNG được bị parse_xml `.trim()` nuốt.
        let padded = men.replace(
            "<DisplayName>TextVN</DisplayName>",
            "<DisplayName> TextVN </DisplayName>",
        );
        let fails = verify_full(&fake_msix_archive(&padded), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("DisplayName ' TextVN '")));

        // Capability sai namespace (không phải rescap) phải bị bắt.
        let wrong_ns = men.replace(
            "<rescap:Capability Name=\"runFullTrust\"/>",
            "<Capability Name=\"runFullTrust\"/>",
        );
        let fails = verify_full(&fake_msix_archive(&wrong_ns), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("rescap:Capability")));

        // Version có dấu '+' đầu ("+1.2.85.0") phải bị từ chối VÀ vẫn báo lỗi mismatch want_ver.
        let plus_ver = mini_manifest("+1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let fails = verify_full(
            &fake_msix_archive(&plus_ver),
            Some("0.2.85"),
            false,
            "TextVN",
            None,
        );
        assert!(fails.iter().any(|f| f.contains("4 số 0..65535")));
        assert!(fails.iter().any(|f| f.contains("map từ 0.2.85")));

        // thiếu dấu tiếng Việt trong Description
        let men = mini_manifest("1.2.85.0", "may go tieng viet");
        let fails = verify_full(&fake_msix_archive(&men), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("không còn dấu tiếng Việt")));

        // mojibake
        let men = mini_manifest(
            "1.2.85.0",
            "Tr\u{00C3}\u{00A9}nh g\u{00C3}\u{00B5} mojibake",
        );
        let fails = verify_full(&fake_msix_archive(&men), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("mojibake")));

        // phiên bản 4 số sai: đầu = 0
        let men = mini_manifest("0.2.85.0", "Trình gõ tiếng Việt an toàn");
        let fails = verify_full(&fake_msix_archive(&men), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("từ chối phần đầu = 0")));

        // Arch sai
        let men =
            mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn").replace("x64\"", "arm64\"");
        let fails = verify_full(&fake_msix_archive(&men), None, false, "TextVN", None);
        assert!(fails.iter().any(|f| f.contains("ProcessorArchitecture")));

        // expect-name sai
        let men = mini_manifest("1.2.85.0", "Trình gõ tiếng Việt an toàn");
        let fails = verify_full(
            &fake_msix_archive(&men),
            None,
            false,
            "TextVN",
            Some("Khac.Name"),
        );
        assert!(fails.iter().any(|f| f.contains("Khac.Name")));
    }

    #[test]
    fn verify_msix_cli_args_strict() {
        assert!(parse_verify_msix_args(&["a.msix".into(), "--expect-version".into()]).is_err());
        assert!(parse_verify_msix_args(&[
            "a.msix".into(),
            "--expect-version".into(),
            "--require-store-identity".into()
        ])
        .is_err());
        assert!(parse_verify_msix_args(&["a.msix".into(), "b.msix".into()]).is_err());
        let ok = parse_verify_msix_args(&[
            "--require-store-identity".into(),
            "--display-name=TextVN".into(),
            "a.msix".into(),
        ])
        .unwrap();
        assert_eq!(ok.msix, "a.msix");
        assert!(ok.require_identity);
        assert_eq!(ok.display_name.as_deref(), Some("TextVN"));
    }

    #[test]
    fn xml_parser_manifest_that_va_edge_cases() {
        let xml = parse_xml(&mini_manifest("1.2.85.0", "Trình gõ tiếng Việt")).expect("parse");
        let ident = xml.child_ns(NS_FOUNDATION, "Identity").expect("Identity");
        assert_eq!(ident.attr("Name"), Some("23651Linhi.TextVN"));
        assert_eq!(ident.attr("Version"), Some("1.2.85.0"));
        let props = xml
            .child_ns(NS_FOUNDATION, "Properties")
            .expect("Properties");
        assert_eq!(
            props.child_ns(NS_FOUNDATION, "DisplayName").unwrap().text,
            "TextVN"
        );
        assert_eq!(
            props.child_ns(NS_FOUNDATION, "Description").unwrap().text,
            "Trình gõ tiếng Việt"
        );
        let mut all = Vec::new();
        xml.descendants(&mut all);
        assert!(all.iter().any(|n| n.ns == NS_RESCAP
            && n.name == "Capability"
            && n.attr("Name") == Some("runFullTrust")));
        let ve = all.iter().find(|n| n.name == "VisualElements").expect("VE");
        assert_eq!(ve.attrs.len(), 4);

        // Text xen comment, CDATA, `>` trong attr, whitespace quanh `=`, numeric entities.
        let rich = parse_xml(
            r#"<root a = "1 > 0" b="&#65;&#x42;"><!-- note -->Xin <![CDATA[chào & <VN>]]></root>"#,
        )
        .expect("rich xml");
        assert_eq!(rich.attr("a"), Some("1 > 0"));
        assert_eq!(rich.attr("b"), Some("AB"));
        assert_eq!(rich.text, "Xin chào & <VN>");
    }

    #[test]
    fn xml_parser_bat_xml_sai_well_formed() {
        for bad in [
            "<a><b></b>",
            "<a></b></a>",
            "<a attr=b></a>",
            "van ban tu do",
            "<a><!-- co -- ben trong --></a>",
            "<a><!-- duoi gach ---></a>",
            "<?><a/>",
            "<a/><b/>",
            "<123/>",
            "<a:b:c/>",
            "<a x=\"1\"y=\"2\"/>",
            "<a x=\"1\" x=\"2\"/>",
            "<a>&unknown;</a>",
            "<a>bare &amp and &</a>",
            "<a></a foo=\"1\">",
            "<a xmlns:f=\"u1\" xmlns:g=\"u1\"><f:X></g:X></a>",
            "<a xmlns:p=\"\"><p:X/></a>",
            "<a>&#+65;</a>",
            "<a>&#x+41;</a>",
            "<!garbage><a/>",
        ] {
            assert!(parse_xml(bad).is_err(), "phải báo lỗi XML hỏng: {bad}");
        }
    }

    #[test]
    fn zip_tu_choi_trung_entry_khong_phan_biet_hoa_thuong() {
        let dup = build_zip(&[
            ("AppxManifest.xml", b"<Package/>", false),
            ("appxmanifest.xml", b"<Package/>", true),
        ]);
        let err = read_msix_bytes(&dup).err().expect("phải chặn entry trùng");
        assert!(err.contains("trùng entry"), "lỗi bất ngờ: {err}");
    }

    #[test]
    fn zip_chan_zip64_thieu_extra_va_zip_bomb_bounded_inflate() {
        // 1) ZIP64 sentinel 0xFFFFFFFF trong CD nhưng không có extra field 0x0001 → phải Err.
        let mut z = build_zip(&[("a.txt", b"hello", false)]);
        let cd_pos = z
            .windows(4)
            .position(|w| w == b"PK\x01\x02")
            .expect("cd header");
        z[cd_pos + 24..cd_pos + 28].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        let err = read_msix_bytes(&z)
            .err()
            .expect("phải chặn thiếu ZIP64 extra");
        assert!(err.contains("ZIP64"), "lỗi bất ngờ: {err}");

        // 2) Deflate payload lớn hơn uncomp_size khai báo trong CD → inflate_raw_bounded dừng sớm.
        let payload = vec![b'X'; 4096];
        let mut zb = build_zip(&[("bomb.bin", &payload, true)]);
        let cd_pos2 = zb
            .windows(4)
            .position(|w| w == b"PK\x01\x02")
            .expect("cd header");
        zb[cd_pos2 + 24..cd_pos2 + 28].copy_from_slice(&32u32.to_le_bytes());
        let err2 = read_msix_bytes(&zb)
            .err()
            .expect("phải chặn zip-bomb vượt uncomp_size");
        assert!(err2.contains("vượt giới hạn"), "lỗi bất ngờ: {err2}");
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
