# 004 — Reactive Signals

**Status:** Normative
**Version:** 0.1.0-pre-alpha
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [TC39 Signals Foundation](#2-tc39-signals-foundation)
3. [Primitive Operations](#3-primitive-operations)
   - 3.1 [`Signal.State<T>`](#31-signalstatet)
   - 3.2 [`Signal.Computed<T>`](#32-signalcomputedt)
   - 3.3 [`Signal.subtle.Watcher`](#33-signalsubtlewatcher)
4. [Forge Sugar Layer](#4-forge-sugar-layer)
   - 4.1 [`$signal`](#41-signal)
   - 4.2 [`$derived`](#42-derived)
   - 4.3 [`$async`](#43-async)
   - 4.4 [`$effect`](#44-effect)
   - 4.5 [`$` Read Syntax](#45--read-syntax)
5. [`$async` State Machine](#5-async-state-machine)
6. [Reactive Context Rules](#6-reactive-context-rules)
7. [Dependency Tracking](#7-dependency-tracking)
8. [Batching](#8-batching)
9. [Ownership and Disposal](#9-ownership-and-disposal)
10. [Compile-Time DOM Wiring](#10-compile-time-dom-wiring)
11. [Error Propagation](#11-error-propagation)
12. [Formal Semantics Summary](#12-formal-semantics-summary)

---

## 1. Purpose

The Forge reactive model provides a fine-grained, pull-based signal system for tracking and propagating state changes. It is the mechanism by which components update the DOM without re-running the component function. Signal changes produce targeted DOM patches, not full re-renders.

This specification defines:

- The TC39 Signals API that Forge builds on top of.
- The Forge syntax sugar layer (`$signal`, `$derived`, `$async`, `$effect`).
- The formal evaluation and dependency-tracking semantics.
- Ownership, disposal, and memory-safety guarantees.
- How the compiler wires signals to DOM nodes at build time.

---

## 2. TC39 Signals Foundation

Forge's reactive model is built directly on the TC39 Signals proposal. Forge does not implement a custom reactivity system — it uses the proposal's polyfill/implementation as a runtime dependency and adds syntax sugar and async support on top.

The TC39 Signals proposal specifies three primitives:

- `Signal.State<T>` — a mutable source of truth
- `Signal.Computed<T>` — a derived value
- `Signal.subtle.Watcher` — an effect runner

The `Signal` namespace is available in all Forge runtime environments (server runtime, edge runtime, client). It is not exposed as a global in user code but is imported internally by the desugared compiler output.

**Specification reference:** The behaviour of these primitives as defined by the TC39 proposal is normative for Forge. Where this document adds constraints (e.g. batching guarantees, disposal semantics), those constraints are additive and do not contradict the proposal.

---

## 3. Primitive Operations

### 3.1 `Signal.State<T>`

A `Signal.State<T>` holds a mutable value and notifies dependents when it changes.

**Constructor:**

```typescript
new Signal.State<T>(initialValue: T, options?: SignalOptions<T>): Signal.State<T>
```

- `initialValue` is the starting value. It is stored but dependents are not notified at construction time.
- `options.equals` is an optional equality function `(a: T, b: T) => boolean`. If provided, `.set()` only notifies dependents when `equals(oldValue, newValue)` returns `false`. The default is `Object.is`.

**Methods:**

```typescript
.get(): T
```

Reads the current value. If called inside a reactive context (inside a `Signal.Computed` computation function or a `Signal.subtle.Watcher` watch function), this call registers the signal as a dependency of that context.

```typescript
.set(newValue: T): void
.set(updater: (prev: T) => T): void
```

Updates the stored value. If the new value is not equal to the current value (per the equality function), all direct and transitive dependents are marked stale. The updater form receives the current value and returns the new value; this is a convenience for immutable updates.

**Identity guarantee:** The `Signal.State` object itself is stable for the lifetime of the owning scope. It is safe to pass as a prop, close over, or store in a collection.

---

### 3.2 `Signal.Computed<T>`

A `Signal.Computed<T>` is a lazily-evaluated derived value. Its computation function is called only when its value is read and at least one dependency has changed since the last evaluation.

**Constructor:**

```typescript
new Signal.Computed<T>(computation: () => T, options?: SignalOptions<T>): Signal.Computed<T>
```

- `computation` is a synchronous function that reads zero or more signals. All signal reads inside `computation` are automatically tracked as dependencies.
- `options.equals` — same as `Signal.State`.

**Methods:**

```typescript
.get(): T
```

If the computed value is stale (any dependency has changed), re-runs `computation` and stores the result. Returns the (potentially cached) value. Registers this computed signal as a dependency of any enclosing reactive context.

**Lazy evaluation:** A `Signal.Computed` does not run its computation until `.get()` is called. Constructing a `Signal.Computed` is side-effect-free.

**Caching invariant:** Between any two calls to `.get()` where no dependency has changed, the same value is returned without re-running `computation`. This is the core performance guarantee of the reactive model.

**Throwing computation:** If `computation` throws, the exception propagates through `.get()` to the caller. The computed signal is marked as errored; subsequent `.get()` calls re-throw the last exception until a dependency changes and causes re-evaluation.

---

### 3.3 `Signal.subtle.Watcher`

`Signal.subtle.Watcher` is the low-level mechanism for running effects. It is not used directly in `.fx` files — it is the target of `$effect` desugaring.

**Constructor:**

```typescript
new Signal.subtle.Watcher(notify: () => void): Signal.subtle.Watcher
```

`notify` is called synchronously when any watched signal becomes stale. It must not read signal values — its sole purpose is to schedule a re-run of the effect.

**Methods:**

```typescript
.watch(...signals: Signal<unknown>[]): void
```

Registers signals to watch. After calling `.watch()`, the watcher's `notify` callback will be called when any of the watched signals (or their dependencies) change.

```typescript
.unwatch(...signals: Signal<unknown>[]): void
```

Unregisters signals. After `.unwatch()`, the watcher's `notify` callback will no longer be called due to changes in those signals.

```typescript
.getPending(): Signal.Computed<unknown>[]
```

Returns the list of computed signals that are stale and need re-evaluation before the effect re-runs.

---

## 4. Forge Sugar Layer

### 4.1 `$signal`

**Syntax:**

```typescript
const count = $signal(0)
const name = $signal<string>("")
const items = $signal<string[]>([])
```

**Desugaring:**

```typescript
const count = new Signal.State(0)
const name = new Signal.State<string>("")
const items = new Signal.State<string[]>([])
```

The type parameter is inferred from the initial value or may be supplied explicitly. `$signal` is only valid at the top level of a component body or at module scope in a `.fx` file. It is not valid inside loops, conditionals, or nested functions. This restriction mirrors the React hooks rules but is enforced at compile time rather than at runtime.

**Compile-time enforcement of stable identity:** Because the compiler maps `$signal` calls to signal construction in a deterministic order, signals have stable identity within their scope. Moving a `$signal` call inside a conditional is a compile error:

```text
error[E007]: $signal must not be called conditionally
  --> app/pages/Home.fx:14:7
   |
14 |   if (flag) { const x = $signal(0) }
   |               ^^^^^^^^^^^^^^^^^^^^^
```

---

### 4.2 `$derived`

**Syntax:**

```typescript
const doubled = $derived(() => $count * 2)
const label = $derived(() => `Hello, ${$name}!`)
```

**Desugaring:**

The `$` reads inside the arrow function are desugared to `.get()` calls, and the whole expression becomes a `Signal.Computed`:

```typescript
const doubled = new Signal.Computed(() => count.get() * 2)
const label = new Signal.Computed(() => `Hello, ${name.get()}!`)
```

`$derived` accepts a zero-argument arrow function only. Using a named function expression or an async arrow function is a compile error.

---

### 4.3 `$async`

**Syntax:**

```typescript
const user = $async(() => fetchUser($userId))
const results = $async<SearchResult[]>(() => search($query))
```

**Semantics:**

`$async` wraps a Promise-returning function in a `Signal.Computed` with an embedded state machine. The `$async` call itself is synchronous and returns an `AsyncSignal<T>` immediately in the `pending` state.

`$async` signals re-run their fetch whenever a `$`-read dependency inside the arrow function changes. The previous Promise is abandoned (not cancelled) when a new fetch starts; only the most recent Promise's resolution is reflected in the signal state.

**Type of `$async` signal:**

```typescript
type AsyncSignal<T> = Signal.Computed<AsyncSignalValue<T>>

type AsyncSignalValue<T> =
  | { state: 'pending'; value: undefined; error: undefined }
  | { state: 'ready';   value: T;         error: undefined }
  | { state: 'error';   value: undefined; error: Error     }
```

Accessing the `$user` read shorthand returns the current `AsyncSignalValue<T>`.

---

### 4.4 `$effect`

**Syntax:**

```typescript
// Simple effect
$effect(() => {
  document.title = `${$count} items`
})

// Effect with cleanup
$effect(() => {
  const id = setInterval(() => count.set(n => n + 1), 1000)
  return () => clearInterval(id)
})
```

**Desugaring (conceptual):**

```typescript
{
  let cleanupFn: (() => void) | undefined

  const run = () => {
    if (cleanupFn) { cleanupFn(); cleanupFn = undefined }
    const result = (() => {
      // effect body with $-reads replaced by .get()
      document.title = `${count.get()} items`
    })()
    if (typeof result === 'function') cleanupFn = result
  }

  const watcher = new Signal.subtle.Watcher(() => {
    queueMicrotask(run)
  })

  // Initial run — establishes dependency set
  run()

  // Register with owning scope for disposal
  __forge_register_effect(watcher, run, () => {
    if (cleanupFn) cleanupFn()
    watcher.unwatch(...watcher.getPending())
  })
}
```

**Scheduling:** The `notify` callback of the underlying `Watcher` always schedules the re-run via `queueMicrotask`, not synchronously. This ensures effects do not run inside other reactive computations.

---

### 4.5 `$` Read Syntax

Inside a reactive context (component body, `$derived` arrow function, `$effect` arrow function, `$async` arrow function), the `$` prefix on a signal binding reads its current value via `.get()`:

| Expression | Desugars to |
|------------|-------------|
| `$count` | `count.get()` |
| `$user.value` | `user.get().value` |
| `$items.length` | `items.get().length` |
| `$user.state === 'ready'` | `user.get().state === 'ready'` |

The `$` prefix is a compile-time transformation — it has no runtime cost beyond the `.get()` call.

**Precedence:** `$identifier` is parsed as `($identifier)` — the `$` binds to the identifier only. Member access chains after the `$identifier` apply to the read value, not to the signal itself.

**Outside reactive contexts:** Using `$` read syntax outside a reactive context is a compile error (E006). Use `.get()` directly.

---

## 5. `$async` State Machine

```text
            ┌─────────────────────────────────────┐
            │             dependency               │
            │              changes                 │
            ▼                                      │
         [pending] ──── Promise resolves ──── [ready]
            │                                      ▲
            │                                      │
            └──────── Promise rejects ─────── [error]
                                                   │
                                          dependency changes
                                                   │
                                                   ▼
                                               [pending]
```

**State transitions:**

| From | Event | To |
|------|-------|----|
| (initial) | `$async` constructed | `pending` |
| `pending` | Promise resolves | `ready` |
| `pending` | Promise rejects | `error` |
| `ready` | dependency changes, new Promise starts | `pending` |
| `error` | dependency changes, new Promise starts | `pending` |

**Promise abandonment:** When a dependency changes while a Promise is in-flight, a new Promise is started. The old Promise is not cancelled (there is no standard cancellation mechanism for arbitrary Promises). Its resolution is silently ignored — it will never transition the signal to `ready` or `error` because it is no longer the current Promise.

**Re-fetch on error:** An errored `$async` signal does not automatically retry. It transitions back to `pending` only when a tracked dependency changes.

---

## 6. Reactive Context Rules

A **reactive context** is any of the following:

1. A component function body (between `export component Name(...) {` and the closing `}`).
2. The arrow function argument of `$derived`.
3. The arrow function argument of `$async`.
4. The arrow function argument of `$effect`.
5. Any function called from within one of the above that is itself pure and synchronous (i.e. reads are tracked transitively).

**What is valid inside a reactive context:**

- `$`-read syntax.
- `$derived`, `$async`, `$effect` calls.
- Signal `.set()` calls (though `.set()` inside a `$derived` is a runtime error — computed signals must be pure).
- Arbitrary TypeScript/JavaScript.

**What is invalid inside a reactive context:**

- `$signal` inside a conditional or loop (compile error E007).
- `await` inside `$derived` or `$effect` (compile error E008 — use `$async` for async derived values).
- `.set()` inside `$derived` (runtime error from TC39 Signals: writing inside a computation causes a cycle).

---

## 7. Dependency Tracking

Dependency tracking is **automatic** and **dynamic**. The set of dependencies for a `Signal.Computed` or `Signal.subtle.Watcher` is determined at runtime by which signals' `.get()` methods are called during the computation. Dependencies are re-tracked on each evaluation.

**Conditional dependencies:**

```typescript
const result = $derived(() => {
  if ($flag) {
    return $a + $b  // depends on flag, a, b
  } else {
    return $c       // depends on flag, c
  }
})
```

When `flag` is `true`, the computed depends on `flag`, `a`, and `b`. When `flag` is `false`, the computed depends on `flag` and `c`. If `flag` changes from `true` to `false`, `a` and `b` are automatically removed from the dependency set on the next evaluation.

**Glitch freedom:** The TC39 Signals model is glitch-free by construction. A computed signal is never observed in an intermediate state where some of its dependencies have updated but others have not. The pull-based evaluation order ensures consistency.

---

## 8. Batching

Multiple `.set()` calls within the same synchronous task are **batched**. Dependents (computed signals and effects) are notified once after all mutations in the batch, not once per mutation.

**Mechanism:** Signal writes mark dependents as stale synchronously but do not trigger re-evaluation or notification until the current synchronous task completes (i.e., on the next microtask checkpoint, consistent with the TC39 Signals proposal).

**Example:**

```typescript
// These two .set() calls are batched:
firstName.set('Jane')
lastName.set('Smith')
// fullName (which reads both) is re-evaluated once, not twice
```

**Explicit batching:** No explicit batch API is required. The default behaviour is already batched within a synchronous task. Batching across async boundaries (e.g. two `.set()` calls in separate `await` continuations) is not guaranteed — each continuation is a separate task.

---

## 9. Ownership and Disposal

**Component-scoped signals:** Signals and effects created inside a component body are **owned by the component**. When the component is unmounted (its DOM node is removed from the document), the Forge runtime:

1. Calls the cleanup function of every `$effect` in the component (if any was returned).
2. Calls `.unwatch()` on every `Signal.subtle.Watcher` in the component.
3. Releases all references to the component's signals.

This guarantees no memory leaks from stale subscriptions after component teardown.

**Module-scoped signals:** Signals and effects declared at module scope (outside any component function) live for the lifetime of the application. They are never disposed. This is appropriate for global application state.

**`$async` disposal:** When a `$async` signal is disposed (component unmount), the in-flight Promise (if any) is abandoned. Its resolution will be ignored, and the Promise itself will be garbage collected if no other references exist.

**Manual disposal:** There is no public `dispose()` API for user-created signals. Disposal is managed entirely by the Forge runtime based on component lifecycle. This is intentional — manual disposal is a common source of bugs (double-dispose, use-after-dispose).

---

## 10. Compile-Time DOM Wiring

The `signal_transform` compiler pass analyzes JSX return expressions in component functions and generates `Signal.subtle.Watcher` subscriptions that directly update affected DOM nodes. This is the mechanism by which signal changes produce targeted DOM patches.

**Algorithm:**

1. Walk the JSX tree in the component's return expression.
2. For each JSX expression slot (attribute value or child expression) that contains one or more `$`-reads:
   a. Generate a unique variable for the DOM node that will be updated.
   b. Generate a `Signal.subtle.Watcher` that, when notified, updates only that DOM node.
   c. Record which signals are read in the expression — these are the watcher's initial watch list.
3. The component function, when called, performs the initial DOM construction and sets up all watchers.

**Example:**

```typescript
// Source component:
export component Counter() {
  const count = $signal(0)
  return (
    <div>
      <span>{$count}</span>
      <button onClick={() => count.set(n => n + 1)}>+</button>
    </div>
  )
}

// Compiled output (conceptual):
export function Counter(): HTMLElement {
  const count = new Signal.State(0)

  // DOM construction (runs once)
  const __div = document.createElement('div')
  const __span = document.createElement('span')
  const __button = document.createElement('button')
  __button.textContent = '+'

  // Initial render
  __span.textContent = String(count.get())

  // Signal wiring (runs once, patches run on change)
  const __w0 = new Signal.subtle.Watcher(() => {
    queueMicrotask(() => {
      __span.textContent = String(count.get())
    })
  })
  __w0.watch(count)

  // Event wiring
  __button.addEventListener('click', () => count.set(n => n + 1))

  __div.appendChild(__span)
  __div.appendChild(__button)

  // Register watchers for disposal on unmount
  __forge_register_watchers(__div, [__w0])

  return __div
}
```

**Granularity guarantee:** Only the DOM nodes whose expressions contain `$`-reads are wired to watchers. Static subtrees (no signal reads) are constructed once and never updated. This minimises DOM mutations on state changes.

---

## 11. Error Propagation

**In `$derived`:** If the computation function throws, the error is propagated to any caller of `.get()` on the computed signal (including the `dom_wire`-generated watcher). Unhandled errors from watcher re-runs are reported via `reportError()` (the `ErrorEvent` mechanism) to avoid swallowing reactive errors silently.

**In `$async`:** Promise rejections transition the signal to the `error` state. They are not propagated as unhandled rejections — the `error` state is the intended mechanism for handling async failures.

**In `$effect`:** If the effect body throws synchronously, the error propagates through the watcher's scheduled microtask. Unhandled, it becomes an unhandled rejection on the microtask queue. The cleanup function (if any) is not called when the effect throws — this matches the semantics of an exception interrupting cleanup in other contexts. The effect will not re-run until a dependency changes.

---

## 12. Formal Semantics Summary

Let `S` be the set of all `Signal.State` values and `C` the set of all `Signal.Computed` values in the application. The reactive system maintains the following invariants:

**I1 — Consistency:** At any point where no reactive update is in progress, for every `Signal.Computed` `c` whose dependencies have not changed since its last evaluation, `c.get()` returns the same value without re-running the computation.

**I2 — Minimality:** A `Signal.Computed` re-evaluates its computation only when at least one of its current dependencies has changed value (per its equality function).

**I3 — Glitch-freedom:** Reading any computed signal during an effect re-run observes a consistent snapshot: all mutations from the triggering batch have been applied; no intermediate states are visible.

**I4 — No cycles:** A `Signal.Computed` must not directly or transitively write to any signal it reads. The TC39 Signals runtime detects cycles and throws `RangeError: Cycle detected`.

**I5 — Effect ordering:** Effects are scheduled via `queueMicrotask` and run in the order they were first created within a component. Effects in parent components run before effects in child components (based on mounting order).

**I6 — Disposal completeness:** After a component is unmounted, no watcher or effect associated with that component will ever call its callback or notify function again.
