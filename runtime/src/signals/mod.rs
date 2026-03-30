//! Server-side signal runtime for SSR.
//!
//! During server-side rendering, components execute in the V8 isolate
//! and may read signal values. This module provides the server-side
//! implementation of the TC39 Signals polyfill used during SSR,
//! and the mechanism for serializing the initial signal state snapshot
//! embedded in the rendered HTML.

pub mod reactive_runtime;
