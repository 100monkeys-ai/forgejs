//! Source file location types: file paths, byte offsets, line/column positions, and spans.
//!
//! These types are used throughout the compiler and diagnostics system to
//! pinpoint exactly where in source code a particular construct appears.
//! They are designed to be cheap to copy and serialize for LSP/IDE integration.

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

/// A position within a source file, identified by byte offset and line/column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePosition {
    /// Zero-based byte offset from the start of the file
    pub offset: u32,
    /// One-based line number
    pub line: u32,
    /// One-based column number (in UTF-8 code units)
    pub column: u32,
}

/// A contiguous range of source text within a single file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// The source file this span refers to
    pub file: Utf8PathBuf,
    /// Start of the span (inclusive)
    pub start: SourcePosition,
    /// End of the span (exclusive)
    pub end: SourcePosition,
}

impl SourceSpan {
    /// Returns true if this span covers a single point (zero-length).
    pub fn is_point(&self) -> bool {
        self.start.offset == self.end.offset
    }

    /// Returns the byte length of this span.
    pub fn len(&self) -> u32 {
        self.end.offset.saturating_sub(self.start.offset)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_span_is_empty() {
        let pos1 = SourcePosition {
            offset: 10,
            line: 1,
            column: 11,
        };
        let pos2 = SourcePosition {
            offset: 20,
            line: 2,
            column: 5,
        };

        let empty_span = SourceSpan {
            file: Utf8PathBuf::from("test.js"),
            start: pos1,
            end: pos1,
        };

        let non_empty_span = SourceSpan {
            file: Utf8PathBuf::from("test.js"),
            start: pos1,
            end: pos2,
        };

        // Test is_empty
        assert!(empty_span.is_empty(), "Span with identical start and end should be empty");
        assert!(!non_empty_span.is_empty(), "Span with different start and end should not be empty");

        // Test len
        assert_eq!(empty_span.len(), 0, "Empty span length should be 0");
        assert_eq!(non_empty_span.len(), 10, "Non-empty span length should be the difference between offsets");

        // Test is_point
        assert!(empty_span.is_point(), "Empty span should be a point");
        assert!(!non_empty_span.is_point(), "Non-empty span should not be a point");
    }
}
