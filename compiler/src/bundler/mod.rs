//! Module graph construction and JavaScript bundling.
//!
//! After the compiler emits individual module JavaScript, the bundler:
//!
//! 1. Resolves the full module import graph from the entry point
//! 2. Splits the graph into chunks (route-based code splitting)
//! 3. Processes non-JS assets (CSS via Lightning CSS, images)
//! 4. Assembles the final output bundle(s)
//!
//! The bundler uses Rolldown's Rust API for fast, correct bundling.
//! Rolldown is not exposed as a plugin surface — it is an implementation
//! detail of the bundler pass.

pub mod asset_pipeline;
pub mod chunk_splitter;
pub mod module_graph;
