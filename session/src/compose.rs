// SPDX-License-Identifier: GPL-3.0-or-later
//! Khối `compose` dùng chung 3 OS (P0-6 §5, task SF-3): kế hoạch preedit/commit cho một phím.
//!
//! Nguồn hợp nhất: `adapters/windows-tsf/src/compose.rs` (`plan_key`/`text_after`/
//! `CompositionPlan`) và `adapters/linux-common/src/compose.c` (`lc_plan_key`). Hàm thuần, không
//! I/O, không gọi engine — adapter (hoặc session ở các bước sau) đưa kết quả engine vào dưới dạng
//! [`EngineStep`].
//!
//! Luật P0-6 §5 hàng *compose* (mỗi luật có test ở `session/tests/compose.rs`):
//! - ranh giới in được khi đang soạn: [`BoundaryPolicy::CommitWithWord`] (TSF/CUAS — tránh
//!   `" được"`) hoặc [`BoundaryPolicy::ForwardKey`] (IBus/Fcitx5 — commit từ rồi app nhận phím);
//!   khác biệt theo năng lực OS, ghi ở `corpus/traces/ALLOWED_DIFF.md` (`CUAS_PRINTABLE_BOUNDARY`);
//! - Enter/Tab trả phím thật, bỏ `\n`/`\t` khỏi text (B2);
//! - `PASS && CONSUMED` ⇒ ăn im lặng (E62 Math);
//! - Ctrl+Space không giữ composition (R2-121);
//! - action lạ (ABI mới hơn) ⇒ commit nguyên văn, phím đi thẳng.

use textvn_core::keymap::{vk, MOD_ALT, MOD_CAPS, MOD_CTRL, MOD_SHIFT};
use textvn_core::{ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE};
use textvn_core::{FLAG_CONSUMED, FLAG_WORD_END};

/// Phân loại phím sau khi adapter đã dịch phím → ký tự theo layout bàn phím.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// Phím sinh ký tự in được (gồm Space).
    Char(char),
    /// Backspace.
    Backspace,
    /// Esc.
    Escape,
    /// Enter.
    Enter,
    /// Tab.
    Tab,
    /// Ctrl+Space (trigger Text Prediction & Text Correct).
    CtrlSpace,
    /// Tổ hợp AltGr (`Ctrl+Alt+<phím in được>`) dành cho Math & Science:
    /// khớp biểu tượng thì thay thế, không khớp thì pass-through nguyên phím.
    AltGr(char),
    /// Không sinh ký tự: mũi tên, Delete, Home/End, F-key, PageUp…
    Other,
}

impl KeyKind {
    /// `ch` = ký tự của phím theo layout (0 nếu không sinh ký tự). Ký tự điều khiển
    /// (`\r`, `\t`, `\b`, ESC) được quy về phím tương ứng; phím không sinh ký tự tuyệt đối
    /// không được coi là chữ.
    #[must_use]
    pub fn classify(vk: u32, ch: u32) -> KeyKind {
        match vk {
            vk::BACK => return KeyKind::Backspace,
            vk::TAB => return KeyKind::Tab,
            vk::RETURN => return KeyKind::Enter,
            vk::ESCAPE => return KeyKind::Escape,
            vk::PACKET | vk::PROCESSKEY => return KeyKind::Other,
            _ => {}
        }
        match char::from_u32(ch) {
            Some('\n' | '\r') => KeyKind::Enter,
            Some('\t') => KeyKind::Tab,
            Some('\u{8}') => KeyKind::Backspace,
            Some('\u{1b}') => KeyKind::Escape,
            Some(c) if ch != 0 && !c.is_control() => KeyKind::Char(c),
            _ => KeyKind::Other,
        }
    }

    /// Phân loại kèm modifier: nhận diện Ctrl+Space và AltGr (Ctrl+Alt[+Shift]+chữ).
    #[must_use]
    pub fn classify_with_mods(vk: u32, ch: u32, mods: u32) -> KeyKind {
        let mods_no_caps = mods & !MOD_CAPS;
        if vk == vk::SPACE && mods_no_caps == MOD_CTRL {
            return KeyKind::CtrlSpace;
        }
        let kind = Self::classify(vk, ch);
        if mods_no_caps == (MOD_CTRL | MOD_ALT) || mods_no_caps == (MOD_CTRL | MOD_ALT | MOD_SHIFT)
        {
            if let KeyKind::Char(c) = kind {
                return KeyKind::AltGr(c);
            }
        }
        kind
    }

