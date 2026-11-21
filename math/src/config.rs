// SPDX-License-Identifier: GPL-3.0-or-later
//! Cấu hình math — serde-backed, default tương thích ngược (P0-3).
//!
//! Toàn bộ phần logic thuần, zero-unsafe, không I/O. Adapter ánh xạ section
//! `config.math` (textvn-config) sang [`MathConfig`] rồi nạp vào [`MathEngine`].

use serde::{Deserialize, Serialize};

/// Chế độ kết xuất math (`ime.math.mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum MathOutputMode {
    /// Ký tự Unicode thuần (÷, √, ±) — mode mặc định.
    #[default]
    Normal,
    Latex,
    UnicodeMath,
    Typst,
    /// Chọn theo app đang focus (`app_bridge::resolve_output_mode`).
    AutoByApp,
}

/// Hành động trả về cho MỘT phím đã qua FSM math.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathAction {
    /// Math không can thiệp — phím đi thẳng.
    PassThrough,
    /// Nuốt phím lặng lẽ (leader armed...).
    Consume,
    /// Chèn `text` (không xoá) + dịch con trỏ trái `cursor_left_steps` bước.
    Emit {
        text: String,
        cursor_left_steps: usize,
        show_hud: Option<HudHint>,
    },
    /// Xoá `delete_chars` ký tự trước con trỏ rồi chèn `text`
    /// (mở rộng alias/template, tính =expr=).
    ReplaceBuffer {
        delete_chars: usize,
        text: String,
        cursor_left_steps: usize,
    },
}

impl MathAction {
    /// Con trỏ trái cần dịch sau khi áp hành động.
    pub fn cursor_left_steps(&self) -> usize {
        match self {
            MathAction::PassThrough | MathAction::Consume => 0,
            MathAction::Emit {
                cursor_left_steps, ..
            }
            | MathAction::ReplaceBuffer {
                cursor_left_steps, ..
            } => *cursor_left_steps,
        }
    }

    /// Text do hành động chèn (`PassThrough`/`Consume` → rỗng).
    pub fn inserted_text(&self) -> &str {
        match self {
            MathAction::PassThrough | MathAction::Consume => "",
            MathAction::Emit { text, .. } | MathAction::ReplaceBuffer { text, .. } => text,
        }
    }
}

/// Mini-HUD không lấy focus: gợi ý phím mnemonic khi Armed (pause > 200 ms).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudHint {
    pub title: String,
    pub items: Vec<(String, String)>,
}

/// 6 lĩnh vực khoa học — tag cho mỗi entry lexicon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Discipline {
    PureMath,
    PhysicsMech,
    ElectricalEmbedded,
    ChemistryBio,
    ComputerScienceAi,
    QuantFinance,
}

impl Discipline {
    pub fn all() -> &'static [Discipline; 6] {
        &[
            Discipline::PureMath,
            Discipline::PhysicsMech,
            Discipline::ElectricalEmbedded,
            Discipline::ChemistryBio,
            Discipline::ComputerScienceAi,
            Discipline::QuantFinance,
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Discipline::PureMath => "pure_math",
            Discipline::PhysicsMech => "physics_mech",
            Discipline::ElectricalEmbedded => "electrical_embedded",
            Discipline::ChemistryBio => "chemistry_bio",
            Discipline::ComputerScienceAi => "cs_ai",
            Discipline::QuantFinance => "quant_finance",
        }
    }
}

/// Tổ hợp phím toàn cục do TextVN quản lý (user remap được).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeySpec {
    /// Chuỗi như "Ctrl+Alt+M" — adapter đăng ký theo nền tảng.
    pub combo: String,
    /// Tên hành động: `cycle_mode` | `toggle_sticky` | `smart_paste` | `palette`.
    pub action: String,
}

impl Default for HotkeySpec {
    fn default() -> Self {
        Self {
            combo: String::new(),
            action: String::new(),
        }
    }
}

