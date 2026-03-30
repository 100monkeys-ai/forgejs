# Signals & Reactivity

Forge's reactivity system is built on the TC39 Signals proposal. There is no virtual DOM. When a signal changes, only the DOM nodes that depend on that signal are updated — no diffing, no reconciler.

## $signal — Reactive State

`$signal` creates a reactive value. Reading `.value` inside a component establishes a dependency; writing to `.value` triggers updates to all dependents.

```typescript
"use module client"

const $count = $signal(0)

export default component Counter() {
  return (
    <div>
      <p>Count: {$count.value}</p>
      <button onClick={() => $count.value++}>+</button>
      <button onClick={() => $count.value--}>-</button>
    </div>
  )
}
```

Signals declared at module scope are singletons. Signals declared inside a component are local to that component instance.

## $derived — Computed Values

`$derived` creates a signal whose value is computed from other signals. It is lazy — the computation only runs when `.value` is read and one of its dependencies has changed.

```typescript
const $firstName = $signal('Ada')
const $lastName = $signal('Lovelace')
const $fullName = $derived(() => `${$firstName.value} ${$lastName.value}`)

console.log($fullName.value) // "Ada Lovelace"
$firstName.value = 'Grace'
console.log($fullName.value) // "Grace Lovelace"
```

## $async — Async Derived Signals

`$async` wraps an async function and exposes `.loading`, `.value`, and `.error`:

```typescript
const $user = $async(() => fetchCurrentUser())

// In a component:
if ($user.loading) return <Spinner />
if ($user.error) return <Error message={$user.error.message} />
return <UserCard user={$user.value} />
```

`$async` automatically re-runs when the function's signal dependencies change. It does not run on the server during SSR by default — use a server function data loader for that.

## $effect — Side Effects

`$effect` runs a callback whenever its signal dependencies change. It runs once immediately after the component mounts.

```typescript
const $query = $signal('')

$effect(() => {
  document.title = $query.value ? `Search: ${$query.value}` : 'My App'
})
```

Effects are cleaned up automatically when the component unmounts. Return a cleanup function for explicit teardown:

```typescript
$effect(() => {
  const id = setInterval(() => $tick.value++, 1000)
  return () => clearInterval(id)
})
```

## Signal Scope

Signals declared outside components are application-scoped singletons. This is appropriate for global state (current user, theme, feature flags). For per-component state, declare signals inside the component function body.

```typescript
// Global — shared across all components
const $theme = $signal<'light' | 'dark'>('light')

// Local — each Counter instance has its own $count
export default component Counter() {
  const $count = $signal(0)
  // ...
}
```

## Reactivity and Server Functions

Signals are client-only. Server functions are not reactive. The bridge between them is `$async`, which wraps server function calls in a reactive signal on the client. Changes to server state (via mutations) propagate back to the client by calling `.refresh()` on the relevant `$async` signal.
