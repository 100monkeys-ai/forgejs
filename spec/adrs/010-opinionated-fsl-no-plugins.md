# ADR-010: Opinionated Forge Standard Library — No Plugin System

**Number**: 010
**Title**: Opinionated Forge Standard Library — No Plugin System
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#fsl` `#architecture` `#opinions` `#extensibility` `#plugins`

---

## Context

### How Plugin Systems Become the Framework

Every framework that succeeds eventually faces the same pressure: "can you add X?" The framework author cannot add every X. The plugin system is the answer — expose hooks, and let the community add what the framework does not.

This seems like the right answer. It scales the framework's capability beyond what the core team can build. It gives the community ownership and investment. It reduces the framework's maintenance burden.

The reality is different. Plugin systems, at scale, become the framework. Webpack is no longer meaningfully describable as "a bundler" — it is the sum of its plugin ecosystem. The core bundler behavior is almost irrelevant; the question is which plugins are installed and how they are configured. The framework's design principles disappear into "whatever the installed plugins do."

The consequences:

**Configuration explosion.** Each plugin adds its own configuration surface. A production Webpack configuration commonly has 200+ lines. The semantics are: "the combination of these plugins, configured this way, produces the output I want." No single document describes this — it is the emergent result of plugins interacting.

**Version conflict fragility.** Plugin A requires webpack@4. Plugin B requires webpack@5. Plugin A's peer dependency on plugin-helper@^1.0.0 conflicts with plugin C's peer dependency on plugin-helper@^2.0.0. These conflicts are routine. They are resolved by trial and error, by reading GitHub issues from three years ago, and by downgrading packages until the conflict resolves.

**The "it works on my machine" class of bugs.** The installed plugin versions on the developer's machine differ from the versions in CI, which differ from the versions in production. Each combination is a different program with potentially different behavior.

**Plugin abandonment.** The plugin ecosystem is only as healthy as the volunteer maintainers who wrote each plugin. When a plugin author loses interest, the plugin may become incompatible with a webpack upgrade, a Node.js upgrade, or a dependency upgrade — and the framework's users are stuck until someone else picks it up.

**No coherent design.** A plugin-based framework has no design beyond the design of its plugin API. The framework is defined by its extension points, not by its opinions. "Use this plugin for X" is not the same as "Forge does X this way, and here is why." The former produces configuration; the latter produces understanding.

### What Opinionated Frameworks Accomplish

Rails is the canonical example. DHH made opinionated decisions on authentication, routing, ORM, migrations, email, background jobs, asset management, testing, and deployment. The choices are sometimes controversial. The decisions are occasionally wrong in specific situations. But the frame is coherent:

> Here is a Rails application. It has these patterns. Follow them and you will be productive. Deviate from them and you are on your own.

The coherence of Rails produces a community that shares knowledge effectively. A Rails answer on Stack Overflow from 2015 is still useful in 2026 because Rails 7 is recognizably the same framework as Rails 3. The patterns are durable because the opinions are maintained.

Compare: a Next.js answer from 2021 about data fetching (`getStaticProps`) may be actively wrong in 2024, because Next.js App Router changed the recommended pattern entirely. The framework's "opinions" were reversed in a major version. The community's accumulated knowledge is partially invalidated.

Forge's thesis: an opinionated framework where the opinions are made for good reasons and maintained with conviction produces better outcomes than a plugin-based framework where each project assembles its own configuration.

### What the 80% Case Actually Is

Any web application needs:

1. **Routing** — mapping URLs to handlers, nested layouts, parameterized paths
2. **Data** — schema definition, database migrations, querying, type-safe results
3. **Authentication** — session management, user login, OAuth integration, passkeys
4. **Testing** — test runner, assertions, mocking of external dependencies
5. **Email** — transactional email (password resets, confirmations, notifications)
6. **Background Jobs** — asynchronous task execution, scheduled jobs, retry logic
7. **File Storage** — user-uploaded files, generated assets, static assets
8. **Real-time** — WebSocket channels, live updates, presence

Every web application needs these. Not some web applications — every web application eventually needs all of them. The JavaScript ecosystem's answer: choose from competing options for each. The Rails answer: here is the standard way, use it.

Forge's answer: here is the FSL. It covers these eight categories. Use it.

## Decision

The Forge Standard Library (FSL) covers the 80% case with first-party, opinionated implementations. The Forge compiler has no plugin API for core transforms. There is a defined escape hatch for the 20%.

### The FSL Packages

All FSL packages are first-party, maintained by the Forge team, published under AGPL-3.0, distributed via the Foundry, and versioned atomically with the Forge compiler and runtime.

**`forge:router`** — Routing, layouts, middleware, request handling.

File-system-based routing (routes defined by file structure) with explicit override capability. Nested layouts. Type-safe parameters. Middleware composition. Route guards. The router is the only way to define HTTP endpoints in a Forge application — there is no `app.get('/path', handler)` escape hatch at the framework level.

The opinion: file-system routing with TypeScript files is more readable and maintainable than imperative router configuration. The file structure is the documentation.