/// Cấu hình math (`config.math`) — mọi field có default nên config cũ
/// không có section `math` vẫn parse được (tương thích ngược).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MathConfig {
    /// Master switch `math.enabled` — mặc định TẮT (hành vi cũ giữ nguyên).
    pub enabled: bool,
    /// Sticky Math Mode ON/OFF (`Ctrl+Shift+M`).
    pub sticky_math_mode: bool,
    /// Chế độ kết xuất.
    pub output_mode: MathOutputMode,
    /// Phím leader (default: backtick `` ` ``) khi Math Mode active.
    pub leader_key: char,
    /// Layer AltGr một-phím (opt-in).
    pub altgr_enabled: bool,
    /// Ctrl+Alt+V Smart Formula Paste.
    pub clipboard_smart_paste: bool,
    /// Palette tìm kiếm ký tự/công thức (Ctrl+Alt+Space).
    pub palette_enabled: bool,
    /// Binding tùy chỉnh `trigger → output` (user định nghĩa thêm).
    pub custom_keybindings: Vec<(String, String)>,
    /// Timeout chord (ms) — quá hạn về Idle lặng lẽ.
    pub chord_timeout_ms: u64,
    /// Ngưỡng pause hiện Mini-HUD (ms) khi Armed.
    pub hud_pause_ms: u64,
    /// Hotkeys toàn cục (4 mặc định khai báo đủ để UI/docs nhất quán).
    pub hotkeys: Vec<HotkeySpec>,
    /// Số kết quả tối đa của palette.
    pub palette_max_results: usize,
}

impl Default for MathConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sticky_math_mode: false,
            output_mode: MathOutputMode::Normal,
            leader_key: '`',
            altgr_enabled: false,
            clipboard_smart_paste: true,
            palette_enabled: false,
            custom_keybindings: Vec::new(),
            chord_timeout_ms: 1500,
            hud_pause_ms: 200,
            hotkeys: default_hotkeys(),
            palette_max_results: 20,
        }
    }
}

/// 4 hotkey mặc định — không đụng OS reserved (Win+*, Alt+Tab, Ctrl+Alt+Del)
/// và không đụng Ctrl+A/C/V/X/Z/Y/S toàn cục.
pub fn default_hotkeys() -> Vec<HotkeySpec> {
    vec![
        HotkeySpec {
            combo: "Ctrl+Alt+M".into(),
            action: "cycle_mode".into(),
        },
        HotkeySpec {
            combo: "Ctrl+Shift+M".into(),
            action: "toggle_sticky".into(),
        },
        HotkeySpec {
            combo: "Ctrl+Alt+V".into(),
            action: "smart_paste".into(),
        },
        HotkeySpec {
            combo: "Ctrl+Alt+Space".into(),
            action: "palette".into(),
        },
    ]
}

impl MathConfig {
    /// Leader phát một lần — same leader × 2 → literal leader.
    pub fn is_leader(self, c: char) -> bool {
        c == self.leader_key
    }

    /// Binding tùy chỉnh khớp chính xác trigger (`\tên` → output).
    pub fn custom_output(&self, trigger: &str) -> Option<&str> {
        let needle = trigger.strip_prefix('\\').unwrap_or(trigger);
        self.custom_keybindings
            .iter()
            .find(|(k, _)| k.strip_prefix('\\').unwrap_or(k) == needle)
            .map(|(_, v)| v.as_str())
    }

    /// Ghi đè leader qua config (chars đa byte không nhận — chỉ ASCII 1 byte).
    pub fn set_leader_key(&mut self, c: char) -> Result<(), ConfigError> {
        if c.len_utf8() != 1 || c.is_whitespace() || c.is_control() {
            return Err(ConfigError::InvalidLeader);
        }
        self.leader_key = c;
        Ok(())
    }
}

/// Lỗi cấu hình math — validator trả về trước khi nạp engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// Leader không phải 1 ký tự ASCII in được.
    InvalidLeader,
    /// Hai trigger/alias trùng nhau (mở rộng sẽ mơ hồ).
    DuplicateTrigger,
    /// Trigger/alias chứa khoảng trắng hoặc ký tự điều khiển.
    InvalidTrigger,
    /// Hotkey đè phím tắt được bảo vệ (OS reserved / Ctrl+A/C/V/X/Z/Y/S).
    ProtectedHotkeyConflict,
    /// Timeout ngoài [100, 10_000] ms.
    InvalidTimeout,
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            ConfigError::InvalidLeader => "leader key phải là 1 ký tự ASCII in được",
            ConfigError::DuplicateTrigger => "trigger/alias trùng nhau",
            ConfigError::InvalidTrigger => "trigger chứa khoảng trắng/ký tự điều khiển",
            ConfigError::ProtectedHotkeyConflict => "hotkey đè phím tắt được bảo vệ",
            ConfigError::InvalidTimeout => "chord_timeout_ms ngoài [100, 10000]",
        };
        f.write_str(s)
    }
}

