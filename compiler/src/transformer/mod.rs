//! AST transformation passes that run after analysis.
//!
//! The transformer takes the analyzed, annotated AST and produces
//! target-specific JavaScript. It is responsible for:
//!
//! - Splitting the module graph into server and client halves
//! - Generating typed RPC stubs for server functions
//! - Compiling signal syntax (`$signal`, `$derived`, `$async`, `$effect`)
//!   into TC39 Signal API calls with compile-time DOM wiring
//!
//! The transformer does not emit JavaScript directly — that is the job of
//! [`codegen`]. The transformer produces a transformed AST that codegen
//! converts to text.
//!
//! [`codegen`]: crate::codegen

pub mod client_transform;
pub mod server_transform;
pub mod signal_transform;
