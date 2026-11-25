// SPDX-License-Identifier: GPL-3.0-or-later
//! Công cụ Linux — bản Rust thay các đoạn Python inline trong script gói
//! Linux (E59: policy cấm Python).
//!
//! `cargo xtask check-ibus-component <component.xml> <base>` — đối chiếu
//! component IBus sinh bởi `tv_ibus_component` (scripts/test-linux-package.sh):
//! `<exec>` tách theo quy tắc shell (g_shell_parse_argv / shlex.split) phải ra
//! `<base>/lib/textvn/textvn-ibus-engine --ibus`, và `<engines>/<engine>/<setup>`
//! phải ra `<base>/bin/textvn-settings`. Đường dẫn có khoảng trắng/&/'/<>/\`#\`
//! là hồi quy của B13 (IBus tách exec bằng shell rule — sai trích dẫn = engine
//! không nạp).

use std::fs;
use std::path::Path;

use crate::win_tools::parse_xml;

/// Tách chuỗi lệnh theo quy tắc shell POSIX (đủ cho `tv_shell_quote`):
/// - từ trần tách theo khoảng trắng;
/// - `'…'`: nội dung literal tới `'` kế (nh best-effort `g_shell_parse_argv`);
/// - `"…"`: hiểu `\"` `\\` (phòng hờ — tv_shell_quote không sinh dạng này);
/// - trích dẫn chưa đóng → Err (thành phần bị hỏng là lỗi, không đoán mò).
fn sh_split(line: &str) -> Result<Vec<String>, String> {
    // FSM dung POSIX g_shell_parse_argv:
    // - ' dong ben trong KHONG thoat;
    // - \\x trong " thoat 1 ky tu; '\'\\'' idom chen dau nhay vao vung quote-don.
    let b = line.as_bytes();
    let mut words: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut have = false;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            if have {
                words.push(String::from_utf8_lossy(&cur).into_owned());
                cur.clear();
                have = false;
            }
            i += 1;
        } else if c == b'\'' {
            // 'region: literal toi ' ke.
            let mut j = i + 1;
            loop {
                if j >= b.len() {
                    return Err("trich dan don chua dong".into());
                }
                if b[j] == b'\'' {
                    break;
                }
                cur.push(b[j]);
                j += 1;
            }
            i = j + 1;
            have = true;
        } else if c == b'"' {
            // " region: \\x thoat chuyen.
            let mut j = i + 1;
            loop {
                if j >= b.len() {
                    return Err("trich dan kep chua dong".into());
                }
                match b[j] {
                    b'"' => break,
                    b'\\' if j + 1 < b.len() => {
                        let next = b[j + 1];
                        if matches!(next, b'$' | b'`' | b'"' | b'\\' | b'\n') {
                            cur.push(next);
                        } else {
                            cur.push(b'\\');
                            cur.push(next);
                        }
                        j += 2;
                    }
                    x => {
                        cur.push(x);
                        j += 1;
                    }
                }
            }
            i = j + 1;
            have = true;
        } else if c == b'\\' {
            // Ngoai trich dan: \\x = x literal (POSIX).
            if i + 1 < b.len() {
                cur.push(b[i + 1]);
                i += 2;
            } else {
                return Err("ky tu backslash cuoi dong lo lung".into());
            }
            have = true;
        } else {
            cur.push(c);
            i += 1;
            have = true;
        }
    }
    if have {
        words.push(String::from_utf8_lossy(&cur).into_owned());
    }
    Ok(words)
}

/// Tìm text của thẻ con trực tiếp `name` (như `root.find(...)` của
/// ElementTree — chỉ 1 cấp).
fn child_text(node: &crate::win_tools::Xml, name: &str) -> Option<String> {
    node.child(name).map(|n| n.text.clone())
}

