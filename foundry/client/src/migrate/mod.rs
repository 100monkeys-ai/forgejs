//! Migration engine: converts Node.js applications into Forge projects.
//!
//! The migration pipeline runs these stages in order:
//!
//! 1. **Resolve** — discover entry points from `package.json` and build
//!    the full import graph by following `import`/`require` statements.
//! 2. **Call Graph** — determine which modules and exports are transitively
//!    reachable from the discovered entry points.
//! 3. **Tree Shake** — strip unreachable modules and unused exports to
//!    produce a minimal source set.
//! 4. **Analyze** — classify each remaining source file by compatibility
//!    tier (compatible, shimmable, or needs manual attention).
//! 5. **Pattern Match** — detect framework idioms (React hooks, Express
//!    routes, Next.js conventions) and produce transformation instructions.
//! 6. **Convert** — apply source-level transforms: rename extensions to
//!    `.fx`, rewrite imports, convert CJS to ESM, apply framework patterns.
//! 7. **Scaffold** — generate the Forge project directory structure with
//!    `forge.toml`, appropriate subdirectories, and converted sources.
//! 8. **Report** — produce a human-readable or JSON migration report
//!    summarizing what was converted, what needs attention, and next steps.

pub mod analyzer;
pub mod call_graph;
pub mod converter;
pub mod framework_patterns;
pub mod report;
pub mod resolver;
pub mod scaffold;
pub mod tree_shaker;

use camino::Utf8PathBuf;
use tracing::info;

use crate::migrate::analyzer::analyze_compatibility;
use crate::migrate::call_graph::analyze_reachability;
use crate::migrate::converter::convert_sources;
use crate::migrate::framework_patterns::{detect_framework, match_patterns};
use crate::migrate::report::{generate_report, MigrationReport};
use crate::migrate::resolver::resolve_import_graph;
use crate::migrate::scaffold::scaffold_project;
use crate::migrate::tree_shaker::shake;
use forge_shared::manifest::MigrationMetadata;

/// Configuration for the migration pipeline.
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    /// Path to the root of the Node.js project to migrate.
    pub source_path: Utf8PathBuf,
    /// Path where the Forge project should be written.
    pub output_path: Utf8PathBuf,
    /// When true, run the full analysis pipeline but do not write any files.
    pub dry_run: bool,
    /// Optional hint to override automatic framework detection.
    pub framework_hint: Option<String>,
    /// Whether to include dev-dependencies in the import graph.
    pub include_dev_deps: bool,
}

/// The result of a completed migration.
#[derive(Debug)]
pub struct MigrateResult {
    /// The full migration report with per-file details and summary.
    pub report: MigrationReport,
    /// The output directory where the Forge project was written
    /// (or would have been written in dry-run mode).
    pub output_path: Utf8PathBuf,
    /// Migration metadata suitable for embedding in `forge.toml`.
    pub metadata: MigrationMetadata,
}

/// Run the full migration pipeline: resolve → call_graph → tree_shake →
/// analyze → pattern_match → convert → scaffold → report.
pub async fn migrate_app(options: MigrateOptions) -> anyhow::Result<MigrateResult> {
    info!(source = %options.source_path, output = %options.output_path, "starting migration");

    // 1. Resolve entry points and import graph
    let (entry_points, import_graph) = resolve_import_graph(&options.source_path).await?;
    info!(
        entry_points = entry_points.len(),
        modules = import_graph.edges.len(),
        "import graph resolved"
    );

    // 2. Reachability analysis
    let reachability = analyze_reachability(&entry_points, &import_graph)?;
    info!(
        reachable_modules = reachability.module_usage.len(),
        "reachability analysis complete"
    );

    // 3. Tree-shake to minimal source set
    let shaken = shake(&import_graph, &reachability)?;
    info!(files = shaken.sources.len(), "tree-shaking complete");

    // 4. Compatibility analysis
    let analysis = analyze_compatibility(&shaken)?;
    info!(
        compatible = analysis.summary.compatible_count,
        shimmable = analysis.summary.shimmable_count,
        manual = analysis.summary.needs_manual_count,
        "compatibility analysis complete"
    );

    // 5. Framework detection and pattern matching
    let package_json_path = options.source_path.join("package.json");
    let package_json_text = tokio::fs::read_to_string(package_json_path.as_std_path()).await?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_text)?;

    let framework = options
        .framework_hint
        .as_ref()
        .map(|hint| framework_patterns::FrameworkDetection {
            name: hint.clone(),
            version: None,
        })
        .unwrap_or_else(|| detect_framework(&package_json));

    let mut all_patterns = Vec::new();
    for source in &shaken.sources {
        let patterns = match_patterns(&source.content, &framework);
        all_patterns.extend(patterns);
    }
    info!(
        framework = %framework.name,
        patterns = all_patterns.len(),
        "pattern matching complete"
    );

    // 6. Convert sources
    let conversion = convert_sources(&shaken, &analysis, &all_patterns)?;
    info!(
        converted = conversion.files.len(),
        warnings = conversion.warnings.len(),
        "source conversion complete"
    );

    // 7. Build metadata
    let auto_converted_pct = if analysis.file_analyses.is_empty() {
        100.0
    } else {
        let auto = analysis
            .file_analyses
            .iter()
            .filter(|f| {
                f.compatibility == forge_shared::manifest::Compatibility::Compatible
                    || f.compatibility == forge_shared::manifest::Compatibility::Shimmable
            })
            .count();
        (auto as f32 / analysis.file_analyses.len() as f32) * 100.0
    };

    let metadata = MigrationMetadata {
        source_framework: framework.name.clone(),
        source_node_version: extract_node_version(&package_json),
        migration_date: chrono::Utc::now().to_rfc3339(),
        original_dep_count: count_deps(&package_json),
        resolved_function_count: reachability.module_usage.len(),
        auto_converted_pct,
    };

    // 8. Scaffold project (unless dry-run)
    if !options.dry_run {
        scaffold_project(&options.output_path, &conversion, &metadata)?;
        info!(output = %options.output_path, "project scaffolded");
    }

    // 9. Generate report
    let report = generate_report(&analysis, &conversion);

    Ok(MigrateResult {
        report,
        output_path: options.output_path,
        metadata,
    })
}

/// Extract the Node.js version from `package.json` `engines.node` or `.nvmrc`-style fields.
fn extract_node_version(package_json: &serde_json::Value) -> Option<String> {
    package_json
        .get("engines")
        .and_then(|e| e.get("node"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Count total dependencies (production + dev) in `package.json`.
fn count_deps(package_json: &serde_json::Value) -> usize {
    let prod = package_json
        .get("dependencies")
        .and_then(|d| d.as_object())
        .map_or(0, |o| o.len());
    let dev = package_json
        .get("devDependencies")
        .and_then(|d| d.as_object())
        .map_or(0, |o| o.len());
    prod + dev
}
