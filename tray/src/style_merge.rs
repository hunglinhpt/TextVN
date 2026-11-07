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

use crate::svc::SvcManager;

/// Merge JSON drain vào config hiện hành + persist + trả version mới.
/// Không có entry hợp lệ hoặc style_profile chưa bật → không đụng config.
pub fn merge_from_json(svc: &SvcManager, json: &str) -> Result<u64, String> {
    let cfg = svc.config();
    if !cfg.style_profile.enabled {
        return Err("style profile chưa bật (opt-in G8)".into());
    }
    let (words, corrections) =
        textvn_config::parse_style_drain(json).ok_or_else(|| "drain JSON hỏng".to_string())?;
    if words.is_empty() && corrections.is_empty() {
        return Err("drain rỗng".into());
    }
    svc.update_config(|c| {
        c.style_profile.merge_deltas(&words, &corrections);
    })
    .map_err(|e| format!("persist lỗi: {:?}", e))
}
