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
fn tat_that_va_dau_vao_hong_khong_ghi_gi() {
    // Người dùng đã chủ động tắt (nút Cài đặt ghi `decided:true`) → không ghi gì.
    assert!(merge_style_drain(
        r#"{"config_version":1,"style_profile":{"enabled":false,"decided":true}}"#,
        DRAIN
    )
    .is_none());
    assert!(
        merge_style_drain("{}", DRAIN).is_none(),
        "thiếu config_version"
    );
    assert!(merge_style_drain("không phải json", DRAIN).is_none());
    assert!(merge_style_drain(ON, "{}").is_none(), "drain rỗng");
    assert!(merge_style_drain(ON, "hỏng").is_none());
    assert!(merge_style_drain(ON, r#"{"rsi":{"keys_typed":1}}"#).is_none());
}

/// 0.2.74: tự học BẬT mặc định. File cũ (≤ 0.2.73) ghi `enabled:false` do tự lưu mặc định, chưa có
/// `decided` ⇒ vẫn coi là bật; config thiếu khối style_profile cũng bật.
#[test]
fn bat_mac_dinh_ke_ca_file_cu_ghi_enabled_false() {
    for cfg in [
        r#"{"config_version":1}"#,
        r#"{"config_version":1,"style_profile":{"enabled":false}}"#,
        r#"{"config_version":1,"style_profile":{"enabled":false,"words":[],"corrections":[],"max_total_bytes":16384}}"#,
    ] {
        assert!(
            parse_config(cfg).unwrap().style_profile.effective_enabled(),
            "{cfg}"
        );
        assert!(merge_style_drain(cfg, DRAIN).is_some(), "{cfg}");
    }
}

#[test]
fn rsi_delta_cong_don_khong_dem_dup() {
    const D1: &str = r#"{"words":[],"corrections":[],"rsi":{"keys_typed":999},"rsi_delta":{"keys_typed":10,"accepts":2,"chars_saved":7}}"#;
    let once = merge_style_drain(ON, D1).expect("chỉ có số đo vẫn phải ghi");
    let twice = merge_style_drain(&once, D1).unwrap();
    let rsi = parse_config(&twice).unwrap().style_profile.rsi;
    // Cộng dồn từ `rsi_delta`, KHÔNG lấy khối `rsi` (cộng dồn trong phiên — sẽ đếm đúp).
    assert_eq!((rsi.keys_typed, rsi.accepts, rsi.chars_saved), (20, 4, 14));
    assert_eq!(rsi.savings_rate_pct(), 41); // 14 / (14 + 20)
                                            // Drain cũ không có `rsi_delta` → 0, không hỏng việc gộp từ.
    assert!(textvn_config::parse_style_rsi_delta(DRAIN).is_zero());
    assert!(textvn_config::parse_style_rsi_delta("hỏng").is_zero());
}

#[test]
fn rsi_khong_tran_so() {
    let mut t = textvn_config::RsiTotals {
        keys_typed: u64::MAX,
        accepts: u64::MAX,
        chars_saved: u64::MAX,
    };
    t.add(&textvn_config::RsiTotals {
        keys_typed: 5,
        accepts: 5,
        chars_saved: 5,
    });
    assert_eq!(t.keys_typed, u64::MAX);
    assert_eq!(t.savings_rate_pct(), 50);
    assert_eq!(textvn_config::RsiTotals::default().savings_rate_pct(), 0);
}

#[test]
fn drain_chi_doc_tu_va_cap() {
    let (words, corrections) = parse_style_drain(DRAIN).unwrap();
    assert_eq!(words, vec![("kubernetes".to_string(), 3)]);
    assert_eq!(corrections.len(), 1);
    assert!(parse_style_drain("[1,2]").is_none());
}