    /// `ch` gửi engine: chỉ ký tự in được; phím đặc biệt để engine tự nhận qua `vk`.
    #[must_use]
    pub fn engine_ch(self) -> u32 {
        match self {
            KeyKind::Char(c) | KeyKind::AltGr(c) => c as u32,
            _ => 0,
        }
    }

    /// Phím cần edit session khi chưa có composition (có thể sinh text/macro).
    #[must_use]
    pub fn needs_session_when_idle(self) -> bool {
        matches!(
            self,
            KeyKind::Char(_)
                | KeyKind::AltGr(_)
                | KeyKind::Enter
                | KeyKind::Tab
                | KeyKind::CtrlSpace
        )
    }

    /// Ký tự ranh giới engine gắn cuối `insert` (RESTORE/COMMIT) mà app phải nhận dưới dạng PHÍM
    /// thật theo `policy`: Enter/Tab luôn vậy; ký tự in được chỉ khi [`BoundaryPolicy::ForwardKey`].
    #[must_use]
    pub fn native_boundary_char(self, policy: BoundaryPolicy) -> Option<char> {
        match self {
            KeyKind::Enter => Some('\n'),
            KeyKind::Tab => Some('\t'),
            KeyKind::Char(c) if policy == BoundaryPolicy::ForwardKey => Some(c),
            _ => None,
        }
    }
}

/// Ranh giới in được (Space, dấu câu, số) khi đang soạn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryPolicy {
    /// Ký tự ranh giới vào cuối text, commit CÙNG từ, phím bị ăn (TSF: app IMM32 qua CUAS nhận
    /// kết quả composition SAU `WM_CHAR` ⇒ để app tự chèn sẽ ra `" được"`).
    CommitWithWord,
    /// Commit từ rồi app nhận phím gốc (IBus/Fcitx5: forward key giữ thứ tự).
    ForwardKey,
}

/// Chính sách compose theo năng lực OS (cùng một luật, khác cách áp — G22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComposePolicy {
    /// Ranh giới in được khi đang soạn.
    pub boundary: BoundaryPolicy,
    /// Độ dài tối đa của text đang soạn (`None` = không giới hạn). Vượt ⇒ commit nguyên văn,
    /// phím đi thẳng, `reset_engine` (Linux `LC_COMP_MAX`, C5 audit).
    pub max_len: Option<usize>,
    /// Ctrl+Space không khớp (PASS) ⇒ `reset_engine` (R2-121: app có thể chèn dấu cách / dời
    /// caret mà engine không thấy). TSF xử lý ở lớp COM nên tắt ở đây.
    pub reset_on_ctrl_space_pass: bool,
}

impl ComposePolicy {
    /// Windows TSF (composition do TIP sở hữu, CUAS).
    pub const TSF: ComposePolicy = ComposePolicy {
        boundary: BoundaryPolicy::CommitWithWord,
        max_len: None,
        reset_on_ctrl_space_pass: false,
    };
    /// Linux IBus/Fcitx5 (preedit + forward key, `LC_COMP_MAX` = 256).
    pub const LINUX: ComposePolicy = ComposePolicy {
        boundary: BoundaryPolicy::ForwardKey,
        max_len: Some(256),
        reset_on_ctrl_space_pass: true,
    };
}

/// Kết quả engine đã giải mã (không phụ thuộc layout C của `ime_result_v1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineStep {
    /// `ACTION_*` của core.
    pub action: u32,
    /// Số ký tự xoá trước caret.
    pub delete_count: u16,
    /// Text chèn.
    pub insert: Vec<char>,
    /// `FLAG_*` của core.
    pub flags: u32,
    /// P11 math: số bước dịch con trỏ trái sau khi áp (0 = không đụng).
    pub cursor_left: u16,
}

