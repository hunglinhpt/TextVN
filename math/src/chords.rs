// SPDX-License-Identifier: GPL-3.0-or-later
//! Deterministic Leader-Key / Dead-Key / AltGr FSM — zero-alloc, không bao giờ
//! treo trạng thái (timeout / Esc / focus-loss đều về Idle sạch).
//!
//! Priority chain (spec §3.1): OS reserved → protected app shortcuts →
//! TextVN global hotkeys → leader chords → AltGr layer → pass-through.

use crate::config::{HudHint, MathAction, MathConfig, MathOutputMode};
use crate::templates;

/// Trạng thái FSM chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordState {
    Idle,
    /// Leader đã nhấn — đợi phím chord (≤ 1500 ms).
    Armed,
    /// Chain superscript/subscript (UnicodeMath): digits tiếp tục chain.
    SupChain,
    SubChain,
}

/// Ngữ cảnh app để bảo vệ shortcut (spec §3.1) — adapter điền từ appdb.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppGuard {
    /// MathType / MTWindow / Equation OLE đang focus.
    pub is_mathtype: bool,
    /// MS Word Equation Editor đang focus.
    pub is_word_equation: bool,
    /// AltGr layer bị chặn (app đụng AltGr riêng).
    pub altgr_blocked: bool,
}

impl AppGuard {
    /// App này có đang giữ shortcut mà chord/hotkey KHÔNG ĐƯỢC nuốt không.
    pub fn blocks_chords(&self) -> bool {
        self.is_mathtype || self.is_word_equation
    }
}

/// Bảng Hy Lạp thường: (phím, ký tự, latex).
/// KHÔNG chứa các phím spec dành cho symbol: i(∫) o(°) x(×) s? — chỉ Greek thuần.
const GREEK_LOWER: &[(char, &str, &str)] = &[
    ('a', "α", "\\alpha"),
    ('b', "β", "\\beta"),
    ('g', "γ", "\\gamma"),
    ('d', "δ", "\\delta"),
    ('e', "ε", "\\epsilon"),
    ('z', "ζ", "\\zeta"),
    ('h', "η", "\\eta"),
    ('t', "θ", "\\theta"),
    ('k', "κ", "\\kappa"),
    ('l', "λ", "\\lambda"),
    ('m', "µ", "\\mu"),
    ('p', "π", "\\pi"),
    ('r', "ρ", "\\rho"),
    ('s', "σ", "\\sigma"),
    ('f', "φ", "\\phi"),
    ('c', "χ", "\\chi"),
    ('y', "ψ", "\\ψ"),
    ('w', "ω", "\\omega"),
    ('u', "υ", "\\upsilon"),
    ('v', "ν", "\\nu"),
    ('j', "ξ", "\\xi"),
];

/// Bảng Hy Lạp hoa — chỉ phím spec KHÔNG định nghĩa symbol uppercase.
/// (A/E/I/N/O/S bị spec chiếm: ∀ ∃ ∬ ∂ ∅ ∑.)
const GREEK_UPPER: &[(char, &str, &str)] = &[
    ('B', "Β", "B"),
    ('G', "Γ", "\\Gamma"),
    ('D', "Δ", "\\Delta"),
    ('Z', "Ζ", "Z"),
    ('H', "Η", "H"),
    ('T', "Θ", "\\Theta"),
    ('K', "Κ", "Κ"),
    ('L', "Λ", "\\Λambda"),
    ('M', "Μ", "M"),
    ('P', "Π", "\\Pi"),
    ('R', "Ρ", "P"),
    ('U', "Υ", "\\Upsilon"),
    ('F', "Φ", "\\Phi"),
    ('C', "Χ", "X"),
    ('Y', "Ψ", "\\Psi"),
    ('W', "Ω", "\\Omega"),
    ('X', "Ξ", "\\Xi"),
];

