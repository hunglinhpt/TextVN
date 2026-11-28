// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/ui/strings.toml + data/ui/settings_model.toml (digest FNV-1a 64 = 0x961b6c2f9503062f) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).
// Đổi chữ/cài đặt: sửa nguồn TOML rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.
//! Chữ UI & mô hình cài đặt dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.
#![allow(dead_code)]

pub const ABOUT_TAGLINE: &str = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)";

pub const ABOUT_DETAILS: [&str; 3] = [
    "Phát triển bởi: (C) 2026 Bui Hung Linh - LinhBH.CoM",
    "Giấy phép: GPL-3.0-or-later",
    "https://github.com/hunglinhpt/TextVN",
];

pub const URL_DISCUSSIONS: &str = "https://github.com/hunglinhpt/TextVN/discussions";

pub const URL_ISSUES: &str = "https://github.com/hunglinhpt/TextVN/issues";

pub const URL_MATH_GUIDE: &str =
    "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md";

pub const METHOD_LABELS: [&str; 4] = ["Telex", "VNI", "VIQR", "Telex đơn giản"];

pub const CHARSET_LABELS: [&str; 4] = [
    "Unicode dựng sẵn",
    "Unicode tổ hợp",
    "TCVN3 (ABC)",
    "VNI Windows",
];

pub const ASSIST_MODE_LABELS: [&str; 3] = ["Tắt", "Chỉ gợi ý", "Tự sửa lỗi an toàn"];

pub const ASSIST_MODE_HINTS: [&str; 3] = [
    "Không kiểm tra ngữ pháp và chính tả.",
    "Dừng gõ 1 giây: hiện lỗi và gợi ý, bạn tự chọn sửa.",
    "Tự sửa lỗi chắc chắn khi hết câu; Ctrl+Z hoàn tác.",
];

pub const METHOD_VALUES: [&str; 4] = ["telex", "vni", "viqr", "simple_telex"];

pub const OUTPUT_CHARSET_VALUES: [&str; 4] = [
    "unicode_precomposed",
    "unicode_decomposed",
    "tcvn3",
    "vni_windows",
];

pub const DIACRITIC_STYLE_VALUES: [&str; 2] = ["new", "old"];

pub const ASSIST_MODE_VALUES: [&str; 3] = ["off", "highlight", "autocorrect"];

pub const ASSIST_AUTO_FIX_VALUES: [&str; 2] = ["off", "safe"];

pub const ASSIST_IDLE_MS_DEFAULT: u32 = 700;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSettingSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub config_key: &'static str,
    pub kind: &'static str,
    pub default_val: &'static str,
    pub depends_on: &'static str,
}

pub const SETTINGS_MODEL: [UiSettingSpec; 18] = [
    UiSettingSpec {
        id: "method",
        group: "typing",
        config_key: "method",
        kind: "enum",
        default_val: "telex",
        depends_on: "",
    },
    UiSettingSpec {
        id: "output_charset",
        group: "typing",
        config_key: "output_charset",
        kind: "enum",
        default_val: "unicode_precomposed",
        depends_on: "",
    },
    UiSettingSpec {
        id: "diacritic_style",
        group: "typing",
        config_key: "diacritic_style",
        kind: "enum",
        default_val: "old",
        depends_on: "",
    },
    UiSettingSpec {
        id: "free_marking",
        group: "options",
        config_key: "free_marking",
        kind: "bool",
        default_val: "true",
        depends_on: "",
    },
    UiSettingSpec {
        id: "auto_restore_english",
        group: "options",
        config_key: "auto_restore_english",
        kind: "bool",
        default_val: "true",
        depends_on: "",
    },
    UiSettingSpec {
        id: "auto_capitalize",
        group: "options",
        config_key: "auto_capitalize",
        kind: "bool",
        default_val: "true",
        depends_on: "",
    },
    UiSettingSpec {
        id: "quick_telex",
        group: "options",
        config_key: "quick_telex",
        kind: "bool",
        default_val: "false",
        depends_on: "method=telex",
    },
    UiSettingSpec {
        id: "allow_macro_when_vi_off",
        group: "options",
        config_key: "allow_macro_when_vi_off",
        kind: "bool",
        default_val: "false",
        depends_on: "",
    },
    UiSettingSpec {
        id: "show_dialog_on_startup",
        group: "system",
        config_key: "show_dialog_on_startup",
        kind: "bool",
        default_val: "true",
        depends_on: "",
    },
    UiSettingSpec {
        id: "assist_mode",
        group: "assist",
        config_key: "assist.mode",
        kind: "enum",
        default_val: "highlight",
        depends_on: "",
    },
    UiSettingSpec {
        id: "assist_auto_fix",
        group: "assist",
        config_key: "assist.auto_fix",
        kind: "enum",
        default_val: "off",
        depends_on: "assist.mode=autocorrect",
    },
    UiSettingSpec {
        id: "assist_idle_ms",
        group: "assist",
        config_key: "assist.display.idle_ms",
        kind: "u32",
        default_val: "700",
        depends_on: "",
    },
    UiSettingSpec {
        id: "suggest_enabled",
        group: "assist",
        config_key: "suggest.enabled",
        kind: "bool",
        default_val: "false",
        depends_on: "",
    },
    UiSettingSpec {
        id: "style_profile_enabled",
        group: "assist",
        config_key: "style_profile.enabled",
        kind: "bool",
        default_val: "true",
        depends_on: "",
    },
    UiSettingSpec {
        id: "math_enabled",
        group: "math",
        config_key: "math.enabled",
        kind: "bool",
        default_val: "false",
        depends_on: "",
    },
    UiSettingSpec {
        id: "math_sticky",
        group: "math",
        config_key: "math.sticky_math_mode",
        kind: "bool",
        default_val: "false",
        depends_on: "math.enabled=true",
    },
    UiSettingSpec {
        id: "math_altgr",
        group: "math",
        config_key: "math.altgr_enabled",
        kind: "bool",
        default_val: "false",
        depends_on: "math.enabled=true",
    },
    UiSettingSpec {
        id: "math_smart_paste",
        group: "math",
        config_key: "math.clipboard_smart_paste",
        kind: "bool",
        default_val: "true",
        depends_on: "math.enabled=true",
    },
];
