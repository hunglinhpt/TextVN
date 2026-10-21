// SPDX-License-Identifier: GPL-3.0-or-later
//! Test tich hop: doc corpus `corpus/assist/*.ass` va so khop loi mong doi.
//!
//! Dinh dang corpus (P4-1, Fassist-43): TSV - `<cau>\t<kind1,kind2,...>`, cot
//! kinds rong = cau sach. Cau kiem tra nhu **fragment** (`check`).
//!
//! Ky vong co the kem **span**: `kind@start-end` (span = ky tu Unicode) - khi
//! co span, test doi chinh xac vi tri (Fassist-93).
//!
//! Chay: `cargo test -p textvn-assist` (CI 3 OS).

use std::fs;
use std::path::{Path, PathBuf};

use textvn_assist::{check, IssueKind};

/// Mot ky vong: loai loi + (tuy chon) span mong doi.
type Expect = (String, Option<(u32, u32)>);

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

/// Parse 1 file corpus thanh danh sach `(cau, ky vong)`.
fn parse_corpus(path: &Path) -> Vec<(String, Vec<Expect>)> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("doc {}: {e}", path.display()));
    let mut cases = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (sentence, expected) = line.split_once('\t').unwrap_or((line, ""));
        let kinds: Vec<Expect> = expected
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| match s.split_once('@') {
                Some((kind, span)) => {
                    let parts: Vec<&str> = span.split('-').collect();
                    let sp = match (parts.first(), parts.get(1)) {
                        (Some(a), Some(b)) => match (a.parse::<u32>(), b.parse::<u32>()) {
                            (Ok(x), Ok(y)) => Some((x, y)),
                            _ => None,
                        },
                        _ => None,
                    };
                    (kind.to_string(), sp)
                }
                None => (s.to_string(), None),
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
            let issues = check(&sentence);
            let got_kinds: Vec<String> =
                issues.iter().map(|i| i.kind.as_str().to_string()).collect();
            let want_kinds: Vec<String> = expected.iter().map(|(k, _)| k.clone()).collect();
            assert_eq!(
                got_kinds,
                want_kinds,
                "{} :: cau `{}`",
                file.display(),
                sentence
            );
            // Span (neu corpus chi dinh ky vong) phai khop chinh xac (Fassist-93).
            for (kind, span) in &expected {
                if let Some((s, e)) = span {
                    let matched = issues.iter().any(|i| {
                        i.kind.as_str() == kind
                            && i.span.start as u32 == *s
                            && i.span.end as u32 == *e
                    });
                    assert!(
                        matched,
                        "{} :: cau `{}` -> span cua `{kind}` phai la {s}..{e}",
                        file.display(),
                        sentence
                    );
                }
            }
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
            kinds.extend(expected.into_iter().map(|(k, _)| k));
        }
    }
    // V1 phai phu 6 loai loi (L1/L2/L11 gop 1 loai) + L6 + L8 + L9/L10/L12.
    for required in [
        "duplicate_word",
        "space_before_punct",
        "space_after_punct",
        "double_space",
        "lowercase_sentence_start",
        "diacritic_typo",
        "sentence_repeat",
        "double_negative",
        "comma_at_end",
    ] {
        assert!(kinds.contains(required), "corpus chua co loai {required}");
    }
}

#[test]
fn cau_tieng_anh_khong_bao_loi() {
    // L0: bo qua cau khong phai tieng Viet (D16).
    assert!(check("this is a test.").is_empty());
    assert!(check("I am going to the office.").is_empty());
}

#[test]
fn kind_id_khop_voi_enum() {
    // Id chuoi dung trong corpus phai khop `IssueKind::as_str` (tranh loi ghi).
    for kind in [
        IssueKind::DuplicateWord,
        IssueKind::SpaceBeforePunct,
        IssueKind::SpaceAfterPunct,
        IssueKind::DoubleSpace,
        IssueKind::LowercaseSentenceStart,
        IssueKind::DiacriticTypo,
        IssueKind::SentenceRepeat,
        IssueKind::DoubleNegative,
        IssueKind::CommaAtEnd,
    ] {
        assert_eq!(
            IssueKind::from_id(kind.as_str()),
            Some(kind),
            "id cua {:?} khong doi xuyen",
            kind
        );
    }
}
