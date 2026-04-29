//! Source-level transforms: `.ts`/`.tsx`/`.js`/`.jsx` → `.fx`
//!
//! Applies the full set of source transformations to convert Node.js source
//! files into Forge `.fx` files:
//!
//! - File extension renaming to `.fx`
//! - Import path rewriting (bare specifiers → relative paths)
//! - CJS → ESM conversion (`require` → `import`, `module.exports` → `export`)
//! - Framework pattern application (React hooks → Signals, Express → server functions)
//! - Shim injection for shimmable APIs

use std::collections::HashMap;

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use tracing::debug;

use super::analyzer::AppAnalysis;
use super::framework_patterns::{PatternMatch, TransformInstruction};
use super::tree_shaker::ShakenApp;
use forge_shared::manifest::Compatibility;

/// A single converted output file.
#[derive(Debug, Clone)]
pub struct ConvertedFile {
    /// The output path with `.fx` extension.
    pub path: Utf8PathBuf,
    /// The transformed source content.
    pub content: String,
    /// The original source path before conversion.
    pub original_path: Utf8PathBuf,
}

/// The result of converting all source files.
#[derive(Debug, Clone, Default)]
pub struct ConversionResult {
    /// All converted files.
    pub files: Vec<ConvertedFile>,
    /// Warnings generated during conversion (non-fatal issues).
    pub warnings: Vec<ConversionWarning>,
}

/// A non-fatal warning generated during source conversion.
#[derive(Debug, Clone)]
pub struct ConversionWarning {
    /// The source file that triggered the warning.
    pub path: Utf8PathBuf,
    /// Human-readable warning message.
    pub message: String,
    /// Line number where the issue was found, if applicable.
    pub line: Option<u32>,
}

/// Convert all shaken source files to `.fx` format.
///
/// Applies import rewriting, CJS→ESM conversion, framework pattern transforms,
/// and shim injection based on the analysis and pattern match results.
pub fn convert_sources(
    shaken: &ShakenApp,
    analysis: &AppAnalysis,
    patterns: &[PatternMatch],
) -> Result<ConversionResult> {
    let mut result = ConversionResult::default();

    let analyses_by_path: HashMap<_, _> = analysis
        .file_analyses
        .iter()
        .map(|fa| (&fa.path, fa))
        .collect();

    for source in &shaken.sources {
        let file_analysis = analyses_by_path.get(&source.path).copied();

        let mut content = source.content.clone();

        // 1. CJS → ESM conversion
        content = convert_cjs_to_esm(&content, &source.path, &mut result.warnings);

        // 2. Apply framework pattern transforms
        content = apply_pattern_transforms(&content, patterns);

        // 3. Rewrite import paths (bare specifiers → relative)
        content = rewrite_import_paths(&content, &source.path, &mut result.warnings);

        // 4. Inject shims for shimmable APIs
        if let Some(fa) = file_analysis {
            if fa.compatibility == Compatibility::Shimmable {
                content = inject_shims(&content, fa);
            }
            if fa.compatibility == Compatibility::NeedsManualAttention {
                result.warnings.push(ConversionWarning {
                    path: source.path.clone(),
                    message: "file contains Node.js APIs that require manual migration".into(),
                    line: None,
                });
            }
        }

        // 5. Rename extension to .fx
        let fx_path = rename_to_fx(&source.path);

        result.files.push(ConvertedFile {
            path: fx_path,
            content,
            original_path: source.path.clone(),
        });
    }

    debug!(
        files = result.files.len(),
        warnings = result.warnings.len(),
        "source conversion complete"
    );

    Ok(result)
}

/// Convert CommonJS patterns to ESM equivalents.
///
/// - `const x = require('mod')` → `import x from 'mod'`
/// - `const { a, b } = require('mod')` → `import { a, b } from 'mod'`
/// - `module.exports = x` → `export default x`
/// - `module.exports.name = x` → `export const name = x`
/// - `exports.name = x` → `export const name = x`
fn convert_cjs_to_esm(
    source: &str,
    path: &Utf8Path,
    warnings: &mut Vec<ConversionWarning>,
) -> String {
    let mut output = String::with_capacity(source.len());

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // const/let/var x = require('mod')
        if let Some(converted) = try_convert_require_declaration(trimmed) {
            output.push_str(&converted);
        }
        // module.exports = expr
        else if trimmed.starts_with("module.exports") {
            if let Some(converted) = try_convert_module_exports(trimmed) {
                output.push_str(&converted);
            } else {
                output.push_str(line);
                warnings.push(ConversionWarning {
                    path: Utf8PathBuf::from(path),
                    message: format!("could not auto-convert CJS export: {trimmed}"),
                    line: Some(line_num as u32 + 1),
                });
            }
        }
        // exports.name = expr
        else if trimmed.starts_with("exports.") && trimmed.contains('=') {
            if let Some(converted) = try_convert_named_export(trimmed) {
                output.push_str(&converted);
            } else {
                output.push_str(line);
            }
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !source.ends_with('\n') {
        output.pop();
    }

    output
}

/// Try to convert `const/let/var x = require('mod')` to an ESM import.
fn try_convert_require_declaration(line: &str) -> Option<String> {
    // Match: const/let/var IDENT = require('SPECIFIER')
    let rest = line
        .strip_prefix("const ")
        .or_else(|| line.strip_prefix("let "))
        .or_else(|| line.strip_prefix("var "))?;

    let eq_idx = rest.find('=')?;
    let binding = rest[..eq_idx].trim();
    let after_eq = rest[eq_idx + 1..].trim();

    // Match require('...' or require("...")
    let spec_start = after_eq.strip_prefix("require(")?;
    let spec_content = spec_start.strip_suffix(");")?;
    let spec_content = spec_content
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| {
            spec_content
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
        })?;

    Some(format!("import {binding} from '{spec_content}';"))
}

