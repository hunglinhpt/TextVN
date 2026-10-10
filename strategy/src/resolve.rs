// SPDX-License-Identifier: GPL-3.0-or-later
//! Quy tắc phân quyền chọn strategy — P0-3 §3.1 (đọc từ trên xuống, match là dừng).

use crate::rules_field::default_for_field;
use crate::{
    IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_SECURE, IME_STRATEGY_BACKSPACE_TYPE,
    IME_STRATEGY_FORWARD_AS_COMMIT, IME_STRATEGY_PASSTHROUGH, IME_STRATEGY_PREEDIT,
    IME_STRATEGY_SELECTION_REPLACE,
};

/// Strategy xuất chữ (bảng JSON: `Preedit | BackspaceType | SelectionReplace | ForwardAsCommit | Passthrough`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Preedit,
    BackspaceType,
    SelectionReplace,
    ForwardAsCommit,
    Passthrough,
}

impl Strategy {
    /// id phía FFI (`IME_STRATEGY_*`).
    pub fn id(self) -> i64 {
        match self {
            Strategy::Preedit => IME_STRATEGY_PREEDIT,
            Strategy::BackspaceType => IME_STRATEGY_BACKSPACE_TYPE,
            Strategy::SelectionReplace => IME_STRATEGY_SELECTION_REPLACE,
            Strategy::ForwardAsCommit => IME_STRATEGY_FORWARD_AS_COMMIT,
            Strategy::Passthrough => IME_STRATEGY_PASSTHROUGH,
        }
    }

    pub fn from_id(id: i64) -> Option<Strategy> {
        match id {
            IME_STRATEGY_PREEDIT => Some(Strategy::Preedit),
            IME_STRATEGY_BACKSPACE_TYPE => Some(Strategy::BackspaceType),
            IME_STRATEGY_SELECTION_REPLACE => Some(Strategy::SelectionReplace),
            IME_STRATEGY_FORWARD_AS_COMMIT => Some(Strategy::ForwardAsCommit),
            IME_STRATEGY_PASSTHROUGH => Some(Strategy::Passthrough),
            _ => None,
        }
    }
}

/// Đầu vào resolve — adapter điền từ context hiện tại + preset đã lookup.
pub struct ResolveInput {
    /// `ctx.secure` (P0-3 bước 1, S3)
    pub secure: bool,
    /// `ctx.enabled` — VN tắt / ignore list (bước 2)
    pub enabled: bool,
    /// Strategy user override từ `appdb.json` (bước 3)
    pub user_preset: Option<Strategy>,
    /// Strategy preset hệ thống match theo app_id + field_role (bước 4).
    /// Slice appdb chưa có — luôn `None`, giữ chỗ cho P0-3 §4.
    pub system_preset: Option<Strategy>,
    /// `ime_context_v1.hint`: adapter gợi ý strategy (0..4); -1 = tự resolve.
    /// P0-3 §3.1 không xếp hint vào 6 bước — coi như adapter đã resolve xong,
    /// engine tôn trọng nhưng **vẫn áp downgrade bắt buộc** theo caps.
    pub hint: i64,
    /// `ime_context_v1.field_role`
    pub field_role: u32,
    /// `ime_context_v1.caps`
    pub caps: u32,
}

/// Resolve theo đúng thứ tự P0-3 §3.1 + quy tắc downgrade bắt buộc.
pub fn resolve(inp: ResolveInput) -> Strategy {
    // Bước 1–2: gate an toàn — không preset nào được thắng. Ô có role `secure` là ô mật
    // khẩu kể cả khi adapter quên bật cờ `secure` (S3): hint/preset không được vượt qua.
    if inp.secure || inp.field_role == IME_FIELD_SECURE {
        return Strategy::Passthrough;
    }
    if !inp.enabled {
        return Strategy::Passthrough;
    }

    // Gợi ý từ adapter (P0-2 §1) — đứng trước preset vì adapter nhìn thấy field thật.
    let hinted = if inp.hint >= 0 {
        Strategy::from_id(inp.hint)
    } else {
        None
    };

    let chosen = hinted
        .or(inp.user_preset)
        .or(inp.system_preset)
        .unwrap_or_else(|| default_for_field(inp.field_role, inp.caps));

    downgrade(chosen, inp.caps)
}