/// Trigger/alias hợp lệ: 1..=32 ký tự, không whitespace/control; không bắt đầu `\`
/// (backslash do engine thêm khi khớp từ trong document).
pub fn valid_trigger_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && !s.starts_with('\\')
        && s.chars()
            .all(|c| !c.is_whitespace() && !c.is_control() && c != '\\' && c != '{' && c != '}')
}

/// Combo hotkey được bảo vệ tuyệt đối — validator CHẶN binding trùng.
const PROTECTED_COMBOS: &[&str] = &[
    "Ctrl+A",
    "Ctrl+C",
    "Ctrl+V",
    "Ctrl+X",
    "Ctrl+Z",
    "Ctrl+Y",
    "Ctrl+S",
    "Alt+Tab",
    "Alt+F4",
    "Ctrl+Alt+Del",
];

/// Chuẩn hoá combo cho so khớp không phân biệt thứ tự modifier:
/// "alt+ctrl+M" == "Ctrl+Alt+M".
pub fn normalize_combo(combo: &str) -> String {
    let mut parts: Vec<String> = combo
        .split('+')
        .map(|p| {
            let p = p.trim();
            if p.eq_ignore_ascii_case("ctrl")
                || p.eq_ignore_ascii_case("alt")
                || p.eq_ignore_ascii_case("shift")
            {
                p.to_ascii_uppercase()
            } else {
                p.to_string()
            }
        })
        .collect();
    // Sort modifier trước, phím chính sau (giữ nguyên thứ tự tương đối khi cùng loại)
    parts.sort_by_key(|p| match p.as_str() {
        "CTRL" => 0,
        "ALT" => 1,
        "SHIFT" => 2,
        _ => 3,
    });
    parts.join("+")
}

