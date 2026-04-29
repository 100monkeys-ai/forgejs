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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_success() {
        let version = parse_version("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_parse_version_error() {
        let err1 = parse_version("1.2").unwrap_err();
        assert_eq!(err1, "invalid version '1.2': unexpected end of input while parsing minor version number");

        let err2 = parse_version("invalid").unwrap_err();
        assert_eq!(err2, "invalid version 'invalid': unexpected character 'i' while parsing major version number");
    }
}
