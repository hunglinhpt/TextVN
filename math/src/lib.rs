// SPDX-License-Identifier: GPL-3.0-or-later
//! TextVN Math Engine — Universal Scientific, Engineering & Math Engine

pub mod config;
pub mod context;
pub mod chords;
pub mod lexicon;
pub mod templates;
pub mod ast;
pub mod converter;
pub mod eval;
pub mod clipboard;
pub mod app_bridge;

pub use app_bridge::{AppProfile, AppFormatKind, resolve_output_mode};
pub use config::{MathConfig, MathOutputMode, MathAction, Discipline, HudHint};
pub use chords::{ChordFsm, ChordState, AppGuard};
pub use context::{MathContextFsm, SpanKind};

/// Public facade orchestrator
pub struct MathEngine {
    pub config: MathConfig,
    pub chord_fsm: ChordFsm,
    pub context_fsm: MathContextFsm,
    pub guard: AppGuard,
    pub profile: AppProfile,
}

impl MathEngine {
    pub fn new(config: MathConfig) -> Self {
        Self {
            chord_fsm: ChordFsm::new(&config),
            context_fsm: MathContextFsm::new(),
            guard: AppGuard::default(),
            profile: AppProfile::default(),
            config,
        }
    }

    /// Process a character key through the math engine
    pub fn process_char(&mut self, c: char, now_ms: u64, guard: &AppGuard) -> Option<MathAction> {
        self.chord_fsm.process(c, now_ms, guard)
    }

    /// Process a boundary character (space, enter, etc.)
    pub fn process_boundary(&mut self, c: char, mode: MathOutputMode, guard: &AppGuard) -> Option<MathAction> {
        // Check for inline eval (= expr =)
        // Stub: return None for now
        let _ = (c, mode, guard);
        None
    }

    /// Feed context character (for delimiter tracking)
    pub fn feed_context(&mut self, c: char) {
        self.context_fsm.feed_text_char(c);
    }

    /// Check if current word is a math command (starts with \)
    pub fn word_is_command(&self) -> bool {
        false // stub
    }

    /// Set new config
    pub fn set_config(&mut self, config: MathConfig) {
        self.config = config;
    }

    /// Reset all state
    pub fn reset_all(&mut self) {
        self.chord_fsm.reset();
        self.context_fsm = MathContextFsm::new();
    }
}
