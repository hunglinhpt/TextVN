// SPDX-License-Identifier: GPL-3.0-or-later
//! Lõi state machine an toàn cho TSF edit session.
//!
//! Phần không-OS (test được trên mọi nền tảng):
//! - [`compose`]: mô hình composition — engine → kế hoạch sửa text cho mỗi phím;
//! - [`classify_tsf_field`]: security gate từ tín hiệu TSF in-proc (InputScope,
//!   read-only, style `ES_PASSWORD`) — không cần UIA xuyên process;
//! - [`ThreadState`]/[`EngineSession`]: engine per-thread + gate S3.
//!
//! Phần COM (`cfg(windows)`) nằm ở `tip.rs`, `key_event.rs`, `edit_session.rs`.

use textvn_appdb::{AppDb, EngineOwner};
use textvn_ffi::{
    ime_instance, ime_instance_free, ime_instance_new, ime_key, ime_key_v1, ime_reset,
    ime_result_v1, IME_ABI_VERSION, IME_OK,
};
use textvn_field_detect::{rules_win::UiaElement, FieldContext, ProbeSnapshot, SecurityState};
use textvn_strategy::{Strategy, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY, IME_FIELD_SEARCH};

pub mod compose;

#[cfg(windows)]
pub mod class;
#[cfg(windows)]
pub mod edit_session;
#[cfg(windows)]
pub mod guids;
pub mod ipc_client;
#[cfg(windows)]
pub mod key_event;
#[cfg(windows)]
pub mod tip;

#[cfg(windows)]
use windows::core::{Interface, GUID, HRESULT};
#[cfg(windows)]
use windows::Win32::System::Com::IClassFactory;

#[cfg(windows)]
const HR_S_OK: HRESULT = HRESULT(0);
#[cfg(windows)]
const HR_S_FALSE: HRESULT = HRESULT(1);
#[cfg(windows)]
const HR_E_POINTER: HRESULT = HRESULT(0x8000_4003_u32 as i32);
#[cfg(windows)]
const HR_CLASSNOTAVAILABLE: HRESULT = HRESULT(0x8004_0111_u32 as i32);

/// `DllGetClassObject` — Windows COM entrypoint khi loader cần ClassFactory của TIP (WIN-010).
///
/// # Safety
/// Caller phải đảm bảo `rclsid`, `riid`, `ppv` trỏ tới bộ nhớ hợp lệ nếu non-null.
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return HR_E_POINTER;
    }
    // SAFETY: ppv non-null
    unsafe { *ppv = std::ptr::null_mut() };
    if unsafe { *rclsid } != guids::CLSID_TEXTVN_TIP {
        return HR_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = class::ClassFactory::new().into();
    unsafe { factory.query(riid, ppv) }
}

/// `DllCanUnloadNow` — Windows COM entrypoint hỏi DLL có thể unload khỏi bộ nhớ không (WIN-010).
///
/// # Safety
/// Hàm đọc biến atomic toàn cục, an toàn khi gọi từ COM runtime trên mọi luồng.
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    // Thread IPC nền sống suốt đời process (không bao giờ join trên STA của app):
    // DLL phải ở lại bộ nhớ để code của thread đó không bị unmap.
    if class::OBJECT_COUNT.load(std::sync::atomic::Ordering::SeqCst) == 0
        && !ipc_client::worker_started()
    {
        HR_S_OK
    } else {
        HR_S_FALSE
    }
}

/// Range UTF-16 theo offset của TSF text context, `start <= end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: i32,
    pub end: i32,
}

impl TextRange {
    pub fn new(start: i32, end: i32) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }

    pub fn len(self) -> i32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// `InputScope` (Win32 `IS_*`) mà TSF adapter quan tâm — giá trị cố định trong SDK
/// (`InputScope.h`); test `cfg(windows)` trong `edit_session.rs` đối chiếu crate `windows`.
pub mod input_scope {
    pub const IS_URL: i32 = 1;
    pub const IS_PASSWORD: i32 = 31;
    pub const IS_SEARCH: i32 = 50;
    pub const IS_NUMERIC_PASSWORD: i32 = 63;
    pub const IS_NUMERIC_PIN: i32 = 64;
    pub const IS_ALPHANUMERIC_PIN: i32 = 65;
    pub const IS_ALPHANUMERIC_PIN_SET: i32 = 66;
}

