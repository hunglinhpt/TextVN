// SPDX-License-Identifier: GPL-3.0-or-later
//! Service layer quản lý State & Config in-process cho TextVN Tray (P1-4 §1/§2).
//!
//! Một nguồn sự thật duy nhất (Single Source of Truth) cho file `%APPDATA%\TextVN\{config.json, state.json}`.
//! Các thao tác cập nhật cấu hình và trạng thái per-app đi qua đây, được ghi nguyên tử (atomic write qua file tạm)
//! và tăng version đơn điệu để broadcast tới các TSF TIP và Hook qua IPC.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use textvn_config::{
    Config, DiacriticStyle, DocError, DocKind, MacroEntry, MacroTrigger, Method, SettingsDoc,
};

/// Dữ liệu trạng thái bật/tắt gõ per-app lưu trong `state.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct StateData {
    pub global_enabled: bool,
    pub apps: BTreeMap<String, bool>,
    /// Phiên bản app ghi state lần trước — đổi phiên bản = kích hoạt migration
    /// dọn dữ liệu cũ (báo cáo chủ repo 0.2.9: "cập nhật app mới vẫn bị app cũ
    /// ảnh hưởng"). `#[serde(default)]` để state.json cũ không có nhãn vẫn đọc được.
    #[serde(default)]
    pub last_version: String,
}

pub struct SvcManager {
    config_dir: PathBuf,
    config: RwLock<Config>,
    state: RwLock<StateData>,
    config_version: AtomicU64,
    state_version: AtomicU64,
    /// Đổi phiên bản lúc khởi động này → đăng ký TSF được unregister+register lại.
    version_migrated: std::sync::atomic::AtomicBool,
    /// Tray chưa từng chạy cho tài khoản này (chưa có `state.json`).
    first_run: bool,
}

