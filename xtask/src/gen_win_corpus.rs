// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask/src/gen_win_corpus.rs` — điều phối sinh corpus Windows `corpus/win/*.keys` (WIN-006).

use std::fs;
use std::path::Path;

use crate::win_corpus_cases::all_cases;

pub fn run(write: bool) -> Result<(), String> {
    let out_dir = Path::new("corpus/win");
    if write {
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("không thể tạo thư mục corpus/win: {e}"))?;
    }

    let mut count = 0;
    for c in all_cases() {
        count += 1;
        let file_path = out_dir.join(c.name);
        if write {
            fs::write(&file_path, c.content)
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

    if write {
        println!("  gen  corpus/win: đã sinh {count} case (chuẩn WIN-006)");
    } else {
        println!("  ok   corpus/win: đủ {count} case khớp chuẩn");
    }
    Ok(())
}
