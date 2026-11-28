// SPDX-License-Identifier: GPL-3.0-or-later
// == GENERATED FILE — KHÔNG SỬA TAY ==
// Nguồn: data/ui/strings.toml + data/ui/settings_model.toml (digest FNV-1a 64 = 0x961b6c2f9503062f) · sinh bằng `cargo xtask gen-ui` (SF-9, P0-6 §5b).
// Đổi chữ/cài đặt: sửa nguồn TOML rồi chạy lại `cargo xtask gen-ui`; `cargo xtask check-ui` (CI) fail nếu lệch.

/// Chữ UI & mô hình cài đặt dùng chung Windows/Linux/macOS (G22) — chỉ NỘI DUNG; vẽ UI vẫn native.
enum UiStrings {
    static let aboutTagline = "Bộ gõ tiếng Việt thông minh, đa nền tảng (Windows, macOS, Linux)"

    static let aboutDetails: [String] = [
        "Phát triển bởi: (C) 2026 Bui Hung Linh - LinhBH.CoM",
        "Giấy phép: GPL-3.0-or-later",
        "https://github.com/hunglinhpt/TextVN",
    ]

    static let urlDiscussions = "https://github.com/hunglinhpt/TextVN/discussions"

    static let urlIssues = "https://github.com/hunglinhpt/TextVN/issues"

    static let urlMathGuide = "https://github.com/hunglinhpt/TextVN/blob/main/docs/MATH_SCIENCE_GUIDE.md"

    static let methodLabels: [String] = [
        "Telex",
        "VNI",
        "VIQR",
        "Telex đơn giản",
    ]

    static let charsetLabels: [String] = [
        "Unicode dựng sẵn",
        "Unicode tổ hợp",
        "TCVN3 (ABC)",
        "VNI Windows",
    ]

    static let assistModeLabels: [String] = [
        "Tắt",
        "Chỉ gợi ý",
        "Tự sửa lỗi an toàn",
    ]

    static let assistModeHints: [String] = [
        "Không kiểm tra ngữ pháp và chính tả.",
        "Dừng gõ 1 giây: hiện lỗi và gợi ý, bạn tự chọn sửa.",
        "Tự sửa lỗi chắc chắn khi hết câu; Ctrl+Z hoàn tác.",
    ]
}

struct UiSettingSpec: Equatable {
    let id: String
    let group: String
    let configKey: String
    let kind: String
    let defaultVal: String
    let dependsOn: String
}

enum UiSettingsModel {
    static let methodValues: [String] = ["telex", "vni", "viqr", "simple_telex"]
    static let outputCharsetValues: [String] = ["unicode_precomposed", "unicode_decomposed", "tcvn3", "vni_windows"]
    static let diacriticStyleValues: [String] = ["new", "old"]
    static let assistModeValues: [String] = ["off", "highlight", "autocorrect"]
    static let assistAutoFixValues: [String] = ["off", "safe"]
    static let assistIdleMsDefault: Int = 700
    static let settingsModel: [UiSettingSpec] = [
        UiSettingSpec(id: "method", group: "typing", configKey: "method", kind: "enum", defaultVal: "telex", dependsOn: ""),
        UiSettingSpec(id: "output_charset", group: "typing", configKey: "output_charset", kind: "enum", defaultVal: "unicode_precomposed", dependsOn: ""),
        UiSettingSpec(id: "diacritic_style", group: "typing", configKey: "diacritic_style", kind: "enum", defaultVal: "old", dependsOn: ""),
        UiSettingSpec(id: "free_marking", group: "options", configKey: "free_marking", kind: "bool", defaultVal: "true", dependsOn: ""),
        UiSettingSpec(id: "auto_restore_english", group: "options", configKey: "auto_restore_english", kind: "bool", defaultVal: "true", dependsOn: ""),
        UiSettingSpec(id: "auto_capitalize", group: "options", configKey: "auto_capitalize", kind: "bool", defaultVal: "true", dependsOn: ""),
        UiSettingSpec(id: "quick_telex", group: "options", configKey: "quick_telex", kind: "bool", defaultVal: "false", dependsOn: "method=telex"),
        UiSettingSpec(id: "allow_macro_when_vi_off", group: "options", configKey: "allow_macro_when_vi_off", kind: "bool", defaultVal: "false", dependsOn: ""),
        UiSettingSpec(id: "show_dialog_on_startup", group: "system", configKey: "show_dialog_on_startup", kind: "bool", defaultVal: "true", dependsOn: ""),
        UiSettingSpec(id: "assist_mode", group: "assist", configKey: "assist.mode", kind: "enum", defaultVal: "highlight", dependsOn: ""),
        UiSettingSpec(id: "assist_auto_fix", group: "assist", configKey: "assist.auto_fix", kind: "enum", defaultVal: "off", dependsOn: "assist.mode=autocorrect"),
        UiSettingSpec(id: "assist_idle_ms", group: "assist", configKey: "assist.display.idle_ms", kind: "u32", defaultVal: "700", dependsOn: ""),
        UiSettingSpec(id: "suggest_enabled", group: "assist", configKey: "suggest.enabled", kind: "bool", defaultVal: "false", dependsOn: ""),
        UiSettingSpec(id: "style_profile_enabled", group: "assist", configKey: "style_profile.enabled", kind: "bool", defaultVal: "true", dependsOn: ""),
        UiSettingSpec(id: "math_enabled", group: "math", configKey: "math.enabled", kind: "bool", defaultVal: "false", dependsOn: ""),
        UiSettingSpec(id: "math_sticky", group: "math", configKey: "math.sticky_math_mode", kind: "bool", defaultVal: "false", dependsOn: "math.enabled=true"),
        UiSettingSpec(id: "math_altgr", group: "math", configKey: "math.altgr_enabled", kind: "bool", defaultVal: "false", dependsOn: "math.enabled=true"),
        UiSettingSpec(id: "math_smart_paste", group: "math", configKey: "math.clipboard_smart_paste", kind: "bool", defaultVal: "true", dependsOn: "math.enabled=true"),
    ]
}