/// Chuỗi superscript/subscript digit chain (UnicodeMath).
const SUP_DIGITS: &[char] = &['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
const SUB_DIGITS: &[char] = &['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

/// Zero-alloc FSM chord — 1 instance / engine.
#[derive(Debug, Clone)]
pub struct ChordFsm {
    state: ChordState,
    leader: char,
    mode: MathOutputMode,
    armed_at_ms: u64,
    pause_started_ms: Option<u64>,
    timeout_ms: u64,
    hud_pause_ms: u64,
}

impl ChordFsm {
    pub fn new(config: &MathConfig) -> Self {
        Self {
            state: ChordState::Idle,
            leader: config.leader_key,
            mode: config.output_mode,
            armed_at_ms: 0,
            pause_started_ms: None,
            timeout_ms: config.chord_timeout_ms,
            hud_pause_ms: config.hud_pause_ms,
        }
    }

    pub fn state(&self) -> ChordState {
        self.state
    }

    pub fn mode(&self) -> MathOutputMode {
        self.mode
    }

    /// Đổi mode (Ctrl+Alt+M cycle / ConfigReload).
    pub fn set_mode(&mut self, mode: MathOutputMode) {
        self.mode = mode;
        self.reset();
    }

    /// Reset lặng lẽ về Idle (focus loss / app đổi / cancel ngoài).
    pub fn reset(&mut self) {
        self.state = ChordState::Idle;
        self.armed_at_ms = 0;
        self.pause_started_ms = None;
    }

    /// Leader đã armed quá `hud_pause_ms` chưa — adapter hiện Mini-HUD.
    pub fn hud_due(&self, now_ms: u64) -> Option<HudHint> {
        if self.state == ChordState::Armed {
            if let Some(t0) = self.pause_started_ms {
                if now_ms.saturating_sub(t0) >= self.hud_pause_ms {
                    return Some(hud_chords());
                }
            }
        }
        None
    }

    /// Xử lý MỘT phím printable trong math mode. `None` = math không quan tâm
    /// (adapter/engine cho phím đi thẳng).
    pub fn process(&mut self, c: char, now_ms: u64, guard: &AppGuard) -> Option<MathAction> {
        // Protected app: chord layer NGỪNG hoàn toàn (spec §3.1).
        if guard.blocks_chords() {
            self.reset();
            return None;
        }
        match self.state {
            ChordState::Idle => self.process_idle(c, now_ms),
            ChordState::Armed => self.process_armed(c, now_ms),
            ChordState::SupChain => self.process_chain(c, SUP_DIGITS),
            ChordState::SubChain => self.process_chain(c, SUB_DIGITS),
        }
    }

    fn process_idle(&mut self, c: char, now_ms: u64) -> Option<MathAction> {
        if c == self.leader {
            self.state = ChordState::Armed;
            self.armed_at_ms = now_ms;
            self.pause_started_ms = Some(now_ms);
            return Some(MathAction::Consume);
        }
        None
    }

    fn process_armed(&mut self, c: char, now_ms: u64) -> Option<MathAction> {
        // Timeout: phát literal leader + phím này, về Idle (zero stuck).
        if now_ms.saturating_sub(self.armed_at_ms) > self.timeout_ms {
            self.reset();
            return Some(emit_literal(format!("{}{}", self.leader, c)));
        }
        // Double leader → literal leader.
        if c == self.leader {
            self.reset();
            return Some(emit_literal(self.leader.to_string()));
        }
        // Esc / Space → cancel sạch.
        if c == '\u{1b}' || c == ' ' {
            self.reset();
            return Some(MathAction::Consume);
        }
        self.pause_started_ms = None;
        self.state = ChordState::Idle;

        // 1. Greek (case-sensitive).
        if let Some(found) = lookup_greek(c) {
            let (uni, latex) = found;
            let text = match self.mode {
                MathOutputMode::Latex => latex.to_string(),
                _ => uni.to_string(),
            };
            return Some(emit(text, 0));
        }
        // 2. Symbol / structure chords.
        if let Some(out) = symbol_chord(c, self.mode) {
            // `^` / `_` ở UnicodeMath mở chain.
            if self.mode == MathOutputMode::UnicodeMath && c == '^' {
                self.state = ChordState::SupChain;
            } else if self.mode == MathOutputMode::UnicodeMath && c == '_' {
                self.state = ChordState::SubChain;
            }
            return Some(out);
        }
        // 3. Unmapped → literal leader + key (không bao giờ nuốt input).
        Some(emit_literal(format!("{}{}", self.leader, c)))
    }

    fn process_chain(&mut self, c: char, table: &[char]) -> Option<MathAction> {
        if let Some(d) = c.to_digit(10) {
            let uni = table[d as usize];
            return Some(emit(uni.to_string(), 0)); // chain giữ nguyên
        }
        // Phím không phải digit → thoát chain, xử lý như từ Idle (re-entry).
        self.reset();
        if c == self.leader {
            self.state = ChordState::Armed;
            self.armed_at_ms = 0;
            return Some(MathAction::Consume);
        }
        None
    }

    /// Alias `process_chain` naming helper cho test.
    #[allow(dead_code)]
    fn is_chain(state: ChordState) -> bool {
        matches!(state, ChordState::SupChain | ChordState::SubChain)
    }
}

impl Default for ChordFsm {
    fn default() -> Self {
        Self::new(&MathConfig::default())
    }
}

fn lookup_greek(c: char) -> Option<(&'static str, &'static str)> {
    let mut it = GREEK_LOWER.iter().filter(|(k, _, _)| *k == c);
    if let Some(&(_, uni, latex)) = it.next() {
        return Some((uni, latex));
    }
    GREEK_UPPER
        .iter()
        .find(|(k, _, _)| *k == c)
        .map(|&(_, uni, latex)| (uni, latex))
}

/// Phát text với con trỏ trái `steps` bước (placeholder đầu tiên).
fn emit(text: String, steps: usize) -> MathAction {
    MathAction::Emit {
        text,
        cursor_left_steps: steps,
        show_hud: None,
    }
}

fn emit_literal(text: String) -> MathAction {
    emit(text, 0)
}

/// Symbol/structure chords — mode-specific text + cursor_left_steps đúng spec.
pub fn symbol_chord(c: char, mode: MathOutputMode) -> Option<MathAction> {
    let (text, steps): (String, usize) = match c {
        '/' => match mode {
            MathOutputMode::Latex => ("\\frac{}{}".into(), 3),
            MathOutputMode::Typst => ("frac(, )".into(), 3),
            MathOutputMode::UnicodeMath => ("⁄".into(), 0),
            _ => ("÷".into(), 0),
        },
        'q' => match mode {
            MathOutputMode::Latex => ("\\sqrt{}".into(), 1),
            MathOutputMode::Typst => ("sqrt()".into(), 1),
            _ => ("√".into(), 0),
        },
        '^' => match mode {
            MathOutputMode::Latex => ("^{}".into(), 1),
            MathOutputMode::Typst => ("^()".into(), 1),
            // UnicodeMath: mở chain superscript — phát rỗng (chain lo digit kế).
            MathOutputMode::UnicodeMath => (String::new(), 0),
            _ => ("^".into(), 0),
        },
        '_' => match mode {
            MathOutputMode::Latex => ("_{}".into(), 1),
            MathOutputMode::Typst => ("_()".into(), 1),
            MathOutputMode::UnicodeMath => (String::new(), 0),
            _ => ("_".into(), 0),
        },
        'i' => match mode {
            MathOutputMode::Latex => ("\\int_{}^{}".into(), 4),
            MathOutputMode::Typst => ("integral_(, )^".into(), 6),
            _ => ("∫".into(), 0),
        },
        'I' => match mode {
            MathOutputMode::Latex => ("\\iint_{}^{}".into(), 4),
            MathOutputMode::Typst => ("integral.double_(, )^".into(), 6),
            _ => ("∬".into(), 0),
        },
        'S' => match mode {
            MathOutputMode::Latex => ("\\sum_{}^{}".into(), 4),
            MathOutputMode::Typst => ("sum_(, )^".into(), 6),
            _ => ("∑".into(), 0),
        },
        '8' => sym_by_mode("∞", "\\infty", "infinity", mode),
        '+' => match mode {
            MathOutputMode::Latex => ("\\pm".into(), 0),
            _ => ("±".into(), 0),
        },
        'x' => match mode {
            MathOutputMode::Latex => ("\\times".into(), 0),
            _ => ("×".into(), 0),
        },
        '.' => match mode {
            MathOutputMode::Latex => ("\\cdot".into(), 0),
            _ => ("·".into(), 0),
        },
        '=' => match mode {
            MathOutputMode::Latex => ("\\neq".into(), 0),
            _ => ("≠".into(), 0),
        },
        '~' => match mode {
            MathOutputMode::Latex => ("\\approx".into(), 0),
            _ => ("≈".into(), 0),
        },
        '<' => match mode {
            MathOutputMode::Latex => ("\\leq".into(), 0),
            _ => ("≤".into(), 0),
        },
        '>' => match mode {
            MathOutputMode::Latex => ("\\geq".into(), 0),
            _ => ("≥".into(), 0),
        },
        'o' => match mode {
            MathOutputMode::Latex => ("^\\circ".into(), 0),
            _ => ("°".into(), 0),
        },
        'O' => match mode {
            MathOutputMode::Latex => ("\\emptyset".into(), 0),
            _ => ("∅".into(), 0),
        },
        'n' => match mode {
            MathOutputMode::Latex => ("\\nabla".into(), 0),
            _ => ("∇".into(), 0),
        },
        'N' => match mode {
            MathOutputMode::Latex => ("\\partial".into(), 0),
            _ => ("∂".into(), 0),
        },
        'A' => match mode {
            MathOutputMode::Latex => ("\\forall".into(), 0),
            _ => ("∀".into(), 0),
        },
        'E' => match mode {
            MathOutputMode::Latex => ("\\exists".into(), 0),
            _ => ("∃".into(), 0),
        },
        '!' => match mode {
            MathOutputMode::Latex => ("\\exists!".into(), 0),
            _ => ("∃!".into(), 0),
        },
        _ => return None,
    };
    // UnicodeMath `^`/`_` mở chain — text rỗng là tín hiệu mở chain (không chèn gì).
    if text.is_empty() && mode == MathOutputMode::UnicodeMath {
        return Some(MathAction::Consume);
    }
    Some(emit(text, steps))
}

fn sym_by_mode(uni: &str, latex: &str, typst: &str, mode: MathOutputMode) -> (String, usize) {
    let text = match mode {
        MathOutputMode::Latex => latex,
        MathOutputMode::Typst => typst,
        _ => uni,
    };
    (text.to_string(), 0)
}

/// AltGr layer một-phím (opt-in) — ký tự → ký tự phát thẳng.
/// Conflict validator của config chặn binding đè phím app bảo vệ.
pub fn altgr_chord(c: char) -> Option<MathAction> {
    let uni: &str = match c {
        'a' => "α",
        'p' => "π",
        'm' => "µ",
        'o' => "Ω",
        '=' => "≠",
        '+' => "±",
        '2' => "²",
        '3' => "³",
        'd' => "°",
        'x' => "×",
        '/' => "÷",
        'q' => "√",
        '<' => "≤",
        '>' => "≥",
        '8' => "∞",
        _ => return None,
    };
    Some(emit(uni.to_string(), 0))
}

/// Mini-HUD tĩnh — mnemonic top keys, không lấy focus, không chứa text user.
pub fn hud_chords() -> HudHint {
    HudHint {
        title: "Math chords".into(),
        items: vec![
            ("a".into(), "α".into()),
            ("g".into(), "γ".into()),
            ("p".into(), "π".into()),
            ("S".into(), "∑".into()),
            ("i".into(), "∫".into()),
            ("/".into(), "phân số".into()),
            ("q".into(), "√".into()),
            ("^".into(), "mũ".into()),
        ],
    }
}

/// Mở rộng template động từ word gõ: `\mat3x3`, `\det4`, `\hpt2`, `\cases3`.
pub fn expand_template_word(word: &str, mode: MathOutputMode) -> Option<MathAction> {
    let (kind, n, m) = templates::parse_trigger(word)?;
    let out = templates::generate(kind, n, m, mode)?;
    Some(MathAction::ReplaceBuffer {
        delete_chars: word.chars().count(),
        text: out.text,
        cursor_left_steps: out.cursor_left_steps,
    })
}

/// Mở rộng trigger/alias lexicon: `\alpha` → ký tự/latex theo mode.
pub fn expand_lexicon_word(
    word: &str,
    mode: MathOutputMode,
    custom: impl Fn(&str) -> Option<String>,
) -> Option<MathAction> {
    let w = word.strip_prefix('\\').unwrap_or(word);
    if w.is_empty() || w.len() > 33 {
        return None;
    }
    // 1. custom binding ưu tiên (user-defined).
    if let Some(v) = custom(w) {
        return Some(MathAction::ReplaceBuffer {
            delete_chars: word.chars().count(),
            text: v,
            cursor_left_steps: 0,
        });
    }
    // 2. template động.
    if let Some(action) = expand_template_word(word, mode) {
        return Some(action);
    }
    // 3. lexicon (trigger trước, alias sau — deterministic).
    let entry = crate::lexicon::lookup_token(w)?;
    let text = match mode {
        MathOutputMode::Latex => entry.latex,
        MathOutputMode::Typst => entry.typst,
        MathOutputMode::UnicodeMath => entry.unicode,
        _ => entry.unicode,
    };
    Some(MathAction::ReplaceBuffer {
        delete_chars: word.chars().count(),
        text: text.to_string(),
        cursor_left_steps: 0,
    })
}

/// Ghi nhận tiết kiệm phím Dream RSI: emit − số phím đã tiêu (đã-gõ).
pub fn dream_rsi_delta(action: &MathAction, keys_consumed: usize) -> (bool, u64) {
    let n = action.inserted_text().chars().count();
    let saved = n.saturating_sub(keys_consumed) as u64;
    let accept = matches!(
        action,
        MathAction::Emit { .. } | MathAction::ReplaceBuffer { .. }
    );
    (accept, saved)
}

// ============================================================
// TESTS — mỗi key × mode 1 test (macro sinh), FSM edge cases
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn fsm() -> ChordFsm {
        ChordFsm::new(&MathConfig::default())
    }

    fn armed() -> ChordFsm {
        let mut f = fsm();
        assert_eq!(
            f.process('`', 0, &AppGuard::default()),
            Some(MathAction::Consume)
        );
        f
    }

    fn text_of(f: &mut ChordFsm, c: char, ms: u64) -> String {
        match f.process(c, ms, &AppGuard::default()) {
            Some(MathAction::Emit { text, .. }) | Some(MathAction::ReplaceBuffer { text, .. }) => {
                text
            }
            Some(MathAction::Consume) | Some(MathAction::PassThrough) => String::new(),
            None => String::new(),
        }
    }

    #[test]
    fn leader_arms_then_greek() {
        let mut f = armed();
        let t = text_of(&mut f, 'a', 100);
        assert_eq!(t, "α");
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn greek_upper_sigma() {
        let mut f = armed();
        assert_eq!(text_of(&mut f, 'S', 100), "∑"); // S là symbol chord
        let mut f = armed();
        // W = Ω
        assert_eq!(text_of(&mut f, 'W', 100), "Ω");
    }

    #[test]
    fn greek_case_sensitive() {
        // 'p' → π (chord symbol bảng lower); 'P' → Π hoa
        let mut f = armed();
        assert_eq!(text_of(&mut f, 'p', 100), "π");
        let mut f = armed();
        assert_eq!(text_of(&mut f, 'P', 100), "Π");
    }

    #[test]
    fn double_leader_literal() {
        let mut f = armed();
        assert_eq!(text_of(&mut f, '`', 100), "`");
    }

    #[test]
    fn esc_and_space_cancel() {
        let mut f = armed();
        assert_eq!(
            f.process('\u{1b}', 100, &AppGuard::default()),
            Some(MathAction::Consume)
        );
        assert_eq!(f.state(), ChordState::Idle);
        let mut f = armed();
        assert_eq!(
            f.process(' ', 100, &AppGuard::default()),
            Some(MathAction::Consume)
        );
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn unmapped_key_fallback() {
        let mut f = armed();
        assert_eq!(text_of(&mut f, '1', 100), "`1");
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn timeout_emits_literal_and_resets() {
        let mut f = armed();
        assert_eq!(text_of(&mut f, 'a', 2000), "`a");
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn timeout_boundary_exactly() {
        let mut f = armed();
        // 1500 ms = đúng timeout — không quá hạn
        assert_eq!(text_of(&mut f, 'a', 1500), "α");
    }

    #[test]
    fn focus_loss_resets_clean() {
        let mut f = armed();
        f.reset();
        assert_eq!(f.state(), ChordState::Idle);
        // Sau reset, phím kế không emit gì lạ
        assert_eq!(f.process('a', 200, &AppGuard::default()), None);
    }

    #[test]
    fn frac_cursor_latex() {
        let mut f = fsm();
        f.set_mode(MathOutputMode::Latex);
        f.process('`', 0, &AppGuard::default());
        match f.process('/', 100, &AppGuard::default()) {
            Some(MathAction::Emit {
                text,
                cursor_left_steps,
                ..
            }) => {
                assert_eq!(text, "\\frac{}{}");
                assert_eq!(cursor_left_steps, 3);
            }
            other => panic!("expected Emit, got {other:?}"),
        }
    }

    #[test]
    fn sqrt_typst() {
        let mut f = fsm();
        f.set_mode(MathOutputMode::Typst);
        f.process('`', 0, &AppGuard::default());
        match f.process('q', 100, &AppGuard::default()) {
            Some(MathAction::Emit {
                text,
                cursor_left_steps,
                ..
            }) => {
                assert_eq!(text, "sqrt()");
                assert_eq!(cursor_left_steps, 1);
            }
            other => panic!("expected Emit, got {other:?}"),
        }
    }

    #[test]
    fn unicode_chain_superscript() {
        let mut f = fsm();
        f.set_mode(MathOutputMode::UnicodeMath);
        f.process('`', 0, &AppGuard::default());
        assert_eq!(
            f.process('^', 100, &AppGuard::default()),
            Some(MathAction::Consume)
        );
        assert_eq!(f.state(), ChordState::SupChain);
        let t = text_of(&mut f, '2', 150);
        assert_eq!(t, "²");
        assert_eq!(f.state(), ChordState::SupChain, "digit giữ chain");
        let t = text_of(&mut f, '5', 150);
        assert_eq!(t, "⁵");
        // Non-digit thoát chain
        assert_eq!(f.process('x', 150, &AppGuard::default()), None);
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn unicode_chain_subscript_and_rearm() {
        let mut f = fsm();
        f.set_mode(MathOutputMode::UnicodeMath);
        f.process('`', 0, &AppGuard::default());
        f.process('_', 100, &AppGuard::default());
        assert_eq!(text_of(&mut f, '3', 150), "₃");
        // chain → leader lại re-arm được
        assert_eq!(
            f.process('`', 150, &AppGuard::default()),
            Some(MathAction::Consume)
        );
        assert_eq!(f.state(), ChordState::Armed);
    }

    #[test]
    fn latex_mode_no_chain() {
        let mut f = fsm();
        f.set_mode(MathOutputMode::Latex);
        f.process('`', 0, &AppGuard::default());
        match f.process('^', 100, &AppGuard::default()) {
            Some(MathAction::Emit {
                text,
                cursor_left_steps,
                ..
            }) => {
                assert_eq!(text, "^{}");
                assert_eq!(cursor_left_steps, 1);
            }
            other => panic!("got {other:?}"),
        }
        assert_eq!(f.state(), ChordState::Idle);
    }

    #[test]
    fn protected_app_blocks_everything() {
        let mut f = armed();
        let guard = AppGuard {
            is_mathtype: true,
            ..AppGuard::default()
        };
        assert_eq!(f.process('a', 100, &guard), None, "MathType: không nuốt gì");
        assert_eq!(f.state(), ChordState::Idle);
        // Từ Idle cũng không arm
        assert_eq!(f.process('`', 100, &guard), None);
    }

    #[test]
    fn word_equation_blocks_chords() {
        let mut f = fsm();
        let guard = AppGuard {
            is_word_equation: true,
            ..AppGuard::default()
        };
        assert_eq!(f.process('`', 0, &guard), None);
    }

    #[test]
    fn all_symbol_chords_three_modes() {
        // Mỗi phím phải emit KHÁC RỖNG ở cả 3 mode (trừ chain mở)
        for c in [
            '/', 'q', 'i', 'I', 'S', '8', '+', 'x', '.', '=', '~', '<', '>', 'o', 'O', 'n', 'N',
            'A', 'E',
        ] {
            for mode in [
                MathOutputMode::Normal,
                MathOutputMode::Latex,
                MathOutputMode::Typst,
            ] {
                let action = symbol_chord(c, mode)
                    .unwrap_or_else(|| panic!("chord {c:?} mode {mode:?} thiếu"));
                let text = action.inserted_text();
                assert!(!text.is_empty(), "chord {c:?} mode {mode:?} text rỗng");
            }
        }
    }

    #[test]
    fn chord_symbol_unicode_values() {
        assert_eq!(
            symbol_chord('8', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∞"
        );
        assert_eq!(
            symbol_chord('+', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "±"
        );
        assert_eq!(
            symbol_chord('x', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "×"
        );
        assert_eq!(
            symbol_chord('.', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "·"
        );
        assert_eq!(
            symbol_chord('=', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "≠"
        );
        assert_eq!(
            symbol_chord('~', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "≈"
        );
        assert_eq!(
            symbol_chord('<', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "≤"
        );
        assert_eq!(
            symbol_chord('>', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "≥"
        );
        assert_eq!(
            symbol_chord('o', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "°"
        );
        assert_eq!(
            symbol_chord('O', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∅"
        );
        assert_eq!(
            symbol_chord('n', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∇"
        );
        assert_eq!(
            symbol_chord('N', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∂"
        );
        assert_eq!(
            symbol_chord('A', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∀"
        );
        assert_eq!(
            symbol_chord('E', MathOutputMode::Normal)
                .unwrap()
                .inserted_text(),
            "∃"
        );
    }

    #[test]
    fn hud_due_after_pause() {
        let mut f = fsm();
        assert!(f.hud_due(500).is_none());
        f.process('`', 0, &AppGuard::default());
        assert!(f.hud_due(100).is_none(), "chưa quá 200 ms");
        assert!(f.hud_due(300).is_some(), "quá 200 ms pause → HUD");
        let hud = f.hud_due(300).unwrap();
        assert!(!hud.items.is_empty());
        assert!(!hud.title.is_empty());
    }

    #[test]
    fn altgr_layer_full() {
        for (k, want) in [
            ('a', "α"),
            ('p', "π"),
            ('m', "µ"),
            ('o', "Ω"),
            ('=', "≠"),
            ('+', "±"),
            ('2', "²"),
            ('3', "³"),
        ] {
            let action = altgr_chord(k).unwrap_or_else(|| panic!("altgr {k:?} thiếu"));
            assert_eq!(action.inserted_text(), want);
        }
        assert!(altgr_chord('z').is_none(), "phím không map → None");
    }

    #[test]
    fn expand_lexicon_alias_vn() {
        let action = expand_lexicon_word("\\tong", MathOutputMode::Normal, |_| None).unwrap();
        assert_eq!(action.inserted_text(), "∑");
        match action {
            MathAction::ReplaceBuffer { delete_chars, .. } => assert_eq!(delete_chars, 5),
            _ => panic!(),
        }
    }

    #[test]
    fn expand_lexicon_template_dynamic() {
        let action = expand_lexicon_word("\\mat3x3", MathOutputMode::Latex, |_| None).unwrap();
        let t = action.inserted_text();
        assert!(t.contains("\\begin{matrix}"));
        assert!(t.contains("a_{33}"));
    }

    #[test]
    fn expand_lexicon_custom_first() {
        let action = expand_lexicon_word("\\alpha", MathOutputMode::Normal, |w| {
            if w == "alpha" {
                Some("A-hero".into())
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(action.inserted_text(), "A-hero");
    }

    #[test]
    fn expand_lexicon_unknown_none() {
        assert!(expand_lexicon_word("\\zzz_khongco", MathOutputMode::Normal, |_| None).is_none());
        assert!(expand_lexicon_word("\\", MathOutputMode::Normal, |_| None).is_none());
    }

    #[test]
    fn expand_lexicon_mode_outputs() {
        let action_l = expand_lexicon_word("\\alpha", MathOutputMode::Latex, |_| None).unwrap();
        assert_eq!(action_l.inserted_text(), "\\alpha");
        let action_t = expand_lexicon_word("\\alpha", MathOutputMode::Typst, |_| None).unwrap();
        assert_eq!(action_t.inserted_text(), "alpha");
    }

    #[test]
    fn dream_rsi_delta_counts() {
        let a = MathAction::ReplaceBuffer {
            delete_chars: 5,
            text: "∑".into(),
            cursor_left_steps: 0,
        };
        let (accept, saved) = dream_rsi_delta(&a, 5);
        assert!(accept);
        assert_eq!(saved, 0, "∑ = 1 ký tự, 5 phím → 0 tiết kiệm");
        let b = MathAction::ReplaceBuffer {
            delete_chars: 7,
            text: "\\begin{matrix} a_{11} & a_{12} \\\\ a_{21} & a_{22} \\end{matrix}".into(),
            cursor_left_steps: 0,
        };
        let (_, saved) = dream_rsi_delta(&b, 7);
        assert!(saved > 40, "template dài phải tiết kiệm lớn");
        let c = MathAction::Consume;
        assert!(!dream_rsi_delta(&c, 1).0, "Consume không phải accept");
    }

    // ==== Per-key × mode macro tests (mỗi chord một test thật) ====
    macro_rules! chord_case {
        ($name:ident, $key:literal, $mode:expr, $want:literal) => {
            #[test]
            fn $name() {
                let action = symbol_chord($key, $mode).unwrap_or_else(|| panic!("missing chord"));
                assert_eq!(action.inserted_text(), $want);
            }
        };
    }
    chord_case!(c_frac_latex, '/', MathOutputMode::Latex, "\\frac{}{}");
    chord_case!(c_frac_typst, '/', MathOutputMode::Typst, "frac(, )");
    chord_case!(c_frac_unicode, '/', MathOutputMode::UnicodeMath, "⁄");
    chord_case!(c_frac_normal, '/', MathOutputMode::Normal, "÷");
    chord_case!(c_sqrt_latex, 'q', MathOutputMode::Latex, "\\sqrt{}");
    chord_case!(c_sqrt_typst, 'q', MathOutputMode::Typst, "sqrt()");
    chord_case!(c_sqrt_normal, 'q', MathOutputMode::Normal, "√");
    chord_case!(c_int_latex, 'i', MathOutputMode::Latex, "\\int_{}^{}");
    chord_case!(c_int_typst, 'i', MathOutputMode::Typst, "integral_(, )^");
    chord_case!(c_int_normal, 'i', MathOutputMode::Normal, "∫");
    chord_case!(c_iint_latex, 'I', MathOutputMode::Latex, "\\iint_{}^{}");
    chord_case!(c_iint_normal, 'I', MathOutputMode::Normal, "∬");
    chord_case!(c_sum_latex, 'S', MathOutputMode::Latex, "\\sum_{}^{}");
    chord_case!(c_sum_typst, 'S', MathOutputMode::Typst, "sum_(, )^");
    chord_case!(c_sum_normal, 'S', MathOutputMode::Normal, "∑");
    chord_case!(c_inf_latex, '8', MathOutputMode::Latex, "\\infty");
    chord_case!(c_inf_typst, '8', MathOutputMode::Typst, "infinity");
    chord_case!(c_inf_normal, '8', MathOutputMode::Normal, "∞");
    chord_case!(c_pm_latex, '+', MathOutputMode::Latex, "\\pm");
    chord_case!(c_times_latex, 'x', MathOutputMode::Latex, "\\times");
    chord_case!(c_cdot_latex, '.', MathOutputMode::Latex, "\\cdot");
    chord_case!(c_neq_latex, '=', MathOutputMode::Latex, "\\neq");
    chord_case!(c_approx_latex, '~', MathOutputMode::Latex, "\\approx");
    chord_case!(c_leq_latex, '<', MathOutputMode::Latex, "\\leq");
    chord_case!(c_geq_latex, '>', MathOutputMode::Latex, "\\geq");
    chord_case!(c_degree_latex, 'o', MathOutputMode::Latex, "^\\circ");
    chord_case!(c_emptyset_latex, 'O', MathOutputMode::Latex, "\\emptyset");
    chord_case!(c_nabla_latex, 'n', MathOutputMode::Latex, "\\nabla");
    chord_case!(c_partial_latex, 'N', MathOutputMode::Latex, "\\partial");
    chord_case!(c_forall_latex, 'A', MathOutputMode::Latex, "\\forall");
    chord_case!(c_exists_latex, 'E', MathOutputMode::Latex, "\\exists");
    chord_case!(c_neq_normal, '=', MathOutputMode::Normal, "≠");
    chord_case!(c_degree_normal, 'o', MathOutputMode::Normal, "°");
    chord_case!(c_emptyset_normal, 'O', MathOutputMode::Normal, "∅");

    // ==== Per-Greek-key macro tests (mỗi phím Hy Lạp một test) ====
    macro_rules! greek_case {
        ($name:ident, $key:literal, $want:literal) => {
            #[test]
            #[allow(non_snake_case)] // tên test theo phím Hy Lạp (gk_W, gk_Su...)
            fn $name() {
                let mut f = armed();
                assert_eq!(text_of(&mut f, $key, 100), $want);
            }
        };
    }
    greek_case!(gk_a, 'a', "α");
    greek_case!(gk_b, 'b', "β");
    greek_case!(gk_g, 'g', "γ");
    greek_case!(gk_d, 'd', "δ");
    greek_case!(gk_e, 'e', "ε");
    greek_case!(gk_z, 'z', "ζ");
    greek_case!(gk_h, 'h', "η");
    greek_case!(gk_t, 't', "θ");
    greek_case!(gk_k, 'k', "κ");
    greek_case!(gk_l, 'l', "λ");
    greek_case!(gk_m, 'm', "µ");
    greek_case!(gk_p, 'p', "π");
    greek_case!(gk_r, 'r', "ρ");
    greek_case!(gk_s, 's', "σ");
    greek_case!(gk_f, 'f', "φ");
    greek_case!(gk_c, 'c', "χ");
    greek_case!(gk_y, 'y', "ψ");
    greek_case!(gk_w, 'w', "ω");
    greek_case!(gk_W, 'W', "Ω");
    greek_case!(gk_G, 'G', "Γ");
    greek_case!(gk_D, 'D', "Δ");
    greek_case!(gk_T, 'T', "Θ");
    greek_case!(gk_L, 'L', "Λ");
    greek_case!(gk_P, 'P', "Π");
    greek_case!(gk_Su, 'S', "∑"); // S symbol chord ưu tiên
}
