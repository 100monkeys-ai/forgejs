# Installation

## Requirements

Forge ships as a single pre-compiled binary for Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), and Windows (x86_64). No Node.js, no npm, no Rust toolchain required to run Forge applications.

To build Forge from source, you need:

- Rust 1.78 or later (`rustup install stable`)
- A C compiler (clang or gcc)
- `pkg-config` and OpenSSL development headers on Linux

## Install via Script

The fastest way to install Forge is via the install script:

```sh
curl -fsSL https://forgejs.com/install.sh | sh
```

The script detects your platform, downloads the latest release binary, verifies the SHA-256 checksum, and places `forge` in `~/.forge/bin/`. It also adds that directory to your shell profile (`~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish`).

Restart your terminal or source your shell profile, then verify the installation:

```sh
forge --version
```

```
forge 0.1.0-pre-alpha (rustc 1.78.0, commit a3f9c12)
```

## Install via Cargo

If you have the Rust toolchain installed and prefer to build from source:

```sh
cargo install forgejs-cli
```

## Install via Package Managers

Homebrew (macOS and Linux):

```sh
brew install 100monkeys-ai/tap/forge
```

## The Foundry Cache

When you install packages with `forge install`, they are cached at `~/.forge/cache/`. The cache is content-addressed: each package version is stored exactly once regardless of how many projects use it. To clear the cache:

```sh
forge cache clean
```

To inspect cache usage:

```sh
forge cache info
```

## Updating Forge

```sh
forge self-update
```

This downloads and installs the latest release, preserving your Foundry cache.
