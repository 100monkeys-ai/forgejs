//! Signal syntax desugaring and compile-time DOM wiring.
//!
//! This is the pass that eliminates the need for a virtual DOM.
//!
//! ## How It Works
//!
//! After the signal analyzer builds the dependency graph for each component,
//! this pass transforms the component's JSX return into direct imperative
//! DOM operations with fine-grained reactive wiring:
//!
//! **Input** (`.fx` component):
//! ```typescript
//! export component Counter() {
//!   const count = $signal(0)
//!   return <div>{$count}</div>
//! }
//! ```
//!
//! **Output** (compiled JavaScript):
//! ```javascript
//! function Counter() {
//!   const count = new Signal.State(0);
//!   // --- compiled from JSX ---
//!   const _root = document.createElement("div");
//!   const _text0 = document.createTextNode("");
//!   _root.appendChild(_text0);
//!   // Fine-grained wiring: only this text node updates when `count` changes
//!   new Signal.subtle.Watcher(() => {
//!     _text0.data = String(count.get());
//!   }).watch(count);
//!   return _root;
//! }
//! ```
//!
//! The component function runs **once** at mount time. No diffing. No
//! re-renders. Only the specific DOM nodes tied to changed signals update.
//!
//! ## Why This Matters
//!
//! React's virtual DOM re-renders the entire component subtree on every state
//! change, then diffs the result to find what actually changed. This is
//! correct but wasteful — for most updates, 95% of the diff result is
//! "nothing changed here."
//!
//! SolidJS proved in 2021 that compile-time analysis of JSX expressions can
//! identify exactly which DOM nodes depend on which signals, eliminating the
//! need for diffing entirely. Forge adopts this approach and targets the
//! TC39 Signals standard (rather than a framework-specific implementation)
//! so the compiled output benefits from native browser support as it ships.
//!
//! See ADR-006 for the full rationale.
