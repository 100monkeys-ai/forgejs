# CLI Reference

The `forge` binary is the single entrypoint for all project operations.

## forge new

Scaffold a new Forge project.

```sh
forge new <name> [options]
```

| Flag | Default | Description |
| --- | --- | --- |
| `--template <name>` | `default` | Project template: `default`, `minimal`, `saas` |
| `--target <target>` | `server` | Deployment target: `server`, `edge`, `static`, `spa` |
| `--no-git` | — | Skip `git init` |

## forge dev

Start the development server with hot module reload.

```sh
forge dev [options]
```

| Flag | Default | Description |
| --- | --- | --- |
| `--port <n>` | `3000` | HTTP port |
| `--host <addr>` | `127.0.0.1` | Bind address |
| `--no-open` | — | Do not open browser on start |

## forge build

Build the application for production.

```sh
forge build [options]
```

| Flag | Default | Description |
| --- | --- | --- |
| `--target <target>` | Value from `forge.toml` | Override deployment target |
| `--release` | — | Enable release optimizations |
| `--out <dir>` | `dist/` | Output directory (static/spa targets) |
| `--sourcemap` | — | Emit source maps |

## forge test

Run the test suite.

```sh
forge test [pattern] [options]
```

| Flag | Default | Description |
| --- | --- | --- |
| `--watch` | — | Re-run on file changes |
| `--coverage` | — | Emit coverage report |
| `--update-snapshots` | — | Update snapshot files |
| `--reporter <name>` | `pretty` | Output reporter: `pretty`, `json`, `tap` |
| `--timeout <ms>` | `5000` | Per-test timeout |

## forge migrate

Manage database migrations.

```sh
forge migrate <subcommand> [options]
```

| Subcommand | Description |
| --- | --- |
| `generate` | Generate a new migration from schema diff |
| `run` | Apply pending migrations |
| `rollback` | Roll back the last migration |
| `status` | Show applied and pending migrations |
| `reset` | Roll back all migrations and re-apply (development only) |

| Flag | Description |
| --- | --- |
| `--database <url>` | Override the database URL |
| `--name <name>` | Migration name (for `generate`) |

## forge install

Install Foundry packages.

```sh
forge install [packages...] [options]
```

| Flag | Description |
| --- | --- |
| `--save-dev` | Add to `[dev-dependencies]` |
| `--exact` | Pin to exact version |

If called with no arguments, installs all dependencies listed in `forge.toml`.

## forge publish

Publish a package to the Foundry registry.

```sh
forge publish [options]
```

| Flag | Description |
| --- | --- |
| `--dry-run` | Validate without publishing |
| `--tag <tag>` | Publish under a dist-tag (default: `latest`) |

## forge check

Type-check the project without emitting output.

```sh
forge check
```

## forge check-env

Validate that all required environment variables are set.

```sh
forge check-env
```

## forge cache

Manage the Foundry package cache.

```sh
forge cache <subcommand>
```

| Subcommand | Description |
| --- | --- |
| `info` | Show cache location and disk usage |
| `clean` | Remove all cached packages |
| `clean <package>` | Remove a specific package from cache |

## forge self-update

Update the `forge` binary to the latest release.

```sh
forge self-update [--version <version>]
```
