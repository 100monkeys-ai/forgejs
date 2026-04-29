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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_foundry_toml_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("foundry.toml");
        let toml_content = r#"
[package]
name = "jeshua/my-lib"
version = "1.2.3"
description = "A useful library"
license = "MIT"

[dependencies]
"jeshua/other-lib" = "2.0.0"

[dev-dependencies]
"forge:test" = "*"
"#;
        fs::write(&file_path, toml_content).unwrap();

        let utf8_path = camino::Utf8Path::from_path(&file_path).unwrap();
        let result = parse_foundry_toml(utf8_path);
        assert!(result.is_ok());

        let manifest = result.unwrap();
        assert_eq!(manifest.package.name, "jeshua/my-lib");
        assert_eq!(manifest.package.version, "1.2.3");
        assert_eq!(
            manifest.package.description.as_deref(),
            Some("A useful library")
        );
        assert_eq!(manifest.package.license.as_deref(), Some("MIT"));
        assert!(manifest.dependencies.contains_key("jeshua/other-lib"));
        assert!(manifest.dev_dependencies.contains_key("forge:test"));
    }

    #[test]
    fn test_parse_foundry_toml_io_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("does_not_exist.toml");

        let utf8_path = camino::Utf8Path::from_path(&file_path).unwrap();
        let result = parse_foundry_toml(utf8_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            FoundryError::Io(_) => {} // Expected
            err => panic!("Expected Io error, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_foundry_toml_manifest_parse_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("foundry.toml");
        // Missing quotes around version value
        let toml_content = r#"
[package]
name = "jeshua/my-lib"
version = 1.2.3
"#;
        fs::write(&file_path, toml_content).unwrap();

        let utf8_path = camino::Utf8Path::from_path(&file_path).unwrap();
        let result = parse_foundry_toml(utf8_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            FoundryError::ManifestParse { path, message } => {
                assert_eq!(path, utf8_path.to_string());
                assert!(
                    message.contains("expected newline, `#`")
                        || message.contains("TOML parse error"),
                    "Message was: {}",
                    message
                );
            }
            err => panic!("Expected ManifestParse error, got {:?}", err),
        }
    }
}
