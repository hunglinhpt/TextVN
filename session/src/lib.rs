// SPDX-License-Identifier: GPL-3.0-or-later
//! `textvn-session` — khung session nghiệp vụ dùng chung cho 3 OS (P0-6 §3, task SF-1a).
//!
//! Một [`Session`] là trạng thái nghiệp vụ của MỘT ô nhập (input context): một instance
//! session / context TSF / `IBusEngine` / `fcitx::InputContext` / `IMKInputController`.
//! Adapter dịch sự kiện OS thành [`SessionEvent`] đưa vào [`Session::event`], nhận lại
//! [`SessionCommand`] để áp bằng API OS. Mọi nghiệp vụ chung (toggle, compose, batch,
//! ghost, trợ lý, field, ipc) sẽ nằm ở crate này — adapter KHÔNG ra quyết định nghiệp vụ
//! (P0-6 §2, G22).
//!
//! # Trạng thái hiện tại (SF-1a — khung đầu)
//!
//! Chưa có logic nghiệp vụ thật (SF-2 trở đi mới chuyển toggle/batch/compose/… vào).
//! Hợp đồng của khung rỗng:
//!
//! - [`SessionEvent::KeyDown`] ⇒ đúng một [`SessionCommand::ForwardKey`] (phím đi thẳng
//!   app, session không ăn).
//! - Mọi [`SessionEvent`] khác ⇒ danh sách lệnh rỗng.
//!
//! # Bất biến kiến trúc (P0-6 §2 — giữ nguyên trong mọi SF-*)
//!
//! 1. Không I/O (`std::fs`, mạng), không thread, không đồng hồ hệ thống — thời gian đến
//!    qua sự kiện (`now_ms`) ⇒ tất định, test được, chạy được trong `cli replay`.
//! 2. Adapter chỉ: (a) dịch sự kiện OS → `SessionEvent`, (b) áp `SessionCommand` bằng
//!    API OS, (c) vẽ UI, (d) cấp timer/IPC transport.
//! 3. Không log nội dung gõ (G8) — crate này không có API log nào.
//! 4. Một session = một luồng (thread của context); không `Send` qua FFI.
//! 5. Panic phải bị chặn ở biên FFI (`catch_unwind` — SF-1b), không vượt ra adapter.
//!
//! # Luật đầu ra (P0-6 §3 — dùng chung mọi OS, R2-165/R2-167)
//!
//! - Mỗi sự kiện: tối đa 1 [`SessionCommand::DeleteBefore`], tối đa 1
//!   [`SessionCommand::Commit`]; `Commit` đứng TRƯỚC `SetPreedit { text: "" }` (ẩn
//!   preedit) — [`Commands::push`] chặn bằng `debug_assert`, test thuộc tính ở
//!   `session/tests/invariants.rs`.
//! - Với mỗi [`SessionEvent::KeyDown`]: đúng MỘT trong [`SessionCommand::ForwardKey`] /
//!   [`SessionCommand::EatKey`].
//! - Ô `secure` ⇒ không `Ghost`, không `AssistShow`, không IPC chứa chữ (S3).
#![forbid(unsafe_code)]

/// Năng lực adapter khai báo cho session — session chỉ ra lệnh adapter làm được
/// (P0-6 §3). Mỗi adapter lập một `Caps` khi tạo session theo nền tảng của nó.
///
/// Trên TSF `forward_key` thường là `false` (không ăn phím ⇒ phím tự đi tiếp),
/// IBus/Fcitx5 là `true` (phải gọi forward tường minh). `inline_ghost` là `true`
/// duy nhất trên TSF; IBus (aux) và IMK (panel) vẽ chữ mờ ở dòng/phụ-panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Caps {
    /// Vẽ được preedit tại caret (TSF composition · IBus/Fcitx5 preedit · IMK marked text).
    pub preedit: bool,
    /// Đọc được chữ trước caret (surrounding text).
    pub surrounding_text: bool,
    /// Xoá trước caret không cần giả lập Backspace.
    pub delete_surrounding: bool,
    /// Vẽ được chữ mờ TẠI CARET (TSF) — `false`: dòng phụ (IBus aux, IMK panel).
    pub inline_ghost: bool,
    /// Adapter nhận key-up (E79: vẽ chữ mờ ở key-up).
    pub key_up_events: bool,
    /// Trả phím cho app tường minh được (IBus/Fcitx5 forward; TSF "không ăn" là đủ).
    pub forward_key: bool,
}

