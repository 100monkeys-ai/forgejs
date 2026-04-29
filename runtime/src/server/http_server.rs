//! HTTP server initialization and lifecycle.
//!
//! The server starts an axum router on the configured port, sets up
//! the route table from the compiled route manifest, and begins
//! accepting connections.

use crate::error::RuntimeError;

/// Configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The port to listen on (default: 3000)
    pub port: u16,
    /// The host to bind to (default: 0.0.0.0)
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
        }
    }
}

/// Start the HTTP server with the given configuration.
///
/// This function does not return until the server is shut down.
pub async fn serve(config: ServerConfig) -> Result<(), RuntimeError> {
    let app = axum::Router::new();

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| RuntimeError::Http(format!("Failed to bind to {}: {}", addr, e)))?;

    tracing::info!("HTTP server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| RuntimeError::Http(format!("Server error: {}", e)))?;

    Ok(())
}
