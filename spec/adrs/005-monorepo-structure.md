# ADR-005: Monorepo Structure with Cargo Workspace

**Number**: 005
**Title**: Monorepo Structure with Cargo Workspace
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#repository` `#structure` `#cargo` `#workspace` `#governance`

---

## Context

Forge consists of multiple distinct components that must be developed, tested, and released together but have different technical characters:

- `forge-compiler` — Rust crate, the compilation pipeline
- `forge-runtime` — Rust crate, the embedded V8/deno_core server runtime
- `forge-cli` — Rust binary, the developer-facing command-line tool
- `forge-server` — Rust binary, the production HTTP server
- `foundry-server` — Rust binary, the Foundry package registry server
- `foundry-client` — TypeScript package, the Foundry CLI client
- `fsl` — TypeScript packages, the Forge Standard Library (`forge:auth`, `forge:data`, etc.)
- `spec` — This directory. Specifications and ADRs.

The question is how to organize these components across one or more git repositories.

The 100monkeys platform itself uses a multi-repository structure — each bounded context is an independent repository (aegis-orchestrator, aegis-cortex, aegis-temporal-worker, etc.). This approach makes sense for a mature platform where the bounded contexts have stable interfaces and independent deployment cadences. The AEGIS components are independently deployable services with well-defined API contracts between them.

Forge is different in three ways:

1. **Early-stage development**: The API boundaries between components are still being established. The interface between `forge-compiler` and `forge-runtime`, between the FSL packages and the runtime op system, between the CLI and the compiler — these will change frequently during initial development. Cross-repository refactors require coordinated PRs across multiple repositories, which is high ceremony for changes that should be fast iterations.

2. **Deep coupling by design**: The FSL packages are not independently consumed libraries — they are FSL *because* the compiler knows about them and generates optimized code for them. `forge:data`'s query builder works the way it does because the compiler understands it and can enforce its type invariants. This coupling means `fsl` and `forge-compiler` need to evolve together, which is awkward in separate repositories.

3. **Single release cadence**: Forge users install Forge and get a version. Not "forge-compiler v1.2.3 + forge-runtime v1.4.1 + fsl v2.0.0 + forge-cli v1.1.7." The version is atomic. The monorepo structure mirrors this release model — a single git tag produces a coherent release of all components.

Cargo workspaces are Rust's native monorepo mechanism. A workspace defines multiple crates that share a single `Cargo.lock`, enabling coordinated versioning and cross-crate dependencies within the same build graph.

## Decision

Single monorepo at `https://github.com/100monkeys-ai/forgejs` with a Cargo workspace for Rust crates.

Directory structure:

```text
forgejs/
├── Cargo.toml          # workspace root
├── Cargo.lock          # shared lockfile, checked in
├── crates/
│   ├── forge-compiler/ # Rust: compiler pipeline
│   ├── forge-runtime/  # Rust: deno_core-based server runtime
│   ├── forge-cli/      # Rust: CLI binary (forge dev, forge build, forge check)
│   └── forge-server/   # Rust: production HTTP server binary
├── packages/
│   ├── foundry-client/ # TypeScript: Foundry CLI client
│   └── fsl/            # TypeScript: Forge Standard Library packages
│       ├── forge-auth/
│       ├── forge-data/
│       ├── forge-router/
│       └── ...
├── spec/               # This directory: specifications and ADRs
├── examples/           # Example Forge applications
└── .github/
    └── workflows/      # CI/CD
```

The TypeScript packages in `packages/` use a `package.json` workspace (npm/pnpm workspaces) nested within the Cargo workspace. This is the standard hybrid monorepo structure used by projects like Turborepo and NX.

## Consequences

### Positive

