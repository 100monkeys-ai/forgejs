//! The `forge migrate` command.
//!
//! Converts an existing Node.js application into a Forge project by resolving
//! the full import graph from entry points, tree-shaking unused code, analyzing
//! the residual for Node.js API compatibility, applying framework pattern
//! transforms (React → Signals, Express → server functions), and scaffolding
//! a new Forge project with `.fx` source files.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Migrate an existing Node.js application to a Forge project.
///
/// Resolves the app's actual dependency usage, tree-shakes dead code,
/// and converts the residual to Forge `.fx` files with framework-aware
/// pattern matching.
#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Path to the Node.js project root (must contain package.json)
    pub source: PathBuf,

    /// Output directory for the generated Forge project
    #[arg(long, short, default_value = "forge-app")]
    pub output: PathBuf,

    /// Analyze and report only — do not write any files
    #[arg(long)]
    pub dry_run: bool,

    /// Framework detection hint (auto-detected from dependencies by default)
    #[arg(long, value_enum, default_value = "auto")]
    pub framework: FrameworkHint,

    /// Output format for the migration report
    #[arg(long, value_enum, default_value = "human")]
    pub report_format: ReportFormatArg,
}

/// Framework hint for pattern matching.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum FrameworkHint {
    Auto,
    React,
    Express,
    Next,
    Fastify,
    None,
}

/// Report output format.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ReportFormatArg {
    Human,
    Json,
}

pub async fn run(args: MigrateArgs) -> Result<()> {
    use foundry_client::migrate::report::ReportFormat;
    use foundry_client::migrate::{migrate_app, MigrateOptions};

    // Validate source
    let source = args
        .source
        .canonicalize()
        .map_err(|_| anyhow::anyhow!("Source path does not exist: {}", args.source.display()))?;

    if !source.join("package.json").exists() {
        anyhow::bail!("No package.json found at {}", source.display());
    }

    tracing::info!("Migrating Node.js app at {}", source.display());

    let options = MigrateOptions {
        source_path: camino::Utf8PathBuf::try_from(source)
            .map_err(|e| anyhow::anyhow!("source path is not valid UTF-8: {e}"))?,
        output_path: camino::Utf8PathBuf::try_from(args.output.clone())
            .map_err(|e| anyhow::anyhow!("output path is not valid UTF-8: {e}"))?,
        dry_run: args.dry_run,
        framework_hint: match args.framework {
            FrameworkHint::Auto => None,
            FrameworkHint::React => Some("react".into()),
            FrameworkHint::Express => Some("express".into()),
            FrameworkHint::Next => Some("next".into()),
            FrameworkHint::Fastify => Some("fastify".into()),
            FrameworkHint::None => Some("none".into()),
        },
        include_dev_deps: false,
    };

    let result = migrate_app(options).await?;

    let report_format = match args.report_format {
        ReportFormatArg::Human => ReportFormat::Human,
        ReportFormatArg::Json => ReportFormat::Json,
    };

    use foundry_client::migrate::report::format_report;
    let formatted = format_report(&result.report, report_format);
    println!("{formatted}");

    if !args.dry_run {
        tracing::info!("Forge project generated at {}", result.output_path);
    }

    Ok(())
}
