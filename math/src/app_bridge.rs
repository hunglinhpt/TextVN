// SPDX-License-Identifier: GPL-3.0-or-later
//! Target app/window-class resolver + Dream RSI hooks
//! Determines best output format for currently focused app

use crate::config::MathOutputMode;

/// Target application category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCategory {
    /// Word / MathType — native MathML
    OfficeEquation,
    /// Code editors, Overleaf, Obsidian, TeXstudio — LaTeX/Typst
    CodeEditor,
    /// Chat apps, PowerPoint, Slack, Chrome Docs — Unicode
    ChatPresentation,
    /// Terminal / unknown — plain text
    Terminal,
    /// Unknown
    Unknown,
}

/// Resolve app category from process name / window class
pub fn resolve_app(process_name: &str, window_class: &str, window_title: &str) -> AppCategory {
    let proc = process_name.to_ascii_lowercase();
    let class = window_class.to_ascii_lowercase();
    let title = window_title.to_ascii_lowercase();

    // Office / MathType
    if proc.contains("winword")
        || proc.contains("mathtype")
        || class.contains("mtequation")
        || class.contains("mtewindow")
        || class.contains("eqnedt32")
    {
        return AppCategory::OfficeEquation;
    }

    // Code editors / LaTeX
    if proc.contains("code")
        || proc.contains("obsidian")
        || proc.contains("texstudio")
        || proc.contains("notepad++")
        || title.contains("overleaf")
    {
        return AppCategory::CodeEditor;
    }

    // Chat / presentation
    if proc.contains("zalo")
        || proc.contains("teams")
        || proc.contains("powerpnt")
        || proc.contains("slack")
        || proc.contains("discord")
        || title.contains("google docs")
    {
        return AppCategory::ChatPresentation;
    }

    // Terminal
    if proc.contains("cmd")
        || proc.contains("powershell")
        || proc.contains("terminal")
        || proc.contains("wt.exe")
        || class.contains("consolewindowclass")
    {
        return AppCategory::Terminal;
    }

    AppCategory::Unknown
}

/// Pick best output mode for target app
pub fn best_mode_for_app(app: AppCategory, preferred: MathOutputMode) -> MathOutputMode {
    match app {
        AppCategory::OfficeEquation => MathOutputMode::Normal, // MathML via clipboard
        AppCategory::CodeEditor => MathOutputMode::Latex,
        AppCategory::ChatPresentation => MathOutputMode::UnicodeMath,
        AppCategory::Terminal => MathOutputMode::Normal,
        AppCategory::Unknown => preferred,
    }
}

/// Protected shortcuts that must NEVER be intercepted in certain apps
pub fn is_protected_shortcut(app: AppCategory, ctrl: bool, alt: bool, key: char) -> bool {
    // Global: never hijack Ctrl+A/C/V/X/Z/Y/S
    if ctrl && !alt {
        let k = key.to_ascii_lowercase();
        if matches!(k, 'a' | 'c' | 'v' | 'x' | 'z' | 'y' | 's') {
            return true;
        }
    }

    // MathType / Word equation: never intercept native shortcuts
    if app == AppCategory::OfficeEquation {
        if ctrl && matches!(key.to_ascii_lowercase(), 'f' | 'r' | 'h' | 'l' | 'g' | 'i') {
            return true;
        }
        if alt && (key == '\\' || key == '=') {
            return true;
        }
    }

    false
}

