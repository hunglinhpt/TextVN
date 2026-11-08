// SPDX-License-Identifier: GPL-3.0-or-later
//! 0.2.74 — công tắt tự học qua `SettingsDoc` (đường chung của Linux C-API và macOS): mặc định bật,
//! `decided`, khoá ảo cho UI, xoá dữ liệu học, "Mặc định" giữ dữ liệu.

use textvn_config::{parse_config, DocKind, SettingsDoc};

const OLD_FILE: &str = r#"{"config_version":1,"style_profile":{"enabled":false,
 "words":[{"word":"kubernetes","count":3}],
 "corrections":[{"from":"nghanh","to":"ngành","count":1}],
 "rsi":{"keys_typed":60,"accepts":4,"chars_saved":40}}}"#;

fn doc(text: &str) -> SettingsDoc {
    SettingsDoc::from_json(text, DocKind::Config)
}

#[test]
fn file_cu_enabled_false_chua_decided_van_bat() {
    let d = doc(OLD_FILE);
    assert_eq!(d.get_bool("style_profile.effective_enabled"), Some(true));
    assert_eq!(d.get_u32("style_profile.words_count"), Some(1));
    assert_eq!(d.get_u32("style_profile.corrections_count"), Some(1));
    assert_eq!(d.get_u32("style_profile.rsi_rate_pct"), Some(40)); // 40 / (40 + 60)
    assert_eq!(d.get_u32("style_profile.rsi.chars_saved"), Some(40));
    // Config trống: mặc định bật, chưa học gì.
    let empty = doc(r#"{"config_version":1}"#);
    assert_eq!(
        empty.get_bool("style_profile.effective_enabled"),
        Some(true)
    );
    assert_eq!(empty.get_u32("style_profile.words_count"), Some(0));
}

#[test]
fn bam_cong_tac_ghi_decided_va_ton_trong_lua_chon() {
    let mut d = doc(OLD_FILE);
    d.set_bool("style_profile.enabled", false).unwrap();
    assert_eq!(d.get_bool("style_profile.effective_enabled"), Some(false));
    assert_eq!(
        d.get_u32("style_profile.words_count"),
        Some(1),
        "tắt không xoá dữ liệu"
    );
    let cfg = parse_config(&d.to_json()).unwrap();
    assert!(cfg.style_profile.decided && !cfg.style_profile.effective_enabled());
    d.set_bool("style_profile.enabled", true).unwrap();
    assert_eq!(d.get_bool("style_profile.effective_enabled"), Some(true));
    // Sai kiểu bị từ chối, tài liệu giữ nguyên.
    assert!(d.set_str("style_profile.enabled", "x").is_err());
    assert!(
        d.set_bool("style_profile.words_count", true).is_err(),
        "khoá ảo không ghi được"
    );
}

#[test]
fn xoa_du_lieu_hoc_giu_cong_tac_va_khoa_khac() {
    let mut d = doc(r#"{"config_version":1,"auto_capitalize":false,"khoa_la":7,
 "style_profile":{"enabled":false,"decided":true,
 "words":[{"word":"kubernetes","count":3}],"corrections":[],
 "rsi":{"keys_typed":1,"accepts":1,"chars_saved":1}}}"#);
    d.set_bool("style_profile.clear_learned", true).unwrap();
    assert_eq!(d.get_u32("style_profile.words_count"), Some(0));
    assert_eq!(d.get_u32("style_profile.rsi_rate_pct"), Some(0));
    assert_eq!(
        d.get_bool("style_profile.effective_enabled"),
        Some(false),
        "công tắt giữ nguyên"
    );
    let json: serde_json::Value = serde_json::from_str(&d.to_json()).unwrap();
    assert_eq!(json["khoa_la"], 7);
    assert_eq!(json["auto_capitalize"], false);
    // `false` là no-op, kiểu khác bị từ chối.
    d.set_bool("style_profile.clear_learned", false).unwrap();
    assert!(d.set_str("style_profile.clear_learned", "x").is_err());
}

#[test]
fn mac_dinh_giu_du_lieu_hoc_va_tra_cong_tac_ve_bat() {
    let mut d = doc(r#"{"config_version":1,"quick_telex":true,
 "style_profile":{"enabled":false,"decided":true,
 "words":[{"word":"kubernetes","count":3}],"corrections":[],
 "rsi":{"keys_typed":60,"accepts":4,"chars_saved":40}}}"#);
    d.reset_defaults();
    let cfg = parse_config(&d.to_json()).unwrap();
    assert!(!cfg.quick_telex, "tuỳ chọn gõ về mặc định");
    assert_eq!(cfg.style_profile.words.len(), 1, "dữ liệu học được giữ");
    assert_eq!(cfg.style_profile.rsi.chars_saved, 40);
    assert!(cfg.style_profile.effective_enabled() && !cfg.style_profile.decided);
}
