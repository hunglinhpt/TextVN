// SPDX-License-Identifier: GPL-3.0-or-later
//! P7 (vòng 57) — merge dữ liệu học từ adapter vào config tray.
//!
//! Adapter (TSF/IMK/engine) gọi `ime_style_profile_drain` định kỳ rồi gửi
//! `Message::StyleUpdate { json }` lên tray. JSON shape khớp output drain:
//! `{"words":[{word,count}..],"corrections":[{from,to,count}..]}` (hoặc "{}").
//!
//! Merge qua `StyleProfileSettings::merge_deltas` (đã có sanitize + cap
//! 200/50/16KB) rồi persist + bump version → `ConfigReload` tự phát tới mọi
//! adapter (engine nhận style_words/style_corrections mới, personal prediction
//! sống lại với dữ liệu học được).
//!
//! G8: JSON chỉ chứa từ/cặp + count — không parse bất kỳ trường nào khác;
//! lỗi parse → `Err`, tray bỏ im lặng (fail-open, không chết config).

use serde::Deserialize;

use crate::svc::SvcManager;

/// Shape trung gian khớp output drain của engine.
#[derive(Debug, Deserialize)]
struct DrainShape {
    #[serde(default)]
    words: Vec<DrainWord>,
    #[serde(default)]
    corrections: Vec<DrainCorrection>,
}

#[derive(Debug, Deserialize)]
struct DrainWord {
    word: String,
    count: u32,
}

#[derive(Debug, Deserialize)]
struct DrainCorrection {
    from: String,
    to: String,
    count: u32,
}

/// Merge JSON drain vào config hiện hành + persist + trả version mới.
/// Không có entry hợp lệ hoặc style_profile chưa bật → không đụng config.
pub fn merge_from_json(svc: &SvcManager, json: &str) -> Result<u64, String> {
    let cfg = svc.config();
    if !cfg.style_profile.enabled {
        return Err("style profile chưa bật (opt-in G8)".into());
    }
    let shape: DrainShape =
        serde_json::from_str(json).map_err(|e| format!("drain JSON hỏng: kind {e}"))?;
    let words: Vec<(String, u32)> = shape.words.into_iter().map(|w| (w.word, w.count)).collect();
    let corrections: Vec<(String, String, u32)> = shape
        .corrections
        .into_iter()
        .map(|c| (c.from, c.to, c.count))
        .collect();
    if words.is_empty() && corrections.is_empty() {
        return Err("drain rỗng".into());
    }
    svc.update_config(|c| {
        c.style_profile.merge_deltas(&words, &corrections);
    })
    .map_err(|e| format!("persist lỗi: {:?}", e))
}
