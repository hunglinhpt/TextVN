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
//! - **Q-2**: ngữ nghĩa `Fix` (xoá span hay thay span) quyết định theo **kind**
//!   FFI (`IME_ASSIST_KIND_*`) — **không** đoán bằng text; đoán bằng text làm
//!   L6 ("chữ thường đầu câu") **xoá nhầm** 1 ký tự khi lỗi nằm ở vị trí > 0.
//!   Khớp Linux (`adapters/linux-common/src/compose.c`) + macOS.
//! - **D38**: định dạng hiển thị lấy chuẩn từ `textvn-cli assist show`
//!   (câu + vạch `^` đúng span + thông điệp + gợi ý) — xem
//!   [`OverlayView::render_plain`].
//! - **D16**: crate `assist` **không** đọc config ⇒ **luôn** trả lỗi L6 ("chữ
//!   thường đầu câu"). Lớp adapter (chỗ đọc được config) **loại** L6 khi
//!   `config.auto_capitalize=true` — xem
//!   [`remove_auto_capitalize_conflicts`] (lọc **sau** khi engine đã check).

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
    /// Kind FFI (`IME_ASSIST_KIND_*`) — **quyết định CÁCH sửa**: `Fix::Delete`
    /// (xoá span) hay `Fix::Replace` (thay span bằng gợi ý).
    ///
    /// Không được đoán bằng **text** (`span.to_lowercase() == suggest.to_lowercase()`):
    /// L6 ("chữ thường đầu câu") có span = 1 chữ thường, gợi ý = 1 chữ hoa ⇒
    /// `"b" == "B"` ⇒ rơi vào nhánh **XOÁ** ⇒ **mất 1 ký tự** (Q-2). Linux làm
    /// đúng (`adapters/linux-common/src/compose.c` — dùng `kind`).
    pub kind: u32,
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
                kind: raw.kind,
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

/// Ứng viên sửa cho **một** lỗi — `(delete_from, delete_end, replacement)`.
///
/// Nguồn **duy nhất** cho cả A2 ([`plan_fix`]) + A3 ([`plan_sentence_auto_fix`]) —
/// khớp crate `assist` (`assist/src/fix.rs`) và Linux
/// (`adapters/linux-common/src/compose.c`, làm đúng — dùng `kind`):
/// - `DUPLICATE_WORD` (+ space liền trước) ⇒ **xoá** lần lặp **+ 1 space** đó.
/// - `SPACE_BEFORE_PUNCT` ⇒ **xoá** span (khoảng trắng thừa); `suggest` chỉ là
///   đầu câu để **hiển thị** (chèn ⇒ nhân đôi dấu câu "chào bạn,,").
/// - còn lại ⇒ **thay** span bằng `suggest` (`Fix::Replace`).
///
/// `None` = span đã đúng ⇒ không có thay đổi ⇒ không sửa.
///
/// Vì sao **không** đoán bằng text (`span.to_lowercase() == suggest.to_lowercase()`):
/// L6 ("chữ thường đầu câu") có span = 1 chữ thường, gợi ý = 1 chữ hoa ⇒
/// `"b" == "B"` ⇒ rơi vào nhánh **xoá** ⇒ **mất 1 ký tự** (Q-2 — "Xin chào. bạn
/// khỏe không" → "Xin chào. ạn khỏe không").
fn fix_candidate(
    kind: u32,
    span_text: &str,
    suggestion: &str,
    start: usize,
    end: usize,
    prev_is_space: bool,
) -> Option<(usize, usize, String)> {
    if kind == textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD && prev_is_space {
        return Some((start - 1, end, String::new()));
    }
    if kind == textvn_ffi::IME_ASSIST_KIND_SPACE_BEFORE_PUNCT {
        return Some((start, end, String::new()));
    }
    if span_text == suggestion {
        return None;
    }
    Some((start, end, suggestion.to_string()))
}

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
    let span_text: String = view.sentence[issue.start..issue.end].iter().collect();
    let prev_is_space = issue.start > 0 && view.sentence[issue.start - 1] == ' ';
    // Ngữ nghĩa `Fix` theo **kind** (xoá span hay thay span) — xem [`fix_candidate`].
    // Quyết định này KHÔNG được đoán bằng text: so sánh
    // `span.to_lowercase() == suggestion.to_lowercase()` cho L6 ("b" vs "B") ⇒
    // rơi vào nhánh xoá ⇒ **mất 1 ký tự** (Q-2).
    let Some((delete_from, delete_end, replacement_text)) = fix_candidate(
        issue.kind,
        &span_text,
        suggestion,
        issue.start,
        issue.end,
        prev_is_space,
    ) else {
        // Span đã đúng (gợi ý trùng nội dung đang có) ⇒ không có thay đổi ⇒
        // không đụng document.
        return None;
    };
    // A2: xoá tới hết câu rồi gõ lại = phần thay thế + đuôi giữ nguyên.
    let mut replacement: Vec<char> = replacement_text.chars().collect();
    replacement.extend_from_slice(&view.sentence[delete_end..]);

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
// A3 — tự sửa cả câu bằng Backspace+Type (Mode 2 / AS-050 · D12 · D22 · D27)
// ---------------------------------------------------------------------------

