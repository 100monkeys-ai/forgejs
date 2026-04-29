//! Development server initialization.
//!
//! Starts the dev server with HMR and Forge Studio.
//! Listens on port 3000 (app) and port 3001 (Studio) by default.

use crate::error::RuntimeError;

/// Start the development server.
pub async fn start_dev_server(_port: u16, _studio_port: u16) -> Result<(), RuntimeError> {
    // TODO: Initialize file watcher, incremental compiler, HMR, and Studio
    Ok(())
}