impl SvcManager {
    /// Khởi tạo service manager với đường dẫn thư mục cấu hình (mặc định `%APPDATA%\TextVN`).
    pub fn new(config_dir: Option<PathBuf>) -> Arc<Self> {
        let dir = config_dir.unwrap_or_else(default_config_dir);
        let _ = fs::create_dir_all(&dir);

        let config_file = dir.join("config.json");
        // Đọc qua SettingsDoc: file hỏng không bị ghi đè mất mà được sao lưu `.bak` ở lần
        // lưu đầu; khoá lạ (hotkeys, …) được giữ nguyên khi tray ghi lại.
        let mut doc = SettingsDoc::load(&config_file, DocKind::Config);
        let initial_config = textvn_config::parse_config(&doc.to_json()).unwrap_or_default();
        if !config_file.exists() || doc.was_corrupt() {
            doc.merge_config(&initial_config);
            let _ = doc.save(&config_file);
        }

        let state_file = dir.join("state.json");
        // R2-25: lần đầu tray chạy cho TÀI KHOẢN này (bản cài cho mọi người dùng tự khởi
        // động qua HKLM Run ở tài khoản khác) → caller đăng ký TSF per-user một lần.
        let first_run = !state_file.exists();
        let mut initial_state = if state_file.exists() {
            fs::read_to_string(&state_file)
                .ok()
                .and_then(|s| serde_json::from_str::<StateData>(&s).ok())
                .unwrap_or_else(|| StateData {
                    global_enabled: true,
                    apps: BTreeMap::new(),
                    last_version: String::new(),
                })
        } else {
            let default_state = StateData {
                global_enabled: true,
                apps: BTreeMap::new(),
                last_version: String::new(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&default_state) {
                let _ = atomic_write_file(&state_file, json.as_bytes());
            }
            default_state
        };

        // ---- Nâng cấp phiên bản ----
        // CR-20/R2-33: bản 0.2.10–0.2.27 reset MỌI tuỳ chọn gõ (kiểu gõ, bảng mã…) và
        // override theo app ở MỖI lần cập nhật — người dùng VNI phải chọn lại sau mỗi bản.
        // Lệch schema đã được `parse_config` (giá trị thiếu/lạ → mặc định) và SettingsDoc
        // (giữ khoá lạ, sao lưu file hỏng) xử lý, nên nâng cấp GIỮ NGUYÊN lựa chọn của
        // người dùng; chỉ nhãn phiên bản đổi. Đăng ký TSF chỉ ghi lại nếu đã lệch.
        let current_version = env!("CARGO_PKG_VERSION");
        let version_migrated =
            !initial_state.last_version.is_empty() && initial_state.last_version != current_version;
        initial_state.last_version = current_version.to_string();
        {
            // Ghi lại state (nhãn phiên bản mới + state reset) một cách atomic.
            if let Ok(json) = serde_json::to_string_pretty(&initial_state) {
                let _ = atomic_write_file(&state_file, json.as_bytes());
            }
        }

        Arc::new(Self {
            config_dir: dir,
            config: RwLock::new(initial_config),
            state: RwLock::new(initial_state),
            // R2-31: khởi đầu duy nhất cho MỖI lần tray chạy (TIP chỉ so khác/bằng) —
            // bắt đầu lại từ 1 thì app chạy lâu (TIP giữ version cũ = 1..n) sau khi tray
            // khởi động lại có thể thấy "trùng version" và bỏ lỡ thay đổi cấu hình.
            config_version: AtomicU64::new(session_version_seed()),
            state_version: AtomicU64::new(session_version_seed()),
            version_migrated: std::sync::atomic::AtomicBool::new(version_migrated),
            first_run,
        })
    }

    /// Lần đầu tray chạy cho tài khoản này (R2-25).
    pub fn first_run(&self) -> bool {
        self.first_run
    }

    /// Đổi phiên bản lúc khởi động này (chỉ để log/chẩn đoán — tuỳ chọn được giữ).
    pub fn version_migrated(&self) -> bool {
        self.version_migrated.load(Ordering::Acquire)
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

    /// R2-28: trạng thái HIỆU LỰC của app — đúng luật TIP (`adapters/windows-tsf`
    /// `ipc_client::is_enabled`): toàn cục AND override (thiếu override = bật). Override
    /// chỉ TẮT riêng được một app; toàn cục đang E thì app nào cũng gõ tiếng Anh.
    pub fn effective_app_enabled(&self, app_id: &str) -> bool {
        let st = self.state.read().unwrap();
        effective_enabled(
            st.global_enabled,
            st.apps.get(&app_id.to_lowercase()).copied(),
        )
    }

    /// Override riêng của app (`None` = theo toàn cục).
    pub fn app_override(&self, app_id: &str) -> Option<bool> {
        self.state
            .read()
            .unwrap()
            .apps
            .get(&app_id.to_lowercase())
            .copied()
    }

    pub fn is_app_enabled(&self, app_id: &str) -> bool {
        let st = self.state.read().unwrap();
        // set_app_enabled chuẩn hoá lowercase khi ghi — lookup cũng phải chuẩn
        // hoá, ngược lại "Chrome.EXE" đọc rơi về global và broadcast sai giá trị.
        if let Some(&enabled) = st.apps.get(&app_id.to_lowercase()) {
            enabled
        } else {
            st.global_enabled
        }
    }

    pub fn app_states(&self) -> BTreeMap<String, bool> {
        self.state.read().unwrap().apps.clone()
    }

    /// Đảo trạng thái bật/tắt toàn cục tiếng Việt (WIN-015 / P1-4 §1).
    ///
    /// Chuẩn "lưu xong mới bump" (giống `update_config` — review R3 minor 8):
    /// persist thất bại thì KHÔNG bump version/không broadcast, TSF giữ state cũ
    /// nhất quán giữa RAM và đĩa.
    pub fn toggle_global_enabled(&self) -> (bool, u64) {
        let mut st = self.state.write().unwrap();
        st.global_enabled = !st.global_enabled;
        if self.persist_state(&st).is_err() {
            st.global_enabled = !st.global_enabled; // rollback RAM
            eprintln!("TextVN: persist state.json thất bại — giữ state cũ");
            return (st.global_enabled, self.state_version.load(Ordering::SeqCst));
        }
        let next_ver = self.state_version.fetch_add(1, Ordering::SeqCst) + 1;
        (st.global_enabled, next_ver)
    }

    /// Đặt trạng thái bật/tắt riêng cho từng app.
    pub fn set_app_enabled(&self, app_id: &str, enabled: bool) -> u64 {
        let mut st = self.state.write().unwrap();
        let prev = st.apps.insert(app_id.to_lowercase(), enabled);
        if self.persist_state(&st).is_err() {
            // rollback RAM theo chuẩn persist-first (review R3 minor 8)
            match prev {
                Some(v) => {
                    st.apps.insert(app_id.to_lowercase(), v);
                }
                None => {
                    st.apps.remove(&app_id.to_lowercase());
                }
            }
            eprintln!("TextVN: persist state.json thất bại — giữ state cũ");
            return self.state_version.load(Ordering::SeqCst);
        }
        self.state_version.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Đặt trạng thái bật/tắt toàn cục tiếng Việt có giá trị chỉ định.
    pub fn set_global_enabled(&self, enabled: bool) -> (bool, u64) {
        let mut st = self.state.write().unwrap();
        let prev = st.global_enabled;
        st.global_enabled = enabled;
        if self.persist_state(&st).is_err() {
            st.global_enabled = prev;
            eprintln!("TextVN: persist state.json thất bại — giữ state cũ");
            return (st.global_enabled, self.state_version.load(Ordering::SeqCst));
        }
        let next_ver = self.state_version.fetch_add(1, Ordering::SeqCst) + 1;
        (st.global_enabled, next_ver)
    }

    /// Ghi config mới trước rồi mới publish trong bộ nhớ/version. Nếu lưu thất bại
    /// (đĩa đầy, ACL, thư mục bị thay bằng file…), UI và TSF phải tiếp tục dùng
    /// cấu hình cũ thay vì báo đã đổi kiểu gõ nhưng engine không thể nạp lại.
    fn update_config(&self, f: impl FnOnce(&mut Config)) -> Result<u64, DocError> {
        let mut cfg = self.config.write().unwrap();
        let mut candidate = cfg.clone();
        f(&mut candidate);
        self.persist_config(&candidate)?;
        *cfg = candidate;
        let next_ver = self.config_version.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(next_ver)
    }

    /// Đổi kiểu gõ (Telex, VNI, VIQR, Telex đơn giản) (P1-4 §1).
    pub fn set_method(&self, method: Method) -> Result<u64, DocError> {
        self.update_config(|c| c.method = method)
    }

    /// Đổi kiểu bỏ dấu (mới `hoà` / cũ `hòa`).
    pub fn set_diacritic_style(&self, style: DiacriticStyle) -> Result<u64, DocError> {
        self.update_config(|c| c.diacritic_style = style)
    }

    /// Đổi bảng mã xuất (Unicode dựng sẵn/tổ hợp, TCVN3, VNI Windows).
    pub fn set_output_charset(
        &self,
        charset: textvn_config::OutputCharset,
    ) -> Result<u64, DocError> {
        self.update_config(|c| c.output_charset = charset)
    }

    /// Bật/tắt tự động khôi phục từ tiếng Anh khi gõ sai.
    pub fn set_auto_restore_english(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.auto_restore_english = enable)
    }

    /// Bật/tắt đặt dấu tự do (free marking).
    pub fn set_free_marking(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.free_marking = enable)
    }

    /// Bật/tắt tự viết hoa chữ đầu câu.
    pub fn set_auto_capitalize(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.auto_capitalize = enable)
    }

