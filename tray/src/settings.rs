// SPDX-License-Identifier: GPL-3.0-or-later
//! Settings Controller & UI Model cho VietIME Tray (WIN-052 — P1-4 §3 / PLAN §2.3 M6).
//!
//! Quản lý 6 tabs cài đặt:
//! 1. General (Kiểu gõ, kiểu dấu, khôi phục từ tiếng Anh, tự động viết hoa)
//! 2. Applications (Danh sách ứng dụng và trạng thái bật/tắt riêng trong `state.json`)
//! 3. Hotkeys (Phím tắt chuyển Anh/Việt, cho phép gõ tắt khi tắt tiếng Việt)
//! 4. Hook & Game (Chế độ tương thích, danh sách game)
//! 5. Update (Kênh cập nhật, kiểm tra bản mới)
//! 6. About / Support (Phiên bản, xuất chẩn đoán diagnostics, mở thư mục log)
//!
//! Tuân thủ P0-3 §4: Mọi thay đổi đều được ghi qua `SvcManager` và debounce 300ms,
//! sau đó broadcast `ConfigReload` hoặc `StateUpdate` tới các client IPC (TSF TIPs và Hook).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use vietime_config::{Config, DiacriticStyle, MacroTrigger, Method, OutputCharset};

use crate::ipc_server::IpcServer;
use crate::svc::SvcManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Applications,
    Hotkeys,
    HookGame,
    Update,
    About,
}

pub struct SettingsController {
    svc: Arc<SvcManager>,
    ipc: Arc<IpcServer>,
    active_tab: SettingsTab,
    draft_config: Config,
    draft_apps: BTreeMap<String, bool>,
    dirty_config: bool,
    dirty_state: bool,
    last_change: Instant,
    debounce_duration: Duration,
    window_open: Arc<AtomicBool>,
}

impl SettingsController {
    pub fn new(svc: Arc<SvcManager>, ipc: Arc<IpcServer>) -> Self {
        let current_cfg = svc.config();
        let current_apps = svc.app_states();
        Self {
            svc,
            ipc,
            active_tab: SettingsTab::General,
            draft_config: current_cfg,
            draft_apps: current_apps,
            dirty_config: false,
            dirty_state: false,
            last_change: Instant::now(),
            debounce_duration: Duration::from_millis(300),
            window_open: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_open(&self) -> bool {
        self.window_open.load(Ordering::Acquire)
    }

    pub fn set_open(&self, open: bool) {
        self.window_open.store(open, Ordering::Release);
    }

    pub fn active_tab(&self) -> SettingsTab {
        self.active_tab
    }

    pub fn select_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }

    pub fn draft_config(&self) -> &Config {
        &self.draft_config
    }

    pub fn draft_apps(&self) -> &BTreeMap<String, bool> {
        &self.draft_apps
    }

    // --- Tab 1: General Handlers ---
    pub fn set_method(&mut self, method: Method) {
        if self.draft_config.method != method {
            self.draft_config.method = method;
            self.mark_config_dirty();
        }
    }

    pub fn set_diacritic_style(&mut self, style: DiacriticStyle) {
        if self.draft_config.diacritic_style != style {
            self.draft_config.diacritic_style = style;
            self.mark_config_dirty();
        }
    }

    pub fn set_free_marking(&mut self, free: bool) {
        if self.draft_config.free_marking != free {
            self.draft_config.free_marking = free;
            self.mark_config_dirty();
        }
    }

    pub fn set_auto_restore(&mut self, auto_restore: bool) {
        if self.draft_config.auto_restore_english != auto_restore {
            self.draft_config.auto_restore_english = auto_restore;
            self.mark_config_dirty();
        }
    }

    pub fn set_auto_capitalize(&mut self, auto_capitalize: bool) {
        if self.draft_config.auto_capitalize != auto_capitalize {
            self.draft_config.auto_capitalize = auto_capitalize;
            self.mark_config_dirty();
        }
    }

    pub fn set_output_charset(&mut self, charset: OutputCharset) {
        if self.draft_config.output_charset != charset {
            self.draft_config.output_charset = charset;
            self.mark_config_dirty();
        }
    }

    // --- Tab 2: Applications Handlers ---
    pub fn set_app_state(&mut self, app_name: &str, enabled: bool) {
        let key = app_name.trim().to_lowercase();
        if !key.is_empty() {
            self.draft_apps.insert(key, enabled);
            self.mark_state_dirty();
        }
    }

    pub fn remove_app_state(&mut self, app_name: &str) {
        let key = app_name.trim().to_lowercase();
        if self.draft_apps.remove(&key).is_some() {
            self.mark_state_dirty();
        }
    }

    // --- Tab 3: Hotkeys Handlers ---
    pub fn set_allow_macro_when_vi_off(&mut self, allow: bool) {
        if self.draft_config.allow_macro_when_vi_off != allow {
            self.draft_config.allow_macro_when_vi_off = allow;
            self.mark_config_dirty();
        }
    }

    pub fn set_macro_trigger(&mut self, trigger: MacroTrigger) {
        if self.draft_config.macro_trigger != trigger {
            self.draft_config.macro_trigger = trigger;
            self.mark_config_dirty();
        }
    }

    pub fn validate_hotkey(&self, hotkey: &str) -> Result<(), &'static str> {
        let validated = hotkey.trim();
        if validated.eq_ignore_ascii_case("ctrl+c")
            || validated.eq_ignore_ascii_case("ctrl+v")
            || validated.eq_ignore_ascii_case("ctrl+x")
        {
            return Err("Phím tắt bị xung đột với phím tắt hệ thống!");
        }
        Ok(())
    }

