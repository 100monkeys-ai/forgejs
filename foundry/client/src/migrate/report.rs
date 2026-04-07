//! Migration report generation.
//!
//! Produces human-readable and JSON-formatted reports summarizing the
//! migration results: compatibility breakdown, auto-converted percentage,
//! files needing manual attention, and suggested next steps.

use serde::Serialize;

use super::analyzer::{AppAnalysis, FileAnalysis};
use super::converter::ConversionResult;
use forge_shared::manifest::Compatibility;

/// The output format for the migration report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Human-readable terminal output with section headers and bullet points.
    Human,
    /// Machine-readable JSON.
    Json,
}

/// A file that needs manual attention after migration.
#[derive(Debug, Clone, Serialize)]
pub struct ManualAttentionItem {
    /// Path to the file.
    pub path: String,
    /// Specific API usages that require manual migration.
    pub apis: Vec<ManualAttentionApi>,
}

/// A single API usage requiring manual attention.
#[derive(Debug, Clone, Serialize)]
pub struct ManualAttentionApi {
    /// The API pattern (e.g., `require('fs')`).
    pub pattern: String,
    /// Line number where it was found.
    pub line: u32,
}

/// Summary statistics for the migration.
#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    /// Total files analyzed.
    pub total_files: usize,
    /// Files classified as fully compatible.
    pub compatible_files: usize,
    /// Files classified as shimmable (auto-convertible with shims).
    pub shimmable_files: usize,
    /// Files needing manual attention.
    pub manual_attention_files: usize,
    /// Percentage of files auto-converted (compatible + shimmable).
    pub auto_converted_pct: f32,
    /// Total warnings generated during conversion.
    pub total_warnings: usize,
}

/// The full migration report.
#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    /// High-level summary statistics.
    pub summary: ReportSummary,
    /// Files that need manual attention, with specific API details.
    pub manual_attention: Vec<ManualAttentionItem>,
    /// All conversion warnings.
    pub warnings: Vec<String>,
}

/// Generate a migration report from the analysis and conversion results.
pub fn generate_report(analysis: &AppAnalysis, conversion: &ConversionResult) -> MigrationReport {
    let total_files = analysis.file_analyses.len();
    let compatible_files = analysis.summary.compatible_count;
    let shimmable_files = analysis.summary.shimmable_count;
    let manual_attention_files = analysis.summary.needs_manual_count;

    let auto_converted_pct = if total_files == 0 {
        100.0
    } else {
        ((compatible_files + shimmable_files) as f32 / total_files as f32) * 100.0
    };

    let manual_attention = build_manual_attention_list(&analysis.file_analyses);

    let warnings = conversion
        .warnings
        .iter()
        .map(|w| {
            if let Some(line) = w.line {
                format!("{}:{}: {}", w.path, line, w.message)
            } else {
                format!("{}: {}", w.path, w.message)
            }
        })
        .collect();

    MigrationReport {
        summary: ReportSummary {
            total_files,
            compatible_files,
            shimmable_files,
            manual_attention_files,
            auto_converted_pct,
            total_warnings: conversion.warnings.len(),
        },
        manual_attention,
        warnings,
    }
}

/// Format a migration report for display.
pub fn format_report(report: &MigrationReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Human => format_human(report),
        ReportFormat::Json => format_json(report),
    }
}

/// Build the list of files needing manual attention with their specific APIs.
fn build_manual_attention_list(analyses: &[FileAnalysis]) -> Vec<ManualAttentionItem> {
    analyses
        .iter()
        .filter(|fa| fa.compatibility == Compatibility::NeedsManualAttention)
        .map(|fa| ManualAttentionItem {
            path: fa.path.to_string(),
            apis: fa
                .detected_apis
                .iter()
                .filter(|api| api.compatibility == Compatibility::NeedsManualAttention)
                .map(|api| ManualAttentionApi {
                    pattern: api.pattern.clone(),
                    line: api.line,
                })
                .collect(),
        })
        .collect()
}

/// Format the report as human-readable terminal output.
fn format_human(report: &MigrationReport) -> String {
    let mut out = String::new();

    out.push_str("=== Forge Migration Report ===\n\n");

    // Summary
    out.push_str("Summary\n");
    out.push_str("-------\n");
    out.push_str(&format!(
        "  Files analyzed:        {}\n",
        report.summary.total_files
    ));
    out.push_str(&format!(
        "  Compatible:            {} (direct conversion)\n",
        report.summary.compatible_files
    ));
    out.push_str(&format!(
        "  Shimmable:             {} (auto-converted with shims)\n",
        report.summary.shimmable_files
    ));
    out.push_str(&format!(
        "  Needs manual work:     {}\n",
        report.summary.manual_attention_files
    ));
    out.push_str(&format!(
        "  Auto-converted:        {:.1}%\n",
        report.summary.auto_converted_pct
    ));
    out.push('\n');

    // Files needing manual attention
    if !report.manual_attention.is_empty() {
        out.push_str("Files Needing Manual Attention\n");
        out.push_str("------------------------------\n");
        for item in &report.manual_attention {
            out.push_str(&format!("  {}\n", item.path));
            for api in &item.apis {
                out.push_str(&format!("    line {}: {}\n", api.line, api.pattern));
            }
        }
        out.push('\n');
    }

    // Warnings
    if !report.warnings.is_empty() {
        out.push_str("Warnings\n");
        out.push_str("--------\n");
        for warning in &report.warnings {
            out.push_str(&format!("  {warning}\n"));
        }
        out.push('\n');
    }

    // Next steps
    out.push_str("Next Steps\n");
    out.push_str("----------\n");
    out.push_str("  1. Review files marked as needing manual attention\n");
    out.push_str("  2. Resolve bare imports: add Foundry packages or inline the code\n");
    out.push_str("  3. Run `forge dev` to start the development server\n");
    out.push_str("  4. Run `forge build` to verify the project compiles\n");

    out
}

/// Format the report as JSON.
fn format_json(report: &MigrationReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}
