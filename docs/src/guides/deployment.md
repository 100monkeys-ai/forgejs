# Deployment

Forge supports four deployment targets, configured in `forge.toml`:

```toml
[app]
target = "server"   # server | edge | static | spa
```

## Targets

### server

The default target. `forge build` produces a self-contained native binary that includes the HTTP server, static asset server, and all application code. Deploy it anywhere you can run a Linux binary:

```sh
forge build --target server --release
./my-app serve --port 3000
```

The binary reads configuration from environment variables at startup. There is no Node.js, no runtime installation, no `node_modules` directory in production.

### edge

Builds for V8 isolate environments (Cloudflare Workers, Fastly Compute). Output is a single JavaScript bundle targeting the Workers runtime API. `forge:realtime` is available; `forge:jobs` writes to a KV-backed queue.

```sh
forge build --target edge
wrangler deploy
```

### static

Pre-renders all routes at build time and produces a directory of HTML, CSS, and JavaScript files. Suitable for content sites, documentation, and marketing pages. No server required.

```sh
forge build --target static --out dist/
rsync -avz dist/ user@host:/var/www/html/
```

Routes that call server functions are not available in the static target. The compiler will error if server functions are used in a static build.

### spa

Builds a single-page application with client-side routing only. No server-side rendering. Suitable for apps behind authentication where SEO is not a requirement.

```sh
forge build --target spa --out dist/
```

## Docker

The `server` target binary is the only runtime dependency. A minimal Docker image:

```dockerfile
FROM debian:bookworm-slim
COPY my-app /usr/local/bin/my-app
ENV PORT=3000
EXPOSE 3000
CMD ["my-app", "serve"]
```

For SSL termination, place a reverse proxy (nginx, Caddy, Traefik) in front of the binary and let it handle TLS.

## Environment Variables

All `forge:auth`, `forge:data`, and adapter configuration reads from environment variables at startup. The `forge.toml` `[env]` section documents required variables; `forge check-env` validates them before the server starts.

## Database Migrations in Production

Run migrations before starting the server:

```sh
my-app migrate --run
my-app serve
```

Or use `forge migrate --run` with the build-time CLI:

```sh
forge migrate --run --database $DATABASE_URL
```

## Health Check

The binary exposes `GET /health` which returns `200 OK` when the server is ready to accept traffic, including after database connection is established.

## Self-Hosted Binary Distribution

`forge release` packages the binary with a checksum and generates a release manifest suitable for hosting on your own infrastructure. Use `forge self-update` pointed at your private registry for internal tooling distribution.