/// Kết quả thô của engine theo layout C `ime_result_v1` (crate `ffi` cài cho struct thật — session
/// không phụ thuộc FFI). Cho phép `EngineStep::from_result(&r)` ở mọi adapter Rust.
pub trait RawEngineResult {
    /// `ACTION_*`.
    fn action(&self) -> u32;
    /// Số ký tự xoá.
    fn delete_count(&self) -> u16;
    /// Mảng UTF-32 chèn (đủ dung lượng) và độ dài hợp lệ.
    fn insert(&self) -> (&[u32], u16);
    /// `FLAG_*`.
    fn flags(&self) -> u32;
    /// P11 math: bước mũi tên trái.
    fn cursor_left(&self) -> u16;
}

impl EngineStep {
    /// Giải mã kết quả thô của engine ([`RawEngineResult`]).
    #[must_use]
    pub fn from_result<R: RawEngineResult + ?Sized>(r: &R) -> EngineStep {
        let (insert, len) = r.insert();
        Self::from_raw(
            r.action(),
            r.delete_count(),
            insert,
            len,
            r.flags(),
            r.cursor_left(),
        )
    }

    /// PASS không cờ.
    #[must_use]
    pub fn pass() -> EngineStep {
        EngineStep {
            action: ACTION_PASS,
            delete_count: 0,
            insert: Vec::new(),
            flags: 0,
            cursor_left: 0,
        }
    }

    /// Dựng từ các trường thô của `ime_result_v1` (UTF-32, cắt theo `insert_len`).
    #[must_use]
    pub fn from_raw(
        action: u32,
        delete_count: u16,
        insert: &[u32],
        insert_len: u16,
        flags: u32,
        cursor_left: u16,
    ) -> EngineStep {
        let len = usize::from(insert_len).min(insert.len());
        EngineStep {
            action,
            delete_count,
            insert: insert[..len]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect(),
            flags,
            cursor_left,
        }
    }

    /// Engine báo hết từ (`FLAG_WORD_END`).
    #[must_use]
    pub fn word_end(&self) -> bool {
        self.flags & FLAG_WORD_END != 0
    }

    /// Engine nuốt phím (`FLAG_CONSUMED`).
    #[must_use]
    pub fn consumed(&self) -> bool {
        self.flags & FLAG_CONSUMED != 0
    }
}

/// Việc adapter phải làm cho một phím.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionPlan {
    /// Số ký tự đã commit NGAY TRƯỚC composition phải xóa (macro/emoji vượt ranh giới
    /// composition). Adapter phải xác minh được mới xóa, nếu không → fail-open.
    pub delete_before: u16,
    /// Text composition sau phím này; `None` = giữ nguyên.
    pub text: Option<Vec<char>>,
    /// Đóng composition (commit) sau khi áp `text`.
    pub end: bool,
    /// `true` = phím bị ăn; `false` = app nhận phím gốc (sau khi composition đã đóng).
    pub eaten: bool,
    /// P11 math: số bước dịch con trỏ trái sau khi commit text (0 = bỏ qua).
    pub cursor_left: u16,
    /// Commit rồi `ime_reset` (text tràn `max_len`; Ctrl+Space tới app — chính sách Linux).
    pub reset_engine: bool,
}

impl CompositionPlan {
    /// Không làm gì với composition, phím đi thẳng (đóng composition nếu đang soạn).
    #[must_use]
    pub fn pass_through(composing: bool) -> CompositionPlan {
        CompositionPlan {
            delete_before: 0,
            text: None,
            end: composing,
            eaten: false,
            cursor_left: 0,
            reset_engine: false,
        }
    }

    /// Commit nguyên văn `current` rồi app nhận phím, reset engine (tràn `max_len`).
    fn overflow(current: &[char]) -> CompositionPlan {
        CompositionPlan {
            delete_before: 0,
            text: Some(current.to_vec()),
            end: true,
            eaten: false,
            cursor_left: 0,
            reset_engine: true,
        }
    }
}

/// [`plan_key_with`] theo [`ComposePolicy::TSF`] (giữ API cũ của adapter TSF).
#[must_use]
pub fn plan_key(current: &[char], key: KeyKind, step: &EngineStep) -> CompositionPlan {
    plan_key_with(ComposePolicy::TSF, current, key, step)
}

