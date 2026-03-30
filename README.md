# Forge

**Forged, not assembled.**

[![CI](https://github.com/100monkeys-ai/forgejs/actions/workflows/ci.yml/badge.svg)](https://github.com/100monkeys-ai/forgejs/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

Forge is an opinionated full-stack JavaScript framework with a Rust compiler at its core. It eliminates the configuration sprawl, runtime surprises, and leaky abstractions that have become the default in modern web development. Every Forge application is compiled — not bundled — from typed source to a self-contained deployable artifact.

Forge makes one bet: that a framework with strong opinions, compile-time enforcement, and a purpose-built runtime can deliver a dramatically better developer experience than the current ecosystem of assembled parts.

---

## Features

- **Rust compiler pipeline** — Built on [Oxc](https://oxc.rs/) for sub-second builds. Transform, type-strip, bundle, and emit in a single compiler pass.
- **TC39 Signals reactivity** — No virtual DOM. Reactivity is a first-class language primitive using the TC39 Signals proposal, compiled to efficient imperative DOM updates.
- **Compile-time boundary enforcement** — Server and client code are separated at compile time. Importing a server module from a client component is a compiler error, not a runtime failure.
- **Self-contained binary** — `forge build` produces a single binary that embeds the V8 runtime, compiled routes, static assets, and all dependencies. Ship a file, not a folder.
- **The Foundry** — A purpose-built package registry for Forge packages (`.fx` format). No npm, no `node_modules`, no dependency resolution surprises.
- **WinterTC server APIs** — Server-side code uses the [WinterTC Minimum Common Web Platform API](https://wintercg.org/) exclusively. No Node.js-specific APIs, no lock-in.
- **Forge Standard Library** — Eight production-ready FSL packages for routing, data access, authentication, testing, email, background jobs, storage, and realtime communication.

---

## Quick Start

```bash
curl -fsSL https://forgejs.com/install.sh | sh
forge new my-app
cd my-app
forge dev
```

Your app is live at `http://localhost:3000`.

---

## How Forge Differs from Next.js

| Capability | Next.js | Forge |
|---|---|---|
| **Compiler** | SWC (Rust), webpack/Turbopack | Oxc-based single-pass Rust pipeline |
| **Reactivity** | React VDOM + reconciler | TC39 Signals, no VDOM |
| **Boundary enforcement** | Convention (`"use client"` / `"use server"`) | Compile-time error — cannot import across boundary |
| **Packages** | npm / node_modules | The Foundry — purpose-built `.fx` registry |
| **Deployment artifact** | Directory of files + Node.js runtime | Single self-contained binary |
| **Server APIs** | Node.js + Web APIs (mixed) | WinterTC Minimum Common API only |
| **Configuration** | next.config.js, tsconfig.json, postcss.config.js, ... | One `forge.toml` per project |

---

## Project Structure

```
my-app/
  forge.toml          # Project manifest
  src/
    pages/            # File-system routing
      index.fx        # => /
      about.fx        # => /about
    components/       # Shared UI components
    server/           # Server-only modules (enforced at compile time)
    public/           # Static assets
```

---

## FSL Packages

The Forge Standard Library ships eight packages out of the box:

| Package | Purpose |
|---|---|
| `forge:router` | File-system routing, layouts, and navigation |
| `forge:data` | Type-safe data fetching and mutation |
| `forge:auth` | Authentication and session management |
| `forge:test` | Testing framework integrated with the compiler |
| `forge:email` | Transactional email with template support |
| `forge:jobs` | Background job queue |
| `forge:storage` | File storage abstraction |
| `forge:realtime` | WebSocket and SSE primitives |

---

## Documentation

- [Full Documentation](https://forgejs.com/docs) — Guides, API reference, and tutorials
- [Specification](spec/README.md) — Normative technical specifications
- [Architecture Decision Records](spec/adrs/) — Why Forge makes the choices it does
- [Philosophy](spec/PHILOSOPHY.md) — The principles behind Forge

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and the PR process.

---

## License

Forge is licensed under the [GNU Affero General Public License v3.0](LICENSE).

Copyright (c) 2026 100monkeys AI.
