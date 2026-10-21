// SPDX-License-Identifier: GPL-3.0-or-later
//! Điểm kiểm tra + mô hình hiển thị Trợ lý ngữ pháp của TSF adapter
//! (P4-0 §4.1/§4.2/§4.4 · D11 · D12 · D27 · D30 · AS-030/AS-031).
//!
//! Ba tầng, **chỉ tầng cuối** đụng Win32/COM:
//! 1. Quyết định (thuần, test mọi OS): [`assist_trigger`], [`should_show_overlay`],
//!    [`OverlayView`], [`overlay_action`], [`plan_fix`], [`can_apply_fix`];
//! 2. Dựng model từ kết quả FFI: [`OverlayView::from_ffi`] (UTF-8 C buffer →
//!    `String`, span **char Unicode** — D27/D29);
//! 3. Điểm kiểm tra `check_sentence`/`after_trigger` (`cfg(windows)`) + overlay
//!    (`edit_session::show_assist_overlay`) + áp sửa A2 (`edit_session.rs`).
//!
//! Bất biến:
//! - **Không chặn đường gõ**: check chạy *sau khi* phím đã được engine xử lý
//!   (gọi sau `apply_plan` trong edit session), vượt ngân sách → bỏ im lặng.
//! - **Không đọc nội dung app khác** (D8): chỉ dùng buffer câu của core
//!   (`Engine::sentence_tail` — ta tự commit shadow buffer của mình).
//! - **Fail-open** (S4): mọi lỗi/panic/borrow thất bại → bỏ im lặng, phím
//!   không bao giờ bị ảnh hưởng.
//! - **S2/R21**: không log câu; toast gửi tray chỉ mang **số lỗi** — nội dung
//!   câu người dùng chỉ nằm trong RAM tạm của 1 lần check / overlay.
//! - **D27**: `Issue` dùng offset **char Unicode**; số ký tự cần xoá trong
//!   document là **glyph** (do core tính — `Engine::glyph_len_of`), vì app
//!   nhận glyph theo bảng mã xuất (VNI/TCVN3: 1 chữ Việt = nhiều code unit).
//! - **D22**: A2 chỉ dùng **Backspace+Type** (xoá đúng `delete_glyphs` glyph
//!   rồi gõ lại qua composition), **không** `SelectionReplace`.
//! - **D38**: định dạng hiển thị lấy chuẩn từ `textvn-cli assist show`
//!   (câu + vạch `^` đúng span + thông điệp + gợi ý) — xem
//!   [`OverlayView::render_plain`].

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
pub const NOTIFY_COOLDOWN_MS: u64 = 2_500;

// ---------------------------------------------------------------------------
// Mode 1 — mô hình hiển thị overlay (D4/D30/D38)
// ---------------------------------------------------------------------------

/// Overlay hiển thị **tối đa 3 lỗi** (P4-0 §6 "1–3 gợi ý"): mỗi lỗi mang 1
/// gợi ý (biên FFI hiện chỉ chuyển gợi ý đầu — `ffi::fill_result`), nên 3 lỗi
/// = 3 lựa chọn `Alt+1..3`. Lỗi thừa gộp thành "+N lỗi khác".
pub const MAX_OVERLAY_ISSUES: usize = 3;

/// Cắt thông điệp / gợi ý theo **ký tự** để panel không phình (cắt giữa chữ
/// tiếng Việt sau đó bằng `…`).
pub const MAX_OVERLAY_MESSAGE_CHARS: usize = 72;
pub const MAX_OVERLAY_SUGGESTION_CHARS: usize = 32;

/// Số ký tự cuối của câu được vẽ trong overlay (câu tối đa 256): panel đọc
/// được, vẫn đủ ngữ cảnh quanh lỗi ở cuối câu (nơi caret đang đứng).
pub const MAX_OVERLAY_SENTENCE_CHARS: usize = 96;

/// Tự tắt sau 8 s không tương tác (P4-0 §6 · D11).
pub const OVERLAY_IDLE_MS: u64 = 8_000;

/// Nhắc phím — chuỗi tĩnh, không chứa nội dung người dùng.
pub const OVERLAY_HINT: &str = "↑/↓ chọn lỗi · Enter sửa · Alt+1…3 gợi ý · Esc đóng";

/// VK dùng cho overlay (khớp Win32).
pub const VK_UP: u32 = 0x26;
pub const VK_DOWN: u32 = 0x28;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_G: u32 = 0x47;

/// Một lỗi đã chuẩn hoá để hiển thị. `start..end` là **char Unicode** trong
/// [`OverlayView::sentence`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueView {
    pub start: usize,
    pub end: usize,
    pub message: String,
    /// Gợi ý đầu tiên (biên FFI chỉ mang 1 gợi ý/lỗi). `None` = không gợi ý.
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

