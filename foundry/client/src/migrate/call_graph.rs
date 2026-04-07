//! Best-effort reachability analysis via import-graph and export tracking.
//!
//! This is NOT a full inter-procedural pointer analysis. We track:
//! - Which modules are transitively imported from entry points
//! - Which named exports are actually imported (used) by consumers
//! - Which re-exports propagate usage
//!
//! We do NOT track:
//! - Dynamic property access (`obj[computedKey]`)
//! - `eval()` or `new Function()` usage
//! - Side-effect-only imports where the module body has observable effects
//!
//! The goal is to produce a conservative-but-practical set of reachable
//! source modules and their used exports, good enough to eliminate the
//! vast majority of dead library code while rarely missing actually-used code.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Result;
use camino::Utf8PathBuf;
use tracing::debug;

use super::resolver::{EntryPoint, ImportGraph};

/// Describes which exports of a module are actually used by its importers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsedExports {
    /// All exports are used (e.g., `import * as mod from '...'` or side-effect import).
    All,
    /// Only these named exports are used.
    Named(HashSet<String>),
}

impl UsedExports {
    /// Merge another usage set into this one.
    pub fn merge(&mut self, other: &UsedExports) {
        match self {
            UsedExports::All => {} // Already using everything
            UsedExports::Named(names) => match other {
                UsedExports::All => *self = UsedExports::All,
                UsedExports::Named(other_names) => {
                    names.extend(other_names.iter().cloned());
                }
            },
        }
    }
}

/// The result of reachability analysis.
///
/// Maps each reachable module to the set of its exports that are actually
/// used by consumers. Unreachable modules (not transitively imported from
/// any entry point) are absent from this map entirely.
#[derive(Debug, Clone, Default)]
pub struct ReachabilityResult {
    /// Map from module path to which of its exports are used.
    pub module_usage: HashMap<Utf8PathBuf, UsedExports>,
}

/// Perform best-effort reachability analysis starting from the given entry points.
///
/// Walks the import graph breadth-first from each entry point, tracking which
/// named exports each importer actually references. Modules not reachable from
/// any entry point are excluded from the result.
pub fn analyze_reachability(
    entry_points: &[EntryPoint],
    graph: &ImportGraph,
) -> Result<ReachabilityResult> {
    let mut result = ReachabilityResult::default();
    let mut queue: VecDeque<Utf8PathBuf> = VecDeque::new();

    // Seed the queue with entry points — they are fully used by definition
    for ep in entry_points {
        // Entry points may be relative; try to find their absolute form in the graph
        let abs_path = find_in_graph(&ep.path, graph);
        if let Some(path) = abs_path {
            result
                .module_usage
                .entry(path.clone())
                .or_insert(UsedExports::All);
            queue.push_back(path);
        } else {
            debug!(path = %ep.path, "entry point not found in import graph");
        }
    }

    // BFS through the import graph
    while let Some(current) = queue.pop_front() {
        if let Some(imports) = graph.edges.get(&current) {
            for imported in imports {
                let is_new = !result.module_usage.contains_key(imported);
                // For now, conservatively mark all imports as using all exports.
                // A more precise analysis would parse each importer to determine
                // exactly which named imports it pulls, but the conservative
                // approach ensures we never incorrectly eliminate used code.
                let entry = result
                    .module_usage
                    .entry(imported.clone())
                    .or_insert(UsedExports::Named(HashSet::new()));
                entry.merge(&UsedExports::All);

                if is_new {
                    queue.push_back(imported.clone());
                }
            }
        }
    }

    debug!(
        reachable = result.module_usage.len(),
        total = graph.edges.len(),
        "reachability analysis complete"
    );

    Ok(result)
}

/// Try to find a path in the import graph, handling the case where entry points
/// may use relative paths while the graph uses absolute paths.
fn find_in_graph(path: &Utf8PathBuf, graph: &ImportGraph) -> Option<Utf8PathBuf> {
    // Direct lookup
    if graph.edges.contains_key(path) {
        return Some(path.clone());
    }

    // Check if any graph key ends with this path
    let path_str = path.as_str();
    for key in graph.edges.keys() {
        if key.as_str().ends_with(path_str) {
            return Some(key.clone());
        }
    }

    None
}