    /// Bật/tắt Quick Telex (`cc→ch`, `nn→ng`, …).
    pub fn set_quick_telex(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.quick_telex = enable)
    }

    /// Cho phép gõ tắt cả khi đang tắt tiếng Việt.
    pub fn set_allow_macro_when_vi_off(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.allow_macro_when_vi_off = enable)
    }

    /// Bật hội thoại cài đặt khi khởi động tray.
    pub fn set_show_dialog_on_startup(&self, enable: bool) -> Result<u64, DocError> {
        self.update_config(|c| c.show_dialog_on_startup = enable)
    }

    /// Thay danh sách từ EN của người dùng (từ điển bổ sung — 0.2.9). Engine
    /// restore/gợi ý theo `english_words` này bất kể kết quả fold có phải âm
    /// tiết Việt thông dụng hay không (quyết định tường minh thắng mọi phỏng đoán).
    pub fn set_english_words(&self, words: Vec<String>) -> Result<u64, DocError> {
        self.update_config(|c| c.english_words = words)
    }

    /// Thay bảng gõ tắt và phím mở rộng.
    pub fn set_macros(
        &self,
        macros: Vec<MacroEntry>,
        trigger: MacroTrigger,
    ) -> Result<u64, DocError> {
        self.update_config(|c| {
            c.macros = macros;
            c.macro_trigger = trigger;
        })
    }

    /// Nút "Mặc định": mọi tuỳ chọn gõ về mặc định; giữ bảng gõ tắt, emoji, từ tiếng Anh
    /// (UniKey cũng không xoá bảng gõ tắt khi bấm Mặc định).
    pub fn reset_config_defaults(&self) -> Result<u64, DocError> {
        self.update_config(|c| {
            let keep = (
                std::mem::take(&mut c.macros),
                std::mem::take(&mut c.emoji),
                std::mem::take(&mut c.english_words),
            );
            *c = Config::default();
            (c.macros, c.emoji, c.english_words) = keep;
        })
    }

    /// Vá các trường engine biết lên file hiện có (giữ khoá lạ do người dùng thêm).
    fn persist_config(&self, cfg: &Config) -> Result<(), DocError> {
        let path = self.config_dir.join("config.json");
        let mut doc = SettingsDoc::load(&path, DocKind::Config);
        doc.merge_config(cfg);
        doc.save(&path)
    }

    fn persist_state(&self, state: &StateData) -> Result<(), DocError> {
        let path = self.config_dir.join("state.json");
        let json = serde_json::to_string_pretty(state).map_err(|_| DocError::Io)?;
        atomic_write_file(&path, json.as_bytes()).map_err(|_| DocError::Io)
    }
}

