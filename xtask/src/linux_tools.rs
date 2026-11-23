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
}