/// Try to convert `module.exports = expr` to `export default expr`.
fn try_convert_module_exports(line: &str) -> Option<String> {
    let rest = line.strip_prefix("module.exports")?;
    let rest = rest.trim();

    if let Some(value) = rest.strip_prefix('=') {
        let value = value.trim().trim_end_matches(';');
        Some(format!("export default {value};"))
    } else if let Some(rest) = rest.strip_prefix('.') {
        // module.exports.name = expr
        let eq_idx = rest.find('=')?;
        let name = rest[..eq_idx].trim();
        let value = rest[eq_idx + 1..].trim().trim_end_matches(';');
        Some(format!("export const {name} = {value};"))
    } else {
        None
    }
}

/// Try to convert `exports.name = expr` to `export const name = expr`.
fn try_convert_named_export(line: &str) -> Option<String> {
    let rest = line.strip_prefix("exports.")?;
    let eq_idx = rest.find('=')?;
    let name = rest[..eq_idx].trim();
    let value = rest[eq_idx + 1..].trim().trim_end_matches(';');
    Some(format!("export const {name} = {value};"))
}

/// Apply framework-specific pattern transforms to source content.
///
/// Uses simple text replacement for hook renames and unwrapping. This is
/// deliberately line-based rather than AST-based because the converter runs
/// after CJS→ESM conversion, which may have altered the AST structure.
fn apply_pattern_transforms(source: &str, patterns: &[PatternMatch]) -> String {
    let mut result = source.to_string();

    for pattern in patterns {
        match &pattern.instruction {
            TransformInstruction::RenameCall { from, to } => {
                // Replace call name: useState( → $signal(
                result = result.replace(&format!("{from}("), &format!("{to}("));
            }
            TransformInstruction::UnwrapCall { wrapper } => {
                // useCallback(fn, [deps]) → fn
                // This is a simplified transform that handles the common case.
                // Complex cases will generate a warning and require manual attention.
                result = unwrap_call(&result, wrapper);
            }
            TransformInstruction::ReplaceWithLet { hook } => {
                // useRef(init) → init (the variable declaration is preserved)
                result = result.replace(&format!("{hook}("), "(");
            }
            TransformInstruction::ExpressRouteToServerFunction { method, path } => {
                // Express route conversion is complex enough that we add a
                // comment annotation for now and let the developer complete
                // the migration. The scaffold step will generate the route
                // configuration.
                let annotation =
                    format!("// FORGE: migrate to `server async function` — {method} {path}");
                // Insert annotation before the line containing this pattern
                if let Some(idx) = result.find(&pattern.source_pattern) {
                    // Find the start of the line
                    let line_start = result[..idx].rfind('\n').map_or(0, |i| i + 1);
                    result.insert_str(line_start, &format!("{annotation}\n"));
                }
            }
            TransformInstruction::ExpressResponseToReturn => {
                // res.json(data) → return data
                result = result.replace("res.json(", "return (");
            }
        }
    }

    result
}

/// Unwrap a `useCallback(fn, deps)` call to just `fn`.
///
/// Handles the common case where the first argument is a function expression
/// or arrow function. Falls back to no change if the pattern is complex.
fn unwrap_call(source: &str, wrapper: &str) -> String {
    let search = format!("{wrapper}(");
    let mut result = String::with_capacity(source.len());
    let mut remaining = source;

    while let Some(idx) = remaining.find(&search) {
        result.push_str(&remaining[..idx]);
        let after = &remaining[idx + search.len()..];

        // Find the matching closing paren, counting nesting
        if let Some(fn_end) = find_first_arg_end(after) {
            let first_arg = &after[..fn_end];
            result.push_str(first_arg);
            // Skip past the closing paren of the wrapper call
            let rest = &after[fn_end..];
            if let Some(close) = find_closing_paren(rest) {
                remaining = &rest[close + 1..];
            } else {
                remaining = rest;
            }
        } else {
            // Couldn't parse — leave unchanged
            result.push_str(&search);
            remaining = after;
        }
    }

    result.push_str(remaining);
    result
}

