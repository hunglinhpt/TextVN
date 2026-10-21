// SPDX-License-Identifier: GPL-3.0-or-later
//! Quyết định hiển thị toast cho Trợ lý ngữ pháp (P4-0 §4.1 · D11/D20).
//!
//! Tách khỏi Win32 để test được **không cần dựng UI**: đầu vào chỉ là config
//! `assist`, kill-switch của tray và kết quả engine (`AssistReport`).
//!
//! Bất biến:
//! - Kill-switch tắt (menu "Trợ lý ngữ pháp") ⇒ **không** toast, bất kể mode.
//! - `assist.mode = off` ⇒ không toast (D20: mặc định tắt).
//! - Không có lỗi và không tự sửa gì ⇒ im lặng (D11: câu sạch không hiện gì).
//! - Tối đa 1 toast / [`TOAST_MIN_INTERVAL_MS`] — chống spam khi adapter gửi dồn.
//! - Nội dung toast **không** chứa toàn bộ câu, chỉ thông điệp engine sinh.

use std::sync::atomic::{AtomicU64, Ordering};

use textvn_config::AssistSettings;
use textvn_ipc::AssistIssue;

use crate::svc::SvcManager;

/// Khoảng cách tối thiểu giữa hai toast Trợ lý ngữ pháp.
///
/// `display.idle_ms` mặc định 700 ms ⇒ nếu không chặn, adapter gửi dồn có thể
/// thay balloon liên tục. 1,5 s vẫn đủ nhanh để người dùng thấy phản hồi.
pub const TOAST_MIN_INTERVAL_MS: u64 = 1_500;

/// Tiêu đề tối đa (WCHAR) — `NOTIFYICONDATAW.szInfoTitle` là 64, chừa 1 cho NUL.
pub const TOAST_TITLE_MAX_CHARS: usize = 60;

/// Nội dung tối đa (WCHAR) — `NOTIFYICONDATAW.szInfo` là 256, chừa 1 cho NUL.
pub const TOAST_BODY_MAX_CHARS: usize = 240;

/// Lý do quyết định — trả mã thay vì `bool` để test/log chẩn đoán rõ ràng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastDecision {
    /// Đủ điều kiện: hiện toast.
    Show,
    /// Người dùng đã tắt Trợ lý ngữ pháp bằng menu (D20).
    KillSwitchOff,
    /// `assist.mode = off` trong config.
    ModeOff,
    /// Không có lỗi và không tự sửa gì (câu sạch — P4-0 §4.1).
    NothingToSay,
    /// Vừa hiện toast cách đây chưa đủ [`TOAST_MIN_INTERVAL_MS`].
    TooSoon,
}

impl ToastDecision {
    /// Có hiện toast hay không.
    pub fn should_show(self) -> bool {
        matches!(self, ToastDecision::Show)
    }
}

/// Quyết định thuần (không I/O, không Win32) — nguồn sự thật duy nhất cho việc
/// "có hiện toast không".
///
/// `since_last_ms`: thời gian từ toast trước (`None` = chưa từng hiện).
pub fn decide(
    assist: &AssistSettings,
    kill_switch_on: bool,
    issue_count: usize,
    auto_fixed: u32,
    since_last_ms: Option<u64>,
) -> ToastDecision {
    // Kill-switch trước tiên: tắt 1 bước phải có hiệu lực ngay, kể cả config
    // còn `mode = autocorrect`.
    if !kill_switch_on {
        return ToastDecision::KillSwitchOff;
    }
    if !assist.is_on() {
        return ToastDecision::ModeOff;
    }
    if issue_count == 0 && auto_fixed == 0 {
        return ToastDecision::NothingToSay;
    }
    if since_last_ms.is_some_and(|since| since < TOAST_MIN_INTERVAL_MS) {
        return ToastDecision::TooSoon;
    }
    ToastDecision::Show
}