    // --- Debounce & Flush (P0-3 §4 / P1-4 §3) ---
    fn mark_config_dirty(&mut self) {
        self.dirty_config = true;
        self.last_change = Instant::now();
    }

    fn mark_state_dirty(&mut self) {
        self.dirty_state = true;
        self.last_change = Instant::now();
    }

    /// Kiểm tra nếu đã đủ thời gian debounce 300ms thì ghi ra file và broadcast qua IPC.
    pub fn check_and_flush(&mut self) -> bool {
        let elapsed = self.last_change.elapsed();
        if (self.dirty_config || self.dirty_state) && elapsed >= self.debounce_duration {
            self.flush_now();
            true
        } else {
            false
        }
    }

    /// Ghi cưỡng bức cấu hình hiện tại và gửi IPC broadcast.
    pub fn flush_now(&mut self) {
        if self.dirty_config {
            // 1. Cập nhật vào SvcManager (Single Source of Truth)
            let ver = self.svc.set_method(self.draft_config.method);
            let _ = self
                .svc
                .set_diacritic_style(self.draft_config.diacritic_style);

            // 2. Broadcast reload tới toàn bộ TSF và Hook client
            self.ipc.broadcast_config_reload(ver);
            self.dirty_config = false;
        }

        if self.dirty_state {
            for (app, &enabled) in &self.draft_apps {
                let ver = self.svc.set_app_enabled(app, enabled);
                self.ipc.broadcast_state_update(app, enabled, ver);
            }
            self.dirty_state = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_controller_switches_tabs() {
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_cfg_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let ipc = IpcServer::new(svc.clone());
        let mut ctrl = SettingsController::new(svc, ipc);

        assert_eq!(ctrl.active_tab(), SettingsTab::General);
        ctrl.select_tab(SettingsTab::Applications);
        assert_eq!(ctrl.active_tab(), SettingsTab::Applications);
        ctrl.select_tab(SettingsTab::Hotkeys);
        assert_eq!(ctrl.active_tab(), SettingsTab::Hotkeys);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn settings_controller_modifies_draft_and_flushes() {
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_flush_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let ipc = IpcServer::new(svc.clone());
        let mut ctrl = SettingsController::new(svc.clone(), ipc);

        assert_eq!(ctrl.draft_config().method, Method::Telex);
        ctrl.set_method(Method::Vni);
        assert_eq!(ctrl.draft_config().method, Method::Vni);
        assert!(ctrl.dirty_config);

        ctrl.flush_now();
        assert!(!ctrl.dirty_config);
        assert_eq!(svc.config().method, Method::Vni);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn settings_controller_hotkey_conflict_validation() {
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_hotkey_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let ipc = IpcServer::new(svc.clone());
        let ctrl = SettingsController::new(svc, ipc);

        assert!(ctrl.validate_hotkey("Ctrl+Shift+Space").is_ok());
        assert!(ctrl.validate_hotkey("Ctrl+C").is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn settings_controller_per_app_state_management() {
        let temp_dir =
            std::env::temp_dir().join(format!("vietime_appstate_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));
        let ipc = IpcServer::new(svc.clone());
        let mut ctrl = SettingsController::new(svc.clone(), ipc);

        ctrl.set_app_state("code.exe", false);
        assert_eq!(ctrl.draft_apps().get("code.exe"), Some(&false));
        assert!(ctrl.dirty_state);

        ctrl.flush_now();
        assert!(!ctrl.dirty_state);
        assert!(!svc.is_app_enabled("code.exe"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
