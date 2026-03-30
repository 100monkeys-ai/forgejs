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
