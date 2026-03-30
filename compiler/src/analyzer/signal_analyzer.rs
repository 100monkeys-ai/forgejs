//! Reactive signal dependency graph analysis.
//!
//! This pass validates the signal usage within components and modules,
//! building a dependency graph used by the transformer to emit efficient
//! DOM wiring.
//!
//! See spec/specs/004-reactive-signals.md for the full signal semantics.

/// The signal dependency graph for a single component.
///
/// The transformer uses this graph to determine which DOM nodes need
/// reactive wiring. Only nodes that directly read a signal value are
/// wired — the rest are static and need no update mechanism.
#[derive(Debug, Default)]
pub struct SignalGraph {
    /// Signals declared in this component (via `$signal`)
    pub signals: Vec<SignalNode>,
    /// Derived values (via `$derived` or `$async`)
    pub derived: Vec<DerivedNode>,
    /// Effects (via `$effect`)
    pub effects: Vec<EffectNode>,
}

/// A mutable signal source (`$signal` sugar over `new Signal.State()`).
#[derive(Debug, Clone)]
pub struct SignalNode {
    pub name: String,
    pub initial_value_expr: String,
}

/// A derived value (`$derived` sugar over `new Signal.Computed()`).
#[derive(Debug, Clone)]
pub struct DerivedNode {
    pub name: String,
    /// Names of signals/derived values this node reads
    pub dependencies: Vec<String>,
    pub is_async: bool,
}

/// A side-effectful computation (`$effect` sugar over `Signal.subtle.Watcher`).
#[derive(Debug, Clone)]
pub struct EffectNode {
    /// Names of signals/derived values this effect reads
    pub dependencies: Vec<String>,
}
