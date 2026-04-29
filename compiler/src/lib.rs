//! # forge-compiler
//!
//! The Forge compiler transforms `.fx` and `.ts` source files into optimized
//! JavaScript artifacts for one or more deployment targets.
//!
//! ## Pipeline
//!
//! The compiler is a sequential pipeline of passes, each operating on the
//! output of the previous:
//!
//! ```text
//! Source (.fx / .ts)
//!     │
//!     ▼
//! [parser]      — Parse source into a typed AST (via Oxc)
//!     │
//!     ▼
//! [analyzer]    — Boundary analysis, signal graph, type narrowing
//!     │
//!     ▼
//! [transformer] — Server/client split, signal desugaring, RPC stub gen
//!     │
//!     ▼
//! [codegen]     — Emit optimized JavaScript + source maps
//!     │
//!     ▼
//! [bundler]     — Module graph, code splitting, asset pipeline
//!     │
//!     ▼
//! Target artifact (JS bundle / server binary / edge worker)
//! ```
//!
//! ## Design Principles
//!
//! - **No plugin system for core transforms.** The compiler owns the entire
//!   pipeline. This is what enables compile-time guarantees (boundary
//!   enforcement, signal validation) that a plugin-based architecture cannot
//!   provide. See ADR-010 for the rationale.
//!
//! - **Errors are compile errors, not runtime errors.** Any invariant that
//!   can be checked at compile time is checked at compile time. The boundary
//!   analyzer, signal analyzer, and type checker all emit [`Diagnostic`]s
//!   with [`Severity::Error`] for violations. The compiler refuses to emit
//!   output if any errors are present.
//!
//! - **Target-aware codegen.** The same source produces different output for
//!   different [`TargetType`]s. The `static` target strips all server code.
//!   The `edge` target enforces WinterTC-only APIs. The `desktop` target
//!   rewrites server function calls to Tauri IPC.
//!
//! [`Diagnostic`]: forge_shared::diagnostics::Diagnostic
//! [`Severity::Error`]: forge_shared::diagnostics::Severity::Error
//! [`TargetType`]: forge_shared::manifest::TargetType

pub mod analyzer;
pub mod bundler;
pub mod codegen;
pub mod error;
pub mod parser;
pub mod transformer;

use forge_shared::{diagnostics::DiagnosticBag, manifest::TargetType};

/// The result of a successful compilation pass.
#[derive(Debug)]
pub struct CompileOutput {
    /// The primary JavaScript bundle for this target
    pub js_bundle: Vec<u8>,
    /// Inline source map (if source maps are enabled)
    pub source_map: Option<Vec<u8>>,
    /// Additional assets (CSS, images) referenced by the bundle
    pub assets: Vec<Asset>,
    /// Diagnostics emitted during compilation (warnings only — errors abort)
    pub diagnostics: DiagnosticBag,
}

/// A static asset produced by the compiler (CSS, image, font, etc.)
#[derive(Debug)]
pub struct Asset {
    /// The asset's path in the output directory
    pub path: camino::Utf8PathBuf,
    /// The asset's raw content
    pub content: Vec<u8>,
    /// MIME type of the asset
    pub mime_type: String,
}

/// Options controlling a single compilation run.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// The deployment target being compiled for
    pub target: TargetType,
    /// Whether to emit source maps
    pub source_maps: bool,
    /// Whether to minify the output
    pub minify: bool,
    /// The project root directory
    pub project_root: camino::Utf8PathBuf,
}

/// Run the compiler pipeline with the given options.
pub fn compile(_options: CompileOptions) -> Result<CompileOutput, error::CompilerError> {
    // TODO: Implement the full compiler pipeline
    Ok(CompileOutput {
        js_bundle: Vec::new(),
        source_map: None,
        assets: Vec::new(),
        diagnostics: DiagnosticBag::new(),
    })
}
