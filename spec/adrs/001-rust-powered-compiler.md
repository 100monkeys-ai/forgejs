# ADR-001: Rust-Powered Compiler Pipeline

**Number**: 001
**Title**: Rust-Powered Compiler Pipeline
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#compiler` `#tooling` `#performance` `#architecture`

---

## Context

JavaScript's build toolchain is the primary source of ecosystem fragility, not JavaScript itself.

Webpack today has over 4,000 community plugins. Each plugin exposes its own configuration API. Each plugin has its own version requirements that may conflict with the version requirements of adjacent plugins. Each plugin has its own failure modes — and because plugins execute at build time in a shared process, a broken plugin produces broken output without the pipeline necessarily knowing it produced broken output. Configuration is a negotiation between the plugin author's mental model and the developer's mental model, mediated by an `options` object that both parties have different understandings of.

This is the configuration hell that defines the JavaScript build experience. It is not intrinsic to the problem of compiling JavaScript. It is a consequence of designing the compiler as a plugin pipeline where each plugin is an independent package maintained by independent parties with independent versioning.

The human cost is real. A senior engineer setting up a new project in 2026 typically spends two to four hours configuring the build pipeline before writing the first line of application code. That build pipeline — webpack config, Babel config, TypeScript config, whatever ESLint configuration the team has standardized on, whatever CSS solution, whatever path alias setup — must then be maintained indefinitely. When a plugin releases a breaking version, someone spends a day debugging it. When a transitive dependency has a security vulnerability, the entire plugin ecosystem must update before the project is safe.

Meanwhile, the Rust-based alternatives have proven what the alternative looks like. SWC performs TypeScript transformation 17 times faster than Babel — the same transformation, producing equivalent output, in 1/17th of the time. This is not a marginal improvement. It is a demonstration that the bottleneck was never algorithmic — it was that Babel is written in JavaScript and JavaScript is slow for this kind of CPU-intensive tree transformation work. `@tailwindcss/oxide`, the Rust rewrite of Tailwind's CSS engine, runs 100 times faster than its JavaScript predecessor. Biome, the Rust rewrite of ESLint + Prettier, runs 50-100 times faster. The ecosystem is voting with its feet: when a Rust rewrite ships, it wins on performance by an order of magnitude, and the JavaScript version gradually loses market share.

The conclusion is not subtle. JavaScript build tools written in JavaScript are slow because they are doing CPU-intensive work in a runtime optimized for I/O. Rust is a better runtime for CPU-intensive work. The right answer for a new compiler is Rust, not "JavaScript that spawns Rust workers."

Beyond performance, a Rust pipeline enables something plugin-based systems cannot provide: compile-time invariant enforcement. When the compiler is a single coherent program rather than a negotiation between plugins, it can understand the full program at once. It can enforce that a server import does not appear in a client module. It can enforce that a non-serializable type is not passed across the client/server boundary. It can enforce that a DB query never executes in a client component. These invariants are impossible to enforce in a plugin-based system because no individual plugin has a complete view of the program.

## Decision

Own the entire compiler pipeline in Rust. Use Oxc as the parser and transformer foundation. Build the analysis and code generation layers as Rust crates within the Forge compiler. Ship a single binary. No plugin API for core transforms.

The pipeline is:

1. **Oxc parse** — source files → typed AST, arena-allocated
2. **Forge analyze** — type-aware boundary analysis, import graph construction, invariant checking
3. **Forge transform** — JSX → signal-wired DOM calls, server function extraction, boundary enforcement
4. **Oxc codegen** — transformed AST → JavaScript source
5. **Rolldown bundle** — module graph → deployment artifact(s)

Each stage is a pure transformation. No side effects. No plugin hooks between stages.

## Consequences

### Positive

- ✅ **Performance**: single-digit millisecond incremental builds for moderate projects. Full builds in under a second for projects with hundreds of modules. Not an aspiration — a consequence of Rust + arena allocation + no plugin overhead.
- ✅ **Compile-time invariant enforcement**: because the compiler understands the full program, violations of the client/server boundary are compile errors before any JavaScript is emitted. This is categorically impossible with a plugin pipeline.
- ✅ **Single binary, zero configuration**: `forge build` requires no configuration file for the compiler pipeline. The compiler knows Forge's semantics and applies them.
- ✅ **Deterministic output**: the same source produces the same output on every machine, every run. No plugin version drift.
- ✅ **Unified error model**: compiler errors reference Forge language semantics, not plugin configuration mismatches.

### Negative

