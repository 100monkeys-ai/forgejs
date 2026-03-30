# ADR-009: The Foundry Package Registry (Over npm)

**Number**: 009
**Title**: The Foundry Package Registry (Over npm)
**Date**: 2026-03-30
**Status**: Implemented
**Deciders**: Jeshua (Architect)
**Tags**: `#packages` `#foundry` `#registry` `#security` `#architecture`

---

## Context

### npm's Original Design and Its Misapplication

npm was created in 2010 to solve a specific, concrete problem: sharing JavaScript browser assets between developers who were building front-end tooling for browsers. It was well-designed for that problem. The registry model (central server, flat namespace, published JSON manifests) worked for distributing browser assets to build pipelines.

In 2010, Isaac Schlueter built npm to complement Node.js, and the JavaScript community adopted it for server-side development without questioning whether the browser asset distribution model was appropriate for server-side dependency management. It was not. The misapplication of npm's design to server-side software has produced structural problems that worsen with each passing year.

### npm's Structural Problems

**Problem 1: The flat namespace and name squatting.**

npm packages are identified by a single name in a flat namespace. Any developer can register `react`, `lodash`, `colors`, `is-odd`, or any other name, and that name becomes permanently associated with that developer's account. The first registrant owns the name.

This creates a structural security vulnerability: malicious actors register typosquats of popular packages (`lodash` → `lodahs`, `react` → `reakt`), common patterns (`node-colors`, `colors-node`), and historically popular names that have been transferred or abandoned. npm removes actively malicious packages but cannot prevent the registration of packages that appear benign.

The 2022 `colors` incident illustrates a different dimension of this problem. Marak Squires, the sole maintainer of `colors` (downloaded 20 million times per week at the time), deliberately published a version that printed gibberish to the terminal as a protest against unpaid open-source labor. Because `colors` had many packages depending on it via semver ranges (`^1.x`), and because npm does not enforce that new versions within a range are non-breaking, millions of projects' builds broke the day the sabotaged version was published.

The 2016 `left-pad` incident is the more famous case: the package was unpublished (not sabotaged), and the npm registry, in an attempt to prevent name squatting conflicts, honored the unpublish. The result: thousands of builds worldwide broke because they depended on an 11-line package for left-padding strings.

These are not edge cases. They are direct consequences of the flat namespace with first-registrant-wins ownership and semver range resolution.

**Problem 2: Non-deterministic installs.**

The semver range specifier (`^1.2.3`, `~1.2.3`, `>=1.0.0`) is npm's mechanism for expressing "compatible version." It has good intentions — allow minor and patch updates to land automatically — and catastrophic consequences in practice.

`^1.2.3` resolves to "the newest version with major version 1 that is ≥ 1.2.3." `npm install` on Monday and `npm install` on Friday may resolve different versions of the same dependency if a new compatible version was published between the two runs. This is why lockfiles exist. But lockfiles are optional, frequently not committed, and frequently out of date. The `npm ci` command was introduced specifically to enforce lockfile-based installs, but `npm install` without lockfile enforcement remains the norm for many projects.

The consequence is a class of bugs that are difficult to reproduce: a production deploy installs a different version of a transitive dependency than the developer had locally, producing a behavior difference that appears intermittently and is hard to trace.

**Problem 3: node\_modules bloat.**

npm stores each package's dependencies in a `node_modules` directory within the project. When package A and package B both depend on `lodash@4.17.21`, npm may store two copies of lodash in the `node_modules` tree. A moderate Next.js project has a `node_modules` directory exceeding 500MB. Across multiple projects on the same machine, the same packages are duplicated many times.

pnpm introduced a global content-addressed store with symlinks to eliminate duplication. Yarn introduced a global cache. These are improvements, but they are workarounds for a design problem rather than solutions to it.

**Problem 4: Published compiled artifacts, separate types.**

npm packages conventionally publish compiled JavaScript (not TypeScript source) plus TypeScript declaration files (`.d.ts`) generated from the source. The TypeScript source itself is often not published — or if it is, it is not the primary artifact.

This creates the `@types/` namespace: because packages were not designed to include TypeScript types, the DefinitelyTyped project maintains a separate repository of type declarations for thousands of packages. `@types/react`, `@types/node`, `@types/lodash` — these are maintained by volunteers, may lag the actual package's API, may have inaccuracies, and require a separate install step.

The `@types/` model is a symptom of a design error: types should not be decoupled from implementation.

**Problem 5: No API change enforcement.**