/// Trả về đường dẫn `%APPDATA%\TextVN` mặc định (review R3 minor 9: bỏ nhánh
/// "legacy" dead-code — primary và legacy từng trùng nhau).
pub fn default_config_dir() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join("TextVN"),
        None => PathBuf::from(".textvn"),
    }
}

/// Ghi file an toàn (file tạm cùng thư mục → fsync → rename) — dùng chung với Linux.
fn atomic_write_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    textvn_config::doc::atomic_write(path, content)
}

/// Luật gõ tiếng Việt của TIP cho một app: toàn cục AND override (thiếu = bật).
pub fn effective_enabled(global: bool, app_override: Option<bool>) -> bool {
    global && app_override.unwrap_or(true)
}

/// Hạt giống version theo phiên tray: mili-giây Unix (≥ 1) — luôn khác các version một
/// phiên trước đã phát (đếm từ hạt giống của phiên đó lên vài nghìn).
fn session_version_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    /// R2-28: menu khay phải hiển thị đúng cái TIP làm — 4 tổ hợp toàn cục × override.
    #[test]
    fn effective_enabled_matches_tip_rule() {
        assert!(super::effective_enabled(true, None));
        assert!(super::effective_enabled(true, Some(true)));
        assert!(!super::effective_enabled(true, Some(false)));
        assert!(!super::effective_enabled(false, None));
        assert!(!super::effective_enabled(false, Some(true)));
        assert!(!super::effective_enabled(false, Some(false)));
    }

    fn seed_upgrade_env(dir: &std::path::Path) {
        // config "phiên bản cũ": method Vni + từ điển user + gõ tắt user
        let cfg = r#"{"config_version":1,"method":"vni","diacritic_style":"old",
            "auto_restore_english":false,"english_words":["cowork","list"],
            "macros":[{"trigger":"vn","expand":"Việt Nam","when":"always"}]}"#;
        let _ = std::fs::write(dir.join("config.json"), cfg);
        // state "phiên bản cũ": tắt global + per-app override + nhãn phiên cũ
        let st = r#"{"global_enabled":false,"apps":{"notepad.exe":false},
            "last_version":"0.2.9-test"}"#;
        let _ = std::fs::write(dir.join("state.json"), st);
    }

    /// CR-20/R2-33: nâng cấp phiên bản GIỮ NGUYÊN mọi lựa chọn của người dùng (kiểu gõ,
    /// tuỳ chọn, từ điển EN, gõ tắt, bật/tắt toàn cục, override theo app) — chỉ đổi nhãn
    /// phiên bản. Bản 0.2.10–0.2.27 reset tuỳ chọn ở mỗi lần cập nhật.
    #[test]
    fn version_change_keeps_user_choices() {
        let dir = std::env::temp_dir().join(format!("textvn_mig_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        seed_upgrade_env(&dir);

        let svc = SvcManager::new(Some(dir.clone()));
        assert!(svc.version_migrated(), "đổi phiên bản phải được nhận ra");
        assert!(!svc.first_run(), "đã có state.json → không phải lần đầu");

        let cfg = svc.config();
        assert_eq!(cfg.method, Method::Vni, "kiểu gõ người dùng chọn phải GIỮ");
        assert!(!cfg.auto_restore_english, "tuỳ chọn người dùng phải GIỮ");
        assert_eq!(
            cfg.english_words,
            vec!["cowork".to_string(), "list".to_string()],
            "từ điển EN user thêm phải GIỮ"
        );
        assert_eq!(cfg.macros.len(), 1, "gõ tắt user thêm phải GIỮ");

        let st = svc.state.read().unwrap();
        assert!(!st.global_enabled, "bật/tắt toàn cục phải GIỮ");
        assert_eq!(
            st.apps.get("notepad.exe"),
            Some(&false),
            "override app phải GIỮ"
        );
        assert_eq!(st.last_version, env!("CARGO_PKG_VERSION"));
        drop(st);

        // state.json trên đĩa cũng có nhãn mới
        let on_disk = std::fs::read_to_string(dir.join("state.json")).unwrap();
        assert!(on_disk.contains(env!("CARGO_PKG_VERSION")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// R2-25: chưa có state.json = lần đầu tray chạy cho tài khoản này.
    #[test]
    fn first_run_detected_only_without_state_file() {
        let dir = std::env::temp_dir().join(format!("textvn_first_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(SvcManager::new(Some(dir.clone())).first_run());
        assert!(!SvcManager::new(Some(dir.clone())).first_run());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Cùng phiên bản → KHÔNG reset (tuỳ chọn người dùng giữ nguyên lần mở sau).
    #[test]
    fn same_version_does_not_migrate() {
        let dir = std::env::temp_dir().join(format!("textvn_mig2_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let svc = SvcManager::new(Some(dir.clone()));
        assert!(
            !svc.version_migrated(),
            "lần đầu (chưa có nhãn) không phải migration"
        );
        let _ = svc.set_method(Method::Vni);

        // Mở lại: method Vni phải còn — chứng tỏ không bị reset
        let svc2 = SvcManager::new(Some(dir.clone()));
        assert!(!svc2.version_migrated());
        assert_eq!(svc2.config().method, Method::Vni);
        let _ = std::fs::remove_dir_all(&dir);
    }
    use super::*;

    #[test]
    fn svc_manager_initializes_and_toggles_state() {
        let temp_dir = std::env::temp_dir().join(format!("textvn_test_{}", std::process::id()));
        let svc = SvcManager::new(Some(temp_dir.clone()));

        assert!(svc.is_global_enabled());
        // Version khởi đầu duy nhất theo phiên (R2-31) — chỉ kiểm tăng đúng 1 mỗi lần.
        let start = svc.state_version();
        let (new_val, ver1) = svc.toggle_global_enabled();
        assert!(!new_val);
        assert_eq!(ver1, start + 1);

        let ver2 = svc.set_app_enabled("chrome.exe", true);
        assert_eq!(ver2, start + 2);
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
        let start = svc.config_version();
        let next_ver = svc.set_method(Method::Vni).unwrap();
        assert_eq!(next_ver, start + 1);
        assert_eq!(svc.config().method, Method::Vni);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn persist_keeps_unknown_keys_and_reset_keeps_macros() {
        let dir = std::env::temp_dir().join(format!("textvn_test_keep_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.json"),
            r#"{"config_version":1,"hotkeys":{"toggle_vi_en":"Ctrl+Shift+Space"},
               "macros":[{"trigger":"vn","expand":"Việt Nam"}]}"#,
        )
        .unwrap();
        let svc = SvcManager::new(Some(dir.clone()));
        svc.set_quick_telex(true).unwrap();
        svc.set_method(Method::Vni).unwrap();
        let on_disk: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("config.json")).unwrap()).unwrap();
        assert_eq!(on_disk["hotkeys"]["toggle_vi_en"], "Ctrl+Shift+Space");
        assert_eq!(on_disk["quick_telex"], true);
        assert_eq!(on_disk["macros"][0]["expand"], "Việt Nam");

        svc.reset_config_defaults().unwrap();
        let cfg = svc.config();
        assert_eq!(cfg.method, Method::Telex);
        assert!(!cfg.quick_telex);
        assert_eq!(cfg.macros.len(), 1, "Mặc định không được xoá bảng gõ tắt");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_config_is_backed_up_not_lost() {
        let dir = std::env::temp_dir().join(format!("textvn_test_corrupt_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("config.json"), "{ not json").unwrap();
        let svc = SvcManager::new(Some(dir.clone()));
        assert_eq!(svc.config(), Config::default());
        assert_eq!(
            fs::read_to_string(dir.join("config.json.bak")).unwrap(),
            "{ not json"
        );
        assert!(
            textvn_config::parse_config(&fs::read_to_string(dir.join("config.json")).unwrap())
                .is_ok()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_config_save_keeps_memory_and_version_unchanged() {
        let path =
            std::env::temp_dir().join(format!("textvn_test_save_failure_{}", std::process::id()));
        let _ = fs::remove_file(&path);
        fs::write(&path, "not a directory").unwrap();
        let svc = SvcManager::new(Some(path.clone()));

        let before = svc.config();
        let version = svc.config_version();
        assert_eq!(svc.set_method(Method::Vni), Err(DocError::Io));
        assert_eq!(svc.config(), before);
        assert_eq!(svc.config_version(), version);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn failed_state_save_keeps_memory_and_version_unchanged() {
        let path =
            std::env::temp_dir().join(format!("textvn_test_state_failure_{}", std::process::id()));
        let _ = fs::remove_file(&path);
        fs::write(&path, "not a directory").unwrap();
        let svc = SvcManager::new(Some(path.clone()));

        let enabled = svc.is_global_enabled();
        let app = svc.is_app_enabled("chrome.exe");
        let version = svc.state_version();

        assert_eq!(svc.toggle_global_enabled(), (enabled, version));
        assert_eq!(svc.set_global_enabled(!enabled), (enabled, version));
        assert_eq!(svc.set_app_enabled("chrome.exe", !app), version);
        assert_eq!(svc.is_global_enabled(), enabled);
        assert_eq!(svc.is_app_enabled("chrome.exe"), app);
        assert_eq!(svc.state_version(), version);

        let _ = fs::remove_file(&path);
    }
}