impl Caps {
    /// Không năng lực gì — passthrough tuyệt đối (ô secure/terminal tối giản).
    #[must_use]
    pub const fn none() -> Self {
        Self {
            preedit: false,
            surrounding_text: false,
            delete_surrounding: false,
            inline_ghost: false,
            key_up_events: false,
            forward_key: false,
        }
    }
}

/// Loại reset toolkit/app yêu cầu (P0-6 §5 hàng *reset*, R2-171/E78).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetKind {
    /// Toolkit chủ động reset (IBus `engine_reset`, Fcitx5 `reset`).
    Toolkit,
    /// Reset do mất/nhận focus.
    Focus,
    /// Reset do app (ví dụ app tự xoá buffer).
    App,
}

/// Định danh timer do adapter cấp (P0-6 §2: adapter là nơi cấp timer thật).
/// Session không có đồng hồ — chỉ đặt/huỷ qua lệnh [`SessionCommand::TimerStart`] /
/// [`SessionCommand::TimerCancel`], nhận lại qua [`SessionEvent::Timer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimerId(pub u32);

/// Hành động người dùng trên Trợ lý ngữ pháp (P0-6 §5 hàng *assist*; SF-5 chốt bộ
/// variant đầy đủ — tên variant hiện tại là bản khung của SF-1a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistAction {
    /// Chọn gợi ý thứ `index` trong danh sách đang hiện (0-based).
    Select {
        /// Chỉ số gợi ý trong [`SessionCommand::AssistShow`].
        index: u16,
    },
    /// Esc — đóng trợ lý, không áp gì.
    Dismiss,
    /// Ctrl+Z — hoàn tác lần áp gần nhất (R-3: undo chỉ sống tới phím kế).
    Undo,
}

/// Một mục gợi ý Trợ lý ngữ pháp hiển thị trên dòng gợi ý (SF-5 sẽ chốt cấu trúc
/// đầy đủ; panel vẽ ở adapter — G22 dữ liệu dùng chung, vẽ native).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistItem {
    /// Chuỗi hiển thị trên dòng gợi ý.
    pub text: String,
    /// Văn bản thay thế khi người dùng áp gợi ý này.
    pub replace: String,
}

/// Sự kiện adapter đưa vào session (P0-6 §3). Thuần dữ liệu — không tham chiếu
/// OS, không owning text ngoài `app_id`/`surrounding` (mượn `&str` từ adapter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent<'a> {
    /// Phím chữ/ký tự gõ xuống. `ch > 0` chỉ với phím sinh ký tự thật (Dream RSI
    /// G19b: loại key-up/injected/modifier/điều hướng ở adapter trước khi đưa vào).
    KeyDown {
        /// Virtual-key code của nền tảng.
        vk: u32,
        /// Mã ký tự Unicode sinh ra (0 nếu không sinh ký tự).
        ch: u32,
        /// Bit cờ modifier (Shift/Ctrl/Alt… — quy ước adapter).
        mods: u32,
        /// Mốc thời gian do adapter cung cấp (ms) — session không có đồng hồ.
        now_ms: u64,
    },
    /// Phím thả (E79: vẽ chữ mờ ở key-up nếu adapter có [`Caps::key_up_events`]).
    KeyUp {
        /// Virtual-key code của nền tảng.
        vk: u32,
        /// Bit cờ modifier.
        mods: u32,
        /// Mốc thời gian do adapter cung cấp (ms).
        now_ms: u64,
    },
    /// Nhận focus cho một ô nhập — adapter khai báo ngữ cảnh ô.
    FocusIn {
        /// Định danh app (không phải nội dung gõ — G8 cho phép log mã định danh).
        app_id: &'a str,
        /// Vai trò ô theo field-detect (u32 để adapter map giá trị OS).
        field_role: u32,
        /// Ô mật khẩu/secure ⇒ passthrough, không ghost/assist/IPC chữ (S3).
        secure: bool,
        /// Terminal ⇒ chiến lược Backspace-type, không dự đoán (R2-144/145).
        terminal: bool,
    },
    /// Mất focus — session phải tự dọn composition.
    FocusOut,
    /// Caret di chuyển (click / phím điều hướng / app tự dời) — composition phải
    /// commit nguyên văn (self-heal) nếu rời ranh giới.
    CaretMoved,
    /// Chữ trước caret do adapter đọc được (≤ 256 ký tự).
    Surrounding {
        /// Chữ trước caret.
        before: &'a str,
    },
    /// Reset theo P0-6 §5 (R2-171: reset trong 250 ms sau Enter giữ mốc đầu dòng;
    /// E78: reset khác xoá sạch).
    Reset {
        /// Loại reset.
        kind: ResetKind,
        /// Mốc thời gian do adapter cung cấp (ms).
        now_ms: u64,
    },
    /// Timer adapter cấp nhảy (assist idle 1 s — R2-154/155).
    Timer {
        /// Định danh timer đã đặt bằng [`SessionCommand::TimerStart`].
        id: TimerId,
        /// Mốc thời gian do adapter cung cấp (ms).
        now_ms: u64,
    },
    /// Message IPC v1 từ tray/app đến (codec `textvn-ipc`).
    Ipc(textvn_ipc::Message),
    /// Hành động trợ lý (chọn gợi ý, Esc, Ctrl+Z…).
    AssistAction {
        /// Hành động cụ thể.
        action: AssistAction,
    },
}

