# ADR-011: App-Level Migration via Resolved Call Graph

**Number**: 011
**Title**: App-Level Migration via Resolved Call Graph
**Date**: 2026-04-06
**Status**: Accepted
**Deciders**: Jeshua (Architect)
**Tags**: `#migration` `#adoption` `#compiler` `#tree-shaking` `#framework-patterns`

---

## Context

### Foundry's Cold-Start Problem

ADR-009 established the Foundry as Forge's purpose-built package registry and acknowledged the most significant consequence of that decision: the Foundry starts with zero packages. The FSL covers the most common framework use cases (ADR-010), but the npm ecosystem has over two million packages. No developer migrates a production application to a new framework if the migration requires rewriting every dependency from scratch.

The adoption barrier is not "Forge lacks packages." The adoption barrier is "I have a working Node.js application with 200 dependencies, and I cannot justify rewriting it." This is the same barrier that killed every previous attempt at a clean-break JavaScript runtime — the gravity of the npm ecosystem is enormous, and ignoring it is architectural hubris.

Forge must provide a credible migration path that does not require developers to manually rewrite their entire dependency tree.

### Why Package-Level npm Compatibility Is the Wrong Approach

The obvious approach — import npm packages into Forge or provide a compatibility layer — is architecturally wrong for a subtle reason.

Consider `pg`, the PostgreSQL client for Node.js. It uses `net.Socket`, `tls.connect`, `dns.resolve`, `crypto.createHash`, `fs.readFileSync` (for reading SSL certificates), `stream.Transform`, and `events.EventEmitter`. A package-level compatibility analysis concludes: "pg is incompatible with WinterTC. It uses 7 Node.js-specific APIs."

But an application that uses `pg` may only call `pool.query()` and `pool.end()`. At those two call sites, `pg` uses `net.Socket` (for the TCP connection) and `crypto.createHash` (for the SCRAM-SHA-256 authentication handshake). It does not use `fs.readFileSync` because the application does not configure SSL from a file. It does not use `dns.resolve` because the application connects by IP address. It does not use `stream.Transform` because the application uses the promise-based API, not the streaming API.

The package-level analysis says "7 incompatible APIs." The app-level analysis says "2 incompatible APIs, both shimmable." These are profoundly different conclusions, and only the app-level analysis is accurate for this application's actual migration requirements.

This asymmetry is not an edge case. It is the norm. Most applications use a small fraction of their dependencies' API surface. A package that uses `fs` in 90% of its code may be fully compatible at the 3 call sites the application actually invokes. Package-level analysis is pessimistic by construction — it reports the union of all possible incompatibilities, not the intersection of actually-used incompatibilities.

### The Key Insight: Apps Are Concrete Call Graphs

A Node.js application is, before resolution, a collection of import statements pointing at packages that point at other packages in an arbitrarily deep dependency tree. After resolution, it is a concrete call graph of actually-used functions. The vast majority of `node_modules` is dead code — functions that are exported but never called by this application, code paths that are conditionally reachable but never reached by this application's configuration and usage patterns.

Bundlers have understood this for a decade. Webpack, Rollup, and esbuild all perform tree-shaking: they trace which exports are actually imported, and they eliminate dead code. The resulting bundle contains only the code the application actually uses.

Migration analysis should work the same way. Instead of asking "is this package compatible?", ask "are the functions this application actually calls compatible?" The unit of analysis is not the package — it is the resolved, tree-shaken call graph of the application.

App-level resolution plus tree-shaking produces an accurate compatibility picture. Package-level analysis produces a pessimistic one. The pessimistic picture discourages migration that would actually succeed. The accurate picture enables migration that developers would otherwise not attempt.

## Decision

Provide `forge migrate <path>` as a CLI command that converts a full Node.js application to a Forge project. The migration operates on the resolved call graph of the application, not on individual packages.

### Resolution-First Approach

The migration proceeds in five phases:

