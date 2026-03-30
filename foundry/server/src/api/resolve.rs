//! POST /resolve — Batch dependency resolution.
//!
//! Accepts a list of `author/name@version` specifiers and returns the
//! full resolved dependency graph including transitive dependencies.
//! This single endpoint replaces npm's per-package metadata lookups,
//! reducing the number of round trips required to resolve a project's
//! full dependency tree.
