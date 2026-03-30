# 005 — Client/Server Boundary Enforcement

**Status:** Normative
**Version:** 0.1.0-pre-alpha
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [Boundary Concepts](#2-boundary-concepts)
   - 2.1 [Module Classification](#21-module-classification)
   - 2.2 [Classification Precedence](#22-classification-precedence)
   - 2.3 [Module Graph Analysis](#23-module-graph-analysis)
3. [Compile Errors](#3-compile-errors)
   - 3.1 [E001 — Server Import in Client Code](#31-e001--server-import-in-client-code)
   - 3.2 [E002 — DOM API in Server Code](#32-e002--dom-api-in-server-code)
   - 3.3 [E003 — Non-Serializable Type Crossing Boundary](#33-e003--non-serializable-type-crossing-boundary)
   - 3.4 [E004 — Closure Capture Across Boundary](#34-e004--closure-capture-across-boundary)
4. [Serialization Whitelist](#4-serialization-whitelist)
   - 4.1 [Whitelisted Types](#41-whitelisted-types)
   - 4.2 [Rejected Types](#42-rejected-types)
   - 4.3 [Recursive Composition](#43-recursive-composition)
   - 4.4 [Type-Level Enforcement](#44-type-level-enforcement)
5. [RPC Stub Generation](#5-rpc-stub-generation)
   - 5.1 [Stub Contract](#51-stub-contract)
   - 5.2 [Generated Code](#52-generated-code)
   - 5.3 [Stub File Location](#53-stub-file-location)
6. [HTTP Handler Registration](#6-http-handler-registration)
   - 6.1 [Route Naming](#61-route-naming)
   - 6.2 [Request Handling](#62-request-handling)
   - 6.3 [Error Handling](#63-error-handling)
7. [DOM API Blocklist](#7-dom-api-blocklist)
8. [Boundary Enforcement for FSL Packages](#8-boundary-enforcement-for-fsl-packages)
9. [Incremental Analysis](#9-incremental-analysis)

---

## 1. Purpose

Forge enforces a strict compile-time boundary between code that runs on the client (browser) and code that runs on the server or edge. This enforcement:

- Prevents accidental exposure of server-only secrets, database credentials, or internal business logic to the client bundle.
- Prevents use of browser-only APIs (DOM, `window`, `document`) in server contexts where they do not exist.
- Ensures that all data crossing the client/server boundary is JSON-serializable, eliminating a class of runtime errors.
- Makes the boundary explicit and auditable through the type system.

Boundary enforcement is a **compile-time** guarantee. If a Forge project compiles without boundary errors, it is structurally impossible for server implementation code to appear in the client bundle or for client DOM code to execute in the server runtime.

---

## 2. Boundary Concepts

### 2.1 Module Classification

Every module in a Forge project is classified into exactly one of three categories:

**Server module** — A module is classified as server if any of the following are true:

- It contains the `"use module server"` directive.
- It contains at least one `server` function declaration (even without the directive).
- It imports from a module that is itself classified as server and does not re-export those imports in a way that would make them accessible to client code.

**Client module** — A module is classified as client if any of the following are true:

- It contains the `"use module client"` directive.
- It contains at least one `export component` declaration.
- It is the application entry module (`app/root.fx`).
- It directly uses a DOM API (see §7 for the blocklist) — classified as client with a warning if no explicit directive is present.

**Shared module** — A module is classified as shared if:

- It has no module directive.
- It contains no `export component` declarations.
- It contains no `server` function declarations.
- It uses no DOM APIs.
- It does not import from any server module.

Shared modules are compiled into both the client bundle and the server bundle. They must be safe to execute in both environments.

---

### 2.2 Classification Precedence

If a module contains conflicting signals (e.g. both a `server` function and a `export component`), the compiler emits an error before classification:

```text
error[E009]: module contains both server functions and component declarations
  --> app/pages/Profile.fx:5:1
   |
 5 | export component Profile(...) { ... }
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   ...
10 | export const save = server async function(...) { ... }
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: move server functions to a separate server module and import the generated RPC stub
```

---

### 2.3 Module Graph Analysis

The `boundary_analysis` compiler pass builds the full module import graph and propagates classifications:

1. Classify all modules with explicit directives or `export component` declarations first.
2. For each unclassified module, inspect its imports:
   - If it imports from a server module: classify as server.
   - If it imports from a client module: classify as client.
   - If it imports from both: compile error E001.
3. Repeat until stable (fixed-point iteration).
4. Remaining unclassified modules are shared.
5. Validate all edges: server → client (E001), client → server (E001), server → DOM API (E002).

**Cycle detection:** Circular imports within the same boundary (all client or all server) are allowed. Circular imports that cross the boundary are a compile error.

---

## 3. Compile Errors

### 3.1 E001 — Server Import in Client Code

**Trigger:** A client module or shared module directly imports a symbol from a server module.

**Error format:**

```text
error[E001]: server module imported from client context
  --> app/pages/Home.fx:3:1
   |
 3 | import { getUserSecret } from '../server/users.fx'
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: 'server/users.fx' is a server module (contains server functions)
   = help: use the generated RPC stub at '.forge/rpc/server/users.ts' instead,
           or wrap this call in a server function that returns only serializable data
```

**Also triggered by:**

- Importing a module that transitively imports a server module.
- Type-only imports (`import type`) from server modules are **not** an error — types are erased at compile time and do not appear in any bundle.

**Resolution:** Replace the direct import with an import from the generated RPC stub at `.forge/rpc/<module-path>.ts`.

---

### 3.2 E002 — DOM API in Server Code

**Trigger:** A server module or shared module uses a DOM API (see §7 for the full blocklist).

**Error format:**

```text
error[E002]: DOM API used in server module
  --> server/render.fx:12:3
   |
12 |   document.getElementById('root')
   |   ^^^^^^^^
   |
   = note: 'document' is a browser-only API not available in the server runtime
   = note: 'server/render.fx' is classified as a server module
   = help: if this file must use DOM APIs, add '"use module client"' or
           move this code to a client module
```

**Scope of detection:** The check operates on the module as classified, not just on the literal file content. If a shared module (with no directive) uses `window.location`, it is reclassified as client and a warning is emitted recommending the explicit `"use module client"` directive.

---

### 3.3 E003 — Non-Serializable Type Crossing Boundary

**Trigger:** A `server` function's parameter types or return type contain a type that is not on the serialization whitelist (§4).

**Error format (return type):**

```text
error[E003]: non-serializable return type in server function
  --> server/users.fx:8:47
   |
 8 | export const getUser = server async function(): Promise<UserService> {
   |                                                               ^^^^^^^^^^^
   |
   = note: 'UserService' is a class instance type; class instances are not JSON-serializable
   = note: server functions may only return JSON-compatible types, File, Blob, or ReadableStream
   = help: return a plain object type instead:
           interface UserData { id: string; name: string; email: string }
```

**Error format (parameter type):**

```text
error[E003]: non-serializable parameter type in server function
  --> server/users.fx:15:47
   |
15 | export const updateUser = server async function(svc: UserService): Promise<void> {
   |                                                       ^^^^^^^^^^^
   |
   = note: 'UserService' is not a serializable type
```

**What triggers E003:**

- Class instance types (not interfaces or plain object types).
- Function types.
- Symbol types.
- `undefined` as a return type (use `null` instead; `undefined` serializes inconsistently across JSON implementations).
- `bigint` (not JSON-serializable by default).
- `Map`, `Set`, `WeakMap`, `WeakRef` (not JSON-serializable).
- Recursive types that contain any of the above.

---

### 3.4 E004 — Closure Capture Across Boundary

**Trigger:** A `server` function closes over a variable that is defined in client scope (i.e. inside a component body or a client-only scope), creating an implicit data dependency that cannot be serialized for the RPC call.

**Error format:**

```text
error[E004]: server function closes over client-scoped variable
  --> app/pages/Profile.fx:15:20
   |
13 |   const formData = new FormData(formRef)
   |   -------- client-scoped variable defined here
   |
15 |   const save = server async function() {
   |                ^^^^^^^^^^^^^^^^^^^^^^
16 |     await db.profiles.update(formData)
   |                              ^^^^^^^^ closes over 'formData' here
   |
   = note: server functions are executed on the server and cannot access client-scoped variables
   = help: pass 'formData' as an explicit parameter to the server function
```

**Explicit parameters are allowed:** If the closed-over variable is passed as an explicit parameter, it must satisfy the serialization whitelist (checked by E003). The compiler enforces this transitively.

**Module-level variables from shared modules are allowed:** A `server` function may reference a module-level constant from a shared module because shared module code is available in both bundles.

---

## 4. Serialization Whitelist

### 4.1 Whitelisted Types

The following TypeScript types are permitted as server function parameters and return types:

| Type | Notes |
|------|-------|
| `string` | Any string value |
| `number` | `NaN` and `Infinity` are serialized as `null` per JSON spec |
| `boolean` | `true` or `false` |
| `null` | Serializes as JSON `null` |
| `undefined` | Only allowed in parameter position; serializes as omitted JSON key |
| Plain object types | Interface or object type alias with all-whitelisted property types |
| Arrays | `T[]` where `T` is whitelisted |
| Tuples | `[T1, T2, ...]` where all elements are whitelisted |
| `Date` | Serialized as ISO 8601 string; deserialized back to `Date` by the stub |
| `File` | Serialized via multipart form data (special handling) |
| `Blob` | Serialized via multipart form data (special handling) |
| `ReadableStream` | Streamed response (only valid as the sole return type, not nested in an object) |
| `Uint8Array` | Serialized as base64 string |
| `Record<string, T>` | Where `T` is whitelisted |
| Union types | `T1 \| T2 \| ...` where all members are whitelisted |
| Optional properties | `{ x?: T }` where `T` is whitelisted |

### 4.2 Rejected Types

| Type | Reason |
|------|--------|
| Class instances | Not JSON-serializable; prototype chain is lost |
| Functions | Cannot be serialized |
| `Symbol` | Cannot be serialized |
| `bigint` | Not JSON-serializable by default |
| `Map<K, V>` | Use `Record<string, V>` instead |
| `Set<T>` | Use `T[]` instead |
| `WeakMap`, `WeakRef`, `WeakSet` | Cannot be serialized |
| `Promise<T>` in parameter position | Cannot be serialized |
| `HTMLElement`, `Node`, DOM types | Client-only; cannot cross boundary |
| `Signal.State`, `Signal.Computed` | Runtime objects; not serializable |
| `Error` | Use a plain object `{ message: string }` instead |
| Circular object types | Would cause infinite serialization loop |

### 4.3 Recursive Composition

The whitelist check is applied **recursively**. A plain object type `{ user: UserProfile }` is only whitelisted if `UserProfile` itself recursively satisfies the whitelist. Circular type references (directly or through structural types) are detected and rejected.

### 4.4 Type-Level Enforcement

The whitelist is enforced at the TypeScript type level using a compile-time type traversal. This means:

- `type UserData = { name: string; createdAt: Date }` — allowed.
- `type UserData = { name: string; service: UserService }` — rejected (class instance property).
- Generic server functions are validated at the usage site with concrete type arguments.

---

## 5. RPC Stub Generation

### 5.1 Stub Contract

For each `server` function that passes boundary validation (E001–E004), the compiler generates a client-side RPC stub. The stub:

- Has the same name as the server function.
- Has the same parameter types and return type.
- Is `async` (even if the original function was synchronous, the RPC call is always async).
- Is placed in `.forge/rpc/<module-path>.ts`.
- Is regenerated on every build; must not be edited.

Consumers import from the stub path, not from the server module directly.

### 5.2 Generated Code

```typescript
// .forge/rpc/server/users.ts
// Generated by Forge v0.1.0-pre-alpha — do not edit
// Source: server/users.fx

import type { User } from '../../server/users.fx'

export async function getUser(id: string): Promise<User> {
  const response = await fetch('/_forge/rpc/server/users/getUser', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ args: [id] }),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(`RPC error [server/users/getUser]: ${text}`)
  }
  return response.json() as Promise<User>
}

export async function createUser(
  name: string,
  email: string
): Promise<User> {
  const response = await fetch('/_forge/rpc/server/users/createUser', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ args: [name, email] }),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(`RPC error [server/users/createUser]: ${text}`)
  }
  return response.json() as Promise<User>
}
```

**`import type`:** The stub imports the return type using `import type` so that the type is available for TypeScript checking but no server module code is included in the client bundle.

**`File` and `Blob` parameters:** When a parameter is `File` or `Blob`, the stub switches to multipart form data:

```typescript
// Generated stub for: server async function uploadFile(file: File): Promise<{ url: string }>
export async function uploadFile(file: File): Promise<{ url: string }> {
  const form = new FormData()
  form.append('__forge_arg_0', file)
  const response = await fetch('/_forge/rpc/server/uploads/uploadFile', {
    method: 'POST',
    body: form,
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(`RPC error [server/uploads/uploadFile]: ${text}`)
  }
  return response.json()
}
```

**`ReadableStream` return type:** When the return type is `ReadableStream`, the stub returns `response.body` directly:

```typescript
export async function streamData(query: string): Promise<ReadableStream<Uint8Array>> {
  const response = await fetch('/_forge/rpc/server/data/streamData', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ args: [query] }),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(`RPC error [server/data/streamData]: ${text}`)
  }
  return response.body!
}
```

### 5.3 Stub File Location

Stubs are written to `.forge/rpc/` mirroring the source module path relative to the project root, with `.fx` replaced by `.ts`:

| Server module | Generated stub |
|---------------|---------------|
| `server/users.fx` | `.forge/rpc/server/users.ts` |
| `services/auth/tokens.fx` | `.forge/rpc/services/auth/tokens.ts` |
| `app/api/search.fx` | `.forge/rpc/app/api/search.ts` |

The `.forge/` directory is generated output and must be added to `.gitignore`.

---

## 6. HTTP Handler Registration

### 6.1 Route Naming

Server functions are exposed at:

```text
POST /_forge/rpc/<module-path>/<function-name>
```

Where:

- `<module-path>` is the server module's file path relative to the project root, with the `.fx` extension removed and directory separators preserved as `/`.
- `<function-name>` is the export name of the `server` function.

**Examples:**

| Server function | HTTP path |
|-----------------|-----------|
| `getUser` in `server/users.fx` | `/_forge/rpc/server/users/getUser` |
| `createUser` in `server/users.fx` | `/_forge/rpc/server/users/createUser` |
| `search` in `services/search/index.fx` | `/_forge/rpc/services/search/index/search` |

### 6.2 Request Handling

**Method:** POST only. GET requests to `/_forge/rpc/*` return 405 Method Not Allowed.

**Content-Type for JSON:** `application/json`. The body is `{ "args": [...] }` where `args` is a positional array of the function's arguments.

**Content-Type for file uploads:** `multipart/form-data`. Arguments are named `__forge_arg_0`, `__forge_arg_1`, etc. Non-file arguments are serialized as JSON strings within the form.

**Argument binding:** Arguments are extracted from the `args` array by position and coerced to the declared TypeScript parameter types. Type coercion errors (e.g. a string where a number is expected) return 400 Bad Request.

### 6.3 Error Handling

| Condition | HTTP Status | Response Body |
|-----------|-------------|---------------|
| Success | 200 | JSON-serialized return value |
| Invalid JSON body | 400 | `{ "error": "invalid_request", "message": "..." }` |
| Argument type mismatch | 400 | `{ "error": "invalid_args", "message": "..." }` |
| Server function throws | 500 | `{ "error": "<ErrorClass>", "message": "<message>", "stack": "..." (dev only) }` |
| Unknown route | 404 | `{ "error": "not_found", "message": "No RPC handler at this path" }` |

**Production vs. development:** Stack traces are included in 500 responses only when `NODE_ENV !== 'production'`. In production, only the error class name and message are included.

---

## 7. DOM API Blocklist

The following global identifiers and namespaces are blocked in server and shared modules:

**Global objects:**
`window`, `document`, `navigator`, `location`, `history`, `screen`, `performance` (browser), `localStorage`, `sessionStorage`, `indexedDB`, `caches` (Service Worker cache), `crypto` (browser `window.crypto` — use `globalThis.crypto.subtle` which is available in WinterTC)

**DOM constructors:**
`HTMLElement`, `Element`, `Node`, `DocumentFragment`, `MutationObserver`, `IntersectionObserver`, `ResizeObserver`, `PerformanceObserver`, `XMLHttpRequest`

**DOM events:**
`CustomEvent` (blocked in server — available in WinterTC edge/client), `MouseEvent`, `KeyboardEvent`, `PointerEvent`, `TouchEvent`, `FocusEvent`, `InputEvent`

**Layout/rendering:**
`requestAnimationFrame`, `cancelAnimationFrame`, `requestIdleCallback`, `cancelIdleCallback`

**Note:** `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, `queueMicrotask`, and `structuredClone` are available in all environments (see spec 006) and are not blocked.

---

## 8. Boundary Enforcement for FSL Packages

FSL (`forge:*`) packages are themselves classified as server-only, client-only, or shared:

| Package | Classification |
|---------|---------------|
| `forge:db` | Server only |
| `forge:env` | Shared (read-only env access) |
| `forge:router` | Shared |
| `forge:crypto` | Shared |
| `forge:fs` | Server only |
| `forge:kv` | Server only |
| `forge:email` | Server only |

Importing a server-only FSL package from a client module triggers E001:

```text
error[E001]: server package imported from client context
  --> app/pages/Home.fx:2:1
   |
 2 | import { db } from 'forge:db'
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: 'forge:db' is a server-only FSL package (database access is not available in the browser)
   = help: create a server function that performs the database query and returns the data you need
```

---

## 9. Incremental Analysis

The `boundary_analysis` pass supports incremental compilation. The module graph and boundary classifications are cached and invalidated only when:

- A module's directive changes.
- A `server` function is added to or removed from a module.
- An import edge is added or removed.
- A DOM API usage is added to or removed from a module.

On incremental builds, only the affected portion of the module graph is re-analyzed. Error messages reference the original source spans via source maps.
