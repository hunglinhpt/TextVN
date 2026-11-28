// SPDX-License-Identifier: GPL-3.0-or-later
//! Luật nhận diện ô nhập macOS AX (R1–R10), tách từ `adapters/macos-imk/Sources/IMKLib/FieldDetect.swift`
//! và danh sách bundle terminal trong `TextVNInputController.swift`.
//!
//! Nguồn sự thật: `docs/30-macos/P2-3-field-detect-strategy.md §2` và `docs/10-shared/P0-7-TASKS-ARM-SF.md §SF-6`.
//! Crate thuần không I/O và không phụ thuộc AppKit/ApplicationServices.

use crate::SecurityState;
use textvn_strategy::{
    IME_FIELD_ADDRESS_BAR, IME_FIELD_CANDIDATE, IME_FIELD_COMBO, IME_FIELD_EDITBOX,
    IME_FIELD_SEARCH, IME_FIELD_SECURE, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN,
    IME_FIELD_WEB,
};

/// 10 bundle ID terminal macOS chuẩn (khớp 1:1 `TextVNInputController.isTerminalApp`).
pub const TERMINAL_BUNDLES: &[&str] = &[
    "com.apple.terminal",
    "com.googlecode.iterm2",
    "net.kovidgoyal.kitty",
    "io.alacritty",
    "org.alacritty",
    "com.github.wez.wezterm",
    "dev.warp.warp-stable",
    "co.zeit.hyper",
    "com.mitchellh.ghostty",
    "org.tabby",
];

/// Snapshot các thuộc tính Accessibility (AX) trên macOS.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AxSnapshot {
    pub role: Option<String>,
    pub subrole: Option<String>,
    pub placeholder: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub identifier: Option<String>,
    /// `true` khi process đang bật `IsSecureEventInputEnabled()` (S8).
    pub secure_input_mode: bool,
}

/// Kết quả phân loại từ snapshot macOS AX.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacProbeResult {
    pub field_role: u32,
    pub security: SecurityState,
}

/// Kiểm tra `bundle_id` có thuộc danh sách 10 terminal macOS chuẩn hay không.
pub fn is_terminal_app(bundle_id: &str) -> bool {
    let lower = bundle_id.trim().to_ascii_lowercase();
    TERMINAL_BUNDLES.contains(&lower.as_str())
}

/// Chuẩn hoá app_id cho macOS (`bundle_id` chữ thường → fallback `executable_name` basename → `"unknown"`),
/// khớp `FieldDetect.normalizeAppID` bên Swift.
pub fn normalize_mac_app_id(bundle_id: Option<&str>, executable_name: Option<&str>) -> String {
    if let Some(bundle) = bundle_id {
        let trimmed = bundle.trim();
        if !trimmed.is_empty() {
            return trimmed.to_lowercase();
        }
    }
    if let Some(exe) = executable_name {
        let normalized = crate::normalize_app_id(exe);
        if !normalized.is_empty() {
            return normalized;
        }
    }
    String::from("unknown")
}

/// Phân loại snapshot macOS AX theo đúng ưu tiên R1–R10 (`FieldRules.map`).
pub fn classify(ax: Option<&AxSnapshot>) -> MacProbeResult {
    let Some(ax) = ax else {
        return MacProbeResult {
            field_role: IME_FIELD_UNKNOWN,
            security: SecurityState::Unknown,
        };
    };

    // R1 — secure (S3/S8: không override được)
    if ax.secure_input_mode || ax.subrole.as_deref() == Some("AXSecureTextField") {
        return MacProbeResult {
            field_role: IME_FIELD_SECURE,
            security: SecurityState::Secure,
        };
    }

    let role = ax.role.as_deref();
    let subrole = ax.subrole.as_deref();

    // R2 — AXTextField với placeholder/description/title gợi URL hoặc search
    if role == Some("AXTextField") {
        if let Some(hint) = first_hint(ax) {
            if hint.contains("address") || hint.contains("url") {
                return MacProbeResult {
                    field_role: IME_FIELD_ADDRESS_BAR,
                    security: SecurityState::NonSecure,
                };
            }
            if hint.contains("search") {
                return MacProbeResult {
                    field_role: IME_FIELD_SEARCH,
                    security: SecurityState::NonSecure,
                };
            }
        }
    }

    // R3 — subrole AXSearchField
    if subrole == Some("AXSearchField") {
        return MacProbeResult {
            field_role: IME_FIELD_SEARCH,
            security: SecurityState::NonSecure,
        };
    }

    // R4 — combo / popup
    if matches!(role, Some("AXComboBox" | "AXPopUpButton")) {
        return MacProbeResult {
            field_role: IME_FIELD_COMBO,
            security: SecurityState::NonSecure,
        };
    }

    // R5 — candidate (Excel/Numbers grid cell)
    if role == Some("AXTextField")
        && ax
            .identifier
            .as_deref()
            .is_some_and(|id| id.to_lowercase().contains("grid"))
    {
        return MacProbeResult {
            field_role: IME_FIELD_CANDIDATE,
            security: SecurityState::NonSecure,
        };
    }

    // R6 — terminal (AXTextArea với identifier/title/description chứa "term")
    if role == Some("AXTextArea") && is_terminal_hint(ax) {
        return MacProbeResult {
            field_role: IME_FIELD_TERMINAL,
            security: SecurityState::NonSecure,
        };
    }

    // R7 — text area
    if role == Some("AXTextArea") {
        return MacProbeResult {
            field_role: IME_FIELD_TEXTAREA,
            security: SecurityState::NonSecure,
        };
    }

    // R8 — web area
    if role == Some("AXWebArea") {
        return MacProbeResult {
            field_role: IME_FIELD_WEB,
            security: SecurityState::NonSecure,
        };
    }

    // R9 — text field thường
    if role == Some("AXTextField") {
        return MacProbeResult {
            field_role: IME_FIELD_EDITBOX,
            security: SecurityState::NonSecure,
        };
    }

    // R10 — không query được
    MacProbeResult {
        field_role: IME_FIELD_UNKNOWN,
        security: SecurityState::NonSecure,
    }
}