/// Tính kế hoạch cho một phím. `current` = text đang soạn (rỗng = không composing).
/// Hàm thuần, không đoán text của app.
#[must_use]
pub fn plan_key_with(
    policy: ComposePolicy,
    current: &[char],
    key: KeyKind,
    step: &EngineStep,
) -> CompositionPlan {
    let composing = !current.is_empty();
    match step.action {
        // E62 (P11 Math): engine nuốt phím im lặng (backtick arm chord, Esc huỷ chord)
        // → KHÔNG chèn vào composition và KHÔNG forward cho app.
        ACTION_PASS if step.consumed() => CompositionPlan {
            eaten: true,
            ..CompositionPlan::pass_through(composing)
        },
        ACTION_PASS => match key {
            // Engine đã nhận ký tự vào từ nhưng chưa biến đổi → đưa vào composition.
            KeyKind::Char(c) if !step.word_end() => {
                if policy.max_len.is_some_and(|m| current.len() + 1 >= m) {
                    return CompositionPlan::overflow(current);
                }
                let mut text = current.to_vec();
                text.push(c);
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end: false,
                    eaten: true,
                    cursor_left: 0,
                    reset_engine: false,
                }
            }
            // Ranh giới in được khi đang composing (TSF/CUAS): ký tự ranh giới vào cuối
            // composition rồi commit CÙNG từ, phím bị ăn — một lần commit nguyên khối đúng
            // thứ tự ở mọi loại app.
            KeyKind::Char(c) if composing && policy.boundary == BoundaryPolicy::CommitWithWord => {
                let mut text = current.to_vec();
                text.push(c);
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end: true,
                    eaten: true,
                    cursor_left: 0,
                    reset_engine: false,
                }
            }
            // Engine đã bỏ 1 ký tự chưa biến đổi khỏi từ → bỏ khỏi composition.
            KeyKind::Backspace if composing => {
                let mut text = current.to_vec();
                text.pop();
                let end = text.is_empty();
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end,
                    eaten: true,
                    cursor_left: 0,
                    reset_engine: false,
                }
            }
            // Ranh giới / điều hướng / Ctrl+Space không khớp: commit nguyên văn rồi app xử lý
            // phím. Ctrl+Space KHÔNG giữ composition mở (R2-121).
            _ => CompositionPlan {
                reset_engine: key == KeyKind::CtrlSpace && policy.reset_on_ctrl_space_pass,
                ..CompositionPlan::pass_through(composing)
            },
        },
        ACTION_REPLACE | ACTION_RESTORE | ACTION_COMMIT => {
            let (keep, delete_before) = if step.action == ACTION_COMMIT {
                (current.len(), 0)
            } else {
                let del = usize::from(step.delete_count);
                if del <= current.len() {
                    (current.len() - del, 0)
                } else {
                    (0, (del - current.len()) as u16)
                }
            };
            if policy.max_len.is_some_and(|m| keep + step.insert.len() > m) {
                return CompositionPlan {
                    reset_engine: true,
                    ..CompositionPlan::pass_through(composing)
                };
            }
            let mut text = current[..keep].to_vec();
            text.extend_from_slice(&step.insert);
            let mut eaten = true;
            // RESTORE/COMMIT ở ranh giới: engine đặt ký tự ranh giới cuối `insert`. Enter/Tab
            // (và ký tự in được khi policy ForwardKey) bỏ ra để app nhận phím gốc — Enter phải là
            // phím Enter thật (chat gửi tin), không phải `\n` chèn vào text. REPLACE có WORD_END
            // là macro/emoji/math: trigger bị nuốt, riêng `\n` cuối `insert` thì trả phím Enter.
            if step.word_end() {
                if let Some(b) = key.native_boundary_char(policy.boundary) {
                    if (step.action != ACTION_REPLACE || b == '\n') && text.last() == Some(&b) {
                        text.pop();
                        eaten = false;
                    }
                }
            }
            let end = step.word_end() || text.is_empty() || !eaten;
            CompositionPlan {
                delete_before,
                text: Some(text),
                end,
                eaten,
                cursor_left: step.cursor_left,
                reset_engine: false,
            }
        }
        // Action lạ (ABI mới hơn adapter): an toàn nhất là commit và để phím đi thẳng.
        _ => CompositionPlan::pass_through(composing),
    }
}

/// Text đang soạn sau khi áp kế hoạch (model của adapter sau khi áp thành công).
#[must_use]
pub fn text_after(current: &[char], plan: &CompositionPlan) -> Vec<char> {
    if plan.end || plan.reset_engine {
        return Vec::new();
    }
    plan.text.clone().unwrap_or_else(|| current.to_vec())
}
