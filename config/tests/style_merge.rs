// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-152 — gộp dữ liệu học vào `config.json` (đường lưu của macOS/Linux).

use textvn_config::{merge_style_drain, parse_config, parse_style_drain};

const ON: &str =
    r#"{"config_version":1,"auto_capitalize":false,"khoa_la":1,"style_profile":{"enabled":true}}"#;
const DRAIN: &str = r#"{"words":[{"word":"kubernetes","count":3}],"corrections":[{"from":"nghanh","to":"ngành","count":1}],"rsi":{"keys_typed":5,"accepts":1,"chars_saved":2,"savings_rate_pct":28}}"#;

#[test]
fn gop_va_giu_nguyen_khoa_khac() {
    let merged = merge_style_drain(ON, DRAIN).expect("phải gộp được");
    let doc: serde_json::Value = serde_json::from_str(&merged).unwrap();
    assert_eq!(doc["auto_capitalize"], false);
    assert_eq!(doc["khoa_la"], 1, "khoá không thuộc schema phải còn");
    let cfg = parse_config(&merged).expect("kết quả phải là config hợp lệ");
    assert_eq!(cfg.style_profile.words[0].word, "kubernetes");
    assert_eq!(cfg.style_profile.words[0].count, 3);
    assert_eq!(cfg.style_profile.corrections[0].to, "ngành");
}

#[test]
fn gop_hai_lan_cong_don() {
    let once = merge_style_drain(ON, DRAIN).unwrap();
    let twice = merge_style_drain(&once, DRAIN).unwrap();
    let cfg = parse_config(&twice).unwrap();
    assert_eq!(cfg.style_profile.words.len(), 1);
    assert_eq!(cfg.style_profile.words[0].count, 6);
}

#[test]
fn opt_in_va_dau_vao_hong_khong_ghi_gi() {
    assert!(merge_style_drain(
        r#"{"config_version":1,"style_profile":{"enabled":false}}"#,
        DRAIN
    )
    .is_none());
    assert!(
        merge_style_drain("{}", DRAIN).is_none(),
        "mặc định tắt (G8)"
    );
    assert!(merge_style_drain("không phải json", DRAIN).is_none());
    assert!(merge_style_drain(ON, "{}").is_none(), "drain rỗng");
    assert!(merge_style_drain(ON, "hỏng").is_none());
    assert!(merge_style_drain(ON, r#"{"rsi":{"keys_typed":1}}"#).is_none());
}

#[test]
fn drain_chi_doc_tu_va_cap() {
    let (words, corrections) = parse_style_drain(DRAIN).unwrap();
    assert_eq!(words, vec![("kubernetes".to_string(), 3)]);
    assert_eq!(corrections.len(), 1);
    assert!(parse_style_drain("[1,2]").is_none());
}
