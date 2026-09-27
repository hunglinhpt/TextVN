// SPDX-License-Identifier: GPL-3.0-or-later
//! Service layer quản lý State & Config in-process cho TextVN Tray (P1-4 §1/§2).
//!
//! Một nguồn sự thật duy nhất (Single Source of Truth) cho file `%APPDATA%\TextVN\{config.json, state.json}`.
//! Các thao tác cập nhật cấu hình và trạng thái per-app đi qua đây, được ghi nguyên tử (atomic write qua file tạm)
//! và tăng version đơn điệu để broadcast tới các TSF TIP và Hook qua IPC.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use textvn_config::{Config, DiacriticStyle, Method};

/// Dữ liệu trạng thái bật/tắt gõ per-app lưu trong `state.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct StateData {
    pub global_enabled: bool,
    pub apps: BTreeMap<String, bool>,
}

pub struct SvcManager {
    config_dir: PathBuf,
    config: RwLock<Config>,
    state: RwLock<StateData>,
    config_version: AtomicU64,
    state_version: AtomicU64,
}

impl SvcManager {
    /// Khởi tạo service manager với đường dẫn thư mục cấu hình (mặc định `%APPDATA%\TextVN`).
    pub fn new(config_dir: Option<PathBuf>) -> Arc<Self> {
        let dir = config_dir.unwrap_or_else(default_config_dir);
        let _ = fs::create_dir_all(&dir);

        let config_file = dir.join("config.json");
        let initial_config = if config_file.exists() {
            fs::read_to_string(&config_file)
                .ok()
                .and_then(|s| textvn_config::parse_config(&s).ok())
                .unwrap_or_default()
        } else {
            let default_cfg = Config::default();
            if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
                let _ = atomic_write_file(&config_file, json.as_bytes());
            }
            default_cfg
        };

        let state_file = dir.join("state.json");
        let initial_state = if state_file.exists() {
            fs::read_to_string(&state_file)
                .ok()
                .and_then(|s| serde_json::from_str::<StateData>(&s).ok())
                .unwrap_or_else(|| StateData {
                    global_enabled: true,
                    apps: BTreeMap::new(),
                })
        } else {
            let default_state = StateData {
                global_enabled: true,
                apps: BTreeMap::new(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&default_state) {
                let _ = atomic_write_file(&state_file, json.as_bytes());
            }
            default_state
        };

        Arc::new(Self {
            config_dir: dir,
            config: RwLock::new(initial_config),
            state: RwLock::new(initial_state),
            config_version: AtomicU64::new(1),
            state_version: AtomicU64::new(1),
        })
    }

