//! Client-side transformation: prepares the client bundle.
//!
//! Responsibilities:
//!
//! - Remove all server-only modules from the client module graph
//! - Inject generated RPC stubs in place of stripped server imports
//! - Inject the TC39 Signals polyfill if the target runtime does not
//!   natively support `Signal.State` / `Signal.Computed`
//! - Inject hydration bootstrapping for server-rendered HTML
