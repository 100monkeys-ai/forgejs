# 002 — `forge.toml` Schema

**Status:** Normative
**Version:** 0.1.0-pre-alpha
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [File Location and Loading](#2-file-location-and-loading)
3. [Schema Reference](#3-schema-reference)
   - 3.1 [`[project]`](#31-project)
   - 3.2 [`[build]`](#32-build)
   - 3.3 [`[dev]`](#33-dev)
   - 3.4 [`[[target]]`](#34-target)
   - 3.5 [`[server]`](#35-server)
   - 3.6 [`[dependencies]`](#36-dependencies)
   - 3.7 [`[dev-dependencies]`](#37-dev-dependencies)
4. [Target Type Reference](#4-target-type-reference)
5. [Complete Example](#5-complete-example)
6. [Validation Rules](#6-validation-rules)
7. [Environment Variable Interpolation](#7-environment-variable-interpolation)

---

## 1. Purpose

`forge.toml` is the project manifest for a Forge application or library. It is the single authoritative source of project identity, build configuration, target definitions, and dependency declarations.

`forge.toml` is always located at the project root. Its presence is what makes a directory a Forge project. The Forge CLI searches upward from the current working directory to locate the project root.

---

## 2. File Location and Loading

**Discovery:** The CLI walks parent directories from the current working directory until it finds `forge.toml` or reaches the filesystem root. If no `forge.toml` is found, the CLI exits with an error.

**Encoding:** UTF-8 required. BOM is rejected.

**TOML version:** TOML v1.0.0.

**Schema version:** Implicitly `0.1`. Future schema versions will be signalled by a `schema_version` field in `[project]`.

---

## 3. Schema Reference

### 3.1 `[project]`

The `[project]` table defines project identity. It is **required**.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | **Yes** | — | Project name. Must match `[a-z][a-z0-9-]*`. Maximum 64 characters. |
| `version` | string | **Yes** | — | Semantic version string conforming to SemVer 2.0.0 (e.g. `"1.0.0"`, `"0.1.0-pre-alpha"`). |
| `description` | string | No | `""` | Human-readable description. Maximum 256 characters. |
| `authors` | array of string | No | `[]` | Author names/emails. No format constraint. |
| `license` | string | No | `""` | SPDX license identifier (e.g. `"MIT"`, `"AGPL-3.0-only"`). |

**Example:**

```toml
[project]
name = "my-app"
version = "0.3.1"
description = "A Forge application"
authors = ["Jeshua Maxey <jeshua@example.com>"]
license = "AGPL-3.0-only"
```

**Validation:**

- `name` must match `^[a-z][a-z0-9-]*$`.
- `version` must be a valid SemVer string. Pre-release identifiers are allowed.
- `version` must not contain build metadata (`+`) — use `version = "1.0.0"` not `"1.0.0+build.1"`.

---

### 3.2 `[build]`

The `[build]` table controls compiler behaviour. It is **optional**; all fields have defaults.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `entry` | path string | No | `"app/root.fx"` | Entry point relative to project root. Must be a `.fx` file. |
| `output` | path string | No | `".forge/dist"` | Output directory relative to project root. Created if absent. |
| `source_maps` | bool | No | `true` in dev, `false` in prod | Whether to emit `.map` files alongside compiled output. |

**Example:**

```toml
[build]
entry = "app/root.fx"
output = ".forge/dist"
source_maps = true
```

**Notes:**

- `source_maps` can be overridden at the CLI level with `--source-maps` / `--no-source-maps`.
- The `output` path is always cleaned before a production build. Incremental builds in development preserve the output directory.
- If `entry` does not exist at project load time, the CLI emits a warning (not an error) — the file may be created before the build runs.

---

### 3.3 `[dev]`

The `[dev]` table configures the development server. It is **optional**.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `port` | u16 | No | `3000` | Port for the application dev server. |
| `studio_port` | u16 | No | `3001` | Port for Forge Studio (visual inspector and signal graph). |
| `open` | bool | No | `false` | Whether to open the browser automatically on `forge dev`. |

**Example:**

```toml
[dev]
port = 4000
studio_port = 4001
open = true
```

**Notes:**

- `port` and `studio_port` must be different. If they collide, the CLI exits with an error before starting.
- Ports below 1024 require elevated privileges on most systems. The CLI warns but does not prevent use.
- `studio_port = 0` disables Forge Studio entirely.

---

### 3.4 `[[target]]`

Targets are defined as an **array of tables** (`[[target]]`). A project may define zero or more targets. If no targets are defined, `forge build` compiles for the default `server` target with default settings.

Each `[[target]]` entry represents an independent compilation output. Multiple targets with the same `type` are allowed (e.g. two `edge` targets for different providers).

#### Common Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | **Yes** | — | Unique identifier for this target within the project. Used in CLI `--target` flag. |
| `type` | string (enum) | **Yes** | — | Target type. One of: `static`, `server`, `edge`, `desktop`, `mobile`. |
| `output` | path string | No | `".forge/dist/<name>"` | Output directory for this target's artifacts. |

#### `type = "static"`

Generates a fully static HTML + JS + CSS bundle. No server runtime required. Server functions are forbidden in the component tree (compiler error if any `server` functions are reachable from the entry).

No additional fields.

```toml
[[target]]
name = "web-static"
type = "static"
output = "dist/static"
```

#### `type = "server"`

Generates a self-contained server binary (via Forge's server runtime). Server functions are compiled into the binary as HTTP handlers.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `embed_assets` | bool | No | `true` | Embed compiled client assets into the server binary. If `false`, assets are served from `output/assets/`. |

```toml
[[target]]
name = "app-server"
type = "server"
embed_assets = true
```

#### `type = "edge"`

Generates a bundle compatible with edge compute platforms. Currently only Cloudflare Workers is supported.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `provider` | string (enum) | **Yes** | — | Edge provider. Only `"cloudflare"` is supported in this version. |

```toml
[[target]]
name = "edge-cf"
type = "edge"
provider = "cloudflare"
```

**Notes:**

- Edge targets use the WinterTC API surface (spec 006). Node.js-specific APIs in server functions are a compile error for edge targets.
- The Cloudflare Workers output format is a single ESM bundle compatible with the `wrangler` deploy tool.

#### `type = "desktop"`

Generates a desktop application wrapper using Tauri.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `shell` | string (enum) | **Yes** | — | Desktop shell. Only `"tauri"` is supported in this version. |
| `identifier` | string | **Yes** | — | Reverse-DNS application identifier (e.g. `"com.example.myapp"`). |
| `window_title` | string | No | project `name` | Window title bar text. |

```toml
[[target]]
name = "desktop"
type = "desktop"
shell = "tauri"
identifier = "com.example.my-app"
window_title = "My Application"
```

**Notes:**

- `identifier` must match `^[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*){2,}$`.
- Tauri must be installed separately. The Forge CLI checks for `tauri-cli` on PATH and emits an error if absent.

#### `type = "mobile"`

Mobile target is defined in schema but not yet implemented. Specifying it emits a compile-time warning:

```text
warning: target type 'mobile' is not yet implemented and will be skipped
```

---

### 3.5 `[server]`

The `[server]` table configures the production server runtime (applies to `type = "server"` targets). It is **optional**.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `port` | u16 | No | `3000` | Port the production server listens on. |
| `host` | string | No | `"0.0.0.0"` | Address to bind. Use `"127.0.0.1"` for localhost-only. |

**Example:**

```toml
[server]
port = 8080
host = "127.0.0.1"
```

**Notes:**

- These values can be overridden at runtime via environment variables `FORGE_PORT` and `FORGE_HOST` respectively.
- Environment variable values take precedence over `forge.toml` values.

---

### 3.6 `[dependencies]`

A TOML table mapping Foundry package names to version constraints. It is **optional**; omitting it is equivalent to `[dependencies] = {}`.

**Format:**

```toml
[dependencies]
"jeshua/date-utils" = "1.2.3"
"acme/ui-kit" = "3.0.0"
"forge:db" = "*"
```

**Version constraint syntax:**

| Syntax | Meaning |
|--------|---------|
| `"1.2.3"` | Exact version. Pinned immediately. |
| `"*"` | Latest stable at resolution time. Pinned in `foundry.lock`. |

Ranges (e.g. `"^1.0"`, `">=2.0"`) are **not supported**. Forge uses exact pinning only.

**`forge:*` packages** may appear in dependencies to declare an explicit dependency on an FSL package. The version must be `"*"` (FSL packages are versioned with the Forge toolchain).

---

### 3.7 `[dev-dependencies]`

Identical schema to `[dependencies]`. Packages listed here are available during development and testing but are excluded from production builds.

```toml
[dev-dependencies]
"forge/test-utils" = "*"
"jeshua/mock-db" = "0.4.1"
```

---

## 4. Target Type Reference

| Type | Server Runtime | DOM Available | WinterTC Only | Deployment Artifact |
|------|---------------|---------------|--------------|---------------------|
| `static` | None | ✅ | — | HTML/JS/CSS files |
| `server` | Forge server | ✅ (client) | No | Binary + (optional) assets |
| `edge` | Edge isolate | ✅ (client) | Yes | ESM bundle |
| `desktop` | Tauri + Forge server | ✅ | No | Platform binary |
| `mobile` | (not implemented) | — | — | — |

---

## 5. Complete Example

The following `forge.toml` configures a project with a server target and an edge target, development server settings, and a mix of dependencies.

```toml
[project]
name = "acme-platform"
version = "0.4.0-pre-alpha"
description = "ACME's customer-facing platform"
authors = ["Jeshua Maxey <jeshua@acme.io>", "Jane Smith <jane@acme.io>"]
license = "AGPL-3.0-only"

[build]
entry = "app/root.fx"
output = ".forge/dist"
source_maps = true

[dev]
port = 3000
studio_port = 3001
open = false

[[target]]
name = "app-server"
type = "server"
output = ".forge/dist/server"
embed_assets = true

[[target]]
name = "edge-cf"
type = "edge"
provider = "cloudflare"
output = ".forge/dist/edge"

[server]
port = 8080
host = "0.0.0.0"

[dependencies]
"jeshua/date-utils" = "2.1.0"
"acme/design-system" = "1.0.0"
"jeshua/auth-helpers" = "0.9.3"

[dev-dependencies]
"forge/test-utils" = "*"
```

---

## 6. Validation Rules

The Forge CLI validates `forge.toml` at startup. Validation errors abort the command with a structured error message pointing to the offending field.

| Rule | Error Message |
|------|--------------|
| `[project]` table missing | `forge.toml: missing required table [project]` |
| `project.name` missing | `forge.toml: project.name is required` |
| `project.name` invalid format | `forge.toml: project.name must match ^[a-z][a-z0-9-]*$` |
| `project.version` missing | `forge.toml: project.version is required` |
| `project.version` not SemVer | `forge.toml: project.version must be a valid SemVer string` |
| `target.name` missing | `forge.toml: target at index N is missing required field 'name'` |
| `target.type` missing | `forge.toml: target '<name>' is missing required field 'type'` |
| `target.type` invalid | `forge.toml: target '<name>'.type must be one of: static, server, edge, desktop, mobile` |
| `target.provider` missing for edge | `forge.toml: target '<name>'.provider is required when type = "edge"` |
| `target.shell` missing for desktop | `forge.toml: target '<name>'.shell is required when type = "desktop"` |
| `target.identifier` missing for desktop | `forge.toml: target '<name>'.identifier is required when type = "desktop"` |
| Duplicate target names | `forge.toml: duplicate target name '<name>'` |
| `dev.port` == `dev.studio_port` | `forge.toml: dev.port and dev.studio_port must be different` |
| Dependency version range used | `forge.toml: dependency '<name>' uses a version range; only exact versions or "*" are supported` |

---

## 7. Environment Variable Interpolation

`forge.toml` does **not** support environment variable interpolation in the TOML source itself. All environment overrides are applied at runtime by the CLI or server binary, not during file parsing.

The following environment variables are recognized at runtime:

| Variable | Overrides |
|----------|-----------|
| `FORGE_PORT` | `[server].port` |
| `FORGE_HOST` | `[server].host` |
| `FORGE_DEV_PORT` | `[dev].port` |
| `FORGE_OUTPUT` | `[build].output` |

Environment variables are applied after `forge.toml` is parsed and validated. They do not affect validation (validation always operates on the raw TOML values).