/// Tín hiệu field đọc được ngay trong process của app, trên thread UI, trong edit
/// session — không UIA, không truy cập process khác.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsfFieldSignals {
    /// `GUID_PROP_INPUTSCOPE` của vùng chọn (rỗng = app không khai báo).
    pub input_scopes: Vec<i32>,
    /// `TS_SD_READONLY` trong `ITfContext::GetStatus`.
    pub read_only: bool,
    /// HWND có focus là Edit/RichEdit có style `ES_PASSWORD`.
    pub password_style: bool,
}

/// Security gate S3 cho TSF. Ô mật khẩu/PIN → `Secure`; context chỉ đọc → `Unknown`
/// (gate đóng, không sửa text); còn lại `NonSecure`. Không có tín hiệu nào = field
/// văn bản bình thường: TSF không activate trong secure desktop/CredUI
/// (`docs/specs/tsf-spike.md` #9), nên mặc định an toàn là gõ được.
pub fn classify_tsf_field(signals: &TsfFieldSignals) -> ProbeSnapshot {
    use input_scope::*;
    let secure = signals.password_style
        || signals.input_scopes.iter().any(|s| {
            matches!(
                *s,
                IS_PASSWORD
                    | IS_NUMERIC_PASSWORD
                    | IS_NUMERIC_PIN
                    | IS_ALPHANUMERIC_PIN
                    | IS_ALPHANUMERIC_PIN_SET
            )
        });
    let field_role = if signals.input_scopes.contains(&IS_URL) {
        IME_FIELD_ADDRESS_BAR
    } else if signals.input_scopes.contains(&IS_SEARCH) {
        IME_FIELD_SEARCH
    } else {
        IME_FIELD_BODY
    };
    let security = if secure {
        SecurityState::Secure
    } else if signals.read_only {
        SecurityState::Unknown
    } else {
        SecurityState::NonSecure
    };
    ProbeSnapshot {
        field_role,
        security,
    }
}

/// Quyết định sau khi TSF trả lỗi RequestEditSession.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditSessionResult {
    Applied,
    /// Key phải forward nguyên vẹn, đồng thời adapter gọi `ime_reset()` trước
    /// key kế tiếp vì engine đã tính outcome trước RequestEditSession.
    RejectedResetEngine,
}

/// State chỉ sở hữu range do adapter đã insert/compose. Selection từ app không
/// tự nhiên trở thành owned range.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextOwnership {
    owned: Option<TextRange>,
}

impl TextOwnership {
    /// Gọi sau khi DoEditSession thành công thay text/preedit do engine tạo.
    pub fn record_engine_text(&mut self, range: TextRange) {
        self.owned = Some(range);
    }

    /// Focus change, external edit, composition end bất thường: không đoán text
    /// app, xóa ownership; caller phải reset engine.
    pub fn invalidate_external_edit(&mut self) -> bool {
        self.owned.take().is_some()
    }

    pub fn owned_range(&self) -> Option<TextRange> {
        self.owned
    }

    /// Cho phép SelectionReplace chỉ trong suffix engine sở hữu và caret đúng
    /// cuối owned range. Selection không rỗng chỉ hợp lệ khi đúng bằng range sẽ
    /// replace; mọi selection khác là text người dùng → fail-open.
    pub fn selection_replace_range(
        &self,
        caret: i32,
        selection: Option<TextRange>,
        delete_count: u16,
    ) -> Option<TextRange> {
        let owned = self.owned?;
        if caret != owned.end {
            return None;
        }
        let delete_count = i32::from(delete_count);
        if delete_count > owned.len() {
            return None;
        }
        let target = TextRange::new(owned.end - delete_count, owned.end)?;
        match selection {
            None => Some(target),
            Some(actual) if actual == target => Some(target),
            Some(_) => None,
        }
    }
}

/// TSF wrapper gọi khi `RequestEditSession` return. `Applied` không đụng engine;
/// `RejectedResetEngine` bắt buộc engine reset + `*eaten = FALSE`.
pub fn after_request_edit_session(success: bool) -> EditSessionResult {
    if success {
        EditSessionResult::Applied
    } else {
        EditSessionResult::RejectedResetEngine
    }
}

