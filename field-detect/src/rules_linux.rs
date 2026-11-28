// SPDX-License-Identifier: GPL-3.0-or-later
//! Luật nhận diện ô nhập Linux AT-SPI (R1–R9 + fallback), tách từ `adapters/linux-common/src/field_detect.c`.
//!
//! Nguồn sự thật: `docs/40-linux/P3-4-strategy-appdb.md §1–§3` và `docs/10-shared/P0-7-TASKS-ARM-SF.md §SF-6`.
//! Crate thuần không I/O và không phụ thuộc D-Bus/AT-SPI runtime.

use crate::SecurityState;
use textvn_strategy::{
    IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO, IME_FIELD_EDITBOX,
    IME_FIELD_SEARCH, IME_FIELD_SECURE, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN,
    IME_FIELD_WEB, IME_STRATEGY_BACKSPACE_TYPE, IME_STRATEGY_FORWARD_AS_COMMIT,
    IME_STRATEGY_PASSTHROUGH, IME_STRATEGY_PREEDIT, IME_STRATEGY_SELECTION_REPLACE,
};

/// 20 tên ứng dụng terminal chuẩn trên Linux (khớp 1:1 `lc_is_terminal_app` trong `field_detect.c`).
pub const TERMINAL_APPS: &[&str] = &[
    "terminal",
    "konsole",
    "kitty",
    "alacritty",
    "xterm",
    "uxterm",
    "foot",
    "footclient",
    "wezterm",
    "console",
    "lxterminal",
    "qterminal",
    "tilix",
    "terminator",
    "terminology",
    "urxvt",
    "rxvt",
    "rxvt-unicode",
    "ptyxis",
    "kgx",
];

/// Bit cờ password/protected trong `AtspiSnapshot::state_flags` (khớp `field_detect.c`).
pub const ATSPI_FLAG_PASSWORD: u64 = 0x1;

/// Snapshot thuộc tính AT-SPI đã đọc trên adapter Linux.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AtspiSnapshot {
    pub app_id: String,
    pub role: String,
    pub name: String,
    pub description: String,
    pub state_flags: u64,
}

/// Kết quả phân loại ô nhập Linux AT-SPI (`strategy_hint == -1` = nhường `textvn-strategy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxProbeResult {
    pub field_role: u32,
    pub security: SecurityState,
    pub strategy_hint: i64,
}

/// Tìm từ khoá đứng riêng theo ranh giới ký tự chữ-số ASCII (ví dụ `org.gnome.Console`,
/// `xfce4-terminal`, nhưng KHÔNG khớp chuỗi con như `footage-editor` với `"foot"`).
pub fn has_word_icase(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return true;
    }
    let wlen = word.len();
    if text.len() < wlen {
        return false;
    }
    for (i, _) in text.char_indices() {
        let Some(end) = i.checked_add(wlen) else {
            break;
        };
        if end > text.len() {
            break;
        }
        let prev_is_alnum = text[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if prev_is_alnum {
            continue;
        }
        let Some(slice) = text.get(i..end) else {
            continue;
        };
        if slice.eq_ignore_ascii_case(word) {
            let next_is_alnum = text[end..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric());
            if !next_is_alnum {
                return true;
            }
        }
    }
    false
}

/// Kiểm tra `app_id` có phải ứng dụng terminal Linux đã biết hay không.
pub fn is_terminal_app(app_id: &str) -> bool {
    TERMINAL_APPS
        .iter()
        .any(|name| has_word_icase(app_id, name))
}