/// Lệnh session trả ra để adapter ÁP bằng API OS (P0-6 §3). Adapter không được
/// suy luận thêm lệnh nào ngoài danh sách này.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    /// Đặt/preedit; `text: ""` = ẨN preedit. `cursor` là vị trí caret trong
    /// preedit (UTF-16 code unit — quy ước TSF/IMK).
    SetPreedit {
        /// Nội dung preedit (rỗng = ẩn).
        text: String,
        /// Vị trí caret trong preedit.
        cursor: u16,
    },
    /// Commit chữ vào buffer app (R2-165/R2-167: tối đa 1 mỗi event, đứng trước
    /// ẩn preedit).
    Commit {
        /// Chữ commit.
        text: String,
    },
    /// Xoá `chars` ký tự trước caret (tối đa 1 mỗi event — R2-165 `lc_edit_batch`).
    DeleteBefore {
        /// Số ký tự xoá.
        chars: u16,
    },
    /// Phím hiện tại đi thẳng app (không ăn). Đúng một trong
    /// `ForwardKey`/`EatKey` cho mỗi [`SessionEvent::KeyDown`].
    ForwardKey,
    /// Phím hiện tại bị session ăn. Đúng một trong `ForwardKey`/`EatKey` cho
    /// mỗi [`SessionEvent::KeyDown`].
    EatKey,
    /// Chữ mờ (ghost) — rỗng = ẩn; adapter vẽ theo [`Caps::inline_ghost`]
    /// (R2-173: chữ mờ = đúng phần Tab sẽ nối).
    Ghost {
        /// Phần chữ nối thêm (rỗng = ẩn).
        text: String,
    },
    /// E79: hoãn vẽ chữ mờ tới key-up của phím hiện tại (phím đi thẳng app).
    GhostAfterKeyUp,
    /// Hiện trợ lý với danh sách gợi ý (ô secure ⇒ cấm — S3).
    AssistShow {
        /// Danh sách mục gợi ý.
        items: Vec<AssistItem>,
    },
    /// Ẩn trợ lý (P9: ẩn ngay khi gõ tiếp).
    AssistHide,
    /// Đặt timer `id` nhảy sau `ms` (adapter cấp timer thật — P0-6 §2).
    TimerStart {
        /// Định danh timer.
        id: TimerId,
        /// Thời lượng (ms).
        ms: u32,
    },
    /// Huỷ timer `id`.
    TimerCancel {
        /// Định danh timer.
        id: TimerId,
    },
    /// Message IPC v1 session muốn gửi đi (adapter là transport, không tự thêm
    /// nội dung — E80/E81 sandbox client).
    Ipc(textvn_ipc::Message),
    /// Cập nhật chỉ báo chế độ VI/EN trên UI adapter.
    ModeIndicator {
        /// `true` = đang chế độ tiếng Việt.
        vi_on: bool,
    },
}

