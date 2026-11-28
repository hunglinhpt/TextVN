// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `assist` dùng chung 3 OS (P0-6 §5 hàng *assist*, task SF-5) — luật QUYẾT ĐỊNH của Trợ lý
//! ngữ pháp. Chuyển nguyên văn từ TSF `assist.rs`/`panel_idle.rs` (bản đã có test đầy đủ); Linux
//! (`lc_assist_idle_arms`, `lc_assist_live_key`) và macOS (`assistIdleArms`) dùng qua C ABI.
//!
//! Luật (test ở `session/tests/assist.rs` + bộ test TSF dùng lại qua `pub use`):
//! - **D11/AS-022**: trigger theo phím — `. ! ?` kết thúc câu (điểm hiển thị), `space , ; :` điểm
//!   kiểm tra im lặng.
//! - **R2-154/155**: idle 1 s — MỌI phím gõ chữ/dấu câu/xoá (kể cả khi đang soạn) hẹn lại timer khi
//!   tiếng Việt bật; Enter/Tab/Esc/Ctrl+Space/điều hướng không đổi câu nên không hẹn lại.
//! - **R-5/R-6**: Mode 2 tự sửa cả câu CHỈ khi `assist.mode == autocorrect` (mức an toàn); điểm hiển thị
//!   (Mode 1) bật khi `mode != off`; tự sửa chỉ ở trigger kết thúc câu.
//! - **R2-129/P9**: phím khi bảng gợi ý SỐNG đang hiện không chiếm phím gõ (gõ tiếp ⇒ giữ, adapter ẩn
//!   theo P9), `Esc` đóng, `Alt+1..3` áp gợi ý, `Ctrl+Shift+G` gọi lại, phím khác đóng rồi đi tiếp.

use crate::compose::KeyKind;
use textvn_config::AssistMode;
use textvn_core::keymap::{vk, MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_SHIFT, MOD_SUPER};

/// Loại trigger điểm kiểm tra assist (D11 · P4-0 §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistTrigger {
    /// Không phải trigger.
    None,
    /// Ranh giới từ/vế: `space` `,` `;` `:` — điểm kiểm tra silent.
    Word,
    /// Kết thúc câu: `.` `!` `?` — điểm kiểm tra + điểm hiển thị.
    Sentence,
}

/// Phân loại phím trigger. Chỉ nhận phím **sinh ký tự thật** (`KeyKind::Char`) — Enter/Tab/
/// Backspace gây reset buffer, không phải check.
#[must_use]
pub fn assist_trigger(key: KeyKind) -> AssistTrigger {
    match key {
        KeyKind::Char('.' | '!' | '?') => AssistTrigger::Sentence,
        KeyKind::Char(' ' | ',' | ';' | ':') => AssistTrigger::Word,
        _ => AssistTrigger::None,
    }
}

/// R2-154/155: phím nào (re)lên lịch timer idle — MỌI phím gõ chữ/dấu câu/xoá, kể cả khi đang
/// composing, khi tiếng Việt bật.
#[must_use]
pub fn idle_arms(key: KeyKind, vi_enabled: bool) -> bool {
    vi_enabled && matches!(key, KeyKind::Char(_) | KeyKind::Backspace)
}

/// **Ngữ nghĩa Mode 2 duy nhất cho 3 OS** (R-5 · P4-0 §4.3): tự sửa cả câu bật khi
/// `assist.mode == "autocorrect"` — luôn ở mức **an toàn** (chỉ luật objective — D7). Khoá
/// `assist.auto_fix` còn trong config cho tương thích nhưng KHÔNG còn quyết định gì.
#[must_use]
pub fn mode2_auto_fix_enabled(mode: AssistMode) -> bool {
    mode == AssistMode::Autocorrect
}

/// **Điểm hiển thị (Mode 1) bật khi `assist.mode != "off"`** (R-6 · P4-0 §4.3): ở Mode 2, lỗi
/// **mơ hồ** vẫn phải hiện candidate (chỉ lỗi objective mới tự sửa).
#[must_use]
pub fn mode1_display_enabled(mode: AssistMode) -> bool {
    mode != AssistMode::Off
}

/// Có nên tự sửa cả câu (A3 / Mode 2) tại trigger này (R-5): Mode 2 đầy đủ VÀ trigger kết thúc
/// câu (`.` `!` `?`); space/,/: chỉ là điểm kiểm tra silent (D11).
#[must_use]
pub fn should_auto_fix_sentence(
    trigger: AssistTrigger,
    assist_enabled: bool,
    auto_fix_enabled: bool,
) -> bool {
    assist_enabled && auto_fix_enabled && trigger == AssistTrigger::Sentence
}

/// Có hiện overlay không: chỉ ở điểm hiển thị (kết thúc câu — D11), assist bật (D20), có lỗi.
#[must_use]
pub fn should_show_overlay(
    trigger: AssistTrigger,
    assist_enabled: bool,
    issue_count: usize,
) -> bool {
    assist_enabled && trigger == AssistTrigger::Sentence && issue_count > 0
}