/// Model overlay: câu vừa gõ + lỗi + lỗi đang chọn. Thuần, không Win32 — dựng
/// được trong test từ [`OverlayView::from_parts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayView {
    /// Câu (char Unicode) — chính là buffer câu của core lúc hiển thị (D8).
    pub sentence: Vec<char>,
    /// Lỗi đang hiển thị (≤ [`MAX_OVERLAY_ISSUES`]).
    pub issues: Vec<IssueView>,
    /// Tổng số lỗi engine trả về (có thể > `issues.len()`, phần dư gộp "+N").
    pub total_issues: usize,
    /// Chỉ số lỗi đang chọn (`issues`).
    pub selected: usize,
}

impl OverlayView {
    /// Dựng từ kết quả FFI + câu UTF-8. `None` = không có gì để hiện.
    ///
    /// Span lạ (rỗng/ngoài câu) bị **bỏ** thay vì đoán vị trí — fail-safe.
    /// Không panic với C buffer hỏng/không phải UTF-8 (fallback rỗng).
    pub fn from_ffi(res: &textvn_ffi::ime_assist_result_v1, sentence: &str) -> Option<Self> {
        let chars: Vec<char> = sentence.chars().collect();
        if chars.is_empty() {
            return None;
        }
        let reported = res.count as usize;
        let usable = reported.min(res.issues.len());
        let mut issues = Vec::new();
        for raw in res.issues.iter().take(usable) {
            let start = (raw.start as usize).min(chars.len());
            let end = (raw.end as usize).min(chars.len());
            if start >= end {
                continue; // span rỗng/lạ → bỏ
            }
            let message = c_buf_to_string(&raw.message);
            let suggestion = c_buf_to_string(&raw.suggest);
            issues.push(IssueView {
                start,
                end,
                message: if message.is_empty() {
                    "Lỗi ngữ pháp".to_string()
                } else {
                    message
                },
                suggestion: (!suggestion.is_empty()).then_some(suggestion),
                auto_fixable: raw.auto_fixable != 0,
            });
        }
        Self::from_parts(chars, issues, reported)
    }

    /// Dựng từ dữ liệu thuần (test / UI khác dùng lại).
    pub fn from_parts(
        sentence: Vec<char>,
        mut issues: Vec<IssueView>,
        total: usize,
    ) -> Option<Self> {
        if sentence.is_empty() {
            return None;
        }
        issues.retain(|i| i.start < i.end && i.end <= sentence.len());
        if issues.is_empty() {
            return None;
        }
        let total = total.max(issues.len());
        issues.truncate(MAX_OVERLAY_ISSUES);
        Some(Self {
            sentence,
            issues,
            total_issues: total,
            selected: 0,
        })
    }

    /// Số lỗi không hiển thị (gộp "+N lỗi khác").
    pub fn hidden_count(&self) -> usize {
        self.total_issues.saturating_sub(self.issues.len())
    }

    pub fn selected_issue(&self) -> Option<&IssueView> {
        self.issues.get(self.selected)
    }

    /// Chọn lỗi kế/trước (vòng tròn). Trả `true` nếu chỉ số đổi (cần vẽ lại).
    pub fn select(&mut self, delta: isize) -> bool {
        let len = self.issues.len();
        if len < 2 {
            return false;
        }
        let before = self.selected;
        let step = delta.rem_euclid(len as isize) as usize;
        self.selected = (self.selected + step) % len;
        self.selected != before
    }

    /// Chọn lỗi theo chỉ số tuyệt đối (Alt+1..3).
    pub fn select_index(&mut self, index: usize) -> bool {
        if index >= self.issues.len() {
            return false;
        }
        let changed = index != self.selected;
        self.selected = index;
        changed
    }

    /// Số ký tự cuối câu được vẽ (offset bắt đầu, slice) — panel chỉ rộng vừa.
    pub fn visible_sentence(&self, max: usize) -> (usize, &[char]) {
        let skip = self.sentence.len().saturating_sub(max);
        (skip, &self.sentence[skip..])
    }

    /// Một dòng "vạch lỗi" của lỗi `index` trong **khoảng đã vẽ** (cùng quy ước
    /// `textvn-cli assist show`: `^` đúng span char Unicode, bỏ space cuối dòng).
    pub fn marker_line(&self, index: usize, skip: usize, visible: usize) -> String {
        let Some(issue) = self.issues.get(index) else {
            return String::new();
        };
        let mut out = vec![' '; visible];
        if visible > 0 {
            for (i, slot) in out.iter_mut().enumerate() {
                let pos = skip + i;
                if pos >= issue.start && pos < issue.end {
                    *slot = '^';
                }
            }
        }
        // CLI `assist show` không có space thừa sau vạch cuối.
        let end = out.iter().rposition(|c| *c != ' ').map_or(0, |i| i + 1);
        out[..end].iter().collect()
    }

