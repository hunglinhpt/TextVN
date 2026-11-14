// SPDX-License-Identifier: GPL-3.0-or-later
//! State machine thuần cho `textvn-hook.exe` (WIN-040/041/044).
//!
//! Không gọi Win32 trong crate này: callback native chỉ snapshot event rồi hỏi
//! policy ở đây. Vì vậy mọi đường không đủ bằng chứng (focus pending, secure,
//! injected, chord, owner khác hook) đều trả `Pass`; không thể vô tình nuốt
//! phím khi UIA/IPC đang chậm hoặc tray đã chết.

use std::time::Duration;

use textvn_appdb::{AppDb, EngineOwner};
use textvn_ffi::{
    ime_instance, ime_instance_free, ime_instance_new, ime_key, ime_key_v1, ime_reload_config,
    ime_reset, ime_result_v1, ACTION_PASS, IME_ABI_VERSION, IME_FLAG_ERROR, IME_OK,
};
use textvn_field_detect::{FieldContext, SecurityState};

/// Chế độ từ config. `Auto` chỉ chạy với preset `engine_owner: hook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookMode {
    Off,
    Auto,
    Always,
}

/// Event tối thiểu callback native phải snapshot, không chứa text gõ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyEvent {
    pub key_down: bool,
    pub injected: bool,
    pub system_chord: bool,
    pub vk: u32,
    pub ch: u32,
    pub mods: u32,
}

/// Quyết định callback: `Process` mới được phép gọi engine/inject; mọi giá trị
/// khác map sang `CallNextHookEx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackDecision {
    Pass,
    Process,
    /// Quá 50 callback chậm liên tiếp: caller gửi `CrashReport` rồi vẫn pass.
    SelfDisabled,
}

/// Kết quả thuần sau khi gọi FFI. Adapter Win32 chỉ inject khi nhận
/// `Transform`; mọi lỗi/PASS đều đi `CallNextHookEx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineOutcome {
    /// Phím gốc đi qua cho OS (modifier, hotkey, key engine không nhận).
    Pass,
    /// P8 (vòng 55 — root cause "ô chữ nhật nhỏ/mũi tên xanh" trên TSF-bypass
    /// apps): engine xử lý phím letter nhưng KHÔNG biến đổi (raw == display).
    /// Hook vẫn PHẢI nuốt phím gốc và inject lại qua KEYEVENTF_UNICODE —
    /// forward VK gốc thì Windows VIE IME bắt phím và dựng composition box
    /// kẹt giữa hook-inject và OS-IME (xung đột 2 bộ gõ — RCA chủ repo).
    PassUnicode,
    Transform,
}

/// Owner-thread wrapper cho FFI engine của hook. Không `SendInput`, không COM;
/// giữ ranh giới để callback native không cần hiểu ABI hay tự diễn giải lỗi.
pub struct HookEngine {
    instance: *mut ime_instance,
}

impl HookEngine {
    pub fn new() -> Result<Self, i32> {
        Self::new_with_config("")
    }

    pub fn new_with_config(config_json: &str) -> Result<Self, i32> {
        let mut instance = std::ptr::null_mut();
        let (ptr, len) = if config_json.is_empty() {
            (std::ptr::null(), 0)
        } else {
            (config_json.as_ptr(), config_json.len())
        };
        let status = ime_instance_new(ptr, len, &mut instance);
        if instance.is_null() {
            return Err(status);
        }
        Ok(Self { instance })
    }

