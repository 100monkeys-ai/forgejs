//! Terminal output formatting.
//!
//! All user-facing output from the CLI uses these helpers to ensure
//! consistent formatting with appropriate colors and symbols.

/// Print a success message (green checkmark).
pub fn success(msg: &str) {
    println!("\x1b[32m✓\x1b[0m {}", msg);
}

/// Print an informational message (blue arrow).
pub fn info(msg: &str) {
    println!("\x1b[34m→\x1b[0m {}", msg);
}

/// Print a warning message (yellow warning sign).
pub fn warn(msg: &str) {
    eprintln!("\x1b[33m⚠\x1b[0m {}", msg);
}

/// Print an error message (red cross).
pub fn error(msg: &str) {
    eprintln!("\x1b[31m✗\x1b[0m {}", msg);
}
