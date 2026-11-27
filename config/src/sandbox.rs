// SPDX-License-Identifier: GPL-3.0-or-later
//! R2-26 — bản sao cấu hình cho tiến trình **sandbox** Windows (AppContainer / LPAC: ô tìm kiếm
//! Start, Calculator, ứng dụng Store). Thuần, không I/O — tray ghi, DLL TSF đọc
//! (`docs/specs/sandbox-appcontainer.md`).
//!
//! Tiến trình AppContainer không đọc được `%APPDATA%\TextVN\config.json` (đo thật: lỗi 5). Tray
//! ghi `%APPDATA%\TextVN\sandbox\config.json` vào thư mục có ACE đọc cho `ALL APPLICATION
//! PACKAGES` — **mọi** ứng dụng Store của người dùng đọc được thư mục đó, nên bản sao không bao
//! giờ được chứa nội dung người dùng tự soạn hoặc đã học (G8/S2):
//!
//! - giữ **danh sách cho phép** [`SANDBOX_KEEP_KEYS`] (tuỳ chọn gõ, không phải nội dung);
//! - bỏ [`SANDBOX_STRIP_KEYS`] (gõ tắt, emoji riêng, từ tiếng Anh tự thêm);
//! - `style_profile` chỉ giữ 2 công tắc `enabled`/`decided` (bỏ từ, cặp sửa, tổng Dream RSI);
//! - khoá lạ không có trong 2 danh sách bị **bỏ** (an toàn mặc định: tính năng mới chứa nội
//!   dung người dùng không tự lọt ra; test `sandbox_keys_cover_schema_and_code` buộc phân loại).

use serde_json::{Map, Value};

use crate::{parse_style_rsi_delta, RsiTotals};

/// Khoá cấp gốc chép nguyên sang bản sao sandbox (tuỳ chọn hành vi, không phải nội dung).
pub const SANDBOX_KEEP_KEYS: [&str; 21] = [
    "config_version",
    "enabled",
    "method",
    "diacritic_style",
    "free_marking",
    "auto_restore_english",
    "auto_capitalize",
    "macro_trigger",
    "allow_macro_when_vi_off",
    "output_charset",
    "show_dialog_on_startup",
    "quick_telex",
    "suggest",
    "math",
    "assist",
    "hotkeys",
    "ignore_apps",
    "app_overrides",
    "secure_fields",
    "diagnostics",
    "updates",
];

/// Khoá chứa nội dung người dùng tự soạn — không bao giờ vào bản sao sandbox.
pub const SANDBOX_STRIP_KEYS: [&str; 3] = ["macros", "emoji", "english_words"];

/// Trong `style_profile` chỉ giữ công tắc tự học (tôn trọng lựa chọn tắt tự học của người dùng).
pub const SANDBOX_STYLE_PROFILE_KEEP: [&str; 2] = ["enabled", "decided"];

/// Bản sao `config.json` cho tiến trình sandbox. `None` khi `raw` không phải JSON object
/// (tray khi đó ghi bản sao từ cấu hình đã nạp trong bộ nhớ).
pub fn sandbox_config_json(raw: &str) -> Option<String> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let src = value.as_object()?;
    let mut out = Map::new();
    for key in SANDBOX_KEEP_KEYS {
        if let Some(v) = src.get(key) {
            out.insert(key.to_string(), v.clone());
        }
    }
    if let Some(sp) = src.get("style_profile").and_then(Value::as_object) {
        let mut keep = Map::new();
        for key in SANDBOX_STYLE_PROFILE_KEEP {
            if let Some(v) = sp.get(key) {
                keep.insert(key.to_string(), v.clone());
            }
        }
        out.insert("style_profile".to_string(), Value::Object(keep));
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(out)).ok()?;
    text.push('\n');
    Some(text)
}

/// Trần mỗi bộ đếm Dream RSI trong MỘT drain từ tiến trình sandbox (drain thật gửi theo lô vài
/// chục giây — vài nghìn phím là cùng). Ứng dụng Store độc hại không thổi phồng được số đo.
pub const SANDBOX_RSI_DELTA_CAP: u64 = 100_000;

