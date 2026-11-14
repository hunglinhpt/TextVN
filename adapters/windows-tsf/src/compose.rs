// SPDX-License-Identifier: GPL-3.0-or-later
//! Mô hình composition thuần của TSF adapter (không gọi Win32 — test được mọi OS).
//!
//! **Quyết định kiến trúc:** mọi ký tự của từ đang gõ nằm trong MỘT `ITfComposition`
//! do TIP sở hữu; tới ranh giới từ composition được đóng (commit). Ranh giới in được
//! (Space, dấu câu, số) được commit CÙNG từ và phím bị ăn; Enter, Tab, phím điều hướng,
//! chord… thì commit rồi phím gốc mới đi tới app.
//!
//! Lý do: engine trả kết quả theo mô hình "xóa `delete_count` ký tự trước con trỏ
//! rồi chèn `insert`". Với app không hỗ trợ TSF đầy đủ (IMM32 qua CUAS: Win32 Edit,
//! WinForms, Delphi, Java, game…) text store chỉ chứa **composition** — ký tự đã
//! commit không thể xóa lại qua `ITfRange`. Giữ cả từ trong composition khiến mọi
//! `delete_count` của engine rơi vào vùng TIP sở hữu, nên cùng một đường code chạy
//! đúng cho app TSF-aware lẫn app IMM32, không cần gửi Backspace giả (không SendInput).
//!
//! Module này chỉ tính *kế hoạch* ([`CompositionPlan`]); lớp COM (`edit_session.rs`)
//! áp kế hoạch trong `DoEditSession`. Bất biến kiểm chứng bằng test:
//! - phím không bị ăn (`eaten == false`) ⇒ composition đã đóng trước khi app nhận phím
//!   (không còn "chữ treo" khi Enter gửi tin nhắn — bug B2);
//! - `delete_before > 0` chỉ xuất hiện khi engine cần xóa text ngoài composition
//!   (macro/emoji có trigger chứa dấu câu) — lớp COM phải xác minh được mới xóa.

use textvn_ffi::{
    ime_result_v1, ACTION_COMMIT, ACTION_PASS, ACTION_REPLACE, ACTION_RESTORE, IME_FLAG_WORD_END,
};

/// Win VK dùng để phân loại phím (khớp `textvn_core::keymap::vk`).
pub mod vk {
    pub const BACK: u32 = 0x08;
    pub const TAB: u32 = 0x09;
    pub const RETURN: u32 = 0x0D;
    pub const SPACE: u32 = 0x20;
    pub const SHIFT: u32 = 0x10;
    pub const CONTROL: u32 = 0x11;
    pub const MENU: u32 = 0x12;
    pub const CAPITAL: u32 = 0x14;
    pub const ESCAPE: u32 = 0x1B;
    pub const LWIN: u32 = 0x5B;
    pub const RWIN: u32 = 0x5C;
    pub const LSHIFT: u32 = 0xA0;
    pub const RMENU: u32 = 0xA5;
    /// Unicode do `SendInput(KEYEVENTF_UNICODE)` bơm vào — không phải phím người gõ.
    pub const PACKET: u32 = 0xE7;
    /// App/IME khác đã xử lý phím này.
    pub const PROCESSKEY: u32 = 0xE5;
}

/// Phân loại phím sau khi adapter đã dịch VK → ký tự theo layout bàn phím.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// Phím sinh ký tự in được (gồm Space).
    Char(char),
    Backspace,
    Escape,
    Enter,
    Tab,
    CtrlSpace,
    /// Không sinh ký tự: mũi tên, Delete, Home/End, F-key, PageUp…
    Other,
}

impl KeyKind {
    /// `ch` = kết quả `ToUnicodeEx` (0 nếu không sinh ký tự). Ký tự điều khiển
    /// (`\r`, `\t`, `\b`, ESC) được quy về phím tương ứng; VK không sinh ký tự
    /// tuyệt đối không được coi là chữ (trước đây `ch = vk` biến F1 thành `p`,
    /// Delete thành `.`).
    pub fn classify(vk: u32, ch: u32) -> KeyKind {
        match vk {
            vk::BACK => return KeyKind::Backspace,
            vk::TAB => return KeyKind::Tab,
            vk::RETURN => return KeyKind::Enter,
            vk::ESCAPE => return KeyKind::Escape,
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

    /// `ime_key_v1.ch` gửi engine: chỉ ký tự in được; phím đặc biệt để engine tự
    /// nhận qua `vk` (engine map Enter→`\n`, Tab→`\t`).
    /// Phân loại kèm modifier: nhận diện Ctrl+Space (trigger Text Prediction & Text Correct).
    pub fn classify_with_mods(vk: u32, ch: u32, mods: u32) -> KeyKind {
        if vk == 0x20 && (mods & !textvn_ffi::MOD_CAPS) == textvn_ffi::MOD_CTRL {
            return KeyKind::CtrlSpace;
        }
        Self::classify(vk, ch)
    }

    pub fn engine_ch(self) -> u32 {
        match self {
            KeyKind::Char(c) => c as u32,
            _ => 0,
        }
    }

    /// Ranh giới engine gắn vào cuối `insert` khi RESTORE/COMMIT mà app phải nhận
    /// dưới dạng PHÍM thật (không chèn thành text). Ký tự in được không nằm ở đây:
    /// chúng được commit cùng từ.
    fn native_boundary_char(self) -> Option<char> {
        match self {
            KeyKind::Enter => Some('\n'),
            KeyKind::Tab => Some('\t'),
            _ => None,
        }
    }

    /// Phím cần edit session khi chưa có composition (có thể sinh text/macro).
    pub fn needs_session_when_idle(self) -> bool {
        matches!(
            self,
            KeyKind::Char(_) | KeyKind::Enter | KeyKind::Tab | KeyKind::CtrlSpace
        )
    }
}

/// Phím chỉ là modifier/lock: không đóng composition, không tới engine.
pub fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        vk::SHIFT | vk::CONTROL | vk::MENU | vk::CAPITAL | vk::LWIN | vk::RWIN
    ) || (vk::LSHIFT..=vk::RMENU).contains(&vk)
}

/// Lớp cửa sổ của control **không nhập văn bản** (danh sách file, cây thư mục, thanh công cụ, màn
/// hình nền…): TextVN không được mở composition ở đây. Nếu mở, TSF không có text extent nên vẽ
/// khung `CiceroUIWndFrame` chứa chữ vừa gõ ở góc trái trên màn hình (chủ repo: bấm vào thư mục
/// Explorer rồi gõ phím → hiện popup nhỏ ở góc trái — R2-159). Phím phải tới app (gõ để nhảy tới
/// mục, type-ahead). So khớp không phân biệt hoa/thường; chỉ lớp control chuẩn của Windows/Explorer —
/// ô nhập (Edit, RichEdit, ComboBox…) và app tự vẽ ô nhập KHÔNG nằm trong danh sách.
pub fn is_non_text_window_class(class: &str) -> bool {
    let mut c = class.to_ascii_lowercase();
    // WinForms đổi tên lớp control chuẩn: `WindowsForms10.SysListView32.app.0.<hash>` → lấy phần giữa.
    if let Some(rest) = c.strip_prefix("windowsforms10.") {
        c = rest.split(".app.").next().unwrap_or(rest).to_string();
    }
    matches!(
        c.as_str(),
        "syslistview32"
            | "systreeview32"
            | "sysheader32"
            | "systabcontrol32"
            | "toolbarwindow32"
            | "rebarwindow32"
            | "msctls_statusbar32"
            | "msctls_trackbar32"
            | "msctls_updown32"
            | "msctls_progress32"
            | "shelldll_defview"
            | "directuihwnd"
            | "ctrlnotifysink"
            | "progman"
            | "workerw"
            | "syslink"
            | "button"
            | "static"
            | "scrollbar"
            | "listbox"
            | "combolbox"
            | "#32769"
            | "listview20wndclass"
            | "treeview20wndclass"
    )
}

/// Lớp cửa sổ GỐC (top-level) không có ô nhập nào: thanh tác vụ, khay hệ thống, màn hình nền. Win11
/// dựng thanh tác vụ bằng XAML (`Windows.UI.Input.InputSite.WindowClass` — lớp này cũng là ô nhập XAML
/// ở nơi khác nên KHÔNG chặn theo lớp con) nhưng cửa sổ gốc của nó thì không bao giờ có ô nhập (R2-161).
pub fn is_non_text_top_level_class(class: &str) -> bool {
    matches!(
        class.to_ascii_lowercase().as_str(),
        "shell_traywnd"
            | "shell_secondarytraywnd"
            | "notifyiconoverflowwindow"
            | "toplevelwindowforoverflowxamlisland"
            | "tasklistthumbnailwnd"
            | "tasklistoverlaywnd"
            | "progman"
            | "workerw"
    )
}

/// App có `engine_owner = hook` + `ForwardAsCommit` đang bật trong appdb mặc định (MobaXterm, PuTTY,
/// mintty, KiTTY, Tera Term, SecureCRT, Xshell…): terminal KHÔNG hỗ trợ composition inline — TIP mà
/// dựng composition ở đó thì Windows vẽ khung nhỏ chứa chữ gõ ở góc trái màn hình (chủ repo: "ô nhỏ mũi
/// tên xanh conflict" ở MobaXterm). TIP nhường (không ăn phím); bản cũ ghi "TSF nhường qua owns_engine"
/// nhưng `owns_engine` không được gọi ở đường phím (R2-161). `exe` đã lowercase.
pub fn is_hook_owned_exe(exe: &str) -> bool {
    use std::sync::OnceLock;
    use textvn_appdb::{AppDb, EngineOwner};
    use textvn_strategy::Strategy;
    static DB: OnceLock<Option<AppDb>> = OnceLock::new();
    let db = DB.get_or_init(|| AppDb::parse(include_str!("../../../data/appdb.default.json")).ok());
    // Role 0 = chưa biết: chỉ entry không gắn `when` (mọi role) mới khớp — cmd/pwsh theo role terminal
    // không bị nhường (Windows Terminal/conhost hiện đại có composition inline).
    db.as_ref()
        .and_then(|db| db.preset_for(exe, 0))
        .is_some_and(|p| {
            p.engine_owner == Some(EngineOwner::Hook)
                && p.strategy == Some(Strategy::ForwardAsCommit)
                && p.enabled_default != Some(false)
        })
}

