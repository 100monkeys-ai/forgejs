//! # forge-shared
//!
//! Shared types used across all Forge crates. This crate has no business logic —
//! it exists solely to define the common data structures that `forge-compiler`,
//! `forge-runtime`, `forge-cli`, `foundry-server`, and `foundry-client` all need
//! without creating circular dependencies.
//!
//! ## Contents
//!
//! - [`manifest`] — `forge.toml` and `foundry.toml` parsed representations
//! - [`diagnostics`] — Structured compiler/runtime diagnostic types
//! - [`source_location`] — File position, span, and source range types
//! - [`version`] — Semver version types used across the ecosystem

pub mod diagnostics;
pub mod manifest;
pub mod source_location;
pub mod version;
