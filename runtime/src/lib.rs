//! # forge-runtime
//!
//! The Forge runtime is responsible for executing compiled JavaScript in a
//! managed, permissioned environment and serving it over HTTP.
//!
//! ## Architecture
//!
//! The runtime is built on [deno_core](https://crates.io/crates/deno_core),
//! which provides:
//!
//! - A V8 JavaScript isolate via `rusty_v8`
//! - A Tokio-based async event loop
//! - A permissioned op system for registering Rust-implemented APIs
//! - A module loader for resolving and executing ES modules
//!
//! See ADR-003 for the full rationale for choosing deno_core over alternatives.
//!
//! ## Why deno_core Instead of Node.js
//!
//! The self-contained server binary target requires a JavaScript engine
//! embedded in a Rust binary. deno_core provides exactly this:
//!
//! - **Production-proven**: Deno has used deno_core in production for 4+ years
//! - **Tokio-native**: the event loop integrates directly with Tokio, the same
//!   async runtime used throughout the Forge codebase
//! - **Permissioned op system**: Forge's WinterTC API surface is implemented as
//!   deno_core ops, with Forge controlling exactly which APIs are available
//! - **Standalone binary**: `deno compile` has proven the model; Forge does
//!   the same by embedding the compiled JS as static bytes in the binary
//!
//! ## Modules
//!
//! - [`isolate`] — V8 isolate setup, op registration, module execution
//! - [`server`] — HTTP server, request routing, SSR pipeline
//! - [`dev`] — Development server with hot module replacement
//! - [`signals`] — Server-side signal runtime (for SSR)

pub mod dev;
pub mod error;
pub mod isolate;
pub mod server;
pub mod signals;
pub mod test;
