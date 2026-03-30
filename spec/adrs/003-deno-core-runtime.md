# ADR-003: deno_core as the Embedded JavaScript Runtime

**Number**: 003
**Title**: deno\_core as the Embedded JavaScript Runtime
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#runtime` `#v8` `#deno` `#tokio` `#embedding`

---

## Context

Forge's server binary embeds a JavaScript engine. This is not optional — the framework must execute JavaScript (server functions, middleware, request handlers) within the same process as the Rust host code that manages HTTP connections, the database pool, the session store, and the file serving layer. The alternative — spawning a Node.js subprocess — would reintroduce the exact operational complexity Forge is designed to eliminate.

Embedding a JavaScript engine in a Rust program means answering several hard questions:

**Which JavaScript engine?** V8 (Chrome), JavaScriptCore (Safari/WebKit), SpiderMonkey (Firefox), or QuickJS (embedded, no JIT). Each has different performance characteristics, different embedding complexity, and different compatibility profiles.

**How to handle async?** Modern JavaScript applications are fundamentally async — fetch, database queries, timers, stream reading. The embedding must integrate the JavaScript microtask queue and macrotask scheduling with the host runtime's async model. Get this wrong and you get deadlocks, dropped futures, or CPU busy-wait.

**How to expose host APIs?** JavaScript running in a Forge server function needs access to the request context, the database pool, the session, the response builder. These are Rust objects. The embedding must provide a safe, ergonomic way to call Rust from JavaScript (and vice versa) without violating V8's single-threaded execution model.

**How to load ES modules?** Modern JavaScript is module-based. The runtime must implement a module loader that resolves import specifiers, caches module evaluation, and handles circular dependencies correctly.

The raw V8 Rust bindings (`rusty_v8`) answer the first question (V8) and leave the other three entirely to the embedder. This means building the event loop, the async integration, the op system, and the module loader from scratch — approximately two to three years of infrastructure work that must be correct before any application code can run.

`deno_core` is `rusty_v8` plus those four hard things, production-proven by the Deno project over four years. It provides:

- A `JsRuntime` that manages V8 isolate lifecycle
- A Tokio-integrated event loop that correctly handles JS microtasks and async Rust futures
- An `#[op]` macro system for registering Rust functions as JavaScript-callable ops
- An ES module loader with pluggable resolution and caching
- A structured concurrency model (`JoinSet`-based) for managing concurrent ops

Deno itself is a production JavaScript runtime used by major companies. `deno_core` is the layer beneath Deno's standard library — it is the tested, maintained foundation that Deno's own team depends on.

## Decision

Use `deno_core` as the embedded JavaScript runtime layer within the Forge server binary.

Forge implements:

- A custom `ModuleLoader` that resolves `forge:*` specifier imports to FSL ops
- A custom op set implementing the WinterTC API surface (see ADR-008)
- A `ForgeRuntime` wrapper around `deno_core::JsRuntime` that manages the Forge-specific lifecycle (request context injection, response extraction, error marshaling)

The server function execution model:

1. The Forge compiler produces a module bundle for each route's server functions
2. At server startup, `ForgeRuntime` pre-loads and evaluates these modules
3. Per-request execution creates a V8 snapshot or uses module caching to avoid re-evaluation overhead
4. The request context (headers, params, session, body) is injected as a Rust-backed op
5. The server function executes within the V8 isolate, calling ops for I/O
6. The response is extracted from the isolate and returned to the Rust HTTP handler

## Consequences

### Positive

- ✅ **Production-proven**: Deno's own runtime depends on `deno_core`. The event loop integration, op system, and module loader have been tested against real workloads at scale. The risk of hitting fundamental async correctness bugs is low.
- ✅ **Tokio-native**: Forge's entire stack — HTTP server (hyper/axum), database pool (sqlx), session management, file serving — runs on Tokio. `deno_core`'s event loop integrates directly with the same Tokio runtime, using Tokio's task scheduler for async op completion. There is no thread-pool bridge between the JavaScript runtime and the Rust async runtime.
- ✅ **Op system maps cleanly to WinterTC APIs**: `deno_core`'s op registration system (`#[op2]` macro) is designed exactly for exposing async Rust functions as JavaScript-callable operations. Implementing the WinterTC API surface (fetch, crypto, streams) as `deno_core` ops is straightforward and follows patterns already established by Deno's own standard library.
- ✅ **ES module loader is correct**: getting ES module loading right — circular dependencies, dynamic imports, import maps, JSON imports — is notoriously difficult. `deno_core`'s module loader has handled all of these correctly in Deno's test suite for years.
- ✅ **V8's JIT performance**: for hot server functions, V8's JIT compiler provides native-code-level performance. The startup overhead (V8 initialization) is paid once per process, not per request.

### Negative

