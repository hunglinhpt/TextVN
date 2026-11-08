// SPDX-License-Identifier: GPL-3.0-or-later
//! P7 (vòng 57) — merge dữ liệu học từ adapter vào config tray.
//!
//! Adapter (TSF/IMK/engine) gọi `ime_style_profile_drain` định kỳ rồi gửi
//! `Message::StyleUpdate { json }` lên tray. JSON shape khớp output drain:
//! `{"words":[{word,count}..],"corrections":[{from,to,count}..],"rsi_delta":{..}}` (hoặc "{}").
//!
//! Merge qua `StyleProfileSettings::merge_deltas` (đã có sanitize + cap
//! 200/50/16KB) rồi persist + bump version → `ConfigReload` tự phát tới mọi
//! adapter (engine nhận style_words/style_corrections mới, personal prediction
//! sống lại với dữ liệu học được).
//!
//! **Số đo Dream RSI (0.2.74)** đi cùng mọi lượt drain nhưng KHÔNG được kéo theo ghi file +
//! `ConfigReload` mỗi 20 phím: reload làm engine nạp lại tuỳ chọn (reset từ đang gõ). Vì vậy
//! drain chỉ-có-số-đo được GOM trong tray và ghi tối đa mỗi [`RSI_FLUSH_EVERY`], không phát
//! reload; khi có từ/cặp thật cần gộp thì ghi kèm luôn phần số đo đang gom.
//!
//! G8: JSON chỉ chứa từ/cặp + count + bộ đếm — không parse trường nào khác;
//! lỗi parse → `Err`, tray bỏ im lặng (fail-open, không chết config).

use crate::svc::SvcManager;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use textvn_config::RsiTotals;

/// Khoảng tối thiểu giữa hai lần ghi file chỉ vì số đo Dream RSI.
pub const RSI_FLUSH_EVERY: Duration = Duration::from_secs(60);

/// Số đo đang gom chưa ghi + mốc bắt đầu gom.
static PENDING_RSI: Mutex<(RsiTotals, Option<Instant>)> = Mutex::new((
    RsiTotals {
        keys_typed: 0,
        accepts: 0,
        chars_saved: 0,
    },
    None,
));

/// Kết quả gộp: `Some(version)` = config đổi và cần phát `ConfigReload`; `None` = đã gom/ghi số
/// đo, KHÔNG cần reload.
pub type MergeOutcome = Option<u64>;

/// Merge JSON drain vào config hiện hành + persist. Không có entry hợp lệ hoặc người dùng đã tắt
/// tự học → không đụng config.
pub fn merge_from_json(svc: &SvcManager, json: &str) -> Result<MergeOutcome, String> {
    merge_at(svc, json, Instant::now())
}

/// Như [`merge_from_json`] với đồng hồ truyền vào (test không phải ngủ).
pub fn merge_at(svc: &SvcManager, json: &str, now: Instant) -> Result<MergeOutcome, String> {
    let cfg = svc.config();
    if !cfg.style_profile.effective_enabled() {
        return Err("người dùng đã tắt tự học".into());
    }
    let (words, corrections) =
        textvn_config::parse_style_drain(json).ok_or_else(|| "drain JSON hỏng".to_string())?;
    let rsi = textvn_config::parse_style_rsi_delta(json);
    let has_learning = !words.is_empty() || !corrections.is_empty();
    if !has_learning && rsi.is_zero() {
        return Err("drain rỗng".into());
    }

    let mut pending = PENDING_RSI.lock().unwrap_or_else(|e| e.into_inner());
    pending.0.add(&rsi);
    let started = *pending.1.get_or_insert(now);
    if !has_learning && now.saturating_duration_since(started) < RSI_FLUSH_EVERY {
        return Ok(None); // đang gom — chưa ghi, chưa reload
    }
    let take = std::mem::take(&mut pending.0);
    pending.1 = None;
    let result = svc.update_config(|c| {
        c.style_profile.merge_deltas(&words, &corrections);
        c.style_profile.rsi.add(&take);
    });
    match result {
        Ok(ver) => Ok(has_learning.then_some(ver)),
        Err(e) => {
            // Ghi lỗi: trả số đo về hàng đợi để lần sau thử lại (không mất bộ đếm).
            pending.0.add(&take);
            pending.1 = Some(started);
            Err(format!("persist lỗi: {:?}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc(tag: &str) -> (std::sync::Arc<SvcManager>, std::path::PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("textvn-stylemerge-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (SvcManager::new(Some(dir.clone())), dir)
    }

    const RSI_ONLY: &str = r#"{"words":[],"corrections":[],"rsi_delta":{"keys_typed":20,"accepts":1,"chars_saved":6}}"#;
    const WITH_WORD: &str = r#"{"words":[{"word":"blorptastic","count":1}],"corrections":[],"rsi_delta":{"keys_typed":10,"accepts":0,"chars_saved":0}}"#;

    /// Các test dùng chung `PENDING_RSI` tĩnh → chạy tuần tự trong MỘT test để không giẫm nhau.
    #[test]
    fn rsi_only_duoc_gom_khong_reload_va_ghi_kem_khi_co_tu_hoc() {
        let (svc, dir) = svc("flow");
        *PENDING_RSI.lock().unwrap() = (RsiTotals::default(), None);
        let t0 = Instant::now();

        // Mặc định BẬT: drain chỉ-có-số-đo được gom, không đổi config, không phát reload.
        assert_eq!(merge_at(&svc, RSI_ONLY, t0), Ok(None));
        assert_eq!(
            merge_at(&svc, RSI_ONLY, t0 + Duration::from_secs(10)),
            Ok(None)
        );
        assert!(svc.config().style_profile.rsi.is_zero(), "chưa ghi");

        // Quá 60 s: ghi (cộng dồn 3 lượt = 60 phím) nhưng vẫn KHÔNG reload.
        assert_eq!(
            merge_at(
                &svc,
                RSI_ONLY,
                t0 + RSI_FLUSH_EVERY + Duration::from_secs(1)
            ),
            Ok(None)
        );
        let rsi = svc.config().style_profile.rsi;
        assert_eq!((rsi.keys_typed, rsi.accepts, rsi.chars_saved), (60, 3, 18));

        // Có từ thật: ghi ngay, phát reload, và mang theo số đo của chính lượt đó.
        let ver = merge_at(&svc, WITH_WORD, t0 + Duration::from_secs(200)).unwrap();
        assert!(ver.is_some(), "từ mới → cần ConfigReload");
        let p = svc.config().style_profile;
        assert_eq!(p.words[0].word, "blorptastic");
        assert_eq!(p.rsi.keys_typed, 70);

        // Người dùng tắt (decided) → không gom, không ghi.
        svc.set_style_learning(false).unwrap();
        assert!(merge_at(&svc, WITH_WORD, t0).is_err());
        assert!(merge_at(&svc, RSI_ONLY, t0).is_err());
        // Bật lại + xoá dữ liệu học → sạch (giữ công tắt).
        svc.set_style_learning(true).unwrap();
        svc.clear_learned_style().unwrap();
        let p = svc.config().style_profile;
        assert!(p.words.is_empty() && p.rsi.is_zero() && p.effective_enabled());
        // Drain rỗng/hỏng → lỗi, không panic.
        assert!(merge_at(&svc, "{}", t0).is_err());
        assert!(merge_at(&svc, "hỏng", t0).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
