//! V8 JavaScript isolate management.
//!
//! The isolate is the core of the Forge runtime. It owns the V8 context,
//! the event loop, and all registered ops (Forge's WinterTC API implementations).
//!
//! ## Isolate Lifecycle
//!
//! 1. A [`ForgeRuntime`] is created, initializing a deno_core `JsRuntime`
//!    with Forge's op set registered
//! 2. Compiled JavaScript modules are loaded into the isolate
//! 3. The event loop runs until all pending operations complete
//! 4. For server use, the isolate is kept alive across requests
//!
//! ## Op System
//!
//! deno_core's op system allows Rust functions to be called from JavaScript.
//! Forge registers ops for every WinterTC API (fetch, crypto, streams, etc.)
//! and for Forge-specific APIs (server function dispatch, SSR rendering).
//!
//! The permission model ([`permissions`]) controls which ops are available
//! based on the deployment target and runtime configuration.

pub mod ops;
pub mod permissions;
pub mod v8_runtime;
