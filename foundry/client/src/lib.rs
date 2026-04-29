//! # foundry-client
//!
//! The Foundry registry client. Handles everything related to package
//! management from the developer's perspective:
//!
//! - Reading `foundry.toml` package manifests
//! - Resolving dependency trees to exact versions
//! - Writing and reading `foundry.lock` lockfiles
//! - Downloading packages from the Foundry registry server
//! - Maintaining the local package cache (`~/.forge/cache/`)
//! - Publishing packages to the registry
//!
//! ## The Foundry vs npm
//!
//! The Foundry is not a drop-in npm replacement. It makes different
//! trade-offs that prioritize correctness and security over compatibility:
//!
//! | Property | npm | Foundry |
//! |----------|-----|---------|
//! | Package identity | name only (squattable) | author/name (cryptographic) |
//! | Version pinning | ranges (`^`, `~`) | exact or lockfile |
//! | Artifact type | compiled JS + typedefs | TypeScript source |
//! | Install location | per-project `node_modules/` | global content-addressed cache |
//! | Integrity | SHA-512 (optional) | BLAKE3 (mandatory) |
//! | API breaking changes | honor system | enforced by `forge publish` |
//!
//! See ADR-009 for the full rationale.
//!
//! ## Modules
//!
//! - [`manifest`] — Parse `foundry.toml` package manifests
//! - [`resolver`] — Resolve dependency trees to exact versions
//! - [`registry_client`] — HTTP client for the Foundry registry API
//! - [`cache`] — Local package cache management

pub mod cache;
pub mod error;
pub mod manifest;
pub mod migrate;
pub mod publish;
pub mod registry_client;
pub mod resolver;
pub mod publish;
