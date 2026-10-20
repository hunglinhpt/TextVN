// SPDX-License-Identifier: GPL-3.0-or-later
//! L4 fuzz - engine kiem tra cau tieng Viet `textvn-assist` (P4-1 · P4-0 §9).
//!
//! Input la **byte bat ky** (ke ca UTF-8 hong, ky tu dieu khong hop le) →
//! mo rong thanh chuoi hop le roi chay toan bo pipeline:
//! 1. `check` / `check_sentence` **khong panic** (ke ca L0 bo qua cau).
//! 2. `apply_auto_fixes` **khong panic** va **khong lam hong** van ban:
//!    - ket qua phai la chuoi hop le,
//!    - do dai ket qua khong vuot qua do dai goc (chi xoa/chuyen, khong them),
//!    - chay 2 lan cho cung dau vao → **idempotent** (fix khong lam loi moi).
//! 3. Moi issue phai co `span` **trong** pham vi chuoi va `fix` (neu co) phai
//!    hop le (khong lan ra ngoai chuoi).
//!
//! Khong log, khong ghi file (S2).

#![no_main]

use libfuzzer_sys::fuzz_target;
use textvn_assist::{apply_auto_fixes, check, check_sentence, Issue};

/// Chuyen byte -> chuoi co the dung (thay the moi byte lap lai) de luon vao
/// duoc pipeline ma khong phai bo input.
fn lossy_text(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

/// Kiem tra moi issue: span hop le, fix (neu co) nam trong chuoi.
fn assert_issues_valid(text: &str, issues: &[Issue]) {
    let len = text.chars().count();
    for issue in issues {
        assert!(
            issue.span.start <= issue.span.end,
            "span bat nguoc: {:?}",
            issue.span
        );
        assert!(
            issue.span.end <= len,
            "span vuot qua do dai chuoi: {:?} (len {})",
            issue.span,
            len
        );
        if let Some(fix) = &issue.fix {
            match fix {
                textvn_assist::Fix::Delete { start, end } => {
                    assert!(*start <= *end && *end <= len, "fix delete ngoai chuoi");
                }
                textvn_assist::Fix::Insert { at, .. } => {
                    assert!(*at <= len, "fix insert ngoai chuoi");
                }
                textvn_assist::Fix::Replace { start, end, .. } => {
                    assert!(*start <= *end && *end <= len, "fix replace ngoai chuoi");
                }
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let text = lossy_text(data);

    let issues = check(&text);
    assert_issues_valid(&text, &issues);

    let sentence_issues = check_sentence(&text);
    assert_issues_valid(&text, &sentence_issues);

    // Fix khong duoc lam text dai hon ban dau (chi xoa / thay the).
    let fixed = apply_auto_fixes(&text, &issues);
    assert!(
        fixed.chars().count() <= text.chars().count(),
        "fix lam text dai ra: {} -> {}",
        text.chars().count(),
        fixed.chars().count()
    );

    // Idempotent: fix 2 lan cho cung dau vao phai cho cung ket qua.
    let fixed2 = apply_auto_fixes(&fixed, &check(&fixed));
    assert_eq!(fixed, fixed2, "fix khong idempotent");

    // Issue cua phien fix moi phai hop le tren chuoi da fix.
    assert_issues_valid(&fixed, &check(&fixed));
});