/// Kế hoạch tự sửa cả câu (A3 — Mode 2 / AS-050).
///
/// Áp dụng cho các lỗi objective khi `auto_fix == "safe"`.
/// Tìm đoạn khác biệt sớm nhất (`delete_from`), xoá từ đó tới hết câu và gõ lại
/// phiên bản câu đã được sửa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceFixPlan {
    /// Số lượng lỗi objective được sửa.
    pub fixed_count: usize,
    /// Vị trí ký tự char đầu tiên bị thay đổi trong câu (0-based).
    pub delete_from: usize,
    /// Số ký tự char trong câu bị xoá (từ delete_from tới hết câu).
    pub delete_chars: usize,
    /// Số glyph tương ứng cần xoá lùi trong document (theo bảng mã xuất của core).
    pub delete_glyphs: u32,
    /// Ký tự gõ lại (đuôi câu mới đã sửa từ delete_from).
    pub replacement: Vec<char>,
    /// Đuôi câu gốc bị xoá (dùng để hoàn tác Undo 1 bước).
    pub original_tail: String,
}

/// Tính kế hoạch sửa cả câu cho các lỗi objective (A3 — Mode 2).
///
/// `sentence`: chuỗi câu gốc từ buffer core (UTF-8).
/// `issues`: danh sách lỗi từ engine.
/// `glyph_len`: hàm tính độ dài glyph của chuỗi (từ `Engine::glyph_len_of`).
pub fn plan_sentence_auto_fix<F>(
    sentence: &str,
    issues: &[IssueView],
    glyph_len: F,
) -> Option<SentenceFixPlan>
where
    F: Fn(&str) -> usize,
{
    let chars: Vec<char> = sentence.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // R-7 (G15 — một quyết định, một nơi): ngữ nghĩa "sửa kiểu gì, xoá đâu"
    // nằm DUY NHẤT ở `textvn_ffi::plan_issues_from_raw` (bản Rust duy nhất của
    // `ime_assist_plan_sentence_fix` — cùng bản mà macOS/Linux đã dùng). Adapter
    // chỉ quy đổi `IssueView` (mô hình overlay) sang biên chung:
    // - `message` không đi vào kế hoạch (`plan_sentence_fix` chỉ xem `fix`);
    // - `confidence` của Windows luôn 100 (IssueView không giữ).
    let raws: Vec<textvn_ffi::PlanIssueRaw> = issues
        .iter()
        .map(|issue| textvn_ffi::PlanIssueRaw {
            kind: issue.kind,
            start: issue.start,
            end: issue.end,
            auto_fixable: issue.auto_fixable,
            confidence: 100,
            suggest: issue.suggestion.clone().unwrap_or_default(),
        })
        .collect();
    let core_issues: Vec<textvn_assist::Issue> = textvn_ffi::plan_issues_from_raw(sentence, &raws);

    let p = textvn_assist::plan_sentence_fix(sentence, &core_issues)?;
    let glyphs = glyph_len(&p.original_tail) as u32;
    if glyphs == 0 {
        return None;
    }

    Some(SentenceFixPlan {
        fixed_count: p.fixed_count,
        delete_from: p.delete_from,
        delete_chars: p.delete_chars,
        delete_glyphs: glyphs,
        replacement: p.replacement.chars().collect(),
        original_tail: p.original_tail,
    })
}

/// Trạng thái hoàn tác (Undo) của lần tự sửa câu A3 gần nhất (giữ trong 5 giây).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistUndoState {
    /// Thời điểm áp dụng tự sửa (để hết hạn sau 5 giây theo spec P4-0 §4.3).
    pub timestamp: std::time::Instant,
    /// Đoạn câu gốc đã bị xoá.
    pub original_tail: String,
    /// Text **đã gõ lại** sau khi sửa (Unicode) — dùng để SO vùng sắp xoá khi
    /// hoàn tác (R-3 · fail-safe): nếu vùng đó không còn là text ta vừa gõ
    /// (người dùng gõ thêm / app tự sửa) ⇒ **không xoá**.
    pub replacement: String,
    /// Số glyph đã gõ lại sau khi sửa.
    pub replacement_glyphs: u32,
    /// Số lỗi đã được sửa trong lần này.
    pub fixed_count: usize,
}

impl AssistUndoState {
    pub fn new(
        original_tail: String,
        replacement: String,
        replacement_glyphs: u32,
        fixed_count: usize,
    ) -> Self {
        Self {
            timestamp: std::time::Instant::now(),
            original_tail,
            replacement,
            replacement_glyphs,
            fixed_count,
        }
    }

    /// Kiểm tra trạng thái Undo có còn hiệu lực không (trong vòng 5 giây).
    pub fn is_valid(&self) -> bool {
        self.timestamp.elapsed() <= std::time::Duration::from_secs(5)
    }
}

/// **Ngữ nghĩa Mode 2 duy nhất cho 3 OS** (R-5 · P4-0 §4.3):
/// tự sửa cả câu chỉ bật khi `assist.mode == "autocorrect"` **và**
/// `assist.auto_fix == "safe"` (chỉ luật objective — D7).
///
/// Hàm thuần — test được không cần TSF. macOS/Linux dùng cùng quy tắc (viết
/// lại theo ngôn ngữ của từng adapter, cùng test).
pub fn mode2_auto_fix_enabled(
    mode: textvn_config::AssistMode,
    auto_fix: textvn_config::AssistAutoFix,
) -> bool {
    mode == textvn_config::AssistMode::Autocorrect && auto_fix == textvn_config::AssistAutoFix::Safe
}