**Phase 1: Entry Point Discovery.** Scan the application for entry points: `package.json` `main`/`module`/`exports` fields, framework-specific entry points (Next.js `pages/` and `app/` directories, Express/Fastify server files), test entry points, and CLI scripts. The entry point set defines what code is reachable.

**Phase 2: Import/Export Reachability Graph.** Starting from the discovered entry points, build the full import/export reachability graph. For each `import { foo } from 'bar'`, trace `foo` to its definition in `bar`'s source, then trace `bar`'s internal imports recursively. This graph includes the application's own source files and all files in `node_modules` that are transitively reachable from the entry points.

This is best-effort reachability analysis via import-graph and export tracking. It is NOT full inter-procedural pointer analysis. Dynamic imports (`import()` with non-literal arguments), `eval()`, `require()` with computed paths, and `Reflect.get` with dynamic property names are not traced — they appear as opaque nodes in the graph. This is an explicit limitation: the migration tool acknowledges these as "unresolvable" and flags them for manual review rather than silently ignoring them or attempting unsound analysis.

**Phase 3: Tree-Shake.** Eliminate all code that is not reachable from the entry points. The residual — the code that survives tree-shaking — is the actual migration target. This is typically a small fraction of `node_modules`.

**Phase 4: Classify the Residual.** Every function, class, and module in the residual is classified into one of three tiers:

| Tier | Description | Action |
| --- | --- | --- |
| **Compatible** | Pure JavaScript/TypeScript with no Node.js-specific API usage. Standard language features, WinterTC APIs (`fetch`, `crypto.subtle`, `URL`, `TextEncoder`, etc.). | Direct conversion to `.fx` — no changes needed beyond syntax adaptation. |
| **Shimmable** | Node.js APIs that have WinterTC or Forge Standard Library equivalents. `fs.readFile` → `forge:storage`, `http.createServer` → `forge:router`, `crypto.createHash` → `crypto.subtle`, `Buffer` → `Uint8Array`, `process.env` → `Deno.env` or compile-time env. | Automated rewrite to the equivalent API. The migration tool performs the substitution. |
| **Needs Manual Attention** | Node.js APIs with no portable equivalent. `child_process.spawn`, `net.Socket` (raw TCP), `dgram` (UDP), `cluster`, `worker_threads`, `vm.runInContext`, native addons (`.node` files), and any opaque dynamic import/eval nodes. | Flagged in the migration report with the specific API, its call site, and suggestions for alternative approaches. |

**Phase 5: Convert and Emit.** Generate a new Forge project directory containing:

- `forge.toml` — project manifest with metadata derived from the original `package.json`
- `.fx` source files — the application's source converted to Forge's file format with FSL imports replacing Node.js API usage where shimmable
- Inlined dependency code — used functions from `node_modules` are pulled into the application's `.fx` source tree as internal modules, not as external Foundry dependencies. No npm dependency survives the migration.
- Migration report — a detailed Markdown file listing every classification decision, every automated rewrite, every manual-attention item, and overall migration statistics (percentage compatible, percentage shimmable, percentage manual)

### Framework Pattern Matching

Modern Node.js applications are not raw Node.js — they use frameworks with well-defined patterns. These patterns have direct Forge equivalents, and the migration tool recognizes and converts them:

**React patterns:**

- `useState`, `useEffect`, `useMemo`, `useCallback` → TC39 Signals: `$signal`, `$effect`, `$derived` (ADR-006)
- JSX → Forge's compiled JSX (same syntax, different runtime — no VDOM reconciler)
- `React.createContext` + `useContext` → Forge's signal-based context propagation

**Express/Fastify patterns:**

- Route handlers (`app.get('/path', handler)`) → server functions + `forge:router` route definitions
- Middleware chains → Forge's compile-time middleware composition
- `req.body`, `req.params`, `req.query` → typed request destructuring in server functions

**Next.js patterns:**