- ❌ **Not extensible via plugins**: intentional by design (see ADR-010), but this will be a genuine objection from developers who have built workflows around Babel plugins or Webpack loaders. The correct response is to evaluate whether those workflows represent real requirements or accidental complexity.
- ❌ **Requires Rust expertise to contribute to the compiler**: the barrier to contributing to the parser, transformer, or code generator is higher than for a JavaScript-based tool. This is an acceptable tradeoff — the compiler is the core of the framework, not a place for casual experimentation.
- ❌ **Longer feedback loop for compiler bugs**: when the compiler is wrong, fixing it requires a Rust build cycle. This is faster than it sounds (incremental Rust compilation is fast), but it is different from the "edit a JavaScript file and re-run" loop some developers are accustomed to.

### Neutral

- ℹ️ The compiler binary is distributed as a platform-specific release artifact. `forge install` downloads the correct binary for the host platform. No Rust toolchain required for end users.
- ℹ️ The compiler exposes a stable JSON API for IDE integration, language server protocol (LSP), and other tooling. The internal pipeline is Rust; the external surface is JSON.

## Alternatives Considered

### esbuild

esbuild is written in Go, not Rust, but it proves the same thesis: a compiled language compiler for JavaScript is dramatically faster than a JavaScript compiler for JavaScript. esbuild is the right choice if you need a fast bundler and nothing else.

The reasons Forge does not build on esbuild:

1. **No AST plugin API**: esbuild deliberately does not expose an AST-level plugin API. You can transform text before parsing or after codegen, but not the AST. Forge requires AST analysis for boundary enforcement — you cannot determine whether an import is a server import without understanding the module graph at the AST level.
2. **Go vs Rust memory model**: Forge is a Rust-first project. Mixing Go and Rust in a single binary is possible but complex. A Rust-native compiler integrates cleanly with the rest of the Forge codebase.
3. **Limited TypeScript support**: esbuild strips TypeScript types but does not perform type checking. Forge's boundary enforcement is type-aware.

### SWC

SWC is the most obvious alternative — it is Rust, it is fast, and it has an AST plugin system. SWC is used by Deno and by Next.js as its compiler.

The reasons Forge does not build on SWC:

1. **The plugin system is the problem**: SWC's Wasm-based plugin system exists, but plugin authors report significant complexity in using it. More importantly, Forge does not want a plugin system for core transforms. Building on SWC and then not using its plugin system means carrying its complexity without using it.
2. **SWC is a transformer, not a full pipeline**: SWC handles parse → transform → codegen. It does not handle module bundling, dependency resolution, or the analysis passes Forge requires for boundary enforcement. Using SWC means also integrating a separate bundler, which reintroduces the coordination problem.
3. **Oxc is faster and more purpose-built**: Oxc was designed from the start as a unified pipeline (lint + transform + bundle). SWC was designed as a Babel replacement. For Forge's full-pipeline needs, Oxc is the better foundation.

### Webpack

Webpack represents the world Forge is trying to escape. 4,000+ plugins. Configuration files measured in hundreds of lines. Version conflicts as a routine occurrence. Build times measured in tens of seconds for moderate projects.

Not considered seriously. The entire motivation for this ADR is to not be Webpack.

### Vite

Vite is the best of the current JavaScript-based build tools. It uses esbuild for development (fast) and Rollup for production (correct but slower). It has a well-designed plugin API that is significantly less chaotic than Webpack's.

Vite is not appropriate as Forge's compiler because:

1. It is JavaScript running JavaScript — the performance ceiling is lower than Rust by an order of magnitude
2. Its plugin API, while good, still means Forge's semantics are implemented as Vite plugins, not as a coherent compiler
3. Vite will eventually migrate to Rolldown (VoidZero's Rust bundler), at which point Vite's architecture will look more like Forge's — but Forge can use Rolldown directly without the Vite abstraction layer

## Implementation Notes

The compiler lives in the `forge-compiler` crate within the Forge monorepo.

The pipeline stages correspond to modules within `forge-compiler`:

- `crates/forge-compiler/src/parse.rs` — Oxc integration, arena setup
- `crates/forge-compiler/src/analyze.rs` — import graph, boundary analysis, type flow
- `crates/forge-compiler/src/transform.rs` — JSX transform, server function extraction
- `crates/forge-compiler/src/codegen.rs` — Oxc codegen integration
- `crates/forge-compiler/src/bundle.rs` — Rolldown integration, artifact emission

The compiler is invoked by the Forge CLI (`forge build`, `forge dev`) and by the LSP server for IDE integration.

## Related Decisions

- [ADR-002: Oxc Parser Foundation](./002-oxc-parser-foundation.md) — why Oxc specifically
- [ADR-010: Opinionated FSL, No Plugin System](./010-opinionated-fsl-no-plugins.md) — why there is no plugin API