- ❌ **Large binary**: V8 compiles to approximately 100MB of native code. Stripped, the Forge server binary will be meaningfully larger than an equivalent pure-Rust binary. For server deployment contexts, this is generally acceptable. For edge deployment contexts with binary size limits, this may require building a JavaScript-free variant of the Forge server (which is possible — static sites require no JS engine).
- ❌ **V8 initialization latency**: cold-starting a process with V8 takes longer than starting a pure-Rust binary. For serverless/functions-as-a-service deployments, this cold start time is observable. Forge mitigates this with V8 snapshots (pre-serialized heap state) for faster startup, but it does not eliminate the overhead.
- ❌ **deno_core API stability**: `deno_core` is maintained by the Deno team, and its API surface does not have the same stability guarantees as a 1.0 library. Deno itself can introduce breaking changes to `deno_core` between releases. Forge pins to a specific `deno_core` version and updates deliberately.
- ❌ **V8 memory model complexity**: V8 runs with its own garbage collector. Forge's Rust code must be careful about how long it holds `v8::Local` handles — they are only valid within a `HandleScope`. The `deno_core` API abstracts most of this correctly, but Forge developers writing ops must understand V8's handle scoping rules.

### Neutral

- ℹ️ `deno_core` is MIT licensed, compatible with Forge's AGPL-3.0 license.
- ℹ️ V8 is used in production by the most widely deployed JavaScript runtime in the world (Node.js, Chrome). Its correctness for the ECMAScript specification is the highest of any available engine.
- ℹ️ Deno's `deno_core` and Cloudflare's `workerd` (which powers Cloudflare Workers) both embed V8 with Tokio-style async op systems. Forge's runtime model is therefore aligned with the production patterns of both major edge deployment targets.

## Alternatives Considered

### rusty_v8 directly

`rusty_v8` provides raw V8 bindings for Rust. Using it directly would give Forge maximum control over the runtime model.

The cost: everything `deno_core` provides — the event loop, the op system, the module loader, the async integration — must be built from scratch. This is not a small engineering effort. The event loop alone (correctly integrating JS microtasks, promise resolution, and Tokio futures without starvation or priority inversion) took the Deno team significant effort to get right. Building it again from `rusty_v8` raw bindings would consume engineering time that is better spent on Forge's unique capabilities.

Rejected: `deno_core` provides the right abstraction. The lower level is not worth the development cost.

### QuickJS (via quickjs-rs or rquickjs)

QuickJS is a small, embeddable JavaScript engine written in C. Its key advantages are small binary size (roughly 1MB vs V8's 100MB), fast startup (no JIT initialization), and simpler embedding (pure C, no complex build system).

QuickJS lacks a JIT compiler. For cold code paths, QuickJS's interpreter performance is adequate. For hot server functions called thousands of times per second, the absence of JIT means QuickJS runs 5-10x slower than V8 on comparable workloads. For a server framework positioning itself as production-grade, this performance gap is not acceptable.

QuickJS's ECMAScript compatibility, while improving, lags V8's. Edge cases in `Proxy`, `WeakRef`, async generators, and some newer TC39 proposals are handled differently or incompletely. Forge's FSL components use modern JavaScript features; compatibility gaps would surface.

Rejected: performance ceiling too low for production server use.

### JavaScriptCore (via webkit-rs)

JavaScriptCore is Safari's JavaScript engine. It has a JIT compiler, good performance, and excellent ECMAScript compliance. Several Rust embedding libraries exist.

The build system complexity for JavaScriptCore is significant. It is part of the WebKit monorepo and building it as a standalone dependency requires careful version management. The Rust bindings are less maintained than `rusty_v8` / `deno_core`. Embedding JavaScriptCore in a Tokio-based Rust binary requires building the async integration layer that `deno_core` provides — the same problem as using `rusty_v8` directly.

Rejected: comparable complexity to `rusty_v8` without the same ecosystem momentum.

### SpiderMonkey (via mozjs-sys)

SpiderMonkey is Firefox's JavaScript engine. Similar arguments to JavaScriptCore. The Rust bindings exist but the ecosystem around them is small. The async integration would need to be built from scratch.

Rejected: least ecosystem momentum of the available JIT-capable engines.

## Implementation Notes

The Forge runtime layer lives in `crates/forge-runtime`.

Key types:

- `ForgeRuntime` — wraps `deno_core::JsRuntime`, manages the Forge-specific op set and module loader
- `ForgeModuleLoader` — implements `deno_core::ModuleLoader`, handles `forge:*` specifiers and file-system module resolution
- `ForgeOpSet` — the complete WinterTC API surface implemented as `deno_core` ops
- `RequestContext` — a Rust struct exposed to JavaScript via an op, providing the current request's data
- `ResponseBuilder` — a Rust struct that JavaScript server functions write to via ops, consumed by the HTTP handler after execution

The op registration uses `deno_core`'s `#[op2]` macro, which generates the V8 binding boilerplate and handles async Rust future → JavaScript promise bridging automatically.

V8 snapshots are built as a separate compile step and embedded in the Forge binary as a `static` byte array. The snapshot captures the FSL's JavaScript surface after initialization, reducing per-request startup cost to the cost of restoring heap state from the snapshot rather than re-evaluating all module code.

## Related Decisions

- [ADR-001: Rust-Powered Compiler Pipeline](./001-rust-powered-compiler.md) — the compiler that produces the JS modules the runtime executes
- [ADR-008: WinterTC-Only Server APIs](./008-wintertc-only-server-apis.md) — the API surface the runtime exposes to server functions
