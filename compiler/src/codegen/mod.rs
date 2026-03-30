//! JavaScript code generation from the transformed AST.
//!
//! The codegen pass converts the transformer's output into emitted JavaScript
//! text, with optional source maps for debugging.

pub mod js_emitter;
pub mod source_map;
