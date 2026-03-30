//! Server-side transformation: extracts server functions and generates RPC stubs.
//!
//! For each validated server function (from the [`ServerFunctionRegistry`]):
//!
//! 1. **Server bundle**: the function implementation is kept as-is, wrapped
//!    in a minimal HTTP handler that parses arguments from the request body
//!    and serializes the return value to JSON.
//!
//! 2. **Client bundle**: the function implementation is stripped entirely.
//!    A typed RPC stub is generated with the same TypeScript signature,
//!    implementing the function as a `fetch` call to the server's RPC endpoint.
//!
//! The stub generation ensures that callers on the client cannot distinguish
//! between calling a local function and calling a remote server function.
//! The only observable difference is latency.
//!
//! [`ServerFunctionRegistry`]: crate::analyzer::boundary_analyzer::ServerFunctionRegistry