/// App terminal theo tên exe: Tab là của shell, phím phải tới app ngay (vim,
/// less) và chữ gõ có thể là mật khẩu — không dự đoán, không học (R2-144/R2-145).
/// So khớp **không phân biệt hoa/thường** — nhất quán với `is_hook_owned_exe`
/// (AppDb so khớp case-insensitive); caller quên lowercase vẫn đúng.
pub fn is_terminal_exe(exe: &str) -> bool {
    let exe = exe.to_ascii_lowercase();
    matches!(
        exe.as_str(),
        "windowsterminal.exe"
            | "openconsole.exe"
            | "conhost.exe"
            | "cmd.exe"
            | "powershell.exe"
            | "pwsh.exe"
            | "wezterm-gui.exe"
            | "alacritty.exe"
            | "putty.exe"
            | "mintty.exe"
            | "kitty.exe"
            | "kitty_portable.exe"
            | "ttermpro.exe"
            | "securecrt.exe"
            | "xshell.exe"
            | "xshell_premium.exe"
            | "tabby.exe"
            | "hyper.exe"
            | "warp.exe"
    ) || exe.starts_with("mobaxterm")
        || exe.starts_with("conemu")
}

fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk, vk::CONTROL | 0xA2 | 0xA3)
}

fn is_shift_vk(vk: u32) -> bool {
    matches!(vk, vk::SHIFT | vk::LSHIFT | 0xA1)
}

/// Phím chuyển kiểu UniKey/EVKey: nhấn Ctrl và Shift (thứ tự bất kỳ) rồi nhả, KHÔNG
/// kèm phím nào khác. Hộp thoại cài đặt và hook đều hứa "Ctrl + Shift"; TSF trước đây
/// chỉ có Ctrl+Shift+Space nên phím chuyển quen thuộc không có tác dụng.
/// Không bao giờ ăn phím modifier — chỉ quan sát.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierToggle {
    armed: bool,
    ctrl_armed: bool,
    /// Lúc bấm Ctrl của lần tap đang chờ (phím giữ lặp key-down không đổi mốc này).
    ctrl_since: Option<std::time::Instant>,
}

/// Giữ Ctrl lâu hơn ngần này không còn là "tap": người dùng đang Ctrl+click, Ctrl+lăn
/// chuột… mà bộ gõ không thấy (R2-143). Macos/Linux dùng cùng trị số.
pub const CTRL_TAP_MAX: std::time::Duration = std::time::Duration::from_millis(400);

impl ModifierToggle {
    /// Key-down bất kỳ. `ctrl_down`/`shift_down` = trạng thái phím còn lại của cặp
    /// (đã được xử lý trước phím hiện tại); `alt_or_win` = Alt/Win đang giữ.
    pub fn on_key_down(&mut self, vk: u32, ctrl_down: bool, shift_down: bool, alt_or_win: bool) {
        self.armed = if is_ctrl_vk(vk) {
            shift_down && !alt_or_win
        } else if is_shift_vk(vk) {
            ctrl_down && !alt_or_win
        } else {
            false
        };
        self.ctrl_armed = is_ctrl_vk(vk) && !shift_down && !alt_or_win;
    }

    /// Ghi mốc bấm Ctrl của lần tap vừa được `on_key_down` chấp nhận.
    pub fn mark_ctrl_down(&mut self, now: std::time::Instant) {
        self.ctrl_since = if self.ctrl_armed {
            self.ctrl_since.or(Some(now))
        } else {
            None
        };
    }

    /// Ctrl đã giữ quá lâu để còn là tap?
    pub fn ctrl_held_too_long(&self, now: std::time::Instant) -> bool {
        self.ctrl_since
            .is_some_and(|t| now.duration_since(t) > CTRL_TAP_MAX)
    }

    /// Key-up: `true` đúng MỘT lần khi nhả Ctrl/Shift của một lần bấm hợp lệ
    /// (gọi lại ở pha thứ hai `OnKeyUp` sẽ trả `false`).
    pub fn on_key_up(&mut self, vk: u32) -> bool {
        if (is_ctrl_vk(vk) || is_shift_vk(vk)) && self.armed {
            self.armed = false;
            self.ctrl_armed = false;
            return true;
        }
        false
    }

    /// Key-up của riêng phím Ctrl (standalone Ctrl tap): `true` đúng MỘT lần khi
    /// Ctrl được bấm và nhả riêng lẻ không kèm Shift/Alt/Win hay phím khác.
    pub fn on_ctrl_tap_up(&mut self, vk: u32) -> bool {
        if is_ctrl_vk(vk) && self.ctrl_armed {
            self.ctrl_armed = false;
            self.ctrl_since = None;
            return true;
        }
        if is_modifier_vk(vk) {
            self.ctrl_armed = false;
            self.ctrl_since = None;
        }
        false
    }

    /// Focus đổi / hotkey khác đã xử lý: hủy lần bấm đang chờ.
    pub fn reset(&mut self) {
        self.armed = false;
        self.ctrl_armed = false;
        self.ctrl_since = None;
    }
}

/// Kết quả engine đã giải mã (không phụ thuộc layout C của `ime_result_v1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineStep {
    pub action: u32,
    pub delete_count: u16,
    pub insert: Vec<char>,
    pub flags: u32,
}

impl EngineStep {
    pub fn pass() -> EngineStep {
        EngineStep {
            action: ACTION_PASS,
            delete_count: 0,
            insert: Vec::new(),
            flags: 0,
        }
    }

    pub fn from_result(r: &ime_result_v1) -> EngineStep {
        let len = usize::from(r.insert_len).min(r.insert.len());
        EngineStep {
            action: r.action,
            delete_count: r.delete_count,
            insert: r.insert[..len]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect(),
            flags: r.flags,
        }
    }

    fn word_end(&self) -> bool {
        self.flags & IME_FLAG_WORD_END != 0
    }
}

/// Việc lớp COM phải làm trong edit session cho một phím.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionPlan {
    /// Số ký tự đã commit NGAY TRƯỚC composition phải xóa (macro/emoji vượt ranh giới
    /// composition). Lớp COM phải xác minh dịch chuyển đủ, nếu không → fail-open.
    pub delete_before: u16,
    /// Text composition sau phím này; `None` = giữ nguyên.
    pub text: Option<Vec<char>>,
    /// Đóng composition sau khi áp `text`.
    pub end: bool,
    /// `true` = phím bị ăn; `false` = app nhận phím gốc (sau khi composition đã đóng).
    pub eaten: bool,
}

impl CompositionPlan {
    /// Không làm gì với composition, phím đi thẳng.
    pub fn pass_through(composing: bool) -> CompositionPlan {
        CompositionPlan {
            delete_before: 0,
            text: None,
            end: composing,
            eaten: false,
        }
    }
}