**`forge:data`** — Schema, migrations, queries, type-safe results.

Schema is defined in TypeScript using a Forge-specific DSL that compiles to SQL migrations. Queries are written in TypeScript against the schema's types. No ORM magic — queries compile to exact SQL that the developer can inspect. Type safety is end-to-end: the schema defines the types, the query builder constrains its API to valid schema operations, and the result type is inferred.

The opinion: ORMs that hide SQL are bad for production applications because developers do not understand the queries being executed. `forge:data` exposes exactly the SQL being run while providing TypeScript type safety.

**`forge:auth`** — Session management, OAuth, passkeys, email/password.

Authentication is a solved problem. Every framework reinvents it badly. `forge:auth` provides: session-based authentication (HTTP cookies, server-side sessions), OAuth 2.0 (Google, GitHub, and any OIDC provider), passkeys (WebAuthn), and email/password with PBKDF2 hashing. Multi-factor authentication support.

The opinion: authentication belongs in the standard library. "Choose your own auth library" produces applications where authentication is the least-tested, most-varied component. `forge:auth` is tested by the Forge team as part of the core.

**`forge:test`** — Test runner, assertions, mocking.

A test runner integrated with the Forge compiler. Tests run in the same V8 runtime as the application, with full access to the Forge op system. Type-aware mocking (mock a `forge:data` query, mock a `forge:auth` session). Assertions have error messages that include type information.

The opinion: the test runner should understand the application. Testing a server function's interaction with `forge:data` should not require configuring a mock database separately — the test runner should understand `forge:data`'s query model.

**`forge:email`** — Transactional email, template rendering.

Send email from server functions. Templates are Forge components rendered to HTML. The FSL integrates with SMTP and with email providers (Postmark, Resend) via WinterTC `fetch`. Preview emails in development.

**`forge:jobs`** — Background jobs, scheduled tasks, retry logic.

Define jobs as typed server functions. Schedule them from other server functions or via cron. The Forge runtime manages job execution, retry (with exponential backoff), and observability (job duration, success/failure rates). Jobs run in the same V8 runtime as request handlers.

**`forge:storage`** — File storage, CDN integration.

Upload files, store them in object storage (S3-compatible), serve them through the Forge server or via CDN. Image processing (resize, format conversion) is a first-party operation. The storage abstraction is provider-neutral: the same code works with local storage in development and object storage in production.

**`forge:realtime`** — WebSocket channels, live queries, presence.

Typed WebSocket channels with server-side rooms. Live queries (a query whose results update in real-time when the underlying data changes). Presence (track which users are connected and what they are doing). Integration with `forge:auth` for authenticated WebSocket connections.

### The Compiler's Role

The compiler has no plugin API for core transforms. The compilation pipeline (ADR-001) is a closed system: Oxc parse → Forge analyze → Forge transform → Oxc codegen → Rolldown bundle.

Why no plugin API:

1. **Plugins cannot maintain compile-time guarantees.** The client/server boundary enforcement (ADR-007) requires the compiler to understand every transformation applied to the code. A third-party plugin that transforms imports, wraps function calls, or modifies the module graph could violate the compiler's assumptions and produce incorrect boundary analysis. The only safe model is: the compiler understands all transformations.

2. **Plugins undermine performance.** The Forge compiler's performance comes from a coherent pipeline with no coordination overhead between stages. A plugin API introduces external code into the pipeline, with external allocations, external error handling, and external interface requirements. Each plugin is a performance regression.

3. **Plugins become the framework over time.** The history of plugin-based compilers (Babel, Webpack, rollup) demonstrates that the plugin ecosystem becomes the primary surface of the framework. Forge does not want to be defined by its plugin ecosystem.

### The Escape Hatch

The 20% case — things the FSL does not cover — is handled at the server function level, not the compiler level.

A `server async function` is the escape hatch. It is an ordinary async function that runs in the Forge server runtime. It has access to:

- All WinterTC APIs (fetch, crypto, streams, etc.)
- Any Foundry package that is WinterTC-compatible
- Any Rust native op registered in the Forge server's op set

A server function can call any external HTTP API via `fetch`. It can use any database driver available as a Foundry package (as long as the driver uses WinterTC APIs, not Node.js APIs). It can be extended by native Forge plugins (Rust crates that register new deno_core ops).

The escape hatch is at the right level of abstraction: below the FSL (which provides opinions about how to structure common operations) and above the compiler (which must remain a closed system). The escape hatch does not compromise the compile-time guarantees — server functions are still subject to boundary enforcement. It provides the flexibility needed for use cases the FSL does not cover.

## Consequences

### Positive

