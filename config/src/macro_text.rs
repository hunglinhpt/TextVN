// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng gõ tắt dạng text cho bảng cài đặt (Windows + Linux dùng chung một định dạng).
//!
//! Mỗi dòng một mục `gõ tắt = nội dung` (kiểu file macro của UniKey/OpenKey, nhưng
//! sửa được ngay trong ô văn bản). Dòng trống và dòng bắt đầu bằng `#` bị bỏ qua.
//! Trong nội dung, `\n` là xuống dòng và `\\` là một dấu `\`.
//!
//! Kiểm tra ở đây để người dùng thấy lỗi **khi bấm Lưu** thay vì macro âm thầm không chạy:
//! - trigger không rỗng, không chứa khoảng trắng (engine khớp đuôi text trước con trỏ);
//! - nội dung ≤ 64 ký tự — giới hạn `insert` của `ime_result_v1` (P0-2 §2), dài hơn sẽ
//!   bị engine cắt;
//! - trigger không trùng (không phân biệt hoa/thường — engine khớp như vậy).

use crate::{MacroEntry, MacroWhen};

/// Độ dài tối đa của trigger (ký tự) — engine nhớ 64 ký tự cuối, 32 là dư cho mọi trigger thật.
pub const TRIGGER_MAX: usize = 32;
/// Độ dài tối đa của nội dung (ký tự) — `MAX_TEXT` của engine.
pub const EXPAND_MAX: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroLineErrorKind {
    MissingEquals,
    EmptyTrigger,
    TriggerHasSpace,
    TriggerTooLong,
    EmptyExpansion,
    ExpansionTooLong,
    DuplicateTrigger,
}

impl MacroLineErrorKind {
    pub const ALL: [MacroLineErrorKind; 7] = [
        MacroLineErrorKind::MissingEquals,
        MacroLineErrorKind::EmptyTrigger,
        MacroLineErrorKind::TriggerHasSpace,
        MacroLineErrorKind::TriggerTooLong,
        MacroLineErrorKind::EmptyExpansion,
        MacroLineErrorKind::ExpansionTooLong,
        MacroLineErrorKind::DuplicateTrigger,
    ];

    /// Mã ổn định cho C ABI (`ime_settings_set_macros_text`).
    pub fn code(self) -> i32 {
        match self {
            MacroLineErrorKind::MissingEquals => 1,
            MacroLineErrorKind::EmptyTrigger => 2,
            MacroLineErrorKind::TriggerHasSpace => 3,
            MacroLineErrorKind::TriggerTooLong => 4,
            MacroLineErrorKind::EmptyExpansion => 5,
            MacroLineErrorKind::ExpansionTooLong => 6,
            MacroLineErrorKind::DuplicateTrigger => 7,
        }
    }

    pub fn message_vi(self) -> &'static str {
        match self {
            MacroLineErrorKind::MissingEquals => "thiếu dấu '=' giữa gõ tắt và nội dung",
            MacroLineErrorKind::EmptyTrigger => "chưa có chữ gõ tắt trước dấu '='",
            MacroLineErrorKind::TriggerHasSpace => "chữ gõ tắt không được chứa khoảng trắng",
            MacroLineErrorKind::TriggerTooLong => "chữ gõ tắt dài quá 32 ký tự",
            MacroLineErrorKind::EmptyExpansion => "chưa có nội dung sau dấu '='",
            MacroLineErrorKind::ExpansionTooLong => "nội dung dài quá 64 ký tự",
            MacroLineErrorKind::DuplicateTrigger => "chữ gõ tắt bị trùng với một dòng phía trên",
        }
    }
}

/// Lỗi tại dòng `line` (đếm từ 1, theo đúng dòng người dùng thấy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroLineError {
    pub line: usize,
    pub kind: MacroLineErrorKind,
}

