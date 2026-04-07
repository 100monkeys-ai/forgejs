//! Parsed representations of `forge.toml` and `foundry.toml` manifests.
//!
//! These types are the single source of truth for manifest schemas across the
//! entire Forge toolchain. The compiler reads `forge.toml` to understand build
//! targets. The foundry client reads `foundry.toml` to resolve dependencies.
//! Both produce these types from raw TOML.

use camino::Utf8PathBuf;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The parsed representation of a `forge.toml` project manifest.
///
/// This is the root configuration file for a Forge application. It declares
/// the project identity, dependency list, build configuration, and one or
/// more deployment targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeManifest {
    /// Core project metadata
    pub project: ProjectMetadata,
    /// Build configuration (optional, all fields have defaults)
    #[serde(default)]
    pub build: BuildConfig,
    /// Development server configuration
    #[serde(default)]
    pub dev: DevConfig,
    /// One or more deployment targets
    #[serde(default)]
    pub target: Vec<TargetConfig>,
    /// Runtime dependencies from the Foundry registry
    #[serde(default)]
    pub dependencies: IndexMap<String, DependencySpec>,
    /// Development-only dependencies
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: IndexMap<String, DependencySpec>,
}

/// Core project identity metadata from `[project]` in `forge.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
}

/// Build configuration from `[build]` in `forge.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// Entry point component (defaults to `app/root.fx`)
    #[serde(default)]
    pub entry: Option<Utf8PathBuf>,
    /// Output directory (defaults to `.forge/dist`)
    #[serde(default)]
    pub output: Option<Utf8PathBuf>,
    /// Whether to generate source maps (defaults to true in dev, false in release)
    #[serde(default)]
    pub source_maps: Option<bool>,
}

/// Development server configuration from `[dev]` in `forge.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevConfig {
    /// Port for the dev server (defaults to 3000)
    #[serde(default)]
    pub port: Option<u16>,
    /// Port for Forge Studio (defaults to 3001)
    #[serde(default)]
    pub studio_port: Option<u16>,
    /// Whether to open the browser on start (defaults to false)
    #[serde(default)]
    pub open: Option<bool>,
}

/// A deployment target declared via `[[target]]` in `forge.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Human-readable name for this target (e.g., "production", "edge")
    pub name: String,
    /// The target type determining what artifact is produced
    #[serde(rename = "type")]
    pub target_type: TargetType,
    /// Output path for this target's artifacts
    #[serde(default)]
    pub output: Option<Utf8PathBuf>,
    /// Target-specific configuration
    #[serde(flatten)]
    pub options: HashMap<String, toml::Value>,
}

/// The type of deployment target, determining the compilation output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TargetType {
    /// Pre-rendered HTML + client-side JS bundle. No server required.
    Static,
    /// Self-contained Rust binary embedding the JS runtime (deno_core).
    Server,
    /// Cloudflare Workers / WinterTC-compatible edge bundle.
    Edge,
    /// Tauri desktop application wrapping the UI in a native WebView.
    Desktop,
    /// Mobile WebView application (Capacitor pattern).
    Mobile,
}

/// A dependency specification in `[dependencies]` or `[dev-dependencies]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string: `"forge:router" = "1.0"`
    Version(String),
    /// Detailed spec with version and optional features
    Detailed {
        version: String,
        #[serde(default)]
        features: Vec<String>,
    },
}

/// The parsed representation of a `foundry.toml` package manifest.
///
/// Used by the Foundry registry client to publish and resolve packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryManifest {
    pub package: FoundryPackageMetadata,
    #[serde(default)]
    pub dependencies: IndexMap<String, DependencySpec>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: IndexMap<String, DependencySpec>,
}

/// Package identity in `foundry.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryPackageMetadata {
    /// Package name in `author/name` format (e.g., `jeshua/my-lib`)
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    /// BLAKE3 hash of the package source tree (set by `forge publish`)
    #[serde(default)]
    pub integrity: Option<String>,
}

/// Metadata tracking the provenance of a migrated Node.js application.
///
/// Stored in `[project.metadata.migration]` in the generated `forge.toml`
/// to record how the project was created and the quality of the automatic
/// conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    /// The source framework detected (e.g., "react", "express", "next")
    pub source_framework: String,
    /// Node.js version from `.nvmrc` or `package.json` `engines` field
    #[serde(default)]
    pub source_node_version: Option<String>,
    /// ISO 8601 date when the migration was performed
    pub migration_date: String,
    /// Total dependency count in the original `package.json`
    pub original_dep_count: usize,
    /// Number of functions resolved as actually reachable from entry points
    pub resolved_function_count: usize,
    /// Percentage of source that was auto-converted without manual intervention
    pub auto_converted_pct: f32,
}

/// Compatibility classification for a source file or function during migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Compatibility {
    /// Pure JS/TS — no Node.js-specific APIs, converts directly to `.fx`
    Compatible,
    /// Uses Node.js APIs that have WinterTC equivalents (e.g., Buffer → Uint8Array)
    Shimmable,
    /// Uses non-portable Node.js APIs — requires manual migration
    NeedsManualAttention,
}
