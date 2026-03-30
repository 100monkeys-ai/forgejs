//! API key authentication for the Foundry registry.
//!
//! Authentication uses API keys associated with an author namespace.
//! Keys are generated via `forge login` and stored in `~/.forge/keys/`.
//!
//! The `Authorization: Bearer <key>` header is required for:
//! - Package publishing (POST /packages/*)
//! - Private package access
//!
//! Public package downloads (GET /packages/*) do not require authentication.
