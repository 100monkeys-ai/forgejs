//! Client/server boundary enforcement.
//!
//! ## The Problem This Solves
//!
//! In every existing JavaScript full-stack framework, the client/server
//! boundary is enforced by convention, not by the compiler. Next.js Server
//! Actions are annotated with `"use server"`, but nothing prevents a developer
//! from accidentally importing a server module into a client component — the
//! error only appears at runtime, in production, after deployment.
//!
//! Forge makes boundary violations **compile errors**. The boundary analyzer
//! runs before any code is emitted, walking the full module import graph to
//! detect violations. If the analyzer finds an error, the compiler refuses
//! to produce output.
//!
//! ## The Four Boundary Rules
//!
//! These rules are the normative specification (also see spec/specs/005-boundary-enforcement.md):
//!
//! ### Rule 1: No Server Imports in Client Code
//!
//! A module marked `"use module server"` (or containing `server` functions)
//! **must not** be imported — directly or transitively — by any module that
//! runs on the client.
//!
//! ```text
//! ERROR: Client module 'app/pages/Home.fx' imports server module 'server/users.fx'
//! help: Move the import into a server function, or use the generated RPC stub instead
//! ```
//!
//! ### Rule 2: No DOM APIs in Server Code
//!
//! Server modules must not reference DOM globals (`document`, `window`,
//! `navigator`, `localStorage`, etc.). These APIs do not exist in the
//! server runtime and would throw at runtime if not caught here.
//!
//! ### Rule 3: No Non-Serializable Cross-Boundary Types
//!
//! Server functions can only return values that can be serialized over HTTP
//! (JSON-compatible types plus `File`, `Blob`, `ReadableStream`). A server
//! function that returns a class instance with methods, a function, or a
//! symbol is a compile error.
//!
//! ### Rule 4: No Closure Capture Across Boundaries
//!
//! A server function cannot close over a variable defined in client scope.
//! This would create a hidden dependency that cannot be satisfied at runtime.
//!
//! ## RPC Stub Generation
//!
//! For each server function that passes validation, the boundary analyzer
//! records it in the [`ServerFunctionRegistry`] for the transformer to
//! generate a typed HTTP RPC stub. The stub has the same TypeScript signature
//! as the original function — callers on the client see no difference.

/// Registry of validated server functions, populated by the boundary analyzer
/// and consumed by the transformer to generate RPC stubs.
#[derive(Debug, Default)]
pub struct ServerFunctionRegistry {
    /// Map from fully-qualified function name to its signature metadata
    pub functions: std::collections::HashMap<String, ServerFunctionMeta>,
}

/// Metadata about a validated server function.
#[derive(Debug, Clone)]
pub struct ServerFunctionMeta {
    /// The source module path (e.g., `server/users.fx`)
    pub source_module: camino::Utf8PathBuf,
    /// The function's exported name
    pub name: String,
    /// The HTTP route path generated for this function (e.g., `/_forge/rpc/users/getUser`)
    pub rpc_path: String,
}
