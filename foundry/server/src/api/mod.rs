//! HTTP API handlers for the Foundry registry.
//!
//! See spec/specs/007-foundry-protocol.md for the full API specification.
//!
//! ## Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/packages/{author}/{name}` | Publish a new package version |
//! | GET | `/packages/{author}/{name}/{version}` | Download a package tarball |
//! | GET | `/packages/{author}/{name}` | List all versions of a package |
//! | POST | `/resolve` | Batch dependency resolution |
//! | GET | `/search` | Full-text package search |

pub mod auth;
pub mod publish;
pub mod resolve;
pub mod search;

use axum::{routing::get, Router};
use sqlx::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(|| async { "Foundry Registry" }))
        .with_state(pool)
}