/// Owner-thread wrapper cho FFI engine của TSF. Khi edit session bị TSF từ chối,
/// caller dùng `reject_edit_session` thay vì chỉ đổi `*eaten`: outcome đã được
/// engine tính, nên phải xóa buffer để key tiếp theo không replace text chưa hề
/// được app nhận.
pub struct EngineSession {
    instance: *mut ime_instance,
}

/// State thuộc một TSF thread. Focus mới luôn tạo `FieldContext::pending`, vì
/// vậy adapter không thể gõ tiếng Việt vào field mới trước khi snapshot UIA/AX
/// xác minh non-secure. `generation` đơn điệu lọc kết quả worker stale.
pub struct ThreadState {
    pub engine: EngineSession,
    field: FieldContext,
    generation: u64,
    user_enabled: Option<bool>,
}

impl ThreadState {
    pub fn new(app_id: impl Into<String>, caps: u32) -> Result<Self, i32> {
        let generation = 1;
        Ok(Self {
            engine: EngineSession::new()?,
            field: FieldContext::pending(app_id, caps, generation),
            generation,
            user_enabled: None,
        })
    }

    /// Gọi trên focus change trước khi kick worker UIA. Overflow chỉ quay về 1
    /// (zero không có ý nghĩa wire), đồng thời context cũ bị bỏ hoàn toàn.
    pub fn begin_focus(&mut self, app_id: impl Into<String>, caps: u32) -> u64 {
        self.generation = self.generation.checked_add(1).unwrap_or(1);
        self.field = FieldContext::pending(app_id, caps, self.generation);
        self.generation
    }

    pub fn publish_uia(&mut self, generation: u64, element: Option<&UiaElement>) -> bool {
        self.field.apply_windows_uia(generation, element)
    }

    /// Verdict đồng bộ từ tín hiệu TSF in-proc, đọc lại ở MỖI phím trong edit
    /// session: một context TSF (Chrome/Edge) có thể dùng chung cho nhiều field,
    /// nên verdict không được cache qua phím.
    pub fn apply_tsf_probe(&mut self, snapshot: ProbeSnapshot) -> bool {
        let app_id = self.field.app_id.clone();
        let caps = self.field.caps;
        let generation = self.begin_focus(app_id, caps);
        self.field
            .apply_probe(generation, snapshot.field_role, snapshot.security)
    }

    pub fn field_context(&self) -> &FieldContext {
        &self.field
    }

    /// Chỉ trả về strategy sau khi `FieldContext` đã qua security gate. Cầu
    /// nối này là điểm dùng chung của OnKeyDown/OnKeyUp: không có caller nào
    /// được lookup AppDB rồi bỏ qua kết quả UIA của focus hiện hành.
    pub fn resolve_strategy(&self, enabled: bool, appdb: Option<&AppDb>) -> Strategy {
        self.field.resolve_strategy(enabled, appdb)
    }

    /// Trạng thái lưu bởi người dùng (nếu có) luôn ưu tiên preset đóng gói.
    /// Không có trạng thái lưu thì mới dùng `enabled_default` của entry khớp;
    /// không preset cũng mặc định bật để giữ tương thích với bản cũ. Giá trị này
    /// vẫn phải đi qua `resolve_strategy_with_state`: security/capability gate
    /// không được thay thế bởi AppDB.
    pub fn effective_enabled(&self, user_enabled: Option<bool>, appdb: Option<&AppDb>) -> bool {
        user_enabled
            .or_else(|| {
                appdb
                    .and_then(|db| db.preset_for(&self.field.app_id, self.field.field_role))
                    .and_then(|preset| preset.enabled_default)
            })
            .unwrap_or(true)
    }

    /// Phiên bản resolver dành cho adapter có state per-app bền vững. `None`
    /// nghĩa là chưa có lựa chọn người dùng cho app/role hiện hành.
    pub fn resolve_strategy_with_state(
        &self,
        user_enabled: Option<bool>,
        appdb: Option<&AppDb>,
    ) -> Strategy {
        let eff = user_enabled.or(self.user_enabled);
        self.field
            .resolve_strategy(self.effective_enabled(eff, appdb), appdb)
    }

