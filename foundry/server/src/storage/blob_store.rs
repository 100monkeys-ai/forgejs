//! Package tarball storage.
//!
//! Stores package source tarballs in a content-addressed blob store.
//! The blob key is the BLAKE3 hash of the tarball content, ensuring
//! that identical tarballs are stored only once.
//!
//! ## Storage Backends
//!
//! - **Local filesystem**: for development and self-hosted registries
//! - **S3-compatible**: for production deployments (AWS S3, Cloudflare R2)

/// A blob storage backend for package tarballs.
#[async_trait::async_trait]
pub trait BlobStore: Send + Sync {
    /// Store a blob and return its content-addressed key (BLAKE3 hash).
    async fn put(&self, content: &[u8]) -> Result<String, crate::error::ServerError>;
    /// Retrieve a blob by its BLAKE3 hash key.
    async fn get(&self, key: &str) -> Result<Vec<u8>, crate::error::ServerError>;
    /// Check whether a blob exists.
    async fn exists(&self, key: &str) -> Result<bool, crate::error::ServerError>;
}
