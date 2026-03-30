# Installing Packages

Forge resolves packages from the Foundry registry and stores them in the content-addressed local cache at `~/.forge/cache/`.

## Installing Dependencies

To install all dependencies declared in `forge.toml`:

```sh
forge install
```

To add a new package:

```sh
forge install foundry:stripe
forge install foundry:stripe@^2.1.0   # with version constraint
forge install foundry:stripe@2.1.3    # exact version
```

After installation, `forge.toml` is updated with the new dependency and `foundry.lock` is updated with the resolved version and content hash.

## foundry.lock

The lockfile records the exact version and content hash of every installed package and its transitive dependencies. It ensures that `forge install` on any machine — CI, a team member's laptop, a production build — resolves to identical packages.

**Always commit `foundry.lock` to version control.**

Example lockfile entry:

```toml
[["foundry:stripe"]]
version = "2.1.3"
hash = "sha256:a3f9c128d4e7b2c1..."
resolved = "https://registry.forgejs.com/foundry:stripe/2.1.3.tar"
dependencies = []
```

## Cache

The Foundry cache is content-addressed. Each package version is stored once regardless of how many projects use it:

```
~/.forge/cache/
└── packages/
    ├── sha256:a3f9c128.../   # foundry:stripe@2.1.3
    └── sha256:b8d2e416.../   # forge:router@0.1.0-pre-alpha
```

The cache is never automatically cleaned. To free disk space:

```sh
forge cache clean              # Remove all cached packages
forge cache clean foundry:stripe  # Remove a specific package (all versions)
```

Packages are re-downloaded from the registry on next `forge install`.

## Offline Installation

If all dependencies are already in the local cache, `forge install` works without network access. This is useful in air-gapped environments or CI pipelines with pre-warmed caches.

To pre-warm a cache for CI, install dependencies in a setup step and cache the `~/.forge/cache/` directory between runs.

## Workspaces

Forge supports monorepo workspaces. Declare member packages in the root `forge.toml`:

```toml
[workspace]
members = ["packages/app", "packages/shared", "packages/api"]
```

`forge install` run from the workspace root installs dependencies for all members and deduplicates shared packages in the cache.

## Updating Packages

```sh
forge update                     # Update all dependencies within their declared constraints
forge update foundry:stripe      # Update a specific package
forge update foundry:stripe@3    # Update to a specific major version
```

After `forge update`, review the changes to `foundry.lock` before committing.
