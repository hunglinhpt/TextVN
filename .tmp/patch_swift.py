import io

p = 'adapters/macos-app/Sources/TextVNAppLib/ConfigModel.swift'
s = io.open(p, encoding='utf-8').read()

# 1. MathSettings struct — chèn trước "public struct TextVNConfig"
anchor = 'public struct TextVNConfig: Codable, Equatable {'
assert anchor in s
math_struct = '''/// `math` — Math Engine (P11, Universal Scientific & Math Engine). Mặc định TẮT.
/// Model PHẢI biết khoá này (giống `english_words`/`emoji`/`assist`) nếu không
/// lần lưu nào từ app cũng XOÁ cài đặt math của người dùng.
/// Khớp 1-1 với `textvn_math::MathConfig` (crate math) và schema.
public struct MathSettings: Codable, Equatable {
    /// Master switch — mặc định `false` (hành vi gõ cũ giữ nguyên 100%).
    public var enabled: Bool
    /// Sticky Math Mode (Ctrl+Shift+M) — chords hoạt động ở mọi app.
    public var sticky_math_mode: Bool
    /// `Normal` | `Latex` | `UnicodeMath` | `Typst` | `AutoByApp`.
    public var output_mode: String
    /// Phím leader (1 ký tự ASCII, mặc định backtick).
    public var leader_key: String
    /// Layer AltGr một-phím (opt-in).
    public var altgr_enabled: Bool
    /// Ctrl+Alt+V Smart Formula Paste.
    public var clipboard_smart_paste: Bool
    /// Ctrl+Alt+Space Quick Math Palette.
    public var palette_enabled: Bool
    /// Binding tùy chỉnh `[trigger, output]`.
    public var custom_keybindings: [[String]]
    /// Timeout chord (ms).
    public var chord_timeout_ms: Int
    /// Ngưỡng pause hiện Mini-HUD (ms).
    public var hud_pause_ms: Int
    /// Hotkey toàn cục (4 mặc định).
    public var hotkeys: [MathHotkey]
    /// Số kết quả palette tối đa.
    public var palette_max_results: Int

    public struct MathHotkey: Codable, Equatable {
        public var combo: String
        public var action: String
        public init(combo: String, action: String) {
            self.combo = combo
            self.action = action
        }
    }

    public init(
        enabled: Bool = false,
        sticky_math_mode: Bool = false,
        output_mode: String = "Normal",
        leader_key: String = "`",
        altgr_enabled: Bool = false,
        clipboard_smart_paste: Bool = true,
        palette_enabled: Bool = false,
        custom_keybindings: [[String]] = [],
        chord_timeout_ms: Int = 1500,
        hud_pause_ms: Int = 200,
        hotkeys: [MathHotkey] = [
            .init(combo: "Ctrl+Alt+M", action: "cycle_mode"),
            .init(combo: "Ctrl+Shift+M", action: "toggle_sticky"),
            .init(combo: "Ctrl+Alt+V", action: "smart_paste"),
            .init(combo: "Ctrl+Alt+Space", action: "palette"),
        ],
        palette_max_results: Int = 20
    ) {
        self.enabled = enabled
        self.sticky_math_mode = sticky_math_mode
        self.output_mode = output_mode
        self.leader_key = leader_key
        self.altgr_enabled = altgr_enabled
        self.clipboard_smart_paste = clipboard_smart_paste
        self.palette_enabled = palette_enabled
        self.custom_keybindings = custom_keybindings
        self.chord_timeout_ms = chord_timeout_ms
        self.hud_pause_ms = hud_pause_ms
        self.hotkeys = hotkeys
        self.palette_max_results = palette_max_results
    }

    /// Decode khoan dung: thiếu trường lấy mặc định (cùng quy ước TextVNConfig).
    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let d = MathSettings()
        enabled = try c.decodeIfPresent(Bool.self, forKey: .enabled) ?? d.enabled
        sticky_math_mode = try c.decodeIfPresent(Bool.self, forKey: .sticky_math_mode) ?? d.sticky_math_mode
        output_mode = try c.decodeIfPresent(String.self, forKey: .output_mode) ?? d.output_mode
        leader_key = try c.decodeIfPresent(String.self, forKey: .leader_key) ?? d.leader_key
        altgr_enabled = try c.decodeIfPresent(Bool.self, forKey: .altgr_enabled) ?? d.altgr_enabled
        clipboard_smart_paste = try c.decodeIfPresent(Bool.self, forKey: .clipboard_smart_paste) ?? d.clipboard_smart_paste
        palette_enabled = try c.decodeIfPresent(Bool.self, forKey: .palette_enabled) ?? d.palette_enabled
        custom_keybindings = try c.decodeIfPresent([[String]].self, forKey: .custom_keybindings) ?? d.custom_keybindings
        chord_timeout_ms = try c.decodeIfPresent(Int.self, forKey: .chord_timeout_ms) ?? d.chord_timeout_ms
        hud_pause_ms = try c.decodeIfPresent(Int.self, forKey: .hud_pause_ms) ?? d.hud_pause_ms
        hotkeys = try c.decodeIfPresent([MathHotkey].self, forKey: .hotkeys) ?? d.hotkeys
        palette_max_results = try c.decodeIfPresent(Int.self, forKey: .palette_max_results) ?? d.palette_max_results
    }
}

'''
s = s.replace(anchor, math_struct + anchor, 1)

# 2. TextVNConfig: field + init param + assignment
old = '''    /// User Style Profile (P7, vòng 54) — model PHẢI biết khoá này nếu không
    /// lần lưu nào từ app cũng XOÁ dữ liệu học của người dùng.
    public var style_profile: StyleProfileSettings
'''
new = old + '''    /// Math Engine (P11) — model PHẢI biết khoá này nếu không lần lưu nào
    /// từ app cũng XOÁ cài đặt math của người dùng (mẫu english_words 0.2.13).
    public var math: MathSettings
'''
assert old in s
s = s.replace(old, new, 1)

old = '''        assist: AssistSettings = AssistSettings(),
        style_profile: StyleProfileSettings = StyleProfileSettings()
    ) {'''
new = '''        assist: AssistSettings = AssistSettings(),
        style_profile: StyleProfileSettings = StyleProfileSettings(),
        math: MathSettings = MathSettings()
    ) {'''
assert old in s
s = s.replace(old, new, 1)

old = '''        self.assist = assist
        self.style_profile = style_profile
    }'''
new = '''        self.assist = assist
        self.style_profile = style_profile
        self.math = math
    }'''
assert old in s
s = s.replace(old, new, 1)

old = '''        style_profile = try c.decodeIfPresent(StyleProfileSettings.self, forKey: .style_profile) ?? d.style_profile
    }'''
new = '''        style_profile = try c.decodeIfPresent(StyleProfileSettings.self, forKey: .style_profile) ?? d.style_profile
        // P11 (math): thiếu khoá = default TẮT — không xoá cài đặt khi lưu.
        math = try c.decodeIfPresent(MathSettings.self, forKey: .math) ?? d.math
    }'''
assert old in s
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('swift patched')
