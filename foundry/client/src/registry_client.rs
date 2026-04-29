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
    pub async fn download(
        &self,
        package_name: &str,
        url: &str,
        expected_integrity: &str,
    ) -> Result<Vec<u8>, FoundryError> {
        let mut request = self.http.get(url);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(FoundryError::Network)?;
        let status = response.status();
        if !status.is_success() {
            return Err(FoundryError::Registry(format!(
                "HTTP error {} when downloading {}",
                status, package_name
            )));
        }

        let bytes = response.bytes().await.map_err(FoundryError::Network)?;
        let actual_integrity = blake3::hash(&bytes).to_hex().to_string();

        if actual_integrity != expected_integrity {
            return Err(FoundryError::IntegrityMismatch {
                package: package_name.to_string(),
                expected: expected_integrity.to_string(),
                actual: actual_integrity,
            });
        }

        Ok(bytes.to_vec())
    }
}
