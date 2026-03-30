//! Local package cache management.
//!
//! The Foundry cache stores downloaded packages at `~/.forge/cache/`.
//! Packages are stored by their BLAKE3 content hash, making the cache
//! content-addressed: the same package version always maps to the same
//! path, and the path is immutable once written.
//!
//! ## Cache Layout
//!
//! ```text
//! ~/.forge/
//! ├── cache/
//! │   ├── <blake3-hash>/          # Package source tree
//! │   │   ├── foundry.toml
//! │   │   └── src/
//! │   └── ...
//! ├── config.toml                 # CLI config
//! └── keys/                       # Signing keys for package publishing
//!     └── default.key
//! ```
//!
//! ## Why Content-Addressed
//!
//! npm stores packages under `node_modules/<name>/` in each project,
//! meaning the same package is downloaded and stored once per project.
//! On a machine with 50 projects, that's 50 copies of React.
//!
//! The Foundry cache stores each package version once globally, keyed by
//! its content hash. When two projects depend on the same version, they
//! share the same cache entry. The compiler resolves imports directly from
//! the cache — there is no per-project `node_modules/`.

use camino::Utf8PathBuf;

/// Returns the path to the global Foundry cache directory.
pub fn cache_dir() -> Utf8PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Utf8PathBuf::from(home).join(".forge").join("cache")
}

/// Check if a package with the given BLAKE3 hash is in the cache.
pub fn is_cached(integrity: &str) -> bool {
    let hash = integrity.strip_prefix("blake3:").unwrap_or(integrity);
    cache_dir().join(hash).exists()
}
