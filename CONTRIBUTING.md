# Contributing to Forge

## Development Setup

**Requirements:**

- Rust 1.88 or later (`rustup update stable`)
- `cargo` (ships with Rust)
- Git

**Clone and build:**

```bash
git clone https://github.com/100monkeys-ai/forgejs.git
cd forgejs
cargo build --workspace
```

The workspace builds all six crates: `forge-compiler`, `forge-runtime`, `forge-cli`, `foundry-server`, `foundry-client`, and `forge-shared`.

## Running Tests

```bash
# All workspace tests
cargo test --workspace --locked

# A single crate
cargo test -p forge-compiler --locked

# A specific test
cargo test -p forge-compiler --locked -- parser::tests::test_jsx_transform
```

## Code Style

**Formatting** — enforced by `rustfmt`:

```bash
cargo fmt --all
```

**Linting** — warnings are errors:

```bash
cargo clippy --workspace --locked -- -D warnings
```

Both checks run in CI on every push and PR. A PR that fails either check will not be merged.

**General guidelines:**

- Prefer explicit error types over `anyhow` in library crates; `anyhow` is acceptable in binary crates
- Use `thiserror` for error definitions
- Document all public items with `///` doc comments
- Write unit tests in the same file as the code they test (`#[cfg(test)]` module)
- Write integration tests in `tests/`

## Commit Message Format

```
<type>: <short summary>

<optional body — explain why, not what>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`

Examples:

```
feat: add boundary enforcement to compiler pipeline
fix: resolve signals subscription leak in runtime
refactor: extract emit stage from compiler into separate module
docs: document Foundry resolution algorithm
```

Keep the subject line under 72 characters. Use the body for anything that needs explanation.

## Pull Request Process

1. Fork the repository and create a branch: `git checkout -b feat/your-feature`
2. Make your changes
3. Ensure all checks pass:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --locked -- -D warnings
   cargo test --workspace --locked
   ```
4. Open a PR against `main`
5. The PR description should explain what the change does and why

PRs that skip the verification checklist will be closed without review.

## Repository Layout

```
forgejs/
  compiler/       # forge-compiler — Oxc build pipeline
  runtime/        # forge-runtime — V8 execution + HTTP server
  cli/            # forge-cli — the forge binary
  foundry/
    server/       # foundry-server — package registry
    client/       # foundry-client — registry client library
  shared/         # forge-shared — shared types
  fsl/            # Forge Standard Library (.fx packages)
  spec/           # Normative specifications and ADRs
  docs/           # mdBook documentation
  docker/         # Dockerfiles
  scripts/        # Install and utility scripts
```

## Reporting Issues

Use GitHub Issues. Fill out the bug report or feature request template completely. Incomplete reports will be closed.

For security vulnerabilities, see [SECURITY.md](SECURITY.md).
