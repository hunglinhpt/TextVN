// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng field_role → strategy mặc định khi không preset/không hint (P0-3 §3.1 bước 5).

use crate::resolve::Strategy;
use crate::{
    IME_CAP_FIELD_DETECT, IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR,
    IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_COMBO, IME_FIELD_EDITBOX, IME_FIELD_SEARCH,
    IME_FIELD_SECURE, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN, IME_FIELD_WEB,
};

/// Danh sách role theo đúng chuỗi JSON (P0-3 §2.1) — thứ tự = thứ tự FFI.
pub const FIELD_ROLES: [(&str, u32); 11] = [
    ("unknown", IME_FIELD_UNKNOWN),
    ("body", IME_FIELD_BODY),
    ("editbox", IME_FIELD_EDITBOX),
    ("address_bar", IME_FIELD_ADDRESS_BAR),
    ("search", IME_FIELD_SEARCH),
    ("combo", IME_FIELD_COMBO),
    ("candidate", IME_FIELD_CANDIDATE),
    ("textarea", IME_FIELD_TEXTAREA),
    ("web", IME_FIELD_WEB),
    ("terminal", IME_FIELD_TERMINAL),
    ("secure", IME_FIELD_SECURE),
];

/// Parse chuỗi role JSON → số FFI. Không khớp → `None`.
pub fn parse_field_role(s: &str) -> Option<u32> {
    FIELD_ROLES
        .iter()
        .find(|(name, _)| *name == s)
        .map(|(_, id)| *id)
}

/// Bước 5 — default theo field, có gate capability.
/// Bước 6 — fallback cuối: `BackspaceType`.
pub fn default_for_field(field_role: u32, caps: u32) -> Strategy {
    match field_role {
        IME_FIELD_ADDRESS_BAR | IME_FIELD_SEARCH | IME_FIELD_COMBO => {
            if caps & IME_CAP_SELECTION != 0 && caps & IME_CAP_FIELD_DETECT != 0 {
                Strategy::SelectionReplace
            } else {
                Strategy::BackspaceType
            }
        }
        IME_FIELD_TERMINAL => Strategy::ForwardAsCommit,
        IME_FIELD_CANDIDATE => {
            if caps & IME_CAP_SELECTION != 0 {
                Strategy::SelectionReplace
            } else {
                Strategy::BackspaceType
            }
        }
        IME_FIELD_SECURE => Strategy::Passthrough,
        // body/editbox/web/textarea/unknown
        _ => {
            if caps & IME_CAP_PREEDIT != 0 {
                Strategy::Preedit
            } else {
                Strategy::BackspaceType
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_table_matches_p0_3_mapping() {
        assert_eq!(parse_field_role("body"), Some(IME_FIELD_BODY));
        assert_eq!(parse_field_role("address_bar"), Some(IME_FIELD_ADDRESS_BAR));
        assert_eq!(parse_field_role("secure"), Some(IME_FIELD_SECURE));
        assert_eq!(parse_field_role("candidate"), Some(IME_FIELD_CANDIDATE));
        assert_eq!(parse_field_role("nope"), None);
        assert_eq!(FIELD_ROLES.len(), 11);
    }
}