fn is_terminal_hint(ax: &AxSnapshot) -> bool {
    [&ax.identifier, &ax.title, &ax.description]
        .into_iter()
        .flatten()
        .any(|s| s.to_lowercase().contains("term"))
}

fn first_hint(ax: &AxSnapshot) -> Option<String> {
    [&ax.placeholder, &ax.description, &ax.title]
        .into_iter()
        .flatten()
        .map(|s| s.to_lowercase())
        .find(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_ax_snapshot_is_fail_safe_unknown() {
        let r = classify(None);
        assert_eq!(r.field_role, IME_FIELD_UNKNOWN);
        assert_eq!(r.security, SecurityState::Unknown);
    }

    #[test]
    fn r1_through_r10_matrix_matches_swift() {
        let cases: Vec<(&str, AxSnapshot, u32, SecurityState)> = vec![
            (
                "secure input mode",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    secure_input_mode: true,
                    ..Default::default()
                },
                IME_FIELD_SECURE,
                SecurityState::Secure,
            ),
            (
                "secure subrole",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    subrole: Some("AXSecureTextField".into()),
                    placeholder: Some("Search address".into()),
                    ..Default::default()
                },
                IME_FIELD_SECURE,
                SecurityState::Secure,
            ),
            (
                "address bar via placeholder",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    placeholder: Some("Search or enter website address".into()),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
                SecurityState::NonSecure,
            ),
            (
                "address bar via url title when placeholder empty",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    placeholder: Some(String::new()),
                    title: Some("URL field".into()),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
                SecurityState::NonSecure,
            ),
            (
                "search via hint",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    description: Some("Search messages".into()),
                    ..Default::default()
                },
                IME_FIELD_SEARCH,
                SecurityState::NonSecure,
            ),
            (
                "search via AXSearchField subrole",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    subrole: Some("AXSearchField".into()),
                    ..Default::default()
                },
                IME_FIELD_SEARCH,
                SecurityState::NonSecure,
            ),
            (
                "combo box",
                AxSnapshot {
                    role: Some("AXComboBox".into()),
                    ..Default::default()
                },
                IME_FIELD_COMBO,
                SecurityState::NonSecure,
            ),
            (
                "popup button",
                AxSnapshot {
                    role: Some("AXPopUpButton".into()),
                    ..Default::default()
                },
                IME_FIELD_COMBO,
                SecurityState::NonSecure,
            ),
            (
                "candidate grid",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    identifier: Some("ExcelGridEditor".into()),
                    ..Default::default()
                },
                IME_FIELD_CANDIDATE,
                SecurityState::NonSecure,
            ),
            (
                "terminal textarea",
                AxSnapshot {
                    role: Some("AXTextArea".into()),
                    identifier: Some("terminal-view".into()),
                    ..Default::default()
                },
                IME_FIELD_TERMINAL,
                SecurityState::NonSecure,
            ),
            (
                "regular textarea",
                AxSnapshot {
                    role: Some("AXTextArea".into()),
                    identifier: Some("notes-editor".into()),
                    ..Default::default()
                },
                IME_FIELD_TEXTAREA,
                SecurityState::NonSecure,
            ),
            (
                "web area",
                AxSnapshot {
                    role: Some("AXWebArea".into()),
                    ..Default::default()
                },
                IME_FIELD_WEB,
                SecurityState::NonSecure,
            ),
            (
                "plain editbox",
                AxSnapshot {
                    role: Some("AXTextField".into()),
                    ..Default::default()
                },
                IME_FIELD_EDITBOX,
                SecurityState::NonSecure,
            ),
            (
                "unknown fallback",
                AxSnapshot {
                    role: Some("AXWindow".into()),
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
                SecurityState::NonSecure,
            ),
        ];

        for (label, ax, expected_role, expected_sec) in cases {
            let got = classify(Some(&ax));
            assert_eq!(got.field_role, expected_role, "case {label}");
            assert_eq!(got.security, expected_sec, "case {label}");
        }
    }

    #[test]
    fn first_hint_takes_first_non_empty_among_placeholder_description_title() {
        // Placeholder "notes" is non-empty, so later description "URL" does not match R2.
        let ax = AxSnapshot {
            role: Some("AXTextField".into()),
            placeholder: Some("notes".into()),
            description: Some("URL".into()),
            ..Default::default()
        };
        assert_eq!(classify(Some(&ax)).field_role, IME_FIELD_EDITBOX);
    }

    #[test]
    fn terminal_bundle_list_and_normalize_app_id() {
        assert_eq!(TERMINAL_BUNDLES.len(), 10);
        for b in TERMINAL_BUNDLES {
            assert!(is_terminal_app(b));
        }
        assert!(is_terminal_app(" com.apple.Terminal "));
        assert!(!is_terminal_app("com.apple.Safari"));

        assert_eq!(
            normalize_mac_app_id(Some("com.google.Chrome"), Some("Chrome")),
            "com.google.chrome"
        );
        assert_eq!(
            normalize_mac_app_id(None, Some("/Applications/Safari.app/Contents/MacOS/Safari")),
            "safari"
        );
        assert_eq!(normalize_mac_app_id(Some("  "), None), "unknown");
    }
}
