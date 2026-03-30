//! TC39 Signals polyfill for the server runtime.
//!
//! The TC39 Signals proposal (`Signal.State`, `Signal.Computed`,
//! `Signal.subtle.Watcher`) is polyfilled in the V8 isolate for both
//! SSR and client-side execution until V8 ships native support.
//!
//! This module provides the Rust-side support for the polyfill:
//! tracking the current evaluation context for automatic dependency
//! tracking, and serializing the signal state snapshot for hydration.