/// Dream RSI ranking — FinalScore formula
/// FinalScore = PrefixMatchScore + ExactAliasBonus + DisciplineAffinityBoost + RecencyDecayedPersonalWeight
pub fn dream_rsi_score(
    prefix_match: f32,
    exact_alias: bool,
    discipline_boost: f32,
    recency_weight: f32,
) -> f32 {
    let exact_bonus = if exact_alias { 10.0 } else { 0.0 };
    prefix_match + exact_bonus + discipline_boost + recency_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_word() {
        assert_eq!(
            resolve_app("WINWORD.EXE", "", ""),
            AppCategory::OfficeEquation
        );
        assert_eq!(
            resolve_app("mathtype.exe", "", ""),
            AppCategory::OfficeEquation
        );
        assert_eq!(
            resolve_app("", "MTEquationWindow", ""),
            AppCategory::OfficeEquation
        );
    }

    #[test]
    fn resolve_code() {
        assert_eq!(resolve_app("Code.exe", "", ""), AppCategory::CodeEditor);
        assert_eq!(
            resolve_app("", "", "Overleaf | project"),
            AppCategory::CodeEditor
        );
        assert_eq!(resolve_app("obsidian.exe", "", ""), AppCategory::CodeEditor);
    }

    #[test]
    fn resolve_chat() {
        assert_eq!(
            resolve_app("Zalo.exe", "", ""),
            AppCategory::ChatPresentation
        );
        assert_eq!(
            resolve_app("ms-teams.exe", "", ""),
            AppCategory::ChatPresentation
        );
        assert_eq!(
            resolve_app("POWERPNT.EXE", "", ""),
            AppCategory::ChatPresentation
        );
    }

    #[test]
    fn protected_shortcuts() {
        // Ctrl+C always protected
        assert!(is_protected_shortcut(
            AppCategory::Unknown,
            true,
            false,
            'c'
        ));
        assert!(is_protected_shortcut(
            AppCategory::Unknown,
            true,
            false,
            'v'
        ));
        // MathType: Ctrl+F protected
        assert!(is_protected_shortcut(
            AppCategory::OfficeEquation,
            true,
            false,
            'f'
        ));
        // Alt+= in Word equation
        assert!(is_protected_shortcut(
            AppCategory::OfficeEquation,
            false,
            true,
            '='
        ));
    }

    #[test]
    fn best_mode() {
        assert_eq!(
            best_mode_for_app(AppCategory::OfficeEquation, MathOutputMode::Latex),
            MathOutputMode::Normal
        );
        assert_eq!(
            best_mode_for_app(AppCategory::CodeEditor, MathOutputMode::Normal),
            MathOutputMode::Latex
        );
        assert_eq!(
            best_mode_for_app(AppCategory::ChatPresentation, MathOutputMode::Latex),
            MathOutputMode::UnicodeMath
        );
    }

    #[test]
    fn dream_rsi_formula() {
        let score = dream_rsi_score(5.0, true, 2.0, 3.0);
        assert_eq!(score, 20.0); // 5 + 10 + 2 + 3
    }
}

/// Output format kind for smart paste
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppFormatKind {
    #[default]
    Unicode,
    Latex,
    MathML,
    Typst,
}

/// App profile for smart paste targeting
#[derive(Debug, Clone, Default)]
pub struct AppProfile {
    pub format: AppFormatKind,
    pub name: String,
}

impl AppProfile {
    pub fn new(process_name: &str, window_class: &str, window_title: &str) -> Self {
        let cat = resolve_app(process_name, window_class, window_title);
        let format = match cat {
            AppCategory::OfficeEquation => AppFormatKind::MathML,
            AppCategory::CodeEditor => AppFormatKind::Latex,
            AppCategory::ChatPresentation => AppFormatKind::Unicode,
            AppCategory::Terminal => AppFormatKind::Unicode,
            AppCategory::Unknown => AppFormatKind::Unicode,
        };
        Self {
            format,
            name: process_name.to_string(),
        }
    }
}

/// Resolve output mode for an app
pub fn resolve_output_mode(profile: &AppProfile, current: MathOutputMode) -> MathOutputMode {
    match profile.format {
        AppFormatKind::MathML => MathOutputMode::Normal,
        AppFormatKind::Latex => MathOutputMode::Latex,
        AppFormatKind::Unicode => MathOutputMode::UnicodeMath,
        AppFormatKind::Typst => MathOutputMode::Typst,
    }
    .max(current)
}
