//! Compatibility analysis: classify residual code after tree-shaking.
//!
//! Parses each retained source file with `oxc_parser` and scans for
//! Node.js-specific API usage. Produces per-file and aggregate compatibility
//! classifications using the `Compatibility` enum from `forge-shared`.

use anyhow::Result;
use camino::Utf8PathBuf;
use forge_shared::manifest::Compatibility;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tracing::debug;

use super::tree_shaker::ShakenApp;

/// A Node.js API detected in a source file.
#[derive(Debug, Clone)]
pub struct DetectedApi {
    /// The API pattern detected (e.g., `require('fs')`, `process.exit`).
    pub pattern: String,
    /// The line number where the usage was found.
    pub line: u32,
    /// The compatibility tier of this particular API usage.
    pub compatibility: Compatibility,
}

/// Per-file compatibility analysis result.
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    /// Path to the analyzed source file.
    pub path: Utf8PathBuf,
    /// The overall compatibility tier for this file (worst-case across all APIs).
    pub compatibility: Compatibility,
    /// Individual API detections.
    pub detected_apis: Vec<DetectedApi>,
}

/// Aggregate summary statistics.
#[derive(Debug, Clone, Default)]
pub struct AnalysisSummary {
    /// Number of files classified as `Compatible`.
    pub compatible_count: usize,
    /// Number of files classified as `Shimmable`.
    pub shimmable_count: usize,
    /// Number of files classified as `NeedsManualAttention`.
    pub needs_manual_count: usize,
    /// Total number of individual API detections.
    pub total_api_detections: usize,
}

/// Aggregate analysis result across all source files.
#[derive(Debug, Clone)]
pub struct AppAnalysis {
    /// Per-file analysis results.
    pub file_analyses: Vec<FileAnalysis>,
    /// Aggregate summary.
    pub summary: AnalysisSummary,
}

/// Analyze all shaken source files for Node.js API compatibility.
pub fn analyze_compatibility(shaken: &ShakenApp) -> Result<AppAnalysis> {
    let mut file_analyses = Vec::with_capacity(shaken.sources.len());

    for source in &shaken.sources {
        let analysis = analyze_file(&source.path, &source.content)?;
        file_analyses.push(analysis);
    }

    let summary = compute_summary(&file_analyses);

    debug!(
        files = file_analyses.len(),
        compatible = summary.compatible_count,
        shimmable = summary.shimmable_count,
        manual = summary.needs_manual_count,
        "compatibility analysis complete"
    );

    Ok(AppAnalysis {
        file_analyses,
        summary,
    })
}

/// Analyze a single source file for Node.js API usage.
fn analyze_file(path: &Utf8PathBuf, source: &str) -> Result<FileAnalysis> {
    let source_type = source_type_from_path(path);
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source, source_type);
    let parse_result = parser.parse();

    let mut detected_apis = Vec::new();

    if !parse_result.panicked {
        let program = &parse_result.program;
        scan_statements(&program.body, source, &mut detected_apis);
    }

    let compatibility = worst_case_compatibility(&detected_apis);

    Ok(FileAnalysis {
        path: path.clone(),
        compatibility,
        detected_apis,
    })
}

/// Scan top-level statements for Node.js API usage.
fn scan_statements(
    stmts: &oxc_allocator::Vec<'_, Statement<'_>>,
    source: &str,
    detected: &mut Vec<DetectedApi>,
) {
    for stmt in stmts.iter() {
        match stmt {
            // Check require() calls for Node.js built-in modules
            Statement::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(init) = &declarator.init {
                        check_expression(init, source, detected);
                    }
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                check_expression(&expr_stmt.expression, source, detected);
            }
            // Check import declarations for node: protocol and Node.js builtins
            Statement::ImportDeclaration(decl) => {
                let specifier = decl.source.value.as_str();
                if let Some(api) = classify_import_specifier(specifier) {
                    let line = line_number_at_offset(source, decl.span.start);
                    detected.push(api.with_line(line));
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source_lit) = &decl.source {
                    let specifier = source_lit.value.as_str();
                    if let Some(api) = classify_import_specifier(specifier) {
                        let line = line_number_at_offset(source, decl.span.start);
                        detected.push(api.with_line(line));
                    }
                }
            }
            _ => {}
        }
    }
}
/// Check an expression tree for Node.js API patterns.
fn check_expression(expr: &Expression<'_>, source: &str, detected: &mut Vec<DetectedApi>) {
    match expr {
        Expression::CallExpression(call) => {
            check_call_expression(call, source, detected);
        }
        Expression::StaticMemberExpression(member) => {
            check_static_member_expression(member, source, detected);
        }
        Expression::Identifier(ident) => {
            check_identifier(ident, source, detected);
        }
        Expression::AssignmentExpression(assign) => {
            check_assignment_expression(assign, source, detected);
        }
        _ => {}
    }
}