/// Nội dung toast từ kết quả engine; `None` = không có gì để nói.
///
/// Chỉ dùng `message` (thông điệp tiếng Việt ngắn của engine) + số lượng —
/// **không** ghép lại câu người dùng vừa gõ (S2/D8).
pub fn toast_text(issues: &[AssistIssue], auto_fixed: u32) -> Option<(String, String)> {
    if issues.is_empty() && auto_fixed == 0 {
        return None;
    }
    let mut body = String::new();
    if auto_fixed > 0 {
        body.push_str(&format!("Đã tự sửa {auto_fixed} lỗi."));
        if !issues.is_empty() {
            body.push(' ');
        }
    }
    if let Some(first) = issues.first() {
        body.push_str(&first.message);
        let extra = issues.len() - 1;
        if extra > 0 {
            body.push_str(&format!(" (+{extra} lỗi khác)"));
        }
    }
    Some((
        truncate_chars("TextVN — Trợ lý ngữ pháp", TOAST_TITLE_MAX_CHARS),
        truncate_chars(&body, TOAST_BODY_MAX_CHARS),
    ))
}

/// Cắt theo **ký tự** (không theo byte) + dấu `…`, để không bao giờ cắt giữa
/// một chữ tiếng Việt khi ghi vào buffer UTF-16 của Win32.
fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Mốc toast gần nhất (ms, 0 = chưa từng) — tách static để `decide` vẫn thuần.
static LAST_ASSIST_TOAST_MS: AtomicU64 = AtomicU64::new(0);

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Kênh thông báo assist **đơn giản** (`AssistNotification` — giữ cho tương
/// thích ngược). Chỉ check kill-switch: người dùng đã tắt Trợ lý ngữ pháp thì
/// không được nhận thêm thông báo của nó nữa (D20).
///
/// Trả `true` khi đã hiện.
pub fn notify_simple(svc: &SvcManager, title: &str, message: &str) -> bool {
    if !svc.is_assist_enabled() {
        return false;
    }
    crate::show_tray_notification(title, message);
    true
}

/// Đường thật: đọc config + kill-switch, quyết định rồi hiện balloon.
///
/// Trả `true` khi đã hiện toast (để test/đo được mà không cần UI).
pub fn notify_report(svc: &SvcManager, issues: &[AssistIssue], auto_fixed: u32) -> bool {
    let assist = svc.config().assist;
    let kill_switch_on = svc.is_assist_enabled();
    let now = now_millis();
    let last = LAST_ASSIST_TOAST_MS.load(Ordering::Acquire);
    let since_last = (last != 0).then(|| now.saturating_sub(last));
    let decision = decide(
        &assist,
        kill_switch_on,
        issues.len(),
        auto_fixed,
        since_last,
    );
    if !decision.should_show() {
        return false;
    }
    let Some((title, body)) = toast_text(issues, auto_fixed) else {
        return false;
    };
    // Claim nguyên tử: hai luồng client IPC cùng gửi report không được hiện 2 toast.
    if !crate::claim_debounced(&LAST_ASSIST_TOAST_MS, now, TOAST_MIN_INTERVAL_MS) {
        return false;
    }
    crate::show_tray_notification(&title, &body);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use textvn_config::{AssistAutoFix, AssistMode};

    fn issue(message: &str) -> AssistIssue {
        AssistIssue {
            kind: "duplicate_word".into(),
            span_start: 0,
            span_end: 2,
            message: message.into(),
            suggestions: vec!["của".into()],
            auto_fixable: true,
            confidence: 100,
        }
    }

    fn enabled(mode: AssistMode) -> AssistSettings {
        AssistSettings {
            mode,
            auto_fix: AssistAutoFix::Safe,
            ..AssistSettings::default()
        }
    }

    /// Kill-switch tắt thì im lặng tuyệt đối — kể cả mode autocorrect + có lỗi.
    #[test]
    fn kill_switch_off_wins_over_everything() {
        let a = enabled(AssistMode::Autocorrect);
        assert_eq!(decide(&a, false, 3, 2, None), ToastDecision::KillSwitchOff);
        assert!(!decide(&a, false, 3, 2, None).should_show());
    }

    /// `mode = off` (mặc định) thì không toast dù kill-switch đang bật.
    #[test]
    fn mode_off_never_notifies() {
        let a = AssistSettings::default();
        assert_eq!(a.mode, AssistMode::Off);
        assert_eq!(decide(&a, true, 5, 0, None), ToastDecision::ModeOff);
    }

    /// Câu sạch (0 lỗi, 0 tự sửa) → không hiện gì (D11).
    #[test]
    fn clean_sentence_is_silent() {
        let a = enabled(AssistMode::Highlight);
        assert_eq!(decide(&a, true, 0, 0, None), ToastDecision::NothingToSay);
        assert_eq!(
            toast_text(&[], 0),
            None,
            "không có gì để nói thì không dựng body"
        );
    }

    /// Mode 1 có lỗi → hiện; Mode 2 chỉ tự sửa (không còn lỗi) → vẫn hiện "Đã sửa".
    #[test]
    fn highlight_and_autocorrect_show_when_there_is_something() {
        assert_eq!(
            decide(&enabled(AssistMode::Highlight), true, 1, 0, None),
            ToastDecision::Show
        );
        assert_eq!(
            decide(&enabled(AssistMode::Autocorrect), true, 0, 2, None),
            ToastDecision::Show
        );
    }

    /// Chống spam: trong cửa sổ 1,5 s thì bỏ; sau đó hiện lại.
    #[test]
    fn toast_is_rate_limited() {
        let a = enabled(AssistMode::Highlight);
        assert_eq!(decide(&a, true, 1, 0, Some(0)), ToastDecision::TooSoon);
        assert_eq!(decide(&a, true, 1, 0, Some(1_499)), ToastDecision::TooSoon);
        assert_eq!(
            decide(&a, true, 1, 0, Some(TOAST_MIN_INTERVAL_MS)),
            ToastDecision::Show
        );
    }

    /// Body: thông điệp lỗi đầu + số lỗi còn lại; nhiều lỗi tự sửa có câu riêng.
    #[test]
    fn toast_text_summarises_without_the_sentence() {
        let issues = vec![issue("Lặp từ: \"của của\""), issue("Khoảng trắng kép")];
        let (title, body) = toast_text(&issues, 0).unwrap();
        assert!(title.contains("Trợ lý ngữ pháp"), "{title}");
        assert!(body.starts_with("Lặp từ"), "{body}");
        assert!(body.contains("(+1 lỗi khác)"), "{body}");

        let (_, body) = toast_text(&[], 3).unwrap();
        assert_eq!(body, "Đã tự sửa 3 lỗi.");

        let (_, body) = toast_text(&issues, 2).unwrap();
        assert!(body.starts_with("Đã tự sửa 2 lỗi."), "{body}");
        assert!(body.contains("(+1 lỗi khác)"), "{body}");
    }

    /// Body dài bị cắt theo KÝ TỰ + `…`, không cắt giữa chữ tiếng Việt.
    #[test]
    fn long_message_is_truncated_by_chars() {
        let long = "Lỗi ".repeat(200);
        let (title, body) = toast_text(&[issue(&long)], 0).unwrap();
        assert!(title.chars().count() <= TOAST_TITLE_MAX_CHARS);
        assert!(
            body.chars().count() <= TOAST_BODY_MAX_CHARS,
            "body {} > {}",
            body.chars().count(),
            TOAST_BODY_MAX_CHARS
        );
        assert!(body.ends_with('…'), "{body}");
        // Không còn nửa chữ: chuỗi vẫn là UTF-8 hợp lệ (String bảo đảm) và
        // không có replacement char.
        assert!(!body.contains('\u{fffd}'));
    }

    #[test]
    fn truncate_chars_keeps_short_text_and_counts_chars_not_bytes() {
        assert_eq!(truncate_chars("hoà", 10), "hoà");
        assert_eq!(truncate_chars("hoà thuỷ", 4), "hoà…");
        assert_eq!(truncate_chars("hoà thuỷ", 0), "…");
    }

    /// Kill-switch cũng chặn kênh `AssistNotification` đơn giản (D20), không chỉ
    /// toast kết quả. Balloon thật là no-op khi tray chưa có HWND (test headless).
    #[test]
    fn simple_notification_respects_kill_switch() {
        let dir = std::env::temp_dir().join(format!(
            "textvn_assist_toast_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let svc = SvcManager::new(Some(dir.clone()));
        assert!(!svc.is_assist_enabled(), "mặc định tắt (D20)");
        assert!(!notify_simple(&svc, "TextVN", "Lỗi chính tả"));
        svc.set_assist_enabled(true);
        assert!(notify_simple(&svc, "TextVN", "Lỗi chính tả"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
