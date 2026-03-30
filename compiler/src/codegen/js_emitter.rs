//! Emit JavaScript text from the transformed AST.
//!
//! Uses Oxc's code generator for fast, correct JavaScript emission.
//! The emitter applies target-specific optimizations:
//!
//! - **Development**: readable output with source maps for debugging
//! - **Production**: minified output via Oxc's minifier pass
//! - **Edge**: ensures all emitted code uses only WinterTC-compatible APIs