/// Chuỗi lệnh trả về cho MỘT sự kiện, chặn luật đầu ra P0-6 §3 ngay lúc dựng
/// (`debug_assert` + test thuộc tính): tối đa 1 xoá, tối đa 1 commit, commit
/// trước ẩn preedit, đúng một quyết định phím.
///
/// Luật vẫn được thắt chặt ở cấp cấu trúc để mọi SF-* sau này không thể phá
/// R2-165/R2-167 một cách vô ý khi thêm logic vào [`Session::event`].
#[derive(Debug, Clone, Default)]
pub struct Commands {
    cmds: Vec<SessionCommand>,
    seen_commit: bool,
    seen_delete: bool,
    seen_key_decision: bool,
    hidden_preedit: bool,
}

impl Commands {
    /// Chuỗi lệnh rỗng.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Thêm một lệnh. Vi phạm luật P0-6 §3 ⇒ `debug_assert` chết ngay lúc viết
    /// logic (release vẫn không panic — biên FFI SF-1b lo `catch_unwind`).
    pub fn push(&mut self, cmd: SessionCommand) {
        match &cmd {
            SessionCommand::Commit { .. } => {
                debug_assert!(
                    !self.seen_commit,
                    "R2-165: moi event toi da 1 Commit (da co Commit truoc do)"
                );
                debug_assert!(
                    !self.hidden_preedit,
                    "R2-167: Commit phai dung TRUOC khi an preedit (SetPreedit{{\"\"}} da co)"
                );
                self.seen_commit = true;
            }
            SessionCommand::DeleteBefore { .. } => {
                debug_assert!(
                    !self.seen_delete,
                    "R2-165: moi event toi da 1 DeleteBefore (da co DeleteBefore truoc do)"
                );
                self.seen_delete = true;
            }
            SessionCommand::SetPreedit { text, .. } => {
                if text.is_empty() {
                    self.hidden_preedit = true;
                }
            }
            SessionCommand::ForwardKey | SessionCommand::EatKey => {
                debug_assert!(
                    !self.seen_key_decision,
                    "P0-6 §3: dung MOT trong ForwardKey/EatKey cho moi KeyDown"
                );
                self.seen_key_decision = true;
            }
            _ => {}
        }
        self.cmds.push(cmd);
    }

    /// Số lệnh trong chuỗi.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    /// Chuỗi rỗng?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    /// Xem chuỗi lệnh (adapter iterate qua đây).
    #[must_use]
    pub fn as_slice(&self) -> &[SessionCommand] {
        &self.cmds
    }

    /// Tiêu thụ chuỗi thành `Vec` (shadow mode so sánh/ghi vết dùng).
    #[must_use]
    pub fn into_vec(self) -> Vec<SessionCommand> {
        self.cmds
    }
}

impl IntoIterator for Commands {
    type Item = SessionCommand;
    type IntoIter = std::vec::IntoIter<SessionCommand>;

    fn into_iter(self) -> Self::IntoIter {
        self.cmds.into_iter()
    }
}

/// Session nghiệp vụ của MỘT ô nhập (P0-6 §0/§2). Không I/O, không thread,
/// không đồng hồ; một session sống đúng một luồng của context.
///
/// SF-1a: khung rỗng — [`Session::event`] trả [`SessionCommand::ForwardKey`]
/// cho mỗi [`SessionEvent::KeyDown`] và rỗng cho mọi event khác. SF-2+ chuyển
/// toggle/batch/compose/ghost/assist vào đây theo P0-6 §5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    caps: Caps,
}

impl Session {
    /// Tạo session cho một ô nhập với năng lực adapter khai báo.
    #[must_use]
    pub fn new(caps: Caps) -> Self {
        Self { caps }
    }

    /// Năng lực adapter đang khai báo cho session này.
    #[must_use]
    pub fn caps(&self) -> Caps {
        self.caps
    }

    /// Đưa một sự kiện vào session, nhận chuỗi lệnh để adapter áp.
    ///
    /// Hợp đồng khung rỗng (SF-1a): `KeyDown` ⇒ đúng một `ForwardKey`; event
    /// khác ⇒ chuỗi rỗng. Tất định: cùng event stream ⇒ cùng chuỗi lệnh.
    pub fn event(&mut self, event: SessionEvent<'_>) -> Commands {
        let mut out = Commands::new();
        if let SessionEvent::KeyDown { .. } = event {
            // Khung rỗng chưa có logic ăn phím nào — trả phím cho app.
            out.push(SessionCommand::ForwardKey);
        }
        out
    }
}