/// Tính kế hoạch cho một phím. `current` = text composition hiện tại (rỗng =
/// không composing). Hàm thuần, không đoán text của app.
pub fn plan_key(current: &[char], key: KeyKind, step: &EngineStep) -> CompositionPlan {
    let composing = !current.is_empty();
    match step.action {
        ACTION_PASS => match key {
            // Engine đã nhận ký tự vào từ nhưng chưa biến đổi → TIP tự đưa vào composition.
            KeyKind::Char(c) if !step.word_end() => {
                let mut text = current.to_vec();
                text.push(c);
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end: false,
                    eaten: true,
                }
            }
            // Ranh giới in được (Space, dấu câu, số) khi đang composing: ký tự ranh giới
            // vào cuối composition rồi commit CÙNG từ, phím bị ăn. Để app tự chèn ký tự
            // thì với app IMM32 (CUAS: Notepad cổ điển, WinForms…) kết quả composition
            // tới SAU WM_CHAR → " được" thay vì "được " (bắt được bằng test gõ thật trên
            // Windows). Một lần commit nguyên khối đúng thứ tự ở mọi loại app.
            KeyKind::Char(c) if composing => {
                let mut text = current.to_vec();
                text.push(c);
                CompositionPlan {
                    delete_before: 0,
                    text: Some(text),
                    end: true,
                    eaten: true,
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
                }
            }
            // Ranh giới / phím điều hướng / Ctrl+Space không khớp dự đoán: commit
            // nguyên văn rồi để app xử lý phím. Ctrl+Space KHÔNG được giữ composition
            // mở (R2-121): engine đã nhả từ (WORD_END) — composition còn mở thì phím
            // kế tiếp lệch model, và overlay on-demand sửa vào giữa composition.
            _ => CompositionPlan::pass_through(composing),
        },
        ACTION_REPLACE | ACTION_RESTORE | ACTION_COMMIT => {
            let (mut text, delete_before) = if step.action == ACTION_COMMIT {
                (current.to_vec(), 0)
            } else {
                let del = usize::from(step.delete_count);
                if del <= current.len() {
                    (current[..current.len() - del].to_vec(), 0)
                } else {
                    (Vec::new(), (del - current.len()) as u16)
                }
            };
            text.extend_from_slice(&step.insert);
            let mut eaten = true;
            // RESTORE/COMMIT ở ranh giới: engine đặt ký tự ranh giới cuối `insert`.
            // Ký tự in được ở lại trong text commit (phím bị ăn, như nhánh PASS ở trên).
            // Enter/Tab thì bỏ ra và để app nhận phím gốc — Enter phải là phím Enter
            // thật (chat gửi tin), không phải ký tự `\n` chèn vào text.
            // REPLACE có WORD_END là macro/emoji: trigger bị nuốt theo P0-3 §1.1.
            if step.word_end() && step.action != ACTION_REPLACE {
                if let Some(b) = key.native_boundary_char() {
                    if text.last() == Some(&b) {
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
            }
        }
        // Action lạ (ABI mới hơn adapter): an toàn nhất là commit và để phím đi thẳng.
        _ => CompositionPlan::pass_through(composing),
    }
}

/// Text sau khi áp kế hoạch — dùng cho model của adapter sau khi COM áp thành công.
pub fn text_after(current: &[char], plan: &CompositionPlan) -> Vec<char> {
    if plan.end {
        return Vec::new();
    }
    plan.text.clone().unwrap_or_else(|| current.to_vec())
}

#[cfg(test)]
mod tests {
    /// R2-161: thanh tác vụ/khay/màn hình nền (cửa sổ gốc) không có ô nhập; ô nhập XAML ở app khác thì có.
    #[test]
    fn non_text_top_level_classes() {
        for c in [
            "Shell_TrayWnd",
            "Shell_SecondaryTrayWnd",
            "Progman",
            "WorkerW",
            "NotifyIconOverflowWindow",
        ] {
            assert!(is_non_text_top_level_class(c), "{c}");
        }
        for c in [
            "Notepad",
            "CabinetWClass",
            "Chrome_WidgetWin_1",
            "Windows.UI.Core.CoreWindow",
            "ApplicationFrameWindow",
            "",
        ] {
            assert!(!is_non_text_top_level_class(c), "{c}");
        }
    }

    /// R2-161: terminal không hỗ trợ composition inline do hook sở hữu → TIP nhường.
    #[test]
    fn hook_owned_terminals_are_yielded() {
        for exe in [
            "mobaxterm.exe",
            "mobaxterm_personal_24.exe",
            "putty.exe",
            "mintty.exe",
            "kitty.exe",
            "ttermpro.exe",
            "securecrt.exe",
            "xshell.exe",
        ] {
            assert!(is_hook_owned_exe(exe), "{exe}");
        }
        // Không nhường: app thường, Windows Terminal (inline composition), cmd/pwsh theo role, game chat
        // (hook chỉ bật khi người dùng cho phép — enabled_default=false).
        for exe in [
            "notepad.exe",
            "code.exe",
            "windowsterminal.exe",
            "pwsh.exe",
            "cmd.exe",
            "cs2.exe",
            "winword.exe",
        ] {
            assert!(!is_hook_owned_exe(exe), "{exe}");
        }
    }

    /// R2-159: control không nhập văn bản (danh sách/cây/nền Explorer) → phím phải tới app,
    /// không mở composition (tránh khung Cicero ở góc trái màn hình).
    #[test]
    fn non_text_window_classes_are_recognised() {
        for c in [
            "SysListView32",
            "DirectUIHWND",
            "SHELLDLL_DefView",
            "SysTreeView32",
            "Progman",
            "WorkerW",
            "ToolbarWindow32",
            "ListBox",
            "#32769",
            "syslistview32",
            "WindowsForms10.SysListView32.app.0.141b42a_r6_ad1",
            "WindowsForms10.SysTreeView32.app.0.141b42a_r6_ad1",
            "WindowsForms10.BUTTON.app.0.141b42a_r6_ad1",
        ] {
            assert!(is_non_text_window_class(c), "{c}");
        }
        // Ô nhập và app tự vẽ ô nhập KHÔNG được chặn.
        for c in [
            "Edit",
            "RichEdit20W",
            "RICHEDIT50W",
            "ComboBox",
            "Scintilla",
            "TEdit",
            "Chrome_RenderWidgetHostHWND",
            "Windows.UI.Core.CoreWindow",
            "Notepad",
            "SearchEditBoxWrapperClass",
            "WindowsForms10.EDIT.app.0.141b42a_r6_ad1",
            "WindowsForms10.RichEdit20W.app.0.141b42a_r6_ad1",
            "WindowsForms10.Window.8.app.0.141b42a_r6_ad1",
            "",
        ] {
            assert!(!is_non_text_window_class(c), "{c}");
        }
    }

    use super::*;
    use crate::EngineSession;

    /// Tài liệu mô phỏng: `committed` + composition ở cuối, caret sau composition.
    #[derive(Default)]
    struct Doc {
        committed: Vec<char>,
        comp: Vec<char>,
    }

    impl Doc {
        fn text(&self) -> String {
            self.committed.iter().chain(self.comp.iter()).collect()
        }
    }

    fn press(doc: &mut Doc, engine: &mut EngineSession, vk: u32, ch: u32) -> CompositionPlan {
        let key = KeyKind::classify(vk, ch);
        let r = engine
            .key_event_raw(vk, key.engine_ch(), 0)
            .expect("ime_key");
        let plan = plan_key(&doc.comp, key, &EngineStep::from_result(&r));
        if plan.delete_before > 0 {
            let n = usize::from(plan.delete_before);
            assert!(
                n <= doc.committed.len(),
                "delete_before vượt text đã commit"
            );
            let keep = doc.committed.len() - n;
            doc.committed.truncate(keep);
        }
        if let Some(t) = &plan.text {
            doc.comp = t.clone();
        }
        if plan.end {
            let comp = std::mem::take(&mut doc.comp);
            doc.committed.extend(comp);
        }
        if !plan.eaten {
            assert!(
                doc.comp.is_empty(),
                "phím đi tới app khi composition còn mở: {:?}",
                doc.comp
            );
            match key {
                KeyKind::Char(c) => doc.committed.push(c),
                KeyKind::Enter => doc.committed.push('\n'),
                KeyKind::Tab => doc.committed.push('\t'),
                KeyKind::Backspace => {
                    doc.committed.pop();
                }
                KeyKind::Escape | KeyKind::CtrlSpace | KeyKind::Other => {}
            }
        }
        plan
    }

    fn type_str(doc: &mut Doc, engine: &mut EngineSession, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => press(doc, engine, vk::RETURN, 0),
                '\u{8}' => press(doc, engine, vk::BACK, 0),
                '\u{1b}' => press(doc, engine, vk::ESCAPE, 0),
                '\t' => press(doc, engine, vk::TAB, 0),
                ' ' => press(doc, engine, 0x20, ' ' as u32),
                c => press(doc, engine, 0, c as u32),
            };
        }
    }

    fn run(config: &[u8], input: &str) -> Doc {
        let mut engine = EngineSession::new().unwrap();
        if !config.is_empty() {
            engine.reload_config(config).unwrap();
        }
        let mut doc = Doc::default();
        type_str(&mut doc, &mut engine, input);
        doc
    }

    const NO_CAPS: &[u8] = br#"{"config_version":1,"auto_capitalize":false}"#;

    #[test]
    fn classify_never_turns_non_char_vk_into_letters() {
        assert_eq!(KeyKind::classify(0x70, 0), KeyKind::Other); // F1 ≠ 'p'
        assert_eq!(KeyKind::classify(0x2E, 0), KeyKind::Other); // Delete ≠ '.'
        assert_eq!(KeyKind::classify(0x25, 0), KeyKind::Other); // Left ≠ '%'
        assert_eq!(KeyKind::classify(0x0D, '\r' as u32), KeyKind::Enter);
        assert_eq!(KeyKind::classify(0x08, 8), KeyKind::Backspace);
        assert_eq!(KeyKind::classify(0x41, 'a' as u32), KeyKind::Char('a'));
        assert_eq!(KeyKind::classify(0x20, ' ' as u32), KeyKind::Char(' '));
        assert_eq!(KeyKind::Enter.engine_ch(), 0);
        assert_eq!(
            KeyKind::classify_with_mods(0x20, 0, textvn_ffi::MOD_CTRL),
            KeyKind::CtrlSpace
        );
        assert_eq!(
            KeyKind::classify_with_mods(0x20, ' ' as u32, 0),
            KeyKind::Char(' ')
        );
        assert!(is_modifier_vk(vk::SHIFT) && is_modifier_vk(0xA3) && !is_modifier_vk(0x41));
    }

    #[test]
    fn telex_word_is_composed_then_committed_on_space() {
        let doc = run(NO_CAPS, "dduocj ");
        assert_eq!(doc.text(), "được ");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn printable_boundary_is_committed_with_the_word_enter_stays_native() {
        // App IMM32 (CUAS) nhận kết quả composition SAU WM_CHAR của phím không bị ăn:
        // ký tự ranh giới in được phải nằm trong chính lần commit, đúng thứ tự.
        let cfg = br#"{"config_version":1,"auto_capitalize":false,"auto_restore_english":true}"#;
        for (input, word) in [("dduocj", "được"), ("asdf", "asdf"), ("hello", "hello")] {
            let mut engine = EngineSession::new().unwrap();
            engine.reload_config(cfg).unwrap();
            let mut doc = Doc::default();
            type_str(&mut doc, &mut engine, input);
            let plan = press(&mut doc, &mut engine, 0xBC, ',' as u32);
            let want: Vec<char> = format!("{word},").chars().collect();
            assert_eq!(plan.text.as_deref(), Some(&want[..]), "{input}");
            assert!(plan.eaten && plan.end, "{input}: {plan:?}");

            let mut engine = EngineSession::new().unwrap();
            engine.reload_config(cfg).unwrap();
            let mut doc = Doc::default();
            type_str(&mut doc, &mut engine, input);
            let plan = press(&mut doc, &mut engine, vk::RETURN, 0);
            assert!(!plan.eaten && plan.end, "{input}: Enter phải tới app");
            assert_eq!(doc.text(), format!("{word}\n"));
        }
        // Không composing: phím đi thẳng như bàn phím thường.
        let mut engine = EngineSession::new().unwrap();
        let mut doc = Doc::default();
        let plan = press(&mut doc, &mut engine, 0x20, ' ' as u32);
        assert!(!plan.eaten && !plan.end && plan.text.is_none());
    }

    #[test]
    fn composition_holds_whole_word_until_boundary() {
        let doc = run(NO_CAPS, "Vieetj");
        assert_eq!(doc.committed, Vec::<char>::new(), "chưa commit giữa từ");
        assert_eq!(doc.text(), "Việt");
        let doc = run(NO_CAPS, "Vieetj Nam");
        assert_eq!(doc.text(), "Việt Nam");
    }

    #[test]
    fn english_letters_pass_unchanged() {
        assert_eq!(run(NO_CAPS, "hello world ").text(), "hello world ");
    }

    #[test]
    fn enter_commits_before_app_receives_key_b2() {
        let doc = run(NO_CAPS, "chaof banj\n");
        assert_eq!(doc.text(), "chào bạn\n");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn backspace_inside_word_refolds_and_empty_word_ends_composition() {
        // R2-55: xoá ký tự cuối đang hiển thị (như UniKey), dấu thanh còn lại giữ nguyên.
        let doc = run(NO_CAPS, "dduocj\u{8}");
        assert_eq!(doc.text(), "đượ");
        let doc = run(NO_CAPS, "ab\u{8}\u{8}\u{8}");
        assert_eq!(doc.text(), "");
        let doc = run(NO_CAPS, "ab \u{8}\u{8}");
        assert_eq!(
            doc.text(),
            "a",
            "Backspace sau commit là Backspace thật của app"
        );
    }

    #[test]
    fn escape_restores_raw_keys() {
        let doc = run(NO_CAPS, "dduocj\u{1b}");
        assert_eq!(doc.text(), "dduocj");
        assert!(doc.comp.is_empty());
    }

    #[test]
    fn auto_restore_english_keeps_boundary_key_native() {
        let cfg = br#"{"config_version":1,"auto_capitalize":false,"auto_restore_english":true}"#;
        assert_eq!(run(cfg, "asdf ").text(), "asdf ");
        assert_eq!(run(cfg, "asdf\n").text(), "asdf\n");
    }

    #[test]
    fn digits_and_punctuation_are_boundaries_in_telex() {
        assert_eq!(run(NO_CAPS, "vieetj2026.").text(), "việt2026.");
    }

    #[test]
    fn vni_markers_stay_inside_composition() {
        let cfg = br#"{"config_version":1,"method":"vni","auto_capitalize":false}"#;
        assert_eq!(run(cfg, "tie6ng1 Vie65t ").text(), "tiếng Việt ");
    }

    #[test]
    fn navigation_key_commits_as_is_and_starts_new_word() {
        let mut engine = EngineSession::new().unwrap();
        engine.reload_config(NO_CAPS).unwrap();
        let mut doc = Doc::default();
        type_str(&mut doc, &mut engine, "vie");
        let plan = press(&mut doc, &mut engine, 0x25, 0); // Left
        assert!(!plan.eaten && plan.end);
        assert_eq!(doc.text(), "vie");
        type_str(&mut doc, &mut engine, "ej");
        // "ej" là từ mới → "ẹ"; "vie" đã commit giữ nguyên (không thành "viẹ").
        assert_eq!(doc.committed.iter().collect::<String>(), "vie");
        assert_eq!(doc.text(), "vieẹ");
    }

    #[test]
    fn macro_trigger_is_swallowed_and_expanded() {
        let cfg = r#"{"config_version":1,"auto_capitalize":false,"macro_trigger":"tab","macros":[{"trigger":"cty","expand":"Công ty TNHH","when":"always"}]}"#;
        let doc = run(cfg.as_bytes(), "cty\t");
        assert_eq!(doc.text(), "Công ty TNHH");
    }

    /// OP-003 qua adapter: `=<số>bc` + Space (macro_trigger = space) — phần
    /// `=` và chữ số đã commit TRƯỚC composition "bc" nên phải xoá vượt ranh
    /// giới composition (delete_before).
    #[test]
    fn op003_money_shorthand_through_adapter() {
        for trig in ["space", "tab"] {
            let cfg = format!(r#"{{"config_version":1,"macro_trigger":"{trig}"}}"#);
            let key = if trig == "space" { " " } else { "\t" };
            let doc = run(cfg.as_bytes(), &format!("=15000000bc{key}"));
            assert_eq!(doc.text(), "Mười lăm triệu đồng chẵn", "trigger {trig}");
        }
    }

    #[test]
    fn auto_capitalize_goes_into_composition() {
        let cfg = br#"{"config_version":1,"auto_capitalize":true}"#;
        assert_eq!(run(cfg, "chao. ban").text(), "chao. Ban");
    }

    #[test]
    fn ctrl_shift_tap_toggles_once_and_only_without_other_keys() {
        let mut t = ModifierToggle::default();
        // Ctrl ↓, Shift ↓, Shift ↑ → toggle; pha thứ hai (OnKeyUp) không toggle lại.
        t.on_key_down(0xA2, false, false, false);
        t.on_key_down(0xA0, true, false, false);
        assert!(t.on_key_up(0xA0));
        assert!(!t.on_key_up(0xA0));
        assert!(!t.on_key_up(0xA2));

        // Shift trước Ctrl cũng được; auto-repeat của modifier giữ trạng thái.
        t.on_key_down(vk::SHIFT, false, false, false);
        t.on_key_down(vk::CONTROL, false, true, false);
        t.on_key_down(vk::CONTROL, false, true, false);
        assert!(t.on_key_up(vk::CONTROL));

        // Ctrl+Shift+Z (redo) / Ctrl+Shift+Space: phím thứ ba hủy lần bấm.
        t.on_key_down(vk::CONTROL, false, false, false);
        t.on_key_down(vk::SHIFT, true, false, false);
        t.on_key_down(0x5A, true, true, false);
        assert!(!t.on_key_up(vk::SHIFT));

        // Kèm Alt/Win không tính; reset() hủy khi focus đổi.
        t.on_key_down(vk::SHIFT, true, false, true);
        assert!(!t.on_key_up(vk::SHIFT));
        t.on_key_down(vk::SHIFT, true, false, false);
        t.reset();
        assert!(!t.on_key_up(vk::SHIFT));

        // Chỉ một modifier: không toggle.
        t.on_key_down(vk::SHIFT, false, false, false);
        assert!(!t.on_key_up(vk::SHIFT));
    }

    #[test]
    fn plan_invariant_unknown_action_is_fail_open() {
        let step = EngineStep {
            action: 99,
            delete_count: 3,
            insert: vec!['x'],
            flags: 0,
        };
        let plan = plan_key(&['a', 'b'], KeyKind::Char('c'), &step);
        assert_eq!(plan, CompositionPlan::pass_through(true));
    }

    #[test]
    fn ctrl_space_plan_never_inserts_space_on_pass() {
        let step = EngineStep::pass();
        let plan = plan_key(&['a', 'b'], KeyKind::CtrlSpace, &step);
        assert!(!plan.eaten);
        assert_eq!(plan.text, None);
        assert_eq!(plan.delete_before, 0);
        // R2-121: chốt composition (engine đã nhả từ) — không giữ mở.
        assert!(plan.end);
        // Không composing: không có gì để chốt.
        assert!(!plan_key(&[], KeyKind::CtrlSpace, &step).end);
    }

    #[test]
    fn delete_beyond_composition_is_reported_not_guessed() {
        let step = EngineStep {
            action: ACTION_REPLACE,
            delete_count: 4,
            insert: vec!['😀'],
            flags: IME_FLAG_WORD_END,
        };
        let plan = plan_key(&['a'], KeyKind::Tab, &step);
        assert_eq!(plan.delete_before, 3);
        assert_eq!(plan.text, Some(vec!['😀']));
        assert!(plan.end && plan.eaten);
        assert!(text_after(&['a'], &plan).is_empty());
    }

    #[test]
    fn standalone_ctrl_tap_locks_valid_word_and_restores_invalid_syllable() {
        let mut t = ModifierToggle::default();
        // Ctrl ↓, Ctrl ↑ → ctrl tap (1 lần), không toggle Ctrl+Shift.
        t.on_key_down(vk::CONTROL, false, false, false);
        assert!(!t.on_key_up(vk::CONTROL));
        assert!(t.on_ctrl_tap_up(vk::CONTROL));
        assert!(!t.on_ctrl_tap_up(vk::CONTROL));

        for exe in [
            "windowsterminal.exe",
            "pwsh.exe",
            "mobaxterm_personal_24.exe",
        ] {
            assert!(is_terminal_exe(exe), "{exe}");
        }
        for exe in ["notepad.exe", "code.exe", "winword.exe"] {
            assert!(!is_terminal_exe(exe), "{exe}");
        }

        // Mọi terminal trong appdb phải có mặt ở đây — hai danh sách không được lệch.
        let appdb = include_str!("../../../data/appdb.default.json");
        for line in appdb
            .lines()
            .filter(|l| l.contains("\"id\":\"win.") && l.contains("ForwardAsCommit"))
        {
            for part in line.split("\"exe\":\"").skip(1) {
                let exe = part.split('"').next().unwrap_or("").to_lowercase();
                assert!(
                    is_terminal_exe(&exe),
                    "appdb có {exe} nhưng thiếu ở is_terminal_exe"
                );
            }
        }

        // Ctrl+C → không phải ctrl tap.
        t.on_key_down(vk::CONTROL, false, false, false);
        t.on_key_down(0x43, true, false, false);
        assert!(!t.on_ctrl_tap_up(vk::CONTROL));

        // Giữ Ctrl quá lâu (Ctrl+click, Ctrl+lăn chuột) → không phải tap; phím lặp không dời mốc.
        let t0 = std::time::Instant::now();
        t.on_key_down(vk::CONTROL, false, false, false);
        t.mark_ctrl_down(t0);
        t.on_key_down(vk::CONTROL, false, false, false);
        t.mark_ctrl_down(t0 + CTRL_TAP_MAX);
        assert!(!t.ctrl_held_too_long(t0 + CTRL_TAP_MAX));
        assert!(t.ctrl_held_too_long(t0 + CTRL_TAP_MAX + std::time::Duration::from_millis(1)));
        assert!(t.on_ctrl_tap_up(vk::CONTROL));

        // 1) `tunx` + Ctrl tap + `s` → giữ nguyên tử `tũns` (Ctrl tap chốt composition `tũn`).
        let mut eng = EngineSession::new().unwrap();
        let _ = eng.reload_config(b"{\"auto_capitalize\":false}");
        let mut doc = Doc::default();
        type_str(&mut doc, &mut eng, "tunx");
        assert_eq!(doc.text(), "tũn");
        let r = eng.ctrl_tap().unwrap();
        let plan = plan_key(&doc.comp, KeyKind::Other, &EngineStep::from_result(&r));
        assert!(
            plan.end,
            "Ctrl tap trên âm tiết hợp lệ phải chốt composition"
        );
        if let Some(txt) = &plan.text {
            doc.comp = txt.clone();
        }
        doc.committed.append(&mut doc.comp);
        type_str(&mut doc, &mut eng, "s ");
        assert_eq!(doc.text(), "tũns ");

        // 2) `sort` (`sỏt`) + Ctrl tap → khôi phục ngay thành `sort` và chốt composition.
        let mut eng2 = EngineSession::new().unwrap();
        let _ = eng2.reload_config(b"{\"auto_capitalize\":false,\"auto_restore_english\":true}");
        let mut doc2 = Doc::default();
        type_str(&mut doc2, &mut eng2, "sort");
        assert_eq!(doc2.text(), "sỏt");
        let r2 = eng2.ctrl_tap().unwrap();
        let plan2 = plan_key(&doc2.comp, KeyKind::Other, &EngineStep::from_result(&r2));
        assert!(plan2.text.is_some() && plan2.eaten);
        if let Some(txt) = &plan2.text {
            doc2.comp = txt.clone();
        }
        assert_eq!(doc2.text(), "sort");
    }

    #[test]
    fn mode_en_english_completion_works_within_composition() {
        let mut eng = EngineSession::new().unwrap();
        let _ = eng.reload_config(
            b"{\"config_version\":1,\"auto_capitalize\":false,\"suggest\":{\"enabled\":true}}",
        );
        eng.set_enabled(false);
        let mut doc = Doc::default();
        type_str(&mut doc, &mut eng, "devel");
        assert_eq!(doc.text(), "devel");
        assert_eq!(eng.peek_completion().as_deref(), Some("op"));
        press(&mut doc, &mut eng, vk::TAB, 0);
        assert_eq!(doc.text(), "develop");
    }

    // ====================================================================
    // Ma trận chống lệch máy trạng thái (E47 — yêu cầu audit "1.000 case")
    // ====================================================================

    /// Ma trận kế hoạch phím: MỌI KeyKind × MỌI action engine × composing/idle.
    /// Bất biến máy trạng thái composition (tài liệu compose.rs):
    /// - P1: phím KHÔNG bị ăn khi đang composing ⇒ composition phải đóng trước
    ///   khi app nhận phím (`end == true`);
    /// - P2: `delete_before > 0` chỉ khi engine xoá vượt composition
    ///   (delete_count > len) — không bao giờ xoá mù text của app;
    /// - P3: `end == true` ⇒ model sau phím rỗng (composition đã commit);
    /// - P4: action lạ ⇒ fail-open (`pass_through`).
    #[test]
    fn plan_key_cross_product_state_invariants() {
        use textvn_ffi::{IME_FLAG_WORD_END as WORD_END, IME_OK};
        let _ = IME_OK;
        let kinds = [
            KeyKind::Char('a'),
            KeyKind::Char(' '),
            KeyKind::Backspace,
            KeyKind::Escape,
            KeyKind::Enter,
            KeyKind::Tab,
            KeyKind::CtrlSpace,
            KeyKind::Other,
        ];
        let steps = [
            EngineStep::pass(),
            EngineStep {
                action: ACTION_PASS,
                delete_count: 0,
                insert: Vec::new(),
                flags: WORD_END,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 1,
                insert: vec!['đ'],
                flags: 0,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 2,
                insert: vec!['x'],
                flags: 0,
            },
            EngineStep {
                action: ACTION_RESTORE,
                delete_count: 1,
                insert: vec!['a', 's', 'd', 'f'],
                flags: WORD_END,
            },
            EngineStep {
                action: ACTION_COMMIT,
                delete_count: 0,
                insert: vec!['m'],
                flags: WORD_END,
            },
            EngineStep {
                action: 99,
                delete_count: 3,
                insert: vec!['?'],
                flags: 0,
            },
        ];
        let currents: Vec<Vec<char>> = vec![Vec::new(), vec!['đ', 'ư'], vec!['a', 'b', 'c']];
        let mut checked = 0usize;
        for current in &currents {
            let composing = !current.is_empty();
            for kind in kinds {
                for step in &steps {
                    let plan = plan_key(current, kind, step);
                    // P1
                    if composing && !plan.eaten {
                        assert!(
                            plan.end,
                            "P1 {kind:?}: phím tới app mà composition còn mở ({plan:?})"
                        );
                    }
                    // P2
                    if plan.delete_before > 0 {
                        assert!(
                            usize::from(step.delete_count) > current.len()
                                && step.action != ACTION_PASS,
                            "P2 {kind:?}: delete_before chỉ khi xoá vượt composition"
                        );
                        assert_eq!(
                            plan.delete_before,
                            step.delete_count - current.len() as u16,
                            "P2 {kind:?}: sai số đếm xoá vượt"
                        );
                    }
                    // P3
                    if plan.end {
                        assert!(text_after(current, &plan).is_empty(), "P3 {kind:?}");
                    } else if composing {
                        assert!(
                            plan.text.is_some(),
                            "P3 {kind:?}: giữ composition mà không có text"
                        );
                    }
                    // P4
                    if step.action == 99 {
                        assert_eq!(
                            plan,
                            CompositionPlan::pass_through(composing),
                            "P4 {kind:?}"
                        );
                    }
                    // Bất biến văn bản: REPLACE/RESTORE luôn giữ phần đầu của composition.
                    if matches!(step.action, ACTION_REPLACE | ACTION_RESTORE | ACTION_COMMIT)
                        && step.delete_count as usize <= current.len()
                    {
                        let keep = current[..current.len() - step.delete_count as usize].to_vec();
                        let mut want = keep;
                        want.extend_from_slice(&step.insert);
                        assert_eq!(
                            plan.text.as_deref(),
                            Some(&want[..]),
                            "{kind:?}: mất/giữ sai phần đầu composition"
                        );
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(
            checked,
            currents.len() * kinds.len() * steps.len(),
            "ma trận plan_key thiếu case"
        );
        assert!(checked >= 150, "plan_key ma trận chỉ có {checked} case");
    }

    /// Mô phỏng đổi focus GIỮA từ (OnSetFocus: commit composition + reset engine)
    /// — text cửa sổ CŨ không được bị xoá lẹm bởi phím của cửa sổ MỚI.
    #[test]
    fn focus_switch_mid_word_commits_and_never_deletes_old_text() {
        let cfg = br#"{"config_version":1,"auto_capitalize":false,"auto_restore_english":true}"#;
        let words = ["dduocj", "Vieetj", "text", "hello", "chaof"];
        let mut checked = 0usize;
        for word in words {
            let chars: Vec<char> = word.chars().collect();
            for cut in 1..chars.len() {
                // Cửa sổ A: gõ phần đầu từ.
                let mut eng = EngineSession::new().unwrap();
                let _ = eng.reload_config(cfg);
                let mut doc = Doc::default();
                for &c in &chars[..cut] {
                    press(&mut doc, &mut eng, 0, c as u32);
                }
                // OnSetFocus (tip.rs): commit composition vào doc cũ, reset engine.
                doc.committed.append(&mut doc.comp);
                eng.reset();
                let old_text = doc.text();
                // Cửa sổ B (mô hình mới): gõ phần còn lại như một từ mới.
                let mut doc_b = Doc::default();
                for &c in &chars[cut..] {
                    press(&mut doc_b, &mut eng, 0, c as u32);
                }
                // Không xoá lẹm: text cửa sổ A bất biến sau mọi phím cửa sổ B.
                assert_eq!(
                    doc.text(),
                    old_text,
                    "{word}/{cut}: phím cửa sổ mới xoá text cửa sổ cũ"
                );
                // delete_before của mọi phím cửa sổ B không vượt composition của B
                // (Doc::press đã assert sẵn) — ở đây kiểm thêm tổng thể.
                assert!(
                    doc_b.comp.len() <= chars.len(),
                    "{word}/{cut}: comp dài bất thường"
                );
                checked += 1;
            }
        }
        assert!(checked >= 20, "ma trận focus chỉ có {checked} case");
    }

    /// Ma trận Backspace trong composition (R2-55 qua kế hoạch compose): text
    /// composition phải đúng = display trừ 1 glyph, không âm dư, không nhân đôi.
    #[test]
    fn backspace_in_composition_matrix() {
        let cfg = br#"{"config_version":1,"auto_capitalize":false}"#;
        let cases = [
            ("dduocj", "đượ"),
            ("dduowcj", "đượ"),
            ("vieetj", "việ"),
            ("tieeng", "tiên"),
            ("aaaa", "aa"),
            ("aww", "a"),
            ("asss", "as"),
            ("hello", "hell"),
        ];
        for (word, want) in cases {
            let mut eng = EngineSession::new().unwrap();
            let _ = eng.reload_config(cfg);
            let mut doc = Doc::default();
            type_str(&mut doc, &mut eng, word);
            press(&mut doc, &mut eng, vk::BACK, 0);
            assert_eq!(doc.text(), want, "`{word}` + BS trong composition");
        }
        // BS đúp liên tiếp và BS hết từ.
        for word in ["dduocj", "vieetj", "text"] {
            let mut eng = EngineSession::new().unwrap();
            let _ = eng.reload_config(cfg);
            let mut doc = Doc::default();
            type_str(&mut doc, &mut eng, word);
            for _ in 0..word.chars().count() + 2 {
                press(&mut doc, &mut eng, vk::BACK, 0);
            }
            assert!(
                doc.text().is_empty(),
                "`{word}` + BS vượt phải về rỗng, không âm dư: {:?}",
                doc.text()
            );
        }
    }

    /// R2-161/E46 — hợp đồng nguồn: MỌI preset `engine_owner=hook` trong appdb
    /// phải được `is_hook_owned_exe` nhận (nhường phím), và MỌI exe trong danh
    /// sách phải là terminal (`is_terminal_exe`) — hai danh sách không được lệch.
    #[test]
    fn hook_owned_yield_covers_whole_appdb_and_is_terminal() {
        let appdb = include_str!("../../../data/appdb.default.json");
        let mut count = 0;
        // Chỉ preset hook-owned KHÔNG gắn `when` (mọi role) mới được nhường ở
        // role 0; cmd/pwsh/windowsterminal gắn role terminal hoặc engine_owner
        // tsf — cố ý không nhường (composition inline hoạt động).
        for line in appdb.lines().filter(|l| {
            l.contains("ForwardAsCommit")
                && l.contains("\"engine_owner\":\"hook\"")
                && !l.contains("\"when\"")
        }) {
            for part in line.split("\"exe\":\"").skip(1) {
                let exe = part.split('"').next().unwrap_or("").to_lowercase();
                if exe.starts_with("mobaxterm_") {
                    continue; // exe_prefix — kiểm bằng biến thể riêng bên dưới
                }
                assert!(
                    is_hook_owned_exe(&exe),
                    "appdb có `{exe}` (hook, mọi role) nhưng is_hook_owned_exe không nhường"
                );
                assert!(is_terminal_exe(&exe), "`{exe}` phải là terminal");
                count += 1;
            }
        }
        assert!(count >= 8, "appdb chỉ có {count} preset hook-owned");
        // Danh sách âm: app có composition inline KHÔNG được nhường.
        for exe in [
            "notepad.exe",
            "code.exe",
            "winword.exe",
            "excel.exe",
            "chrome.exe",
            "windowsterminal.exe",
            "explorer.exe",
        ] {
            assert!(!is_hook_owned_exe(exe), "{exe} không được nhường");
            count += 1;
        }
        // Họ MobaXterm mọi biến thể tên file đều nhường (prefix match).
        for exe in [
            "MobaXterm.exe",
            "mobaxterm_personal_24.2.exe",
            "MobaXterm_PES_v24.exe",
        ] {
            assert!(is_hook_owned_exe(exe) || is_terminal_exe(exe), "{exe}");
            count += 1;
        }
        assert!(
            count >= 15,
            "ma trận hook-owned chỉ có {count} case — appdb/danh sách bị cắt?"
        );
    }

    // ====================================================================
    // Ma trận máy trạng thái TSF/focus mở rộng (audit ≥ 1.000 case MỚI —
    // nhóm Bug 1: xung đột IME/TSF, khung nổi góc trái, focus chuyển nhanh)
    // ====================================================================

    /// T1 — cross-product mở rộng của `plan_key`: 5 trạng thái composition ×
    /// 8 loại phím × 12 bước engine. Bất biến mới so với
    /// `plan_key_cross_product_state_invariants`:
    /// - P9: `ACTION_PASS` BỎ QUA `insert` của engine (không chèn phím lạ);
    /// - P5 (nâng cấp): RESTORE/COMMIT + WORD_END với Enter/Tab → phím native
    ///   bị nhả (`eaten=false`, ký tự `\n`/`\t` bị pop khỏi text commit) —
    ///   Enter phải là phím Enter thật của app (chat gửi tin);
    /// - P10: Space trong RESTORE/COMMIT commit CÙNG khối (eaten=true);
    /// - P2 mở rộng cho current rỗng: `delete_before == delete_count`.
    #[test]
    fn plan_key_extended_cross_product_matrix() {
        use textvn_ffi::IME_FLAG_WORD_END as WORD_END;
        let currents: Vec<Vec<char>> = vec![
            Vec::new(),
            vec!['đ'],
            vec!['đ', 'ư'],
            vec!['a', 'b', 'c', 'd'],
            vec!['T', 'ẽ', 'x', 't', 'V', 'N', 'x', 'j'],
        ];
        let kinds = [
            KeyKind::Char('a'),
            KeyKind::Char(' '),
            KeyKind::Backspace,
            KeyKind::Escape,
            KeyKind::Enter,
            KeyKind::Tab,
            KeyKind::CtrlSpace,
            KeyKind::Other,
        ];
        let steps: Vec<EngineStep> = vec![
            EngineStep::pass(),
            EngineStep {
                action: ACTION_PASS,
                delete_count: 0,
                insert: vec!['q', 'z'],
                flags: 0,
            }, // P9: PASS phải bỏ qua insert
            EngineStep {
                action: ACTION_PASS,
                delete_count: 0,
                insert: Vec::new(),
                flags: WORD_END,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 0,
                insert: vec!['x'],
                flags: 0,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 1,
                insert: vec!['đ'],
                flags: 0,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 2,
                insert: vec!['ư', 'ơ'],
                flags: 0,
            },
            EngineStep {
                action: ACTION_REPLACE,
                delete_count: 9,
                insert: vec!['x'],
                flags: 0,
            }, // xoá vượt → delete_before
            EngineStep {
                action: ACTION_RESTORE,
                delete_count: 2,
                insert: vec!['t', 'e', 'x', ' '],
                flags: WORD_END,
            }, // ranh giới Space — commit cùng khối
            EngineStep {
                action: ACTION_RESTORE,
                delete_count: 2,
                insert: vec!['d', 'd', 'u', 'o', 'c', 'j', '\n'],
                flags: WORD_END,
            }, // ranh giới Enter — nhả phím native
            EngineStep {
                action: ACTION_COMMIT,
                delete_count: 0,
                insert: vec!['m'],
                flags: WORD_END,
            },
            EngineStep {
                action: ACTION_COMMIT,
                delete_count: 0,
                insert: vec!['h', 'e', 'l', 'l', 'o', '\t'],
                flags: WORD_END,
            }, // ranh giới Tab — nhả phím native
            EngineStep {
                action: 99,
                delete_count: 3,
                insert: vec!['?'],
                flags: 0,
            }, // action lạ — fail-open
        ];
        let mut checked = 0usize;
        for current in &currents {
            let composing = !current.is_empty();
            for kind in kinds {
                for step in &steps {
                    let plan = plan_key(current, kind, step);
                    // P1: phím tới app khi đang composing ⇒ composition phải đóng trước.
                    if composing && !plan.eaten {
                        assert!(plan.end, "P1 {kind:?}: {plan:?}");
                    }
                    // P2: delete_before chỉ khi xoá vượt composition; số đúng công thức.
                    if plan.delete_before > 0 {
                        assert!(
                            usize::from(step.delete_count) > current.len()
                                && step.action != ACTION_PASS,
                            "P2 {kind:?}: delete_before không hợp lệ"
                        );
                        assert_eq!(
                            plan.delete_before,
                            step.delete_count - current.len() as u16,
                            "P2 {kind:?}: sai số đếm xoá vượt"
                        );
                    } else if step.action != ACTION_PASS {
                        assert!(
                            usize::from(step.delete_count) <= current.len() || plan.text.is_none(),
                            "P2 {kind:?}: xoá vượt mà delete_before = 0 — mất chữ của app"
                        );
                    }
                    // P3: end ⇒ model rỗng sau phím (đã commit).
                    if plan.end {
                        assert!(text_after(current, &plan).is_empty(), "P3 {kind:?}");
                    } else if composing {
                        assert!(plan.text.is_some(), "P3 {kind:?}");
                    }
                    // P4: action lạ ⇒ pass_through.
                    if step.action == 99 {
                        assert_eq!(
                            plan,
                            CompositionPlan::pass_through(composing),
                            "P4 {kind:?}"
                        );
                    }
                    // P9: PASS bỏ qua insert của engine.
                    if step.action == ACTION_PASS {
                        if let Some(t) = &plan.text {
                            assert!(
                                step.insert.iter().all(|c| !t.contains(c)),
                                "P9 {kind:?}: PASS chèn insert của engine vào composition"
                            );
                        }
                    }
                    // P5: RESTORE/COMMIT + WORD_END với Enter/Tab → nhả phím native.
                    if step.word_end() && step.action != ACTION_REPLACE {
                        let native = kind.native_boundary_char();
                        let insert_ends_native =
                            step.insert.last().is_some_and(|c| Some(*c) == native);
                        match native {
                            Some(b) if insert_ends_native => {
                                assert!(
                                    !plan.eaten && plan.end,
                                    "P5 {kind:?}: phím native bị ăn — Enter phải tới app"
                                );
                                let t = plan.text.as_ref().expect("P5 text");
                                assert!(
                                    !t.contains(&b),
                                    "P5 {kind:?}: ký tự native còn nằm trong text commit"
                                );
                            }
                            _ => {}
                        }
                    }
                    // P10: Space trong RESTORE/COMMIT ở ranh giới → commit cùng khối (ăn phím).
                    if step.word_end()
                        && step.action != ACTION_REPLACE
                        && matches!(kind, KeyKind::Char(' '))
                        && step.insert.last() == Some(&' ')
                    {
                        assert!(
                            plan.eaten,
                            "P10: Space ranh giới phải commit cùng khối (đúng thứ tự CUAS)"
                        );
                    }
                    // Bất biến văn bản: giữ phần đầu composition. RESTORE/COMMIT
                    // + WORD_END với Enter/Tab: ký tự native bị pop (P5) — head
                    // + insert TRỪ ký tự native cuối.
                    if matches!(step.action, ACTION_REPLACE | ACTION_RESTORE | ACTION_COMMIT)
                        && usize::from(step.delete_count) <= current.len()
                    {
                        let mut insert_want = step.insert.clone();
                        if step.word_end()
                            && step.action != ACTION_REPLACE
                            && kind.native_boundary_char().is_some()
                            && insert_want
                                .last()
                                .is_some_and(|&c| Some(c) == kind.native_boundary_char())
                        {
                            insert_want.pop();
                        }
                        let mut want =
                            current[..current.len() - step.delete_count as usize].to_vec();
                        want.extend_from_slice(&insert_want);
                        assert_eq!(
                            plan.text.as_deref(),
                            Some(&want[..]),
                            "{kind:?}: mất/giữ sai phần đầu composition"
                        );
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, currents.len() * kinds.len() * steps.len());
        assert!(checked >= 400, "T1 chỉ có {checked} case");
    }

    /// T2 — focus chuyển NHANH GIỮA từ, 3 cửa sổ A→B→C (OnSetFocus commit +
    /// reset mỗi lần): text cửa sổ cũ bất biến, cửa sổ mới gõ đúng như engine
    /// mới tinh (không dư trạng thái, không xoá lẹm).
    #[test]
    fn focus_switch_rapid_three_window_matrix() {
        let cfg = br#"{"config_version":1,"auto_capitalize":false,"auto_restore_english":true}"#;
        let words = [
            "dduocj", "Vieetj", "text", "hello", "chaof", "nguowif", "TextVN", "tieeng",
        ];
        let mut checked = 0usize;
        for word in words {
            let chars: Vec<char> = word.chars().collect();
            for a in 1..chars.len() {
                for b in (a + 1)..chars.len() {
                    let mut eng = EngineSession::new().unwrap();
                    let _ = eng.reload_config(cfg);
                    // Cửa sổ A
                    let mut doc_a = Doc::default();
                    for &c in &chars[..a] {
                        press(&mut doc_a, &mut eng, 0, c as u32);
                    }
                    doc_a.committed.append(&mut doc_a.comp);
                    eng.reset();
                    // Cửa sổ B
                    let mut doc_b = Doc::default();
                    for &c in &chars[a..b] {
                        press(&mut doc_b, &mut eng, 0, c as u32);
                    }
                    doc_b.committed.append(&mut doc_b.comp);
                    eng.reset();
                    // Text cũ bất biến — chốt trước khi gõ cửa sổ C.
                    let (sa, sb) = (doc_a.text(), doc_b.text());
                    // Cửa sổ C (gõ ĐÚNG MỘT LẦN phần còn lại)
                    let mut doc_c = Doc::default();
                    for &c in &chars[b..] {
                        press(&mut doc_c, &mut eng, 0, c as u32);
                    }
                    doc_c.committed.append(&mut doc_c.comp);
                    assert_eq!(doc_a.text(), sa, "{word}/{a}/{b}: A bị đụng sau switch");
                    assert_eq!(doc_b.text(), sb, "{word}/{a}/{b}: B bị đụng sau switch");
                    // C phải gõ đúng như engine mới tinh gõ phần còn lại.
                    let mut fresh = EngineSession::new().unwrap();
                    let _ = fresh.reload_config(cfg);
                    let mut doc_fresh = Doc::default();
                    for &c in &chars[b..] {
                        press(&mut doc_fresh, &mut fresh, 0, c as u32);
                    }
                    assert_eq!(
                        doc_c.text(),
                        doc_fresh.text(),
                        "{word}/{a}/{b}: cửa sổ C lệch engine mới tinh — trạng thái mồ côi sau reset"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 60, "T2 chỉ có {checked} case");
    }

    /// T3 — fuzz predicate lớp cửa sổ: biến thể WinForms hash, hoa/thường,
    /// gần-nhưng-khác (SysListView32X…) — cho phép/cấm không được lệch.
    #[test]
    fn non_text_class_predicate_fuzz_matrix() {
        let winforms_hashes = [
            "141b42a_r6_ad1",
            "2a8b8f9",
            "3c9d2e1",
            "4e0f3a2",
            "5f1a4b3",
            "6a2b5c4",
            "7b3c6d5",
            "8c4d7e6",
            "9d5e8f7",
            "ae6f908",
            "bf70a19",
            "c081b2a",
        ];
        let mut checked = 0usize;
        // Positive: 24 lớp chuẩn × 3 biến thể hoa/thường.
        let allow = [
            "SysListView32",
            "SysTreeView32",
            "SysHeader32",
            "SysTabControl32",
            "ToolbarWindow32",
            "ReBarWindow32",
            "msctls_statusbar32",
            "msctls_trackbar32",
            "msctls_updown32",
            "msctls_progress32",
            "SHELLDLL_DefView",
            "DirectUIHWND",
            "CtrlNotifySink",
            "Progman",
            "WorkerW",
            "SysLink",
            "Button",
            "Static",
            "ScrollBar",
            "ListBox",
            "ComboLBox",
            "#32769",
            "ListView20WndClass",
            "TreeView20WndClass",
        ];
        for c in allow {
            for v in [c, &c.to_ascii_uppercase(), &c.to_ascii_lowercase()] {
                assert!(
                    is_non_text_window_class(v),
                    "{v}: lớp non-text chuẩn không được nhận"
                );
                checked += 1;
            }
        }
        // Positive: WinForms prefix với hash tuỳ ý → phần giữa quyết định.
        for seg in [
            "SysListView32",
            "SysTreeView32",
            "BUTTON",
            "Static",
            "ListBox",
        ] {
            for h in winforms_hashes {
                let class = format!("WindowsForms10.{seg}.app.0.{h}");
                assert!(
                    is_non_text_window_class(&class),
                    "{class}: hash WinForms không được phá predicate"
                );
                checked += 1;
            }
        }
        // Positive: cửa sổ gốc (taskbar/desktop) × hoa/thường.
        let roots = [
            "Shell_TrayWnd",
            "Shell_SecondaryTrayWnd",
            "NotifyIconOverflowWindow",
            "TopLevelWindowForOverflowXamlIsland",
            "TaskListThumbnailWnd",
            "TaskListOverlayWnd",
            "Progman",
            "WorkerW",
        ];
        for c in roots {
            for v in [c, &c.to_ascii_uppercase()] {
                assert!(is_non_text_top_level_class(v), "{v}: root non-text sai");
                checked += 1;
            }
        }
        // Negative: ô nhập thật + app tự vẽ ô nhập + gần-nhưng-khác.
        let deny = [
            "Edit",
            "RichEdit20W",
            "RICHEDIT50W",
            "ComboBox",
            "Scintilla",
            "TEdit",
            "Chrome_RenderWidgetHostHWND",
            "Windows.UI.Core.CoreWindow",
            "Notepad",
            "SearchEditBoxWrapperClass",
            "ApplicationFrameWindow",
            "SysListView32X",
            "SysListView321",
            "XSysListView32",
            "SysListView",
            "SysTreeView32X",
            "ToolbarWindow",
            "ComboLBoxX",
            "#32768",
            "#32770",
            "",
        ];
        for c in deny {
            assert!(!is_non_text_window_class(c), "{c}: ô nhập bị chặn oan");
            assert!(!is_non_text_top_level_class(c), "{c}: root bị chặn oan");
            checked += 1;
        }
        // Negative: WinForms EDIT (ô nhập WinForms) không được chặn.
        for h in winforms_hashes {
            for seg in ["EDIT", "RichEdit20W", "Window.8"] {
                let class = format!("WindowsForms10.{seg}.app.0.{h}");
                assert!(!is_non_text_window_class(&class), "{class} bị chặn oan");
                checked += 1;
            }
        }
        assert!(checked >= 130, "T3 chỉ có {checked} case");
    }

    /// T4 — ngữ nghĩa role-0 của `is_hook_owned_exe`: entry appdb gắn `when`
    /// (cmd/powershell/pwsh theo role terminal, mstsc, game chat) KHÔNG được
    /// nhường ở role 0; `exe_prefix` phủ mọi biến thể tên file; hoa/thường
    /// tên exe không đổi kết quả; hai danh sách hook-owned/terminal không lệch.
    #[test]
    fn hook_owned_role_zero_and_prefix_matrix() {
        let appdb = include_str!("../../../data/appdb.default.json");
        let mut checked = 0usize;
        // Entry hook-owned CÓ `when` → role 0 (chưa biết field) KHÔNG nhường.
        for line in appdb.lines().filter(|l| {
            l.contains("ForwardAsCommit")
                && l.contains("\"engine_owner\":\"hook\"")
                && l.contains("\"when\"")
        }) {
            for part in line.split("\"exe\":\"").skip(1) {
                let exe = part.split('"').next().unwrap_or("").to_lowercase();
                assert!(
                    !is_hook_owned_exe(&exe),
                    "{exe}: gắn `when` nhưng bị nhường ở role 0 — lệch ngữ nghĩa role"
                );
                checked += 1;
            }
        }
        // exe_prefix MobaXterm: mọi biến thể tên file (portable/PES/Personal) + hoa/thường.
        for exe in [
            "MobaXterm.exe",
            "MOBAXTERM.EXE",
            "mobaxterm_personal_24.2.exe",
            "MobaXterm_PES_v24.exe",
            "MobaXterm_PES_v25.1.exe",
            "MobaXterm_Portable_v23.6.exe",
        ] {
            assert!(
                is_hook_owned_exe(exe),
                "{exe}: biến thể prefix không được nhường"
            );
            assert!(is_terminal_exe(exe), "{exe}: phải là terminal");
            checked += 1;
        }
        // exe_prefix ConEmu — terminal nhưng KHÔNG hook-owned (engine_owner tsf
        // trong appdb hoặc không có preset) — chỉ cross-check is_terminal_exe.
        for exe in ["ConEmu64.exe", "conemu_x64.exe"] {
            assert!(is_terminal_exe(exe), "{exe}: terminal list sai");
            checked += 1;
        }
        // Hoa/thường exe không đổi kết quả cho cả hai phía.
        for exe in ["PuTTY.exe", "MINTTY.EXE", "SecureCRT.exe", "XShell.exe"] {
            assert!(is_hook_owned_exe(exe), "{exe}: hoa/thường lệch kết quả");
            assert!(is_terminal_exe(exe), "{exe}: terminal list sai");
            checked += 1;
        }
        // Negative mở rộng: app có composition inline.
        for exe in [
            "notepad.exe",
            "code.exe",
            "winword.exe",
            "excel.exe",
            "chrome.exe",
            "msedge.exe",
            "firefox.exe",
            "explorer.exe",
            "windowsterminal.exe",
            "wezterm-gui.exe",
            "alacritty.exe",
            "tabby.exe",
            "hyper.exe",
        ] {
            assert!(!is_hook_owned_exe(exe), "{exe} không được nhường");
            checked += 1;
        }
        assert!(checked >= 25, "T4 chỉ có {checked} case");
    }

    /// T5 — máy trạng thái `ModifierToggle` (Ctrl+Shift switch / Ctrl-tap):
    /// mọi thứ tự nhấn-nhả, can thiệp Alt, chord xen giữa, key-repeat, giữ lâu
    /// — toggle phải búng ĐÚNG MỘT LẦN mỗi lần bấm hợp lệ, không bao giờ nuốt
    /// phím modifier (xung đột input routing — nhóm Bug 1).
    #[test]
    fn modifier_toggle_rapid_sequence_matrix() {
        use std::time::Instant;
        let ctrl_keys = [vk::CONTROL, 0xA2, 0xA3];
        let shift_keys = [vk::SHIFT, vk::LSHIFT, 0xA1];
        let mut checked = 0usize;
        // S1–S4: 2 thứ tự nhấn × 2 thứ tự nhả → đúng 1 lần fire.
        for &ck in &ctrl_keys {
            for &sk in &shift_keys {
                // Shift↓ Ctrl↓ nhả Ctrl
                let mut t = ModifierToggle::default();
                t.on_key_down(sk, false, false, false);
                assert!(!t.on_key_up(sk), "S1 shift nhả sớm khi ctrl chưa xuống");
                t.on_key_down(ck, false, true, false);
                assert!(t.on_key_up(ck), "S1: nhả Ctrl phải fire toggle");
                assert!(!t.on_key_up(sk), "S1: nhả Shift sau đó KHÔNG fire lần 2");
                checked += 1;
                // Ctrl↓ Shift↓ nhả Shift
                let mut t = ModifierToggle::default();
                t.on_key_down(ck, false, false, false);
                t.on_key_down(sk, true, false, false);
                assert!(t.on_key_up(sk), "S2: nhả Shift phải fire toggle");
                assert!(!t.on_key_up(ck), "S2: nhả Ctrl sau đó KHÔNG fire lần 2");
                checked += 1;
                // Alt giữ trong lúc bấm cặp → KHÔNG toggle (Alt+Shift+Ctrl là chord khác).
                let mut t = ModifierToggle::default();
                t.on_key_down(sk, false, false, true);
                t.on_key_down(ck, false, true, true);
                assert!(!t.on_key_up(ck), "S5: Alt giữ vẫn fire — sai");
                assert!(!t.on_key_up(sk), "S5: Alt giữ vẫn fire — sai");
                checked += 1;
                // Key-repeat Ctrl: mốc giữ phải giữ nguyên (or()).
                let mut t = ModifierToggle::default();
                t.on_key_down(ck, false, false, false);
                let now = Instant::now();
                t.mark_ctrl_down(now);
                t.mark_ctrl_down(now + std::time::Duration::from_millis(100));
                assert!(
                    !t.ctrl_held_too_long(now + std::time::Duration::from_millis(200)),
                    "S9: repeat Ctrl phải giữ mốc lần bấm đầu"
                );
                assert!(
                    t.ctrl_held_too_long(now + std::time::Duration::from_millis(450)),
                    "S9: giữ > 400ms phải hết hạn tap (Ctrl+click)"
                );
                assert!(
                    !t.on_key_up(ck),
                    "S9: Ctrl tap đơn (không Shift) không fire switch"
                );
                checked += 1;
                // Chord xen giữa: Shift↓ Ctrl↓ 'x'↓ → armed phải gỡ, nhả không fire.
                let mut t = ModifierToggle::default();
                t.on_key_down(sk, false, false, false);
                t.on_key_down(ck, false, true, false);
                t.on_key_down('X' as u32, true, true, false);
                assert!(!t.on_key_up(ck), "S12: chord xen giữa vẫn fire — sai");
                assert!(!t.on_key_up(sk), "S12: chord xen giữa vẫn fire — sai");
                checked += 1;
            }
        }
        // Hai lần bấm hợp lệ liên tiếp → đúng 2 fire.
        let mut t = ModifierToggle::default();
        let mut fires = 0;
        for _ in 0..2 {
            t.on_key_down(vk::SHIFT, false, false, false);
            t.on_key_down(vk::CONTROL, false, true, false);
            if t.on_key_up(vk::CONTROL) {
                fires += 1;
            }
            if t.on_key_up(vk::SHIFT) {
                fires += 1;
            }
        }
        assert_eq!(fires, 2, "S6: 2 lần bấm phải đúng 2 fire, được {fires}");
        checked += 1;
        assert!(checked >= 16, "T5 chỉ có {checked} scenario");
    }

    /// T6 — ma trận phân loại phím: `classify` không bao giờ biến VK không
    /// sinh ký tự thành chữ (F1 ≠ 'p'), ranh giới native chỉ Enter/Tab,
    /// Ctrl+Space được nhận diện kể cả khi Caps bật.
    #[test]
    fn keykind_classify_boundary_matrix() {
        use textvn_ffi::MOD_CAPS;
        use textvn_ffi::MOD_CTRL;
        let mut checked = 0usize;
        let rows: &[(u32, u32, KeyKind)] = &[
            (0x41, 'a' as u32, KeyKind::Char('a')),
            (0x41, 0, KeyKind::Other),
            (0x20, ' ' as u32, KeyKind::Char(' ')),
            (0x20, 0, KeyKind::Other),
            (vk::RETURN, 0, KeyKind::Enter),
            (vk::RETURN, '\r' as u32, KeyKind::Enter),
            (vk::TAB, 0, KeyKind::Tab),
            (vk::TAB, '\t' as u32, KeyKind::Tab),
            (vk::BACK, 0, KeyKind::Backspace),
            (vk::BACK, 8, KeyKind::Backspace),
            (vk::ESCAPE, 0, KeyKind::Escape),
            (vk::ESCAPE, 0x1b, KeyKind::Escape),
            (0x70, 0, KeyKind::Other),       // F1 ≠ 'p'
            (0x2E, 0, KeyKind::Other),       // Delete ≠ '.'
            (0x25, 0, KeyKind::Other),       // Left ≠ '%'
            (vk::PACKET, 0, KeyKind::Other), // inject Unicode — Other
            (vk::PROCESSKEY, 0, KeyKind::Other),
        ];
        for (vknum, ch, want) in rows {
            assert_eq!(
                &KeyKind::classify(*vknum, *ch),
                want,
                "classify {vknum:#x}/{ch}"
            );
            assert_eq!(
                KeyKind::classify(*vknum, *ch).needs_session_when_idle(),
                matches!(
                    want,
                    KeyKind::Char(_) | KeyKind::Enter | KeyKind::Tab | KeyKind::CtrlSpace
                ),
                "needs_session {vknum:#x}"
            );
            let native = matches!(want, KeyKind::Enter | KeyKind::Tab);
            assert_eq!(
                KeyKind::classify(*vknum, *ch)
                    .native_boundary_char()
                    .is_some(),
                native,
                "native_boundary {vknum:#x}"
            );
            checked += 1;
        }
        // Ctrl+Space (Text Prediction/Correct trigger): Ctrl thắng, Caps không cản.
        assert_eq!(
            KeyKind::classify_with_mods(0x20, 0, MOD_CTRL),
            KeyKind::CtrlSpace
        );
        assert_eq!(
            KeyKind::classify_with_mods(0x20, ' ' as u32, MOD_CTRL),
            KeyKind::CtrlSpace
        );
        assert_eq!(
            KeyKind::classify_with_mods(0x20, 0, MOD_CTRL | MOD_CAPS),
            KeyKind::CtrlSpace
        );
        assert_eq!(
            KeyKind::classify_with_mods(0x20, ' ' as u32, 0),
            KeyKind::Char(' ')
        );
        checked += 4;
        assert!(checked >= 21, "T6 chỉ có {checked} case");
    }
}