impl MathConfig {
    /// Validator tổng — trả về lỗi ĐẦU TIÊN gặp phải.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(100..=10_000).contains(&self.chord_timeout_ms) {
            return Err(ConfigError::InvalidTimeout);
        }
        if self.leader_key.len_utf8() != 1
            || self.leader_key.is_whitespace()
            || self.leader_key.is_control()
        {
            return Err(ConfigError::InvalidLeader);
        }
        for hk in &self.hotkeys {
            let norm = normalize_combo(&hk.combo);
            if PROTECTED_COMBOS.iter().any(|p| normalize_combo(p) == norm) {
                return Err(ConfigError::ProtectedHotkeyConflict);
            }
        }
        // custom bindings + trùng với hotkey action name (đề phòng mơ hồ)
        let mut seen: Vec<&str> = Vec::new();
        for (k, _) in &self.custom_keybindings {
            // Chấp nhận cả dạng có/không backslash đầu — so khớp ở dạng stripped.
            let key = k.strip_prefix('\\').unwrap_or(k);
            if !valid_trigger_token(key) {
                return Err(ConfigError::InvalidTrigger);
            }
            if seen.contains(&key) {
                return Err(ConfigError::DuplicateTrigger);
            }
            seen.push(key);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_backward_compatible() {
        let cfg = MathConfig::default();
        assert!(!cfg.enabled, "math mặc định TẮT — hành vi cũ giữ nguyên");
        assert!(!cfg.sticky_math_mode);
        assert_eq!(cfg.leader_key, '`');
        assert_eq!(cfg.output_mode, MathOutputMode::Normal);
        assert_eq!(cfg.chord_timeout_ms, 1500);
        assert_eq!(cfg.hud_pause_ms, 200);
        assert!(cfg.clipboard_smart_paste);
        assert_eq!(cfg.hotkeys.len(), 4);
    }

    #[test]
    fn serde_roundtrip_preserves_fields() {
        let mut cfg = MathConfig::default();
        cfg.enabled = true;
        cfg.output_mode = MathOutputMode::Typst;
        cfg.set_leader_key('\\').unwrap();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: MathConfig = serde_json::from_str(&json).unwrap();
        assert!(back.enabled);
        assert_eq!(back.output_mode, MathOutputMode::Typst);
        assert_eq!(back.leader_key, '\\');
    }

    #[test]
    fn serde_missing_section_uses_defaults() {
        // Config cũ không có section math → parse rỗng vẫn được (tương thích ngược).
        let back: MathConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(back, MathConfig::default());
    }

    #[test]
    fn custom_output_strips_backslash() {
        let cfg = MathConfig {
            custom_keybindings: vec![("\\myop".into(), "⊗".into())],
            ..MathConfig::default()
        };
        assert_eq!(cfg.custom_output("\\myop"), Some("⊗"));
        assert_eq!(cfg.custom_output("myop"), Some("⊗"));
        assert_eq!(cfg.custom_output("\\other"), None);
    }

    #[test]
    fn validate_rejects_protected_hotkey() {
        let cfg = MathConfig {
            hotkeys: vec![HotkeySpec {
                combo: "Ctrl+V".into(),
                action: "smart_paste".into(),
            }],
            ..MathConfig::default()
        };
        assert_eq!(cfg.validate(), Err(ConfigError::ProtectedHotkeyConflict));
    }

    #[test]
    fn validate_normalizes_modifier_order() {
        let cfg = MathConfig {
            hotkeys: vec![HotkeySpec {
                combo: "alt+CTRL+M".into(),
                action: "cycle_mode".into(),
            }],
            ..MathConfig::default()
        };
        assert_eq!(cfg.validate(), Ok(()));
        assert_eq!(normalize_combo("alt+CTRL+M"), "CTRL+ALT+M");
    }

    #[test]
    fn validate_rejects_duplicate_and_invalid_trigger() {
        let dup = MathConfig {
            custom_keybindings: vec![("ab".into(), "α".into()), ("\\ab".into(), "β".into())],
            ..MathConfig::default()
        };
        assert_eq!(dup.validate(), Err(ConfigError::DuplicateTrigger));
        let bad = MathConfig {
            custom_keybindings: vec![("a b".into(), "α".into())],
            ..MathConfig::default()
        };
        assert_eq!(bad.validate(), Err(ConfigError::InvalidTrigger));
    }

    #[test]
    fn validate_timeout_bounds_and_leader() {
        let cfg = MathConfig {
            chord_timeout_ms: 50,
            ..MathConfig::default()
        };
        assert_eq!(cfg.validate(), Err(ConfigError::InvalidTimeout));
        let cfg = MathConfig {
            leader_key: 'é', // 2 byte
            ..MathConfig::default()
        };
        assert_eq!(cfg.validate(), Err(ConfigError::InvalidLeader));
    }

    #[test]
    fn set_leader_rejects_non_ascii() {
        let mut cfg = MathConfig::default();
        assert!(cfg.set_leader_key('\\').is_ok());
        assert!(cfg.set_leader_key('ế').is_err());
        assert!(cfg.set_leader_key(' ').is_err());
    }

    #[test]
    fn action_helpers() {
        let emit = MathAction::Emit {
            text: "α".into(),
            cursor_left_steps: 0,
            show_hud: None,
        };
        assert_eq!(emit.inserted_text(), "α");
        assert_eq!(emit.cursor_left_steps(), 0);
        let rep = MathAction::ReplaceBuffer {
            delete_chars: 5,
            text: "∠".into(),
            cursor_left_steps: 0,
        };
        assert_eq!(rep.cursor_left_steps(), 0);
    }

    #[test]
    fn discipline_roundtrip() {
        for d in Discipline::all() {
            let json = serde_json::to_string(d).unwrap();
            let back: Discipline = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, d);
        }
    }

    #[test]
    fn valid_trigger_token_rules() {
        assert!(valid_trigger_token("can"));
        assert!(valid_trigger_token("mat2x2"));
        assert!(!valid_trigger_token(""));
        assert!(!valid_trigger_token("a b"));
        assert!(!valid_trigger_token("\\can"));
        assert!(!valid_trigger_token("a{b}"));
        assert!(!valid_trigger_token(&"x".repeat(33)));
    }
}
