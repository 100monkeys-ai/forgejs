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
        name: &str,
        version: &str,
    ) -> Result<crate::resolver::dependency_graph::ResolvedPackage, FoundryError> {
        let url = format!(
            "{}/packages/{}/{}",
            self.base_url.trim_end_matches('/'),
            name,
            version
        );

        let mut req = self.http.get(&url).header("X-Forge-Protocol", "1");
        if let Some(token) = &self.auth_token {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let integrity = resp
                    .headers()
                    .get("X-Forge-Integrity")
                    .and_then(|h| h.to_str().ok())
                    .ok_or_else(|| {
                        FoundryError::Registry("missing X-Forge-Integrity header".into())
                    })?
                    .to_string();

                Ok(crate::resolver::dependency_graph::ResolvedPackage {
                    name: name.to_string(),
                    version: version.to_string(),
                    integrity,
                    download_url: url,
                })
            }
            reqwest::StatusCode::NOT_FOUND => Err(FoundryError::VersionNotFound {
                package: name.to_string(),
                version: version.to_string(),
            }),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                Err(FoundryError::AuthRequired)
            }
            status => Err(FoundryError::Registry(format!(
                "unexpected registry status: {}",
                status
            ))),
        }
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
