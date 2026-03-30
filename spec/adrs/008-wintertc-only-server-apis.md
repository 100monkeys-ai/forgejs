# ADR-008: WinterTC-Only Server APIs

**Number**: 008
**Title**: WinterTC-Only Server APIs
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#runtime` `#standards` `#portability` `#wintertc` `#edge`

---

## Context

Forge's server functions must run in multiple contexts:

1. **The Forge server binary** — the self-contained Rust binary embedding V8 via deno_core (ADR-003), deployed to a VPS, bare metal, or container. The primary deployment target.
2. **Cloudflare Workers** — via `workerd`, Cloudflare's JavaScript runtime. Edge deployment, globally distributed, sub-millisecond cold starts.
3. **Future WinterTC-compatible runtimes** — Deno Deploy, Bun's edge runtime, and any other runtime that implements the WinterTC standard API surface.

For the same server function code to run correctly in all three environments, the code must only use APIs that all three environments provide. This is the portability constraint.

The challenge is that JavaScript on the server has two competing API surfaces:

**Node.js APIs**: The de facto standard since 2009. `node:http`, `node:fs`, `node:crypto`, `process.env`, `Buffer`, `__dirname`, `require()`, `stream.Readable`. These APIs are what most JavaScript backend developers know and what most npm packages were written against. They are Node.js-specific — Cloudflare Workers do not implement them (except for a partial `node:` compatibility layer that covers ~60% of the API surface), and edge runtimes often do not implement them at all.

**WinterTC APIs** (Ecma TC55, formerly WinterCG): The standardized server-side JavaScript API surface, defined by the TC55 committee with members including Cloudflare, Deno, Node.js, and others. The WinterTC API set is the intersection of what all modern server-side JavaScript runtimes provide. It is based on web platform APIs — the same APIs that exist in the browser — plus server-specific additions.

WinterTC includes: `fetch`, `Request`, `Response`, `Headers`, `URL`, `URLSearchParams`, `ReadableStream`, `WritableStream`, `TransformStream`, `crypto.subtle`, `TextEncoder`, `TextDecoder`, `FormData`, `Blob`, `File`, `EventTarget`, `AbortController`, `AbortSignal`, `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`, `queueMicrotask`, `structuredClone`, `atob`, `btoa`, `console`, and the `Cache` API.

The decision is: which of these two API surfaces does Forge server code target?

If Forge targets Node.js APIs, server functions are locked to Node.js (and the partial Node.js compatibility layers in other runtimes). Edge deployment requires a compatibility shim. Portability is a best-effort promise, not a guarantee.

If Forge targets WinterTC APIs only, server functions are portable across any WinterTC-compliant runtime by construction. The portability is a compile-time guarantee, not a best-effort promise. The cost is that Node.js-specific packages from npm that assume `process`, `Buffer`, or `require()` do not work without shims.

### WinterTC Is Where the Industry Is Going

The WinterTC API set is not an obscure standard. Node.js 18+ implements native `fetch`. Node.js 20+ ships with the `Headers`, `Request`, and `Response` globals. The web-platform APIs that WinterTC standardizes are becoming available in Node.js, closing the gap from the Node.js side.

Cloudflare Workers, Deno, and Bun all implement WinterTC APIs natively. The standard is converging: the web platform APIs that started in the browser are becoming the common API surface for server-side JavaScript.

Forge, by targeting WinterTC exclusively, is aligning with the convergence point rather than the Node.js legacy point.

## Decision

Forge server code must use only WinterTC-compliant APIs. Node.js-specific APIs are not available in the Forge server runtime.

### Available APIs (The WinterTC Surface)

The Forge server runtime implements the following globals, matching the WinterTC specification:

**Network**:

- `fetch(input, init?)` — HTTP requests
- `Request`, `Response`, `Headers` — HTTP primitives
- `URL`, `URLSearchParams` — URL manipulation
- `WebSocket` — WebSocket client

**Streams**:

