// SPDX-License-Identifier: GPL-3.0-or-later
//! Smart Clipboard — bounded latest-wins worker, in-memory only, zero persistence

pub mod classifier;
pub mod serializer;

#[cfg(windows)]
pub mod windows;

use std::sync::Mutex;

/// Maximum clipboard payload size (64 KB)
pub const MAX_PAYLOAD: usize = 64 * 1024;

/// Cached formula store — in-memory only, never written to disk
pub struct CachedFormulaStore {
    /// Latest detected formula (latest-wins coalescing)
    pub latest: Mutex<Option<CachedFormula>>,
}

#[derive(Debug, Clone)]
pub struct CachedFormula {
    pub source_text: String,
    pub confidence: f32,
    pub mathml: String,
    pub latex: String,
    pub unicode: String,
}

impl Default for CachedFormulaStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedFormulaStore {
    pub fn new() -> Self {
        Self {
            latest: Mutex::new(None),
        }
    }

    /// Store latest formula (latest-wins)
    pub fn store(&self, formula: CachedFormula) {
        if let Ok(mut guard) = self.latest.lock() {
            *guard = Some(formula);
        }
    }

    /// Get latest formula
    pub fn get(&self) -> Option<CachedFormula> {
        self.latest.lock().ok()?.clone()
    }

    /// Clear cache
    pub fn clear(&self) {
        if let Ok(mut guard) = self.latest.lock() {
            *guard = None;
        }
    }
}

/// Clipboard watcher trait (platform-specific impls)
pub trait ClipboardWatcher {
    /// Start watching clipboard changes
    fn start(&self);
    /// Stop watching
    fn stop(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_get() {
        let store = CachedFormulaStore::new();
        assert!(store.get().is_none());

        store.store(CachedFormula {
            source_text: "x^2".into(),
            confidence: 0.95,
            mathml: "<math/>".into(),
            latex: "x^2".into(),
            unicode: "x²".into(),
        });

        let got = store.get().unwrap();
        assert_eq!(got.source_text, "x^2");
        assert_eq!(got.confidence, 0.95);
    }

    #[test]
    fn latest_wins() {
        let store = CachedFormulaStore::new();
        store.store(CachedFormula {
            source_text: "first".into(),
            confidence: 0.9,
            mathml: String::new(),
            latex: String::new(),
            unicode: String::new(),
        });
        store.store(CachedFormula {
            source_text: "second".into(),
            confidence: 0.95,
            mathml: String::new(),
            latex: String::new(),
            unicode: String::new(),
        });
        assert_eq!(store.get().unwrap().source_text, "second");
    }

    #[test]
    fn clear_works() {
        let store = CachedFormulaStore::new();
        store.store(CachedFormula {
            source_text: "x".into(),
            confidence: 1.0,
            mathml: String::new(),
            latex: String::new(),
            unicode: String::new(),
        });
        store.clear();
        assert!(store.get().is_none());
    }
}