/// Find the end of the first argument in a comma-separated argument list,
/// respecting nested parentheses, brackets, and braces.
fn find_first_arg_end(s: &str) -> Option<usize> {
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;

    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth_paren += 1,
            ')' => {
                if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 {
                    return Some(i);
                }
                depth_paren -= 1;
            }
            '[' => depth_bracket += 1,
            ']' => depth_bracket -= 1,
            '{' => depth_brace += 1,
            '}' => depth_brace -= 1,
            ',' if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                return Some(i);
            }
            _ => {}
        }
    }

    None
}

/// Find the closing parenthesis, handling nesting.
fn find_closing_paren(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Rewrite import paths for the Forge module system.
///
/// Bare specifiers that resolved through `node_modules/` need to be rewritten
/// since Forge does not have a `node_modules/` directory. For now, this adds
/// a comment marking imports that need manual attention.
fn rewrite_import_paths(
    source: &str,
    path: &Utf8Path,
    warnings: &mut Vec<ConversionWarning>,
) -> String {
    let mut output = String::with_capacity(source.len());

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check for import statements with bare specifiers
        if (trimmed.starts_with("import ") || trimmed.starts_with("export "))
            && trimmed.contains(" from ")
        {
            if let Some(specifier) = extract_import_specifier_from_line(trimmed) {
                if !specifier.starts_with('.')
                    && !specifier.starts_with('/')
                    && !specifier.starts_with("node:")
                {
                    // Bare specifier — mark for attention
                    output.push_str(line);
                    output.push_str(" // FORGE: bare import — resolve via Foundry or inline");
                    output.push('\n');

                    warnings.push(ConversionWarning {
                        path: Utf8PathBuf::from(path),
                        message: format!(
                            "bare import '{specifier}' needs Foundry package or inline resolution"
                        ),
                        line: Some(line_num as u32 + 1),
                    });
                    continue;
                }
            }
        }

        output.push_str(line);
        output.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !source.ends_with('\n') {
        output.pop();
    }

    output
}

/// Extract the module specifier from an import/export line.
fn extract_import_specifier_from_line(line: &str) -> Option<String> {
    // Find the last quoted string (the `from 'specifier'` part)
    let from_idx = line.rfind(" from ")?;
    let after_from = line[from_idx + 6..].trim().trim_end_matches(';');

    let specifier = after_from
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .or_else(|| {
            after_from
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
        })?;

    Some(specifier.to_string())
}

/// Rename a file path's extension to `.fx`.
fn rename_to_fx(path: &Utf8Path) -> Utf8PathBuf {
    let stem = path.file_stem().unwrap_or("unknown");
    if let Some(parent) = path.parent() {
        parent.join(format!("{stem}.fx"))
    } else {
        Utf8PathBuf::from(format!("{stem}.fx"))
    }
}

/// Inject Forge shim imports for shimmable Node.js APIs.
fn inject_shims(source: &str, analysis: &super::analyzer::FileAnalysis) -> String {
    let mut shim_imports: Vec<String> = Vec::new();

    for api in &analysis.detected_apis {
        if api.compatibility != Compatibility::Shimmable {
            continue;
        }

        match api.pattern.as_str() {
            "Buffer.from()" | "Buffer.alloc()" | "Buffer (global)" => {
                let import = "import { Buffer } from 'forge:buffer';".to_string();
                if !shim_imports.contains(&import) {
                    shim_imports.push(import);
                }
            }
            "crypto.randomBytes()" => {
                let import = "import { randomBytes } from 'forge:crypto';".to_string();
                if !shim_imports.contains(&import) {
                    shim_imports.push(import);
                }
            }
            "process.env" => {
                let import = "import { env } from 'forge:runtime';".to_string();
                if !shim_imports.contains(&import) {
                    shim_imports.push(import);
                }
            }
            p if p.starts_with("import '") && p.contains("events") => {
                let import = "import { EventEmitter } from 'forge:events';".to_string();
                if !shim_imports.contains(&import) {
                    shim_imports.push(import);
                }
            }
            _ => {}
        }
    }

    if shim_imports.is_empty() {
        return source.to_string();
    }

    let mut output = String::with_capacity(source.len() + shim_imports.len() * 60);
    output.push_str("// Forge shims for Node.js API compatibility\n");
    for import in &shim_imports {
        output.push_str(import);
        output.push('\n');
    }
    output.push('\n');
    output.push_str(source);

    output
}
