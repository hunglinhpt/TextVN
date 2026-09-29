// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask/src/gen_mac_corpus.rs` — điều phối sinh corpus macOS `corpus/mac/*.keys` (MAC-006).

use std::fs;
use std::path::Path;

use crate::mac_corpus_cases::all_cases;

pub fn run(write: bool) -> Result<(), String> {
    let out_dir = Path::new("corpus/mac");
    if write {
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("không thể tạo thư mục corpus/mac: {e}"))?;
    }

    let mut count = 0;
    let mut expected: Vec<std::path::PathBuf> = Vec::new();
    for c in all_cases() {
        count += 1;
        let file_path = out_dir.join(&c.name);
        expected.push(file_path.clone());
        if write {
            fs::write(&file_path, &c.content)
                .map_err(|e| format!("ghi file `{}`: {e}", file_path.display()))?;
        } else {
            if !file_path.exists() {
                return Err(format!("thiếu case corpus `{}`", file_path.display()));
            }
            let cur = fs::read_to_string(&file_path)
                .map_err(|e| format!("đọc `{}`: {e}", file_path.display()))?;
            if cur.replace("\r\n", "\n") != c.content.replace("\r\n", "\n") {
                return Err(format!(
                    "case corpus `{}` bị lệch nội dung",
                    file_path.display()
                ));
            }
        }
    }

    // Chế độ check: file `.keys` mồ côi (đã xoá khỏi danh sách nhưng còn trên
    // disk) phải bị báo — không thì case "sống ngoài nguồn sự thật" (review R1
    // finding 12). Ở chế độ write: dọn luôn file thừa.
    let mut orphans: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(out_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("keys") && !expected.contains(&p) {
                if write {
                    fs::remove_file(&p).map_err(|e| format!("xoá `{}`: {e}", p.display()))?;
                    println!("  rm   {} (mồ côi)", p.display());
                } else {
                    orphans.push(p);
                }
            }
        }
    }
    if !orphans.is_empty() {
        let list: Vec<String> = orphans.iter().map(|p| p.display().to_string()).collect();
        return Err(format!(
            "case corpus mồ côi (không có trong mac_corpus_cases.rs): {}",
            list.join(", ")
        ));
    }

    if write {
        println!("  gen  corpus/mac: đã sinh {count} case (chuẩn MAC-006)");
    } else {
        println!("  ok   corpus/mac: đủ {count} case khớp chuẩn");
    }
    Ok(())
}
