// SPDX-License-Identifier: GPL-3.0-or-later
//! Vòng 59 — **Dream consolidation** (F-P10-3) của khối `style_profile`
//! trong `config.v1.json`:
//!
//! 1. **Prune mục count-1** (nhiễu một-lần — typo lọt validator): hồ sơ
//!    vượt `STYLE_ONEOFF_TRIGGER` (120) từ chỉ giữ `STYLE_ONEOFF_KEEP` (60)
//!    mục count-1; count ≥ 2 luôn giữ; hồ sơ nhỏ KHÔNG cắt.
//! 2. **EMA-decay**: count ≥ `STYLE_DECAY_AT` (256) → giảm nửa toàn hồ sơ
//!    lúc persist (khớp engine halve trong phiên) — thói quen mới không bị
//!    đè bởi count cũ.
//! 3. **Idempotent**: sanitize hai lần = cùng kết quả.
//!
//! G8: chỉ từ + count, không nội dung khác.

use textvn_config::{StyleProfileSettings, StyleWord};

fn words(v: &[(&str, u32)]) -> Vec<StyleWord> {
    v.iter()
        .map(|(w, c)| StyleWord {
            word: (*w).to_string(),
            count: *c,
        })
        .collect()
}

/// Hồ sơ trưởng thành (vượt 120 từ): count-1 bị prune về 60, count ≥ 2 giữ hết.
#[test]
fn prune_mot_lan_khi_ho_so_truong_thanh() {
    // 30 từ count-5 + 110 từ count-1 = 140 > trigger 120.
    let mut v: Vec<(String, u32)> = (0..30u32)
        .map(|i| {
            (
                format!(
                    "core{}{}",
                    char::from_u32(97 + ((i / 26) % 26)).unwrap(),
                    char::from_u32(97 + (i % 26)).unwrap()
                ),
                5,
            )
        })
        .collect();
    for i in 0..110u32 {
        v.push((
            format!(
                "noise{}{}{}",
                char::from_u32(97 + ((i / 676) % 26)).unwrap(),
                char::from_u32(97 + ((i / 26) % 26)).unwrap(),
                char::from_u32(97 + (i % 26)).unwrap()
            ),
            1,
        ));
    }
    let mut prof = StyleProfileSettings {
        words: words(&v.iter().map(|(w, c)| (w.as_str(), *c)).collect::<Vec<_>>()),
        ..Default::default()
    };
    prof.sanitize();
    let oneoff = prof.words.iter().filter(|w| w.count <= 1).count();
    let multi = prof.words.iter().filter(|w| w.count >= 2).count();
    assert_eq!(multi, 30, "count >= 2 phải giữ hết");
    assert!(oneoff <= 60, "count-1 phải prune về ≤ 60, còn {oneoff}");
    assert_eq!(prof.words.len(), multi + oneoff);
}

/// Hồ sơ nhỏ (≤ trigger): KHÔNG cắt — người dùng mới cần mục count-1 để học.
#[test]
fn ho_so_nho_khong_prune() {
    let v: Vec<(String, u32)> = (0..90)
        .map(|i| {
            (
                format!(
                    "new{}{}{}",
                    char::from_u32(97 + ((i / 676) % 26) as u32).unwrap(),
                    char::from_u32(97 + ((i / 26) % 26) as u32).unwrap(),
                    char::from_u32(97 + (i % 26) as u32).unwrap()
                ),
                1,
            )
        })
        .collect();
    let mut prof = StyleProfileSettings {
        words: words(&v.iter().map(|(w, c)| (w.as_str(), *c)).collect::<Vec<_>>()),
        ..Default::default()
    };
    prof.sanitize();
    assert_eq!(prof.words.len(), 90, "90 count-1 ≤ trigger không được cắt");
}

/// EMA-decay: count ≥ 256 → giảm nửa toàn hồ sơ (min 1) — idempotent.
#[test]
fn decay_count_lon_giam_nua() {
    let mut prof = StyleProfileSettings {
        words: words(&[("stale", 512), ("mature", 300), ("fresh", 4), ("tiny", 1)]),
        ..Default::default()
    };
    prof.sanitize();
    assert_eq!(prof.words[0].count, 256, "512/2 = 256");
    assert_eq!(prof.words[1].count, 150);
    assert_eq!(prof.words[2].count, 2);
    assert_eq!(prof.words[3].count, 1, "min 1");
    // Idempotent về hành vi decay: lần sanitize kế tiếp halve tiếp (256 ≥ 256)
    // — hội tụ, không loop vô hạn; count nhỏ giữ nguyên.
    prof.sanitize();
    assert_eq!(prof.words[0].count, 128);
    assert_eq!(prof.words[3].count, 1);
}

/// Merge delta (đường drain thật) + consolidation: profile 150 count-1 cũ +
/// merge 10 count-1 mới → prune giữ lượng ổn định, không đếm đúp.
#[test]
fn merge_delta_va_prune() {
    let v: Vec<(String, u32)> = (0..150)
        .map(|i| {
            (
                format!(
                    "old{}{}{}",
                    char::from_u32(97 + ((i / 676) % 26) as u32).unwrap(),
                    char::from_u32(97 + ((i / 26) % 26) as u32).unwrap(),
                    char::from_u32(97 + (i % 26) as u32).unwrap()
                ),
                1,
            )
        })
        .collect();
    let mut prof = StyleProfileSettings {
        words: words(&v.iter().map(|(w, c)| (w.as_str(), *c)).collect::<Vec<_>>()),
        ..Default::default()
    };
    prof.sanitize();
    assert!(
        prof.words.len() <= 60,
        "150 count-1 → giữ 60: {}",
        prof.words.len()
    );
    // Merge delta trùng → cộng count; sanitize lại vẫn ổn định.
    prof.merge_deltas(&[("oldaaa".to_string(), 2)], &[]);
    let oldaaa = prof.words.iter().find(|w| w.word == "oldaaa");
    assert!(oldaaa.is_some_and(|w| w.count == 3), "1 + 2 = 3");
}
