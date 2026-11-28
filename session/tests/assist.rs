// SPDX-License-Identifier: GPL-3.0-or-later
//! SF-5 · khối assist (P0-6 §5): luật quyết định của Trợ lý ngữ pháp dùng chung 3 OS.

use textvn_config::AssistMode;
use textvn_session::assist::{
    assist_trigger, idle_arms, live_action_vk_ch, mode1_display_enabled, mode2_auto_fix_enabled,
    overlay_action_for, should_auto_fix_sentence, should_show_overlay, AssistTrigger,
    OverlayAction, PopupKind,
};
use textvn_session::compose::KeyKind;

const MOD_SHIFT: u32 = 0x1;
const MOD_CTRL: u32 = 0x2;
const MOD_ALT: u32 = 0x4;

#[test]
fn d11_triggers() {
    for c in ['.', '!', '?'] {
        assert_eq!(assist_trigger(KeyKind::Char(c)), AssistTrigger::Sentence);
    }
    for c in [' ', ',', ';', ':'] {
        assert_eq!(assist_trigger(KeyKind::Char(c)), AssistTrigger::Word);
    }
    for k in [
        KeyKind::Char('a'),
        KeyKind::Char('"'),
        KeyKind::Enter,
        KeyKind::Backspace,
    ] {
        assert_eq!(assist_trigger(k), AssistTrigger::None, "{k:?}");
    }
}

#[test]
fn r2_154_idle_arms_on_every_text_or_delete_key_when_vi_on() {
    assert!(idle_arms(KeyKind::Char('a'), true));
    assert!(idle_arms(KeyKind::Char('.'), true));
    assert!(idle_arms(KeyKind::Backspace, true));
    for k in [
        KeyKind::Enter,
        KeyKind::Tab,
        KeyKind::Escape,
        KeyKind::CtrlSpace,
        KeyKind::Other,
    ] {
        assert!(!idle_arms(k, true), "{k:?}");
    }
    assert!(!idle_arms(KeyKind::Char('a'), false), "tiếng Việt tắt");
}

#[test]
fn r_5_mode_semantics() {
    assert!(mode2_auto_fix_enabled(AssistMode::Autocorrect));
    assert!(!mode2_auto_fix_enabled(AssistMode::Highlight));
    assert!(!mode2_auto_fix_enabled(AssistMode::Off));
    assert!(mode1_display_enabled(AssistMode::Highlight));
    assert!(
        mode1_display_enabled(AssistMode::Autocorrect),
        "R-6: Mode 2 vẫn hiện lỗi mơ hồ"
    );
    assert!(!mode1_display_enabled(AssistMode::Off));
    assert!(should_auto_fix_sentence(
        AssistTrigger::Sentence,
        true,
        true
    ));
    assert!(
        !should_auto_fix_sentence(AssistTrigger::Word, true, true),
        "D11: chỉ cuối câu"
    );
    assert!(!should_auto_fix_sentence(
        AssistTrigger::Sentence,
        true,
        false
    ));
    assert!(should_show_overlay(AssistTrigger::Sentence, true, 1));
    assert!(!should_show_overlay(AssistTrigger::Sentence, true, 0));
    assert!(!should_show_overlay(AssistTrigger::Word, true, 3));
}

#[test]
fn r2_129_live_popup_never_takes_typing_keys() {
    use OverlayAction::*;
    let live = |vk, mods| overlay_action_for(vk, mods, PopupKind::Live);
    assert_eq!(live(0x41, 0), Keep);
    assert_eq!(live(0x31, 0), Keep, "số trần = gõ số (giá 100)");
    assert_eq!(live(0x20, 0), Keep);
    assert_eq!(live(0x08, 0), Keep);
    assert_eq!(live(0x1B, 0), Close);
    assert_eq!(live(0x31, MOD_ALT), Suggestion(0));
    assert_eq!(live(0x63, MOD_ALT), Suggestion(2), "Alt+numpad 3");
    assert_eq!(live(0x47, MOD_CTRL | MOD_SHIFT), Recall);
    assert_eq!(live(0x0D, 0), Dismiss, "Enter đóng rồi đi tiếp");
    assert_eq!(live(0x09, 0), Dismiss);
    assert_eq!(live(0x26, 0), Dismiss);
    // Bảng gọi bằng phím tắt: điều hướng được.
    assert_eq!(overlay_action_for(0x28, 0, PopupKind::OnDemand), Next);
    assert_eq!(overlay_action_for(0x0D, 0, PopupKind::OnDemand), Apply);
    assert_eq!(overlay_action_for(0x09, 0, PopupKind::Selection), Dismiss);
}

/// Biến thể (vk, ch) cho IBus/Fcitx5 phải giữ đúng hành vi `lc_assist_live_key` hiện tại.
#[test]
fn r2_129_live_vk_ch_matches_linux_rules() {
    use OverlayAction::*;
    assert_eq!(live_action_vk_ch(0x41, 'a' as u32, 0), Keep);
    assert_eq!(live_action_vk_ch(0x31, '1' as u32, 0), Keep);
    assert_eq!(live_action_vk_ch(0x20, ' ' as u32, 0), Keep);
    assert_eq!(live_action_vk_ch(0xBE, '.' as u32, 0), Keep);
    assert_eq!(live_action_vk_ch(0x08, 0, 0), Keep, "BackSpace");
    assert_eq!(
        live_action_vk_ch(0, 0x1EA1, 0),
        Keep,
        "ạ (XKB vn, keysym Unicode)"
    );
    assert_eq!(live_action_vk_ch(0, '7' as u32, 0), Keep, "bàn phím số");
    assert_eq!(live_action_vk_ch(0x41, 'A' as u32, MOD_SHIFT), Keep);
    assert_eq!(live_action_vk_ch(0x1B, 0, 0), Close);
    assert_eq!(live_action_vk_ch(0x31, '1' as u32, MOD_ALT), Suggestion(0));
    assert_eq!(live_action_vk_ch(0x33, '3' as u32, MOD_ALT), Suggestion(2));
    assert_eq!(live_action_vk_ch(0x34, '4' as u32, MOD_ALT), Dismiss);
    assert_eq!(live_action_vk_ch(0x0D, '\n' as u32, 0), Dismiss);
    assert_eq!(live_action_vk_ch(0x09, '\t' as u32, 0), Dismiss);
    assert_eq!(live_action_vk_ch(0x26, 0, 0), Dismiss, "mũi tên");
    assert_eq!(
        live_action_vk_ch(0x41, 'a' as u32, MOD_CTRL),
        Dismiss,
        "chord"
    );
}