    /// Dòng thông điệp lỗi `index` (spec D38): `  N. thông điệp  -> gợi ý`.
    pub fn issue_line(&self, index: usize) -> String {
        let Some(issue) = self.issues.get(index) else {
            return String::new();
        };
        let message = truncate_chars(&issue.message, MAX_OVERLAY_MESSAGE_CHARS);
        match &issue.suggestion {
            Some(s) => format!(
                "  {}. {}  -> {}",
                index + 1,
                message,
                truncate_chars(s, MAX_OVERLAY_SUGGESTION_CHARS)
            ),
            None => format!("  {}. {}", index + 1, message),
        }
    }

    /// Render văn bản thuần theo đúng spec `textvn-cli assist show` (D38) —
    /// cũng là fallback khi vẽ GDI lỗi. Dùng cho test không cần UI.
    pub fn render_plain(&self) -> String {
        let (skip, visible_slice) = self.visible_sentence(MAX_OVERLAY_SENTENCE_CHARS);
        let mut out = String::new();
        if skip > 0 {
            out.push('…');
        }
        out.extend(visible_slice.iter());
        out.push('\n');
        for i in 0..self.issues.len() {
            out.push_str(&self.marker_line(i, skip, visible_slice.len()));
            out.push('\n');
            out.push_str(&self.issue_line(i));
            out.push('\n');
        }
        if self.hidden_count() > 0 {
            out.push_str(&format!("  (+{} lỗi khác)\n", self.hidden_count()));
        }
        out.push_str(OVERLAY_HINT);
        out
    }
}

/// Kết quả quyết định **có hiện overlay không** (thuần): chỉ điểm hiển thị
/// (kết thúc câu — D11), assist đang bật (D20) và có lỗi.
pub fn should_show_overlay(
    trigger: AssistTrigger,
    assist_enabled: bool,
    issue_count: usize,
) -> bool {
    assist_enabled && trigger == AssistTrigger::Sentence && issue_count > 0
}

/// Phím tác động lên overlay đang hiện (P4-0 §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    /// `↑` — lỗi trước.
    Previous,
    /// `↓` — lỗi kế.
    Next,
    /// `Enter` — áp dụng gợi ý của lỗi đang chọn.
    Apply,
    /// `Alt+1..3` — chọn + áp dụng gợi ý thứ `n` (0-based).
    Suggestion(usize),
    /// `Esc` — đóng, không sửa.
    Close,
    /// `Ctrl+Shift+G` — gọi lại (kiểm tra + hiện overlay nếu còn lỗi).
    Recall,
    /// Phím khác: đóng overlay rồi để phím đi tiếp (gõ tiếp = tự tắt).
    Dismiss,
}

/// Bản đồ phím của overlay — **hàm thuần**, test không cần dựng TSF.
///
/// `mods` dùng bit `textvn_ffi::MOD_*` (khớp `key_event::active_modifiers`).
/// Caps Lock (`MOD_CAPS`) không tính là modifier của overlay.
pub fn overlay_action(vk: u32, mods: u32) -> OverlayAction {
    use textvn_ffi::{MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_SHIFT, MOD_SUPER};
    let mods = mods & !MOD_CAPS;
    let alt = mods & MOD_ALT != 0;
    let ctrl = mods & MOD_CTRL != 0;
    let sup = mods & MOD_SUPER != 0;
    let shift = mods & MOD_SHIFT != 0;

    // Ctrl+Shift+G: gọi lại (không phụ thuộc overlay đang hiện hay không).
    if ctrl && shift && !alt && !sup && vk == VK_G {
        return OverlayAction::Recall;
    }
    // Alt+1..3 (hàng số + numpad).
    if alt && !ctrl && !sup && !shift {
        return match vk {
            0x31..=0x33 => OverlayAction::Suggestion(vk as usize - 0x31),
            0x61..=0x63 => OverlayAction::Suggestion(vk as usize - 0x61),
            _ => OverlayAction::Dismiss,
        };
    }
    // Chord khác (Ctrl/Alt/Win) không phải phím của overlay.
    if ctrl || alt || sup {
        return OverlayAction::Dismiss;
    }
    match vk {
        VK_UP => OverlayAction::Previous,
        VK_DOWN => OverlayAction::Next,
        VK_RETURN if !shift => OverlayAction::Apply,
        VK_ESCAPE => OverlayAction::Close,
        _ => OverlayAction::Dismiss,
    }
}

// ---------------------------------------------------------------------------
// A2 — sửa 1 từ bằng Backspace+Type (D12/D22/D27)
// ---------------------------------------------------------------------------

