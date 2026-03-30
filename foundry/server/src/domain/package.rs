//! The Package aggregate — the core domain entity of the Foundry registry.
//!
//! A Package represents a published Forge library identified by `author/name`.
//! It contains one or more [`Version`]s, each representing a specific published
//! release with its own source tarball and content hash.
//!
//! [`Version`]: crate::domain::version::Version

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A published package in the Foundry registry.
#[derive(Debug, Clone)]
pub struct Package {
    pub id: Uuid,
    /// Package identifier in `author/name` format
    pub name: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
