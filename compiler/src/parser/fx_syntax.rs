//! `.fx` file syntax extensions over TypeScript.
//!
//! After Oxc parses a `.fx` file as TypeScript, this module post-processes
//! the AST to recognize and annotate `.fx`-specific constructs:
//!
//! - `export component` declarations
//! - `server` keyword on function declarations
//! - `"use module server"` / `"use module client"` directives
//! - `$signal` / `$derived` / `$async` / `$effect` reactive primitives
//!
//! The output of this pass is an annotated AST where each node carries
//! metadata flags (is_server_function, is_component, reactive_kind, etc.)
//! consumed by the [`analyzer`] and [`transformer`] passes.
//!
//! [`analyzer`]: crate::analyzer
//! [`transformer`]: crate::transformer

/// Module-level boundary directive parsed from `"use module server"` or
/// `"use module client"` at the top of a `.fx` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleDirective {
    /// `"use module server"` — entire module is server-only.
    /// The compiler strips this module from client bundles and generates
    /// typed RPC stubs for all exported server functions.
    Server,
    /// `"use module client"` — entire module is client-only.
    /// The compiler errors if this module is imported from server code.
    Client,
    /// No directive present — module can be used on both sides unless
    /// individual exports are annotated with `server`.
    None,
}

/// Metadata attached to a function declaration after `.fx` syntax analysis.
#[derive(Debug, Clone)]
pub struct FunctionMeta {
    /// Whether this function was declared with the `server` keyword.
    /// Server functions are stripped from client bundles and replaced
    /// with typed HTTP RPC stubs.
    pub is_server: bool,
    /// Whether this function was declared with `export component`.
    /// Component functions receive special treatment in the signal
    /// transform pass: their JSX return values are compiled to
    /// direct DOM wiring rather than VDOM calls.
    pub is_component: bool,
}
