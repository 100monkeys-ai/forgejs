# 001 — `.fx` File Format

**Status:** Normative
**Version:** 0.1.0-pre-alpha
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [Relationship to TypeScript](#2-relationship-to-typescript)
3. [Module Directives](#3-module-directives)
4. [Component Declarations](#4-component-declarations)
5. [Server Functions](#5-server-functions)
6. [Reactive Primitives](#6-reactive-primitives)
7. [JSX in `.fx` Files](#7-jsx-in-fx-files)
8. [Import Resolution](#8-import-resolution)
9. [BNF Grammar for `.fx` Extensions](#9-bnf-grammar-for-fx-extensions)
10. [Compile-Time Processing Order](#10-compile-time-processing-order)

---

## 1. Purpose

A `.fx` file is a TypeScript source file extended with three additional syntactic constructs that the Forge compiler recognizes:

- **Component declarations** — first-class UI component syntax
- **Server function annotations** — boundary-crossing RPC declarations
- **Reactive primitive syntax** — TC39 Signals sugar (`$signal`, `$derived`, `$async`, `$effect`)

Every construct valid in TypeScript is valid in a `.fx` file. The Forge compiler processes `.fx` files through its own parse front-end before handing off to the TypeScript type-checker. The TypeScript language server receives a desugared `.ts` representation so that IDE tooling (completions, diagnostics, go-to-definition) works without modification.

`.fx` is not a dialect — it is TypeScript with a recognized extension surface. No TypeScript semantics are altered.

---

## 2. Relationship to TypeScript

### 2.1 Superset Contract

The `.fx` extension surface is purely additive. The following invariants hold:

1. Every valid `.ts` or `.tsx` file, when renamed to `.fx`, is a valid `.fx` file.
2. No TypeScript keyword, operator, or grammar production is redefined.
3. The `.fx`-specific constructs desugar to valid TypeScript before the type-checker runs.
4. Type errors from desugared output are reported with source-mapped spans pointing into the original `.fx` source.

### 2.2 Parser Pipeline

```text
.fx source
   │
   ▼
forge-parser (extends TS parser with fx productions)
   │
   ▼
fx-specific AST nodes (ComponentDecl, ServerFn, ReactiveCall)
   │
   ▼
desugar pass → standard TS AST
   │
   ▼
tsc type-checker (full TypeScript semantics)
   │
   ▼
transform passes (signal_transform, boundary_analysis, dom_wire)
   │
   ▼
codegen (client bundle / server bundle)
```

### 2.3 File Extension Rules

| Extension | Parser Behaviour |
|-----------|-----------------|
| `.fx` | Forge parser; JSX enabled; all `.fx` constructs active |
| `.ts` | Standard TypeScript parser; no `.fx` constructs |
| `.tsx` | Standard TypeScript parser with JSX; no `.fx` constructs |
| `.js` / `.mjs` | Treated as ambient imports only; not compiled by Forge |

---

## 3. Module Directives

A module directive is a string literal on the first non-import, non-comment statement of a file. Module directives control which runtime environment a module is compiled for.

### 3.1 `"use module server"`

```typescript
"use module server"

import { db } from 'forge:db'

export async function getUser(id: string) {
  return db.users.find(id)
}
```

**Semantics:**

- The entire module is assigned to the **server bundle** exclusively.
- No symbols from this module are included in the client bundle.
- All exports are automatically eligible for RPC stub generation if they satisfy the serialization constraints (see spec 005).
- Importing a `"use module server"` file from a client module is a compile error (E001).

### 3.2 `"use module client"`

```typescript
"use module client"

export component Spinner() {
  return <div class="spinner" />
}
```

**Semantics:**

- The entire module is assigned to the **client bundle** exclusively.
- Importing a `"use module client"` file from a server module is a compile error.
- DOM APIs are freely available.
- `server` function syntax is forbidden in this module (use a separate server module and import the generated stub).

### 3.3 No Directive (Shared Module)

A file with no module directive is a **shared module**. Shared modules:

- May be imported from both client and server contexts.
- Must not use DOM APIs (triggers E002 if they do and are imported from a server module).
- Must not contain `server` function declarations (use `"use module server"` instead).
- Are compiled into both bundles; tree-shaking removes unused exports per target.

### 3.4 Implicit Client Modules

Any file containing `export component` declarations is implicitly a client module even if no `"use module client"` directive is present. The compiler inserts the implicit directive and emits a warning recommending the explicit form.

### 3.5 Entry Module

The root entry (`app/root.fx` by default, configurable via `forge.toml`) is always implicitly a client module. Its module directive is ignored if present and a warning is emitted.

---

## 4. Component Declarations

### 4.1 Syntax

```typescript
export component UserCard({ name, age }: { name: string; age: number }) {
  return (
    <div class="card">
      <h2>{name}</h2>
      <span>{age}</span>
    </div>
  )
}
```

The `component` keyword replaces `function` in the declaration. The remainder of the syntax is identical to a TypeScript function declaration: an optional generic parameter list, a required parameter list with a destructuring pattern, an optional explicit return type annotation, and a function body.

### 4.2 Desugaring

The `component` keyword desugars as follows:

```typescript
// Source:
export component UserCard({ name }: { name: string }) {
  return <div>{name}</div>
}

// Desugared (type-checker input):
export function UserCard({ name }: { name: string }): HTMLElement {
  // JSX is further desugared by the dom_wire pass — see section 7
  return __forge_jsx('div', null, name)
}
```

The inferred return type is always `HTMLElement`. If an explicit return type annotation other than `HTMLElement` is provided, the compiler emits a type error.

### 4.3 Distinction from Regular Functions

A component declaration differs from a plain `function` in the following ways:

| Property | `component` | `function` |
|----------|-------------|------------|
| Return type | Always `HTMLElement` | Any |
| JSX wiring | Direct DOM ops via `dom_wire` pass | `React.createElement` (if configured) |
| Signal tracking | Compile-time dependency graph | None |
| Disposal | Auto-disposed on unmount | Manual |
| Callable as RPC | Never | If in server module |

### 4.4 Generic Components

```typescript
export component List<T>({
  items,
  render,
}: {
  items: T[]
  render: (item: T) => HTMLElement
}) {
  const el = document.createElement('ul')
  for (const item of items) {
    const li = document.createElement('li')
    li.appendChild(render(item))
    el.appendChild(li)
  }
  return el
}
```

Generic parameters are fully supported. The type-checker resolves them at call sites.

### 4.5 Async Components

Components may not be `async`. Asynchronous data must be loaded via `$async` signals and rendered reactively. This is a hard compile error:

```text
error[E005]: component function may not be async
  --> app/pages/Profile.fx:3:1
   |
 3 | export async component Profile({ id }: { id: string }) {
   |        ^^^^^
```

### 4.6 Nesting and Composition

Components compose by calling one component inside another's JSX:

```typescript
export component App() {
  return (
    <main>
      <UserCard name="Jeshua" age={40} />
    </main>
  )
}
```

`<UserCard />` desugars to a call to the `UserCard` function, which returns an `HTMLElement` that is inserted into the parent DOM tree by the `dom_wire` pass.

---

## 5. Server Functions

### 5.1 Syntax

```typescript
export const getUser = server async function(id: string): Promise<User> {
  return await db.users.find(id)
}
```

The `server` keyword is placed immediately before the `function` keyword (or `async function`). The binding must be a `const` declaration. Named function statement form is not supported for server functions.

### 5.2 What `server` Does

The Forge compiler performs four operations on a `server`-annotated function:

1. **Implementation stripping**: The function body is removed entirely from the client bundle. The client receives only the generated RPC stub (see section 5.4).
2. **Type extraction**: The parameter types and return type are extracted from the TypeScript AST. These types must satisfy the serialization whitelist (spec 005 §Serialization Whitelist). If they do not, compiler error E003 is emitted.
3. **HTTP handler registration**: An HTTP POST handler is registered at the path `/_forge/rpc/<module-path>/<export-name>`. The `<module-path>` is the module's file path relative to the project root with slashes and the `.fx` extension removed (e.g. `server/users` for `server/users.fx`).
4. **RPC stub generation**: A typed async function is generated in the `.forge/rpc/` directory (see section 5.4).

### 5.3 Module Scope Requirement

Server functions must be declared at module scope. Declaring a server function inside a component, another function, or a class is a compile error (E004 variant):

```text
error[E004]: server function must be declared at module scope
  --> app/pages/Home.fx:10:3
   |
10 |   const save = server async function() { ... }
   |   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

### 5.4 RPC Stub Generation

For each validated server function, the compiler writes a stub file to `.forge/rpc/<module-path>.ts`. The stub is regenerated on every build; it must not be edited manually.

```typescript
// .forge/rpc/server/users.ts
// Generated by Forge — do not edit
// Source: server/users.fx :: getUser

import type { User } from '../../server/users.fx'

export async function getUser(id: string): Promise<User> {
  const response = await fetch('/_forge/rpc/server/users/getUser', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ args: [id] }),
  })
  if (!response.ok) {
    const text = await response.text()
    throw new Error(`RPC error (getUser): ${text}`)
  }
  return response.json() as Promise<User>
}
```

### 5.5 HTTP Handler Contract

**Path:** `POST /_forge/rpc/<module-path>/<function-name>`

**Request:**

```json
{
  "args": [ /* positional argument values, JSON-serialized */ ]
}
```

**Success Response (200):**

The raw return value of the server function, JSON-serialized. `Content-Type: application/json`.

**Error Response (500):**

```json
{
  "error": "<error class name>",
  "message": "<error message>"
}
```

Stack traces are included only when `NODE_ENV !== 'production'`.

### 5.6 Synchronous Server Functions

Server functions may be synchronous (non-`async`) if they return a non-Promise type:

```typescript
export const formatName = server function(first: string, last: string): string {
  return `${last}, ${first}`
}
```

The generated stub is still `async` because the RPC call is always a network round-trip.

---

## 6. Reactive Primitives

Forge reactive primitives are syntactic sugar over the TC39 Signals proposal. They are only valid inside `.fx` files. Using them in `.ts` files is a compile error.

The full formal semantics are in spec 004. This section defines the syntax and desugaring rules.

### 6.1 `$signal` — Mutable State

**Syntax:**

```typescript
const count = $signal(0)
const name = $signal<string>("")
```

**Desugars to:**

```typescript
const count = new Signal.State(0)
const name = new Signal.State<string>("")
```

The inferred type of `count` is `Signal.State<number>`. The type parameter is inferred from the initial value or may be supplied explicitly: `$signal<number>(0)`.

**Reading a signal** (the `$` prefix read syntax):

Inside a reactive context (component body, `$derived`, `$effect`), prepending `$` to a signal binding reads its current value:

```typescript
// Source:
const doubled = $derived(() => $count * 2)

// Desugars to:
const doubled = new Signal.Computed(() => count.get() * 2)
```

The `$` read prefix is only syntactically valid in reactive contexts. Using it outside a reactive context is a compile error:

```text
error[E006]: $-read syntax used outside reactive context
  --> app/pages/Home.fx:20:18
   |
20 |   console.log($count)
   |                ^^^^^^
   = help: use count.get() to read a signal outside a reactive context
```

**Writing a signal:**

```typescript
count.set(5)
count.set(prev => prev + 1)
```

`.set()` is not syntactic sugar — it is the direct `Signal.State` API and is called as a normal method.

### 6.2 `$derived` — Computed Signal

**Syntax:**

```typescript
const doubled = $derived(() => $count * 2)
const fullName = $derived(() => `${$first} ${$last}`)
```

**Desugars to:**

```typescript
const doubled = new Signal.Computed(() => count.get() * 2)
const fullName = new Signal.Computed(() => `${first.get()} ${last.get()}`)
```

`$derived` takes a zero-argument function. The function body is a reactive context: `$` reads are tracked as dependencies. When any dependency changes, the computed value is marked stale and re-evaluated on next read.

### 6.3 `$async` — Async Derived Signal

**Syntax:**

```typescript
const user = $async(() => fetchUser($userId))
```

**State Machine:**

An `$async` signal is a `Signal.Computed` that wraps a Promise. It exposes:

```typescript
type AsyncSignal<T> = {
  readonly state: 'pending' | 'ready' | 'error'
  readonly value: T | undefined   // defined when state === 'ready'
  readonly error: Error | undefined  // defined when state === 'error'
}
```

**Desugars to (conceptual):**

```typescript
const user = new Signal.Computed<AsyncSignal<User>>(() => {
  const promise = fetchUser(userId.get())
  const result = new Signal.State<AsyncSignal<User>>({ state: 'pending', value: undefined, error: undefined })
  promise.then(
    value => result.set({ state: 'ready', value, error: undefined }),
    error => result.set({ state: 'error', value: undefined, error: error instanceof Error ? error : new Error(String(error)) })
  )
  return result.get()
})
```

The actual implementation uses an internal scheduler to prevent redundant re-fetches when upstream signals change rapidly (debounced via microtask queue).

**Usage in JSX:**

```typescript
export component UserProfile({ id }: { id: string }) {
  const userId = $signal(id)
  const user = $async(() => fetchUser($userId))

  return (
    <div>
      {$user.state === 'pending' && <Spinner />}
      {$user.state === 'ready' && <UserCard name={$user.value!.name} />}
      {$user.state === 'error' && <ErrorBanner message={$user.error!.message} />}
    </div>
  )
}
```

### 6.4 `$effect` — Side Effect

**Syntax:**

```typescript
$effect(() => {
  document.title = `Count: ${$count}`
})

$effect(() => {
  const handler = () => count.set(prev => prev + 1)
  window.addEventListener('click', handler)
  return () => window.removeEventListener('click', handler)  // cleanup
})
```

**Desugars to:**

```typescript
{
  const watcher = new Signal.subtle.Watcher(() => {
    // schedule re-run on next microtask
    queueMicrotask(() => watcher.run())
  })
  const run = () => {
    const cleanup = (() => {
      document.title = `Count: ${count.get()}`
    })()
    // store cleanup fn if returned
  }
  watcher.watch(.../* tracked signals from run() */)
  run()
  // registered for disposal on component unmount
}
```

**Cleanup:** If the effect function returns a function, that function is called before the effect re-runs (when dependencies change) and when the owning component unmounts.

**Restriction:** `$effect` is only valid inside a component body or at the module top level. Inside a module, effects run when the module is first imported and are never disposed (they live for the application lifetime). Inside a component, effects are disposed on unmount.

---

## 7. JSX in `.fx` Files

### 7.1 JSX is Always Direct DOM

In `.fx` files, JSX does not compile to `React.createElement`. It compiles to direct DOM operations via the `dom_wire` compiler pass. There is no virtual DOM.

```typescript
// Source JSX:
return (
  <div class="card">
    <h1>{$title}</h1>
  </div>
)

// Compiled output (conceptual):
const __root = document.createElement('div')
__root.className = 'card'
const __h1 = document.createElement('h1')
__root.appendChild(__h1)

// Signal wiring — runs once, patches DOM on change:
const __watcher_0 = new Signal.subtle.Watcher(() => {
  queueMicrotask(() => {
    __h1.textContent = title.get()
  })
})
__watcher_0.watch(title)
__h1.textContent = title.get()

return __root
```

### 7.2 Attribute Mapping

JSX attributes map to DOM properties and attributes as follows:

| JSX Attribute | DOM Equivalent |
|---------------|---------------|
| `class` | `element.className` |
| `for` | `element.htmlFor` |
| `onClick` | `element.addEventListener('click', ...)` |
| `onInput` | `element.addEventListener('input', ...)` |
| `style` (object) | `Object.assign(element.style, ...)` |
| `ref` | Invoked with the element reference after mount |
| All others | `element.setAttribute(name, value)` |

Event handler attributes (`on*`) always use `addEventListener`, never `element.onclick`. This ensures multiple handlers can be attached to the same element without overwriting.

### 7.3 Signal Reads in JSX

Any `$signal` or `$derived` read (`$count`, `$user.value`, etc.) inside a JSX expression is automatically wired by the `signal_transform` pass to update only the affected DOM node when the signal changes. The component function body runs once; only the wired DOM patches run on subsequent updates.

### 7.4 JSX Children

- **String literals**: Compiled to `document.createTextNode(...)`.
- **Expressions**: Compiled to `document.createTextNode(String(...))` with signal wiring if the expression contains `$` reads.
- **`HTMLElement` values**: Appended directly.
- **Arrays**: Each element processed in order; arrays of `HTMLElement` are supported.
- **`null`, `undefined`, `false`**: Rendered as empty (no DOM node created).

### 7.5 Fragments

```typescript
return (
  <>
    <h1>Title</h1>
    <p>Body</p>
  </>
)
```

Fragments compile to a `DocumentFragment`. The Forge runtime unwraps fragments when inserting into a parent.

---

## 8. Import Resolution

### 8.1 Relative Imports

```typescript
import { getUser } from './server/users.fx'
import { UserCard } from '../components/UserCard.fx'
```

`.fx` extension must be explicit in import specifiers. Extensionless imports are not supported for `.fx` files.

### 8.2 `forge:*` Package Imports

```typescript
import { db } from 'forge:db'
import { env } from 'forge:env'
import { router } from 'forge:router'
```

Specifiers beginning with `forge:` resolve to FSL (Forge Standard Library) packages. These are not publishable to the Foundry registry; they are provided by the Forge runtime. FSL packages may be server-only, client-only, or shared — the compiler enforces boundary rules for them identically to user modules.

### 8.3 Foundry Package Imports

```typescript
import { format } from 'jeshua/date-utils'
```

Foundry package imports use the `author/name` format. The compiler resolves them to the version pinned in `foundry.lock` and the source extracted to `.forge/packages/`.

### 8.4 Module Boundary Analysis

At the start of each compilation, the compiler builds a **module graph** and assigns each module a boundary:

1. Walk all import edges from the entry point.
2. For each module, determine its boundary:
   - `"use module server"` → server-only
   - `"use module client"` or contains `export component` → client-only
   - Otherwise → shared
3. Validate all import edges against the boundary rules (see spec 005).
4. Assign modules to bundles.

Circular imports that cross boundaries are a compile error.

---

## 9. BNF Grammar for `.fx` Extensions

The following BNF productions extend the TypeScript grammar. Non-terminals in `<UpperCamelCase>` refer to existing TypeScript grammar productions.

```bnf
(* Module directive — must appear before any non-import statement *)
ModuleDirective
  ::= '"use module server"' ';'?
    | '"use module client"' ';'?

(* Component declaration — extends FunctionDeclaration *)
ComponentDeclaration
  ::= ExportKeyword? 'component' BindingIdentifier TypeParameters?
      '(' ComponentParameterList ')' (':' TypeAnnotation)?
      FunctionBody

ComponentParameterList
  ::= DestructuringPattern ':' TypeAnnotation
    | ε

(* Server function — extends VariableDeclaration *)
ServerFunctionExpression
  ::= 'server' 'async'? 'function' TypeParameters?
      '(' FormalParameterList ')' (':' TypeAnnotation)?
      FunctionBody

ServerFunctionDeclaration
  ::= 'const' BindingIdentifier '=' ServerFunctionExpression ';'?

(* Reactive primitives — syntactic forms of CallExpression *)
SignalCall
  ::= '$signal' TypeArguments? '(' AssignmentExpression ')'

DerivedCall
  ::= '$derived' TypeArguments? '(' ArrowFunctionExpression ')'

AsyncCall
  ::= '$async' TypeArguments? '(' ArrowFunctionExpression ')'

EffectCall
  ::= '$effect' '(' ArrowFunctionExpression ')'

(* Signal read — only valid in reactive contexts *)
SignalRead
  ::= '$' IdentifierName MemberExpressionChain?

MemberExpressionChain
  ::= ('.' IdentifierName | '[' Expression ']')*
```

**Precedence note:** `$identifier` has higher precedence than member access. `$user.value` is parsed as `($user).value`, which desugars to `user.get().value`.

---

## 10. Compile-Time Processing Order

The Forge compiler processes a `.fx` file through the following ordered passes:

| Pass | Name | Responsibility |
|------|------|----------------|
| 1 | `parse` | Parse `.fx` extensions; produce augmented AST |
| 2 | `directive_analysis` | Identify module directives; classify module boundary |
| 3 | `desugar` | Desugar `component`, `server`, `$signal`, `$derived`, `$async`, `$effect`, `$read` |
| 4 | `typecheck` | Full TypeScript type-checking on desugared AST |
| 5 | `boundary_analysis` | Validate import edges across server/client boundaries; emit E001–E004 |
| 6 | `signal_transform` | Analyze JSX in component bodies; generate `Signal.subtle.Watcher` subscriptions |
| 7 | `dom_wire` | Replace JSX nodes with direct DOM construction calls |
| 8 | `rpc_stub_gen` | Generate `.forge/rpc/**/*.ts` stubs for server functions |
| 9 | `bundle` | Tree-shake and bundle into client and server outputs |

Passes 1–5 are blocking: if any emit errors, subsequent passes do not run.
