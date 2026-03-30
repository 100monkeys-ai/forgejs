//! Runtime error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("JavaScript execution error: {0}")]
    JsError(String),

    #[error("module not found: {0}")]
    ModuleNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP server error: {0}")]
    Http(String),

    #[error("SSR error rendering route '{route}': {message}")]
    Ssr { route: String, message: String },

    #[error("internal runtime error: {0}")]
    Internal(String),
}
