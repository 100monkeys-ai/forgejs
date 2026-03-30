# Registry API

The Foundry registry exposes an HTTP API used by the `forge` CLI for package resolution, publishing, and metadata queries. The full API specification is in [`spec/specs/007-foundry-registry.md`](https://github.com/100monkeys-ai/forgejs/blob/main/spec/specs/007-foundry-registry.md).

## Base URL

```
https://registry.forgejs.com/v1
```

## Authentication

Publishing endpoints require a bearer token. Obtain one via:

```sh
forge login
```

This performs a browser-based OAuth flow and writes the token to `~/.forge/credentials`.

Read-only endpoints (package resolution, metadata) are unauthenticated.

## Core Endpoints

### Resolve a Package

```
GET /packages/:name/resolve?version=<constraint>
```

Returns the best matching version for the given semver constraint, including the content hash and download URL.

```json
{
  "name": "foundry:stripe",
  "version": "2.1.3",
  "hash": "sha256:a3f9c128d4e7b2c1...",
  "url": "https://registry.forgejs.com/v1/packages/foundry:stripe/2.1.3/tarball",
  "dependencies": {
    "forge:data": "^0.1.0"
  }
}
```

### Download a Package

```
GET /packages/:name/:version/tarball
```

Returns the package tarball. The CLI verifies the SHA-256 hash before extracting.

### Package Metadata

```
GET /packages/:name
```

Returns metadata for all published versions of a package, including deprecation notices and dist-tags.

### Publish a Package

```
POST /packages
Authorization: Bearer <token>
Content-Type: multipart/form-data
```

Request body:

- `manifest` — the `forge.toml` contents as JSON
- `tarball` — the signed package tarball
- `signature` — Ed25519 signature of the tarball (hex-encoded)

The registry verifies the signature against the publisher's registered public key before accepting the upload.

### Yank a Version

```
DELETE /packages/:name/:version
Authorization: Bearer <token>
```

Yanks the specified version. Returns `204 No Content` on success.

## Rate Limits

| Endpoint | Limit |
| --- | --- |
| Resolution / metadata | 1000 req/min (unauthenticated) |
| Tarball downloads | 100 req/min (unauthenticated) |
| Publishing | 10 req/min (authenticated) |

The CLI respects `Retry-After` headers from 429 responses and retries automatically.

## Mirrors

Private Foundry registries can mirror the public registry. Configure the registry URL in `forge.toml`:

```toml
[registry]
url = "https://foundry.internal.example.com"
```

The CLI will resolve packages from the configured registry, falling back to the public registry for packages not present in the mirror.
