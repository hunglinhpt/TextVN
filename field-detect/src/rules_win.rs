// SPDX-License-Identifier: GPL-3.0-or-later
//! Rule R1–R10: snapshot UIA đã đọc bởi adapter Windows → role/security.
//!
//! Không gọi UIA trong module này. Adapter đọc properties trên worker/cached
//! snapshot rồi publish đúng generation vào `FieldContext` owner thread.

use crate::SecurityState;
use textvn_strategy::{
    IME_FIELD_ADDRESS_BAR, IME_FIELD_CANDIDATE, IME_FIELD_COMBO, IME_FIELD_EDITBOX,
    IME_FIELD_SEARCH, IME_FIELD_SECURE, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN,
    IME_FIELD_WEB,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlType {
    Edit,
    ComboBox,
    Document,
    TextArea,
    #[default]
    Other,
}

/// Raw UIA properties, owned strings để snapshot an toàn qua worker boundary.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UiaElement {
    pub control_type: ControlType,
    /// `None` = property không đọc được: phải giữ security Unknown.
    pub is_password: Option<bool>,
    pub name: String,
    pub automation_id: String,
    pub class_name: String,
    pub ancestor_classes: Vec<String>,
    pub keyboard_focusable: bool,
    pub control_element: bool,
    pub multiline: bool,
    pub value_pattern: bool,
    pub expand_collapse_pattern: bool,
}

/// Kết quả dùng để gọi `FieldContext::apply_probe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeResult {
    pub field_role: u32,
    pub security: SecurityState,
}

/// `None` nghĩa UIA không trả được element — fail-safe, không được dùng R10
/// `NonSecure`. UIA element có `is_password=None` cũng không unblock input.
pub fn classify(element: Option<&UiaElement>) -> ProbeResult {
    let Some(element) = element else {
        return ProbeResult {
            field_role: IME_FIELD_UNKNOWN,
            security: SecurityState::Unknown,
        };
    };
    if element.is_password == Some(true) {
        return ProbeResult {
            field_role: IME_FIELD_SECURE,
            security: SecurityState::Secure,
        };
    }
    if element.is_password != Some(false) {
        return ProbeResult {
            field_role: IME_FIELD_UNKNOWN,
            security: SecurityState::Unknown,
        };
    }

    ProbeResult {
        field_role: role(element),
        security: SecurityState::NonSecure,
    }
}

fn role(e: &UiaElement) -> u32 {
    let name = lower(&e.name);
    let automation_id = lower(&e.automation_id);
    let class = lower(&e.class_name);
    let has_class =
        |needle: &str| class == needle || e.ancestor_classes.iter().any(|c| lower(c) == needle);

    // R2
    if class == "windows.ui.core.corewindow"
        && (contains(&name, "address") || contains(&name, "search"))
    {
        return address_or_search(&name);
    }
    // R3
    if e.control_type == ControlType::Edit
        && (matches!(
            automation_id.as_str(),
            "address" | "addressbar" | "searchbox" | "search"
        ) || contains(&name, "address and search bar")
            || contains(&name, "address bar")
            || contains(&name, "search"))
    {
        return address_or_search(&format!("{automation_id} {name}"));
    }
    // R4
    if e.control_type == ControlType::ComboBox
        || (e.control_type == ControlType::Edit && e.value_pattern && e.expand_collapse_pattern)
    {
        return IME_FIELD_COMBO;
    }
    // R5
    if has_class("excel7") || has_class("xlgrid") {
        return IME_FIELD_CANDIDATE;
    }
    // R6
    if e.control_type == ControlType::Edit
        && e.keyboard_focusable
        && e.control_element
        && !e.multiline
    {
        return IME_FIELD_EDITBOX;
    }
    // R7
    if e.control_type == ControlType::Document && has_class("chrome_renderwidgethosthwnd") {
        return IME_FIELD_WEB;
    }
    // R8
    if e.control_type == ControlType::TextArea
        || (e.control_type == ControlType::Edit && e.multiline)
    {
        return IME_FIELD_TEXTAREA;
    }
    // R9
    if ["cascadia_hosting_window_class", "putty", "xterm", "wezterm"]
        .into_iter()
        .any(has_class)
    {
        return IME_FIELD_TERMINAL;
    }
    // R10
    IME_FIELD_UNKNOWN
}

