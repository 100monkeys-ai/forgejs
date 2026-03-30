//! Dependency resolution: convert `foundry.toml` deps into exact versions.
//!
//! The resolver takes the dependency declarations from a project's
//! `foundry.toml` and produces a fully resolved, conflict-free set of
//! exact package versions that satisfies all constraints.
//!
//! ## Resolution Algorithm
//!
//! Forge uses a simplified resolution algorithm compared to npm's:
//!
//! 1. All version specifications in `foundry.toml` must be exact versions
//!    or the `*` wildcard (meaning "latest stable"). Ranges (`^`, `~`) are
//!    not supported — this is a deliberate choice to ensure reproducibility.
//!
//! 2. If two packages require different exact versions of the same dependency,
//!    that is a hard conflict — the developer must resolve it manually. Unlike
//!    npm, Foundry does not silently install multiple versions of the same
//!    package in different locations.
//!
//! 3. The lockfile (`foundry.lock`) records the exact resolved versions with
//!    their BLAKE3 content hashes. Subsequent installs use the lockfile and
//!    never re-resolve if it is present.
//!
//! This approach trades flexibility for predictability. npm's version ranges
//! allow minor/patch updates to flow in silently, which is convenient until
//! a "compatible" update breaks your application.

pub mod dependency_graph;
pub mod lockfile;