/// Kế hoạch sửa **một** lỗi (A2). Toàn bộ vùng bị xoá nằm **bên trái caret**
/// (Backspace chỉ đi về trái): từ `issue.start` tới hết câu; phần đuôi sau
/// `issue.end` được gõ lại **nguyên văn** từ buffer của core ⇒ chỉ **một từ**
/// đổi nội dung (đúng phạm vi A2), không dùng `SelectionReplace` (D22).
///
/// Ngoại lệ **lặp từ** (xem [`plan_fix`]): vùng xoá lùi thêm 1 space và phần gõ
/// lại bắt đầu sau từ lặp — vẫn chỉ 1 từ bị xoá, không sửa từ nào khác.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixPlan {
    pub issue_index: usize,
    pub suggestion: String,
    /// Số **char Unicode** trong câu bị xoá (từ `issue.start` tới hết câu).
    pub delete_chars: usize,
    /// Số **glyph** cần xoá trong document (D27) — core tính theo bảng mã xuất.
    pub delete_glyphs: u32,
    /// Ký tự gõ lại = gợi ý + đuôi giữ nguyên.
    pub replacement: Vec<char>,
}

/// Tính kế hoạch sửa cho lỗi `issue_index`; `None` = không sửa được (fail-safe).
///
/// `glyph_len` do core cung cấp (`Engine::glyph_len_of`) — hàm này **không**
/// tự đổi char → glyph (D27: `Fix` không mang số xoá).
pub fn plan_fix<F>(view: &OverlayView, issue_index: usize, glyph_len: F) -> Option<FixPlan>
where
    F: Fn(&str) -> usize,
{
    let issue = view.issues.get(issue_index)?;
    if !issue.auto_fixable {
        // Lỗi **chỉ gợi ý** (vd. spelling: `auto_fixable = false` — A1/D12):
        // người dùng tự quyết, A2 không tự sửa.
        return None;
    }
    if issue.start >= issue.end || issue.end > view.sentence.len() {
        return None;
    }
    let suggestion = issue.suggestion.as_deref()?;
    if suggestion.is_empty() || suggestion.chars().any(char::is_control) {
        return None; // không có gợi ý / gợi ý chứa ký tự điều khiển → không sửa
    }
    let mut delete_from = issue.start;
    let mut replacement: Vec<char> = suggestion.chars().collect();
    replacement.extend_from_slice(&view.sentence[issue.end..]);

    if replacement.as_slice() == &view.sentence[issue.start..] {
        // Gợi ý **trùng** đoạn đang có ⇒ thay thế không đổi gì. Trường hợp hợp lệ
        // duy nhất là "lặp từ": engine đặt span ở **lần xuất hiện sau** và gợi ý
        // là từ trước đó (`duplicate_word.rs`), sửa đúng là **xoá** lần lặp cùng
        // 1 space ngay trước (đúng semantics `Fix::Delete` của engine). Không
        // đủ điều kiện đó → không sửa (A1: chỉ gợi ý).
        let span_text: String = view.sentence[issue.start..issue.end].iter().collect();
        let same_word = span_text.to_lowercase() == suggestion.to_lowercase();
        let prev_is_space = issue.start > 0 && view.sentence[issue.start - 1] == ' ';
        if !same_word || !prev_is_space {
            return None;
        }
        delete_from = issue.start - 1;
        replacement = view.sentence[issue.end..].to_vec();
    }

    let tail: String = view.sentence[delete_from..].iter().collect();
    let glyphs = glyph_len(&tail);
    if glyphs == 0 {
        return None;
    }
    Some(FixPlan {
        issue_index,
        suggestion: suggestion.to_string(),
        delete_chars: tail.chars().count(),
        delete_glyphs: u32::try_from(glyphs).ok()?,
        replacement,
    })
}

/// Precondition của A2/D12 — **tất cả** phải đúng mới sửa. Mọi giá trị đều là
/// *vị trí / danh tính buffer*, **không** phải nội dung app khác (D8).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FixPrecondition {
    /// Selection là caret rỗng (không bôi đen văn bản của người dùng).
    pub selection_empty: bool,
    /// Caret vẫn đúng vị trí đã chụp lúc hiện overlay (so range — chỉ vị trí).
    pub caret_unchanged: bool,
    /// Buffer câu của core vẫn đúng câu lúc hiện overlay ⇒ **từ do ta gõ**
    /// (người dùng chưa gõ xen / app chưa đổi document theo cách ta biết).
    pub buffer_is_ours: bool,
    /// Lùi được **đúng** `delete_glyphs` glyph (text store không cắt ngang).
    pub backspace_shift_ok: bool,
}

impl FixPrecondition {
    /// Tất cả điều kiện đạt (dùng cho test/đọc code rõ ràng).
    pub fn all_ok(&self) -> bool {
        self.selection_empty
            && self.caret_unchanged
            && self.buffer_is_ours
            && self.backspace_shift_ok
    }
}

