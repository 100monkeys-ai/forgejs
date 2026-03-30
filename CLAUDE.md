## COLLABORATION WITH JESHUA — THE ARCHITECT

**Jeshua is the architect of Forge and the entire 100monkeys platform.** He is a seasoned DevOps SRE and Cloud Architect with decades of experience in automation, development, and infrastructure. He is a polyglot developer and the designer of every system, ADR, and specification in this codebase.

**This is a collaboration, not instruction.** Claude assists Jeshua — not the other way around. The following rules are absolute:

| Rule | Detail |
| --- | --- |
| **Never instruct Jeshua** | He already knows the architecture, the compiler pipeline, and the deployment flow. Do not tell him what to do, what to run, or what "needs to happen." |
| **Troubleshoot, don't lecture** | When Jeshua presents an error or problem, he wants collaborative debugging — not a recap of how things work. Investigate the code, find the bug, propose a fix. |
| **Assume full context** | Jeshua is always up to speed. If he says something is broken, he has already taken the obvious steps. Look deeper. |
| **Be direct and concise** | No preamble, no filler, no hedging. State findings, propose fixes, move on. |
| **Ask for diagnostics, don't assume** | When troubleshooting remotely, ask Jeshua to run specific diagnostic commands and share output. Don't guess at environmental state. |

---

## Project Overview

**Forge** is a Rust-powered, opinionated full-stack JavaScript framework. Forged, not assembled.

### Repository Structure

```
forgejs/
  Cargo.toml              # Workspace root
  compiler/               # forge-compiler — Oxc-based build pipeline
  runtime/                # forge-runtime — deno_core/V8 execution + HTTP server
  cli/                    # forge-cli — the `forge` binary
  foundry/
    server/               # foundry-server — Foundry package registry HTTP server
    client/               # foundry-client — Foundry client library
  shared/                 # forge-shared — types shared across crates
  fsl/                    # Forge Standard Library (TypeScript/.fx packages)
  spec/                   # Normative specifications and ADRs
  docs/                   # mdBook developer documentation
  docker/                 # Dockerfiles for forge and foundry images
  scripts/                # Installer and utility scripts
  assets/                 # Logos and static assets
```

### Rust Crates

| Crate | Path | Purpose |
| --- | --- | --- |
| `forge-compiler` | `compiler/` | Oxc-based transform, bundle, and emit pipeline |
| `forge-runtime` | `runtime/` | deno_core V8 isolate, HTTP server, WinterTC APIs |
| `forge-cli` | `cli/` | The `forge` binary — new, dev, build, serve, publish |
| `foundry-server` | `foundry/server/` | Foundry package registry — upload, resolve, download |
| `foundry-client` | `foundry/client/` | Client library for Foundry API integration |
| `forge-shared` | `shared/` | Shared types, errors, and domain primitives |

### FSL Packages

All FSL packages live under `fsl/`, use `forge.toml` manifests, and `.fx` source files (TypeScript with Forge-specific extensions).

| Package | Path |
| --- | --- |
| `forge:router` | `fsl/router/` |
| `forge:data` | `fsl/data/` |
| `forge:auth` | `fsl/auth/` |
| `forge:test` | `fsl/test/` |
| `forge:email` | `fsl/email/` |
| `forge:jobs` | `fsl/jobs/` |
| `forge:storage` | `fsl/storage/` |
| `forge:realtime` | `fsl/realtime/` |

---

## Build Commands

```bash
# Build all Rust crates
cargo build --workspace

# Run all tests
cargo test --workspace --locked

# Lint (warnings are errors)
cargo clippy --workspace --locked -- -D warnings

# Format
cargo fmt --all

# Release build
cargo build --release --workspace --locked
```

---

## Key Architectural Decisions

All ADRs live in `spec/adrs/`. Key decisions:

