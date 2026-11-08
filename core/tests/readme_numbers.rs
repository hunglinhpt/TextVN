// SPDX-License-Identifier: GPL-3.0-or-later
//! E40 — số liệu corpora trong README (VI và EN) phải khớp số dòng thật của `data/`.
//! Trước đây tổng `41.313` đứng cạnh các thành phần cộng ra 37.967 và README ghi "từ điển
//! 3000 từ" trong khi file có 3.346 dòng.

use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn count(file: &str) -> usize {
    std::fs::read_to_string(root().join("data").join(file))
        .unwrap_or_else(|e| panic!("{file}: {e}"))
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .count()
}

fn dot(n: usize) -> String {
    group(n, '.')
}

fn comma(n: usize) -> String {
    group(n, ',')
}

fn group(n: usize, sep: char) -> String {
    // Không dùng `%`/`is_multiple_of`: clippy mới đòi cái sau, rustc Debian trixie chưa có (R2-120h).
    let mut out: Vec<char> = Vec::new();
    let mut run = 0;
    for c in n.to_string().chars().rev() {
        if run == 3 {
            out.push(sep);
            run = 0;
        }
        out.push(c);
        run += 1;
    }
    out.iter().rev().collect()
}

#[test]
fn readme_corpora_numbers_match_data_files() {
    let readme = std::fs::read_to_string(root().join("README.md")).unwrap();
    let parts = [
        count("vn_syllables.txt"),
        count("vn_compounds.txt"),
        count("en_freq3000.txt"),
        count("en_words10k.txt"),
        count("en_compounds.txt"),
        count("vn_phrases.tsv"),
        count("en_phrases.tsv"),
    ];
    let total: usize = parts.iter().sum();
    assert!(
        readme.contains(&format!("{} mục", dot(total))),
        "README (VI) phải ghi tổng {} mục",
        dot(total)
    );
    assert!(
        readme.contains(&format!("{}-entry", comma(total))),
        "README (EN) phải ghi tổng {}-entry",
        comma(total)
    );
    for n in parts {
        assert!(readme.contains(&dot(n)), "README (VI) thiếu số {}", dot(n));
        assert!(
            readme.contains(&comma(n)),
            "README (EN) thiếu số {}",
            comma(n)
        );
    }
    let phrases = parts[5] + parts[6];
    assert!(
        readme.contains(&format!("{} cặp", dot(phrases))),
        "tổng cặp câu (VI)"
    );
    assert!(
        readme.contains(&format!("{} context→phrase pairs", comma(phrases))),
        "tổng cặp câu (EN)"
    );
}

#[test]
fn readme_has_no_stale_dictionary_claims() {
    let readme = std::fs::read_to_string(root().join("README.md")).unwrap();
    // Phần "Lịch sử phiên bản" ghi số của từng bản đã phát hành — chỉ kiểm phần mô tả hiện tại.
    let current: String = readme
        .lines()
        .filter(|l| !l.starts_with("| [0.2."))
        .collect::<Vec<_>>()
        .join("\n");
    for stale in [
        "từ điển 3000 từ",
        "Từ điển EN 3000 từ",
        "3000 từ tần suất",
        "3000 most frequent",
        "3000-word English",
        "3000 frequency words",
        "tiếng Anh luôn bật",
        "English always on",
        "21% savings",
        "~21%",
    ] {
        assert!(
            !current.contains(stale),
            "README còn câu lỗi thời: `{stale}`"
        );
    }
}
