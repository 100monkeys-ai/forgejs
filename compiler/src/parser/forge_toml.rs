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

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8Path;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_valid_forge_toml() {
        let mut file = NamedTempFile::new().unwrap();
        let valid_toml = r#"
[project]
name = "test-project"
version = "0.1.0"
description = "A test project"
authors = ["Test Author"]
"#;
        file.write_all(valid_toml.as_bytes()).unwrap();
        let path = Utf8Path::from_path(file.path()).unwrap();

        let manifest = parse_forge_toml(path).expect("Failed to parse valid TOML");
        assert_eq!(manifest.project.name, "test-project");
        assert_eq!(manifest.project.version, "0.1.0");
        assert_eq!(manifest.project.description.as_deref(), Some("A test project"));
        assert_eq!(manifest.project.authors, vec!["Test Author"]);
    }

    #[test]
    fn test_parse_missing_file() {
        let path = Utf8Path::new("/path/to/non/existent/file.toml");
        let result = parse_forge_toml(path);

        assert!(matches!(result, Err(CompilerError::Io { .. })));
    }

    #[test]
    fn test_parse_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        let invalid_toml = r#"
[project]
name = "test-project
version = 0.1.0"
"#;
        file.write_all(invalid_toml.as_bytes()).unwrap();
        let path = Utf8Path::from_path(file.path()).unwrap();

        let result = parse_forge_toml(path);
        assert!(matches!(result, Err(CompilerError::ManifestParse(_))));
    }
}