fn descendant_first<'a>(
    node: &'a crate::win_tools::Xml,
    name: &str,
) -> Option<&'a crate::win_tools::Xml> {
    let mut all = Vec::new();
    node.descendants(&mut all);
    if node.name == name {
        return Some(node);
    }
    all.into_iter().find(|n| n.name == name)
}

/// CLI: `xtask check-ibus-component <component.xml> <base>`.
pub fn check_ibus_component(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err("cách dùng: cargo xtask check-ibus-component <component.xml> <base>".into());
    }
    let (xml_path, base) = (&args[0], &args[1]);
    let text =
        fs::read_to_string(xml_path).map_err(|e| format!("không đọc được {xml_path}: {e}"))?;
    let root = parse_xml(&text).map_err(|e| format!("{xml_path}: XML lỗi: {e}"))?;
    if root.name != "component" {
        return Err(format!(
            "{xml_path}: gốc là <{}>, cần <component>",
            root.name
        ));
    }
    let Some(exec) = child_text(&root, "exec") else {
        return Err(format!("{xml_path}: thiếu <exec>"));
    };
    let Some(setup_el) = descendant_first(&root, "engines")
        .and_then(|eng| eng.child("engine"))
        .and_then(|e| e.child("setup"))
    else {
        return Err(format!("{xml_path}: thiếu <engines>/<engine>/<setup>"));
    };
    let setup = setup_el.text.clone();

    let mut errs = Vec::new();
    let want_exec = [
        format!("{base}/lib/textvn/textvn-ibus-engine"),
        "--ibus".to_string(),
    ];
    let got_exec = match sh_split(&exec) {
        Ok(w) => w,
        Err(e) => return Err(format!("{xml_path}: exec không tách được: {e}")),
    };
    if got_exec != want_exec {
        errs.push(format!("exec {got_exec:?} != {want_exec:?}"));
    }
    let want_setup = [format!("{base}/bin/textvn-settings")];
    let got_setup = match sh_split(&setup) {
        Ok(w) => w,
        Err(e) => return Err(format!("{xml_path}: setup không tách được: {e}")),
    };
    if got_setup != want_setup {
        errs.push(format!("setup {got_setup:?} != {want_setup:?}"));
    }
    if errs.is_empty() {
        println!(
            "check_ibus_component: OK ({}) base={base}",
            Path::new(xml_path).display()
        );
        Ok(())
    } else {
        Err(errs.join("; "))
    }
}

