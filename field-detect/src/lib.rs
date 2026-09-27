// SPDX-License-Identifier: GPL-3.0-or-later
//! Context an toàn dùng chung trước khi adapter gọi engine.
//!
//! UIA/AX/AT-SPI là nguồn bất đồng bộ. Adapter tạo context cho mỗi focus với
//! `generation` mới và chỉ publish kết quả probe nếu generation còn khớp.
//! Trước khi biết chắc field không nhạy cảm, strategy phải Passthrough.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use textvn_appdb::AppDb;
use textvn_strategy::{ResolveInput, Strategy, IME_FIELD_SECURE, IME_FIELD_UNKNOWN};

pub mod rules_win;

/// Kết luận bảo mật từ OS. `Unknown` cố ý không phải `NonSecure`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityState {
    Unknown,
    NonSecure,
    Secure,
}

/// Snapshot đã probe, dùng cho cache adapter theo hwnd/app. Đây không chứa text
/// hay UIA object nên an toàn qua thread boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeSnapshot {
    pub field_role: u32,
    pub security: SecurityState,
}

#[derive(Debug, Clone, Copy)]
struct CachedProbe {
    value: ProbeSnapshot,
    observed_at: Duration,
}

/// Cache TTL có clock do caller cung cấp: adapter dùng `Instant::elapsed()`;
/// unit test không cần sleep. Kết quả hết hạn bị xoá ngay để UIA timeout không
/// vô tình tái sử dụng verdict mật khẩu cũ.
#[derive(Debug)]
pub struct ProbeCache<K> {
    ttl: Duration,
    entries: HashMap<K, CachedProbe>,
}

impl<K: Eq + Hash> ProbeCache<K> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: ProbeSnapshot, observed_at: Duration) {
        self.entries.insert(key, CachedProbe { value, observed_at });
    }

    pub fn get(&mut self, key: &K, now: Duration) -> Option<ProbeSnapshot> {
        let entry = self.entries.get(key)?;
        if now.saturating_sub(entry.observed_at) <= self.ttl {
            return Some(entry.value);
        }
        self.entries.remove(key);
        None
    }

    pub fn invalidate(&mut self, key: &K) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Context thuộc focus hiện hành trên owner thread của adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldContext {
    pub app_id: String,
    pub field_role: u32,
    pub security: SecurityState,
    pub caps: u32,
    pub generation: u64,
}

impl FieldContext {
    /// Context mới khởi đầu fail-safe. Adapter chỉ chuyển sang `NonSecure` sau
    /// khi nhận một tín hiệu đồng bộ hoặc kết quả probe khớp generation.
    pub fn pending(app_id: impl Into<String>, caps: u32, generation: u64) -> Self {
        Self {
            app_id: normalize_app_id(&app_id.into()),
            field_role: IME_FIELD_UNKNOWN,
            security: SecurityState::Unknown,
            caps,
            generation,
        }
    }

    /// Apply kết quả detector. `false` nghĩa là focus đã đổi; caller bỏ kết quả
    /// cũ, tuyệt đối không dùng để thay context mới.
    pub fn apply_probe(
        &mut self,
        generation: u64,
        field_role: u32,
        security: SecurityState,
    ) -> bool {
        if generation != self.generation {
            return false;
        }
        self.field_role = if field_role <= IME_FIELD_SECURE {
            field_role
        } else {
            IME_FIELD_UNKNOWN
        };
        self.security = security;
        true
    }

    /// Đường publish duy nhất cho Windows: adapter truyền snapshot UIA (hoặc
    /// `None` khi timeout/lỗi) kèm generation; classifier không bao giờ được
    /// tự ghi context ở worker thread.
    pub fn apply_windows_uia(
        &mut self,
        generation: u64,
        element: Option<&rules_win::UiaElement>,
    ) -> bool {
        let result = rules_win::classify(element);
        self.apply_probe(generation, result.field_role, result.security)
    }

    /// Chuyển context thành input resolver. Unknown/secure đều đóng gate S3.
    pub fn resolve_strategy(&self, enabled: bool, appdb: Option<&AppDb>) -> Strategy {
        let input = ResolveInput {
            secure: self.security != SecurityState::NonSecure,
            enabled,
            user_preset: None,
            system_preset: None,
            hint: -1,
            field_role: self.field_role,
            caps: self.caps,
        };
        match appdb {
            Some(db) => db.resolve(input, &self.app_id),
            None => textvn_strategy::resolve(input),
        }
    }
}

