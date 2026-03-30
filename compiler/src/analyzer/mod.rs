//! Semantic analysis passes that run after parsing.
//!
//! The analyzer is responsible for all checks that require understanding
//! the *meaning* of the code, not just its syntax. It operates on the
//! annotated AST produced by the parser and emits [`Diagnostic`]s for
//! any violations.
//!
//! ## Passes
//!
//! - [`boundary_analyzer`] — Enforces the client/server boundary rules
//!   defined in spec/specs/005-boundary-enforcement.md. This is the most
//!   important pass in the compiler.
//!
//! - [`signal_analyzer`] — Validates the reactive signal dependency graph:
//!   detects cycles, verifies that `$async` is only used in components,
//!   ensures effects don't capture server-only values.
//!
//! - [`type_checker`] — Additional type narrowing beyond what TypeScript's
//!   own checker provides: validates that server function return types are
//!   serializable, checks that WinterTC API usage is correct for the target.
//!
//! All passes run in sequence. Later passes may assume that earlier passes
//! have validated their invariants.
//!
//! [`Diagnostic`]: forge_shared::diagnostics::Diagnostic

pub mod boundary_analyzer;
pub mod signal_analyzer;
pub mod type_checker;
