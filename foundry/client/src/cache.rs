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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Helper struct to manage the HOME environment variable for tests
    struct EnvGuard {
        old_home: Option<String>,
    }

    impl EnvGuard {
        fn new(temp_dir: &Utf8PathBuf) -> Self {
            let old_home = std::env::var("HOME").ok();
            std::env::set_var("HOME", temp_dir.as_str());
            Self { old_home }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old_home {
                Some(home) => std::env::set_var("HOME", home),
                None => std::env::remove_var("HOME"),
            }
        }
    }

    #[test]
    #[serial]
    fn test_cache_dir_resolves_from_home() {
        let temp = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let _guard = EnvGuard::new(&temp_path);

        let expected = temp_path.join(".forge").join("cache");
        assert_eq!(cache_dir(), expected);
    }

    #[test]
    #[serial]
    fn test_cache_dir_defaults_to_dot() {
        let old_home = std::env::var("HOME").ok();
        std::env::remove_var("HOME");

        let expected = Utf8PathBuf::from(".").join(".forge").join("cache");
        assert_eq!(cache_dir(), expected);

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        }
    }

    #[test]
    #[serial]
    fn test_is_cached_with_prefix() {
        let temp = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let _guard = EnvGuard::new(&temp_path);

        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let cache_path = cache_dir().join(hash);
        std::fs::create_dir_all(&cache_path).unwrap();

        assert!(is_cached(&format!("blake3:{}", hash)));
    }

    #[test]
    #[serial]
    fn test_is_cached_without_prefix() {
        let temp = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let _guard = EnvGuard::new(&temp_path);

        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let cache_path = cache_dir().join(hash);
        std::fs::create_dir_all(&cache_path).unwrap();

        assert!(is_cached(hash));
    }

    #[test]
    #[serial]
    fn test_is_cached_not_found() {
        let temp = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let _guard = EnvGuard::new(&temp_path);

        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        assert!(!is_cached(&format!("blake3:{}", hash)));
        assert!(!is_cached(hash));
    }
}