/// OS adapter chuẩn hóa stable identifier trước lookup. Windows exe matching là
/// case-insensitive; hạ ASCII case ở đây để cache/appdb nhất quán, giữ Unicode.
pub fn normalize_app_id(value: &str) -> String {
    value
        .trim()
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use textvn_strategy::{
        IME_CAP_PREEDIT, IME_CAP_SELECTION, IME_FIELD_ADDRESS_BAR, IME_FIELD_BODY,
    };

    #[test]
    fn probe_cache_expires_and_can_be_invalidated() {
        let mut cache = ProbeCache::new(Duration::from_secs(2));
        cache.insert(
            42u64,
            ProbeSnapshot {
                field_role: IME_FIELD_BODY,
                security: SecurityState::NonSecure,
            },
            Duration::from_secs(10),
        );
        assert_eq!(
            cache.get(&42, Duration::from_secs(12)),
            Some(ProbeSnapshot {
                field_role: IME_FIELD_BODY,
                security: SecurityState::NonSecure
            })
        );
        assert!(cache.invalidate(&42));
        assert_eq!(cache.get(&42, Duration::from_secs(12)), None);

        cache.insert(
            42,
            ProbeSnapshot {
                field_role: IME_FIELD_BODY,
                security: SecurityState::NonSecure,
            },
            Duration::from_secs(10),
        );
        assert_eq!(cache.get(&42, Duration::from_secs(13)), None);
    }

    #[test]
    fn pending_or_secure_context_never_transforms() {
        let mut ctx = FieldContext::pending("Chrome.EXE", IME_CAP_PREEDIT, 7);
        assert_eq!(ctx.app_id, "chrome.exe");
        assert_eq!(ctx.resolve_strategy(true, None), Strategy::Passthrough);

        assert!(ctx.apply_probe(7, IME_FIELD_BODY, SecurityState::Secure));
        assert_eq!(ctx.resolve_strategy(true, None), Strategy::Passthrough);
    }

    #[test]
    fn stale_probe_cannot_unblock_new_focus() {
        let mut ctx = FieldContext::pending("keepassxc.exe", IME_CAP_PREEDIT, 12);
        assert!(!ctx.apply_probe(11, IME_FIELD_BODY, SecurityState::NonSecure));
        assert_eq!(ctx.security, SecurityState::Unknown);
        assert_eq!(ctx.resolve_strategy(true, None), Strategy::Passthrough);
    }

    #[test]
    fn current_nonsecure_probe_can_use_capability_and_preset() {
        let db = AppDb::parse(
            r#"{"appdb_version":1,"entries":[{"match":{"exe":"chrome.exe"},"when":{"field_role":"address_bar"},"strategy":"SelectionReplace"}]}"#,
        )
        .unwrap();
        let mut ctx = FieldContext::pending("chrome.exe", IME_CAP_SELECTION, 1);
        assert!(ctx.apply_probe(1, IME_FIELD_ADDRESS_BAR, SecurityState::NonSecure));
        assert_eq!(
            ctx.resolve_strategy(true, Some(&db)),
            Strategy::SelectionReplace
        );
    }

    #[test]
    fn windows_uia_publish_keeps_generation_and_unknown_security_gate() {
        let mut ctx = FieldContext::pending("chrome.exe", IME_CAP_PREEDIT, 2);
        let element = rules_win::UiaElement {
            control_type: rules_win::ControlType::Edit,
            is_password: Some(false),
            keyboard_focusable: true,
            control_element: true,
            ..Default::default()
        };
        assert!(!ctx.apply_windows_uia(1, Some(&element)));
        assert_eq!(ctx.security, SecurityState::Unknown);
        assert_eq!(ctx.resolve_strategy(true, None), Strategy::Passthrough);

        assert!(ctx.apply_windows_uia(2, Some(&element)));
        assert_eq!(ctx.security, SecurityState::NonSecure);
        assert_ne!(ctx.resolve_strategy(true, None), Strategy::Passthrough);
        assert!(ctx.apply_windows_uia(2, None));
        assert_eq!(ctx.security, SecurityState::Unknown);
        assert_eq!(ctx.resolve_strategy(true, None), Strategy::Passthrough);
    }

    #[test]
    fn app_id_uses_stable_executable_basename() {
        assert_eq!(
            normalize_app_id(r"C:\Program Files\Google\Chrome.EXE"),
            "chrome.exe"
        );
        assert_eq!(normalize_app_id("/usr/bin/WezTerm-GUI"), "wezterm-gui");
        assert_eq!(normalize_app_id(" com.google.Chrome "), "com.google.chrome");
    }
}
