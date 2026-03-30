# .fx Syntax

`.fx` files are TypeScript with Forge-specific extensions. The compiler processes `.fx` files; standard `.ts` and `.tsx` files work as-is within a Forge project.

## Module Directives

Module directives are string literals at the top of a file that declare the module's execution context. They are checked by the compiler, not evaluated at runtime.

```typescript
"use module server"    // Runs only on the server. Cannot be imported by client modules (except for function calls).
"use module client"    // Runs only in the browser. Cannot import server-only APIs.
"use module shared"    // Runs in both contexts. Only pure logic — no DOM, no server APIs.
"use module email"     // Compiles to HTML. Available: JSX, no reactivity primitives.
"use module test"      // Available only during forge test.
```

Omitting the directive is equivalent to `"use module shared"` in library code, or context-inferred for files inside known directories (`app/server/` implies server, etc.).

## Component Syntax

The `component` keyword defines a reactive UI component. It is syntactic sugar over a function that returns JSX, with signal dependency tracking wired automatically.

```typescript
export component Greeting({ name }: { name: string }) {
  return <h1>Hello, {name}!</h1>
}
```

Equivalent to:

```typescript
export const Greeting = (({ name }: { name: string }) => {
  return <h1>Hello, {name}!</h1>
}) satisfies ForgeComponent<{ name: string }>
```

Components support all JSX syntax. Forge uses its own JSX transform — no `import React` required.

## Signal Builtins

Signal builtins are available in all `"use module client"` contexts without an import. They are provided by the Forge runtime.

### $signal

```typescript
const $count = $signal(0)
$count.value       // read
$count.value = 5   // write, triggers reactive updates
```

### $derived

```typescript
const $double = $derived(() => $count.value * 2)
$double.value      // read-only
```

### $async

```typescript
const $user = $async(async () => fetchUser(userId))
$user.loading      // boolean
$user.value        // T | undefined
$user.error        // Error | undefined
$user.refresh()    // re-run the async function
```

### $effect

```typescript
$effect(() => {
  console.log('count changed:', $count.value)
  return () => console.log('cleanup')
})
```

## JSX

Forge JSX is a superset of standard JSX:

```typescript
// Standard JSX
<div class="container">
  <h1>{title}</h1>
  {items.map(item => <li key={item.id}>{item.name}</li>)}
</div>
```

Note: Forge uses `class` (not `className`) and `for` (not `htmlFor`) in JSX — matching HTML attribute names.

## Server Function Declaration

Functions in `"use module server"` files are automatically treated as server functions when imported from client modules:

```typescript
"use module server"

export async function createUser(input: { email: string; name: string }): Promise<User> {
  // Server implementation
}
```

No special syntax is needed. The compiler detects cross-boundary imports and generates RPC stubs.

## Type Imports

Types can be imported across module boundaries without restriction:

```typescript
"use module client"

import type { User } from 'app/server/users.fx'  // Types are erased — no boundary violation
```

## Environment Variables

Access environment variables via the `env` global. The compiler validates that `env.*` references match the `[env]` section of `forge.toml`:

```typescript
"use module server"

const secret = env.SESSION_SECRET   // Type-safe, validated at startup
```

`env` is only available in `"use module server"` files. Accessing it in a client module is a compile-time error.

## Imports

Forge resolves imports using its own module resolver:

```typescript
import { Router } from 'forge:router'           // FSL package
import { loadPosts } from 'app/server/posts.fx' // App module (rooted at project root)
import { Button } from './Button.fx'            // Relative import
import type { Post } from 'app/server/posts.fx' // Type-only import (safe across boundaries)
```

`node_modules` are not used. All dependencies are resolved from the Foundry cache.