- `ReadableStream`, `WritableStream`, `TransformStream`
- `ReadableStreamDefaultReader`, `WritableStreamDefaultWriter`
- `ByteLengthQueuingStrategy`, `CountQueuingStrategy`

**Encoding**:

- `TextEncoder`, `TextDecoder`
- `atob`, `btoa`

**Cryptography**:

- `crypto.subtle` — the Web Crypto API (AES, RSA, ECDSA, HMAC, SHA, etc.)
- `crypto.randomUUID()`
- `crypto.getRandomValues()`

**Structured Data**:

- `FormData`, `Blob`, `File`
- `structuredClone()`

**Timers and Scheduling**:

- `setTimeout`, `clearTimeout`
- `setInterval`, `clearInterval`
- `queueMicrotask`

**Observability**:

- `console` (log, warn, error, debug, info, table, time, timeEnd, trace)
- `performance.now()`, `performance.mark()`, `performance.measure()`

**Events**:

- `EventTarget`, `Event`, `CustomEvent`
- `AbortController`, `AbortSignal`

**Environment** (Forge-specific, not Node.js `process.env`):

- `Forge.env(key)` — reads environment variables via the Forge op system. Returns `string | undefined`. The distinction from `process.env.KEY` is that `Forge.env()` is explicit about the lookup and auditable in the module graph.

### Not Available

The following are explicitly absent:

- `process` — no Node.js process object, no `process.env`, no `process.exit()`
- `Buffer` — use `Uint8Array` and `TextEncoder`/`TextDecoder` instead
- `require()` — ESM only, no CommonJS
- `__dirname`, `__filename` — no file-system path inference
- `node:*` imports — no Node.js built-in modules
- `fs`, `path`, `os`, `child_process`, `worker_threads` — no direct OS access from server functions

Direct OS access is available through Forge FSL primitives (`forge:storage` for file operations, `forge:jobs` for subprocess-like operations) or through the Rust escape hatch (a native Forge plugin written in Rust that exposes custom ops).

## Consequences

### Positive

- ✅ **True write-once deploy-anywhere portability**: a Forge server function that passes `forge check --target=edge` is guaranteed to run correctly on Cloudflare Workers, the Forge binary, Deno Deploy, and any other WinterTC runtime. The guarantee is compile-time, not documentation.
- ✅ **Future-proof API surface**: WinterTC APIs are becoming the standard. Node.js is implementing them natively. Code written for WinterTC will still work in future runtimes; code written for Node.js-specific APIs may need migration.
- ✅ **Security boundary clarity**: the absence of `process`, `child_process`, and direct filesystem access from server functions means that a compromised server function cannot read arbitrary environment variables, execute arbitrary commands, or read arbitrary files. Security-sensitive operations are only available through explicitly audited FSL primitives.
- ✅ **Consistent runtime behavior**: server functions run identically in development (the Forge binary) and production (Cloudflare Workers or the Forge binary). The `node:` compatibility layer differences that bite developers when moving between Node.js and edge runtimes do not exist in Forge.
- ✅ **The Forge binary's op implementation is complete**: implementing the WinterTC surface as deno_core ops is a bounded, completable engineering task. Implementing all of Node.js's built-in modules would be unbounded.

### Negative

- ❌ **npm packages that assume Node.js APIs do not work**: many npm packages use `Buffer`, `process.env`, `node:events`, `node:stream`, or other Node.js APIs. These packages cannot be used directly in Forge server functions without a shim or refactoring. This limits the applicable subset of the npm/Foundry ecosystem for server-side use.
- ❌ **Developers familiar with Node.js APIs must learn WinterTC equivalents**: `Buffer.from(str, 'utf8')` becomes `new TextEncoder().encode(str)`. `process.env.MY_VAR` becomes `Forge.env('MY_VAR')`. These are minor translations but represent a learning curve for developers who have internalized Node.js idioms.
- ❌ **WinterTC is still evolving**: the standard is not yet complete. Some APIs (notably filesystem access, which WinterTC is working on via the File System Access API proposal) are not yet part of the WinterTC stable surface. Forge must make pragmatic decisions about where to wait for the standard and where to provide Forge-specific APIs in the gap.
- ❌ **No built-in database driver assumes WinterTC**: most database drivers (`pg`, `mysql2`, `mongoose`) use Node.js TCP sockets under the hood. They are not WinterTC-compatible. Forge's answer is `forge:data`, which uses the Forge runtime's op system for database connections, but this means any database driver not in the FSL requires a Rust-native op implementation.