npm's semver contract (breaking changes require a major version bump) is enforced by convention only. There is no mechanism in npm that prevents a package from publishing a breaking API change as a patch version. The conventions are followed by responsible maintainers and ignored by accidental or malicious ones.

When a transitive dependency publishes an accidental breaking change as a patch version, consumers do not learn about it until their builds break — hours or days after the change was published, depending on their CI cadence.

### JSR as an Improvement

Deno's JavaScript Registry (JSR) represents a genuine improvement over npm on several dimensions:

- Packages are identified by `@author/name` (author-scoped namespaces prevent most squatting)
- Packages can publish TypeScript source (no `@types/` separate package needed)
- ESM-only (no CommonJS compatibility concerns)
- Compatibility score computed automatically

JSR still has limitations for Forge's use case:

- Packages still publish their source and let consumers compile it, but there is no enforce ment that the TypeScript source is the canonical artifact
- Version resolution still uses semver ranges rather than exact content addresses
- No API change enforcement — breaking changes still rely on maintainer discipline

JSR is the right direction; Forge's Foundry goes further in several dimensions.

## Decision

Build the Foundry as Forge's purpose-built package registry, designed with correct architecture from the start rather than inheriting npm's browser-asset-distribution design decisions.

### Foundry's Design

**Author/name scoped namespaces with cryptographic identity.**

Packages are identified by `author/name` where `author` is a namespace controlled by a cryptographic signing key. There is no `colors` that any first registrant can claim — there is `sindresorhus/colors`, which only the holder of Sindre's signing key can publish.

Typosquatting is structurally much harder: `sindreorhus/colors` (typo) is a different author namespace controlled by a different key. A consumer who depends on `sindresorhus/colors` and accidentally types `sindreorhus/colors` gets a dependency on a different, unrelated namespace — which is an error, not a silent substitution of malicious code.

**BLAKE3 content addressing with exact version pinning.**

Every published version of a package is a BLAKE3 hash of its contents. The Forge lockfile records exact content hashes, not semver ranges. `forge install` is deterministic: the same lockfile produces the same installed packages on every machine, every time, with cryptographic verification.

Semver ranges are supported for `forge add` (adding a new dependency) and for developer-readable manifests, but they are resolved to exact content hashes at add time and the hash is what is recorded in the lockfile. There is no semver resolution on `forge install` — the hash is the version.

This eliminates the `npm ci` vs `npm install` confusion: `forge install` always installs exactly what the lockfile specifies, by content hash, with verification.

**TypeScript source as the primary artifact.**

Foundry packages publish TypeScript source. The Forge compiler compiles the package's source as part of the consumer's build. There is no `@types/` separate package — the types are in the source. There is no "compiled JS + declarations" split — the source is the truth.

This means API evolution is visible in the TypeScript source. When a maintainer makes a breaking change, the change is in the TypeScript source, detectable by the compiler.

**Global content-addressed cache, no per-project node\_modules.**

Downloaded packages are stored in `~/.forge/cache/` keyed by content hash. Multiple projects that depend on the same package at the same version share one copy. There is no `node_modules` directory. There is no 500MB `node_modules` per project.

The Forge compiler resolves imports from the global cache. The cache is append-only (content hashes are immutable; new versions are new hashes). Old versions can be garbage-collected when no project's lockfile references them.

**API change enforcement via `forge publish`.**

When a maintainer runs `forge publish`, the Forge CLI computes an API diff between the new version and the previous published version. If the diff contains breaking changes (removed exports, changed function signatures, changed type constraints) and the new version does not bump the major version number, `forge publish` aborts with an error listing the breaking changes.

API stability is enforced at publish time, not at convention time. A maintainer cannot accidentally publish a breaking change as a patch version — the tool prevents it.

## Consequences

### Positive

- ✅ **No name squatting**: the author-scoped namespace with cryptographic identity makes squatting structurally much harder. Claiming `sindresorhus/colors` requires Sindre's signing key.
- ✅ **Deterministic installs by construction**: content hash lockfiles mean `forge install` is identical on every machine, without requiring `forge ci` vs `forge install` discipline.
- ✅ **No `@types/` split**: TypeScript source is the artifact. Types are always up to date with the implementation because they are the same file.
- ✅ **No node\_modules bloat**: global content-addressed cache. Fifty projects with a common dependency store one copy.
- ✅ **API breaking changes are enforced**: `forge publish` prevents accidental semver violations. The community can trust that `^1.2.3` in a Foundry package means what semver says it means, because the tooling enforces it.
- ✅ **Supply chain integrity**: every installed package is verified against its content hash. Tampering with a package in transit is detectable.

