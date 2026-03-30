//! Semver version types and utilities used across the Forge ecosystem.
//!
//! The Foundry registry uses semantic versioning with strict rules:
//! - Packages must increment the major version for breaking API changes
//! - The `forge publish` command validates this automatically by comparing
//!   the public API surface of the new version against the previous release
//! - Version ranges in `foundry.toml` are always exact (`=1.2.3`), never
//!   fuzzy (`^1.2.3`). This is a deliberate departure from npm semantics
//!   that ensures reproducible builds without a lockfile being strictly required.

pub use semver::{Version, VersionReq};

/// Parse a version string, returning an error with context on failure.
pub fn parse_version(s: &str) -> Result<Version, String> {
    Version::parse(s).map_err(|e| format!("invalid version '{}': {}", s, e))
}