/// Drain tự học từ tiến trình sandbox → chỉ giữ bộ đếm Dream RSI (G19), bỏ mọi từ/cặp sửa (nội
/// dung người dùng gõ — G8; cũng chặn ứng dụng sandbox bơm từ vào hồ sơ học). Mỗi bộ đếm bị
/// chặn trần [`SANDBOX_RSI_DELTA_CAP`]. `None` khi không còn gì để gửi.
pub fn style_drain_rsi_only(json: &str) -> Option<String> {
    let mut rsi: RsiTotals = parse_style_rsi_delta(json);
    rsi.keys_typed = rsi.keys_typed.min(SANDBOX_RSI_DELTA_CAP);
    rsi.accepts = rsi.accepts.min(SANDBOX_RSI_DELTA_CAP);
    rsi.chars_saved = rsi.chars_saved.min(SANDBOX_RSI_DELTA_CAP);
    if rsi.is_zero() {
        return None;
    }
    serde_json::to_string(&serde_json::json!({
        "words": [],
        "corrections": [],
        "rsi_delta": rsi,
    }))
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET_ADDR: &str = "12 Lê Lợi, 0912345678";

    fn full_config() -> String {
        format!(
            r#"{{"config_version":1,"method":"vni","diacritic_style":"old","auto_capitalize":true,
            "macros":[{{"trigger":"dc","expand":"{SECRET_ADDR}"}}],
            "emoji":[{{"trigger":"tim","emoji":"❤"}}],
            "english_words":["Khanhvy"],
            "style_profile":{{"enabled":false,"decided":true,
               "words":{{"ngânhàng":3}},"corrections":[{{"from":"ngan","to":"ngân","count":2}}],
               "rsi":{{"keys_typed":10,"accepts":1,"chars_saved":4}}}},
            "hotkeys":{{"toggle_vi_en":"Ctrl+Shift+Space"}},
            "assist":{{"mode":"highlight","per_app":{{"zalo.exe":false}}}},
            "math":{{"enabled":true}},
            "future_user_notes":["bí mật"]}}"#
        )
    }

    #[test]
    fn sandbox_copy_never_contains_user_content() {
        let out = sandbox_config_json(&full_config()).unwrap();
        for leaked in [
            SECRET_ADDR,
            "❤",
            "Khanhvy",
            "ngânhàng",
            "ngân",
            "bí mật",
            "keys_typed",
        ] {
            assert!(
                !out.contains(leaked),
                "lọt `{leaked}` vào bản sao sandbox:\n{out}"
            );
        }
        let v: Value = serde_json::from_str(&out).unwrap();
        for k in SANDBOX_STRIP_KEYS {
            assert!(v.get(k).is_none(), "khoá `{k}` phải bị bỏ");
        }
        assert!(
            v.get("future_user_notes").is_none(),
            "khoá lạ phải bị bỏ (an toàn mặc định)"
        );
    }

    #[test]
    fn sandbox_copy_keeps_typing_options_and_learning_switch() {
        let out = sandbox_config_json(&full_config()).unwrap();
        let cfg = crate::parse_config(&out).expect("bản sao phải là config hợp lệ");
        assert_eq!(cfg.method, crate::Method::Vni);
        assert!(cfg.auto_capitalize);
        assert!(cfg.math.enabled);
        assert!(cfg.macros.is_empty() && cfg.english_words.is_empty());
        // Người dùng đã TẮT tự học → tiến trình sandbox cũng không học.
        assert!(!cfg.style_profile.effective_enabled());
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["hotkeys"]["toggle_vi_en"], "Ctrl+Shift+Space");
        assert_eq!(v["assist"]["per_app"]["zalo.exe"], false);
    }

    #[test]
    fn sandbox_copy_rejects_non_object() {
        assert!(sandbox_config_json("not json").is_none());
        assert!(sandbox_config_json("[1,2]").is_none());
        assert!(sandbox_config_json("{}").is_some());
    }

    /// Gate: mọi khoá trong schema + `Config` phải được PHÂN LOẠI (giữ / bỏ / style_profile) —
    /// thêm tính năng mới mà quên phân loại thì tiến trình sandbox lặng lẽ mất tuỳ chọn đó, hoặc
    /// (nếu ai đó đổi sang danh sách chặn) nội dung người dùng lọt ra.
    #[test]
    fn sandbox_keys_cover_schema_and_code() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/config.v1.schema.json")).unwrap();
        let code: Value =
            serde_json::from_str(&serde_json::to_string(&crate::Config::default()).unwrap())
                .unwrap();
        let mut keys: Vec<String> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        keys.extend(code.as_object().unwrap().keys().cloned());
        for key in keys {
            let classified = SANDBOX_KEEP_KEYS.contains(&key.as_str())
                || SANDBOX_STRIP_KEYS.contains(&key.as_str())
                || key == "style_profile";
            assert!(
                classified,
                "khoá config `{key}` chưa được phân loại cho bản sao sandbox (R2-26) — thêm vào \
                 SANDBOX_KEEP_KEYS nếu là tuỳ chọn, SANDBOX_STRIP_KEYS nếu chứa nội dung người dùng"
            );
        }
    }

    #[test]
    fn rsi_only_drops_words_and_caps_counters() {
        let drain = r#"{"words":[{"word":"mậtkhẩu","count":3}],
            "corrections":[{"from":"ngan","to":"ngân","count":1}],
            "rsi_delta":{"keys_typed":40,"accepts":2,"chars_saved":9}}"#;
        let out = style_drain_rsi_only(drain).unwrap();
        assert!(!out.contains("mậtkhẩu") && !out.contains("ngân"));
        let (words, corrections) = crate::parse_style_drain(&out).unwrap();
        assert!(words.is_empty() && corrections.is_empty());
        let rsi = parse_style_rsi_delta(&out);
        assert_eq!((rsi.keys_typed, rsi.accepts, rsi.chars_saved), (40, 2, 9));

        let huge = r#"{"rsi_delta":{"keys_typed":99999999999,"accepts":5,"chars_saved":1}}"#;
        let rsi = parse_style_rsi_delta(&style_drain_rsi_only(huge).unwrap());
        assert_eq!(rsi.keys_typed, SANDBOX_RSI_DELTA_CAP);

        assert!(style_drain_rsi_only(r#"{"words":[{"word":"a","count":1}]}"#).is_none());
        assert!(style_drain_rsi_only("garbage").is_none());
    }
}
