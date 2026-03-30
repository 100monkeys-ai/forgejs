//! `forge.toml` manifest parser.
//!
//! Reads a `forge.toml` file from disk and deserializes it into a
//! [`ForgeManifest`]. Emits structured errors for invalid TOML or
//! schema violations.
//!
//! [`ForgeManifest`]: forge_shared::manifest::ForgeManifest

use crate::error::CompilerError;
use forge_shared::manifest::ForgeManifest;

/// Parse a `forge.toml` file at the given path.
///
/// # Errors
///
/// Returns [`CompilerError::Io`] if the file cannot be read, or
/// [`CompilerError::ManifestParse`] if the TOML is invalid or does not
/// conform to the `ForgeManifest` schema.
pub fn parse_forge_toml(path: &camino::Utf8Path) -> Result<ForgeManifest, CompilerError> {
    let content = std::fs::read_to_string(path).map_err(|e| CompilerError::Io {
        path: path.to_owned(),
        source: e,
    })?;

    toml::from_str(&content).map_err(|e| CompilerError::ManifestParse(e.to_string()))
}
