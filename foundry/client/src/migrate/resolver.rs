//! Entry point discovery and import graph resolution.
//!
//! This module reads `package.json` to find entry points (`main`, `bin`,
//! `scripts.start`, framework config files like `next.config.*`), then walks
//! all `import` and `require()` statements through `node_modules/` to build
//! the full import graph.
//!
//! The graph is an adjacency list mapping each source file to the set of files
//! it imports. Both ESM (`import`) and CJS (`require`) are supported. Bare
//! specifiers are resolved through `node_modules/` using the Node.js resolution
//! algorithm: `package.json` `exports` map first, then `main`/`module` fields,
//! then `index.js` fallback.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tracing::{debug, warn};

/// The type of an entry point discovered in `package.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPointKind {
    /// The `main` field in `package.json`.
    Main,
    /// A named binary from the `bin` field.
    Bin(String),
    /// The `scripts.start` target (resolved to the actual file if possible).
    StartScript,
    /// A framework-specific entry (e.g., `pages/`, `app/`, `src/index.*`).
    FrameworkConvention(String),
}

/// A discovered entry point into the application.
#[derive(Debug, Clone)]
pub struct EntryPoint {
    /// The resolved file path relative to the project root.
    pub path: Utf8PathBuf,
    /// What kind of entry point this is.
    pub kind: EntryPointKind,
}

/// The import graph: an adjacency list mapping each file to the files it imports.
#[derive(Debug, Clone, Default)]
pub struct ImportGraph {
    /// Map from source file path to the set of file paths it imports.
    pub edges: HashMap<Utf8PathBuf, HashSet<Utf8PathBuf>>,
}

impl ImportGraph {
    /// Return all file paths present in the graph (both importers and imported).
    pub fn all_files(&self) -> HashSet<&Utf8PathBuf> {
        let mut files: HashSet<&Utf8PathBuf> = HashSet::new();
        for (source, targets) in &self.edges {
            files.insert(source);
            for target in targets {
                files.insert(target);
            }
        }
        files
    }
}

/// Discover entry points from `package.json` and walk all imports to build
/// the full import graph.
pub async fn resolve_import_graph(
    project_root: &Utf8Path,
) -> Result<(Vec<EntryPoint>, ImportGraph)> {
    let package_json_path = project_root.join("package.json");
    let package_json_text = tokio::fs::read_to_string(package_json_path.as_std_path())
        .await
        .with_context(|| format!("failed to read {package_json_path}"))?;
    let package_json: serde_json::Value =
        serde_json::from_str(&package_json_text).context("failed to parse package.json")?;

    let entry_points = discover_entry_points(project_root, &package_json).await?;
    debug!(count = entry_points.len(), "discovered entry points");

    let mut graph = ImportGraph::default();
    let mut visited: HashSet<Utf8PathBuf> = HashSet::new();
    let mut queue: VecDeque<Utf8PathBuf> = VecDeque::new();

    for ep in &entry_points {
        let abs = project_root.join(&ep.path);
        if !visited.contains(&abs) {
            visited.insert(abs.clone());
            queue.push_back(abs);
        }
    }

    while let Some(file_path) = queue.pop_front() {
        let imports = extract_imports_from_file(&file_path, project_root).await?;
        let mut resolved_imports: HashSet<Utf8PathBuf> = HashSet::new();

        for specifier in &imports {
            if let Some(resolved) = resolve_specifier(specifier, &file_path, project_root) {
                resolved_imports.insert(resolved.clone());
                if !visited.contains(&resolved) {
                    visited.insert(resolved.clone());
                    queue.push_back(resolved);
                }
            }
        }

        graph.edges.insert(file_path, resolved_imports);
    }

    Ok((entry_points, graph))
}

