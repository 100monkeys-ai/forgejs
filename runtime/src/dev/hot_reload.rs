//! Hot module replacement (HMR) implementation.
//!
//! When a source file changes, the HMR system:
//! 1. Recompiles the changed module (incrementally)
//! 2. Pushes the new module source to connected browsers via a SSE stream
//! 3. The browser's HMR runtime replaces the module in-place if possible,
//!    or triggers a full reload if the module graph changed structurally
