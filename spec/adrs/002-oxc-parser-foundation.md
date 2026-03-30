# ADR-002: Oxc Parser Foundation

**Number**: 002
**Title**: Oxc Parser Foundation
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#compiler` `#parser` `#oxc` `#rust`

---

## Context

A JavaScript/TypeScript compiler requires a parser as its entry point. The parser takes source text and produces a typed abstract syntax tree (AST) that subsequent analysis and transformation passes can operate on. The quality of the parser directly determines the quality of everything built on top of it: how accurately the compiler understands the program, how fast the compilation pipeline runs, and how detailed the error messages can be.

Writing a production-quality JavaScript parser from scratch is a multi-year effort. JavaScript's grammar has accumulated substantial complexity over thirty years: ASI (automatic semicolon insertion), legacy octal literals, class field syntax, optional chaining, nullish coalescing, logical assignment, private class members, top-level await, import assertions, decorators, and TypeScript's entire type system layered on top. A new parser that handles all of this correctly, with good error recovery and performant memory allocation, is not a weekend project.

The available options in 2026:

**Babel** (JavaScript): The reference implementation. Parses everything correctly. Used by millions of projects. Extremely slow — it is a JavaScript program doing CPU-intensive recursive descent parsing. A moderate project can take Babel 30-60 seconds to transform. The AST format is the de facto standard but it was designed for JavaScript-era ergonomics, not for performance.

**SWC** (Rust): A Rust-based parser and transformer designed as a Babel replacement. Fast. Battle-tested — Deno and Next.js both use it in production. The AST uses Rust's owned heap allocations, which means transformation passes pay allocation costs on every node they create or modify.