/// Discover entry points from `package.json` fields and framework conventions.
async fn discover_entry_points(
    project_root: &Utf8Path,
    package_json: &serde_json::Value,
) -> Result<Vec<EntryPoint>> {
    let mut entries = Vec::new();

    // `main` field
    if let Some(main) = package_json.get("main").and_then(|v| v.as_str()) {
        let path = Utf8PathBuf::from(main);
        if project_root.join(&path).as_std_path().exists() {
            entries.push(EntryPoint {
                path,
                kind: EntryPointKind::Main,
            });
        }
    }

    // `bin` field (string or object)
    if let Some(bin) = package_json.get("bin") {
        match bin {
            serde_json::Value::String(s) => {
                let path = Utf8PathBuf::from(s.as_str());
                if project_root.join(&path).as_std_path().exists() {
                    entries.push(EntryPoint {
                        path,
                        kind: EntryPointKind::Bin("default".into()),
                    });
                }
            }
            serde_json::Value::Object(map) => {
                for (name, val) in map {
                    if let Some(s) = val.as_str() {
                        let path = Utf8PathBuf::from(s);
                        if project_root.join(&path).as_std_path().exists() {
                            entries.push(EntryPoint {
                                path,
                                kind: EntryPointKind::Bin(name.clone()),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // `scripts.start` — try to extract the file argument from the command
    if let Some(start) = package_json
        .get("scripts")
        .and_then(|s| s.get("start"))
        .and_then(|v| v.as_str())
    {
        if let Some(file) = extract_file_from_start_script(start) {
            let path = Utf8PathBuf::from(file);
            if project_root.join(&path).as_std_path().exists() {
                entries.push(EntryPoint {
                    path,
                    kind: EntryPointKind::StartScript,
                });
            }
        }
    }

    // Framework conventions
    let framework_paths = [
        ("next.config.js", "next"),
        ("next.config.mjs", "next"),
        ("next.config.ts", "next"),
        ("vite.config.js", "vite"),
        ("vite.config.ts", "vite"),
        ("nuxt.config.js", "nuxt"),
        ("nuxt.config.ts", "nuxt"),
        ("src/index.tsx", "react-spa"),
        ("src/index.ts", "generic"),
        ("src/index.js", "generic"),
        ("src/main.tsx", "react-spa"),
        ("src/main.ts", "generic"),
        ("src/main.js", "generic"),
        ("src/App.tsx", "react-spa"),
        ("index.ts", "generic"),
        ("index.js", "generic"),
    ];

    for (path, convention) in &framework_paths {
        let full = project_root.join(path);
        if full.as_std_path().exists() {
            let already_present = entries.iter().any(|e| e.path.as_str() == *path);
            if !already_present {
                entries.push(EntryPoint {
                    path: Utf8PathBuf::from(*path),
                    kind: EntryPointKind::FrameworkConvention((*convention).to_string()),
                });
            }
        }
    }

    if entries.is_empty() {
        warn!("no entry points found in package.json or framework conventions");
    }

    Ok(entries)
}

/// Try to extract the file path argument from a `scripts.start` value.
///
/// Handles common patterns like `node src/server.js` or `ts-node src/index.ts`.
fn extract_file_from_start_script(script: &str) -> Option<&str> {
    let parts: Vec<&str> = script.split_whitespace().collect();
    // Look for the first argument that looks like a file path
    parts
        .iter()
        .skip(1)
        .find(|part| {
            part.ends_with(".js")
                || part.ends_with(".ts")
                || part.ends_with(".mjs")
                || part.ends_with(".cjs")
        })
        .copied()
}

/// Parse a source file with oxc and extract all import specifiers.
async fn extract_imports_from_file(
    file_path: &Utf8Path,
    _project_root: &Utf8Path,
) -> Result<Vec<String>> {
    let source = match tokio::fs::read_to_string(file_path.as_std_path()).await {
        Ok(s) => s,
        Err(e) => {
            warn!(path = %file_path, err = %e, "skipping unreadable file");
            return Ok(Vec::new());
        }
    };

    let source_type = source_type_from_path(file_path);
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &source, source_type);
    let parse_result = parser.parse();

    if parse_result.panicked {
        warn!(path = %file_path, "parser panicked, skipping");
        return Ok(Vec::new());
    }

    for error in &parse_result.errors {
        debug!(path = %file_path, err = %error, "parse error (continuing)");
    }

    let mut specifiers = Vec::new();
    let program = &parse_result.program;

    for stmt in &program.body {
        match stmt {
            // ESM: import ... from 'specifier'
            Statement::ImportDeclaration(decl) => {
                specifiers.push(decl.source.value.to_string());
            }
            // ESM: export ... from 'specifier'
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    specifiers.push(source.value.to_string());
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                specifiers.push(decl.source.value.to_string());
            }
            // CJS: const x = require('specifier')
            Statement::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(init) = &declarator.init {
                        if let Some(spec) = extract_require_specifier(init) {
                            specifiers.push(spec);
                        }
                    }
                }
            }
            // CJS: require('specifier') as expression statement
            Statement::ExpressionStatement(expr_stmt) => {
                if let Some(spec) = extract_require_specifier(&expr_stmt.expression) {
                    specifiers.push(spec);
                }
            }
            _ => {}
        }
    }

    Ok(specifiers)
}

/// Extract the string argument from a `require('...')` call expression.
fn extract_require_specifier(expr: &Expression<'_>) -> Option<String> {
    if let Expression::CallExpression(call) = expr {
        if let Expression::Identifier(ident) = &call.callee {
            if ident.name == "require" {
                if let Some(Argument::StringLiteral(lit)) = call.arguments.first() {
                    return Some(lit.value.to_string());
                }
            }
        }
    }
    None
}

/// Resolve an import specifier to an absolute file path.
///
/// Handles:
/// - Relative paths (`./foo`, `../bar`)
/// - Bare specifiers via `node_modules/` lookup (simplified Node.js resolution)
fn resolve_specifier(
    specifier: &str,
    importer: &Utf8Path,
    project_root: &Utf8Path,
) -> Option<Utf8PathBuf> {
    if specifier.starts_with('.') {
        // Relative import
        let dir = importer.parent()?;
        resolve_relative(dir, specifier)
    } else if specifier.starts_with("node:") || is_node_builtin(specifier) {
        // Node.js built-in — included in graph as a marker but not walked further
        None
    } else {
        // Bare specifier — resolve through node_modules
        resolve_bare_specifier(specifier, importer, project_root)
    }
}

/// Resolve a relative import specifier to a file path, trying common extensions.
fn resolve_relative(dir: &Utf8Path, specifier: &str) -> Option<Utf8PathBuf> {
    let base = dir.join(specifier);

    // Try exact path first
    if base.as_std_path().is_file() {
        return Some(base);
    }

    // Try common extensions
    let extensions = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];
    for ext in &extensions {
        let with_ext = Utf8PathBuf::from(format!("{base}{ext}"));
        if with_ext.as_std_path().is_file() {
            return Some(with_ext);
        }
    }

    // Try as directory with index file
    let index_extensions = ["index.ts", "index.tsx", "index.js", "index.jsx"];
    for idx in &index_extensions {
        let index_path = base.join(idx);
        if index_path.as_std_path().is_file() {
            return Some(index_path);
        }
    }

    debug!(specifier, dir = %dir, "could not resolve relative import");
    None
}

/// Resolve a bare specifier through `node_modules/` using a simplified
/// Node.js resolution algorithm.
fn resolve_bare_specifier(
    specifier: &str,
    _importer: &Utf8Path,
    project_root: &Utf8Path,
) -> Option<Utf8PathBuf> {
    // Split scoped packages: `@scope/pkg/path` → (`@scope/pkg`, `path`)
    let (package_name, subpath) = split_bare_specifier(specifier);

    let package_dir = project_root.join("node_modules").join(package_name);
    if !package_dir.as_std_path().is_dir() {
        debug!(specifier, "package not found in node_modules");
        return None;
    }

    // If there is a subpath, resolve it directly
    if let Some(sub) = subpath {
        return resolve_relative(&package_dir, &format!("./{sub}"));
    }

    // Read the package's package.json to find its entry point
    let pkg_json_path = package_dir.join("package.json");
    if let Ok(text) = std::fs::read_to_string(pkg_json_path.as_std_path()) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&text) {
            // Try `exports` map (simplified — just the `.` entry)
            if let Some(exports) = pkg.get("exports") {
                if let Some(resolved) = resolve_exports_field(exports, &package_dir) {
                    return Some(resolved);
                }
            }

            // Try `module` field (ESM entry)
            if let Some(module) = pkg.get("module").and_then(|v| v.as_str()) {
                let path = package_dir.join(module);
                if path.as_std_path().is_file() {
                    return Some(path);
                }
            }

            // Try `main` field
            if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                let path = package_dir.join(main);
                if path.as_std_path().is_file() {
                    return Some(path);
                }
            }
        }
    }

    // Fallback: index.js
    let index = package_dir.join("index.js");
    if index.as_std_path().is_file() {
        return Some(index);
    }

    debug!(specifier, "could not resolve bare specifier entry point");
    None
}

