// SPDX-License-Identifier: GPL-3.0-or-later
//! E70 — "Copy tóm tắt lỗi": dựng đoạn văn bản ngắn để dán vào GitHub Issues sau khi xuất
//! gói chẩn đoán. Thuần (test được) và **ẩn PII**: chỉ tên tệp ZIP (không thư mục), mọi
//! token tên người dùng (`USERNAME`, lá của `USERPROFILE`) bị thay `<REDACTED_USER>` không
//! phân biệt hoa/thường. Nguồn dòng phát hiện là output `doctor` (mã + thông điệp) — không
//! có nội dung gõ (G8).

/// Thay mọi lần xuất hiện `token` (ASCII không phân biệt hoa/thường; ký tự non-ASCII so
/// khớp đúng). Dùng `to_ascii_lowercase` — giữ nguyên độ dài byte TỪNG ký tự nên vị trí tìm
/// được trên bản thường luôn là ranh giới ký tự hợp lệ của `text` (audit E70 vòng 1:
/// `to_lowercase` Unicode, vd. `İ` → `i̇`, làm lệch offset → cắt giữa ký tự → panic).
fn redact_token(text: &str, token: &str) -> String {
    if token.chars().count() < 2 {
        return text.to_string();
    }
    let lower_text = text.to_ascii_lowercase();
    let lower_tok = token.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while let Some(pos) = lower_text[i..].find(&lower_tok) {
        let start = i + pos;
        out.push_str(&text[i..start]);
        out.push_str("<REDACTED_USER>");
        i = start + token.len();
    }
    out.push_str(&text[i..]);
    out
}

/// Ẩn mọi token người dùng trong `text`.
pub fn redact_user_tokens(text: &str, tokens: &[String]) -> String {
    let mut out = text.to_string();
    for t in tokens {
        out = redact_token(&out, t);
    }
    out
}

/// Token người dùng của phiên hiện tại (Windows: `USERNAME` + lá `USERPROFILE`).
pub fn current_user_tokens() -> Vec<String> {
    let mut v = Vec::new();
    if let Some(u) = std::env::var_os("USERNAME").and_then(|s| s.into_string().ok()) {
        v.push(u);
    }
    if let Some(p) = std::env::var_os("USERPROFILE") {
        if let Some(leaf) = std::path::Path::new(&p)
            .file_name()
            .and_then(|s| s.to_str())
        {
            if !v.iter().any(|x| x.eq_ignore_ascii_case(leaf)) {
                v.push(leaf.to_string());
            }
        }
    }
    v
}

/// Đoạn tóm tắt dán GitHub Issues. `zip_path` chỉ lấy tên tệp.
pub fn build_issue_summary(
    version: &str,
    zip_path: &str,
    highlights: &[String],
    user_tokens: &[String],
) -> String {
    let zip_name = zip_path
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(zip_path)
        .trim();
    let mut s = format!(
        "TextVN {version} — tóm tắt chẩn đoán (đã ẩn tên người dùng, không chứa nội dung gõ)\n\
         Tệp đính kèm: {zip_name}\n"
    );
    if highlights.is_empty() {
        s.push_str("Phát hiện: không có cảnh báo/lỗi.\n");
    } else {
        s.push_str("Phát hiện:\n");
        for h in highlights.iter().take(12) {
            s.push_str("- ");
            s.push_str(h.trim());
            s.push('\n');
        }
    }
    s.push_str("\nMô tả lỗi: \nỨng dụng đang gõ: \nKết quả mong đợi: \n");
    redact_user_tokens(&s, user_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e70_summary_keeps_only_zip_name_and_redacts_user() {
        let toks = vec!["HungL".to_string()];
        let s = build_issue_summary(
            "0.2.90",
            r"C:\Users\hungl\Desktop\textvn-diag-1795620447.zip",
            &[
                "[CẢNH BÁO] [W_CONFLICT_VI_TIP] Microsoft Vietnamese Telex".into(),
                r"→ Gợi ý: xem C:\Users\HUNGL\AppData\Local\TextVN\logs".into(),
            ],
            &toks,
        );
        assert!(s.contains("Tệp đính kèm: textvn-diag-1795620447.zip"));
        assert!(!s.to_lowercase().contains("hungl"), "{s}");
        assert!(s.contains(r"C:\Users\<REDACTED_USER>\AppData"));
        assert!(s.contains("W_CONFLICT_VI_TIP"));
    }

    #[test]
    fn e70_summary_without_findings() {
        let s = build_issue_summary("0.2.90", "/home/a/textvn-diag-1.zip", &[], &[]);
        assert!(s.contains("không có cảnh báo/lỗi"));
        assert!(s.contains("textvn-diag-1.zip"));
        assert!(!s.contains("/home/"));
    }

    #[test]
    fn e70_redact_handles_unicode_usernames() {
        let toks = vec!["Nguyễn".to_string(), "x".to_string()];
        let out = redact_user_tokens(r"C:\Users\Nguyễn\Desktop x", &toks);
        assert_eq!(out, r"C:\Users\<REDACTED_USER>\Desktop x");
        // Ký tự đổi độ dài khi lowercase Unicode (İ) không được làm panic / lệch offset.
        let toks = vec!["İlker".to_string()];
        let out = redact_user_tokens(r"İİ C:\Users\İlker\x", &toks);
        assert_eq!(out, r"İİ C:\Users\<REDACTED_USER>\x");
    }
}
