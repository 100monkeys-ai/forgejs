//! HTTP client for the Foundry registry API.
//!
//! See spec/specs/007-foundry-protocol.md for the full API contract.

use crate::error::FoundryError;

/// HTTP client for communicating with a Foundry registry server.
#[allow(dead_code)]
pub struct RegistryClient {
    base_url: String,
    auth_token: Option<String>,
    http: reqwest::Client,
}

impl RegistryClient {
    pub fn new(base_url: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth_token,
            http: reqwest::Client::new(),
        }
    }

    /// Resolve a package name and version to its metadata.
    pub async fn resolve(
        &self,
        _name: &str,
        _version: &str,
    ) -> Result<crate::resolver::dependency_graph::ResolvedPackage, FoundryError> {
        // TODO: GET /packages/{author}/{name}/{version}
        Err(FoundryError::PackageNotFound("not implemented".to_string()))
    }

    /// Download a package tarball and return its bytes.
    pub async fn download(&self, _url: &str) -> Result<Vec<u8>, FoundryError> {
        // TODO: Download and verify BLAKE3 integrity
        Err(FoundryError::PackageNotFound("not implemented".to_string()))
    }
}