fn lower(value: &str) -> String {
    value.to_ascii_lowercase()
}

fn contains(value: &str, needle: &str) -> bool {
    value.contains(needle)
}

fn address_or_search(value: &str) -> u32 {
    if contains(value, "address") {
        IME_FIELD_ADDRESS_BAR
    } else {
        IME_FIELD_SEARCH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(mut element: UiaElement) -> UiaElement {
        element.is_password = Some(false);
        element
    }

    fn role_of(element: UiaElement) -> u32 {
        classify(Some(&known(element))).field_role
    }

    #[test]
    fn r1_password_and_missing_uia_are_fail_safe() {
        assert_eq!(
            classify(None),
            ProbeResult {
                field_role: IME_FIELD_UNKNOWN,
                security: SecurityState::Unknown
            }
        );
        assert_eq!(
            classify(Some(&UiaElement {
                is_password: Some(true),
                ..Default::default()
            })),
            ProbeResult {
                field_role: IME_FIELD_SECURE,
                security: SecurityState::Secure
            }
        );
        assert_eq!(
            classify(Some(&UiaElement::default())).security,
            SecurityState::Unknown
        );
    }

    #[test]
    fn r2_core_window_name_is_address_or_search() {
        assert_eq!(
            role_of(UiaElement {
                class_name: "Windows.UI.Core.CoreWindow".into(),
                name: "Search files".into(),
                ..Default::default()
            }),
            IME_FIELD_SEARCH
        );
    }

    #[test]
    fn r3_edit_metadata_is_address() {
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::Edit,
                automation_id: "Address".into(),
                ..Default::default()
            }),
            IME_FIELD_ADDRESS_BAR
        );
    }

    #[test]
    fn r4_combo_and_expandable_edit_win_over_plain_edit() {
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::ComboBox,
                ..Default::default()
            }),
            IME_FIELD_COMBO
        );
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::Edit,
                value_pattern: true,
                expand_collapse_pattern: true,
                ..Default::default()
            }),
            IME_FIELD_COMBO
        );
    }

    #[test]
    fn r5_excel_grid_is_candidate() {
        assert_eq!(
            role_of(UiaElement {
                ancestor_classes: vec!["XLGRID".into()],
                ..Default::default()
            }),
            IME_FIELD_CANDIDATE
        );
    }

    #[test]
    fn r6_to_r10_classify_remaining_controls() {
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::Edit,
                keyboard_focusable: true,
                control_element: true,
                ..Default::default()
            }),
            IME_FIELD_EDITBOX
        );
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::Document,
                ancestor_classes: vec!["Chrome_RenderWidgetHostHWND".into()],
                ..Default::default()
            }),
            IME_FIELD_WEB
        );
        assert_eq!(
            role_of(UiaElement {
                control_type: ControlType::Edit,
                multiline: true,
                ..Default::default()
            }),
            IME_FIELD_TEXTAREA
        );
        assert_eq!(
            role_of(UiaElement {
                class_name: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
                ..Default::default()
            }),
            IME_FIELD_TERMINAL
        );
        assert_eq!(role_of(UiaElement::default()), IME_FIELD_UNKNOWN);
    }

    #[test]
    fn win030_mock_matrix_has_thirty_rule_cases() {
        let cases: Vec<(UiaElement, u32)> = vec![
            // R2: CoreWindow name
            (
                UiaElement {
                    class_name: "Windows.UI.Core.CoreWindow".into(),
                    name: "Address".into(),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
            ),
            (
                UiaElement {
                    class_name: "Windows.UI.Core.CoreWindow".into(),
                    name: "Search".into(),
                    ..Default::default()
                },
                IME_FIELD_SEARCH,
            ),
            // R3: edit metadata/name
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    automation_id: "Address".into(),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    automation_id: "AddressBar".into(),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    automation_id: "SearchBox".into(),
                    ..Default::default()
                },
                IME_FIELD_SEARCH,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    name: "Address and search bar".into(),
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    name: "Search the web".into(),
                    ..Default::default()
                },
                IME_FIELD_SEARCH,
            ),
            // R4: combo
            (
                UiaElement {
                    control_type: ControlType::ComboBox,
                    ..Default::default()
                },
                IME_FIELD_COMBO,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    value_pattern: true,
                    expand_collapse_pattern: true,
                    ..Default::default()
                },
                IME_FIELD_COMBO,
            ),
            // R5: Excel
            (
                UiaElement {
                    class_name: "Excel7".into(),
                    ..Default::default()
                },
                IME_FIELD_CANDIDATE,
            ),
            (
                UiaElement {
                    ancestor_classes: vec!["XLGRID".into()],
                    ..Default::default()
                },
                IME_FIELD_CANDIDATE,
            ),
            // R6: ordinary single-line edit
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: true,
                    control_element: true,
                    ..Default::default()
                },
                IME_FIELD_EDITBOX,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: true,
                    control_element: true,
                    name: "Username".into(),
                    ..Default::default()
                },
                IME_FIELD_EDITBOX,
            ),
            // R7: web document
            (
                UiaElement {
                    control_type: ControlType::Document,
                    class_name: "Chrome_RenderWidgetHostHWND".into(),
                    ..Default::default()
                },
                IME_FIELD_WEB,
            ),
            (
                UiaElement {
                    control_type: ControlType::Document,
                    ancestor_classes: vec!["Chrome_RenderWidgetHostHWND".into()],
                    ..Default::default()
                },
                IME_FIELD_WEB,
            ),
            // R8: multiline
            (
                UiaElement {
                    control_type: ControlType::TextArea,
                    ..Default::default()
                },
                IME_FIELD_TEXTAREA,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    multiline: true,
                    ..Default::default()
                },
                IME_FIELD_TEXTAREA,
            ),
            // R9: terminal classes
            (
                UiaElement {
                    class_name: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
                    ..Default::default()
                },
                IME_FIELD_TERMINAL,
            ),
            (
                UiaElement {
                    class_name: "PuTTY".into(),
                    ..Default::default()
                },
                IME_FIELD_TERMINAL,
            ),
            (
                UiaElement {
                    class_name: "xterm".into(),
                    ..Default::default()
                },
                IME_FIELD_TERMINAL,
            ),
            (
                UiaElement {
                    ancestor_classes: vec!["WezTerm".into()],
                    ..Default::default()
                },
                IME_FIELD_TERMINAL,
            ),
            // R10 / negative paths
            (UiaElement::default(), IME_FIELD_UNKNOWN),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: false,
                    control_element: true,
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: true,
                    control_element: false,
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
            ),
            (
                UiaElement {
                    control_type: ControlType::Document,
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
            ),
            (
                UiaElement {
                    class_name: "notepad".into(),
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
            ),
            (
                UiaElement {
                    control_type: ControlType::Other,
                    name: "address".into(),
                    ..Default::default()
                },
                IME_FIELD_UNKNOWN,
            ),
            // priority: address metadata precedes combo, combo precedes plain edit,
            // Excel precedes plain edit.
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    automation_id: "Address".into(),
                    value_pattern: true,
                    expand_collapse_pattern: true,
                    ..Default::default()
                },
                IME_FIELD_ADDRESS_BAR,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: true,
                    control_element: true,
                    value_pattern: true,
                    expand_collapse_pattern: true,
                    ..Default::default()
                },
                IME_FIELD_COMBO,
            ),
            (
                UiaElement {
                    control_type: ControlType::Edit,
                    keyboard_focusable: true,
                    control_element: true,
                    ancestor_classes: vec!["Excel7".into()],
                    ..Default::default()
                },
                IME_FIELD_CANDIDATE,
            ),
        ];
        assert_eq!(cases.len(), 30, "WIN-030 minimum mock matrix");
        for (element, expected_role) in cases {
            assert_eq!(role_of(element), expected_role);
        }
    }
}