### Neutral

- ℹ️ The Forge compiler's boundary analysis (ADR-007) uses the WinterTC API list to classify modules. A module that imports from `node:*` is automatically classified as a Node.js-specific module and flagged as incompatible with Forge server functions.
- ℹ️ The `forge check --target=edge` flag runs a stricter subset check, verifying that the server function only uses APIs that are available on edge runtimes (which may have a smaller WinterTC surface than the full Forge binary).

## Alternatives Considered

### Node.js API Surface

Target the full Node.js API surface. Server functions can use `node:fs`, `node:http`, `process.env`, `Buffer`, and everything else Node.js provides. The npm ecosystem is fully compatible. Edge deployment is supported through compatibility shims.

The problems:

1. The "compatibility shim" for Node.js APIs on edge runtimes is not complete. Cloudflare Workers' Node.js compatibility covers the most common APIs but has gaps that manifest as runtime errors.
2. Security: `node:child_process` in server functions means server functions can spawn arbitrary processes. `node:fs` means server functions can read arbitrary files. Forge's threat model wants server functions to have limited, auditable access to the host, not full Node.js capabilities.
3. Portability is a lie: "we support edge with Node.js compatibility" means "we support edge for the subset of your code that doesn't use any of the many Node.js APIs that compatibility shims don't cover." This is worse than no portability claim at all, because it fails non-deterministically.

Rejected: false portability, poor security model.

### WinterTC + Node.js Compatibility Layer

Implement both API surfaces. Server functions can use either WinterTC APIs or Node.js APIs. Forge's edge compiler flags usages of Node.js APIs as warnings, and a `--strict-wintertc` flag makes them errors.

The problem: having both available means developers default to what they know (Node.js APIs), and "WinterTC-only" becomes an opt-in. Opt-in portability guarantees are not portability guarantees; they are portability suggestions.

The value of the WinterTC-only decision is that it is a hard constraint. If a developer writes `process.env.DATABASE_URL` in a server function, they get a compile error, not a warning they can ignore. Hard constraints produce reliable guarantees; soft constraints do not.

Rejected: the guarantee value is in the hardness of the constraint.

## Implementation Notes

The WinterTC API surface is implemented as deno_core ops in `crates/forge-runtime/src/ops/`.

Each WinterTC API is a separate op module:

- `ops/fetch.rs` — `fetch`, `Request`, `Response`, `Headers`
- `ops/streams.rs` — `ReadableStream`, `WritableStream`, `TransformStream`
- `ops/crypto.rs` — `crypto.subtle`, `crypto.randomUUID`, `crypto.getRandomValues`
- `ops/encoding.rs` — `TextEncoder`, `TextDecoder`, `atob`, `btoa`
- `ops/url.rs` — `URL`, `URLSearchParams`
- `ops/timers.rs` — `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`, `queueMicrotask`
- `ops/env.rs` — `Forge.env()`
- `ops/console.rs` — `console.*`

The API surface list in the compiler (`crates/forge-compiler/src/analyze/wintertc.rs`) is generated from the same source of truth as the op implementations, preventing drift between what the compiler permits and what the runtime provides.

## Related Decisions

- [ADR-003: deno_core Runtime](./003-deno-core-runtime.md) — the runtime that implements these ops
- [ADR-007: Compile-Time Boundary Enforcement](./007-compile-time-boundary-enforcement.md) — the compiler that verifies API usage
