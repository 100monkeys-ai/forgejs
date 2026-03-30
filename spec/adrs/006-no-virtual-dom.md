# ADR-006: No Virtual DOM — TC39 Signals and Compile-Time DOM Wiring

**Number**: 006
**Title**: No Virtual DOM — TC39 Signals and Compile-Time DOM Wiring
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#reactivity` `#frontend` `#signals` `#jsx` `#performance` `#architecture`

---

## Context

### How the Virtual DOM Became Dominant

The virtual DOM (VDOM) was introduced by React in 2013. To understand why Forge rejects it, it is necessary to understand why it was invented and what problem it was actually solving.

Facebook's news feed in 2013 was a genuinely hard UI problem. The feed updated frequently, had complex nested structure, and the previous generation of jQuery-based code that directly mutated the DOM had become unmaintainable. Engineers were spending significant time debugging scenarios where a state update in one part of the page produced unexpected visual effects in another part because some other handler had cached a stale DOM reference.

React's solution: introduce a declarative model. Instead of imperative DOM mutations (`element.textContent = newValue`), developers describe what the UI *should* look like given the current state (`<FeedItem content={item.content} />`). React maintains a JavaScript object tree — the virtual DOM — that mirrors the desired structure. When state changes, React re-renders the affected components (producing a new VDOM subtree), diffs the new subtree against the previous one, and applies the minimal set of actual DOM mutations.

This was a genuine innovation. The declarative model eliminated the stale-reference bug class. The VDOM diff algorithm meant developers did not need to manually determine "which DOM nodes need to change" — React figured it out.

### The Hidden Cost of VDOM

The VDOM's costs were acceptable in 2013. They became significant as React was applied to increasingly complex UIs.

**Cost 1: Every state change re-renders the subtree.** When a React component's state or props change, React re-renders that component and, by default, all of its descendants. For a component tree with hundreds of nodes, a state change at the root causes React to execute hundreds of component functions, constructing hundreds of VDOM objects, before diffing them against the previous tree.

React's mitigations (`React.memo`, `useMemo`, `useCallback`, `shouldComponentUpdate`) are developer-managed. The developer must manually annotate which components are "pure" and should not re-render if their inputs have not changed. The framework does not know this automatically. Getting memoization wrong — memoizing too aggressively (stale closures) or not enough (unnecessary renders) — is one of the most common sources of React bugs and performance problems.

**Cost 2: The VDOM diff itself has cost.** Diffing two VDOM trees is O(n) in the number of nodes where n is the size of the subtree being compared. For most updates, 95% of the diff result is "this node has not changed." That 95% is wasted work — allocating VDOM objects, traversing them, comparing them, and concluding that the corresponding real DOM node does not need to change.

**Cost 3: Reconciliation overhead.** After diffing, React must reconcile the "patch" against the real DOM. This is inherently sequential — React cannot apply DOM mutations in parallel — and each DOM mutation may trigger browser layout recalculation. The reconciler (in React 18, the Fiber reconciler) is extraordinarily sophisticated precisely because the naive approach produces visible jank.

**Cost 4: Hydration.** In server-rendered React applications, the server produces HTML, and then React must "hydrate" that HTML — traverse the server-rendered DOM, reconstruct the component tree in JavaScript, attach event handlers, and verify that the server-rendered output matches what the client would render. Hydration is a startup cost paid before the page becomes interactive. For complex pages, hydration can take several seconds on low-powered devices.

**Cost 5: Memory.** The VDOM is a JavaScript object representation of the entire component tree. It exists in addition to the real DOM. For complex pages, the VDOM can be a significant memory overhead.

### The SolidJS Insight

SolidJS, first released in 2020, demonstrated an alternative that eliminates all five of these costs while preserving the declarative programming model.

The key insight: if the compiler can analyze JSX at build time and determine exactly which DOM nodes depend on which reactive values, it can generate direct DOM wiring code rather than a VDOM diffing runtime.

In SolidJS, a component like:

```jsx
function Counter() {
  const [count, setCount] = createSignal(0);
  return <div>{count()}</div>;
}
```

Does not produce VDOM diffing code. The compiler analyzes the JSX, sees that the `div`'s text content depends on the `count` signal, and generates code equivalent to:

```javascript
const div = document.createElement('div');
createEffect(() => { div.textContent = count(); });
return div;
```

When `count` changes, `createEffect` runs — and only `createEffect` runs. No component re-render. No VDOM allocation. No diffing. The single DOM text node updates directly.

