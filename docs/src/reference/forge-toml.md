# forge.toml Reference

`forge.toml` is the project manifest. It declares the application metadata, deployment target, dependencies, and environment configuration.

The full `forge.toml` specification is in [`spec/specs/001-forge-toml.md`](https://github.com/100monkeys-ai/forgejs/blob/main/spec/specs/001-forge-toml.md).

## Annotated Example

```toml
# ── Application ──────────────────────────────────────────────────────────────

[app]
# The application name. Used as the binary name for server targets.
name = "my-app"

# Semantic version of this application.
version = "1.0.0"

# Deployment target. One of: server, edge, static, spa.
# Can be overridden per build with --target.
target = "server"

# ── Dependencies ─────────────────────────────────────────────────────────────

[dependencies]
# Forge Standard Library packages. Pinned to a version range.
"forge:router"   = "0.1.0-pre-alpha"
"forge:data"     = "0.1.0-pre-alpha"
"forge:auth"     = "0.1.0-pre-alpha"
"forge:email"    = "0.1.0-pre-alpha"
"forge:jobs"     = "0.1.0-pre-alpha"
"forge:storage"  = "0.1.0-pre-alpha"
"forge:realtime" = "0.1.0-pre-alpha"

# Third-party Foundry packages.
"foundry:stripe" = "^2.1.0"

[dev-dependencies]
# Packages only needed for development and testing.
"forge:test" = "0.1.0-pre-alpha"

# ── Database ──────────────────────────────────────────────────────────────────

[database]
# Adapter: sqlite, postgresql, or libsql.
adapter = "postgresql"

# Connection string. Use environment variable interpolation for secrets.
url = "${DATABASE_URL}"

# ── Environment Variables ─────────────────────────────────────────────────────

[env]
# Declare required environment variables. forge check-env validates these.
# Forge never reads env vars at build time — only at runtime.
required = [
  "DATABASE_URL",
  "SESSION_SECRET",
  "GITHUB_CLIENT_ID",
  "GITHUB_CLIENT_SECRET",
]

# ── Build ─────────────────────────────────────────────────────────────────────

[build]
# Additional entry points beyond app/routes.fx.
# Use for worker scripts, CLI tools built in the same repo, etc.
entries = []

# Assets to copy verbatim to the output. Paths relative to project root.
assets = ["public/"]
```

## Keys

### [app]

| Key | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | Yes | Application name. Alphanumeric and hyphens only. |
| `version` | string | Yes | Semantic version string. |
| `target` | string | Yes | Deployment target. |

### [dependencies] / [dev-dependencies]

Dependency version constraints follow semver. Supported operators: exact (`1.0.0`), caret (`^1.0.0`), tilde (`~1.0.0`), wildcard (`*`).

### [database]

| Key | Type | Required | Description |
| --- | --- | --- | --- |
| `adapter` | string | Yes | `sqlite`, `postgresql`, or `libsql` |
| `url` | string | Yes | Connection string or environment variable reference |

### [env]

| Key | Type | Required | Description |
| --- | --- | --- | --- |
| `required` | string[] | No | Variable names that must be set at runtime |

### [build]

| Key | Type | Required | Description |
| --- | --- | --- | --- |
| `entries` | string[] | No | Additional build entry points |
| `assets` | string[] | No | Directories to copy to output |
