//! Build and resolve the package dependency graph.

use forge_shared::manifest::DependencySpec;
use std::collections::HashMap;

/// A resolved package in the dependency graph.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// Package identifier in `author/name` format
    pub name: String,
    /// Exact resolved version
    pub version: String,
    /// BLAKE3 hash of the package source tree
    pub integrity: String,
    /// URL to download the package tarball from the registry
    pub download_url: String,
}

/// The fully resolved dependency graph for a project.
#[derive(Debug, Default)]
pub struct ResolvedGraph {
    /// Map from package name to resolved package metadata
    pub packages: HashMap<String, ResolvedPackage>,
}