- ✅ **Atomic versioning**: a single `git tag v1.0.0` produces a coherent set of components. The release process is one workflow, not seven coordinated workflows across seven repositories.
- ✅ **Cross-component refactors in a single PR**: when the compiler's type representation changes and the boundary analysis pass, the FSL type stubs, and the test fixtures all need to update together, one PR captures the entire change. No coordinated PRs across repositories, no "waiting for the compiler repo's CI before I can update the runtime repo."
- ✅ **Shared CI/CD**: one `.github/workflows/` directory governs the build, test, lint, and release pipeline for all components. No drift between component CI configurations.
- ✅ **Cargo workspace tooling**: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace` — all components built and tested with one command. The shared `Cargo.lock` ensures all crates use the same transitive dependency versions.
- ✅ **Simple contributor experience**: clone one repository, run one command. No "first clone the compiler repo, then the runtime repo, then link them with this script."

### Negative

- ❌ **Larger repository surface area**: the repository will grow as all components accumulate history. This is manageable for git but becomes inconvenient for developers who only work on one component and must clone everything to get what they need.
- ❌ **Cannot independently version FSL packages**: the FSL packages share the repository's version. A breaking change to `forge:data` cannot be released as `forge:data@2.0.0` while the rest of the framework stays at `v1.x`. Everything moves together. This is acceptable during early development but constrains the ecosystem as the FSL stabilizes.
- ❌ **CI times grow with the repository**: as more components are added, full-workspace CI runs take longer. Mitigation: path-based CI filtering to only run relevant jobs when specific components change.
- ❌ **Merging worktrees requires coordination**: under the AEGIS git worktree policy (CLAUDE.md), feature branches live in `/home/theaxiom/100monkeys/worktrees/`. With a single repository, multiple features in flight simultaneously share the same base. This is standard git practice but requires coordination that would be isolated in separate repositories.

### Neutral

- ℹ️ The monorepo structure is explicitly a development-stage convenience. The `spec/` directory (this document's home) will be consistent across both the monorepo and any future split.
- ℹ️ GitHub Actions supports path-based job filtering. The CI configuration should filter jobs by component path to avoid running the full build on documentation-only changes.

## When to Split

The monorepo is appropriate until the components have stable, independently-consumable APIs. The splits, when they happen, should follow this sequence:

1. **FSL packages** → split to `forgejs-fsl` when the FSL API surface is stable and independent consumers (community packages, third-party integrations) exist that want to depend on FSL without the full Forge toolchain
2. **Foundry server** → split to `forgejs-foundry` when the Foundry registry has independent operators who need to deploy the registry without deploying the framework
3. **CLI** → potentially stays in the main repository indefinitely, as it is the user-facing surface and tightly coupled to the compiler/runtime versions

Do not split proactively. Split in response to a concrete need created by independent consumers or operators. Early splitting adds coordination overhead without corresponding benefit.

## Alternatives Considered

### Multi-Repository (AEGIS Platform Style)

The AEGIS platform uses separate repositories for each bounded context. This is the right architecture for AEGIS because:

- AEGIS's bounded contexts are independently deployable services
- They have stable gRPC/HTTP API contracts between them
- They have independent release cadences
- They have (or will have) separate teams

None of these conditions hold for Forge's components at this stage. The components are not independently deployable (the compiler and runtime must match versions). They do not have stable internal API contracts yet. They release together. They are maintained by the same team.

Applying the AEGIS multi-repo structure to Forge would be organizational cargo-culting — copying a structure that works for one context without evaluating whether it fits the new context.

Rejected for the current development stage. Revisit when the conditions above change.

### Nx or Turborepo

Nx and Turborepo are monorepo management tools for JavaScript/TypeScript projects. They provide caching, affected-file change detection, and remote caching for build pipelines.

Forge's monorepo contains a Cargo workspace (Rust) with TypeScript packages nested inside. Nx and Turborepo are optimized for JavaScript/TypeScript monorepos. Applying them to a Cargo workspace requires configuration gymnastics that adds complexity without proportional benefit.

The Cargo workspace tooling (`cargo build --workspace`, `cargo test --workspace`) is purpose-built for Rust monorepos and is excellent. For the TypeScript packages, pnpm workspaces provide the equivalent. Combining pnpm workspaces for TypeScript with Cargo workspaces for Rust is the standard pattern for hybrid Rust+TypeScript monorepos (Tauri does this, for example).

Rejected: unnecessary complexity for a team of the current size.

## Implementation Notes

The root `Cargo.toml` declares the workspace:

```toml
[workspace]
members = [
    "crates/forge-compiler",
    "crates/forge-runtime",
    "crates/forge-cli",
    "crates/forge-server",
    "crates/foundry-server",
]
resolver = "2"

[workspace.dependencies]
# All third-party dependencies declared here with pinned versions
# Individual crates reference workspace dependencies to avoid version drift
```

The TypeScript packages use a root `package.json` with `pnpm` workspaces:

```json
{
  "name": "forgejs-packages",
  "private": true,
  "workspaces": [
    "packages/foundry-client",
    "packages/fsl/*"
  ]
}
```

## Related Decisions

- [ADR-009: Foundry Over npm](./009-foundry-over-npm.md) — the package registry these FSL packages publish to
- [ADR-010: Opinionated FSL, No Plugin System](./010-opinionated-fsl-no-plugins.md) — the FSL packages in `packages/fsl/`
