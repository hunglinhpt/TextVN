// SPDX-License-Identifier: GPL-3.0-or-later
//! Điểm kiểm tra Trợ lý ngữ pháp của TSF adapter (P4-0 §4.1 · D11 · AS-022).
//!
//! Tách **điểm kiểm tra** khỏi **điểm hiển thị** (D11): mỗi phím trigger
//! `space` / `,` / `.` / `!` / `?` / `;` / `:` SAU khi phím đã commit xong ⇒
//! kiểm tra **câu vừa gõ** (tĩnh lặng), lỗi ⇒ báo tray qua IPC. Hiển thị tại
//! caret (AS-030/031) chỉ ở trigger kết thúc câu — `edit_session::process`.
//!
//! Bất biến:
//! - **Không chặn đường gõ**: check chạy *sau khi* phím đã được engine xử lý
//!   (gọi sau `apply_plan` trong edit session), vượt ngân sách → bỏ im lặng.
//! - **Không đọc nội dung app khác** (D8): chỉ dùng buffer câu của core
//!   (`Engine::sentence_tail` — ta tự commit shadow buffer của mình).
//! - **Fail-open** (S4): mọi lỗi/panic/tr\borrow thất bại → bỏ im lặng, phím
//!   không bao giờ bị ảnh hưởng.
//! - **S2/R21**: không log câu; toast gửi tray chỉ mang **số lỗi** — nội dung
//!   câu người dùng chỉ nằm trong RAM tạm của 1 lần check.
//!
//! Biên FFI chuẩn **UTF-16** (D29): câu từ core (UTF-8 nội bộ) được encode 1
//! lần rồi đi qua `ime_assist_check_v1` / `ime_assist_sentence_tail_v1`.

use crate::compose::KeyKind;

/// Loại trigger điểm kiểm tra assist (D11 · P4-0 §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistTrigger {
    None,
    /// Ranh giới từ/vế: `space` `,` `;` `:` — điểm kiểm tra silent.
    Word,
    /// Kết thúc câu: `.` `!` `?` — điểm kiểm tra + điểm hiển thị.
    Sentence,
}

/// Cap câu (D31): buffer câu của core tối đa **256 ký tự Unicode**.
pub const SENTENCE_MAX_CHARS: usize = 256;

/// Câu < 3 từ → bỏ qua (D11 "Bỏ qua: câu < 3 từ") — tránh to bật mỗi 2 ký tự.
pub const MIN_SENTENCE_WORDS: usize = 3;

/// Ngân sách check trên thread UI của app (P4-0 §3.2).
/// Vượt → bỏ im lặng. Check câu ≤256 chars thường < 1 ms (`P4-0 §8`, S4a đo
/// 0,52 ms/câu) — 10 ms là trần an toàn, không phải p99.
pub const CHECK_BUDGET_MS: u128 = 10;

/// Khoảng cách tối thiểu giữa 2 lần toast tray — thắt chặt D11 (không spam UI
/// khi gõ thường; `Fassist-14`). Toast hiển thị chi tiết theo câu đã được
/// overlay tại caret giữ (AS-030), tray toast chỉ là fallback P4-0 §6.
pub const NOTIFY_COOLDOWN_MS: u64 = 2500;

/// Phân loại phím trigger — **hàm thuần**, test được không cần dựng TSF.
///
/// Bao gồm mọi trigger D11 (`space , . ! ? ; :`); dấu câu khác (`<>\"'`…) và
/// phím không sinh ký tự → `None`. Chỉ nhận phím **sinh ký tự thật**
/// (`KeyKind::Char`) — Enter/Tab/Backspace gây reset buffer, không phải check.
pub fn assist_trigger(key: KeyKind) -> AssistTrigger {
    match key {
        KeyKind::Char('.' | '!' | '?') => AssistTrigger::Sentence,
        KeyKind::Char(' ' | ',' | ';' | ':') => AssistTrigger::Word,
        _ => AssistTrigger::None,
    }
}

/// Đếm từ trong câu (chuỗi trắng tách từ) — đủ cho gate "câu < 3 từ" (D11).
pub fn word_count(sentence: &str) -> usize {
    sentence.split_whitespace().count()
}

/// Còn trong cửa sổ cooldown báo tray không: `true` ⇒ ĐƯỢC báo (đã cách lần
/// trước ≥ `NOTIFY_COOLDOWN_MS`). Hàm thuần, không đụng state toàn cục.
pub fn notify_allowed(now_ms: u64, last_ms: u64) -> bool {
    now_ms.saturating_sub(last_ms) >= NOTIFY_COOLDOWN_MS
}

