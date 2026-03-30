//! POST /packages/{author}/{name} — Publish a new package version.
//!
//! The publish endpoint:
//! 1. Authenticates the request using the API key header
//! 2. Verifies the API key belongs to the `author` namespace
//! 3. Validates the uploaded `foundry.toml` manifest
//! 4. Computes the BLAKE3 hash of the package source tarball
//! 5. Checks that the API version bump is correct for the change type
//! 6. Stores the tarball in blob storage
//! 7. Records the package metadata in the database
//! 8. Returns the assigned content hash