This is not a performance optimization of the VDOM model. It is a fundamentally different model. Component functions run once, at initialization. Reactive updates flow through a dependency graph, not through re-renders. The UI is wired at startup, not reconstructed on every update.

The implications:

- No re-rendering means no memoization burden on the developer
- No VDOM means no diffing cost proportional to the tree size
- No VDOM means no memory overhead for the mirror tree
- No hydration cost — the server renders HTML, the client "resumes" signals (not re-evaluates the entire tree)
- Component functions are simpler because they run once — no rules about "don't call hooks conditionally" because there are no hooks

### TC39 Signals

In 2024, TC39 (the standards body for the JavaScript language) began advancing the Signals proposal. Signals are the reactive primitive that SolidJS (and Preact Signals, Angular Signals, and others) have independently converged on. The TC39 proposal aims to standardize the Signals model as a built-in JavaScript language feature.

The Signals polyfill is production-ready today. When Signals land in V8 as a native built-in, Forge's compiled output benefits automatically — the signal operations that the polyfill implements in user-space JavaScript become native engine operations with lower overhead.

This matters for Forge's architecture because: building on the TC39 Signals model means building on the direction the language is heading. Forge's reactivity model is not Forge-specific — it is the emerging standard. Frameworks that bet on VDOM (React, Vue 3's composition API hybrid) face an eventual migration toward the standard. Forge starts there.

## Decision

No virtual DOM. TC39 Signals as the only reactive primitive. Compile-time DOM wiring via JSX analysis.

Specifically:

1. **No VDOM library is included in the Forge client runtime**. There is no `diff` function, no `reconcile` function, no component re-render mechanism.

2. **The reactive primitive is TC39 Signals** (Signal, Computed, Effect). Forge ships the TC39 polyfill and the compiler generates code targeting the TC39 Signals API.

3. **The Forge compiler analyzes JSX expressions** and generates direct DOM wiring code. Expressions that depend on signals are wrapped in Effects; expressions that do not are evaluated once.

4. **Component functions run once**. There is no "render phase." The component function is an initializer that sets up the DOM structure and signal subscriptions, then returns. Updates flow through the signal graph without re-invoking the component function.

5. **SSR uses a stream-based renderer** that renders signals to their initial values, produces an HTML stream, and embeds the initial signal state in a `<script>` tag for client-side resumption.

## Consequences

### Positive

- ✅ **Zero diffing overhead**: there is no VDOM to diff. DOM updates are exactly as expensive as the DOM mutation itself — no allocation, no traversal, no comparison.
- ✅ **Component functions run once**: the developer does not need to reason about when their component function will run. It runs once, at initialization. Closures in component functions capture values at initialization time and update via signal subscriptions. No stale closure bugs from incorrect `useEffect` dependencies.
- ✅ **No memoization burden**: `React.memo`, `useMemo`, `useCallback` exist because React re-renders on every state change and developers must manually opt out of re-rendering. In Forge, there is no re-rendering — memoization is not a concern.
- ✅ **Aligned with TC39 standard**: Forge's reactivity model is the direction the JavaScript language is heading. No migration required when Signals become native.
- ✅ **Smaller client bundle**: the VDOM runtime (React DOM) is ~42KB gzipped. Forge's client runtime, without a VDOM, is significantly smaller.
- ✅ **Better SSR performance**: no hydration phase. The client "resumes" signal state from the server-rendered HTML rather than re-evaluating the entire component tree.

### Negative

- ❌ **Mental model shift for React developers**: the most common background for JavaScript frontend developers is React. The "think in renders" model is deeply ingrained. Forge's "component functions run once, updates flow through signals" model requires unlearning some React-specific intuitions.
- ❌ **React component ecosystem incompatibility**: the enormous ecosystem of React components (headless UI libraries, charting libraries, form libraries) cannot be used directly in Forge. This is a significant short-term ecosystem disadvantage. Forge's answer is the FSL, which provides first-party implementations of common patterns, but the long tail of community components takes time to develop.
- ❌ **Compiler complexity**: the compile-time JSX analysis that distinguishes "this expression depends on a signal" from "this expression is static" requires the compiler to track signal dependencies through the scope chain. This is non-trivial analysis, particularly for computed values and conditionally executed code.
- ❌ **DevTools ecosystem is less mature**: React DevTools provides component tree inspection, state viewing, and profiling that is well-understood by millions of developers. Forge's DevTools will need to be built from scratch around the signal dependency graph model.

### Neutral

- ℹ️ The signal model is not new — it is used in production by SolidJS applications with demanding performance requirements. The model is proven; what Forge adds is integration with the rest of the framework (compiler enforcement, SSR resumption, Foundry-distributed components).
- ℹ️ Forge components are valid TypeScript/JavaScript — there is no Forge-specific language. The JSX is standard JSX. The signal primitives are TC39 standard. The mental model is different from React, but the syntax will feel familiar.

## Alternatives Considered

### React (VDOM + Hooks + Concurrent Mode)

React is the most widely adopted JavaScript UI library by a significant margin. Its ecosystem (components, DevTools, documentation, developer familiarity) is unmatched.

React's core model is VDOM + re-renders. React 18's concurrent features (Suspense, transitions) improve the prioritization of re-renders but do not eliminate the fundamental overhead — they make expensive re-renders less visible to users, not less expensive.

Forge could build on React and differentiate on the compiler, routing, and server-side story while staying compatible with React's ecosystem. This is what Next.js and Remix do.

Rejected because: the performance ceiling of the VDOM model is lower than the signals model by a fundamental amount. Forge is not building a better Next.js — it is building a framework with correct architecture from the start. Adopting React's VDOM for ecosystem compatibility would compromise the architectural integrity of the client-side model. The ecosystem cost is real, but it is accepted.

### Vue 3

Vue 3's Composition API introduced a signals-like model (refs and computed values). Vue 3's compiler performs some compile-time optimization. Vue 3 is closer in architecture to Forge's model than React is.

Vue 3 is not a clean fit for Forge because:

1. Vue 3's reactive model is Vue-specific, not TC39 standard
2. Vue 3 still uses a VDOM (the Composition API is the reactive primitive layer, but Vue's rendering still goes through VDOM diffing)
3. Building Forge on Vue's runtime would bind Forge to Vue's release cadence and API surface

Rejected: Vue's reactive model is closer to right than React's, but it still uses VDOM and uses a Vue-specific (rather than TC39-standard) reactive primitive.

### Svelte

Svelte uses compile-time reactivity — Svelte's compiler transforms `.svelte` files into imperative DOM manipulation code, similar in concept to Forge's JSX transform.

The key difference: Svelte's reactive model is Svelte-specific. Svelte 5 introduced "runes" as its reactive primitive, which have Svelte-specific semantics rather than TC39 Signals semantics. When TC39 Signals land in V8, Svelte applications compiled with runes do not automatically benefit — they would need migration to use native Signals.

Additionally, Svelte uses `.svelte` files with a custom template syntax rather than JSX. Forge uses JSX — standard TypeScript JSX — because it maximizes tooling compatibility (TypeScript's JSX type checking, editor LSP support, standard formatting tools all understand JSX).

Rejected: the Svelte-specific reactive model is not the right long-term bet. TC39 Signals is the standard; build on the standard.

### Preact

Preact is React-compatible with a smaller runtime. It uses the same VDOM model, the same hooks API, and the same mental model. The performance improvement over React is modest.

Preact also offers "Preact Signals" — a signals library that integrates with the VDOM model. This is the worst of both worlds: you get the signals primitive but still pay the VDOM allocation and diffing costs, because signals are used to trigger re-renders rather than to replace them.

Rejected: VDOM model, smaller React ecosystem, no architectural improvement.

## Implementation Notes

The JSX transform is implemented in `crates/forge-compiler/src/transform/jsx.rs`.

The transform analyzes JSX expressions in three categories:

1. **Static**: expression has no signal dependencies. Generated as a one-time DOM creation.
2. **Dynamic text**: expression is a signal read inside a JSX text position. Generated as `createEffect(() => { node.textContent = expr; })`.
3. **Dynamic attribute**: expression is a signal read in an attribute position. Generated as `createEffect(() => { element.setAttribute(name, expr); })` or the appropriate DOM property assignment.
4. **Dynamic children**: conditional rendering (`{condition() && <Child />}`) and list rendering (`{list().map(...)}`). Generated as a reactive block that replaces its DOM placeholder when the controlling signal changes.

The TC39 Signals polyfill is the `@tc39/signal-polyfill` package, pinned in the Forge package manifest and included in the client runtime bundle.

## Related Decisions

- [ADR-001: Rust-Powered Compiler Pipeline](./001-rust-powered-compiler.md) — the compiler that performs the JSX analysis
- [ADR-007: Compile-Time Boundary Enforcement](./007-compile-time-boundary-enforcement.md) — compile-time enforcement of signal usage patterns