**Oxc** (Rust): Built by the VoidZero team (creators of Vite, Rolldown, and Vue's new toolchain). Uses a bump allocator (arena allocation) — all AST nodes are allocated from a single memory arena, making allocation O(1) and deallocation free (the entire arena is dropped at once). Designed from the start as a unified pipeline: the same parser feeds the linter (Oxlint), the transformer, the type checker (in progress), and the bundler (Rolldown). The fastest available JavaScript parser by a significant margin in all published benchmarks.

**Tree-sitter**: A parsing framework designed for incremental parsing in editor contexts. Excellent for syntax highlighting and code navigation in IDEs. Not designed for compiler use — it parses to a concrete syntax tree (CST) rather than an abstract syntax tree (AST), and it does not perform the type-aware analysis that a compiler requires.

**Custom parser**: Building Forge's own parser would provide maximum control over the AST format and extension points. It would take approximately two to three years to reach production quality and would spend those years chasing ECMAScript spec updates and TypeScript syntax additions instead of building the framework.

The choice is between SWC and Oxc. Both are Rust, both are fast, both handle TypeScript. The distinction that matters for Forge is the memory model and the pipeline architecture.

## Decision

Use Oxc as the parser, semantic transformer, and code generation foundation for the Forge compiler.

Specifically:

- `oxc_parser` for parsing JavaScript and TypeScript source to an arena-allocated AST
- `oxc_semantic` for scope analysis, symbol resolution, and reference tracking
- `oxc_transformer` for standard ECMAScript transforms (decorators, class properties, etc.)
- `oxc_codegen` for AST → source text emission with source maps
- `oxc_span` for source location tracking throughout the pipeline

Forge's analysis and transform passes operate directly on the Oxc AST, taking a mutable reference to the arena and producing a transformed AST that Oxc codegen then emits.

## Consequences

### Positive

- ✅ **Fastest available parser**: Oxc parses JavaScript significantly faster than SWC and orders of magnitude faster than Babel. For a full-project build, this is the difference between sub-second and multi-second compile times.
- ✅ **Arena allocation means zero-copy operations**: analysis passes can traverse the AST without allocating. Transformation passes allocate into the same arena as the parser, so node creation is as fast as bumping a pointer. The entire AST and all transforms are freed by dropping the arena at the end of compilation — no GC pressure, no reference counting overhead.
- ✅ **Unified pipeline**: the same AST nodes used for parsing are used for linting (Oxlint), transformation, and code generation. There is no translation layer between pipeline stages. Forge analysis passes work on the same types that Oxlint rules work on — the Oxc ecosystem's knowledge is directly accessible.
- ✅ **Actively maintained by the right team**: the VoidZero team built Vite and Rolldown. They understand the full-pipeline problem. Oxc is their solution to the parser layer. When Rolldown needs a parser feature, Oxc gets it. When Forge needs a parser feature, the team behind Oxc has the right background to evaluate the request.
- ✅ **Rolldown integration is natural**: Rolldown is Forge's bundler (see ADR-001). Rolldown also uses Oxc internally. The module graph that Rolldown builds and the AST that Oxc produces are designed to work together. Forge's bundling layer integrates without impedance mismatch.

### Negative

- ❌ **Oxc is pre-1.0**: the API surface can change between minor versions. Forge pins to a specific Oxc version in `Cargo.lock` and updates deliberately, but upstream breaking changes require Forge-side migration work. This is a real ongoing cost, not a theoretical one.
- ❌ **AST format is Oxc-specific**: the Oxc AST format is not the Babel AST format (the de facto standard). Any tooling written for the Babel AST (codemod tools, certain analysis libraries) does not work directly on Forge's AST representation. This matters if Forge ever needs to interoperate with the broader JavaScript tooling ecosystem at the AST level.
- ❌ **TypeScript type checker is not yet complete**: Oxc's type checker (`oxc_type_checker`) is in active development but not yet feature-complete as of this writing. Forge's boundary enforcement requires type-aware analysis. Until Oxc's type checker is ready, Forge uses a conservative approximation for cross-boundary type checking, with a fallback to `tsc --noEmit` for type validation during `forge check`. This is a known limitation to be resolved when Oxc's type checker matures.

### Neutral

- ℹ️ Oxc is MIT licensed, compatible with Forge's AGPL-3.0 license (AGPL can incorporate MIT dependencies).
- ℹ️ The arena allocator means AST node lifetimes are tied to the arena's lifetime. Forge's transformation passes must be careful about retaining references to AST nodes across arena resets — in practice, each file is parsed and compiled in its own arena, so this is not a practical constraint.

## Alternatives Considered

### SWC

SWC is production-proven (Deno, Next.js), Rust-based, and handles the full TypeScript grammar. It is the most obvious alternative.

The reason Forge does not build on SWC comes down to the memory model. SWC uses standard Rust heap allocations for AST nodes. This means:

1. Every transformation that creates a new AST node incurs a heap allocation
2. The allocator must track each node individually for deallocation
3. Transformation passes that restructure the AST — like Forge's JSX transform, which rewires component trees into signal-bound DOM calls — produce significant allocation pressure

Oxc's arena allocator eliminates this overhead entirely. For Forge's transformation-heavy pipeline (every JSX expression, every server function, every reactive binding is a transformation), this difference compounds across a full project build into a meaningfully faster compile time.

The secondary consideration is the ecosystem alignment. VoidZero (Oxc's maintainer) is building Rolldown, which is Forge's bundler. Using Oxc means using the parser that Rolldown was designed to work with. Using SWC would mean bridging between SWC's AST and Rolldown's internal representation — an impedance mismatch at the most performance-sensitive layer.

### Babel

Considered and rejected immediately. Babel is a JavaScript program. Running a JavaScript program to parse JavaScript is running JavaScript twice. The performance ceiling is an order of magnitude below what Rust achieves. The only reason to use Babel in 2026 is compatibility with the enormous ecosystem of Babel plugins — and Forge does not have a plugin ecosystem (see ADR-010), so that compatibility provides no benefit.

### Tree-sitter

Tree-sitter is an excellent tool for its intended domain: incremental syntax parsing for editor integrations. It produces a CST (concrete syntax tree) that retains every token including whitespace, which is necessary for syntax highlighting but irrelevant for compilation. The incremental parsing model is designed for "user edited one character" not "compile the full module graph." Not appropriate as a compiler foundation.

### Custom parser

Maximum control, maximum cost. Writing a production JavaScript/TypeScript parser from scratch would take years, delay every other part of Forge's development, and still be catching up to ECMAScript and TypeScript evolution for years afterward. Oxc's parser is already correct for all current ECMAScript and TypeScript syntax. The implementation effort is better spent on the parts of Forge that differentiate it — boundary analysis, signal-based reactivity, the Foundry package registry, the FSL — not on reimplementing a solved problem.

## Implementation Notes

Oxc is a workspace dependency declared in the root `Cargo.toml`. Individual Oxc crates are referenced where needed:

```toml
[workspace.dependencies]
oxc = { version = "x.y.z", features = ["full"] }
oxc_parser = { version = "x.y.z" }
oxc_semantic = { version = "x.y.z" }
oxc_transformer = { version = "x.y.z" }
oxc_codegen = { version = "x.y.z" }
oxc_span = { version = "x.y.z" }
```

The Forge analysis passes are implemented as visitor implementations over `oxc_ast::ast` node types. The arena is owned by the parse function in `forge-compiler` and passed by reference to all downstream passes within the same compilation unit.

Oxc version updates are a deliberate, reviewed operation — not an automated dependency bump. The `Cargo.lock` file is checked in and only updated when a Forge team member has verified the new version's behavior against the Forge test suite.

## Related Decisions

- [ADR-001: Rust-Powered Compiler Pipeline](./001-rust-powered-compiler.md) — the overall compiler architecture
- [ADR-007: Compile-Time Boundary Enforcement](./007-compile-time-boundary-enforcement.md) — the analysis passes that operate on the Oxc AST
