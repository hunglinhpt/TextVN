import io, re

BS = chr(92)

# ============ core/Cargo.toml: thêm textvn-math ============
p = 'core/Cargo.toml'
s = io.open(p, encoding='utf-8').read()
old = '[dependencies]' + chr(10) + '# Alias nội bộ — package vẫn đúng tên chuẩn `textvn-strategy` (00-INDEX §4)' + chr(10) + 'strategy = { package = "textvn-strategy", path = "../strategy" }'
new = old + chr(10) + '# Math engine (P11) — thuần logic, cùng không-IO bất biến với core.' + chr(10) + 'textvn-math = { path = "../math" }'
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('core/Cargo.toml ok')

# ============ core/src/lib.rs ============
p = 'core/src/lib.rs'
s = io.open(p, encoding='utf-8').read()

# 1. Imports — sau use block đầu
old = 'use std::sync::Arc;'
new = ('use std::sync::Arc;' + chr(10) + chr(10)
       + 'use textvn_math::{MathAction as MathActionKind, MathConfig, MathEngine, MathOutputMode};')
assert old in s
s = s.replace(old, new, 1)

# 2. Outcome: thêm cursor_left sau pub flags
old = '    pub flags: u32,' + chr(10) + '}'
new = '    pub flags: u32,' + chr(10) + '    /// Math (P11): số bước mũi tên trái adapter cần bơm sau khi áp hành động' + chr(10) + '    /// (TSF SetSelection hoặc synthetic VK_LEFT). 0 = không đụng con trỏ.' + chr(10) + '    pub cursor_left: u16,' + chr(10) + '}'
assert old in s
s = s.replace(old, new, 1)

# 3. Mọi Outcome literal: thêm cursor_left: 0 sau dòng flags: (skip pub flags def)
lines = s.split(chr(10))
out_lines = []
for ln in lines:
    out_lines.append(ln)
    stripped = ln.strip()
    if stripped.startswith('flags: ') and stripped.endswith(','):
        indent = ln[:len(ln) - len(ln.lstrip())]
        out_lines.append(indent + 'cursor_left: 0,')
s = chr(10).join(out_lines)

# 4. EngineOptions: thêm field math + default
old = '''    /// P7: master-switch HỌC (riêng với dùng-dữ-đã-config) — ffi gate theo
    /// `style_profile.enabled`. Mặc định false (G8).
    pub style_learning_enabled: bool,
}'''
new = '''    /// P7: master-switch HỌC (riêng với dùng-dữ-đã-config) — ffi gate theo
    /// `style_profile.enabled`. Mặc định false (G8).
    pub style_learning_enabled: bool,
    /// P11 — cấu hình Math Engine (`config.math`). Mặc định TẮT: hành vi
    /// gõ cũ giữ nguyên 100% khi user không bật.
    pub math: MathConfig,
}'''
assert old in s
s = s.replace(old, new, 1)

old = '            style_learning_enabled: false,' + chr(10) + '        }' + chr(10) + '    }' + chr(10) + '}'
new = '            style_learning_enabled: false,' + chr(10) + '            math: MathConfig::default(),' + chr(10) + '        }' + chr(10) + '    }' + chr(10) + '}'
assert old in s
s = s.replace(old, new, 1)

# 5. Context: thêm app_id
old = '''    /// Strategy adapter gợi ý; -1 = engine tự resolve.
    pub hint: i64,
}'''
new = '''    /// Strategy adapter gợi ý; -1 = engine tự resolve.
    pub hint: i64,
    /// P11 — exe/window-class của app đang focus (math auto-bypass + smart paste).
    /// Rỗng khi adapter không điền — math dùng profile unknown (Unicode).
    pub app_id: String,
}'''
assert old in s
s = s.replace(old, new, 1)

old = '''            hint: -1,
        }
    }
}'''
new = '''            hint: -1,
            app_id: String::new(),
        }
    }
}'''
assert old in s
s = s.replace(old, new, 1)

