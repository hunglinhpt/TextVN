// SPDX-License-Identifier: GPL-3.0-or-later
//! preset_matrix — ma trận resolve 20 preset mac × field_role × caps (P2-3 §6).
//!
//! Spec chỉ định test ở `strategy/tests` nhưng `strategy` không được phụ thuộc
//! `appdb` (chiều phụ thuộc ngược lại) — matrix nằm ở đây: `appdb` cấp
//! system_preset rồi gọi thẳng resolver dùng chung (`appdb::resolve` →
//! `textvn_strategy::resolve`), KHÔNG nhân bản thuật toán (P2-3 §5).
//!
//! Chạy mọi OS (headless — appdb parse thuần).

use textvn_appdb::AppDb;
use textvn_strategy::{
    ResolveInput, Strategy, IME_CAP_FIELD_DETECT, IME_CAP_PREEDIT, IME_CAP_SELECTION,
    IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_CANDIDATE, IME_FIELD_EDITBOX,
    IME_FIELD_SEARCH, IME_FIELD_TERMINAL, IME_FIELD_TEXTAREA, IME_FIELD_UNKNOWN, IME_FIELD_WEB,
};

const MAC_CAPS: u32 = IME_CAP_PREEDIT | IME_CAP_SELECTION | IME_CAP_FIELD_DETECT;

fn db() -> AppDb {
    AppDb::parse(include_str!("../../data/appdb.default.json")).unwrap()
}

fn resolve(db: &AppDb, app_id: &str, role: u32, caps: u32) -> Strategy {
    db.resolve(
        ResolveInput {
            secure: false,
            enabled: true,
            user_preset: None,
            system_preset: None,
            hint: -1,
            field_role: role,
            caps,
        },
        app_id,
    )
}

/// Nhóm B1 — address bar/search → SelectionReplace (thay selection, không backspace).
#[test]
fn b1_url_search_selection_replace() {
    let d = db();
    for app in [
        "com.apple.safari",
        "com.google.chrome",
        "com.microsoft.edgemac",
        "com.brave.browser",
        "org.mozilla.firefox",
        "com.apple.spotlight",
    ] {
        for role in [IME_FIELD_ADDRESS_BAR, IME_FIELD_SEARCH] {
            assert_eq!(
                resolve(&d, app, role, MAC_CAPS),
                Strategy::SelectionReplace,
                "{app} role={role}"
            );
        }
    }
}

/// Nhóm body/web/textarea → Preedit (marked text IMK).
#[test]
fn body_preedit() {
    let d = db();
    let cases: [(&str, u32); 9] = [
        ("com.apple.safari", IME_FIELD_WEB),
        ("com.apple.safari", IME_FIELD_BODY),
        ("com.microsoft.word", IME_FIELD_BODY),
        ("com.microsoft.word", IME_FIELD_TEXTAREA),
        ("com.apple.notes", IME_FIELD_TEXTAREA),
        ("com.apple.textedit", IME_FIELD_BODY),
        ("com.apple.dt.xcode", IME_FIELD_BODY),
        ("com.tinyspeck.slackmacgap", IME_FIELD_BODY),
        ("com.apple.ichat", IME_FIELD_BODY),
    ];
    for (app, role) in cases {
        assert_eq!(resolve(&d, app, role, MAC_CAPS), Strategy::Preedit, "{app}");
    }
}

/// Terminal → ForwardAsCommit (B8 — giữ marked ngắn).
#[test]
fn terminal_forward_as_commit() {
    let d = db();
    for app in ["com.apple.terminal", "com.googlecode.iterm2"] {
        assert_eq!(
            resolve(&d, app, IME_FIELD_TERMINAL, MAC_CAPS),
            Strategy::ForwardAsCommit
        );
    }
}

/// Candidate (B3) → SelectionReplace khi có selection cap.
#[test]
fn candidate_selection_replace() {
    let d = db();
    for app in [
        "com.microsoft.excel",
        "com.jetbrains.intellij",
        "com.jetbrains.pycharm",
    ] {
        assert_eq!(
            resolve(&d, app, IME_FIELD_CANDIDATE, MAC_CAPS),
            Strategy::SelectionReplace
        );
    }
}

/// App mặc định TẮT (dev list) → Passthrough qua gate enabled (bước 2 P0-3 §3.1).
#[test]
fn disabled_app_passthrough() {
    let d = db();
    // enabled=false đi qua ctx.enabled (bước 2), preset không được thắng.
    let out = d.resolve(
        ResolveInput {
            secure: false,
            enabled: false,
            user_preset: None,
            system_preset: None,
            hint: -1,
            field_role: IME_FIELD_BODY,
            caps: MAC_CAPS,
        },
        "com.microsoft.vscode",
    );
    assert_eq!(out, Strategy::Passthrough);
}

/// Downgrade bắt buộc theo caps (P0-3 §3.1): mất cap → BackspaceType.
#[test]
fn caps_missing_downgrades() {
    let d = db();
    // Mất PREEDIT: body → không Preedit nữa.
    assert_eq!(
        resolve(
            &d,
            "com.apple.textedit",
            IME_FIELD_BODY,
            IME_CAP_FIELD_DETECT
        ),
        Strategy::BackspaceType
    );
    // Mất SELECTION: address bar → BackspaceType (không SelectionReplace).
    assert_eq!(
        resolve(
            &d,
            "com.apple.safari",
            IME_FIELD_ADDRESS_BAR,
            IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT
        ),
        Strategy::BackspaceType
    );
    // Caps rỗng + role lạ → fallback cuối BackspaceType (bước 6).
    assert_eq!(
        resolve(&d, "app.khong.preset", IME_FIELD_UNKNOWN, 0),
        Strategy::BackspaceType
    );
}

/// app không có preset → default_for_field theo caps (không crash).
#[test]
fn unknown_app_field_defaults() {
    let d = db();
    assert_eq!(
        resolve(&d, "com.unknown.app", IME_FIELD_EDITBOX, MAC_CAPS),
        Strategy::Preedit
    );
    assert_eq!(
        resolve(&d, "com.unknown.app", IME_FIELD_EDITBOX, 0),
        Strategy::BackspaceType
    );
}

/// editbox Finder → SelectionReplace (rename file — preset mac.finder.rename).
#[test]
fn finder_rename_editbox() {
    let d = db();
    assert_eq!(
        resolve(&d, "com.apple.finder", IME_FIELD_EDITBOX, MAC_CAPS),
        Strategy::SelectionReplace
    );
}