/// Nội dung toast tray — **chỉ mang đếm** (S2/R21: không chứa câu người dùng).
pub fn notification_payload(issue_count: usize) -> (String, String) {
    (
        "Trợ lý ngữ pháp".to_string(),
        format!("Phát hiện {issue_count} lỗi ngữ pháp trong câu vừa gõ."),
    )
}

#[cfg(windows)]
fn millis_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Giữ lại thời điểm báo tray gần nhất (ms since epoch) — 1 cell toàn cục.
#[cfg(windows)]
static LAST_NOTIFY_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Claim cooldown: `true` = đạt quyền báo (so sánh nguyên tử — 2 thread UI cùng
/// claim cũng chỉ một bên thắng).
#[cfg(windows)]
fn claim_notify_slot(now_ms: u64) -> bool {
    use std::sync::atomic::Ordering;
    let last = LAST_NOTIFY_MS.load(Ordering::Acquire);
    if !notify_allowed(now_ms, last) {
        return false;
    }
    LAST_NOTIFY_MS
        .compare_exchange(last, now_ms, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

#[cfg(windows)]
pub fn after_trigger(shared: &crate::edit_session::TsfShared, trigger: AssistTrigger) -> bool {
    use textvn_ffi::{
        ime_assist_check_v1, ime_assist_issue_v1, ime_assist_result_v1,
        ime_assist_sentence_tail_v1, IME_ABI_VERSION, IME_MAX_ASSIST_ISSUES, IME_MAX_ASSIST_MSG,
        IME_MAX_ASSIST_SUGGEST,
    };

    // D20 kill-switch: tắt 1 bước từ tray → không check gì cả.
    if !shared.ipc.is_assist_enabled() {
        return false;
    }
    if trigger == AssistTrigger::None {
        return false;
    }
    let began = std::time::Instant::now();

    // Câu vừa gõ = buffer câu của core (D8 — không đọc nội dung app khác);
    // engine giữ tối đa 256 ký tự Unicode (D31).
    let sentence = {
        let Ok(thread) = shared.thread.try_borrow() else {
            return false; // fail-open: engine đang bận
        };
        thread.engine.sentence_tail(SENTENCE_MAX_CHARS)
    };
    if began.elapsed().as_millis() > CHECK_BUDGET_MS {
        return false; // ngân sách: bỏ im lặng
    }
    if word_count(&sentence) < MIN_SENTENCE_WORDS {
        return false;
    }

    // Biên FFI chuẩn UTF-16 (D29): encode câu sang UTF-16 rồi lần lượt đi qua
    // `ime_assist_sentence_tail_v1` (cắt đuôi tối đa SENTENCE_MAX_CHARS char,
    // không chẻ surrogate) và `ime_assist_check_v1` (đếm lỗi).
    let utf16: Vec<u16> = sentence.encode_utf16().collect();
    let mut tail = [0_u16; 2 * SENTENCE_MAX_CHARS + 1];
    let written = ime_assist_sentence_tail_v1(
        utf16.as_ptr(),
        utf16.len().try_into().unwrap_or(0),
        SENTENCE_MAX_CHARS as i32,
        tail.as_mut_ptr(),
        tail.len().try_into().unwrap_or(0),
    );
    if written <= 0 {
        return false; // fail-open: đuôi cắt hỏng → im lặng
    }
    let written = written as usize;
    if began.elapsed().as_millis() > CHECK_BUDGET_MS {
        return false; // ngân sách: bỏ im lặng sau encode
    }

    let mut res = ime_assist_result_v1 {
        abi_version: IME_ABI_VERSION,
        count: 0,
        issues: [ime_assist_issue_v1 {
            kind: 0,
            start: 0,
            end: 0,
            auto_fixable: 0,
            confidence: 0,
            message: [0; IME_MAX_ASSIST_MSG],
            suggest_len: 0,
            suggest: [0; IME_MAX_ASSIST_SUGGEST],
        }; IME_MAX_ASSIST_ISSUES],
    };
    let issue_count = ime_assist_check_v1(tail.as_ptr(), written as i32, &mut res);
    if issue_count <= 0 {
        return false; // câu sạch / fail-open → im lặng (D11: kết quả im lặng)
    }

    // Báo tray KHÔNG block: thread nền, đường gõ không bao giờ đợi pipe; tray
    // không chạy → bỏ im lặng (fail-open). Toast chỉ mang số lỗi (S2).
    if !claim_notify_slot(millis_since_epoch()) {
        return true; // đã có lỗi, chỉ đang trong cooldown — đừng spam tray
    }
    let (title, message) = notification_payload(issue_count as usize);
    crate::ipc_client::notify_assist_async(title, message);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::vk;

    #[test]
    fn trigger_phan_loai_dung_theo_d11() {
        // Kết thúc câu (hiển thị + kiểm tra)
        assert_eq!(assist_trigger(KeyKind::Char('.')), AssistTrigger::Sentence);
        assert_eq!(assist_trigger(KeyKind::Char('!')), AssistTrigger::Sentence);
        assert_eq!(assist_trigger(KeyKind::Char('?')), AssistTrigger::Sentence);
        // Ranh giới từ/vế (chỉ kiểm tra silent)
        assert_eq!(assist_trigger(KeyKind::Char(' ')), AssistTrigger::Word);
        assert_eq!(assist_trigger(KeyKind::Char(',')), AssistTrigger::Word);
        assert_eq!(assist_trigger(KeyKind::Char(';')), AssistTrigger::Word);
        assert_eq!(assist_trigger(KeyKind::Char(':')), AssistTrigger::Word);
        // Không phải trigger: chữ/số/dấu khác + phím đặc biệt
        assert_eq!(assist_trigger(KeyKind::Char('a')), AssistTrigger::None);
        assert_eq!(assist_trigger(KeyKind::Char('<')), AssistTrigger::None);
        assert_eq!(
            assist_trigger(KeyKind::Char('\u{1F600}')),
            AssistTrigger::None
        );
        assert_eq!(assist_trigger(KeyKind::Char('"')), AssistTrigger::None);
        assert_eq!(assist_trigger(KeyKind::Enter), AssistTrigger::None);
        assert_eq!(assist_trigger(KeyKind::Tab), AssistTrigger::None);
        assert_eq!(assist_trigger(KeyKind::Backspace), AssistTrigger::None);
        assert_eq!(assist_trigger(KeyKind::Escape), AssistTrigger::None);
    }

    /// Enter/Tab là phím điều hướng thật (không phải `KeyKind::Char`) dù cổng
    /// FFI `ime_assist_trigger_kind` cũng trả None cho `\n`/`\t`.
    #[test]
    fn key_engine_ch_va_trigger_tach_biet() {
        // Phím xuất ra engine: Enter/Tab ch = 0, nhưng classify() trả KeyKind riêng.
        let key = KeyKind::classify(vk::RETURN, 0);
        assert_eq!(key.engine_ch(), 0);
        assert_eq!(assist_trigger(key), AssistTrigger::None);
        // space sinh ' ' cả hai chiều.
        let key = KeyKind::classify(0x20, ' ' as u32);
        assert_eq!(key.engine_ch(), ' ' as u32);
        assert_eq!(assist_trigger(key), AssistTrigger::Word);
    }

    #[test]
    fn word_count_gate_cau_qua_ngan() {
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("   "), 0);
        assert_eq!(word_count("tôi tôi"), 2);
        assert_eq!(word_count("tôi tôi đi học"), 4);
        assert_eq!(word_count("Xin chào 😀 của của nhé"), 6);
        assert!(word_count("tôi tôi") < MIN_SENTENCE_WORDS);
    }

    #[test]
    fn notify_cooldown_chong_spam_tray() {
        let now = 10_000_u64;
        // Lần đầu (chưa có ký lục → last=0, chênh đủ xa) → được.
        assert!(notify_allowed(now, 0));
        // Trong 2.5 s → chặn.
        assert!(!notify_allowed(
            now,
            now.saturating_sub(NOTIFY_COOLDOWN_MS - 1)
        ));
        // Vừa đủ quá cooldown → được.
        assert!(notify_allowed(now + NOTIFY_COOLDOWN_MS, now));
    }

    /// S2/R21: toast chỉ mang ĐẾM — không chứa câu người dùng.
    #[test]
    fn payload_chua_dem_khong_chua_cau() {
        let (title, msg) = notification_payload(2);
        assert_eq!(title, "Trợ lý ngữ pháp");
        assert_eq!(msg, "Phát hiện 2 lỗi ngữ pháp trong câu vừa gõ.");
        // Nội dung toast khép kín (format cố định) — không lọt câu người dùng (S2).
    }
}