fn check_call_expression(call: &oxc_ast::ast::CallExpression<'_>, source: &str, detected: &mut Vec<DetectedApi>) {
    // require('fs'), require('child_process'), etc.
    if let Expression::Identifier(ident) = &call.callee {
        if ident.name == "require" {
            if let Some(Argument::StringLiteral(lit)) = call.arguments.first() {
                if let Some(api) = classify_import_specifier(lit.value.as_str()) {
                    let line = line_number_at_offset(source, call.span.start);
                    detected.push(api.with_line(line));
                }
            }
        }
    }

    // Check for shimmable patterns: Buffer.from(), Buffer.alloc(), crypto.randomBytes()
    if let Expression::StaticMemberExpression(member) = &call.callee {
        let prop = member.property.name.as_str();
        if let Expression::Identifier(obj) = &member.object {
            let obj_name = obj.name.as_str();
            if obj_name == "Buffer" && (prop == "from" || prop == "alloc") {
                let line = line_number_at_offset(source, call.span.start);
                detected.push(DetectedApi {
                    pattern: format!("Buffer.{prop}()"),
                    line,
                    compatibility: Compatibility::Shimmable,
                });
            }
            if obj_name == "crypto" && prop == "randomBytes" {
                let line = line_number_at_offset(source, call.span.start);
                detected.push(DetectedApi {
                    pattern: "crypto.randomBytes()".into(),
                    line,
                    compatibility: Compatibility::Shimmable,
                });
            }
        }
    }

    // process.exit()
    if let Expression::StaticMemberExpression(member) = &call.callee {
        if member.property.name == "exit" {
            if let Expression::Identifier(obj) = &member.object {
                if obj.name == "process" {
                    let line = line_number_at_offset(source, call.span.start);
                    detected.push(DetectedApi {
                        pattern: "process.exit()".into(),
                        line,
                        compatibility: Compatibility::NeedsManualAttention,
                    });
                }
            }
        }
    }

    // Recurse into call arguments
    for arg in &call.arguments {
        if let Argument::FunctionExpression(fn_expr) = arg {
            if let Some(body) = &fn_expr.body {
                scan_statements(&body.statements, source, detected);
            }
        }
        if let Argument::ArrowFunctionExpression(arrow) = arg {
            scan_statements(&arrow.body.statements, source, detected);
        }
    }
}

fn check_static_member_expression(member: &oxc_ast::ast::StaticMemberExpression<'_>, source: &str, detected: &mut Vec<DetectedApi>) {
    // process.env access (shimmable → Forge.env())
    if member.property.name == "env" {
        if let Expression::Identifier(obj) = &member.object {
            if obj.name == "process" {
                let line = line_number_at_offset(source, member.span.start);
                detected.push(DetectedApi {
                    pattern: "process.env".into(),
                    line,
                    compatibility: Compatibility::Shimmable,
                });
            }
        }
    }
    // __dirname, __filename as member access targets are covered below
}

fn check_identifier(ident: &oxc_ast::ast::IdentifierReference<'_>, source: &str, detected: &mut Vec<DetectedApi>) {
    // Standalone identifiers: __dirname, __filename, Buffer (as global)
    let name = ident.name.as_str();
    match name {
        "__dirname" | "__filename" => {
            let line = line_number_at_offset(source, ident.span.start);
            detected.push(DetectedApi {
                pattern: name.to_string(),
                line,
                compatibility: Compatibility::NeedsManualAttention,
            });
        }
        "Buffer" => {
            let line = line_number_at_offset(source, ident.span.start);
            detected.push(DetectedApi {
                pattern: "Buffer (global)".into(),
                line,
                compatibility: Compatibility::NeedsManualAttention,
            });
        }
        _ => {}
    }
}