/// Phím tác động lên bảng gợi ý đang hiện (P4-0 §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    /// `↑` (hoặc `Shift+Tab`) — lỗi trước.
    Previous,
    /// `↓` (hoặc `Tab`) — lỗi kế.
    Next,
    /// `Enter` — áp dụng gợi ý của lỗi đang chọn.
    Apply,
    /// `Alt+1..3` — chọn + áp dụng gợi ý thứ `n` (0-based).
    Suggestion(usize),
    /// `Esc` — đóng, không sửa.
    Close,
    /// `Ctrl+Shift+G` — gọi lại bảng gợi ý gần nhất.
    Recall,
    /// Phím khác — đóng bảng rồi phím đi tiếp.
    Dismiss,
    /// Bảng sống: phím gõ tiếp — không chiếm phím (adapter ẩn theo P9 và hẹn lại idle).
    Keep,
}

/// Loại bảng gợi ý đang hiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    /// Tự hiện theo câu (R2-129: không chiếm phím gõ).
    Live,
    /// Bôi đen + phím tắt.
    Selection,
    /// Gọi bằng phím tắt (Ctrl+Space/Option+Space).
    OnDemand,
}

/// Phím (VK canonical) tác động lên bảng gợi ý tương tác.
#[must_use]
pub fn overlay_action(vk: u32, mods: u32) -> OverlayAction {
    let mods = mods & !MOD_CAPS;
    let alt = mods & MOD_ALT != 0;
    let ctrl = mods & MOD_CTRL != 0;
    let sup = mods & MOD_SUPER != 0;
    let shift = mods & MOD_SHIFT != 0;

    if ctrl && shift && !alt && !sup && vk == VK_G {
        return OverlayAction::Recall;
    }
    if alt && !ctrl && !sup && !shift {
        return match vk {
            0x31..=0x33 => OverlayAction::Suggestion(vk as usize - 0x31),
            0x61..=0x63 => OverlayAction::Suggestion(vk as usize - 0x61),
            _ => OverlayAction::Dismiss,
        };
    }
    if ctrl || alt || sup {
        return OverlayAction::Dismiss;
    }
    match vk {
        vk::UP => OverlayAction::Previous,
        vk::TAB if shift => OverlayAction::Previous,
        vk::DOWN | vk::TAB => OverlayAction::Next,
        vk::RETURN if !shift => OverlayAction::Apply,
        vk::ESCAPE => OverlayAction::Close,
        _ => OverlayAction::Dismiss,
    }
}

/// VK `G` (`Ctrl+Shift+G` gọi lại bảng gợi ý).
pub const VK_G: u32 = 0x47;

/// Phím gõ chữ (VK canonical, không Ctrl/Alt/Win): chữ, số, cách, dấu câu OEM, bàn phím số, xoá.
#[must_use]
pub fn is_typing_key(vk: u32, mods: u32) -> bool {
    if mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0 {
        return false;
    }
    matches!(
        vk,
        0x08 | 0x20 | 0x30..=0x39 | 0x41..=0x5A | 0x60..=0x6F | 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2
    )
}

/// Phím theo loại bảng: bảng SỐNG không chiếm phím gõ (R2-129) — chỉ `Esc`, `Alt+1..3`,
/// `Ctrl+Shift+G` tác động; phím gõ ⇒ `Keep`; phím khác ⇒ `Dismiss`.
#[must_use]
pub fn overlay_action_for(vk: u32, mods: u32, kind: PopupKind) -> OverlayAction {
    match kind {
        PopupKind::OnDemand => overlay_action(vk, mods),
        PopupKind::Selection if vk == vk::TAB => OverlayAction::Dismiss,
        PopupKind::Selection => overlay_action(vk, mods),
        PopupKind::Live => match overlay_action(vk, mods) {
            a @ (OverlayAction::Recall | OverlayAction::Close | OverlayAction::Suggestion(_)) => a,
            _ if is_typing_key(vk, mods) => OverlayAction::Keep,
            _ => OverlayAction::Dismiss,
        },
    }
}

/// Như [`overlay_action_for`] cho adapter không có VK OEM tin được (IBus/Fcitx5 keysym → `(vk, ch)`):
/// phím gõ = có ký tự in được `ch` hoặc Backspace (thay bảng VK OEM của Windows).
#[must_use]
pub fn live_action_vk_ch(vk: u32, ch: u32, mods: u32) -> OverlayAction {
    match overlay_action(vk, mods) {
        a @ (OverlayAction::Recall | OverlayAction::Close | OverlayAction::Suggestion(_)) => a,
        _ if mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) == 0
            && (vk == vk::BACK || matches!(KeyKind::classify(vk, ch), KeyKind::Char(_))) =>
        {
            OverlayAction::Keep
        }
        _ => OverlayAction::Dismiss,
    }
}