/// Fail-safe: thiếu **một** điều kiện là **không sửa** (chỉ gợi ý — D12).
pub fn can_apply_fix(pre: FixPrecondition) -> bool {
    pre.all_ok()
}

// ---------------------------------------------------------------------------
// Tiện ích thuần
// ---------------------------------------------------------------------------

/// Đọc C buffer (`c_char`, NUL-terminated, UTF-8) → `String`; hỏng → rỗng.
fn c_buf_to_string(buf: &[std::ffi::c_char]) -> String {
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..len])
        .map(str::to_string)
        .unwrap_or_default()
}

/// Cắt theo **ký tự** + `…` (không cắt giữa một chữ tiếng Việt).
fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

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

/// Kết quả một lần kiểm tra câu — nguồn **duy nhất** cho cả overlay (AS-030) và
/// toast tray (AS-022), nên span `start..end` trong `result` luôn trỏ đúng vào
/// `sentence` (không phải bản sao thứ hai của câu).
#[cfg(windows)]
pub struct AssistCheck {
    /// Câu đã cắt theo [`SENTENCE_MAX_CHARS`] — đúng chuỗi engine vừa kiểm tra
    /// (UTF-16 → `String`, char Unicode).
    pub sentence: String,
    /// Lỗi của lần kiểm tra này (offset **char Unicode** — D27).
    pub result: textvn_ffi::ime_assist_result_v1,
}

#[cfg(windows)]
impl AssistCheck {
    /// Số lỗi thật sự có dữ liệu (chặn `count` hỏng từ biên FFI).
    pub fn issue_count(&self) -> usize {
        (self.result.count as usize).min(self.result.issues.len())
    }

    /// Model overlay (D38) dựng từ chính kết quả này; `None` = không có gì hiện.
    pub fn view(&self) -> Option<OverlayView> {
        OverlayView::from_ffi(&self.result, &self.sentence)
    }
}

