//! Source parsing: reads `.fx` and `.ts` files and produces typed ASTs.
//!
//! ## The Parser's Role
//!
//! The parser is the entry point to the compiler pipeline. It is responsible
//! for reading source files and producing a structured representation (AST)
//! that downstream passes can analyze and transform.
//!
//! ## Oxc Foundation
//!
//! Forge uses [Oxc](https://oxc.rs) as its parsing foundation. Oxc provides:
//!
//! - The fastest JavaScript/TypeScript parser available (faster than SWC)
//! - Arena-allocated AST nodes for zero-copy operations across pipeline passes
//! - A unified pipeline: the same AST is used for parsing, semantic analysis,
//!   transformation, and code generation — no redundant re-parsing
//!
//! See ADR-002 for the full rationale for choosing Oxc over alternatives.
//!
//! ## .fx File Extensions
//!
//! `.fx` files are valid TypeScript with three additional constructs:
//!
//! 1. `export component Foo(...)` — a component declaration (sugar for a
//!    function returning JSX, with compile-time signal wiring)
//! 2. `server async function foo(...)` — a server function that the compiler
//!    strips from client bundles and replaces with a typed RPC stub
//! 3. `$signal`, `$derived`, `$async`, `$effect` — reactive primitive sugar
//!    that desugars to TC39 Signal API calls
//!
//! The parser handles these extensions in [`fx_syntax`] by post-processing
//! the Oxc AST after initial parsing.
//!
//! See spec/specs/001-fx-file-format.md for the full grammar.

pub mod fx_syntax;
pub mod forge_toml;