    pub fn reload_config(&mut self, config_json: &str) -> Result<(), i32> {
        if self.instance.is_null() {
            return Err(-1);
        }
        let status = ime_reload_config(self.instance, config_json.as_ptr(), config_json.len());
        if status == IME_OK {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Bỏ từ đang gõ (engine không còn sở hữu ký tự nào trước con trỏ). Hook chỉ
    /// thấy phím, không thấy document: mọi lúc con trỏ/text có thể đã đổi ngoài tầm
    /// nhìn (đổi cửa sổ, chord, inject thất bại) phải gọi hàm này, nếu không REPLACE
    /// kế tiếp sẽ gửi Backspace xoá nhầm chữ ở chỗ khác.
    pub fn reset(&mut self) {
        if !self.instance.is_null() {
            let _ = ime_reset(self.instance);
        }
    }

    /// Chỉ forward result transform cho injection layer. FFI error hoặc flag
    /// error đều fail-open, không nuốt key gốc.
    pub fn process(
        &mut self,
        policy: &HookState,
        event: KeyEvent,
        appdb: Option<&AppDb>,
        output: &mut ime_result_v1,
    ) -> EngineOutcome {
        *output = empty_result();
        if policy.decide(event, appdb) != CallbackDecision::Process {
            return EngineOutcome::Pass;
        }
        let key = ime_key_v1 {
            abi_version: IME_ABI_VERSION,
            vk: event.vk,
            ch: event.ch,
            mods: event.mods,
            key_down: 1,
            is_repeat: 0,
            is_injected: 0,
            _reserved: 0,
        };
        let status = ime_key(self.instance, &key, output);
        if status != IME_OK || output.flags & IME_FLAG_ERROR != 0 {
            EngineOutcome::Pass
        } else if output.action == ACTION_PASS {
            // P8 (vòng 55): letter engine không biến đổi — vẫn nuốt + inject
            // UNICODE khi là letter thuần (không Ctrl/Alt/Win) để chặn OS-IME
            // composition box (Windows VIE IME bắt VK gốc).
            if is_plain_letter(&event) {
                EngineOutcome::PassUnicode
            } else {
                EngineOutcome::Pass
            }
        } else {
            EngineOutcome::Transform
        }
    }
}

/// P8 (vòng 55): key_down letter thuần không modifier (Shift đơn chấp nhận —
/// hoa/thường đã nằm trong `ch`; Ctrl/Alt/Win là hotkey phải Pass).
pub fn is_plain_letter(event: &KeyEvent) -> bool {
    const MOD_CTRL: u32 = 0x2;
    const MOD_ALT: u32 = 0x4;
    const MOD_WIN: u32 = 0x8;
    const MOD_SUPER: u32 = 0x20;
    event.key_down
        && event.ch > 0
        && event.vk >= 0x41
        && event.vk <= 0x5A
        && event.mods & (MOD_CTRL | MOD_ALT | MOD_WIN | MOD_SUPER) == 0
}

fn empty_result() -> ime_result_v1 {
    ime_result_v1 {
        abi_version: 0,
        action: ACTION_PASS,
        delete_count: 0,
        insert_len: 0,
        preedit_len: 0,
        _reserved: 0,
        flags: 0,
        insert: [0; 64],
        preedit: [0; 64],
    }
}

impl Drop for HookEngine {
    fn drop(&mut self) {
        ime_instance_free(self.instance);
    }
}

/// Pure state của process hook. `last_pong` là thời điểm tray còn sống theo
/// monotonic clock caller cung cấp; không lấy wall clock để tránh sai khi đổi giờ.
#[derive(Debug, Clone)]
pub struct HookState {
    mode: HookMode,
    field: FieldContext,
    disabled: bool,
    slow_streak: u8,
    last_pong: Duration,
}

impl HookState {
    pub const CALLBACK_BUDGET: Duration = Duration::from_millis(2);
    pub const MAX_SLOW_STREAK: u8 = 50;
    pub const ORPHAN_TIMEOUT: Duration = Duration::from_secs(30);

    /// Focus mới tạo context pending nên hook không process trước UIA verdict.
    pub fn new(app_id: impl Into<String>, caps: u32, mode: HookMode, now: Duration) -> Self {
        Self {
            mode,
            field: FieldContext::pending(app_id, caps, 1),
            disabled: false,
            slow_streak: 0,
            last_pong: now,
        }
    }

    pub fn mode(&self) -> HookMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: HookMode) {
        self.mode = mode;
    }

    pub fn field_context(&self) -> &FieldContext {
        &self.field
    }

    /// Focus change phải được caller gửi từ focus worker; pending giữ fail-safe.
    pub fn begin_focus(&mut self, app_id: impl Into<String>, caps: u32) -> u64 {
        let generation = self.field.generation.checked_add(1).unwrap_or(1);
        self.field = FieldContext::pending(app_id, caps, generation);
        generation
    }

    pub fn publish_probe(
        &mut self,
        generation: u64,
        field_role: u32,
        security: SecurityState,
    ) -> bool {
        self.field.apply_probe(generation, field_role, security)
    }

    /// Chỉ `Pong` hợp lệ mới kéo dài lease của hook. EOF pipe không khiến hook
    /// sống vô hạn bằng config cũ: caller gọi `is_orphaned` ở timer 5s.
    pub fn on_pong(&mut self, now: Duration) {
        self.last_pong = now;
    }

    pub fn is_orphaned(&self, now: Duration) -> bool {
        now.saturating_sub(self.last_pong) >= Self::ORPHAN_TIMEOUT
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Gate trước FFI: low-level hook callback không UIA/IPC/IO, chỉ đọc state
    /// đã publish. `Always` vẫn không vượt secure/pending hoặc injected/chord.
    pub fn decide(&self, event: KeyEvent, appdb: Option<&AppDb>) -> CallbackDecision {
        if self.disabled
            || !event.key_down
            || event.injected
            || event.system_chord
            || self.field.security != SecurityState::NonSecure
        {
            return CallbackDecision::Pass;
        }
        match self.mode {
            HookMode::Off => CallbackDecision::Pass,
            HookMode::Always => CallbackDecision::Process,
            HookMode::Auto if is_hook_owned(&self.field, appdb) => CallbackDecision::Process,
            HookMode::Auto => CallbackDecision::Pass,
        }
    }

    /// Gọi sau mỗi callback đã xử lý. Quá budget tăng streak; callback trong
    /// budget reset streak. Khi chạm ngưỡng, hook tự tắt và caller báo tray.
    pub fn record_callback_duration(&mut self, elapsed: Duration) -> CallbackDecision {
        if self.disabled {
            return CallbackDecision::SelfDisabled;
        }
        if elapsed <= Self::CALLBACK_BUDGET {
            self.slow_streak = 0;
            return CallbackDecision::Process;
        }
        self.slow_streak = self.slow_streak.saturating_add(1);
        if self.slow_streak >= Self::MAX_SLOW_STREAK {
            self.disabled = true;
            CallbackDecision::SelfDisabled
        } else {
            // Chậm: native callback vẫn forward key hiện hành, tuyệt đối retry.
            CallbackDecision::Pass
        }
    }
}

fn is_hook_owned(field: &FieldContext, appdb: Option<&AppDb>) -> bool {
    matches!(
        appdb
            .and_then(|db| db.preset_for(&field.app_id, field.field_role))
            .and_then(|preset| preset.engine_owner),
        Some(EngineOwner::Hook)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use textvn_strategy::{IME_CAP_FIELD_DETECT, IME_CAP_PREEDIT, IME_FIELD_BODY};

    fn hook_db() -> AppDb {
        AppDb::parse(
            r#"{"appdb_version":1,"entries":[{"match":{"exe":"pwsh.exe"},"when":{"field_role":"body"},"engine_owner":"hook"},{"match":{"exe":"notepad.exe"},"engine_owner":"tsf"}]}"#,
        )
        .unwrap()
    }

    fn ready(state: &mut HookState) {
        let generation = state.field_context().generation;
        assert!(state.publish_probe(generation, IME_FIELD_BODY, SecurityState::NonSecure));
    }

    #[test]
    fn reset_drops_word_so_next_key_cannot_replace() {
        let mut engine = HookEngine::new().unwrap();
        let mut state = HookState::new("notepad.exe", 0, HookMode::Always, Duration::ZERO);
        ready(&mut state);
        let mut out = empty_result();
        let key = |c: char| KeyEvent {
            key_down: true,
            ch: c as u32,
            ..Default::default()
        };
        assert!(matches!(
            engine.process(&state, key('d'), None, &mut out),
            EngineOutcome::Pass
        ));
        engine.reset();
        // Không reset thì `d` thứ hai là `dd` → đ (Transform, xoá 1 ký tự).
        assert!(matches!(
            engine.process(&state, key('d'), None, &mut out),
            EngineOutcome::Pass
        ));
    }

    #[test]
    fn auto_only_processes_hook_owned_nonsecure_focus() {
        let db = hook_db();
        let mut state = HookState::new(
            "pwsh.exe",
            IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT,
            HookMode::Auto,
            Duration::ZERO,
        );
        let key = KeyEvent {
            key_down: true,
            ..Default::default()
        };
        assert_eq!(state.decide(key, Some(&db)), CallbackDecision::Pass);
        ready(&mut state);
        assert_eq!(state.decide(key, Some(&db)), CallbackDecision::Process);

        state.begin_focus("notepad.exe", IME_CAP_PREEDIT | IME_CAP_FIELD_DETECT);
        ready(&mut state);
        assert_eq!(state.decide(key, Some(&db)), CallbackDecision::Pass);
    }

    #[test]
    fn injected_chord_secure_and_key_up_always_pass() {
        let db = hook_db();
        let mut state = HookState::new(
            "pwsh.exe",
            IME_CAP_PREEDIT,
            HookMode::Always,
            Duration::ZERO,
        );
        ready(&mut state);
        for event in [
            KeyEvent {
                key_down: false,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                injected: true,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                system_chord: true,
                ..Default::default()
            },
        ] {
            assert_eq!(state.decide(event, Some(&db)), CallbackDecision::Pass);
        }
        let generation = state.field_context().generation;
        assert!(state.publish_probe(generation, IME_FIELD_BODY, SecurityState::Secure));
        assert_eq!(
            state.decide(
                KeyEvent {
                    key_down: true,
                    ..Default::default()
                },
                Some(&db)
            ),
            CallbackDecision::Pass
        );
    }

    #[test]
    fn slow_callback_self_disables_and_heartbeat_expires() {
        let mut state = HookState::new(
            "pwsh.exe",
            IME_CAP_PREEDIT,
            HookMode::Always,
            Duration::ZERO,
        );
        for _ in 0..HookState::MAX_SLOW_STREAK - 1 {
            assert_eq!(
                state.record_callback_duration(Duration::from_millis(3)),
                CallbackDecision::Pass
            );
        }
        assert_eq!(
            state.record_callback_duration(Duration::from_millis(3)),
            CallbackDecision::SelfDisabled
        );
        assert!(state.is_disabled());

        assert!(!state.is_orphaned(Duration::from_secs(29)));
        assert!(state.is_orphaned(Duration::from_secs(30)));
        state.on_pong(Duration::from_secs(30));
        assert!(!state.is_orphaned(Duration::from_secs(59)));
    }

    #[test]
    fn a_fast_callback_resets_slow_streak() {
        let mut state = HookState::new(
            "pwsh.exe",
            IME_CAP_PREEDIT,
            HookMode::Always,
            Duration::ZERO,
        );
        assert_eq!(
            state.record_callback_duration(Duration::from_millis(3)),
            CallbackDecision::Pass
        );
        assert_eq!(
            state.record_callback_duration(Duration::from_millis(1)),
            CallbackDecision::Process
        );
        for _ in 0..HookState::MAX_SLOW_STREAK - 1 {
            assert_eq!(
                state.record_callback_duration(Duration::from_millis(3)),
                CallbackDecision::Pass
            );
        }
        assert!(!state.is_disabled(), "fast callback phải reset streak");
    }

    #[test]
    fn engine_never_transforms_when_policy_says_pass() {
        let db = hook_db();
        let mut state = HookState::new(
            "notepad.exe",
            IME_CAP_PREEDIT,
            HookMode::Auto,
            Duration::ZERO,
        );
        ready(&mut state);
        let mut engine = HookEngine::new().unwrap();
        let mut result = empty_result();
        let event = KeyEvent {
            key_down: true,
            vk: 'd' as u32,
            ch: 'd' as u32,
            ..Default::default()
        };
        assert!(matches!(
            engine.process(&state, event, Some(&db), &mut result),
            EngineOutcome::Pass
        ));

        state.set_mode(HookMode::Always);
        // d đầu tiên PASS từ engine; d thứ hai thành transform (`dd` → đ).
        assert!(matches!(
            engine.process(&state, event, Some(&db), &mut result),
            EngineOutcome::Pass
        ));
        assert!(matches!(
            engine.process(&state, event, Some(&db), &mut result),
            EngineOutcome::Transform
        ));
        assert!(matches!(
            engine.process(
                &state,
                KeyEvent {
                    injected: true,
                    ..event
                },
                Some(&db),
                &mut result
            ),
            EngineOutcome::Pass
        ));
    }

    // ====================================================================
    // Ma trận chống lệch máy trạng thái (E47 — yêu cầu audit "1.000 case")
    // ====================================================================

    /// Bảng chân lý ĐẦY ĐỦ của `HookState::decide`: mode × event × security ×
    /// appdb. Phím injected/chord/secure/pending phải PASS tuyệt đối (fail-open)
    /// — nguồn gốc "phím bị nuốt/không nuốt" của hook, ghim toàn ma trận.
    #[test]
    fn decide_full_truth_table() {
        let db = hook_db(); // pwsh.exe → hook; notepad.exe → tsf
        let modes = [HookMode::Off, HookMode::Auto, HookMode::Always];
        let mut checked = 0usize;
        let events: Vec<(&str, KeyEvent)> = vec![
            (
                "letter",
                KeyEvent {
                    key_down: true,
                    vk: 'd' as u32,
                    ch: 'd' as u32,
                    ..Default::default()
                },
            ),
            (
                "key-up",
                KeyEvent {
                    key_down: false,
                    vk: 'd' as u32,
                    ch: 'd' as u32,
                    ..Default::default()
                },
            ),
            (
                "injected",
                KeyEvent {
                    key_down: true,
                    vk: 'd' as u32,
                    ch: 'd' as u32,
                    injected: true,
                    ..Default::default()
                },
            ),
            (
                "chord",
                KeyEvent {
                    key_down: true,
                    vk: 'd' as u32,
                    ch: 'd' as u32,
                    system_chord: true,
                    ..Default::default()
                },
            ),
            (
                "modifier",
                KeyEvent {
                    key_down: true,
                    vk: 0xA2,
                    ..Default::default()
                },
            ),
        ];
        for mode in modes {
            for (label, event) in events.iter().copied() {
                // 3 trạng thái bảo mật: pending (mới focus), non-secure, secure.
                let pending = HookState::new("pwsh.exe", 0, mode, Duration::ZERO);
                assert_eq!(
                    pending.decide(event, Some(&db)),
                    CallbackDecision::Pass,
                    "{mode:?}/{label}: focus pending phải PASS (fail-open)"
                );
                let mut nonsec = HookState::new("pwsh.exe", 0, mode, Duration::ZERO);
                ready(&mut nonsec);
                let mut secure = HookState::new("pwsh.exe", 0, mode, Duration::ZERO);
                ready(&mut secure);
                assert!(secure.publish_probe(
                    secure.field_context().generation,
                    IME_FIELD_BODY,
                    SecurityState::Secure
                ));
                let want_nonsec = match (mode, label) {
                    (HookMode::Off, _) | (_, "injected") | (_, "key-up") | (_, "chord") => {
                        CallbackDecision::Pass
                    }
                    (_, "letter") | (_, "modifier") => CallbackDecision::Process,
                    _ => CallbackDecision::Pass,
                };
                // Auto: appdb phải xếp hook sở hữu mới Process — pwsh.exe có preset.
                assert_eq!(
                    nonsec.decide(event, Some(&db)),
                    want_nonsec,
                    "{mode:?}/{label}: non-secure"
                );
                assert_eq!(
                    secure.decide(event, Some(&db)),
                    CallbackDecision::Pass,
                    "{mode:?}/{label}: secure phải PASS (S3)"
                );
                checked += 3;
            }
        }
        // Auto KHÔNG Process khi appdb không xếp hook sở hữu (notepad → tsf).
        let mut notepad = HookState::new("notepad.exe", 0, HookMode::Auto, Duration::ZERO);
        ready(&mut notepad);
        assert_eq!(
            notepad.decide(
                KeyEvent {
                    key_down: true,
                    vk: 'd' as u32,
                    ch: 'd' as u32,
                    ..Default::default()
                },
                Some(&db)
            ),
            CallbackDecision::Pass
        );
        checked += 1;
        assert_eq!(checked, 3 * 5 * 3 + 1, "truth table thiếu case");
    }

    /// Ma trận `is_plain_letter` — nguồn gốc PassUnicode (nuốt + inject lại
    /// KEYEVENTF_UNICODE để chặn OS-IME composition box): chỉ letter thuần
    /// key-down không Ctrl/Alt/Win mới đủ điều kiện.
    #[test]
    fn is_plain_letter_matrix() {
        let mut checked = 0usize;
        // VK trong khoảng A..Z với ch>0: letter thuần.
        for vk in [0x41u32, 0x5Au32, 0x43u32] {
            assert!(is_plain_letter(&KeyEvent {
                key_down: true,
                vk,
                ch: vk,
                ..Default::default()
            }));
            checked += 1;
            // Ctrl/Alt/Win/Super là hotkey — KHÔNG plain letter.
            for mods in [0x2u32, 0x4, 0x8, 0x20] {
                assert!(!is_plain_letter(&KeyEvent {
                    key_down: true,
                    vk,
                    ch: vk,
                    mods,
                    ..Default::default()
                }));
                checked += 1;
            }
            // Shift đơn chấp nhận (hoa/thường nằm trong ch).
            assert!(is_plain_letter(&KeyEvent {
                key_down: true,
                vk,
                ch: vk,
                mods: 0x1,
                ..Default::default()
            }));
            checked += 1;
        }
        // key-up / ch=0 / VK ngoài A..Z (số, OEM, F-key) không bao giờ là letter.
        for event in [
            KeyEvent {
                key_down: false,
                vk: 0x41,
                ch: 0x41,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                vk: 0x41,
                ch: 0,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                vk: 0x31, // '1'
                ch: 0x31,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                vk: 0xBA, // OEM
                ch: 0x3B,
                ..Default::default()
            },
            KeyEvent {
                key_down: true,
                vk: 0x70, // F1
                ch: 0,
                ..Default::default()
            },
        ] {
            assert!(!is_plain_letter(&event));
            checked += 1;
        }
        assert_eq!(checked, 3 * 6 + 5, "is_plain_letter matrix thiếu case");
    }

    /// `record_callback_duration` ma trận: budget ≤ 2 ms reset streak; vượt
    /// budget đếm dần; chạm ngưỡng tự tắt (không nuốt phím nữa — fail-open).
    #[test]
    fn callback_budget_matrix() {
        let mut checked = 0usize;
        for (elapsed, expect_process) in [
            (Duration::ZERO, true),
            (Duration::from_millis(2), true),
            (Duration::from_millis(3), false),
            (Duration::from_millis(100), false),
        ] {
            let mut state = HookState::new("pwsh.exe", 0, HookMode::Always, Duration::ZERO);
            let d = state.record_callback_duration(elapsed);
            assert_eq!(
                d == CallbackDecision::Process,
                expect_process,
                "elapsed {elapsed:?}"
            );
            checked += 1;
        }
        // Chuỗi slow: PASS liên tục đến MAX_SLOW_STREAK thì SelfDisabled.
        let mut state = HookState::new("pwsh.exe", 0, HookMode::Always, Duration::ZERO);
        for i in 1..=HookState::MAX_SLOW_STREAK {
            let d = state.record_callback_duration(Duration::from_millis(5));
            if i < HookState::MAX_SLOW_STREAK {
                assert_eq!(d, CallbackDecision::Pass);
            } else {
                assert_eq!(d, CallbackDecision::SelfDisabled);
                assert!(state.is_disabled());
            }
            checked += 1;
        }
        // Đã disabled: mọi callback sau đều SelfDisabled.
        assert_eq!(
            state.record_callback_duration(Duration::ZERO),
            CallbackDecision::SelfDisabled
        );
        checked += 1;
        assert_eq!(checked, 4 + HookState::MAX_SLOW_STREAK as usize + 1);
    }
}
