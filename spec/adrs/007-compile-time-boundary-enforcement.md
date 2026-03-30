# ADR-007: Compile-Time Client/Server Boundary Enforcement

**Number**: 007
**Title**: Compile-Time Client/Server Boundary Enforcement
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#compiler` `#boundary` `#security` `#correctness` `#architecture`

---

## Context

Every modern full-stack JavaScript framework has a client/server boundary. The boundary is the conceptual line between code that runs in the browser (where it is part of the JavaScript bundle delivered to users) and code that runs on the server (where it has access to secrets, databases, and server-only APIs).

Getting this boundary wrong has two failure modes:

**Security failure**: Server-only code (database query implementations, API keys, authentication logic) is accidentally included in the client bundle and delivered to users. This is not hypothetical — it is a documented class of vulnerabilities in Next.js applications. A developer imports a utility function that internally imports a database client; the bundler follows the import chain and includes the database client in the client bundle, along with whatever configuration it reads from `process.env`.

**Runtime failure**: Client-only code (DOM API calls, browser-specific globals like `window` and `document`) is executed in a server context where those APIs do not exist. This produces `ReferenceError: window is not defined` at runtime — often in production, because local development sometimes runs everything in a browser-like context.

### How Existing Frameworks Handle the Boundary

**Next.js "use client" / "use server" directives**: Next.js introduced string directives (`"use client"` at the top of a file) to mark component trees. The compiler uses these directives to determine which modules are included in the client bundle vs the server bundle.

The problems with this approach:

1. The directives are strings — they are invisible to TypeScript's type system. A `"use server"` action imported from a `"use client"` component does not produce a TypeScript error at the import site; it produces a runtime error during bundling or at runtime.
2. The boundary is enforced at the module level, not the function level. A module marked `"use client"` must export only client-safe values. But the compiler cannot verify this automatically — if a client component re-exports something imported from a server module, the violation is only caught at bundle time, not at the TypeScript/IDE level.
3. Forgetting the directive silently changes the bundle inclusion behavior. There is no error for a missing directive — the framework makes a default assumption that may or may not be correct.

**tRPC**: tRPC solves the *type safety* problem across the boundary — API procedures are defined with input and output types, and the client gets a type-safe caller. But tRPC requires explicit procedure definitions: you define the server procedure, then import it on the client side via the tRPC router. The boundary is safe, but the ergonomics require duplicate structure (define the procedure, then define the client call, then use the client call).

tRPC also does not prevent a developer from writing a server procedure that accidentally includes client-only code, or vice versa. It manages the type-safe communication across the boundary but does not enforce what belongs on each side.

**Convention-only (express, koa, early Next.js)**: No enforcement. The developer is responsible for knowing what runs where. This is error-prone by definition.

### The Correct Mental Model

The client/server boundary is a hard constraint of the web platform. Client code and server code are fundamentally different kinds of things:

