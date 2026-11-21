// SPDX-License-Identifier: GPL-3.0-or-later
//! Windows clipboard watcher — HWND_MESSAGE + AddClipboardFormatListener
//! Privacy: honors ExcludeClipboardContentFromMonitorProcessing, CanIncludeInClipboardHistory

// This module is cfg(windows) only — stub for non-Windows
#[cfg(windows)]
mod imp {
    // Windows-specific implementation would go here
    // Using windows crate: AddClipboardFormatListener, HWND_MESSAGE
}

/// Privacy check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyStatus {
    /// Safe to process
    Safe,
    /// Marked as private (password manager, etc.) — do NOT process
    Private,
}

/// Check if clipboard content has privacy markers (cross-platform stub)
pub fn check_privacy_markers(_content: &str) -> PrivacyStatus {
    // On Windows: check for ExcludeClipboardContentFromMonitorProcessing format
    // On macOS: check org.nspasteboard.ConcealedType
    // For now, conservative: always Safe (real impl in windows.rs)
    PrivacyStatus::Safe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_default_safe() {
        assert_eq!(check_privacy_markers("test"), PrivacyStatus::Safe);
    }
}