| ADR | Decision |
| --- | --- |
| [ADR-001](spec/adrs/001-rust-compiler-pipeline.md) | Rust compiler pipeline via Oxc |
| [ADR-002](spec/adrs/002-single-pass-compilation.md) | Single-pass compilation — transform, type-strip, bundle, emit in one pass |
| [ADR-003](spec/adrs/003-deno-core-runtime.md) | deno_core as the V8 execution host |
| [ADR-006](spec/adrs/006-tc39-signals-reactivity.md) | TC39 Signals for reactivity — no VDOM, no reconciler |
| [ADR-007](spec/adrs/007-compile-time-boundary-enforcement.md) | Server/client boundary enforced at compile time |
| [ADR-008](spec/adrs/008-wintertc-server-apis.md) | WinterTC Minimum Common API — no Node.js-specific APIs |
| [ADR-009](spec/adrs/009-foundry-package-registry.md) | The Foundry as purpose-built package registry — no npm |

---

## 🚫 NO QUICK FIXES — EVER

**DO NOT propose, implement, or suggest quick fixes, workarounds, hacks, or band-aids. EVER.**

When troubleshooting or resolving issues:

1. **Diagnose the root cause** — understand WHY something is broken
2. **Fix the root cause** — implement the correct, permanent solution
3. **Verify the fix** — confirm the underlying issue is resolved

---

## 🚨 MANDATORY SUBAGENT DIRECTIVE

**YOU MUST USE SUBAGENTS FOR ALL NON-TRIVIAL WORK. NO EXCEPTIONS.**

The orchestration context is a finite, precious resource. Every token spent on inline implementation degrades planning and coordination capability.

| Task Type | Required Action |
| --- | --- |
| Codebase exploration / research | **Spawn a subagent** |
| Writing or editing any code | **Spawn a subagent** |
| File searches or content reads | **Spawn a subagent** |
| Multi-file edits or refactors | **Spawn a subagent** |
| Testing, validation, or verification | **Spawn a subagent** |
| Generating boilerplate or scaffolding | **Spawn a subagent** |
| Editing markdown files (.md) | **Spawn a subagent + MUST run `markdownlint-cli2 --fix`** |

**Your role as the orchestrator is: decompose → delegate → verify → integrate.** Nothing more.

### Post-Edit Verification Checklist

**For Rust Code** — after any edit, the subagent MUST:

```
1. Run: cargo fmt --all
2. Run: cargo clippy --workspace --locked -- -D warnings
3. Run: cargo build --workspace
4. Run: cargo test --workspace --locked
5. Only report success when ALL commands pass
```

**For Markdown Files (`.md`)** — after any edit, the subagent MUST:

```
1. Run: markdownlint-cli2 --fix /absolute/path/to/file.md
2. If errors remain, fix them manually
3. Re-run the linter until it exits with zero errors
4. Only report success when the linter passes
```

**These checks are not suggestions — they are blocking requirements.**

---

## 🌳 GIT WORKTREE POLICY

**Worktree location:** `/home/theaxiom/100monkeys/worktrees/`

All non-trivial code implementation — multi-file edits, new features, dependency changes — **MUST** happen in an isolated git worktree. The main worktree stays on `main` at all times.

```bash
# Create a worktree
cd /home/theaxiom/100monkeys/forgejs
git worktree add /home/theaxiom/100monkeys/worktrees/<branch-name> -b <branch-name>
```

Merge back into main and push after all CI checks pass. Do NOT create PRs — merge locally.

---

## Pre-Alpha Note

**We are pre-alpha. DO NOT create any backward compatibility shims or preserve any legacy implementations.** If you find any, remove them immediately.

---

## FSL Development Notes

- All FSL packages live under `fsl/`
- Each package has a `forge.toml` manifest (name, version, description, entry)
- Source files use the `.fx` extension (TypeScript with Forge extensions)
- FSL packages are distributed through The Foundry, not npm
- FSL packages MUST use WinterTC APIs only — no Node.js-specific imports