/// Split a bare specifier into package name and optional subpath.
///
/// `"lodash"` → `("lodash", None)`
/// `"lodash/fp"` → `("lodash", Some("fp"))`
/// `"@scope/pkg"` → `("@scope/pkg", None)`
/// `"@scope/pkg/utils"` → `("@scope/pkg", Some("utils"))`
fn split_bare_specifier(specifier: &str) -> (&str, Option<&str>) {
    if specifier.starts_with('@') {
        // Scoped package: find the second `/`
        if let Some(slash_idx) = specifier.find('/') {
            let rest = &specifier[slash_idx + 1..];
            if let Some(second_slash) = rest.find('/') {
                let pkg_end = slash_idx + 1 + second_slash;
                return (&specifier[..pkg_end], Some(&specifier[pkg_end + 1..]));
            }
        }
        (specifier, None)
    } else if let Some(slash_idx) = specifier.find('/') {
        (&specifier[..slash_idx], Some(&specifier[slash_idx + 1..]))
    } else {
        (specifier, None)
    }
}

/// Simplified resolution of the `exports` field in `package.json`.
///
/// Handles the common cases: string value, or object with `.` key pointing
/// to a string or `{ import, require, default }` condition map.
fn resolve_exports_field(
    exports: &serde_json::Value,
    package_dir: &Utf8Path,
) -> Option<Utf8PathBuf> {
    match exports {
        serde_json::Value::String(s) => {
            let path = package_dir.join(s.as_str());
            if path.as_std_path().is_file() {
                return Some(path);
            }
        }
        serde_json::Value::Object(map) => {
            // Try the `.` entry (default export)
            if let Some(dot_entry) = map.get(".") {
                return resolve_condition_value(dot_entry, package_dir);
            }
        }
        _ => {}
    }
    None
}

/// Resolve a condition value from an `exports` entry.
///
/// A condition value can be a string path or an object like
/// `{ import: "./esm/index.js", require: "./cjs/index.js", default: "..." }`.
fn resolve_condition_value(
    value: &serde_json::Value,
    package_dir: &Utf8Path,
) -> Option<Utf8PathBuf> {
    match value {
        serde_json::Value::String(s) => {
            let path = package_dir.join(s.as_str());
            if path.as_std_path().is_file() {
                Some(path)
            } else {
                None
            }
        }
        serde_json::Value::Object(map) => {
            // Prefer `import` → `default` → `require`
            for key in &["import", "default", "require"] {
                if let Some(val) = map.get(*key) {
                    if let Some(resolved) = resolve_condition_value(val, package_dir) {
                        return Some(resolved);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Determine the `SourceType` for oxc parsing based on file extension.
fn source_type_from_path(path: &Utf8Path) -> SourceType {
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
    // Strip subpath for builtins like `fs/promises`
    let base = specifier.split('/').next().unwrap_or(specifier);
    BUILTINS.contains(&base)
}
