//! Compiler error types.
//!
//! [`CompilerError`] represents failures in the compilation pipeline itself
//! (I/O errors, internal invariant violations). It is distinct from
//! [`Diagnostic`], which represents user-facing errors in the source code.
//! A [`CompilerError`] is always a bug in Forge or the environment; a
//! [`Diagnostic`] with [`Severity::Error`] is a bug in the user's code.
//!
//! [`Diagnostic`]: forge_shared::diagnostics::Diagnostic
//! [`Severity::Error`]: forge_shared::diagnostics::Severity::Error

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("I/O error reading source file '{path}': {source}")]
    Io {
        path: camino::Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse manifest: {0}")]
    ManifestParse(String),

    #[error("internal compiler error: {0}")]
    Internal(String),

    #[error("compilation aborted: {error_count} error(s) in source")]
    SourceErrors { error_count: usize },
}