/// Downgrade bắt buộc (P0-3 §3.1): adapter không có cap → không được chạy strategy tương ứng.
fn downgrade(s: Strategy, caps: u32) -> Strategy {
    match s {
        Strategy::Preedit if caps & IME_CAP_PREEDIT == 0 => Strategy::BackspaceType,
        Strategy::SelectionReplace if caps & IME_CAP_SELECTION == 0 => Strategy::BackspaceType,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        IME_CAP_FIELD_DETECT, IME_CAP_INJECT_VK, IME_CAP_PREEDIT, IME_FIELD_ADDRESS_BAR,
        IME_FIELD_BODY, IME_FIELD_SECURE, IME_FIELD_TERMINAL,
    };

    fn inp() -> ResolveInput {
        ResolveInput {
            secure: false,
            enabled: true,
            user_preset: None,
            system_preset: None,
            hint: -1,
            field_role: IME_FIELD_BODY,
            caps: IME_CAP_FIELD_DETECT | IME_CAP_INJECT_VK,
        }
    }

    #[test]
    fn step1_secure_beats_everything() {
        let mut i = inp();
        i.secure = true;
        i.hint = IME_STRATEGY_PREEDIT;
        i.user_preset = Some(Strategy::SelectionReplace);
        assert_eq!(resolve(i), Strategy::Passthrough);
    }

    #[test]
    fn step2_disabled_passthrough() {
        let mut i = inp();
        i.enabled = false;
        assert_eq!(resolve(i), Strategy::Passthrough);
    }

    #[test]
    fn step3_user_preset_beats_system() {
        let mut i = inp();
        i.user_preset = Some(Strategy::ForwardAsCommit);
        i.system_preset = Some(Strategy::SelectionReplace);
        assert_eq!(resolve(i), Strategy::ForwardAsCommit);
    }

    #[test]
    fn step5_field_defaults_with_caps() {
        let mut i = inp();
        i.field_role = IME_FIELD_ADDRESS_BAR;
        i.caps = IME_CAP_FIELD_DETECT | IME_CAP_INJECT_VK | IME_CAP_SELECTION;
        assert_eq!(resolve(i), Strategy::SelectionReplace);

        let mut i = inp();
        i.field_role = IME_FIELD_TERMINAL;
        assert_eq!(resolve(i), Strategy::ForwardAsCommit);

        let mut i = inp();
        i.field_role = IME_FIELD_BODY;
        i.caps = IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT;
        assert_eq!(resolve(i), Strategy::Preedit);
    }

    #[test]
    fn step6_fallback_backspace_type() {
        let i = inp(); // body, không preedit cap
        assert_eq!(resolve(i), Strategy::BackspaceType);
    }

    #[test]
    fn downgrade_without_cap() {
        let mut i = inp();
        i.hint = IME_STRATEGY_PREEDIT; // gợi ý Preedit nhưng caps thiếu
        assert_eq!(resolve(i), Strategy::BackspaceType);

        let mut i = inp();
        i.hint = IME_STRATEGY_SELECTION_REPLACE;
        i.caps = IME_CAP_FIELD_DETECT | IME_CAP_INJECT_VK; // thiếu SELECTION
        assert_eq!(resolve(i), Strategy::BackspaceType);
    }

    #[test]
    fn secure_field_role_passthrough() {
        let mut i = inp();
        i.field_role = IME_FIELD_SECURE;
        i.caps = IME_CAP_PREEDIT;
        assert_eq!(resolve(i), Strategy::Passthrough);
    }

    /// Role `secure` thắng cả hint lẫn preset dù cờ `secure` = 0 (S3 — trước đây
    /// preset app khớp mọi role biến ô mật khẩu thành Preedit).
    #[test]
    fn secure_role_beats_hint_and_presets_without_secure_flag() {
        let mut i = inp();
        i.field_role = IME_FIELD_SECURE;
        i.caps = IME_CAP_PREEDIT | IME_CAP_SELECTION;
        i.hint = IME_STRATEGY_PREEDIT;
        i.user_preset = Some(Strategy::SelectionReplace);
        i.system_preset = Some(Strategy::Preedit);
        assert_eq!(resolve(i), Strategy::Passthrough);
    }
}
