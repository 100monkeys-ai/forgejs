//! Structured diagnostic types for compiler errors, warnings, and hints.
//!
//! All Forge tools (compiler, runtime, linter) emit diagnostics using these
//! types. This ensures consistent error formatting across the entire toolchain
//! and makes it possible to serialize diagnostics for IDE integration (LSP).

use crate::source_location::SourceSpan;
use serde::{Deserialize, Serialize};

/// A single diagnostic message from the compiler, linter, or runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level of this diagnostic
    pub severity: Severity,
    /// Short machine-readable error code (e.g., `E0001`, `W0042`)
    pub code: String,
    /// Human-readable message describing the problem
    pub message: String,
    /// The source location where the problem was detected
    pub span: Option<SourceSpan>,
    /// Additional notes providing context or suggestions
    #[serde(default)]
    pub notes: Vec<DiagnosticNote>,
    /// A suggested fix, if one can be mechanically applied
    #[serde(default)]
    pub suggestion: Option<Suggestion>,
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational hint — the code works but could be improved
    Hint,
    /// Warning — the code may have unintended behavior
    Warning,
    /// Error — the code cannot compile or run correctly
    Error,
    /// Internal compiler error — a bug in Forge itself
    Ice,
}

/// An additional note attached to a diagnostic, with an optional source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticNote {
    pub message: String,
    pub span: Option<SourceSpan>,
}

/// A suggested mechanical fix for a diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub message: String,
    pub replacements: Vec<Replacement>,
}

/// A text replacement to apply as part of a suggested fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replacement {
    pub span: SourceSpan,
    pub new_text: String,
}

/// A collection of diagnostics from a compilation or analysis pass.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiagnosticBag {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity >= Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity >= Severity::Error)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_bag_error_counting() {
        let mut bag = DiagnosticBag::new();
        assert!(!bag.has_errors());
        assert_eq!(bag.error_count(), 0);

        let dummy_diagnostic = |severity: Severity| Diagnostic {
            severity,
            code: "TEST".to_string(),
            message: "test message".to_string(),
            span: None,
            notes: vec![],
            suggestion: None,
        };

        // Hints and Warnings should not count as errors
        bag.push(dummy_diagnostic(Severity::Hint));
        assert!(!bag.has_errors());
        assert_eq!(bag.error_count(), 0);

        bag.push(dummy_diagnostic(Severity::Warning));
        assert!(!bag.has_errors());
        assert_eq!(bag.error_count(), 0);

        // Error should be counted
        bag.push(dummy_diagnostic(Severity::Error));
        assert!(bag.has_errors());
        assert_eq!(bag.error_count(), 1);

        // Ice should also be counted as an error
        bag.push(dummy_diagnostic(Severity::Ice));
        assert!(bag.has_errors());
        assert_eq!(bag.error_count(), 2);
    }
}