/// Chạy kiểm tra câu trong ngân sách [`CHECK_BUDGET_MS`] — **không** side effect
/// UI. `None` = tắt assist / câu ngắn / câu sạch / vượt ngân sách (bỏ im lặng).
#[cfg(windows)]
pub fn check_sentence(shared: &crate::edit_session::TsfShared) -> Option<AssistCheck> {
    use textvn_ffi::{
        ime_assist_check_v1, ime_assist_sentence_tail_v1, IME_ABI_VERSION, IME_MAX_ASSIST_ISSUES,
        IME_MAX_ASSIST_MSG, IME_MAX_ASSIST_SUGGEST,
    };

    // D20 kill-switch: tắt 1 bước từ tray → không check gì cả.
    if !shared.ipc.is_assist_enabled() {
        return None;
    }
    let began = std::time::Instant::now();

    // Câu vừa gõ = buffer câu của core (D8 — không đọc nội dung app khác);
    // engine giữ tối đa 256 ký tự Unicode (D31).
    let sentence = {
        let thread = shared.thread.try_borrow().ok()?; // fail-open: engine đang bận
        thread.engine.sentence_tail(SENTENCE_MAX_CHARS)
    };
    if began.elapsed().as_millis() > CHECK_BUDGET_MS {
        return None; // ngân sách: bỏ im lặng
    }
    if word_count(&sentence) < MIN_SENTENCE_WORDS {
        return None;
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
        return None; // fail-open: đuôi cắt hỏng → im lặng
    }
    let written = written as usize;
    if began.elapsed().as_millis() > CHECK_BUDGET_MS {
        return None; // ngân sách: bỏ im lặng sau encode
    }
    // Span của `result` trỏ vào CHÍNH chuỗi này (không phải `sentence` gốc).
    let checked = String::from_utf16(&tail[..written]).ok()?;

    let mut result = textvn_ffi::ime_assist_result_v1 {
        abi_version: IME_ABI_VERSION,
        count: 0,
        issues: [textvn_ffi::ime_assist_issue_v1 {
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
    let issue_count = ime_assist_check_v1(tail.as_ptr(), written as i32, &mut result);
    if issue_count <= 0 {
        return None; // câu sạch / fail-open → im lặng (D11: kết quả im lặng)
    }
    Some(AssistCheck {
        sentence: checked,
        result,
    })
}

/// Điểm kiểm tra sau mỗi phím trigger: check câu + (cooldown) báo tray chỉ mang
/// **số lỗi**. Trả kết quả để caller hiện overlay ở điểm hiển thị (D11/D38).
#[cfg(windows)]
pub fn after_trigger(
    shared: &crate::edit_session::TsfShared,
    trigger: AssistTrigger,
) -> Option<AssistCheck> {
    if trigger == AssistTrigger::None {
        return None;
    }
    let check = check_sentence(shared)?;
    // Báo tray KHÔNG block: thread nền, đường gõ không bao giờ đợi pipe; tray
    // không chạy → bỏ im lặng (fail-open). Toast chỉ mang số lỗi (S2).
    if claim_notify_slot(millis_since_epoch()) {
        let (title, message) = notification_payload(check.issue_count());
        crate::ipc_client::notify_assist_async(title, message);
    }
    Some(check)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::vk;
    use textvn_ffi::MOD_ALT;

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
        let key = KeyKind::classify(vk::RETURN, 0);
        assert_eq!(key.engine_ch(), 0);
        assert_eq!(assist_trigger(key), AssistTrigger::None);
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
        assert!(notify_allowed(now, 0));
        assert!(!notify_allowed(
            now,
            now.saturating_sub(NOTIFY_COOLDOWN_MS - 1)
        ));
        assert!(notify_allowed(now + NOTIFY_COOLDOWN_MS, now));
    }

    /// S2/R21: toast chỉ mang ĐẾM — không chứa câu người dùng.
    #[test]
    fn payload_chua_dem_khong_chua_cau() {
        let (title, msg) = notification_payload(2);
        assert_eq!(title, "Trợ lý ngữ pháp");
        assert_eq!(msg, "Phát hiện 2 lỗi ngữ pháp trong câu vừa gõ.");
    }

    // -- Mode 1: hiển thị -------------------------------------------------

    fn issue(start: usize, end: usize, message: &str, suggestion: Option<&str>) -> IssueView {
        IssueView {
            start,
            end,
            message: message.to_string(),
            suggestion: suggestion.map(str::to_string),
            auto_fixable: true,
        }
    }

    fn view(sentence: &str, issues: Vec<IssueView>) -> OverlayView {
        let chars = sentence.chars().collect();
        let total = issues.len();
        OverlayView::from_parts(chars, issues, total).expect("view")
    }

    #[test]
    fn chi_hien_overlay_o_diem_hien_thi_va_khi_co_loi() {
        assert!(should_show_overlay(AssistTrigger::Sentence, true, 1));
        // Space/,/: là điểm kiểm tra silent (D11) → không hiện.
        assert!(!should_show_overlay(AssistTrigger::Word, true, 3));
        assert!(!should_show_overlay(AssistTrigger::None, true, 3));
        // Kill-switch / câu sạch → im lặng.
        assert!(!should_show_overlay(AssistTrigger::Sentence, false, 3));
        assert!(!should_show_overlay(AssistTrigger::Sentence, true, 0));
    }

    /// D38: render phải khớp `textvn-cli assist show` — câu, vạch `^` đúng
    /// span char Unicode, dòng "  N. thông điệp  -> gợi ý".
    #[test]
    fn render_theo_spec_cli_show() {
        // Span của `duplicate_word` = lần xuất hiện SAU (12..15 char, không phải byte).
        let v = view(
            "Hôm nay tôi tôi đi học .",
            vec![issue(12, 15, "Lặp từ: \"tôi\"", Some("tôi"))],
        );
        let plain = v.render_plain();
        let lines: Vec<&str> = plain.lines().collect();
        assert_eq!(lines[0], "Hôm nay tôi tôi đi học .");
        assert_eq!(lines[1], "            ^^^");
        assert_eq!(lines[2], "  1. Lặp từ: \"tôi\"  -> tôi");
        assert_eq!(*lines.last().unwrap(), OVERLAY_HINT);
    }

    /// Span > câu bị bỏ (fail-safe) và `count` lỗi vượt 3 → gộp "+N".
    #[test]
    fn span_la_bi_bo_va_loi_thua_gop_lai() {
        let chars: Vec<char> = "một hai ba bốn".chars().collect();
        let issues = vec![
            issue(0, 3, "a", Some("x")),
            issue(4, 7, "b", Some("y")),
            issue(8, 11, "c", Some("z")),
            issue(12, 99, "span ngoài câu", Some("w")),
        ];
        let v = OverlayView::from_parts(chars, issues, 4).expect("view");
        assert_eq!(v.issues.len(), MAX_OVERLAY_ISSUES);
        // Span lạ bị loại trước khi cắt → tổng vẫn 4 ⇒ còn 1 lỗi ẩn.
        assert_eq!(v.hidden_count(), 1);
        assert!(v.render_plain().contains("(+1 lỗi khác)"));

        let empty =
            OverlayView::from_parts("   ".chars().collect(), vec![issue(0, 99, "a", None)], 1);
        assert!(empty.is_none(), "span ngoài câu ⇒ không hiện gì");
    }

    #[test]
    fn chon_loi_vong_tron_va_theo_so() {
        let mut v = view(
            "một hai ba",
            vec![
                issue(0, 3, "a", Some("A")),
                issue(4, 7, "b", Some("B")),
                issue(8, 10, "c", Some("C")),
            ],
        );
        assert_eq!(v.selected, 0);
        assert!(v.select(1));
        assert_eq!(v.selected, 1);
        assert!(v.select(1));
        assert!(v.select(1), "vòng về 0");
        assert_eq!(v.selected, 0);
        assert!(v.select(-1));
        assert_eq!(v.selected, 2);
        assert!(v.select_index(1));
        assert_eq!(v.selected, 1);
        assert!(!v.select_index(9), "ngoài danh sách");
        // Một lỗi duy nhất: ↑/↓ không đổi gì.
        let mut one = view("một hai", vec![issue(0, 3, "a", Some("A"))]);
        assert!(!one.select(1));
        assert_eq!(one.selected, 0);
    }

    #[test]
    fn cau_dai_chi_ve_duoi_va_marker_lech_dung() {
        let sentence: String = "a ".repeat(80) + "tôi tôi";
        let len = sentence.chars().count();
        let v = view(&sentence, vec![issue(len - 7, len, "Lặp từ", Some("tôi"))]);
        let (skip, visible) = v.visible_sentence(MAX_OVERLAY_SENTENCE_CHARS);
        assert_eq!(visible.len(), MAX_OVERLAY_SENTENCE_CHARS);
        assert_eq!(skip, len - MAX_OVERLAY_SENTENCE_CHARS);
        let marker = v.marker_line(0, skip, visible.len());
        let carets = marker.chars().filter(|c| *c == '^').count();
        assert_eq!(carets, 7, "vạch đúng 7 char của span, không lệch theo skip");
        assert_eq!(
            marker.chars().take_while(|c| *c == ' ').count(),
            visible.len() - 7,
            "vạch bắt đầu đúng vị trí span trong phần câu đã vẽ"
        );
        assert!(v.render_plain().starts_with('…'));
    }

    // -- Bản đồ phím -------------------------------------------------------

    #[test]
    fn phim_overlay_theo_p4_0_4_2() {
        use textvn_ffi::{MOD_CTRL, MOD_SHIFT, MOD_SUPER};
        assert_eq!(overlay_action(VK_UP, 0), OverlayAction::Previous);
        assert_eq!(overlay_action(VK_DOWN, 0), OverlayAction::Next);
        assert_eq!(overlay_action(VK_RETURN, 0), OverlayAction::Apply);
        // Shift+Enter là xuống dòng, không phải áp dụng.
        assert_eq!(overlay_action(VK_RETURN, MOD_SHIFT), OverlayAction::Dismiss);
        assert_eq!(overlay_action(VK_ESCAPE, 0), OverlayAction::Close);
        // Alt+1..3 (hàng số + numpad).
        assert_eq!(overlay_action(0x31, MOD_ALT), OverlayAction::Suggestion(0));
        assert_eq!(overlay_action(0x33, MOD_ALT), OverlayAction::Suggestion(2));
        assert_eq!(overlay_action(0x62, MOD_ALT), OverlayAction::Suggestion(1));
        // Alt+4 không phải gợi ý (chỉ 1..3).
        assert_eq!(overlay_action(0x34, MOD_ALT), OverlayAction::Dismiss);
        // Ctrl+Shift+G gọi lại.
        assert_eq!(
            overlay_action(VK_G, MOD_CTRL | MOD_SHIFT),
            OverlayAction::Recall
        );
        // Ctrl+G / Alt+G / Ctrl+Alt+Shift+G không phải gọi lại.
        assert_eq!(overlay_action(VK_G, MOD_CTRL), OverlayAction::Dismiss);
        assert_eq!(overlay_action(VK_G, MOD_ALT), OverlayAction::Dismiss);
        // Caps Lock không làm mất phím.
        assert_eq!(
            overlay_action(VK_DOWN, textvn_ffi::MOD_CAPS),
            OverlayAction::Next
        );
        // Win/khác: đóng để phím đi tiếp (gõ tiếp = tự tắt).
        assert_eq!(overlay_action(0x41, 0), OverlayAction::Dismiss);
        assert_eq!(overlay_action(VK_UP, MOD_SUPER), OverlayAction::Dismiss);
    }

    // -- A2 ----------------------------------------------------------------

    /// Số xoá = **glyph** do core tính trên phần đuôi, không phải `chars`
    /// (D27: bảng mã xuất VNI có thể nhiều code unit/chữ).
    #[test]
    fn ke_hoach_a2_dung_glyph_va_giu_duoi() {
        // Lỗi sai dấu giữa câu: thay từ ở span, đuôi gõ lại nguyên văn.
        let v = view(
            "Hôm nay nghiã là vậy .",
            vec![issue(8, 13, "Sai dấu", Some("nghĩa"))],
        );
        let plan = plan_fix(&v, 0, |tail| tail.chars().count()).expect("plan");
        assert_eq!(plan.issue_index, 0);
        assert_eq!(plan.suggestion, "nghĩa");
        // Xoá từ issue.start tới hết câu (Backspace chỉ về bên trái caret).
        assert_eq!(plan.delete_chars, v.sentence.len() - 8);
        assert_eq!(plan.delete_glyphs as usize, plan.delete_chars);
        assert_eq!(
            plan.replacement.iter().collect::<String>(),
            "nghĩa là vậy .",
            "gợi ý + đuôi nguyên văn: chỉ MỘT từ đổi nội dung"
        );

        // Glyph khác char: core trả 2x → kế hoạch mang đúng số glyph.
        let plan2 = plan_fix(&v, 0, |tail| tail.chars().count() * 2).expect("plan");
        assert_eq!(plan2.delete_glyphs as usize, plan2.delete_chars * 2);
    }

    /// Lặp từ: gợi ý trùng từ đang lặp ⇒ xoá lần lặp + 1 space trước (Fix::Delete
    /// của engine), không "thay thế bằng chính nó" (vô nghĩa).
    #[test]
    fn a2_lap_tu_xoa_dung_lan_lap() {
        let v = view(
            "Hôm nay tôi tôi đi học .",
            vec![issue(12, 15, "Lặp từ: \"tôi\"", Some("tôi"))],
        );
        let plan = plan_fix(&v, 0, |tail| tail.chars().count()).expect("plan");
        assert_eq!(plan.delete_chars, v.sentence.len() - 11); // từ space(11) tới hết
        assert_eq!(
            plan.replacement.iter().collect::<String>(),
            " đi học .",
            "xoá ' tôi' của lần lặp, giữ nguyên phần đuôi"
        );
    }

    #[test]
    fn khong_sua_khi_thieu_goi_y_hoac_goi_y_khong_doi() {
        let v = view("một hai ba", vec![issue(0, 3, "a", None)]);
        assert_eq!(plan_fix(&v, 0, |t| t.chars().count()), None);
        assert_eq!(plan_fix(&v, 7, |t| t.chars().count()), None);

        // Gợi ý trùng nội dung đang có → không đụng document.
        let same = view("một hai", vec![issue(0, 3, "a", Some("một"))]);
        assert_eq!(plan_fix(&same, 0, |t| t.chars().count()), None);

        // Gợi ý chứa ký tự điều khiển (xuống dòng) → từ chối.
        let ctrl = view("một hai", vec![issue(0, 3, "a", Some("một\nhai"))]);
        assert_eq!(plan_fix(&ctrl, 0, |t| t.chars().count()), None);

        // core không đếm được glyph → fail-safe.
        let ok = view("một hai", vec![issue(0, 3, "a", Some("Một"))]);
        assert_eq!(plan_fix(&ok, 0, |_| 0), None);
    }

    #[test]
    fn khong_tu_sua_loi_chi_goi_y() {
        // Spelling: có gợi ý nhưng `auto_fixable = false` ⇒ A2 **không** tự sửa
        // (đúng `assist::fix::apply_auto_fixes` — chỉ áp lỗi auto_fixable).
        let mut suggestion_only = issue(0, 4, "Không có trong từ điển", Some("một"));
        suggestion_only.auto_fixable = false;
        let v = view("môt hai", vec![suggestion_only]);
        assert_eq!(plan_fix(&v, 0, |t| t.chars().count()), None);
    }

    /// D12: thiếu **một** điều kiện là không sửa.
    #[test]
    fn precondition_thieu_mot_dieu_kien_la_khong_sua() {
        let all = FixPrecondition {
            selection_empty: true,
            caret_unchanged: true,
            buffer_is_ours: true,
            backspace_shift_ok: true,
        };
        assert!(can_apply_fix(all));
        for broken in [
            FixPrecondition {
                selection_empty: false,
                ..all
            },
            FixPrecondition {
                caret_unchanged: false,
                ..all
            },
            FixPrecondition {
                buffer_is_ours: false,
                ..all
            },
            FixPrecondition {
                backspace_shift_ok: false,
                ..all
            },
        ] {
            assert!(!can_apply_fix(broken));
        }
        assert!(!can_apply_fix(FixPrecondition::default()));
    }

    #[test]
    fn truncate_theo_ky_tu_khong_cat_giua_chu() {
        assert_eq!(truncate_chars("hoà thuỷ", 10), "hoà thuỷ");
        assert_eq!(truncate_chars("hoà thuỷ", 4), "hoà…");
        assert_eq!(truncate_chars("hoà thuỷ", 0), "…");
        assert_eq!(truncate_chars("hoà", 1), "…");
    }
}
