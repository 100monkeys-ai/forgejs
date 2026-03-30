# Forge

**Forged, not assembled.**

[![CI](https://github.com/100monkeys-ai/forgejs/actions/workflows/ci.yml/badge.svg)](https://github.com/100monkeys-ai/forgejs/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

Forge is a Rust-powered, opinionated full-stack JavaScript framework that breaks cleanly from the Node.js era. It replaces the "assemble 50 npm packages and hope they work together" model with a unified compiler, runtime, and standard library — all first-party, all Rust-powered, all designed to work together.

Forge makes one bet: that a framework with strong opinions, compile-time enforcement, and a purpose-built runtime can deliver a dramatically better developer experience than the current ecosystem of assembled parts.

---

## Features

- **Rust-powered compiler** — Built on [Oxc](https://oxc.rs/), the fastest JavaScript parser available. Transform, type-strip, bundle, and emit in a single compiler pass.
- **TC39 Signals for reactivity** — No virtual DOM, no diffing. Reactivity is a compile-time language primitive wired directly to the DOM. No reconciler, no hydration mismatch.
- **Compile-time client/server boundary enforcement** — Boundary violations are compile errors, not runtime surprises. Importing a server module from a client component fails the build.
- **Self-contained server binary** — `forge build` produces one file with zero runtime dependencies. The V8 runtime, compiled routes, static assets, and all dependencies are embedded.
- **Multiple deployment targets from one codebase** — Server binary, edge (Cloudflare Workers), static site, desktop (Tauri), and mobile — from a single `forge build` with a target flag.
- **The Foundry** — A purpose-built package registry that fixes npm's structural problems: cryptographic author identity, exact versions by default, content-addressed storage.

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

|  | Next.js | Forge |
|---|---|---|
| **Compiler** | Webpack/Turbopack (JS) | Rust (Oxc + Rolldown) |
| **Reactivity** | React VDOM + hooks | TC39 Signals, no VDOM |
| **Boundary enforcement** | Convention (`"use server"`) | Compile-time error |
| **Packages** | npm (name squatting, ranges) | Foundry (cryptographic identity, exact versions) |
| **Deployment** | Server (Node.js) | Binary, edge, static, desktop, mobile |
| **Auth** | next-auth (third-party) | forge:auth (first-party) |
| **Database** | Third-party (Prisma, Drizzle) | forge:data (first-party) |

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

## Links

- **Documentation**: https://forgejs.com/docs
- **Specification**: [spec/](spec/)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **License**: [AGPL-3.0](LICENSE)

---

## License

AGPL-3.0-only © 2026 100monkeys AI, Inc.
