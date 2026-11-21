import io

p = 'schemas/config.v1.schema.json'
s = io.open(p, encoding='utf-8').read()

anchor = '''    "assist": {'''
assert anchor in s

math_block = '''    "math": {
      "type": "object",
      "default": {
        "enabled": false,
        "sticky_math_mode": false,
        "output_mode": "Normal",
        "leader_key": "`",
        "altgr_enabled": false,
        "clipboard_smart_paste": true,
        "palette_enabled": false,
        "custom_keybindings": [],
        "chord_timeout_ms": 1500,
        "hud_pause_ms": 200,
        "hotkeys": [
          { "combo": "Ctrl+Alt+M", "action": "cycle_mode" },
          { "combo": "Ctrl+Shift+M", "action": "toggle_sticky" },
          { "combo": "Ctrl+Alt+V", "action": "smart_paste" },
          { "combo": "Ctrl+Alt+Space", "action": "palette" }
        ],
        "palette_max_results": 20
      },
      "additionalProperties": false,
      "properties": {
        "enabled": {
          "type": "boolean",
          "default": false,
          "$comment": "P11 - Master switch Math Engine. MAC DINH TAT: hanh vi go cuu giu nguyen 100% khi user khong bat."
        },
        "sticky_math_mode": {
          "type": "boolean",
          "default": false,
          "$comment": "P11 - Sticky Math Mode (Ctrl+Shift+M): leader chords hoat dong o moi app, khong can trong $...$."
        },
        "output_mode": {
          "type": "string",
          "enum": ["Normal", "Latex", "UnicodeMath", "Typst", "AutoByApp"],
          "default": "Normal",
          "$comment": "P11 - Che do ket xuat ky tu/template. AutoByApp = chon theo app dang focus (Word/MathType -> Latex, Zalo/Teams -> Unicode, Typst editor -> Typst)."
        },
        "leader_key": {
          "type": "string",
          "minLength": 1,
          "maxLength": 1,
          "default": "`",
          "$comment": "P11 - Phim leader (1 ky tu ASCII in duoc, mac dinh backtick). Nhan 2 lan = ky tu literal, Esc/Space = cancel."
        },
        "altgr_enabled": {
          "type": "boolean",
          "default": false,
          "$comment": "P11 - Layer AltGr mot-phim (opt-in): AltGr+a -> alpha, AltGr+2 -> sup 2... Validator chan binding de phong tu hop bi bao ve."
        },
        "clipboard_smart_paste": {
          "type": "boolean",
          "default": true,
          "$comment": "P11 - Ctrl+Alt+V Smart Formula Paste: convert cong thuc trong clipboard sang dinh dang phu hop app (Word/MathType = MathML, code editor = LaTeX, chat = Unicode)."
        },
        "palette_enabled": {
          "type": "boolean",
          "default": false,
          "$comment": "P11 - Ctrl+Alt+Space Quick Math Palette (tim kiem ky hieu/cong thuc)."
        },
        "custom_keybindings": {
          "type": "array",
          "items": {
            "type": "array",
            "minItems": 2,
            "maxItems": 2,
            "items": [{ "type": "string" }, { "type": "string" }]
          },
          "default": [],
          "$comment": "P11 - Binding tuy chinh [trigger, output] - vi du [[\\\\myop, \\u2297]]. Trigger khong co backslash dau, khong whitespace."
        },
        "chord_timeout_ms": {
          "type": "integer",
          "minimum": 100,
          "maximum": 10000,
          "default": 1500,
          "$comment": "P11 - Timeout chord (ms) - qua han FSM ve Idle lang le, khong bao gio treo trang thai."
        },
        "hud_pause_ms": {
          "type": "integer",
          "minimum": 50,
          "maximum": 2000,
          "default": 200,
          "$comment": "P11 - Nguong pause hien Mini-HUD mnemonic khi Armed (khong lay focus)."
        },
        "hotkeys": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "combo": { "type": "string", "minLength": 1 },
              "action": {
                "type": "string",
                "enum": ["cycle_mode", "toggle_sticky", "smart_paste", "palette"]
              }
            }
          },
          "default": [
            { "combo": "Ctrl+Alt+M", "action": "cycle_mode" },
            { "combo": "Ctrl+Shift+M", "action": "toggle_sticky" },
            { "combo": "Ctrl+Alt+V", "action": "smart_paste" },
            { "combo": "Ctrl+Alt+Space", "action": "palette" }
          ],
          "$comment": "P11 - 4 hotkey toan cuc mac dinh. Validator CHAN combo trung OS reserved (Win+*, Alt+Tab, Ctrl+Alt+Del) va Ctrl+A/C/V/X/Z/Y/S."
        },
        "palette_max_results": {
          "type": "integer",
          "minimum": 5,
          "maximum": 100,
          "default": 20,
          "$comment": "P11 - So ket qua toi da cua palette (Dream RSI ranking: prefix + alias + discipline affinity + recency decay)."
        }
      }
    },
    "assist": {'''

s = s.replace(anchor, math_block, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('schema patched')

# validate JSON
import json
d = json.loads(s)
assert 'math' in d['properties']
print('schema JSON valid, math present')
