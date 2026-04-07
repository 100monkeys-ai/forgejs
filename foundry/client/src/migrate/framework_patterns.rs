//! Framework pattern detection and transformation instructions.
//!
//! Identifies React, Express, and Next.js idioms in source files and produces
//! transformation instructions that the converter applies during migration.
//!
//! ## React → Forge Signals Mappings
//!
//! | React Hook | Forge Equivalent | Notes |
//! |---|---|---|
//! | `useState(init)` | `$signal(init)` | Direct 1:1 mapping |
//! | `useEffect(fn, deps)` | `$effect(fn)` | Deps inferred automatically |
//! | `useMemo(fn, deps)` | `$derived(fn)` | Deps inferred automatically |
//! | `useCallback(fn, deps)` | just `fn` | No wrapper needed — signals track deps |
//! | `useRef(init)` | `let ref = init` | No special wrapper needed |
//!
//! ## Express → Server Function Mappings
//!
//! | Express Pattern | Forge Equivalent | Notes |
//! |---|---|---|
//! | `app.get('/path', handler)` | `server async function` + `forge:router` route | Handler becomes a server function |
//! | `req.params` / `req.query` / `req.body` | Typed function parameters | Type-safe by default |
//! | `res.json(data)` | Return value | Just return the data |
//! | `res.status(code).json(data)` | Return with status | `return { status: code, body: data }` |

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tracing::debug;

/// A detected framework and its version.
#[derive(Debug, Clone)]
pub struct FrameworkDetection {
    /// Framework name (e.g., `"react"`, `"express"`, `"next"`).
    pub name: String,
    /// Version from `package.json` dependencies, if found.
    pub version: Option<String>,
}

impl std::fmt::Display for FrameworkDetection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            Some(v) => write!(f, "{}@{}", self.name, v),
            None => write!(f, "{}", self.name),
        }
    }
}

/// A matched framework pattern with source location and Forge equivalent.
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// The source pattern that was matched (e.g., `"useState"`).
    pub source_pattern: String,
    /// The recommended Forge equivalent (e.g., `"$signal"`).
    pub forge_equivalent: String,
    /// Line number in the source file where the pattern was found.
    pub line: u32,
    /// The transformation instruction to apply.
    pub instruction: TransformInstruction,
}

/// A concrete transformation instruction for the converter to apply.
#[derive(Debug, Clone)]
pub enum TransformInstruction {
    /// Replace a function call name: `useState(x)` → `$signal(x)`.
    RenameCall {
        /// Original call name.
        from: String,
        /// Replacement call name.
        to: String,
    },
    /// Remove a wrapper entirely, keeping the first argument: `useCallback(fn, deps)` → `fn`.
    UnwrapCall {
        /// The wrapper function name to remove.
        wrapper: String,
    },
    /// Replace a variable declaration pattern: `useRef(init)` → `let ref = init`.
    ReplaceWithLet {
        /// The hook call to replace.
        hook: String,
    },
    /// Convert an Express route handler to a Forge server function.
    ExpressRouteToServerFunction {
        /// HTTP method (get, post, put, delete, etc.).
        method: String,
        /// The route path.
        path: String,
    },
    /// Replace an Express response pattern with a return statement.
    ExpressResponseToReturn,
}

/// Detect the primary framework from `package.json` dependencies.
pub fn detect_framework(package_json: &serde_json::Value) -> FrameworkDetection {
    let deps = package_json.get("dependencies").and_then(|d| d.as_object());
    let dev_deps = package_json
        .get("devDependencies")
        .and_then(|d| d.as_object());

    // Check in priority order: Next.js > React > Express > generic
    let all_dep_names: Vec<(&String, &serde_json::Value)> = deps
        .into_iter()
        .chain(dev_deps)
        .flat_map(|m| m.iter())
        .collect();

    // Next.js (includes React)
    if let Some((_, version)) = all_dep_names.iter().find(|(k, _)| k.as_str() == "next") {
        return FrameworkDetection {
            name: "next".into(),
            version: version.as_str().map(String::from),
        };
    }

    // React (standalone SPA or with other bundlers)
    if let Some((_, version)) = all_dep_names.iter().find(|(k, _)| k.as_str() == "react") {
        return FrameworkDetection {
            name: "react".into(),
            version: version.as_str().map(String::from),
        };
    }

    // Express
    if let Some((_, version)) = all_dep_names.iter().find(|(k, _)| k.as_str() == "express") {
        return FrameworkDetection {
            name: "express".into(),
            version: version.as_str().map(String::from),
        };
    }

    // No recognized framework
    FrameworkDetection {
        name: "generic".into(),
        version: None,
    }
}