- `pages/` directory routing → `forge:router` file-based routing
- `getServerSideProps` / `getStaticProps` → server functions with compile-time boundary enforcement (ADR-007)
- API routes (`pages/api/`) → server functions
- `next/image`, `next/link`, `next/head` → FSL equivalents

These framework pattern matchers are finite — the set of major Node.js frameworks is well-known and stable. The matchers do not need to handle arbitrary frameworks; they handle the frameworks that represent the vast majority of production Node.js applications.

### Dependency Inlining, Not Bridging

A critical design decision: migrated dependency code is inlined into the application's source tree, not published to the Foundry or referenced as an external package. The migration tool does not attempt to create Foundry packages from npm packages. It extracts the specific functions the application uses, converts them, and places them in the application's own source tree as internal modules.

This avoids the npm bridge anti-pattern (maintaining a compatibility layer between two package ecosystems) and avoids polluting the Foundry with mechanical conversions of npm packages that may not be correct in the general case (they are correct for this application's usage, but not necessarily for all usage patterns).

The output is local-only. The migration tool does not publish anything to the Foundry registry. The migrated application is a self-contained Forge project.

## Consequences

### Positive

- Dramatically lowers the adoption barrier. A developer with a working Node.js application can run `forge migrate ./my-app` and get a concrete, accurate assessment of what migrates automatically, what needs shimming, and what needs manual work. This is infinitely more actionable than "rewrite everything."
- Accurate compatibility analysis. By analyzing the resolved call graph rather than package manifests, the migration tool avoids the false pessimism of package-level analysis. Applications that would appear "impossible to migrate" at the package level may be 85% automatically convertible at the call-graph level.
- Framework patterns are finite and well-known. React, Express, Fastify, and Next.js cover the vast majority of production Node.js applications. The pattern matchers are a bounded problem, not an open-ended one.
- The migration report provides a clear, honest assessment. Developers can make informed decisions about whether the manual-attention items are tractable before committing to the migration.

### Negative

- Best-effort reachability may miss dynamic imports and eval. Applications that rely heavily on dynamic `require()`, `import()` with computed paths, or `eval()` will have incomplete call graphs. The migration tool flags these as opaque nodes, but it cannot guarantee that all reachable code has been discovered. This is a fundamental limitation of static analysis.
- Framework matchers need ongoing maintenance as frameworks evolve. When React introduces a new hook or Next.js changes its routing convention, the matchers must be updated. This is a bounded maintenance burden (frameworks release major versions infrequently), but it is not zero.
- Large applications may produce migration reports with significant manual-attention sections. An application that uses raw TCP sockets, UDP, native addons, or `child_process` extensively will have a long list of items that cannot be automatically migrated. The migration tool is honest about this — it does not promise zero-effort migration for applications that fundamentally depend on Node.js-specific capabilities.

### Neutral

- The migration tool does not replace the need for a Foundry ecosystem. It bootstraps migration of existing applications, but the community still needs to build native Foundry packages for new development. Migration and ecosystem growth are complementary strategies, not substitutes.
- Inlined dependency code is a snapshot. If the original npm package publishes a security fix after migration, the inlined code does not automatically receive it. The developer is responsible for the inlined code as part of their application, the same as any other first-party code.

## Alternatives Considered

### npm Package-Level Import Bridge

Provide an npm compatibility layer that allows Forge to import npm packages directly, with per-package compatibility analysis.

This is the pessimistic approach described in the Context section. A package that uses `fs` in 90% of its code is classified as "incompatible" even if the application only calls the 10% that is pure JavaScript. The compatibility analysis operates at the wrong granularity — the package — rather than the correct granularity — the application's actual call sites.

Beyond the false pessimism, an npm import bridge creates a permanent dependency on npm's ecosystem and its structural problems (ADR-009). Forge would inherit npm's name squatting, non-deterministic resolution, and supply chain vulnerabilities through the bridge. The bridge becomes a load-bearing compatibility layer that can never be removed because applications depend on it.

Rejected: pessimistic compatibility analysis discourages migration that would actually succeed; the bridge undermines Forge's architectural independence from npm.

### Runtime Node.js Compatibility Layer

Implement Node.js core APIs (`fs`, `net`, `crypto`, `child_process`, etc.) as a runtime compatibility layer in Forge, allowing npm packages to run unmodified.

This is the approach Deno initially took with its `node:` compatibility layer, and it has proven to be an enormous, ongoing maintenance burden. The Node.js API surface is vast — over 40 core modules with thousands of functions, many with subtle behavioral differences across Node.js versions. A compatibility layer that is 99% correct is still broken for applications that depend on the 1% that differs.

More fundamentally, a Node.js compatibility layer undermines the WinterTC guarantee (ADR-008). If Forge can run Node.js code, then Forge applications will use Node.js APIs, and the clean break from Node.js-specific APIs that Forge's architecture depends on will erode. The compatibility layer becomes a gravitational pull back toward the Node.js ecosystem rather than a bridge away from it.

Rejected: undermines the WinterTC guarantee, creates a perpetual maintenance burden, makes Forge a Node.js wrapper rather than a clean break.

### Manual Migration Guides Only

Provide documentation and guides for manually migrating Node.js applications to Forge, without automated tooling.

Manual migration guides are valuable and should exist regardless. But they do not scale. A developer with a 50-file Express application and 10 direct dependencies (200 transitive dependencies) will not spend two weeks manually tracing which Node.js APIs are used at which call sites and manually rewriting each one. The migration will not happen.

Adoption of a new framework is driven by the perceived cost of migration. If the perceived cost is "read a guide and rewrite your app," the answer for most teams is "no." If the perceived cost is "run a command, review a report, fix the flagged items," the answer changes. Automated tooling does not eliminate the work — it quantifies it and automates the mechanical parts, making the decision to migrate a rational cost-benefit analysis rather than an open-ended commitment.

Rejected: does not scale, will not drive real adoption for applications with non-trivial dependency trees.

## Implementation Notes

The migration tool is implemented across two crates:

- `foundry/client/src/migrate/` — the core migration module tree:
  - `entry.rs` — entry point discovery (package.json, framework conventions)
  - `graph.rs` — import/export reachability graph construction
  - `shake.rs` — tree-shaking pass over the reachability graph
  - `classify.rs` — three-tier classification of residual code
  - `convert.rs` — code conversion and `.fx` emission
  - `report.rs` — migration report generation
  - `patterns/` — framework pattern matchers:
    - `react.rs` — React hooks → TC39 Signals conversion
    - `express.rs` — Express/Fastify route → server function conversion
    - `nextjs.rs` — Next.js patterns → Forge equivalents
- `cli/src/commands/migrate.rs` — the `forge migrate <path>` CLI command, wiring the migration pipeline to clap and the terminal UI

The reachability graph construction reuses the Oxc parser (ADR-001) for parsing JavaScript and TypeScript source files in `node_modules`. The tree-shaking pass shares infrastructure with the Forge compiler's existing dead-code elimination.

## Related Decisions

- [ADR-007: Compile-Time Boundary Enforcement](./007-compile-time-boundary-enforcement.md) — server/client boundary enforcement applies to migrated code; server functions converted from Next.js `getServerSideProps` are subject to the same compile-time checks
- [ADR-008: WinterTC Server APIs](./008-wintertc-only-server-apis.md) — the WinterTC API surface defines what is "compatible" in the three-tier classification; the migration tool's shim layer maps Node.js APIs to WinterTC equivalents
- [ADR-009: Foundry Over npm](./009-foundry-over-npm.md) — the cold-start problem acknowledged in ADR-009 is the direct motivation for this migration tool; dependency inlining avoids creating an npm bridge while still enabling migration
- [ADR-010: Opinionated FSL](./010-opinionated-fsl-no-plugins.md) — FSL packages are the target for shimmable API conversions; `forge:router`, `forge:storage`, `forge:auth` replace their Node.js equivalents in migrated code