    pub fn config(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    pub fn config_version(&self) -> u64 {
        self.config_version.load(Ordering::Acquire)
    }

    pub fn state_version(&self) -> u64 {
        self.state_version.load(Ordering::Acquire)
    }

    pub fn is_global_enabled(&self) -> bool {
        self.state.read().unwrap().global_enabled
    }

    pub fn is_app_enabled(&self, app_id: &str) -> bool {
        let st = self.state.read().unwrap();
        if let Some(&enabled) = st.apps.get(app_id) {
            enabled
        } else {
            st.global_enabled
        }
    }

    pub fn app_states(&self) -> BTreeMap<String, bool> {
        self.state.read().unwrap().apps.clone()
    }

    /// Đảo trạng thái bật/tắt toàn cục tiếng Việt (WIN-015 / P1-4 §1).
    pub fn toggle_global_enabled(&self) -> (bool, u64) {
        let mut st = self.state.write().unwrap();
        st.global_enabled = !st.global_enabled;
        let next_ver = self.state_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_state(&st);
        (st.global_enabled, next_ver)
    }

    /// Đặt trạng thái bật/tắt riêng cho từng app.
    pub fn set_app_enabled(&self, app_id: &str, enabled: bool) -> u64 {
        let mut st = self.state.write().unwrap();
        st.apps.insert(app_id.to_lowercase(), enabled);
        let next_ver = self.state_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_state(&st);
        next_ver
    }

    /// Đặt trạng thái bật/tắt toàn cục tiếng Việt có giá trị chỉ định.
    pub fn set_global_enabled(&self, enabled: bool) -> (bool, u64) {
        let mut st = self.state.write().unwrap();
        st.global_enabled = enabled;
        let next_ver = self.state_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_state(&st);
        (st.global_enabled, next_ver)
    }

    /// Đổi kiểu gõ (Telex, VNI, VIQR, Simple Telex) (P1-4 §1).
    pub fn set_method(&self, method: Method) -> u64 {
        let mut cfg = self.config.write().unwrap();
        cfg.method = method;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_config(&cfg);
        next_ver
    }

    /// Đổi kiểu bỏ dấu (Chuẩn mới / Cổ điển).
    pub fn set_diacritic_style(&self, style: DiacriticStyle) -> u64 {
        let mut cfg = self.config.write().unwrap();
        cfg.diacritic_style = style;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_config(&cfg);
        next_ver
    }

    /// Đổi bảng mã xuất (Unicode, Unicode tổ hợp, VNI Windows, TCVN3, VIQR).
    pub fn set_output_charset(&self, charset: textvn_config::OutputCharset) -> u64 {
        let mut cfg = self.config.write().unwrap();
        cfg.output_charset = charset;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_config(&cfg);
        next_ver
    }

    /// Bật/tắt tự động khôi phục từ tiếng Anh khi gõ sai.
    pub fn set_auto_restore_english(&self, enable: bool) -> u64 {
        let mut cfg = self.config.write().unwrap();
        cfg.auto_restore_english = enable;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_config(&cfg);
        next_ver
    }

    /// Bật/tắt đặt dấu tự do (free marking).
    pub fn set_free_marking(&self, enable: bool) -> u64 {
        let mut cfg = self.config.write().unwrap();
        cfg.free_marking = enable;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        self.persist_config(&cfg);
        next_ver
    }

    fn persist_config(&self, cfg: &Config) {
        let path = self.config_dir.join("config.json");
        if let Ok(json) = serde_json::to_string_pretty(cfg) {
            let _ = atomic_write_file(&path, json.as_bytes());
        }
    }

    fn persist_state(&self, state: &StateData) {
        let path = self.config_dir.join("state.json");
        if let Ok(json) = serde_json::to_string_pretty(state) {
            let _ = atomic_write_file(&path, json.as_bytes());
        }
    }
}

/// Trả về đường dẫn %APPDATA%\TextVN mặc định (hoặc legacy %APPDATA%\TextVN nếu đã tồn tại).
pub fn default_config_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let primary = PathBuf::from(&appdata).join("TextVN");
        let legacy = PathBuf::from(&appdata).join("TextVN");
        if !primary.exists() && legacy.exists() {
            legacy
        } else {
            primary
        }
    } else {
        PathBuf::from(".textvn")
    }
}

/// Ghi file an toàn: ghi ra file tạm .tmp rồi rename để tránh hỏng dữ liệu khi mất nguồn.
fn atomic_write_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(content)?;
        file.sync_all()?;
    }
    fs::rename(&tmp_path, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svc_manager_initializes_and_toggles_state() {
        let temp_dir = std::env::temp_dir().join(format!("textvn_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));

        assert!(svc.is_global_enabled());
        let (new_val, ver1) = svc.toggle_global_enabled();
        assert!(!new_val);
        assert_eq!(ver1, 2);

        let ver2 = svc.set_app_enabled("chrome.exe", true);
        assert_eq!(ver2, 3);
        assert!(svc.is_app_enabled("chrome.exe"));
        assert!(!svc.is_app_enabled("notepad.exe")); // theo global_enabled = false

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn svc_manager_switches_method_and_increments_version() {
        let temp_dir =
            std::env::temp_dir().join(format!("textvn_test_method_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));

        assert_eq!(svc.config().method, Method::Telex);
        let next_ver = svc.set_method(Method::Vni);
        assert_eq!(next_ver, 2);
        assert_eq!(svc.config().method, Method::Vni);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