/// Scan source code for framework-specific patterns and produce transformation
/// instructions.
pub fn match_patterns(source: &str, framework: &FrameworkDetection) -> Vec<PatternMatch> {
    let source_type = SourceType::tsx();
    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, source, source_type);
    let parse_result = parser.parse();

    if parse_result.panicked {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let program = &parse_result.program;

    match framework.name.as_str() {
        "react" | "next" => {
            scan_react_patterns(&program.body, source, &mut matches);
        }
        "express" => {
            scan_express_patterns(&program.body, source, &mut matches);
        }
        _ => {}
    }

    // Next.js includes React patterns too
    if framework.name == "next" {
        // Already scanned React patterns above via the combined match arm
    }

    debug!(
        framework = %framework.name,
        patterns = matches.len(),
        "pattern matching complete"
    );

    matches
}

/// Scan for React hook patterns.
fn scan_react_patterns(
    stmts: &oxc_allocator::Vec<'_, Statement<'_>>,
    source: &str,
    matches: &mut Vec<PatternMatch>,
) {
    for stmt in stmts.iter() {
        if let Statement::VariableDeclaration(var_decl) = stmt {
            for declarator in &var_decl.declarations {
                if let Some(init) = &declarator.init {
                    check_react_hook(init, source, matches);
                }
            }
        }
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            check_react_hook(&expr_stmt.expression, source, matches);
        }
    }
}

/// Check if an expression is a React hook call and emit the appropriate pattern match.
fn check_react_hook(expr: &Expression<'_>, source: &str, matches: &mut Vec<PatternMatch>) {
    if let Expression::CallExpression(call) = expr {
        if let Expression::Identifier(ident) = &call.callee {
            let name = ident.name.as_str();
            let line = line_number_at_offset(source, call.span.start);

            match name {
                "useState" => {
                    matches.push(PatternMatch {
                        source_pattern: "useState".into(),
                        forge_equivalent: "$signal".into(),
                        line,
                        instruction: TransformInstruction::RenameCall {
                            from: "useState".into(),
                            to: "$signal".into(),
                        },
                    });
                }
                "useEffect" => {
                    matches.push(PatternMatch {
                        source_pattern: "useEffect".into(),
                        forge_equivalent: "$effect".into(),
                        line,
                        instruction: TransformInstruction::RenameCall {
                            from: "useEffect".into(),
                            to: "$effect".into(),
                        },
                    });
                }
                "useMemo" => {
                    matches.push(PatternMatch {
                        source_pattern: "useMemo".into(),
                        forge_equivalent: "$derived".into(),
                        line,
                        instruction: TransformInstruction::RenameCall {
                            from: "useMemo".into(),
                            to: "$derived".into(),
                        },
                    });
                }
                "useCallback" => {
                    matches.push(PatternMatch {
                        source_pattern: "useCallback".into(),
                        forge_equivalent: "fn".into(),
                        line,
                        instruction: TransformInstruction::UnwrapCall {
                            wrapper: "useCallback".into(),
                        },
                    });
                }
                "useRef" => {
                    matches.push(PatternMatch {
                        source_pattern: "useRef".into(),
                        forge_equivalent: "let ref = init".into(),
                        line,
                        instruction: TransformInstruction::ReplaceWithLet {
                            hook: "useRef".into(),
                        },
                    });
                }
                _ => {}
            }
        }
    }
}

/// Scan for Express route/handler patterns.
fn scan_express_patterns(
    stmts: &oxc_allocator::Vec<'_, Statement<'_>>,
    source: &str,
    matches: &mut Vec<PatternMatch>,
) {
    for stmt in stmts.iter() {
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            check_express_route(&expr_stmt.expression, source, matches);
        }
    }
}

/// Check if an expression is an Express route registration: `app.get('/path', handler)`.
fn check_express_route(expr: &Expression<'_>, source: &str, matches: &mut Vec<PatternMatch>) {
    if let Expression::CallExpression(call) = expr {
        if let Expression::StaticMemberExpression(member) = &call.callee {
            let method = member.property.name.as_str();
            let is_route_method = matches!(method, "get" | "post" | "put" | "delete" | "patch");

            if is_route_method {
                // First argument should be the route path string
                if let Some(Argument::StringLiteral(path_lit)) = call.arguments.first() {
                    let line = line_number_at_offset(source, call.span.start);
                    matches.push(PatternMatch {
                        source_pattern: format!("app.{method}('{}')", path_lit.value),
                        forge_equivalent: "server async function".into(),
                        line,
                        instruction: TransformInstruction::ExpressRouteToServerFunction {
                            method: method.to_string(),
                            path: path_lit.value.to_string(),
                        },
                    });
                }
            }
        }
    }
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
