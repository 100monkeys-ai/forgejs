# Introduction

Forge is a full-stack JavaScript framework with a Rust-native compiler and runtime. It is designed for teams that want the productivity of a unified frontend/backend codebase without the sprawl of assembling a framework from twenty independent packages, each with their own release cadence, configuration format, and breaking change philosophy.

## The Problem

The Node.js ecosystem solved the wrong problem. It gave frontend engineers server-side JavaScript, but it handed them the same fragmented toolchain — bundlers, transpilers, ORMs, auth libraries, test runners — that already existed for every other language, rebuilt with npm. The result is a "full-stack" experience where a new project requires decisions about a dozen foundational tools before writing a single line of product code.

Worse, the boundary between client and server has become a convention rather than a contract. In most JavaScript frameworks, running server code on the client — or leaking secrets to the browser — is a runtime error at best, a silent data exposure at worst. The tooling cannot enforce what the architecture requires.

## Philosophy: Forged, Not Assembled

Forge is vertically integrated. The compiler, runtime, router, data layer, auth system, test runner, job queue, and package registry are built together, tested together, and released together. There are no "community" adapters for core functionality. This is not a framework; it is a platform with a narrow scope and a fixed opinion about what belongs inside it.

The `.fx` file extension is not cosmetic. It is a signal to the compiler that this file participates in Forge's module system — one where `"use module server"` and `"use module client"` are not hints but enforced boundaries. The compiler rejects server imports in client bundles at build time, not at runtime.

## Key Features

| Feature | Description |
| --- | --- |
| **Rust-powered compiler** | The `forge` CLI is a single Rust binary. Build times are measured in milliseconds. |
| **TC39 Signals** | Reactivity is powered by the TC39 Signals proposal — no virtual DOM, no reconciler, no dependency on React. |
| **Compile-time boundaries** | `"use module server"` and `"use module client"` are enforced at build time. Boundary violations are compiler errors. |
| **Self-contained output** | `forge build` produces a single binary (server target) or a static directory (static target) with no external runtime dependencies. |
| **The Foundry** | A curated package registry for `.fx` packages. All packages are signed and verified. |
| **Forge Standard Library** | Eight first-party packages cover routing, data, auth, testing, email, jobs, storage, and real-time — no assembly required. |

## Forge vs. Next.js

| | Forge | Next.js |
| --- | --- | --- |
| Language | TypeScript (.fx) | TypeScript / JavaScript |
| Compiler | Rust (native binary) | SWC + Node.js |
| Reactivity | TC39 Signals | React (VDOM) |
| Client/server boundary | Compile-time enforced | Runtime convention |
| Data layer | `forge:data` (built-in) | Choose your own ORM |
| Auth | `forge:auth` (built-in) | NextAuth or roll your own |
| Test runner | `forge:test` (built-in) | Jest / Vitest |
| Output | Self-contained binary | Node.js process |
| Package registry | The Foundry (curated) | npm (uncurated) |