- ✅ **No "which auth library should I use" decision**: the answer is `forge:auth`. The Forge team has made the decision, tested it, and will maintain it. New projects start with working authentication.
- ✅ **Consistent patterns across all Forge projects**: every Forge application uses `forge:router` for routing, `forge:data` for database access, `forge:auth` for authentication. An engineer joining a Forge project from another Forge project immediately recognizes the structure.
- ✅ **FSL maintained by the same team as the compiler**: when the compiler changes, the FSL changes with it. There is no "the auth library hasn't updated to support the new compiler version yet."
- ✅ **The compiler can reason about FSL usage**: because the compiler knows what `forge:data` is, it can generate optimized code for queries (batching, prefetching, type inference). Plugins cannot receive this treatment.
- ✅ **Reduced decision fatigue for new projects**: start a Forge project and the architecture decisions that consume the first week of a typical JavaScript project (auth library, ORM, routing library, test framework) are already made.

### Negative

- ❌ **Less flexibility than a plugin-based system**: if your use case requires something the FSL does not cover and the escape hatch is insufficient, Forge may not be the right framework. This is an intentional tradeoff.
- ❌ **FSL opinions may not match everyone's preferences**: DHH's opinions about web development are not universally correct. Forge's opinions about data access, routing, and authentication will not match every project's requirements. Developers who strongly prefer a different approach (Prisma, Drizzle, or another ORM for data; NextAuth or Auth.js for authentication) will find Forge's FSL opinionated in the wrong direction for their use case.
- ❌ **The FSL is a significant engineering commitment**: maintaining eight first-party libraries with the quality and compatibility guarantees of the FSL requires sustained engineering investment. This is the cost of the opinionated model.
- ❌ **No Babel plugin compatibility**: developers with existing Babel transforms (codemods, custom syntax extensions, performance instrumentation) cannot apply them to Forge code. The compiler is not Babel-compatible. This is a migration cost for codebases that rely on Babel plugins.

### Neutral

- ℹ️ The FSL packages are AGPL-3.0 like Forge itself. Using the FSL in a commercial application does not require open-sourcing the application (AGPL applies to modifications of the FSL, not to applications built with it).
- ℹ️ The FSL packages are the primary packages in the Foundry registry. They set the quality standard for Foundry packages.

## Alternatives Considered

### Plugin API for Core Transforms

Provide a stable plugin API for the Forge compiler's transformation pipeline. Allow community plugins to add transforms, modify the import graph, or extend the JSX analysis.

This is what Babel and Webpack do. The consequences are described in this ADR's context section. The core problem: plugins cannot maintain the compile-time guarantees that make Forge different from existing tools. Boundary enforcement requires the compiler to understand all transformations. A plugin that transforms imports could violate this.

Rejected: fundamentally incompatible with compile-time guarantee enforcement.

### Minimal Core, Ecosystem Libraries

Provide only the compiler and runtime. Leave routing, data, auth, and all other common concerns to the ecosystem. This is the Express/Koa model for Node.js — a minimal framework with a rich ecosystem.

The minimal core approach works when the ecosystem develops coherent solutions. The Node.js ecosystem has not — there are good libraries for routing (Express, Koa, Fastify, Hono) but no single answer, good libraries for ORMs (Prisma, Drizzle, TypeORM, Sequelize) but no consensus, and a historically fragmented auth landscape (Passport.js, NextAuth, Auth.js, Lucia). The decision overhead is real.

More importantly, the minimal core approach cannot provide the compiler-level integration that makes Forge's FSL unique. `forge:data` is not just a query builder — it is a query builder that the compiler understands, enabling optimizations and type inference that are impossible for a third-party library. Decoupling the FSL from the compiler would eliminate Forge's most technically interesting capability.

Rejected: the ecosystem approach produces the decision fatigue and inconsistency that Forge is designed to eliminate.

### Separate FSL Packages with Independent Versioning

Provide the FSL as independently versioned packages with their own release cadences. `forge:auth@2.0.0` is compatible with `forge-compiler@1.x` and `forge-compiler@2.x`.

Independent versioning is the standard npm model and it produces the version conflict problems described in ADR-009. The FSL packages are deeply coupled to the compiler — they use compiler-understood annotations, they depend on compiler-enforced patterns, they receive compiler-generated optimizations. Independent versioning would create a matrix of compatibility combinations, with the attendant version conflict issues.

Rejected: the atomic versioning of the monorepo (ADR-005) is the correct model for components this tightly coupled.

## Implementation Notes

FSL package source lives in `packages/fsl/` within the monorepo.

Each FSL package has a `forge.json` manifest that declares its Foundry metadata, its compiler integration points (annotations understood by the compiler), and its op dependencies (WinterTC ops or Forge-specific ops it requires).

The compiler's FSL awareness is in `crates/forge-compiler/src/analyze/fsl.rs`. This module understands the import paths `forge:router`, `forge:data`, `forge:auth`, etc., and applies FSL-specific analysis (query type inference, auth-aware boundary analysis, router-specific type checking).

## Related Decisions

- [ADR-001: Rust-Powered Compiler Pipeline](./001-rust-powered-compiler.md) — the closed compiler pipeline that has no plugin API
- [ADR-009: Foundry Over npm](./009-foundry-over-npm.md) — the registry where FSL packages are distributed
- [ADR-007: Compile-Time Boundary Enforcement](./007-compile-time-boundary-enforcement.md) — the compile-time guarantees that require a closed compiler
