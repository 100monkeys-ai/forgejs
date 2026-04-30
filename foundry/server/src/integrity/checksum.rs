//! BLAKE3 checksum computation and verification.
//!
//! All packages in the Foundry registry are integrity-verified using BLAKE3.
//!
//! ## Why BLAKE3 Over SHA-256
//!
//! npm uses SHA-512 for package integrity. BLAKE3 was chosen for Foundry
//! because it is 2-3x faster, supports parallel hashing, and has stronger
//! security properties. The `blake3:` prefix in integrity strings makes the
//! algorithm self-describing and forward-compatible.

use blake3::Hasher;

/// Compute the BLAKE3 hash of a byte slice and return it as a `blake3:<hex>` string.
pub fn compute(data: &[u8]) -> String {
    let hash = Hasher::new().update(data).finalize();
    format!("blake3:{}", hash.to_hex())
}

/// Verify that `data` matches the expected `blake3:<hex>` integrity string.
pub fn verify(data: &[u8], expected: &str) -> bool {
    let actual = compute(data);
    actual == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_empty() {
        // blake3 hash of empty string
        let expected = "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        assert_eq!(compute(b""), expected);
    }

    #[test]
    fn test_compute_hello_world() {
        // blake3 hash of "hello world"
        let expected = "blake3:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24";
        assert_eq!(compute(b"hello world"), expected);
    }

    #[test]
    fn test_verify_happy_path() {
        let data = b"hello world";
        let expected = "blake3:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24";
        assert!(verify(data, expected));
    }

    #[test]
    fn test_verify_modified_data() {
        let data = b"hello world!"; // extra char
        let expected = "blake3:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"; // hash of "hello world"
        assert!(!verify(data, expected));
    }

    #[test]
    fn test_verify_invalid_prefix() {
        let data = b"hello world";
        // same hash value, different prefix
        let expected = "sha256:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24";
        assert!(!verify(data, expected));
    }

    #[test]
    fn test_verify_wrong_length_hash() {
        let data = b"hello world";
        let expected = "blake3:d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e"; // truncated
        assert!(!verify(data, expected));
    }

    #[test]
    fn test_verify_empty_string() {
        let data = b"";
        let expected = "blake3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        assert!(verify(data, expected));
    }
}