# 6. Engine struct: thêm fields
old = '''    last_predict: Option<PredictCycle>,'''
new = '''    /// P11 — Math Engine (chord/alias/eval/bypass). Mọi logic thuần ở textvn-math.
    math: MathEngine,
    /// P11 — profile app hiện tại (set_context điền; resolve AutoByApp + guard).
    math_profile: textvn_math::AppProfile,
    last_predict: Option<PredictCycle>,'''
assert old in s
s = s.replace(old, new, 1)

old = '''            last_predict: None,
            ctrl_tap_armed: false,
        }
    }'''
new = '''            last_predict: None,
            ctrl_tap_armed: false,
            math: MathEngine::new(opts.math.clone()),
            math_profile: textvn_math::AppProfile::unknown(),
        }
    }'''
assert old in s
s = s.replace(old, new, 1)

# 7. set_options: đồng bộ math config
old = '''        self.style_touches = 0;
        self.word.clear();
        self.record(Tail::Clear);
    }

    pub fn options(&self) -> &EngineOptions {'''
new = '''        self.style_touches = 0;
        // P11: math config đổi mới reset FSM math (giữ nguyên nếu chỉ đổi option khác).
        if self.opts.math != self.math.config {
            self.math.set_config(self.opts.math.clone());
        }
        self.word.clear();
        self.record(Tail::Clear);
    }

    pub fn options(&self) -> &EngineOptions {'''
assert old in s
s = s.replace(old, new, 1)

# 8. set_context: điền math profile
old = '''    pub fn set_context(&mut self, ctx: Context) {
        self.ctx = ctx;
    }'''
new = '''    pub fn set_context(&mut self, ctx: Context) {
        // P11: app đổi → profile math mới + reset FSM (zero stuck-chord khi chuyển app).
        if ctx.app_id != self.ctx.app_id {
            self.math_profile = if ctx.app_id.is_empty() {
                textvn_math::AppProfile::unknown()
            } else {
                textvn_math::app_bridge::resolve_app(&ctx.app_id)
            };
            self.math.reset_all();
        }
        self.ctx = ctx;
    }'''
assert old in s
s = s.replace(old, new, 1)

# 9. reset(): reset math FSM
old = '''    pub fn reset(&mut self) {
        // Đổi focus/click sau khi nhận ứng viên = giữ ứng viên đó.
        self.settle_predict(true);
        self.bracket_word = None;
        self.word.clear();
        self.record(Tail::Clear);
    }'''
new = '''    pub fn reset(&mut self) {
        // Đổi focus/click sau khi nhận ứng viên = giữ ứng viên đó.
        self.settle_predict(true);
        self.bracket_word = None;
        self.word.clear();
        self.math.reset_all();
        self.record(Tail::Clear);
    }'''
assert old in s
s = s.replace(old, new, 1)

# 10. reset_with_tail: feed context math từ tail document
i = s.find('pub fn reset_with_tail')
assert i > 0
j = s.find('pub fn glyph_len_of')
seg = s[i:j]
# chèn feed ở cuối hàm — tìm dòng return cuối của hàm (dòng '    }' trước glyph_len_of)
assert 'fn reset_with_tail' in seg
old_seg_tail = seg.rstrip()
# tìm dòng cuối cùng có nội dung trước closing brace
idx = old_seg_tail.rfind(chr(10) + '    }')
assert idx > 0
new_seg_tail = (old_seg_tail[:idx] + chr(10)
                + '        // P11: dựng lại ngữ cảnh math từ tail document ($...$, \\text{...} —bounded).' + chr(10)
                + '        if self.opts.math.enabled {' + chr(10)
                + '            self.math.context_fsm.feed_text(&unicode_tail.chars().rev().take(256).collect::<Vec<_>>().into_iter().rev().collect::<String>());' + chr(10)
                + '        }' + chr(10)
                + old_seg_tail[idx:])
s = s[:i] + new_seg_tail + s[j:]

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('core lib.rs patch 1 done')
