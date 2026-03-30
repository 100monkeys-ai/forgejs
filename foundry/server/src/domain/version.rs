//! A specific version of a published package.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A specific published version of a Foundry package.
#[derive(Debug, Clone)]
pub struct Version {
    pub id: Uuid,
    pub package_id: Uuid,
    /// Semver version string (e.g., "1.2.3")
    pub version: String,
    /// BLAKE3 hash of the source tarball: `blake3:<hex>`
    pub integrity: String,
    /// URL to download the source tarball
    pub download_url: String,
    /// Size of the source tarball in bytes
    pub size_bytes: i64,
    pub published_at: DateTime<Utc>,
}
