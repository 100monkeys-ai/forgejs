# Publishing Packages

The Foundry is Forge's curated package registry. Packages published to the Foundry are signed with the publisher's Ed25519 key and verified by the registry before acceptance.

## Package Structure

A publishable Foundry package requires:

- `forge.toml` — package manifest with `[package]` section (not `[app]`)
- `src/index.fx` — the package's main export entry point
- A `src/` directory with all source files

```
my-package/
├── forge.toml
├── src/
│   ├── index.fx
│   └── ...
└── tests/
    └── my-package.test.fx
```

The `forge.toml` for a package uses `[package]` instead of `[app]`:

```toml
[package]
name = "foundry:my-package"
version = "1.2.0"
description = "A short description of what this package does"
license = "MIT"
repository = "https://github.com/example/my-package"
keywords = ["utilities", "formatting"]

[dependencies]
"forge:data" = "^0.1.0"
```

Package names in the `foundry:` namespace are available to all publishers. Names in `forge:` are reserved for FSL packages.

## Signing Keys

Before you can publish, generate a signing keypair:

```sh
forge keys generate
```

This creates `~/.forge/keys/default.pub` (public key) and `~/.forge/keys/default.key` (private key, never leaves your machine). Register your public key with the Foundry:

```sh
forge keys register
```

You will be prompted to authenticate via your Foundry account. The registry stores your public key and associates it with your publisher identity.

## Publishing

Run `forge publish` from the package directory:

```sh
cd my-package
forge publish
```

The CLI will:

1. Validate `forge.toml` and check that all source files are present
2. Run `forge check` to type-check the package
3. Run `forge test` to ensure tests pass
4. Build a content-addressed tarball of the package
5. Sign the tarball with your private key
6. Upload the signed bundle to the Foundry registry

The registry verifies the signature, indexes the package, and makes it available for `forge install`.

### Dry Run

To validate without publishing:

```sh
forge publish --dry-run
```

### Distribution Tags

By default, `forge publish` publishes under the `latest` dist-tag. Use `--tag` to publish pre-release versions without affecting `latest`:

```sh
forge publish --tag beta
```

Consumers install tagged versions with:

```sh
forge install "foundry:my-package@beta"
```

## Versioning

Foundry packages follow semantic versioning. The registry enforces:

- Versions cannot be re-published once released (immutability)
- Patch versions must not contain breaking changes (best-effort; enforced via API surface diffing in future)
- The `latest` tag always points to the most recently published non-pre-release version

## Yanking a Release

If a release contains a critical bug:

```sh
forge yank "foundry:my-package@1.2.0"
```

Yanked versions remain available for existing users but are excluded from fresh `forge install` resolutions.