fn check_assignment_expression(assign: &oxc_ast::ast::AssignmentExpression<'_>, source: &str, detected: &mut Vec<DetectedApi>) {
    // module.exports (CJS export pattern)
    if let Some(member) = extract_member_pattern(&assign.left) {
        if member == "module.exports" {
            let line = line_number_at_offset(source, assign.span.start);
            detected.push(DetectedApi {
                pattern: "module.exports".into(),
                line,
                compatibility: Compatibility::NeedsManualAttention,
            });
        }
    }
    check_expression(&assign.right, source, detected);
}
/// Try to extract a `obj.prop` pattern string from an assignment target.
fn extract_member_pattern(target: &oxc_ast::ast::AssignmentTarget<'_>) -> Option<String> {
    if let oxc_ast::ast::AssignmentTarget::StaticMemberExpression(member) = target {
        if let Expression::Identifier(obj) = &member.object {
            return Some(format!("{}.{}", obj.name, member.property.name));
        }
    }
    None
}

/// Classify an import specifier as a Node.js API detection, if applicable.
fn classify_import_specifier(specifier: &str) -> Option<ApiClassification> {
    // node: protocol — always a Node.js import
    if let Some(module) = specifier.strip_prefix("node:") {
        return Some(classify_node_module(module, specifier));
    }

    // Check against known Node.js built-in modules
    let base = specifier.split('/').next().unwrap_or(specifier);
    if is_node_builtin(base) {
        return Some(classify_node_module(base, specifier));
    }

    None
}

/// Temporary type to carry classification before line number is known.
struct ApiClassification {
    pattern: String,
    compatibility: Compatibility,
}

impl ApiClassification {
    fn with_line(self, line: u32) -> DetectedApi {
        DetectedApi {
            pattern: self.pattern,
            line,
            compatibility: self.compatibility,
        }
    }
}

/// Classify a Node.js built-in module into a compatibility tier.
fn classify_node_module(module: &str, full_specifier: &str) -> ApiClassification {
    let (compat, pattern) = match module {
        // Incompatible — no WinterTC equivalent
        "fs" | "child_process" | "cluster" | "dgram" | "dns" | "net" | "readline" | "repl"
        | "tls" | "tty" | "v8" | "vm" | "worker_threads" => {
            (Compatibility::NeedsManualAttention, full_specifier)
        }
        // Shimmable — WinterTC or Forge equivalents exist
        "buffer" | "crypto" | "events" | "path" | "stream" | "url" | "util" | "string_decoder"
        | "querystring" => (Compatibility::Shimmable, full_specifier),
        // Everything else defaults to needs-manual
        _ => (Compatibility::NeedsManualAttention, full_specifier),
    };

    ApiClassification {
        pattern: format!("import '{pattern}'"),
        compatibility: compat,
    }
}

/// Determine worst-case compatibility across all detected APIs.
fn worst_case_compatibility(apis: &[DetectedApi]) -> Compatibility {
    let mut worst = Compatibility::Compatible;
    for api in apis {
        worst = match (worst, api.compatibility) {
            (Compatibility::NeedsManualAttention, _) | (_, Compatibility::NeedsManualAttention) => {
                Compatibility::NeedsManualAttention
            }
            (Compatibility::Shimmable, _) | (_, Compatibility::Shimmable) => {
                Compatibility::Shimmable
            }
            _ => Compatibility::Compatible,
        };
    }
    worst
}

/// Compute aggregate summary statistics from per-file analyses.
fn compute_summary(analyses: &[FileAnalysis]) -> AnalysisSummary {
    let mut summary = AnalysisSummary::default();
    for analysis in analyses {
        match analysis.compatibility {
            Compatibility::Compatible => summary.compatible_count += 1,
            Compatibility::Shimmable => summary.shimmable_count += 1,
            Compatibility::NeedsManualAttention => summary.needs_manual_count += 1,
        }
        summary.total_api_detections += analysis.detected_apis.len();
    }
    summary
}

/// Compute the 1-based line number for a byte offset in the source.
fn line_number_at_offset(source: &str, offset: u32) -> u32 {
    let offset = offset as usize;
    source[..offset.min(source.len())]
        .chars()
        .filter(|c| *c == '\n')
        .count() as u32
        + 1
}

/// Determine the `SourceType` for oxc parsing based on file extension.
fn source_type_from_path(path: &Utf8PathBuf) -> SourceType {
    let ext = path.extension().unwrap_or("");
    match ext {
        "tsx" => SourceType::tsx(),
        "ts" | "mts" | "cts" => SourceType::ts(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}

/// Check if a specifier is a Node.js built-in module.
fn is_node_builtin(specifier: &str) -> bool {
    const BUILTINS: &[&str] = &[
        "assert",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "http2",
        "https",
        "module",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "sys",
        "timers",
        "tls",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "worker_threads",
        "zlib",
    ];
    BUILTINS.contains(&specifier)
}