/// Đọc số nguyên little-endian từ lát cắt byte có kiểm tra biên và chống tràn offset.
fn read_u16_le(buf: &[u8], off: usize) -> Option<u16> {
    let end = off.checked_add(2)?;
    let s = buf.get(off..end)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    let s = buf.get(off..end)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn read_u64_le(buf: &[u8], off: usize) -> Option<u64> {
    let end = off.checked_add(8)?;
    let s = buf.get(off..end)?;
    Some(u64::from_le_bytes([
        s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
    ]))
}

fn u64_to_usize(v: u64, label: &str) -> Result<usize, String> {
    usize::try_from(v).map_err(|_| format!("{label} vượt phạm vi usize: {v}"))
}

fn format_ver_tuple(v: (u32, u32, u32)) -> String {
    if v.2 == 0 {
        format!("{}.{}", v.0, v.1)
    } else {
        format!("{}.{}.{}", v.0, v.1, v.2)
    }
}

fn read_cstr(strtab: &[u8], off: usize) -> Option<&str> {
    let rest = strtab.get(off..)?;
    let nul = rest.iter().position(|&b| b == 0)?;
    std::str::from_utf8(&rest[..nul]).ok()
}

/// Phân tích chuỗi phiên bản `major.minor[.patch]` thành `(major, minor, patch)`.
fn parse_ver_tuple(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = match parts.next() {
        Some(p) => p.parse::<u32>().ok()?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ElfAbiInfo {
    glibc_versions: Vec<(u32, u32, u32)>,
    glibcxx_versions: Vec<(u32, u32, u32)>,
    has_glibc_private: bool,
    has_glibcxx_private: bool,
    has_gnu_stack: bool,
    exec_stack: bool,
    is_dynamic: bool,
}

/// Kiểm tra ELF64 little-endian (thuần Rust, không cần `readelf` ngoài):
/// 1. Quét `SHT_GNU_verneed` (0x6ffffffe) lấy mọi yêu cầu `GLIBC_X.Y[.Z]` (và chặn `GLIBC_PRIVATE`).
/// 2. Quét `PT_GNU_STACK` (0x6474e551) đảm bảo có header NX-stack và KHÔNG có cờ PF_X (SELinux RHEL 8).
fn inspect_elf64_abi(buf: &[u8]) -> Result<ElfAbiInfo, String> {
    if buf.len() < 64 || &buf[0..4] != b"\x7fELF" {
        return Err("không phải file ELF hợp lệ".into());
    }
    if buf[4] != 2 || buf[5] != 1 {
        return Err(format!(
            "chỉ hỗ trợ ELF64 little-endian (class={}, data={})",
            buf[4], buf[5]
        ));
    }

    let mut info = ElfAbiInfo::default();

    // Program headers -> PT_DYNAMIC (2) và PT_GNU_STACK (0x6474e551, PF_X = 0x1).
    let e_phoff = u64_to_usize(read_u64_le(buf, 32).ok_or("header phoff thiếu")?, "e_phoff")?;
    let e_phentsize = read_u16_le(buf, 54).ok_or("header phentsize thiếu")? as usize;
    let e_phnum = read_u16_le(buf, 56).ok_or("header phnum thiếu")? as usize;
    if e_phnum > 0 {
        if e_phoff == 0 || e_phentsize < 56 {
            return Err("program header table không hợp lệ".into());
        }
        for i in 0..e_phnum {
            let off = i
                .checked_mul(e_phentsize)
                .and_then(|d| e_phoff.checked_add(d))
                .ok_or("tràn offset program header")?;
            let p_type = read_u32_le(buf, off).ok_or("program header bị cắt")?;
            let p_flags = read_u32_le(buf, off.checked_add(4).ok_or("tràn offset p_flags")?)
                .ok_or("program header flags bị cắt")?;
            if p_type == 2 {
                info.is_dynamic = true;
            } else if p_type == 0x6474_e551 {
                info.has_gnu_stack = true;
                if (p_flags & 0x1) != 0 {
                    info.exec_stack = true;
                }
            }
        }
    }

    // Section headers -> SHT_GNU_verneed (0x6ffffffe).
    let e_shoff = u64_to_usize(read_u64_le(buf, 40).ok_or("header shoff thiếu")?, "e_shoff")?;
    let e_shentsize = read_u16_le(buf, 58).ok_or("header shentsize thiếu")? as usize;
    let mut e_shnum = read_u16_le(buf, 60).ok_or("header shnum thiếu")? as usize;
    if e_shoff == 0 {
        if info.is_dynamic {
            return Err(
                "ELF động bị cắt bỏ section header — không thể xác minh phiên bản GLIBC".into(),
            );
        }
        return Ok(info);
    }
    if e_shentsize < 64 {
        return Err("section header entry size < 64".into());
    }
    if e_shnum == 0 {
        // ELF64 extended numbering: số section nằm ở sh_size của section 0.
        let ext = read_u64_le(buf, e_shoff.checked_add(32).ok_or("tràn offset sh0")?)
            .ok_or("section 0 bị cắt")?;
        e_shnum = u64_to_usize(ext, "sh0.sh_size")?;
    }

    let sh_at = |idx: usize| -> Result<(u32, usize, usize, usize), String> {
        if idx >= e_shnum {
            return Err(format!("section index {idx} vượt quá shnum={e_shnum}"));
        }
        let off = idx
            .checked_mul(e_shentsize)
            .and_then(|d| e_shoff.checked_add(d))
            .ok_or("tràn offset section header")?;
        let sh_type = read_u32_le(buf, off.checked_add(4).ok_or("tràn sh_type")?)
            .ok_or("section header bị cắt")?;
        let sh_offset = u64_to_usize(
            read_u64_le(buf, off.checked_add(24).ok_or("tràn sh_offset")?)
                .ok_or("sh_offset thiếu")?,
            "sh_offset",
        )?;
        let sh_size = u64_to_usize(
            read_u64_le(buf, off.checked_add(32).ok_or("tràn sh_size")?).ok_or("sh_size thiếu")?,
            "sh_size",
        )?;
        let sh_link = read_u32_le(buf, off.checked_add(40).ok_or("tràn sh_link")?)
            .ok_or("sh_link thiếu")? as usize;
        Ok((sh_type, sh_offset, sh_size, sh_link))
    };

    for i in 0..e_shnum {
        let (sh_type, vn_off, vn_size, sh_link) = sh_at(i)?;
        if sh_type != 0x6fff_fffe || vn_size == 0 {
            continue;
        }
        let (_, str_off, str_size, _) = sh_at(sh_link)?;
        let str_end = str_off.checked_add(str_size).ok_or("tràn bảng .dynstr")?;
        let strtab = buf.get(str_off..str_end).ok_or("bảng .dynstr bị cắt")?;
        let vn_end = vn_off
            .checked_add(vn_size)
            .ok_or("tràn section SHT_GNU_verneed")?;
        let verneed = buf
            .get(vn_off..vn_end)
            .ok_or("section SHT_GNU_verneed bị cắt")?;

        let mut pos = 0usize;
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 4096 {
                return Err("section SHT_GNU_verneed quá lớn hoặc lặp vòng".into());
            }
            let entry_end = pos.checked_add(16).ok_or("tràn offset Elf64_Verneed")?;
            if entry_end > verneed.len() {
                return Err("Elf64_Verneed bị cắt".into());
            }
            let vn_cnt = read_u16_le(verneed, pos + 2).ok_or("vn_cnt thiếu")? as usize;
            let vn_aux = read_u32_le(verneed, pos + 8).ok_or("vn_aux thiếu")? as usize;
            let vn_next = read_u32_le(verneed, pos + 12).ok_or("vn_next thiếu")? as usize;

            let mut aux_pos = pos.checked_add(vn_aux).ok_or("tràn offset Elf64_Vernaux")?;
            for _ in 0..vn_cnt {
                let aux_end = aux_pos
                    .checked_add(16)
                    .ok_or("tràn kết thúc Elf64_Vernaux")?;
                if aux_end > verneed.len() {
                    return Err("Elf64_Vernaux bị cắt".into());
                }
                let vna_name = read_u32_le(verneed, aux_pos + 8).ok_or("vna_name thiếu")? as usize;
                let vna_next = read_u32_le(verneed, aux_pos + 12).ok_or("vna_next thiếu")? as usize;
                if let Some(name) = read_cstr(strtab, vna_name) {
                    if name == "GLIBC_PRIVATE" {
                        info.has_glibc_private = true;
                    } else if name == "GLIBCXX_PRIVATE" || name == "CXXABI_PRIVATE" {
                        info.has_glibcxx_private = true;
                    } else if let Some(rest) = name.strip_prefix("GLIBC_") {
                        if let Some(v) = parse_ver_tuple(rest) {
                            info.glibc_versions.push(v);
                        }
                    } else if let Some(rest) = name.strip_prefix("GLIBCXX_") {
                        if let Some(v) = parse_ver_tuple(rest) {
                            info.glibcxx_versions.push(v);
                        }
                    }
                }
                if vna_next == 0 {
                    break;
                }
                aux_pos = aux_pos
                    .checked_add(vna_next)
                    .ok_or("tràn bước nhảy vna_next")?;
            }

            if vn_next == 0 {
                break;
            }
            pos = pos.checked_add(vn_next).ok_or("tràn bước nhảy vn_next")?;
        }
    }

    info.glibc_versions.sort_unstable();
    info.glibc_versions.dedup();
    info.glibcxx_versions.sort_unstable();
    info.glibcxx_versions.dedup();
    Ok(info)
}

/// CLI: `cargo xtask check-elf-abi [--max-glibc <X.Y>] [--max-glibcxx <X.Y.Z>] <elf>...`
/// Gate E67/E68: bảo đảm binary Linux phát hành tương thích đúng trần glibc/GLIBCXX
/// (EL8 glibc 2.28; compat GTK4/Fcitx5 glibc 2.35 + GLIBCXX 3.4.30) và có
/// `PT_GNU_STACK` không thực thi (SELinux `execstack`).
pub fn check_elf_abi(args: &[String]) -> Result<(), String> {
    let mut max_glibc = (2u32, 28u32, 0u32);
    let mut max_glibcxx: Option<(u32, u32, u32)> = None;
    let mut files: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--max-glibc" {
            let val = args
                .get(i + 1)
                .ok_or("--max-glibc cần giá trị kiểu X.Y (ví dụ 2.28)")?;
            max_glibc = parse_ver_tuple(val)
                .ok_or_else(|| format!("phiên bản glibc không hợp lệ: {val}"))?;
            i += 2;
        } else if args[i] == "--max-glibcxx" {
            let val = args
                .get(i + 1)
                .ok_or("--max-glibcxx cần giá trị kiểu X.Y.Z (ví dụ 3.4.30)")?;
            max_glibcxx = Some(
                parse_ver_tuple(val)
                    .ok_or_else(|| format!("phiên bản GLIBCXX không hợp lệ: {val}"))?,
            );
            i += 2;
        } else {
            files.push(&args[i]);
            i += 1;
        }
    }
    if files.is_empty() {
        return Err(
            "cách dùng: cargo xtask check-elf-abi [--max-glibc 2.28] [--max-glibcxx 3.4.30] <elf>..."
                .into(),
        );
    }

    let mut errs = Vec::new();
    for path in files {
        let data = fs::read(path).map_err(|e| format!("không đọc được {path}: {e}"))?;
        let info = inspect_elf64_abi(&data).map_err(|e| format!("{path}: {e}"))?;
        let mut bad: Vec<String> = Vec::new();
        for &v in &info.glibc_versions {
            if v > max_glibc {
                bad.push(format!("GLIBC_{}", format_ver_tuple(v)));
            }
        }
        if info.has_glibc_private {
            bad.push("GLIBC_PRIVATE".into());
        }
        if info.has_glibcxx_private {
            bad.push("GLIBCXX_PRIVATE".into());
        }
        if let Some(cap) = max_glibcxx {
            for &v in &info.glibcxx_versions {
                if v > cap {
                    bad.push(format!("GLIBCXX_{}", format_ver_tuple(v)));
                }
            }
        }
        if !bad.is_empty() {
            let cap_desc = match max_glibcxx {
                Some(cap) => format!(
                    "GLIBC_{}/GLIBCXX_{}",
                    format_ver_tuple(max_glibc),
                    format_ver_tuple(cap)
                ),
                None => format!("GLIBC_{}", format_ver_tuple(max_glibc)),
            };
            errs.push(format!(
                "{path}: yêu cầu symbol vượt trần {cap_desc}: {}",
                bad.join(", ")
            ));
        }
        if !info.has_gnu_stack {
            errs.push(format!(
                "{path}: thiếu PT_GNU_STACK (mặc định executable stack — bị SELinux RHEL 8/9/Fedora chặn)"
            ));
        } else if info.exec_stack {
            errs.push(format!(
                "{path}: PT_GNU_STACK có cờ PF_X (executable stack — bị SELinux RHEL 8/9/Fedora chặn)"
            ));
        }
        if bad.is_empty() && info.has_gnu_stack && !info.exec_stack {
            let top_c = info
                .glibc_versions
                .last()
                .map(|&v| format_ver_tuple(v))
                .unwrap_or_else(|| "static".into());
            let top_cxx = info
                .glibcxx_versions
                .last()
                .map(|&v| format!(", max GLIBCXX={}", format_ver_tuple(v)))
                .unwrap_or_default();
            println!("check-elf-abi: OK {path} (max GLIBC={top_c}{top_cxx}, nx-stack=OK)");
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sh_split_du_cac_dang_trich_dan() {
        assert_eq!(sh_split("a b c").unwrap(), vec!["a", "b", "c"]);
        assert_eq!(
            sh_split("'/Tải về/x' --ibus").unwrap(),
            vec!["/Tải về/x", "--ibus"]
        );
        // best-effort g_shell_parse_argv: 'xxx'"yyy" dán nhau thành 1 từ.
        assert_eq!(sh_split("'a b'\"c d\"").unwrap(), vec!["a bc d"]);
        assert_eq!(sh_split("'/it'\\''s/'").unwrap(), vec!["/it's/"]);
        assert!(sh_split("'chas chưa đóng").is_err());
        assert!(sh_split("dógšt\"").is_err());
        // backtick # $ & | giữ nguyên (không shell-eval — chỉ tách).
        assert_eq!(
            sh_split("'$HOME/`x`#y'").unwrap(),
            vec![std::concat!("$HOME/`x`#y")]
        );
    }

    #[test]
    fn component_that_va_sai_exec() {
        let xml = r#"<component>
  <name>textvn</name>
  <exec>/<base>/lib/textvn/textvn-ibus-engine --ibus</exec>
  <engines><engine><name>textvn</name><setup>/<base>/bin/textvn-settings</setup></engine></engines>
</component>"#
            .replace("/<base>", "/opt/base");
        let dir = std::env::temp_dir().join(format!("textvn-ibus-comp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let f = dir.join("c.xml");
        std::fs::write(&f, &xml).expect("ghi");
        // đúng
        assert!(
            check_ibus_component(&[f.to_string_lossy().to_string(), "/opt/base".into()]).is_ok()
        );
        // sai base → phải fail
        assert!(check_ibus_component(&[f.to_string_lossy().to_string(), "/khac".into()]).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn component_duong_dan_dac_biet() {
        // đúng chu hồi quy B13: khoảng trắng, &, ', <>, \, `# trong đường dẫn.
        for base in [
            "/t/Tải về/TextVN-1",
            "/t/A & B",
            "/t/it's <x> \"q\"",
            "/t/a|b\\c",
            "/t/$HOME/`x`#y",
        ] {
            let q = |s: &str| {
                let safe = !s.is_empty()
                    && s.bytes().all(|b| {
                        b.is_ascii_alphanumeric()
                            || matches!(
                                b,
                                b'_' | b'.' | b'/' | b'+' | b':' | b'@' | b'%' | b',' | b'=' | b'-'
                            )
                    });
                if safe {
                    s.to_string()
                } else {
                    format!("'{}'", s.replace('\'', "'\\''"))
                }
            };
            let exec = format!(
                "{} --ibus",
                q(&format!("{base}/lib/textvn/textvn-ibus-engine"))
            )
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
            let setup = q(&format!("{base}/bin/textvn-settings"))
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            let xml = format!(
                "<component><exec>{exec}</exec><engines><engine><setup>{setup}</setup></engine></engines></component>"
            );
            let dir =
                std::env::temp_dir().join(format!("textvn-ibus-comp2-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("mkdir");
            let f = dir.join("c.xml");
            std::fs::write(&f, &xml).expect("ghi");
            let r = check_ibus_component(&[f.to_string_lossy().to_string(), base.to_string()]);
            let _ = std::fs::remove_dir_all(&dir);
            assert!(r.is_ok(), "base {base:?} phải pass: {r:?}");
        }
    }

    /// Dựng một ảnh ELF64 tối giản trong bộ nhớ với `PT_GNU_STACK` + `SHT_GNU_verneed`
    /// để kiểm tra `inspect_elf64_abi` và `check_elf_abi` trên mọi hệ điều hành.
    fn synthetic_elf64(glibc_tags: &[&str], exec_stack: bool) -> Vec<u8> {
        let mut buf = vec![0u8; 64];
        buf[0..4].copy_from_slice(b"\x7fELF");
        buf[4] = 2; // ELFCLASS64
        buf[5] = 1; // ELFDATA2LSB
        buf[6] = 1; // EV_CURRENT

        // Program header: 1 entry PT_GNU_STACK (56 bytes) ngay sau ELF header.
        let phoff = buf.len() as u64;
        buf[32..40].copy_from_slice(&phoff.to_le_bytes());
        buf[54..56].copy_from_slice(&56u16.to_le_bytes());
        buf[56..58].copy_from_slice(&1u16.to_le_bytes());
        let mut ph = [0u8; 56];
        ph[0..4].copy_from_slice(&0x6474_e551u32.to_le_bytes()); // PT_GNU_STACK
        let p_flags: u32 = if exec_stack { 0x7 } else { 0x6 }; // RWX vs RW
        ph[4..8].copy_from_slice(&p_flags.to_le_bytes());
        buf.extend_from_slice(&ph);

        // .dynstr (section 1)
        let str_off = buf.len();
        buf.push(0); // empty string at 0
        let lib_name_off = (buf.len() - str_off) as u32;
        buf.extend_from_slice(b"libc.so.6\0");
        let mut tag_offs = Vec::new();
        for tag in glibc_tags {
            tag_offs.push((buf.len() - str_off) as u32);
            buf.extend_from_slice(tag.as_bytes());
            buf.push(0);
        }
        let str_size = buf.len() - str_off;

        // SHT_GNU_verneed (section 2): 1 Verneed + N Vernaux (16 bytes mỗi mục)
        let vn_off = buf.len();
        let mut vn = [0u8; 16];
        vn[0..2].copy_from_slice(&1u16.to_le_bytes()); // vn_version
        vn[2..4].copy_from_slice(&(tag_offs.len() as u16).to_le_bytes()); // vn_cnt
        vn[4..8].copy_from_slice(&lib_name_off.to_le_bytes()); // vn_file
        vn[8..12].copy_from_slice(&16u32.to_le_bytes()); // vn_aux
        vn[12..16].copy_from_slice(&0u32.to_le_bytes()); // vn_next
        buf.extend_from_slice(&vn);
        for (idx, &toff) in tag_offs.iter().enumerate() {
            let mut va = [0u8; 16];
            va[8..12].copy_from_slice(&toff.to_le_bytes()); // vna_name
            let next: u32 = if idx + 1 < tag_offs.len() { 16 } else { 0 };
            va[12..16].copy_from_slice(&next.to_le_bytes()); // vna_next
            buf.extend_from_slice(&va);
        }
        let vn_size = buf.len() - vn_off;

        // Section headers: 0=NULL, 1=.dynstr, 2=.gnu.version_r
        let shoff = buf.len() as u64;
        buf[40..48].copy_from_slice(&shoff.to_le_bytes());
        buf[58..60].copy_from_slice(&64u16.to_le_bytes());
        buf[60..62].copy_from_slice(&3u16.to_le_bytes());
        buf.extend_from_slice(&[0u8; 64]); // section 0: SHT_NULL
        let mut sh1 = [0u8; 64];
        sh1[4..8].copy_from_slice(&3u32.to_le_bytes()); // SHT_STRTAB
        sh1[24..32].copy_from_slice(&(str_off as u64).to_le_bytes());
        sh1[32..40].copy_from_slice(&(str_size as u64).to_le_bytes());
        buf.extend_from_slice(&sh1);
        let mut sh2 = [0u8; 64];
        sh2[4..8].copy_from_slice(&0x6fff_fffeu32.to_le_bytes()); // SHT_GNU_verneed
        sh2[24..32].copy_from_slice(&(vn_off as u64).to_le_bytes());
        sh2[32..40].copy_from_slice(&(vn_size as u64).to_le_bytes());
        sh2[40..44].copy_from_slice(&1u32.to_le_bytes()); // sh_link = 1 (.dynstr)
        buf.extend_from_slice(&sh2);
        buf
    }

    #[test]
    fn e67_check_elf_abi_accepts_glibc_228_and_blocks_newer_or_execstack() {
        let dir = std::env::temp_dir().join(format!("textvn-elf-abi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let ok_bin = dir.join("ok_el8");
        let bad_glibc = dir.join("bad_u24");
        let bad_stack = dir.join("bad_stack");
        std::fs::write(
            &ok_bin,
            synthetic_elf64(&["GLIBC_2.2.5", "GLIBC_2.17", "GLIBC_2.28"], false),
        )
        .unwrap();
        std::fs::write(
            &bad_glibc,
            synthetic_elf64(&["GLIBC_2.2.5", "GLIBC_2.28", "GLIBC_2.34"], false),
        )
        .unwrap();
        std::fs::write(&bad_stack, synthetic_elf64(&["GLIBC_2.28"], true)).unwrap();

        assert!(check_elf_abi(&[
            "--max-glibc".into(),
            "2.28".into(),
            ok_bin.to_string_lossy().into_owned(),
        ])
        .is_ok());
        let err_glibc = check_elf_abi(&[
            "--max-glibc".into(),
            "2.28".into(),
            bad_glibc.to_string_lossy().into_owned(),
        ])
        .unwrap_err();
        assert!(err_glibc.contains("GLIBC_2.34"), "{err_glibc}");
        let err_stack = check_elf_abi(&[
            "--max-glibc".into(),
            "2.28".into(),
            bad_stack.to_string_lossy().into_owned(),
        ])
        .unwrap_err();
        assert!(err_stack.contains("PT_GNU_STACK"), "{err_stack}");

        let bad_priv = dir.join("bad_priv");
        std::fs::write(
            &bad_priv,
            synthetic_elf64(&["GLIBC_2.28", "GLIBC_PRIVATE"], false),
        )
        .unwrap();
        let err_priv = check_elf_abi(&[
            "--max-glibc".into(),
            "2.28".into(),
            bad_priv.to_string_lossy().into_owned(),
        ])
        .unwrap_err();
        assert!(err_priv.contains("GLIBC_PRIVATE"), "{err_priv}");

        let ok_cxx = dir.join("ok_cxx.so");
        let bad_cxx = dir.join("bad_cxx.so");
        std::fs::write(
            &ok_cxx,
            synthetic_elf64(&["GLIBC_2.34", "GLIBCXX_3.4.29", "GLIBCXX_3.4.30"], false),
        )
        .unwrap();
        std::fs::write(
            &bad_cxx,
            synthetic_elf64(&["GLIBC_2.35", "GLIBCXX_3.4.32"], false),
        )
        .unwrap();
        assert!(check_elf_abi(&[
            "--max-glibc".into(),
            "2.35".into(),
            "--max-glibcxx".into(),
            "3.4.30".into(),
            ok_cxx.to_string_lossy().into_owned(),
        ])
        .is_ok());
        let err_cxx = check_elf_abi(&[
            "--max-glibc".into(),
            "2.35".into(),
            "--max-glibcxx".into(),
            "3.4.30".into(),
            bad_cxx.to_string_lossy().into_owned(),
        ])
        .unwrap_err();
        assert!(err_cxx.contains("GLIBCXX_3.4.32"), "{err_cxx}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
