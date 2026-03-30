# Contributing to Forge

## Development Setup

**Requirements:**

- Rust 1.88 or later (`rustup update stable`)
- `cargo` (ships with Rust)
- Git
- Optional: `mdbook` for building the documentation site

**Building from source:**

```bash
git clone https://github.com/100monkeys-ai/forgejs
cd forgejs
cargo build --workspace
```

The workspace builds all six crates: `forge-compiler`, `forge-runtime`, `forge-cli`, `foundry-server`, `foundry-client`, and `forge-shared`.

---

## Running Tests

```bash
# All workspace tests
cargo test --workspace --locked

# A single crate
cargo test -p forge-compiler --locked

# A specific test
cargo test -p forge-compiler --locked -- parser::tests::test_jsx_transform
```

---

## Code Style

**Formatting** — all Rust code must pass `rustfmt`:

```bash
cargo fmt --all --check
```

**Linting** — all Rust code must pass `clippy` with warnings as errors:

```bash
cargo clippy --workspace -- -D warnings
```

Both checks run in CI on every push. A PR that fails either check will not be merged.

**General guidelines:**

- No `unwrap()` in library code — use proper error handling with `?` and typed errors
- Prefer explicit error types over `anyhow` in library crates; `anyhow` is acceptable in binary crates
- Use `thiserror` for error type definitions
- Document all public items with `///` doc comments
- Write unit tests in the same file as the code they test (`#[cfg(test)]` module)
- Write integration tests in `tests/`

---

## Commit Message Format

```
<type>: <short summary>

<optional body — explain why, not what>
```

**Types:**

- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `refactor:` — code change that neither fixes a bug nor adds a feature
- `test:` — adding or updating tests
- `chore:` — tooling, CI, or dependency updates
- `perf:` — performance improvement

Keep the subject line under 72 characters. Use the body for anything that needs explanation — the why, not the what.

**Examples:**

```
feat: add compile-time boundary enforcement to analyzer pass
fix: resolve signals subscription leak in runtime isolate
refactor: extract emit stage from compiler into dedicated module
docs: document Foundry resolution algorithm
test: add integration tests for multi-target build pipeline
```

---

## Pull Request Process

1. Fork the repository and create a branch from `main`
2. Make your changes
3. Ensure all checks pass locally before opening a PR:

   ```bash
   cargo fmt --all
   cargo clippy --workspace --locked -- -D warnings
   cargo test --workspace --locked
   ```

4. Open a PR against `main`
5. The PR description must explain what the change does and why

**Additional requirements:**

- All PRs require passing CI (check, fmt, clippy, test)
- Spec changes require a new ADR or an update to an existing one in `spec/adrs/`
- FSL changes require corresponding test updates in the same PR
- PRs that skip the verification checklist will be closed without review

---

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

---

## Reporting Issues

Use GitHub Issues. Fill out the bug report or feature request template completely. Incomplete reports will be closed.

For security vulnerabilities, see [SECURITY.md](SECURITY.md) — do not open a public issue.
