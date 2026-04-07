//! Tree-shaking: reduce the resolved source set to only reachable modules.
//!
//! Takes the import graph and reachability result and produces a minimal set
//! of source files with their content. Modules absent from the reachability
//! result are excluded entirely. In the future, per-export pruning will
//! further reduce individual files to only their used exports.

use std::collections::HashSet;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use tracing::{debug, warn};

use super::call_graph::ReachabilityResult;
use super::resolver::ImportGraph;

/// A source file retained after tree-shaking.
#[derive(Debug, Clone)]
pub struct ShakenSource {
    /// Absolute path to the original source file.
    pub path: Utf8PathBuf,
    /// The file's source content (currently the full file; future versions
    /// will prune to only used exports).
    pub content: String,
}

/// The complete set of sources retained after tree-shaking.
#[derive(Debug, Clone, Default)]
pub struct ShakenApp {
    /// All retained source files.
    pub sources: Vec<ShakenSource>,
    /// Files that were in the import graph but pruned as unreachable.
    pub pruned_files: HashSet<Utf8PathBuf>,
}

/// Shake the import graph down to only the modules that are reachable
/// from entry points.
///
/// Reads each reachable module's content from disk and collects it into a
/// `ShakenApp`. Unreachable modules are recorded in `pruned_files` for
/// reporting purposes.
pub fn shake(graph: &ImportGraph, reachability: &ReachabilityResult) -> Result<ShakenApp> {
    let all_files = graph.all_files();
    let mut shaken = ShakenApp::default();

    for file_path in &all_files {
        if reachability.module_usage.contains_key(*file_path) {
            let content = std::fs::read_to_string(file_path.as_std_path())
                .with_context(|| format!("failed to read reachable file: {file_path}"))?;

            shaken.sources.push(ShakenSource {
                path: (*file_path).clone(),
                content,
            });
        } else {
            shaken.pruned_files.insert((*file_path).clone());
        }
    }

    debug!(
        retained = shaken.sources.len(),
        pruned = shaken.pruned_files.len(),
        "tree-shaking complete"
    );

    if shaken.sources.is_empty() {
        warn!("tree-shaking produced zero source files — the import graph may be empty");
    }

    Ok(shaken)
}
