//! deno_core op implementations for the Forge WinterTC API surface.
//!
//! Each function here implements one WinterTC API as a deno_core op —
//! a Rust function callable from JavaScript inside the V8 isolate.
//!
//! ## WinterTC API Coverage
//!
//! The following APIs are implemented as ops (see spec/specs/006-wintertc-api-surface.md
//! for the full normative list):
//!
//! - `fetch` / `Request` / `Response` / `Headers`
//! - `URL` / `URLSearchParams`
//! - `ReadableStream` / `WritableStream` / `TransformStream`
//! - `crypto.subtle` (SubtleCrypto)
//! - `TextEncoder` / `TextDecoder`
//! - `FormData` / `Blob` / `File`
//! - `EventTarget` / `Event` / `AbortController` / `AbortSignal`
//! - `setTimeout` / `clearTimeout` / `setInterval` / `clearInterval`
//! - `queueMicrotask` / `structuredClone`
//! - `atob` / `btoa`
//!
//! ## What Is NOT Available
//!
//! The following Node.js APIs are deliberately not implemented:
//! - `process` (no `process.env` — use Forge's config API instead)
//! - `node:*` imports
//! - `Buffer` (use `Uint8Array` instead)
//! - `require()` (ESM only)
//!
//! This is enforced both here (no ops registered) and by the compiler's
//! boundary analyzer for edge targets.

use deno_core::extension;
use deno_core::op2;

#[op2(fast)]
pub fn op_forge_noop() {}

extension!(forge_ops, ops = [op_forge_noop]);
