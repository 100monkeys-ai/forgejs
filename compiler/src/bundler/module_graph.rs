//! Build and traverse the module import graph.
//!
//! The module graph is built by starting from the entry point(s) declared
//! in `forge.toml` and recursively resolving all imports. Each node in the
//! graph represents a source file; edges represent import relationships.
//!
//! The graph is used by:
//! - The boundary analyzer (to trace transitive server imports)
//! - The chunk splitter (to determine code splitting boundaries)
//! - The bundler (to assemble the final output)
