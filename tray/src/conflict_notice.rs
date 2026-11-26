// SPDX-License-Identifier: GPL-3.0-or-later
//! E70 — Báo xung đột bộ gõ NGAY khi tray khởi động (không chỉ khi người dùng tự chạy
//! `doctor`): chạy `textvn-cli --check-env --json` ở luồng nền, lọc các phát hiện
//! `category == "conflict"` (UniKey/EVKey/OpenKey/GoTiengViet/VietKey, Microsoft
//! Vietnamese Telex/VNI đang bật…), hiện balloon MỘT lần cho mỗi tập xung đột mới.
//!
//! Marker `%APPDATA%\TextVN\conflict_notice` lưu chữ ký tập xung đột đã báo — cùng tập
//! thì không làm phiền lại; hết xung đột thì xoá marker để lần tái phát vẫn được báo.
//! Chỉ đọc mã/thông điệp chẩn đoán — không có nội dung gõ (G8).

/// Tên file marker trong `%APPDATA%\TextVN`.
pub const MARKER_FILE: &str = "conflict_notice";

/// Mã đã có luồng xử lý riêng (hộp thoại Dành Ctrl+Shift) — không báo trùng.
const HANDLED_ELSEWHERE: [&str; 1] = ["W_CTRL_SHIFT_TAKEN"];

/// Kết quả lọc JSON `doctor --json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictSummary {
    /// Chữ ký ổn định (mã|thông điệp đã sắp xếp) để so với marker.
    pub signature: String,
    /// Các thông điệp xung đột (đã theo thứ tự chữ ký).
    pub messages: Vec<String>,
}

/// Lọc phát hiện xung đột từ JSON `doctor --json`. `None` = không có xung đột (hoặc
/// JSON hỏng — không báo nhầm).
pub fn summarize(doctor_json: &str) -> Option<ConflictSummary> {
    let v: serde_json::Value = serde_json::from_str(doctor_json.trim()).ok()?;
    let mut items: Vec<(String, String)> = v
        .get("diagnostics")?
        .as_array()?
        .iter()
        .filter(|d| d.get("category").and_then(|c| c.as_str()) == Some("conflict"))
        .filter_map(|d| {
            let code = d.get("code")?.as_str()?.to_string();
            if HANDLED_ELSEWHERE.contains(&code.as_str()) {
                return None;
            }
            let msg = d
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            Some((code, msg))
        })
        .collect();
    if items.is_empty() {
        return None;
    }
    items.sort();
    items.dedup();
    let signature = items
        .iter()
        .map(|(c, m)| format!("{c}|{m}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(ConflictSummary {
        signature,
        messages: items.into_iter().map(|(_, m)| m).collect(),
    })
}

/// Có nên hiện balloon? Chỉ khi tập xung đột khác lần đã báo.
pub fn should_notify(previous_marker: Option<&str>, summary: &ConflictSummary) -> bool {
    previous_marker.map(str::trim) != Some(summary.signature.trim())
}

/// Nội dung balloon (giới hạn 255 ký tự của `szInfo` — cắt an toàn theo ký tự).
pub fn balloon_text(summary: &ConflictSummary) -> String {
    let first = summary.messages.first().map(String::as_str).unwrap_or("");
    let more = if summary.messages.len() > 1 {
        format!(" (+{} xung đột khác)", summary.messages.len() - 1)
    } else {
        String::new()
    };
    let tail = " — có thể gây lặp chữ/mất dấu. Thoát bộ gõ cũ; chi tiết: chuột phải TextVN › Chẩn đoán & xuất log.";
    // Cắt phần thông điệp (không cắt hướng dẫn ở cuối) để vừa 250 ký tự.
    let budget = 250usize.saturating_sub(more.chars().count() + tail.chars().count());
    let mut head: String = first.chars().take(budget).collect();
    if head.chars().count() < first.chars().count() && budget > 0 {
        head.pop();
        head.push('…');
    }
    format!("{head}{more}{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{"diagnostics":[
      {"code":"W_CTRL_SHIFT_TAKEN","severity":"warning","category":"conflict","message":"Windows giữ Ctrl+Shift","suggestion":""},
      {"code":"W_CONFLICT_VI_TIP","severity":"warning","category":"conflict","message":"Bộ gõ tiếng Việt khác đang được bật trong danh sách ngôn ngữ Windows: Microsoft Vietnamese Telex","suggestion":"x"},
      {"code":"W_CONFLICT_VI_IME","severity":"warning","category":"conflict","message":"Phát hiện bộ gõ tiếng Việt khác trên Windows: UniKey (startup)","suggestion":"y"},
      {"code":"E_TSF_UNREGISTERED","severity":"error","category":"runtime","message":"m","suggestion":"s"}
    ]}"#;

    #[test]
    fn e70_summarize_keeps_only_conflicts_and_skips_ctrl_shift() {
        let s = summarize(JSON).expect("có xung đột");
        assert_eq!(s.messages.len(), 2);
        assert!(s.signature.starts_with("W_CONFLICT_VI_IME|"));
        assert!(!s.signature.contains("W_CTRL_SHIFT_TAKEN"));
        assert!(!s.signature.contains("E_TSF_UNREGISTERED"));
    }

    #[test]
    fn e70_summarize_none_when_clean_or_garbage() {
        assert_eq!(summarize(r#"{"diagnostics":[]}"#), None);
        assert_eq!(summarize("not json"), None);
        assert_eq!(summarize(""), None);
    }

    #[test]
    fn e70_notify_once_per_conflict_set() {
        let s = summarize(JSON).unwrap();
        assert!(should_notify(None, &s));
        assert!(!should_notify(Some(&s.signature), &s));
        assert!(!should_notify(Some(&format!("{}\n", s.signature)), &s));
        assert!(should_notify(Some("W_CONFLICT_VI_IME|cũ"), &s));
    }

    #[test]
    fn e70_balloon_text_fits_shell_limit() {
        let s = summarize(JSON).unwrap();
        let t = balloon_text(&s);
        assert!(t.chars().count() <= 250);
        assert!(t.contains("+1 xung đột khác"));
        assert!(t.contains("Chẩn đoán & xuất log"));
        let long = ConflictSummary {
            signature: "x".into(),
            messages: vec!["Ư".repeat(400)],
        };
        let t = balloon_text(&long);
        assert!(t.chars().count() <= 250, "{}", t.chars().count());
        assert!(t.ends_with("Chẩn đoán & xuất log."));
    }
}