/// **Điểm hiển thị (Mode 1) bật khi `assist.mode != "off"`** (R-6 · P4-0 §4.3):
/// ở Mode 2, lỗi **mơ hồ** vẫn phải hiện candidate (chỉ lỗi objective mới tự
/// sửa) ⇒ Mode 1 không độc lập với Mode 2.
pub fn mode1_display_enabled(mode: textvn_config::AssistMode) -> bool {
    mode != textvn_config::AssistMode::Off
}

/// Quyết định xem có nên tự sửa cả câu (A3 / Mode 2) tại thời điểm trigger hay không.
///
/// R-5: `auto_fix_enabled` phải là cờ **Mode 2 đầy đủ** (`mode == "autocorrect"`
/// **và** `auto_fix == "safe"` — xem [`mode2_auto_fix_enabled`]) và trigger phải là
/// **kết thúc câu** (`.` `!` `?`); space/,/: chỉ là điểm kiểm tra silent (D11).
pub fn should_auto_fix_sentence(
    trigger: AssistTrigger,
    assist_enabled: bool,
    auto_fix_enabled: bool,
) -> bool {
    assist_enabled && auto_fix_enabled && trigger == AssistTrigger::Sentence
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
#[derive(Clone)]
pub struct AssistCheck {
    /// Câu đã cắt theo [`SENTENCE_MAX_CHARS`] — đúng chuỗi engine vừa kiểm tra
    /// (UTF-16 → `String`, char Unicode).
    pub sentence: String,
    /// Lỗi của lần kiểm tra này (offset **char Unicode** — D27).
    pub result: textvn_ffi::ime_assist_result_v1,
}

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

// ---------------------------------------------------------------------------
// D16 — lỗi L6 ("chữ thường đầu câu") vs `config.auto_capitalize`
// ---------------------------------------------------------------------------

/// Kind FFI của L6 (`IME_ASSIST_KIND_LOWERCASE_SENTENCE`).
pub const ASSIST_KIND_LOWERCASE_SENTENCE: u32 = textvn_ffi::IME_ASSIST_KIND_LOWERCASE_SENTENCE;

/// **D16** (Fassist-20) — crate `assist` **không** đọc config (0-dep, không I/O)
/// ⇒ nó **luôn** trả về lỗi L6 ("chữ thường đầu câu"). Lớp **adapter** (chỗ
/// đọc được config) phải **loại** L6 khi `config.auto_capitalize=true`: khi đó
/// IME/app **đã tự viết hoa** ⇒ báo L6 là **trùng chức năng**.
///
/// Lọc **sau** khi engine đã trả kết quả (không lọc trước khi check) ⇒ mọi
/// luật khác giữ nguyên hành vi. Trả về **số lỗi đã loại**.
pub fn remove_auto_capitalize_conflicts(
    result: &mut textvn_ffi::ime_assist_result_v1,
    auto_capitalize: bool,
) -> usize {
    if !auto_capitalize {
        return 0;
    }
    let reported = (result.count as usize).min(result.issues.len());
    let mut kept = 0usize;
    for idx in 0..reported {
        let issue = result.issues[idx];
        if issue.kind == ASSIST_KIND_LOWERCASE_SENTENCE {
            continue; // trùng chức năng với `auto_capitalize` → bỏ
        }
        result.issues[kept] = issue;
        kept += 1;
    }
    if kept == reported {
        return 0;
    }
    // Xoá phần đuôi không còn dùng (fail-safe: không để lỗi cũ sót lại).
    // SAFETY: `ime_assist_issue_v1` chỉ gồm số nguyên + mảng `c_char` ⇒
    // zero là giá trị hợp lệ (giống `ffi::fill_assist_result`).
    let zero = unsafe { std::mem::zeroed::<textvn_ffi::ime_assist_issue_v1>() };
    for slot in result.issues.iter_mut().skip(kept) {
        *slot = zero;
    }
    result.count = kept as u32;
    reported - kept
}

/// Kiểm tra câu (UTF-8) + lọc D16 — **hàm thuần**, test được mọi OS.
///
/// - `auto_capitalize`: `config.auto_capitalize` (D16) — bật ⇒ lỗi L6 bị loại.
/// - `None` = câu quá ngắn / câu sạch (kể cả sau khi lọc D16) / lỗi FFI ⇒
///   **im lặng** (D11: "kết quả im lặng").
///
/// Biên FFI chuẩn UTF-16 (D29): encode câu sang UTF-16 rồi lần lượt đi qua
/// `ime_assist_sentence_tail_v1` (cắt đuôi tối đa [`SENTENCE_MAX_CHARS`] char,
/// không chẻ surrogate) và `ime_assist_check_v1` (đếm lỗi).
pub fn check_text(sentence: &str, auto_capitalize: bool) -> Option<AssistCheck> {
    use textvn_ffi::{
        ime_assist_check_v1, ime_assist_sentence_tail_v1, IME_ABI_VERSION, IME_MAX_ASSIST_ISSUES,
        IME_MAX_ASSIST_MSG, IME_MAX_ASSIST_SUGGEST,
    };

    let began = std::time::Instant::now();
    if word_count(sentence) < MIN_SENTENCE_WORDS {
        return None; // câu < 3 từ → bỏ qua (D11)
    }
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
    // D16: L6 ("chữ thường đầu câu") bị loại khi `auto_capitalize=true` —
    // lọc **sau** khi engine đã trả kết quả ⇒ các luật khác giữ nguyên hành vi.
    remove_auto_capitalize_conflicts(&mut result, auto_capitalize);
    if result.count == 0 {
        return None; // chỉ có L6 ⇒ câu sạch với config hiện tại ⇒ im lặng
    }
    Some(AssistCheck {
        sentence: checked,
        result,
    })
}

/// Chạy kiểm tra câu trong ngân sách [`CHECK_BUDGET_MS`] — **không** side effect
/// UI. `None` = tắt assist / câu ngắn / câu sạch / vượt ngân sách (bỏ im lặng).
#[cfg(windows)]
pub fn check_sentence(shared: &crate::edit_session::TsfShared) -> Option<AssistCheck> {
    // D20 kill-switch + `assist.mode` (mode=off ⇒ tắt hoàn toàn, không check,
    // không toast) — cùng ngữ nghĩa với macOS/Linux (R-5).
    if !shared.is_assist_enabled() {
        return None;
    }
    let began = std::time::Instant::now();

    // Câu vừa gõ = buffer câu của core (D8 — không đọc nội dung app khác);
    // engine giữ tối đa 256 ký tự Unicode (D31).
    //
    // D16: `auto_capitalize` đọc **cùng đường** với `assist.mode`/`assist.auto_fix`
    // (`ThreadState` — nạp `config.v1.json` từ `%APPDATA%\TextVN\config.json`).
    let (sentence, auto_capitalize, assist_app_disabled) = {
        let thread = shared.thread.try_borrow().ok()?; // fail-open: engine đang bận
                                                       // W4-audit (2026-10-23): `config.assist.per_app` — app bị tắt riêng
                                                       // ⇒ không check, không gợi ý, không tự sửa, không toast (parity với
                                                       // macOS `assistAllowed`). Mặc định (map rỗng) ⇒ không tắt gì.
        let disabled = thread.assist_app_disabled(&thread.field_context().app_id);
        (
            thread.engine.sentence_tail(SENTENCE_MAX_CHARS),
            thread.auto_capitalize(),
            disabled,
        )
    };
    if assist_app_disabled {
        return None;
    }
    if began.elapsed().as_millis() > CHECK_BUDGET_MS {
        return None; // ngân sách: bỏ im lặng
    }
    check_text(&sentence, auto_capitalize)
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

    /// W4-audit (2026-10-23): `config.assist.per_app` — app bị tắt riêng trong
    /// config ⇒ `ThreadState::assist_app_disabled` trả `true` cho app đó; map
    /// rỗng (mặc định) ⇒ không tắt gì. `check_sentence` gate theo
    /// `field_context().app_id` (host exe — chính là app đang gõ).
    #[test]
    fn per_app_tat_assist_rieng_theo_config() {
        let mut t = crate::ThreadState::new("code.exe", 0).unwrap();
        assert!(!t.assist_app_disabled("code.exe"), "mặc định: không tắt");
        let cfg: &str = r#"{"config_version":1,"assist":{"per_app":{"code.exe":false}}}"#;
        t.reload_config_bytes(cfg.as_bytes()).unwrap();
        assert!(t.assist_app_disabled("code.exe"));
        assert!(
            t.assist_app_disabled("Code.EXE"),
            "app_id so không phân biệt hoa/thường"
        );
        assert!(
            !t.assist_app_disabled("notepad.exe"),
            "app khác không bị thu hồi"
        );
        // Config khác (không có per_app) ⇒ giữ mặc định = không tắt.
        t.reload_config_bytes(br#"{"config_version":1,"assist":{"mode":"highlight"}}"#)
            .unwrap();
        assert!(!t.assist_app_disabled("code.exe"));
    }

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

    /// Kind FFI rút gọn cho test — **quyết định "xoá span" hay "thay span"**
    /// (Q-2): không đoán bằng text.
    const K_DUP: u32 = textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD;
    const K_SPACE_BEFORE: u32 = textvn_ffi::IME_ASSIST_KIND_SPACE_BEFORE_PUNCT;
    const K_DOUBLE_SPACE: u32 = textvn_ffi::IME_ASSIST_KIND_DOUBLE_SPACE;
    const K_LOWER: u32 = textvn_ffi::IME_ASSIST_KIND_LOWERCASE_SENTENCE;
    const K_DIACRITIC: u32 = textvn_ffi::IME_ASSIST_KIND_DIACRITIC_TYPO;
    const K_SPELLING: u32 = textvn_ffi::IME_ASSIST_KIND_SPELLING;

    /// Lỗi có **kind** — adapter dùng `kind` (không phải so sánh text) để quyết
    /// định cách sửa.
    fn issue(
        kind: u32,
        start: usize,
        end: usize,
        message: &str,
        suggestion: Option<&str>,
    ) -> IssueView {
        IssueView {
            start,
            end,
            message: message.to_string(),
            suggestion: suggestion.map(str::to_string),
            auto_fixable: true,
            kind,
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
            vec![issue(K_DUP, 12, 15, "Lặp từ: \"tôi\"", Some("tôi"))],
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
            issue(K_DIACRITIC, 0, 3, "a", Some("x")),
            issue(K_DIACRITIC, 4, 7, "b", Some("y")),
            issue(K_DIACRITIC, 8, 11, "c", Some("z")),
            issue(K_DIACRITIC, 12, 99, "span ngoài câu", Some("w")),
        ];
        let v = OverlayView::from_parts(chars, issues, 4).expect("view");
        assert_eq!(v.issues.len(), MAX_OVERLAY_ISSUES);
        // Span lạ bị loại trước khi cắt → tổng vẫn 4 ⇒ còn 1 lỗi ẩn.
        assert_eq!(v.hidden_count(), 1);
        assert!(v.render_plain().contains("(+1 lỗi khác)"));

        let empty = OverlayView::from_parts(
            "   ".chars().collect(),
            vec![issue(K_DIACRITIC, 0, 99, "a", None)],
            1,
        );
        assert!(empty.is_none(), "span ngoài câu ⇒ không hiện gì");
    }

    #[test]
    fn chon_loi_vong_tron_va_theo_so() {
        let mut v = view(
            "một hai ba",
            vec![
                issue(K_DIACRITIC, 0, 3, "a", Some("A")),
                issue(K_DIACRITIC, 4, 7, "b", Some("B")),
                issue(K_DIACRITIC, 8, 10, "c", Some("C")),
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
        let mut one = view("một hai", vec![issue(K_DIACRITIC, 0, 3, "a", Some("A"))]);
        assert!(!one.select(1));
        assert_eq!(one.selected, 0);
    }

    #[test]
    fn cau_dai_chi_ve_duoi_va_marker_lech_dung() {
        let sentence: String = "a ".repeat(80) + "tôi tôi";
        let len = sentence.chars().count();
        let v = view(
            &sentence,
            vec![issue(K_DUP, len - 7, len, "Lặp từ", Some("tôi"))],
        );
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
            vec![issue(K_DIACRITIC, 8, 13, "Sai dấu", Some("nghĩa"))],
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
            vec![issue(K_DUP, 12, 15, "Lặp từ: \"tôi\"", Some("tôi"))],
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
        let v = view("một hai ba", vec![issue(K_DIACRITIC, 0, 3, "a", None)]);
        assert_eq!(plan_fix(&v, 0, |t| t.chars().count()), None);
        assert_eq!(plan_fix(&v, 7, |t| t.chars().count()), None);

        // Gợi ý trùng nội dung đang có → không đụng document.
        let same = view("một hai", vec![issue(K_DIACRITIC, 0, 3, "a", Some("một"))]);
        assert_eq!(plan_fix(&same, 0, |t| t.chars().count()), None);

        // Gợi ý chứa ký tự điều khiển (xuống dòng) → từ chối.
        let ctrl = view(
            "một hai",
            vec![issue(K_DIACRITIC, 0, 3, "a", Some("một\nhai"))],
        );
        assert_eq!(plan_fix(&ctrl, 0, |t| t.chars().count()), None);

        // core không đếm được glyph → fail-safe.
        let ok = view("một hai", vec![issue(K_DIACRITIC, 0, 3, "a", Some("Một"))]);
        assert_eq!(plan_fix(&ok, 0, |_| 0), None);
    }

    #[test]
    fn khong_tu_sua_loi_chi_goi_y() {
        // Spelling: có gợi ý nhưng `auto_fixable = false` ⇒ A2 **không** tự sửa
        // (đúng `assist::fix::apply_auto_fixes` — chỉ áp lỗi auto_fixable).
        let mut suggestion_only = issue(K_SPELLING, 0, 4, "Không có trong từ điển", Some("một"));
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

    #[test]
    fn plan_sentence_auto_fix_nhieu_loi_objective() {
        let sentence = "hôm nay tôi tôi đi  học .";
        let issues = vec![
            issue(K_LOWER, 0, 3, "Chữ thường đầu câu", Some("Hôm")),
            issue(K_DUP, 12, 15, "Lặp từ: \"tôi\"", Some("tôi")),
            issue(K_DOUBLE_SPACE, 18, 20, "Khoảng trắng kép", Some(" ")),
            issue(
                K_SPACE_BEFORE,
                23,
                24,
                "Thừa khoảng trắng trước dấu câu",
                Some(""),
            ),
        ];
        let plan = plan_sentence_auto_fix(sentence, &issues, |tail| tail.chars().count())
            .expect("plan sentence");

        assert_eq!(plan.fixed_count, 4);
        assert_eq!(plan.delete_from, 0);
        assert_eq!(plan.original_tail, sentence);
        let rep_str: String = plan.replacement.iter().collect();
        assert_eq!(rep_str, "Hôm nay tôi đi học.");
    }

    #[test]
    fn plan_sentence_auto_fix_chi_sua_phan_duoi() {
        let sentence = "Tôi nghĩ rằng nghiã là đúng.";
        let issues = vec![issue(K_DIACRITIC, 14, 19, "Sai vị trí dấu", Some("nghĩa"))];
        let plan = plan_sentence_auto_fix(sentence, &issues, |tail| tail.chars().count())
            .expect("plan sentence");

        assert_eq!(plan.fixed_count, 1);
        assert_eq!(plan.delete_from, 14);
        assert_eq!(plan.original_tail, "nghiã là đúng.");
        let rep_str: String = plan.replacement.iter().collect();
        assert_eq!(rep_str, "nghĩa là đúng.");
    }

    #[test]
    fn plan_sentence_auto_fix_bo_qua_loi_chi_goi_y() {
        let sentence = "Hôm nay tôi thấy môt con mèo.";
        let mut sp_issue = issue(K_SPELLING, 17, 20, "Không có trong từ điển", Some("một"));
        sp_issue.auto_fixable = false;
        let plan = plan_sentence_auto_fix(sentence, &[sp_issue], |tail| tail.chars().count());
        assert_eq!(plan, None);
    }

    #[test]
    fn assist_undo_state_hieu_luc_va_het_han() {
        let state = AssistUndoState {
            timestamp: std::time::Instant::now(),
            original_tail: "tôi tôi đi".to_string(),
            replacement: "tôi đi".to_string(),
            replacement_glyphs: 6,
            fixed_count: 1,
        };
        assert!(state.is_valid());

        let expired = AssistUndoState {
            timestamp: std::time::Instant::now() - std::time::Duration::from_secs(6),
            ..state
        };
        assert!(!expired.is_valid());
    }

    #[test]
    fn acceptance_dod_scenarios_1_to_7() {
        // Scenario 1: Câu tiếng Việt hợp lệ -> không lỗi, không tự sửa
        let s1 = "Hôm nay trời đẹp.";
        assert_eq!(plan_sentence_auto_fix(s1, &[], |t| t.chars().count()), None);
        assert!(!should_show_overlay(AssistTrigger::Sentence, true, 0));

        // Scenario 2: Lặp từ ("tôi tôi đi học.") -> Mode 1 hiện gợi ý; Mode 2 tự sửa
        let s2 = "tôi tôi đi học.";
        let v2 = view(s2, vec![issue(K_DUP, 4, 7, "Lặp từ: \"tôi\"", Some("tôi"))]);
        assert!(v2.render_plain().contains("1. Lặp từ: \"tôi\"  -> tôi"));
        let plan2 = plan_sentence_auto_fix(s2, &v2.issues, |t| t.chars().count()).expect("plan2");
        assert_eq!(plan2.delete_from, 3);
        assert_eq!(plan2.original_tail, " tôi đi học.");
        assert_eq!(plan2.replacement.iter().collect::<String>(), " đi học.");
        // Tổng thể document: phần đầu giữ nguyên + replacement = "tôi đi học."
        let prefix: String = s2.chars().take(plan2.delete_from).collect();
        let full_doc2 = format!("{}{}", prefix, plan2.replacement.iter().collect::<String>());
        assert_eq!(full_doc2, "tôi đi học.");

        // Scenario 3: Thừa khoảng trắng trước dấu ("đi học .") -> Mode 2 tự sửa
        let s3 = "đi học .";
        let v3 = view(
            s3,
            vec![issue(K_SPACE_BEFORE, 6, 7, "Thừa khoảng trắng", Some(""))],
        );
        let plan3 = plan_sentence_auto_fix(s3, &v3.issues, |t| t.chars().count()).expect("plan3");
        assert_eq!(plan3.delete_from, 6);
        assert_eq!(plan3.original_tail, " .");
        assert_eq!(plan3.replacement.iter().collect::<String>(), ".");
        let prefix3: String = s3.chars().take(plan3.delete_from).collect();
        let full_doc3 = format!(
            "{}{}",
            prefix3,
            plan3.replacement.iter().collect::<String>()
        );
        assert_eq!(full_doc3, "đi học.");

        // Scenario 4: Chữ thường đầu câu ("hôm nay tôi đi học.") -> Mode 2 tự sửa
        let s4 = "hôm nay tôi đi học.";
        let v4 = view(
            s4,
            vec![issue(K_LOWER, 0, 1, "Chữ thường ở đầu câu", Some("H"))],
        );
        let plan4 = plan_sentence_auto_fix(s4, &v4.issues, |t| t.chars().count()).expect("plan4");
        assert_eq!(
            plan4.replacement.iter().collect::<String>(),
            "Hôm nay tôi đi học."
        );

        // Scenario 5: Hoàn tác 1 bước trong 5s sau khi auto-fix -> Khôi phục chính xác đuôi câu cũ
        let undo = AssistUndoState::new(
            s4.to_string(),
            plan4.replacement.iter().collect(),
            plan4.delete_glyphs,
            plan4.fixed_count,
        );
        assert!(undo.is_valid());
        assert_eq!(undo.original_tail, s4);

        // Scenario 6: Hết hạn hoàn tác sau 5s -> Không hoàn tác
        let expired = AssistUndoState {
            timestamp: std::time::Instant::now() - std::time::Duration::from_secs(6),
            ..undo.clone()
        };
        assert!(!expired.is_valid());

        // Scenario 7: Precondition D12: Di chuyển con trỏ hoặc bôi đen -> Không kích hoạt
        let valid_pre = FixPrecondition {
            selection_empty: true,
            caret_unchanged: true,
            buffer_is_ours: true,
            backspace_shift_ok: true,
        };
        assert!(can_apply_fix(valid_pre));
        // Bôi đen văn bản
        assert!(!can_apply_fix(FixPrecondition {
            selection_empty: false,
            ..valid_pre
        }));
        // Con trỏ di chuyển
        assert!(!can_apply_fix(FixPrecondition {
            caret_unchanged: false,
            ..valid_pre
        }));
        // Buffer không còn là của ta (gõ xen)
        assert!(!can_apply_fix(FixPrecondition {
            buffer_is_ours: false,
            ..valid_pre
        }));
    }

    // ------------------------------------------------------------------
    // Q-2 (BLOCKER) — lỗi L6 ở vị trí > 0 không được xoá nhầm 1 ký tự
    // ------------------------------------------------------------------

    /// Chạy engine **thật** (FFI) trên 1 câu → `OverlayView` (như đường gõ thật).
    fn check_via_engine(sentence: &str) -> OverlayView {
        let utf16: Vec<u16> = sentence.encode_utf16().collect();
        let mut result = textvn_ffi::ime_assist_result_v1 {
            abi_version: textvn_ffi::IME_ABI_VERSION,
            count: 0,
            issues: [textvn_ffi::ime_assist_issue_v1 {
                kind: 0,
                start: 0,
                end: 0,
                auto_fixable: 0,
                confidence: 0,
                message: [0; textvn_ffi::IME_MAX_ASSIST_MSG],
                suggest_len: 0,
                suggest: [0; textvn_ffi::IME_MAX_ASSIST_SUGGEST],
            }; textvn_ffi::IME_MAX_ASSIST_ISSUES],
        };
        let count =
            textvn_ffi::ime_assist_check_v1(utf16.as_ptr(), utf16.len() as i32, &mut result);
        assert!(count > 0, "engine phải bắt được lỗi trong câu {sentence:?}");
        OverlayView::from_ffi(&result, sentence).expect("view")
    }

    /// Q-2 (BLOCKER): lỗi L6 ("chữ thường đầu câu") nằm ở vị trí **> 0** ⇒
    /// adapter **XOÁ nhầm** 1 ký tự nếu quyết định "xoá span" bằng so sánh
    /// **text** (`span.to_lowercase() == suggest.to_lowercase()` ⇒ `"b" == "B"`).
    ///
    /// Hồi quy (case có sẵn trong `corpus/assist/vi_basic.ass`): "Xin chào. bạn
    /// khỏe không" → span `@10-11` ⇒ kết quả phải là **"Xin chào. Bạn khỏe
    /// không"**, KHÔNG phải "Xin chào. ạn khỏe không" (mất 1 ký tự).
    #[test]
    fn q2_l6_o_vi_tri_0_khong_xoa_nham_ky_tu() {
        let sentence = "Xin chào. bạn khỏe không";
        let view = check_via_engine(sentence);

        // Lỗi L6 của engine: kind = LOWERCASE_SENTENCE, span 10..11, gợi ý "B".
        let l6 = view
            .issues
            .iter()
            .find(|i| i.kind == textvn_ffi::IME_ASSIST_KIND_LOWERCASE_SENTENCE)
            .expect("L6");
        assert_eq!((l6.start, l6.end), (10, 11));
        assert_eq!(l6.suggestion.as_deref(), Some("B"));

        let plan =
            plan_sentence_auto_fix(sentence, &view.issues, |t| t.chars().count()).expect("plan");
        let prefix: String = sentence.chars().take(plan.delete_from).collect();
        let fixed = format!("{prefix}{}", plan.replacement.iter().collect::<String>());
        assert_eq!(
            fixed, "Xin chào. Bạn khỏe không",
            "L6 ở vị trí > 0 phải THAY ký tự, không được XOÁ (Q-2)"
        );
        assert_eq!(plan.delete_from, 10);
    }

    /// Q-2: cùng lỗi nhưng qua **A2** (sửa 1 lỗi) cũng phải **thay** span.
    #[test]
    fn q2_a2_thay_span_cho_l6_khong_xoa() {
        let sentence = "Xin chào. bạn khỏe không";
        let view = check_via_engine(sentence);
        let index = view
            .issues
            .iter()
            .position(|i| i.kind == textvn_ffi::IME_ASSIST_KIND_LOWERCASE_SENTENCE)
            .expect("L6");
        let plan = plan_fix(&view, index, |t| t.chars().count()).expect("plan A2");
        assert_eq!(plan.delete_chars, "bạn khỏe không".chars().count());
        assert_eq!(
            plan.replacement.iter().collect::<String>(),
            "Bạn khỏe không",
            "A2 cũng phải THAY ký tự, không được XOÁ (Q-2)"
        );
    }

    /// Q-2: lặp từ vẫn được **xoá** đúng (không đổi hành vi cũ) — kind
    /// `DUPLICATE_WORD` ⇒ xoá lần lặp + 1 space liền trước.
    #[test]
    fn q2_lap_tu_van_xoa_dung_lan_lap() {
        let sentence = "Hôm nay tôi tôi đi học .";
        let view = check_via_engine(sentence);
        let index = view
            .issues
            .iter()
            .position(|i| i.kind == textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD)
            .expect("lặp từ");
        let plan = plan_fix(&view, index, |t| t.chars().count()).expect("plan A2");
        assert_eq!(plan.delete_chars, view.sentence.len() - 11);
        assert_eq!(
            plan.replacement.iter().collect::<String>(),
            " đi học .",
            "xoá ' tôi' của lần lặp, giữ nguyên phần đuôi"
        );
    }

    // ------------------------------------------------------------------
    // Q-5 (MAJOR) — không đếm được glyph thì KHÔNG sửa
    // ------------------------------------------------------------------

    /// Q-5: khi core **không đếm được glyph** (FFI lỗi ⇒ `glyph_len_of` trả 0)
    /// ⇒ kế hoạch bị **từ chối** ⇒ **không sửa** document. TUYỆT ĐỐI không
    /// fallback `chars().count()` (VNI/TCVN3: 1 chữ = nhiều code unit ⇒ xoá lệch).
    #[test]
    fn q5_khong_dem_duoc_glyph_thi_khong_sua() {
        let sentence = "hôm nay tôi tôi đi học.";
        let issues = vec![issue(K_DUP, 12, 15, "Lặp từ: \"tôi\"", Some("tôi"))];
        assert_eq!(
            plan_sentence_auto_fix(sentence, &issues, |_| 0),
            None,
            "glyph = 0 ⇒ không sửa (fail-safe Q-5)"
        );
        let view = view(sentence, issues);
        assert_eq!(plan_fix(&view, 0, |_| 0), None, "A2: glyph = 0 ⇒ không sửa");
    }

    // ------------------------------------------------------------------
    // D16 (Fassist-20) — L6 ("chữ thường đầu câu") vs `auto_capitalize`
    // ------------------------------------------------------------------

    /// Câu có sẵn trong corpus `corpus/assist/vi_basic.ass`:
    /// "Xin chào. bạn khỏe không" → `lowercase_sentence_start@10-11`.
    const CORPUS_L6_SENTENCE: &str = "Xin chào. bạn khỏe không";

    fn kinds_of(check: &AssistCheck) -> Vec<u32> {
        check
            .result
            .issues
            .iter()
            .take(check.issue_count())
            .map(|i| i.kind)
            .collect()
    }

    /// D16: `auto_capitalize=true` ⇒ **không** còn lỗi L6 (IME/app đã tự viết
    /// hoa ⇒ báo L6 là **trùng chức năng**). Câu này chỉ có lỗi L6 ⇒ sau khi
    /// lọc, adapter phải **im lặng** (D11: kết quả im lặng).
    #[test]
    fn d16_auto_capitalize_bat_thi_khong_con_loi_l6() {
        // Trước khi lọc: engine (crate `assist`, không đọc config) **luôn** trả L6.
        let raw = check_text(CORPUS_L6_SENTENCE, false).expect("engine phải bắt được lỗi");
        assert!(
            kinds_of(&raw).contains(&ASSIST_KIND_LOWERCASE_SENTENCE),
            "crate assist luôn trả L6 (không đọc config)"
        );

        // Sau khi lọc (auto_capitalize=true): không còn lỗi nào ⇒ im lặng.
        assert!(
            check_text(CORPUS_L6_SENTENCE, true).is_none(),
            "auto_capitalize=true ⇒ L6 bị loại ⇒ câu sạch ⇒ im lặng"
        );
    }

    /// D16: `auto_capitalize=false` ⇒ lỗi L6 **vẫn còn** (engine không tự viết
    /// hoa ⇒ không ai loại giúp ⇒ trợ lý ngữ pháp phải báo).
    #[test]
    fn d16_auto_capitalize_tat_thi_van_con_loi_l6() {
        let check = check_text(CORPUS_L6_SENTENCE, false).expect("engine phải bắt được lỗi");
        assert!(
            kinds_of(&check).contains(&ASSIST_KIND_LOWERCASE_SENTENCE),
            "auto_capitalize=false ⇒ L6 phải còn"
        );
        // Span L6 = 10..11 (char Unicode) — chữ "b" trong "bạn".
        let l6 = check
            .result
            .issues
            .iter()
            .take(check.issue_count())
            .find(|i| i.kind == ASSIST_KIND_LOWERCASE_SENTENCE)
            .expect("L6");
        assert_eq!((l6.start, l6.end), (10, 11));
        // Còn hiện được trong overlay (Mode 1) — lọc L6 không phá view.
        assert!(check.view().is_some());
    }

    /// D16: cờ `auto_capitalize` **chỉ** loại L6 — các luật khác giữ nguyên
    /// (lọc **sau** khi engine đã check ⇒ không đổi hành vi luật khác).
    #[test]
    fn d16_chi_loai_l6_luat_khac_giu_nguyen() {
        let sentence = "Xin chào. tôi tôi đi học";
        let raw = check_text(sentence, false).expect("engine phải bắt được lỗi");
        assert!(kinds_of(&raw).contains(&ASSIST_KIND_LOWERCASE_SENTENCE));
        assert!(kinds_of(&raw).contains(&textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD));

        let filtered = check_text(sentence, true).expect("còn lỗi lặp từ");
        assert!(
            !kinds_of(&filtered).contains(&ASSIST_KIND_LOWERCASE_SENTENCE),
            "L6 phải bị loại"
        );
        assert!(
            kinds_of(&filtered).contains(&textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD),
            "lặp từ phải còn (chỉ L6 bị loại)"
        );
    }

    /// D16: hàm lọc thuần — `auto_capitalize=false` ⇒ **không** đụng gì.
    #[test]
    fn d16_loc_thuan_khi_tat_capitalize_thi_giu_nguyen() {
        let mut result = textvn_ffi::ime_assist_result_v1 {
            abi_version: textvn_ffi::IME_ABI_VERSION,
            count: 0,
            issues: [textvn_ffi::ime_assist_issue_v1 {
                kind: 0,
                start: 0,
                end: 0,
                auto_fixable: 0,
                confidence: 0,
                message: [0; textvn_ffi::IME_MAX_ASSIST_MSG],
                suggest_len: 0,
                suggest: [0; textvn_ffi::IME_MAX_ASSIST_SUGGEST],
            }; textvn_ffi::IME_MAX_ASSIST_ISSUES],
        };
        result.count = 2;
        result.issues[0].kind = ASSIST_KIND_LOWERCASE_SENTENCE;
        result.issues[1].kind = textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD;

        assert_eq!(remove_auto_capitalize_conflicts(&mut result, false), 0);
        assert_eq!(result.count, 2, "tắt auto_capitalize ⇒ giữ nguyên mọi lỗi");

        assert_eq!(remove_auto_capitalize_conflicts(&mut result, true), 1);
        assert_eq!(result.count, 1, "chỉ còn lỗi lặp từ");
        assert_eq!(
            result.issues[0].kind,
            textvn_ffi::IME_ASSIST_KIND_DUPLICATE_WORD
        );
        assert_eq!(
            result.issues[1].kind, 0,
            "phần đuôi không dùng phải được xoá (fail-safe)"
        );
    }
}
