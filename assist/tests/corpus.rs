// SPDX-License-Identifier: GPL-3.0-or-later
//! Test tich hop: doc corpus `corpus/assist/*.ass` va so khop loi mong doi.
//!
//! Dinh dang corpus (P4-1, Fassist-43): TSV - `<cau>\t<kind1,kind2,...>`,
//! cot kinds rong = cau sach. Cau kiem tra nhu **fragment** (`check`).
//!
//! Chay: `cargo test -p textvn-assist` (CI 3 OS).

use std::fs;
use std::path::{Path, PathBuf};

use textvn_assist::{check, IssueKind};

/// Thu thu tat ca file `.ass` trong `corpus/assist/`.
fn corpus_files() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("corpus")
        .join("assist");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("khong doc duoc corpus dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("ass"))
        .collect();
    files.sort();
    files
}

/// Parse 1 file corpus thanh danh sach `(cau, kinds mong doi)`.
fn parse_corpus(path: &Path) -> Vec<(String, Vec<IssueKind>)> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("doc {}: {e}", path.display()));
    let mut cases = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line = line.trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (sentence, expected) = line.split_once('\t').unwrap_or((line, ""));
        let kinds: Vec<IssueKind> = expected
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                IssueKind::from_id(s).unwrap_or_else(|| {
                    panic!("kind lai `{}` tai {}:{}", s, path.display(), idx + 1)
                })
            })
            .collect();
        cases.push((sentence.to_string(), kinds));
    }
    cases
}

#[test]
fn corpus_vi_basic_passes() {
    let files = corpus_files();
    assert!(!files.is_empty(), "corpus/assist rong");
    let mut total = 0usize;
    for file in files {
        for (sentence, expected) in parse_corpus(&file) {
            let got: Vec<IssueKind> = check(&sentence).into_iter().map(|i| i.kind).collect();
            assert_eq!(got, expected, "{} :: cau `{}`", file.display(), sentence);
            total += 1;
        }
    }
    assert!(total >= 60, "corpus qua it: {total}");
}

#[test]
fn corpus_co_du_lop_hieu_biet() {
    let files = corpus_files();
    let mut kinds = std::collections::BTreeSet::new();
    for file in files {
        for (_, expected) in parse_corpus(&file) {
            kinds.extend(expected);
        }
    }
    // V1 phai phu 6 loai loi (L1/L2/L11 gop 1 loai) + L6 + L8.
    for required in [
        IssueKind::DuplicateWord,
        IssueKind::SpaceBeforePunct,
        IssueKind::SpaceAfterPunct,
        IssueKind::DoubleSpace,
        IssueKind::LowercaseSentenceStart,
        IssueKind::DiacriticTypo,
    ] {
        assert!(
            kinds.contains(&required),
            "corpus chua co loai {required:?}"
        );
    }
}

#[test]
fn cau_tieng_anh_khong_bao_loi() {
    // L0: bo qua cau khong phai tieng Viet (D16).
    assert!(check("this is a test.").is_empty());
    assert!(check("I am going to the office.").is_empty());
}
