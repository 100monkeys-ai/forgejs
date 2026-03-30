//! Foundry client error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FoundryError {
    #[error("package not found: {0}")]
    PackageNotFound(String),

    #[error("version not found: {package}@{version}")]
    VersionNotFound { package: String, version: String },

    #[error("dependency conflict: {0}")]
    DependencyConflict(String),

    #[error("integrity check failed for {package}: expected {expected}, got {actual}")]
    IntegrityMismatch {
        package: String,
        expected: String,
        actual: String,
    },

    #[error("manifest parse error in '{path}': {message}")]
    ManifestParse { path: String, message: String },

    #[error("registry error: {0}")]
    Registry(String),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("authentication required — run `forge login` to authenticate with the Foundry")]
    AuthRequired,
}
