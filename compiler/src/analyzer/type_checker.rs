//! Additional type-level checks beyond TypeScript's own type checker.
//!
//! TypeScript validates types within a single compilation unit. This pass
//! validates Forge-specific type rules that require cross-boundary awareness:
//!
//! 1. **Serialization whitelist** — server function return types must be
//!    JSON-serializable or one of the blessed streaming types.
//! 2. **WinterTC API compliance** — for `edge` targets, only WinterTC APIs
//!    may be referenced in server modules.
//! 3. **Signal type safety** — `$async` signals must return a `Promise<T>`;
//!    `$derived` signals must be synchronous.

/// Types that are permitted to cross the client/server boundary.
///
/// Any type not in this list (or not recursively composed of types in this
/// list) is a boundary violation and a compile error.
pub const SERIALIZABLE_TYPES: &[&str] = &[
    "string",
    "number",
    "boolean",
    "null",
    "undefined",
    "Array",
    "object", // plain object literal, not class instances
    "Date",   // serialized as ISO 8601 string
    "File",
    "Blob",
    "ReadableStream",
    "FormData",
    "Uint8Array",
];