### Negative

- ❌ **Not compatible with the npm/Foundry ecosystem**: the two largest package ecosystems in the world are npm and npm. Forge's Foundry starts with zero packages. This is a significant adoption barrier. The FSL covers the most common framework use cases (ADR-010), but the long tail of community packages takes time to develop.
- ❌ **Requires Foundry registry infrastructure**: npm's registry is operated by npm, Inc. (now GitHub). The Foundry requires 100monkeys to operate registry infrastructure. This is an ongoing operational cost.
- ❌ **TypeScript-source-only packages require the Forge compiler to compile dependencies**: for large dependency trees, this increases build time compared to consuming pre-compiled JavaScript. The Forge compiler is fast enough that this is acceptable, but it is a tradeoff.
- ❌ **Author namespaces require key management**: developers must create and maintain a Foundry signing key. This is more friction than creating an npm account. For developers accustomed to npm's simplicity, this is a genuine UX cost.

### Neutral

- ℹ️ The Foundry is not a Forge-only registry. Any JavaScript or TypeScript package can be published to the Foundry. But the Foundry's design is optimized for the Forge ecosystem — TypeScript source, WinterTC-compatible packages, correct semver enforcement.
- ℹ️ npm packages can be mirrored to the Foundry with a wrapper that converts the package to the Foundry format. This provides a compatibility bridge for the transition period.

## Alternatives Considered

### npm

Keep using npm. The ecosystem is already there.

The problems described in this ADR's context section are npm's structural problems. They are not fixable by a wrapper or a configuration. Name squatting requires a different identity model. Non-deterministic installs require content addressing. The `@types/` problem requires publishing TypeScript source. These are architectural changes that are incompatible with npm backwards compatibility.

Rejected: npm's structural problems are the motivation for Forge's entire approach. Using npm would undermine Forge's architectural integrity.

### JSR (Deno's JavaScript Registry)

JSR is the most viable alternative to Foundry. It has author-scoped namespaces, supports TypeScript source, is ESM-only, and has a growing ecosystem.

The reasons Forge builds the Foundry rather than using JSR:

1. JSR uses semver ranges for version resolution, not content hash pinning. Deterministic installs require lockfile discipline, the same problem as npm (improved, but not solved).
2. JSR does not enforce API change enforcement at publish time.
3. JSR's author namespace is based on a GitHub/Google account, not a cryptographic signing key. Account compromise at GitHub could allow a malicious actor to publish to a JSR author namespace.
4. The Foundry is an integral part of Forge's ecosystem — the compiler understands Foundry packages deeply (WinterTC compliance metadata, FSL package annotations). A third-party registry cannot provide this integration level.

JSR is a genuine improvement over npm and is used by Forge for interoperability. The Foundry is the primary package source for Forge-native packages.

### pnpm + npm

Use npm's registry but pnpm as the package manager for better disk efficiency and stricter dependency resolution.

pnpm significantly improves npm's disk efficiency with its global content store and symlink model. It also provides stricter hoisting rules that prevent accidental dependency access.

pnpm does not address: name squatting (still npm's flat namespace), `@types/` split (still npm's published-JS model), API change enforcement (still convention-only), or supply chain integrity (no content hash verification by default).

Rejected: significant improvement over vanilla npm, but does not address the structural problems.

## Implementation Notes

The Foundry server is implemented in `crates/foundry-server` as a standalone Rust binary. It exposes an HTTP API for package publishing, resolution, and download.

The Foundry client is implemented in `packages/foundry-client` as a TypeScript CLI tool that handles:

- `forge add <author/name>` — resolves the latest version, records the content hash in the lockfile
- `forge install` — installs all packages in the lockfile by content hash, with verification
- `forge publish` — computes the API diff, enforces semver compliance, signs and publishes to the Foundry

The global cache at `~/.forge/cache/` is structured as:

```text
~/.forge/cache/
├── packages/
│   └── <author>/
│       └── <name>/
│           └── <blake3-hash>/
│               ├── package.json
│               └── src/
│                   └── ...
└── index/
    └── <author>/
        └── <name>.json  # version → hash mapping
```

## Related Decisions

- [ADR-005: Monorepo Structure](./005-monorepo-structure.md) — the FSL packages that are distributed via the Foundry
- [ADR-010: Opinionated FSL, No Plugin System](./010-opinionated-fsl-no-plugins.md) — the first-party packages that ship with Forge via the Foundry