/// Phân loại ô nhập Linux theo đúng thứ tự ưu tiên `lc_classify_field`:
/// R1 (Password/Secret) → R6 (Terminal) → R2 (Address bar) → R3 (Search) →
/// Chat (Bug B2) → R4 (Combo) → R5 (Table/Spreadsheet) → R8 (Web) → R7 (Textarea) →
/// R9 (Editbox) → Fallback (`Unknown`, `strategy_hint = -1`).
pub fn classify(snapshot: Option<&AtspiSnapshot>) -> LinuxProbeResult {
    let Some(s) = snapshot else {
        return LinuxProbeResult {
            field_role: IME_FIELD_UNKNOWN,
            security: SecurityState::Unknown,
            strategy_hint: -1,
        };
    };

    let app = s.app_id.to_lowercase();
    let role = s.role.to_lowercase();
    let name = s.name.to_lowercase();
    let desc = s.description.to_lowercase();

    // R1: Password / secret (S3: không bao giờ override được)
    let is_pwd_role = role.contains("password") || role.contains("secret");
    let is_pwd_state = (s.state_flags & ATSPI_FLAG_PASSWORD) != 0;
    if is_pwd_role || is_pwd_state {
        return LinuxProbeResult {
            field_role: IME_FIELD_SECURE,
            security: SecurityState::Secure,
            strategy_hint: IME_STRATEGY_PASSTHROUGH,
        };
    }

    // R6: Terminal (role hoặc terminal app_id)
    if role.contains("terminal") || is_terminal_app(&s.app_id) {
        return LinuxProbeResult {
            field_role: IME_FIELD_TERMINAL,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_FORWARD_AS_COMMIT,
        };
    }

    // R2: Address bar
    if name.contains("address")
        || name.contains("url")
        || name.contains("location")
        || name.contains("địa chỉ")
        || desc.contains("address")
        || desc.contains("url")
    {
        return LinuxProbeResult {
            field_role: IME_FIELD_ADDRESS_BAR,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_SELECTION_REPLACE,
        };
    }

    // R3: Search box
    if name.contains("search")
        || name.contains("tìm kiếm")
        || name.contains("tìm")
        || desc.contains("search")
        || desc.contains("tìm")
    {
        return LinuxProbeResult {
            field_role: IME_FIELD_SEARCH,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_SELECTION_REPLACE,
        };
    }

    // Chat apps & chat inputs (tránh Bug B2 lặp Enter khi preedit)
    let is_chat_app = [
        "slack",
        "discord",
        "telegram",
        "zalo",
        "element",
        "signal",
        "mattermost",
        "teams",
    ]
    .iter()
    .any(|kw| app.contains(kw));
    let is_chat_element = ["message", "tin nhắn", "chat", "soạn tin"]
        .iter()
        .any(|kw| name.contains(kw));
    if is_chat_app || is_chat_element {
        return LinuxProbeResult {
            field_role: IME_FIELD_BODY,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_BACKSPACE_TYPE,
        };
    }

    // R4: Combo box
    if role.contains("combo") {
        return LinuxProbeResult {
            field_role: IME_FIELD_COMBO,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_SELECTION_REPLACE,
        };
    }

    // R5: Spreadsheet / Candidate table cell
    let is_table_role = role.contains("table") || role.contains("cell");
    let is_calc_app = app.contains("calc") || app.contains("gnumeric") || app.contains("excel");
    if is_table_role || is_calc_app {
        return LinuxProbeResult {
            field_role: IME_FIELD_CANDIDATE,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_SELECTION_REPLACE,
        };
    }

    // R8: Web document
    if role.contains("document_web") {
        return LinuxProbeResult {
            field_role: IME_FIELD_WEB,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_PREEDIT,
        };
    }

    // R7: Multi-line text area / section / paragraph
    if role.contains("section") || role.contains("paragraph") || role.contains("text_area") {
        return LinuxProbeResult {
            field_role: IME_FIELD_TEXTAREA,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_PREEDIT,
        };
    }

    // R9: Generic entry / editbox / text
    if role.contains("entry") || role.contains("editbox") || role.contains("text") {
        return LinuxProbeResult {
            field_role: IME_FIELD_EDITBOX,
            security: SecurityState::NonSecure,
            strategy_hint: IME_STRATEGY_PREEDIT,
        };
    }

    // Fallback
    LinuxProbeResult {
        field_role: IME_FIELD_UNKNOWN,
        security: SecurityState::NonSecure,
        strategy_hint: -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(app_id: &str, role: &str, name: &str, description: &str, flags: u64) -> AtspiSnapshot {
        AtspiSnapshot {
            app_id: app_id.into(),
            role: role.into(),
            name: name.into(),
            description: description.into(),
            state_flags: flags,
        }
    }

    #[test]
    fn none_snapshot_is_fail_safe_unknown() {
        let r = classify(None);
        assert_eq!(r.field_role, IME_FIELD_UNKNOWN);
        assert_eq!(r.security, SecurityState::Unknown);
        assert_eq!(r.strategy_hint, -1);
    }

    #[test]
    fn r1_password_role_and_state_flag_win_over_everything() {
        let r1 = classify(Some(&snap(
            "gnome-terminal",
            "password_text",
            "Address",
            "",
            0,
        )));
        assert_eq!(
            r1,
            LinuxProbeResult {
                field_role: IME_FIELD_SECURE,
                security: SecurityState::Secure,
                strategy_hint: IME_STRATEGY_PASSTHROUGH,
            }
        );

        let r2 = classify(Some(&snap("firefox", "entry", "PIN", "", 0x1)));
        assert_eq!(r2, r1);

        let r3 = classify(Some(&snap("chrome", "SecretInput", "", "", 0)));
        assert_eq!(r3, r1);
    }

    #[test]
    fn terminal_word_boundary_matches_and_avoids_false_positives() {
        assert!(is_terminal_app("org.gnome.Console"));
        assert!(is_terminal_app("xfce4-terminal"));
        assert!(is_terminal_app("WezTerm"));
        assert!(is_terminal_app("rxvt-unicode"));
        assert!(is_terminal_app("kgx"));
        assert!(!is_terminal_app("footage-editor"));
        assert!(!is_terminal_app("firefox"));

        let term = classify(Some(&snap("kitty", "entry", "", "", 0)));
        assert_eq!(term.field_role, IME_FIELD_TERMINAL);
        assert_eq!(term.strategy_hint, IME_STRATEGY_FORWARD_AS_COMMIT);

        let by_role = classify(Some(&snap("custom-app", "terminal", "", "", 0)));
        assert_eq!(by_role.field_role, IME_FIELD_TERMINAL);
        assert_eq!(by_role.strategy_hint, IME_STRATEGY_FORWARD_AS_COMMIT);
    }

    #[test]
    fn r2_address_bar_and_unicode_vietnamese_labels() {
        let r_en = classify(Some(&snap(
            "firefox",
            "entry",
            "Search or enter address",
            "",
            0,
        )));
        assert_eq!(r_en.field_role, IME_FIELD_ADDRESS_BAR);
        assert_eq!(r_en.strategy_hint, IME_STRATEGY_SELECTION_REPLACE);

        let r_vi = classify(Some(&snap("firefox", "entry", "Thanh Địa chỉ", "", 0)));
        assert_eq!(r_vi.field_role, IME_FIELD_ADDRESS_BAR);
        assert_eq!(r_vi.strategy_hint, IME_STRATEGY_SELECTION_REPLACE);

        let r_desc = classify(Some(&snap("chrome", "entry", "Location", "URL bar", 0)));
        assert_eq!(r_desc.field_role, IME_FIELD_ADDRESS_BAR);
    }

    #[test]
    fn r3_search_box_and_vietnamese_labels() {
        let r = classify(Some(&snap("nautilus", "entry", "Tìm kiếm tệp", "", 0)));
        assert_eq!(r.field_role, IME_FIELD_SEARCH);
        assert_eq!(r.strategy_hint, IME_STRATEGY_SELECTION_REPLACE);

        let r2 = classify(Some(&snap("app", "entry", "Box", "Quick search", 0)));
        assert_eq!(r2.field_role, IME_FIELD_SEARCH);
    }

    #[test]
    fn chat_apps_and_elements_select_backspace_type() {
        let slack = classify(Some(&snap("slack", "text", "Editor", "", 0)));
        assert_eq!(slack.field_role, IME_FIELD_BODY);
        assert_eq!(slack.strategy_hint, IME_STRATEGY_BACKSPACE_TYPE);

        let element = classify(Some(&snap("firefox", "entry", "Soạn tin nhắn", "", 0)));
        assert_eq!(element.field_role, IME_FIELD_BODY);
        assert_eq!(element.strategy_hint, IME_STRATEGY_BACKSPACE_TYPE);
    }

    #[test]
    fn r4_combo_r5_candidate_r8_web_r7_textarea_r9_editbox_and_fallback() {
        assert_eq!(
            classify(Some(&snap("gedit", "combo_box", "", "", 0))).field_role,
            IME_FIELD_COMBO
        );
        assert_eq!(
            classify(Some(&snap("libreoffice-calc", "entry", "", "", 0))).field_role,
            IME_FIELD_CANDIDATE
        );
        assert_eq!(
            classify(Some(&snap("sheet", "table_cell", "", "", 0))).strategy_hint,
            IME_STRATEGY_SELECTION_REPLACE
        );
        assert_eq!(
            classify(Some(&snap("firefox", "document_web", "", "", 0))).field_role,
            IME_FIELD_WEB
        );
        assert_eq!(
            classify(Some(&snap("gedit", "text_area", "", "", 0))).field_role,
            IME_FIELD_TEXTAREA
        );
        assert_eq!(
            classify(Some(&snap("gedit", "entry", "", "", 0))).field_role,
            IME_FIELD_EDITBOX
        );
        let fallback = classify(Some(&snap("unknown-app", "canvas", "", "", 0)));
        assert_eq!(fallback.field_role, IME_FIELD_UNKNOWN);
        assert_eq!(fallback.strategy_hint, -1);
    }
}
