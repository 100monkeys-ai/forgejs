//! `foundry.toml` parser.
//!
//! `foundry.toml` is the package manifest for Foundry packages — analogous
//! to `package.json` in npm, but with a cleaner schema and stricter semantics.
//!
//! ## Package Identity
//!
//! Packages are identified by `author/name` where `author` is tied to a
//! cryptographic signing key registered with the Foundry registry. This
//! eliminates name squatting: only the key holder for a given author name
//! can publish packages under that namespace.
//!
//! ## Example foundry.toml
//!
//! ```toml
//! [package]
//! name = "jeshua/my-lib"
//! version = "1.2.3"
//! description = "A useful library"
//! license = "MIT"
//!
//! [dependencies]
//! "jeshua/other-lib" = "2.0.0"
//!
//! [dev-dependencies]
//! "forge:test" = "*"
//! ```

use crate::error::FoundryError;
use forge_shared::manifest::FoundryManifest;

/// Parse a `foundry.toml` file at the given path.
pub fn parse_foundry_toml(path: &camino::Utf8Path) -> Result<FoundryManifest, FoundryError> {
    let content = std::fs::read_to_string(path).map_err(FoundryError::Io)?;
    toml::from_str(&content).map_err(|e| FoundryError::ManifestParse {
        path: path.to_string(),
        message: e.to_string(),
    })
}