    /// Đảo trạng thái bộ gõ (WIN-015). Khi chuyển sang tắt, dọn sạch buffer engine ngay.
    pub fn toggle_enabled(&mut self) -> bool {
        let current = self.effective_enabled(self.user_enabled, None);
        let next = !current;
        self.user_enabled = Some(next);
        if !next {
            self.engine.reject_edit_session();
        }
        next
    }

    /// Đặt trạng thái bật/tắt do người dùng chỉ định.
    pub fn set_user_enabled(&mut self, enabled: bool) {
        self.user_enabled = Some(enabled);
        if !enabled {
            self.engine.reject_edit_session();
        }
    }

    /// Trạng thái người dùng đã chọn (nếu có).
    pub fn user_enabled(&self) -> Option<bool> {
        self.user_enabled
    }

    /// TSF là owner mặc định khi AppDB không nói khác. Hook/adapter khác phải
    /// gọi resolver chung tương tự; chỉ đúng một bên được đi tiếp tới engine.
    pub fn owns_engine(&self, appdb: Option<&AppDb>) -> bool {
        !matches!(
            appdb
                .and_then(|db| db.preset_for(&self.field.app_id, self.field.field_role))
                .and_then(|preset| preset.engine_owner),
            Some(EngineOwner::Hook)
                | Some(EngineOwner::Imk)
                | Some(EngineOwner::Tap)
                | Some(EngineOwner::Ibus)
                | Some(EngineOwner::Fcitx5)
                | Some(EngineOwner::X11)
        )
    }

    /// Nạp lại cấu hình từ file %APPDATA%\TextVN\config.json khi nhận thông báo IPC.
    pub fn reload_config_from_file(&mut self) -> Result<(), i32> {
        if let Some(path) = config_file_path() {
            if let Ok(bytes) = std::fs::read(path) {
                return self.engine.reload_config(&bytes);
            }
        }
        Err(-1)
    }
}

/// `%APPDATA%\TextVN\config.json` — file tray là nguồn sự thật (P1-4 §1).
pub fn config_file_path() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA").map(|appdata| {
        std::path::PathBuf::from(appdata)
            .join("TextVN")
            .join("config.json")
    })
}

impl EngineSession {
    pub fn new() -> Result<Self, i32> {
        let mut instance = std::ptr::null_mut();
        let result = ime_instance_new(std::ptr::null(), 0, &mut instance);
        if instance.is_null() {
            return Err(result);
        }
        Ok(Self { instance })
    }