- Client code runs in an untrusted environment (the user's browser). Any secret in client code is a disclosed secret.
- Server code runs in the trusted environment (your infrastructure). It has access to credentials, databases, and user data.
- Client code has access to DOM APIs, the browser's event system, and browser-specific globals.
- Server code has access to the filesystem, network, and the host process environment.

These are not soft conventions — they reflect real capabilities and real security boundaries. A framework that treats the boundary as a convention (easily forgotten, not enforced) is a framework that regularly produces security vulnerabilities and runtime crashes.

The correct model is to treat the boundary the same way a type system treats a type error: as a property that must hold for the program to be correct, and that the compiler verifies rather than the developer manually checking.

## Decision

The client/server boundary is enforced at compile time. Violations are compile errors. No JavaScript is emitted for a program with a boundary violation.

### The Four Boundary Rules (Normative)

These rules are normative — they define correct Forge programs. See `spec/specs/005-boundary-enforcement.md` for the complete specification.

**Rule 1: No server imports in client code.**

A module in a client context (a client component, a client-side utility, the client entry point) must not import any module that is in a server context. "Server context" includes: any module imported by a server function, any module that calls a WinterTC-only API, any module in the `server/` directory, any module explicitly annotated with `@server`.

Violation: compile error "Module 'X' is a server-only module and cannot be imported by client code."

**Rule 2: No DOM APIs in server code.**

A module in a server context must not reference DOM globals: `window`, `document`, `navigator`, `localStorage`, `sessionStorage`, `location`, `history`, `HTMLElement` and its subclasses, `addEventListener`, `removeEventListener`, or any other browser-specific global.

Violation: compile error "DOM API 'X' is not available in server context."

This rule protects against the `ReferenceError: window is not defined` runtime failure class.

**Rule 3: No non-serializable types crossing the boundary.**

Data passed between server and client must be serializable. The boundary crossing points are:

- Server function return values (sent from server to client as JSON)
- Server function arguments that originate from client signals (sent from client to server)
- Initial page data (server-rendered into the HTML as JSON for client resumption)

Non-serializable types include: functions, class instances with methods, Promises, Symbols, undefined values in object positions, circular references, and BigInt (unless the serializer is configured to handle it).

The compiler uses Forge's type information to verify that types crossing the boundary are serializable. This requires the Oxc type checker (see ADR-002 for the current limitation).

Violation: compile error "Type 'X' is not serializable and cannot cross the client/server boundary."

**Rule 4: No closure capture across boundaries.**

A server function cannot close over client-side state. A client component cannot close over server-side state. Closures that capture variables from the other side of the boundary are a source of subtle bugs where the captured value is stale (the client state at the time the server function was defined, not the current client state) or invalid (a client signal object that does not exist in the server context).

Server functions may accept parameters (explicitly passed from the client). Client event handlers may call server functions (via the generated RPC layer). But closures that span the boundary are not permitted.

Violation: compile error "Server function captures client-side variable 'X'. Pass it as a parameter instead."

### How the Compiler Determines Module Context

The compiler infers module context from:

1. **File location**: files under `src/server/` are server context. Files under `src/client/` are client context. Files under `src/shared/` are importable from both contexts, subject to Rule 1 (they may not import server-only modules) and Rule 2 (they may not reference DOM APIs).
2. **Explicit annotation**: `@server` and `@client` JSDoc annotations on modules override location-based inference.
3. **Import graph propagation**: if module A imports module B, and module B is in server context, then A is in server context (transitively). This is the import graph walk that the Forge analyzer performs to detect Rule 1 violations.
4. **API usage**: if a module references a WinterTC-only API that is not available in the browser (for example, an API that reads from the server's file system), it is automatically classified as server context.

## Consequences

### Positive

- ✅ **Boundary bugs caught at build time, not runtime**: the most common consequence of boundary mistakes in Next.js is a production runtime error or a security vulnerability discovered after deployment. In Forge, these are compile errors discovered before `git push`.
- ✅ **Security**: the compiler guarantee that server-only code cannot appear in the client bundle is stronger than a convention. A developer cannot accidentally ship their database password to the browser; the compiler will not allow the import chain that would cause it.
- ✅ **No runtime overhead for boundary checks**: enforcement happens at compile time. The deployed binary has no runtime boundary checks. The client bundle is guaranteed to contain only client-safe code; the guarantee was established at compile time.
- ✅ **IDE integration**: boundary violations appear in the IDE as red squiggles, not as build errors discovered on CI. The LSP server uses the same analysis.
- ✅ **Self-documenting**: the boundary rules are compiler-enforced, which means they are precise. Developers cannot misunderstand the boundary by misreading documentation; the compiler tells them exactly what they violated.

### Negative

- ❌ **Compiler complexity**: the import graph analysis, type serialization checking, and closure capture analysis are non-trivial. The analyzer must handle dynamic imports, re-exports, barrel files, and TypeScript-specific type constructs. Edge cases will be discovered in production use and will require compiler updates.
- ❌ **Type-aware checking requires a working TypeScript type checker**: Rule 3 (non-serializable types) requires the compiler to understand TypeScript types, not just syntax. The Oxc type checker is not yet feature-complete (ADR-002). Until it is, Rule 3 is partially enforced using conservative heuristics, with `forge check` using `tsc` for full type verification.
- ❌ **Developers must understand the boundary model**: developers coming from React/Next.js are accustomed to string directives as the boundary mechanism. Forge's compile-time enforcement is more powerful but also less forgiving — boundary mistakes that Next.js would allow (and produce runtime errors for later) are immediate compile errors in Forge. Some developers find this jarring initially.
- ❌ **Strict file organization required**: the location-based context inference (server/, client/, shared/) requires maintaining a consistent file organization. Projects that put everything in a flat `src/` directory will rely more heavily on explicit annotations.

### Neutral

- ℹ️ The boundary rules apply to application code. The Forge runtime itself (the server binary, the FSL) operates outside the boundary model — it is compiled Rust, not JavaScript subject to boundary analysis.
- ℹ️ Third-party packages imported from the Foundry are analyzed for boundary compliance when they are installed, and their context classification is cached. An FSL package that is server-only is marked as such in the Foundry manifest, so the compiler does not need to analyze its import graph on every build.

## Alternatives Considered

### Runtime Enforcement

Detect boundary violations at runtime — when client code tries to execute an import of a server module, throw an error with a helpful message.

This is how most current frameworks handle it (Next.js throws at bundle time or at runtime, depending on the violation type). The problem is that runtime detection means the error is not discovered until the code path is executed. For a security violation (server code leaked to the client), the code path is "the page loads" — but the damage (the secret being in the bundle, potentially cached by CDNs and browsers) is done before the error is observed.

Compile-time enforcement catches the violation before any JavaScript is emitted. Runtime enforcement catches it after the damage may have been done.

Rejected: correct architecture requires compile-time enforcement, not runtime detection.

### tRPC-style Explicit Procedures

Define every server function as an explicit RPC procedure with input and output types. Clients call procedures through a type-safe client that generates the HTTP request.

tRPC is excellent at what it does: providing type safety for explicitly defined API endpoints. The ergonomic problem is the verbosity of the definition. A Forge server function looks like:

```typescript
export async function getUser(id: string): Promise<User> {
  return await db.users.findById(id);
}
```

The equivalent tRPC setup requires defining a router, defining a procedure with `z.input(...)` for input validation, defining the handler, and then importing the tRPC client in the component. Four locations to express one operation.

Forge's model is closer to what developers want — write a function marked as server-side, call it from the client, and let the compiler handle the boundary. The compile-time enforcement provides the correctness guarantees without the tRPC procedural overhead.

Rejected: correct but ergonomically too verbose for the primary use case.

### Convention Only (No Enforcement)

No compiler enforcement. Document the conventions, trust developers to follow them, and provide good error messages for the runtime failures that result.

This is the philosophy of many frameworks and it produces the security vulnerabilities and runtime crashes that motivated this ADR. Forge's position is that correctness properties that can be verified at compile time should be. The boundary is one of those properties.

Rejected: by design.

## Implementation Notes

The boundary analysis lives in `crates/forge-compiler/src/analyze/boundary.rs`.

The analysis is performed in two passes:

1. **Context classification pass**: classify each module as `Client`, `Server`, or `Shared` based on file location, explicit annotations, and recursive import analysis.
2. **Rule enforcement pass**: for each module, check the four boundary rules against its classified context, its imports, its DOM API references, its type signatures at call sites, and its closure capture sets.

Errors are emitted as Forge diagnostic types that carry a source span, the specific rule violated, and a suggested fix.

## Related Decisions

- [ADR-001: Rust-Powered Compiler Pipeline](./001-rust-powered-compiler.md) — the compiler that performs this analysis
- [ADR-002: Oxc Parser Foundation](./002-oxc-parser-foundation.md) — the AST and semantic analysis that boundary enforcement operates on
- [ADR-008: WinterTC-Only Server APIs](./008-wintertc-only-server-apis.md) — the server API surface that boundary analysis uses to classify modules