impl MacroLineError {
    pub fn message_vi(&self) -> String {
        format!("Dòng {}: {}.", self.line, self.kind.message_vi())
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.peek() {
                Some('n') => {
                    it.next();
                    out.push('\n');
                    continue;
                }
                Some('\\') => {
                    it.next();
                    out.push('\\');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// `macros[]` → text cho ô soạn thảo (giữ thứ tự trong file).
pub fn format(macros: &[MacroEntry]) -> String {
    let mut out = String::new();
    for m in macros {
        out.push_str(&m.trigger);
        out.push_str(" = ");
        out.push_str(&escape(&m.expand));
        out.push('\n');
    }
    out
}

/// Text → `macros[]`. `previous` giữ lại trường `when` của trigger đã có (UI không sửa nó).
pub fn parse(text: &str, previous: &[MacroEntry]) -> Result<Vec<MacroEntry>, MacroLineError> {
    let mut out: Vec<MacroEntry> = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line_no = idx + 1;
        let err = |kind| MacroLineError {
            line: line_no,
            kind,
        };
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((lhs, rhs)) = line.split_once('=') else {
            return Err(err(MacroLineErrorKind::MissingEquals));
        };
        let trigger = lhs.trim();
        let expand = unescape(rhs.trim());
        if trigger.is_empty() {
            return Err(err(MacroLineErrorKind::EmptyTrigger));
        }
        if trigger.chars().any(char::is_whitespace) {
            return Err(err(MacroLineErrorKind::TriggerHasSpace));
        }
        if trigger.chars().count() > TRIGGER_MAX {
            return Err(err(MacroLineErrorKind::TriggerTooLong));
        }
        if expand.is_empty() {
            return Err(err(MacroLineErrorKind::EmptyExpansion));
        }
        if expand.chars().count() > EXPAND_MAX {
            return Err(err(MacroLineErrorKind::ExpansionTooLong));
        }
        let lower = trigger.to_lowercase();
        if out.iter().any(|m| m.trigger.to_lowercase() == lower) {
            return Err(err(MacroLineErrorKind::DuplicateTrigger));
        }
        let when = previous
            .iter()
            .find(|m| m.trigger.to_lowercase() == lower)
            .map(|m| m.when)
            .unwrap_or(MacroWhen::Always);
        out.push(MacroEntry {
            trigger: trigger.to_string(),
            expand,
            when,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(t: &str, e: &str, when: MacroWhen) -> MacroEntry {
        MacroEntry {
            trigger: t.into(),
            expand: e.into(),
            when,
        }
    }

    #[test]
    fn round_trip_keeps_order_and_special_chars() {
        let macros = vec![
            entry("cty", "Công ty TNHH", MacroWhen::Always),
            entry("dc", "Địa chỉ:\n123 Lê Lợi", MacroWhen::ViOn),
            entry("path", r"C:\new", MacroWhen::Always),
            entry("eq", "a = b", MacroWhen::Always),
        ];
        let text = format(&macros);
        assert_eq!(
            text,
            "cty = Công ty TNHH\ndc = Địa chỉ:\\n123 Lê Lợi\npath = C:\\\\new\neq = a = b\n"
        );
        assert_eq!(parse(&text, &macros).unwrap(), macros);
    }

    #[test]
    fn blank_comment_and_crlf_lines_are_ignored() {
        let got = parse("# ghi chú\r\n\r\n  vn   =  Việt Nam  \r\n", &[]).unwrap();
        assert_eq!(got, vec![entry("vn", "Việt Nam", MacroWhen::Always)]);
    }

    #[test]
    fn when_is_preserved_case_insensitively() {
        let prev = vec![entry("VN", "cũ", MacroWhen::ViOn)];
        let got = parse("vn = Việt Nam", &prev).unwrap();
        assert_eq!(got[0].when, MacroWhen::ViOn);
    }

    #[test]
    fn errors_point_at_the_user_visible_line() {
        let cases = [
            ("ok = 1\nnoequals", 2, MacroLineErrorKind::MissingEquals),
            ("= x", 1, MacroLineErrorKind::EmptyTrigger),
            ("a b = x", 1, MacroLineErrorKind::TriggerHasSpace),
            ("x =   ", 1, MacroLineErrorKind::EmptyExpansion),
            ("a = 1\n\nA = 2", 3, MacroLineErrorKind::DuplicateTrigger),
        ];
        for (text, line, kind) in cases {
            assert_eq!(
                parse(text, &[]),
                Err(MacroLineError { line, kind }),
                "{text:?}"
            );
        }
        let long_trigger = format!("{} = x", "a".repeat(TRIGGER_MAX + 1));
        assert_eq!(
            parse(&long_trigger, &[]).unwrap_err().kind,
            MacroLineErrorKind::TriggerTooLong
        );
        let long_expand = format!("x = {}", "ư".repeat(EXPAND_MAX + 1));
        assert_eq!(
            parse(&long_expand, &[]).unwrap_err().kind,
            MacroLineErrorKind::ExpansionTooLong
        );
        // Đúng 64 ký tự (đếm theo ký tự, không theo byte UTF-8) là hợp lệ.
        assert!(parse(&format!("x = {}", "ư".repeat(EXPAND_MAX)), &[]).is_ok());
    }

    #[test]
    fn error_codes_are_distinct_and_messages_non_empty() {
        let kinds = MacroLineErrorKind::ALL;
        let mut codes: Vec<i32> = kinds.iter().map(|k| k.code()).collect();
        codes.dedup();
        assert_eq!(codes.len(), kinds.len());
        assert!(kinds.iter().all(|k| !k.message_vi().is_empty()));
    }
}