    /// Nạp lại cấu hình engine runtime qua C-ABI ime_reload_config.
    pub fn reload_config(&mut self, config_utf8: &[u8]) -> Result<(), i32> {
        let status =
            textvn_ffi::ime_reload_config(self.instance, config_utf8.as_ptr(), config_utf8.len());
        if status == textvn_ffi::IME_OK {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn key_char(&mut self, ch: char) -> Result<ime_result_v1, i32> {
        self.key_event(0, ch as u32, 0)
    }

    /// Phím đã phân loại bởi adapter (`compose::KeyKind`): `vk` + ký tự in được.
    pub fn key_event_raw(&mut self, vk: u32, ch: u32, mods: u32) -> Result<ime_result_v1, i32> {
        self.key_event(vk, ch, mods)
    }

    /// Xóa từ đang gõ (focus change, composition bị app kết thúc, chord…).
    pub fn reset(&mut self) {
        let _ = ime_reset(self.instance);
    }

    /// OnKeyDown gọi hàm này sau khi đã lấy `ToUnicodeEx`. `None` nghĩa là key
    /// system/chord phải đi thẳng vào app, engine không được nhìn thấy nó.
    pub fn key_or_bypass(
        &mut self,
        vk: u32,
        ch: u32,
        mods: u32,
    ) -> Result<Option<ime_result_v1>, i32> {
        if should_bypass_engine(vk, mods) {
            return Ok(None);
        }
        self.key_event(vk, ch, mods).map(Some)
    }

    fn key_event(&mut self, vk: u32, ch: u32, mods: u32) -> Result<ime_result_v1, i32> {
        let key = ime_key_v1 {
            abi_version: IME_ABI_VERSION,
            vk,
            ch,
            mods,
            key_down: 1,
            is_repeat: 0,
            is_injected: 0,
            _reserved: 0,
        };
        let mut result = ime_result_v1 {
            abi_version: 0,
            action: 0,
            delete_count: 0,
            insert_len: 0,
            preedit_len: 0,
            _reserved: 0,
            flags: 0,
            insert: [0; 64],
            preedit: [0; 64],
        };
        let status = ime_key(self.instance, &key, &mut result);
        if status == IME_OK {
            Ok(result)
        } else {
            Err(status)
        }
    }

    /// Returns the rejection signal only after the FFI engine was reset.
    pub fn reject_edit_session(&mut self) -> EditSessionResult {
        let _ = ime_reset(self.instance);
        EditSessionResult::RejectedResetEngine
    }

    /// Authorize SelectionReplace against adapter-owned text. Khi không chứng
    /// minh được range là owned, key phải forward và engine reset để không giữ
    /// outcome REPLACE mà app không hề nhận.
    pub fn selection_replace_or_reset(
        &mut self,
        ownership: &TextOwnership,
        caret: i32,
        selection: Option<TextRange>,
        delete_count: u16,
    ) -> Option<TextRange> {
        let range = ownership.selection_replace_range(caret, selection, delete_count);
        if range.is_none() {
            let _ = ime_reset(self.instance);
        }
        range
    }
}

/// FFI `IME_MOD_*` values. Shift đơn lẻ còn có thể sinh Unicode, còn Ctrl/Alt/
/// Win hoặc chính modifier thì không được feed engine (B6).
pub fn should_bypass_engine(vk: u32, mods: u32) -> bool {
    const MOD_CTRL: u32 = 0x2;
    const MOD_ALT: u32 = 0x4;
    const MOD_SUPER: u32 = 0x8;
    const VK_SHIFT: u32 = 0x10;
    const VK_CONTROL: u32 = 0x11;
    const VK_MENU: u32 = 0x12;
    const VK_LWIN: u32 = 0x5B;
    const VK_RWIN: u32 = 0x5C;
    mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0
        || matches!(vk, VK_SHIFT | VK_CONTROL | VK_MENU | VK_LWIN | VK_RWIN)
}

impl Drop for EngineSession {
    fn drop(&mut self) {
        ime_instance_free(self.instance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use textvn_strategy::{IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR};

    #[test]
    fn rejected_edit_session_requires_engine_reset() {
        assert_eq!(
            after_request_edit_session(false),
            EditSessionResult::RejectedResetEngine
        );
        assert_eq!(after_request_edit_session(true), EditSessionResult::Applied);
    }

    #[test]
    fn selection_replace_refuses_arbitrary_user_selection() {
        let mut ownership = TextOwnership::default();
        ownership.record_engine_text(TextRange::new(10, 15).unwrap());

        assert_eq!(
            ownership.selection_replace_range(15, None, 2),
            TextRange::new(13, 15)
        );
        assert_eq!(
            ownership.selection_replace_range(15, TextRange::new(13, 15), 2),
            TextRange::new(13, 15)
        );
        // User chọn văn bản khác, dù cùng context: không được SetText lên đó.
        assert_eq!(
            ownership.selection_replace_range(15, TextRange::new(3, 9), 2),
            None
        );
        assert_eq!(ownership.selection_replace_range(14, None, 2), None);
        assert_eq!(ownership.selection_replace_range(15, None, 6), None);
    }

    #[test]
    fn external_edit_clears_ownership_before_next_replace() {
        let mut ownership = TextOwnership::default();
        ownership.record_engine_text(TextRange::new(0, 3).unwrap());
        assert!(ownership.invalidate_external_edit());
        assert!(!ownership.invalidate_external_edit());
        assert_eq!(ownership.selection_replace_range(3, None, 1), None);
    }

    #[test]
    fn rejected_edit_session_resets_real_engine_buffer() {
        let mut session = EngineSession::new().expect("default FFI instance");
        assert_eq!(session.key_char('d').unwrap().action, 0); // first d passes
        assert_eq!(
            session.reject_edit_session(),
            EditSessionResult::RejectedResetEngine
        );
        // Nếu không reset, d thứ hai là `dd` -> đ và action REPLACE (1).
        assert_eq!(session.key_char('d').unwrap().action, 0);
    }

    #[test]
    fn unowned_selection_resets_engine_before_forwarding_key() {
        let mut session = EngineSession::new().expect("default FFI instance");
        let _ = session.key_char('d').unwrap();
        let mut ownership = TextOwnership::default();
        ownership.record_engine_text(TextRange::new(10, 11).unwrap());
        assert_eq!(
            session.selection_replace_or_reset(&ownership, 11, TextRange::new(1, 3), 1),
            None
        );
        // Nếu selection không hợp lệ không reset, d thứ hai sẽ thành dd -> REPLACE.
        assert_eq!(session.key_char('d').unwrap().action, 0);
    }

    #[test]
    fn system_chords_and_modifier_keys_bypass_without_mutating_engine() {
        assert!(should_bypass_engine(0, 0x2)); // Ctrl+D
        assert!(should_bypass_engine(0, 0x4)); // Alt+D
        assert!(should_bypass_engine(0x5B, 0)); // Win
        assert!(should_bypass_engine(0x10, 0)); // Shift down itself
        assert!(!should_bypass_engine(0, 0x1)); // Shift+D remains typing

        let mut session = EngineSession::new().unwrap();
        assert!(session.key_or_bypass(0, 'd' as u32, 0x2).unwrap().is_none());
        // Ctrl+D did not become first half of dd; normal d is still literal.
        assert_eq!(session.key_char('d').unwrap().action, 0);
    }

    #[test]
    fn focus_generation_rejects_old_uia_result_and_stays_fail_safe() {
        let mut state = ThreadState::new("chrome.exe", 1).unwrap();
        let old_generation = state.field_context().generation;
        let current_generation = state.begin_focus("keepassxc.exe", 1);
        assert_ne!(old_generation, current_generation);
        let normal_edit = UiaElement {
            is_password: Some(false),
            control_type: textvn_field_detect::rules_win::ControlType::Edit,
            keyboard_focusable: true,
            control_element: true,
            ..Default::default()
        };
        assert!(!state.publish_uia(old_generation, Some(&normal_edit)));
        assert_eq!(
            state.field_context().security,
            textvn_field_detect::SecurityState::Unknown
        );
        assert!(state.publish_uia(current_generation, Some(&normal_edit)));
        assert_eq!(
            state.field_context().security,
            textvn_field_detect::SecurityState::NonSecure
        );
    }

    #[test]
    fn strategy_lookup_cannot_bypass_focus_security_gate() {
        let db = AppDb::parse(
            r#"{"appdb_version":1,"entries":[{"match":{"exe":"chrome.exe"},"when":{"field_role":"address_bar"},"strategy":"SelectionReplace"}]}"#,
        )
        .unwrap();
        let mut state =
            ThreadState::new("chrome.exe", IME_CAP_SELECTION | IME_CAP_PREEDIT).unwrap();

        // UIA chưa phản hồi: dù AppDB match vẫn fail-safe.
        assert_eq!(
            state.resolve_strategy(true, Some(&db)),
            Strategy::Passthrough
        );

        let address_bar = UiaElement {
            is_password: Some(false),
            control_type: textvn_field_detect::rules_win::ControlType::Edit,
            keyboard_focusable: true,
            control_element: true,
            automation_id: "addressbar".into(),
            ..Default::default()
        };
        let generation = state.field_context().generation;
        assert!(state.publish_uia(generation, Some(&address_bar)));
        assert_eq!(state.field_context().field_role, IME_FIELD_ADDRESS_BAR);
        assert_eq!(
            state.resolve_strategy(true, Some(&db)),
            Strategy::SelectionReplace
        );

        // Focus mới reset state; verdict cũ không được tái sử dụng.
        state.begin_focus("chrome.exe", IME_CAP_SELECTION | IME_CAP_PREEDIT);
        assert_eq!(
            state.resolve_strategy(true, Some(&db)),
            Strategy::Passthrough
        );
    }

    #[test]
    fn tsf_does_not_process_hook_owned_preset() {
        let hook_db = AppDb::parse(
            r#"{"appdb_version":1,"entries":[{"match":{"exe":"pwsh.exe"},"when":{"field_role":"terminal"},"strategy":"ForwardAsCommit","engine_owner":"hook"}]}"#,
        )
        .unwrap();
        let mut state = ThreadState::new("pwsh.exe", IME_CAP_PREEDIT).unwrap();
        let terminal = UiaElement {
            is_password: Some(false),
            class_name: "CASCADIA_HOSTING_WINDOW_CLASS".into(),
            ..Default::default()
        };
        let generation = state.field_context().generation;
        assert!(state.publish_uia(generation, Some(&terminal)));
        assert!(!state.owns_engine(Some(&hook_db)));

        state.begin_focus("notepad.exe", IME_CAP_PREEDIT);
        assert!(
            state.owns_engine(Some(&hook_db)),
            "không preset = TSF default"
        );
    }

    #[test]
    fn preset_default_is_used_only_without_a_user_choice() {
        let db = AppDb::parse(
            r#"{"appdb_version":1,"entries":[{"match":{"exe":"code.exe"},"when":{"field_role":"web"},"strategy":"Preedit","engine_owner":"tsf","enabled_default":false}]}"#,
        )
        .unwrap();
        let mut state = ThreadState::new("code.exe", IME_CAP_PREEDIT).unwrap();
        let web_document = UiaElement {
            is_password: Some(false),
            control_type: textvn_field_detect::rules_win::ControlType::Document,
            class_name: "Chrome_RenderWidgetHostHWND".into(),
            ..Default::default()
        };
        let generation = state.field_context().generation;
        assert!(state.publish_uia(generation, Some(&web_document)));

        // VS Code bắt đầu tắt, nhưng người dùng có thể chủ động bật riêng app.
        assert!(!state.effective_enabled(None, Some(&db)));
        assert_eq!(
            state.resolve_strategy_with_state(None, Some(&db)),
            Strategy::Passthrough
        );
        assert!(state.effective_enabled(Some(true), Some(&db)));
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), Some(&db)),
            Strategy::Preedit
        );
        assert!(!state.effective_enabled(Some(false), Some(&db)));

        // Preset không thể mở khóa focus chưa có security verdict.
        state.begin_focus("code.exe", IME_CAP_PREEDIT);
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), Some(&db)),
            Strategy::Passthrough
        );
    }

    #[test]
    fn runtime_config_reload_updates_engine_method() {
        let mut state = ThreadState::new("notepad.exe", IME_CAP_PREEDIT).unwrap();
        // Mặc định là Telex: gõ 'a' + 's' -> 'á' (ACTION_REPLACE)
        let _ = state.engine.key_char('a').unwrap();
        let res = state.engine.key_char('s').unwrap();
        assert_eq!(res.action, textvn_ffi::ACTION_REPLACE);

        // Nạp config chuyển sang VNI (WIN-016 acceptance test)
        let vni_config = br#"{"config_version":1,"method":"vni"}"#;
        assert!(state.engine.reload_config(vni_config).is_ok());

        // Reset buffer cho từ mới
        state.engine.reject_edit_session();

        // Sau khi đổi sang VNI: gõ 'a' + '1' -> 'á' (VNI dùng 1 cho dấu sắc)
        let _ = state.engine.key_char('a').unwrap();
        let res_vni = state.engine.key_char('1').unwrap();
        assert_eq!(res_vni.action, textvn_ffi::ACTION_REPLACE);

        // Reset buffer cho từ mới
        state.engine.reject_edit_session();

        // Phím 's' trong VNI không còn là dấu sắc -> trả ACTION_PASS (0)
        let _ = state.engine.key_char('a').unwrap();
        let res_s = state.engine.key_char('s').unwrap();
        assert_eq!(res_s.action, textvn_ffi::ACTION_PASS);
    }

    #[test]
    fn hotkey_toggle_switches_enabled_state_and_resets_buffer() {
        // WIN-015: Toggle EN/VN (Ctrl+Shift+Space)
        let mut state = ThreadState::new("notepad.exe", IME_CAP_PREEDIT).unwrap();
        let regular_edit = UiaElement {
            is_password: Some(false),
            control_type: textvn_field_detect::rules_win::ControlType::Edit,
            class_name: "Edit".into(),
            ..Default::default()
        };
        let generation = state.field_context().generation;
        assert!(state.publish_uia(generation, Some(&regular_edit)));

        // Mặc định bật tiếng Việt
        assert_eq!(
            state.resolve_strategy_with_state(None, None),
            Strategy::Preedit
        );

        // Đang gõ dở 'a'
        let _ = state.engine.key_char('a').unwrap();

        // Nhấn hotkey toggle -> tắt tiếng Việt (chuyển sang EN)
        let new_state = state.toggle_enabled();
        assert!(!new_state, "Sau lần toggle 1, phải chuyển sang false (EN)");
        assert_eq!(
            state.resolve_strategy_with_state(None, None),
            Strategy::Passthrough,
            "Khi tắt tiếng Việt, strategy phải là Passthrough"
        );

        // Toggle lại -> bật lại tiếng Việt (VN)
        let new_state2 = state.toggle_enabled();
        assert!(new_state2, "Sau lần toggle 2, phải chuyển sang true (VN)");
        assert_eq!(
            state.resolve_strategy_with_state(None, None),
            Strategy::Preedit,
            "Khi bật lại tiếng Việt, strategy quay lại Preedit"
        );
    }

    #[test]
    fn tsf_signals_close_gate_for_password_and_read_only_fields() {
        use input_scope::*;
        let normal = classify_tsf_field(&TsfFieldSignals::default());
        assert_eq!(normal.security, SecurityState::NonSecure);
        assert_eq!(normal.field_role, IME_FIELD_BODY);

        for scope in [
            IS_PASSWORD,
            IS_NUMERIC_PASSWORD,
            IS_NUMERIC_PIN,
            IS_ALPHANUMERIC_PIN,
            IS_ALPHANUMERIC_PIN_SET,
        ] {
            let s = classify_tsf_field(&TsfFieldSignals {
                input_scopes: vec![0, scope],
                ..Default::default()
            });
            assert_eq!(s.security, SecurityState::Secure, "scope {scope}");
        }
        let style = classify_tsf_field(&TsfFieldSignals {
            password_style: true,
            ..Default::default()
        });
        assert_eq!(style.security, SecurityState::Secure);
        let ro = classify_tsf_field(&TsfFieldSignals {
            read_only: true,
            ..Default::default()
        });
        assert_eq!(ro.security, SecurityState::Unknown);

        let url = classify_tsf_field(&TsfFieldSignals {
            input_scopes: vec![IS_URL],
            ..Default::default()
        });
        assert_eq!(url.field_role, IME_FIELD_ADDRESS_BAR);
        assert_eq!(url.security, SecurityState::NonSecure);
    }

    #[test]
    fn tsf_probe_opens_gate_per_key_and_password_closes_it() {
        let mut state = ThreadState::new("notepad.exe", IME_CAP_PREEDIT).unwrap();
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), None),
            Strategy::Passthrough,
            "chưa probe = fail-safe"
        );
        assert!(state.apply_tsf_probe(classify_tsf_field(&TsfFieldSignals::default())));
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), None),
            Strategy::Preedit
        );
        assert!(state.apply_tsf_probe(classify_tsf_field(&TsfFieldSignals {
            input_scopes: vec![input_scope::IS_PASSWORD],
            ..Default::default()
        })));
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), None),
            Strategy::Passthrough
        );
        assert_eq!(
            state.resolve_strategy_with_state(Some(false), None),
            Strategy::Passthrough
        );
    }

    #[test]
    fn secure_field_strictly_forces_passthrough() {
        // WIN-017 / Rule S3: Ô mật khẩu bắt buộc Passthrough
        let mut state =
            ThreadState::new("keepassxc.exe", IME_CAP_PREEDIT | IME_CAP_SELECTION).unwrap();
        let password_element = UiaElement {
            is_password: Some(true),
            control_type: textvn_field_detect::rules_win::ControlType::Edit,
            class_name: "PasswordBox".into(),
            ..Default::default()
        };
        let generation = state.field_context().generation;
        assert!(state.publish_uia(generation, Some(&password_element)));

        // Security gate S3: Tuyệt đối không can thiệp, bất kể user_enabled = true
        assert_eq!(
            state.resolve_strategy_with_state(Some(true), None),
            Strategy::Passthrough,
            "Ô mật khẩu (IsPassword=true) bắt buộc Passthrough"
        );
    }
}
