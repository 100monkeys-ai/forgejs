# FSL Packages

The Forge Standard Library (FSL) is a set of first-party packages that ship with every Forge installation. They cover the foundational concerns of a full-stack application. FSL packages are versioned and released alongside the Forge compiler.

FSL packages are declared as dependencies in `forge.toml` just like third-party Foundry packages. They are not automatically included — only packages you declare are bundled.

## Package Reference

| Package | Description | Targets |
| --- | --- | --- |
| [`forge:router`](#forgerouter) | File-based and explicit routing, nested layouts, typed params, Link component | all |
| [`forge:data`](#forgedata) | Schema-first database layer, type-safe queries, migrations, reactive `$db` signal | server, edge |
| [`forge:auth`](#forgeauth) | Session authentication, email+password, passkeys (WebAuthn), OAuth providers, permission checks | server, edge |
| [`forge:test`](#forgetest) | Test runner, component rendering, server function mocking, snapshot testing | test only |
| [`forge:email`](#forgeemail) | Transactional email with `.fx` templates, SMTP/Resend/Postmark adapters, preview server | server, edge |
| [`forge:jobs`](#forgejobs) | Background job queue, cron scheduling, retry with exponential backoff, durable persistence | server |
| [`forge:storage`](#forgestorage) | File storage with S3/R2/local adapters, presigned URLs, Rust-native image transforms | server, edge |
| [`forge:realtime`](#forgerealtime) | WebSocket pub/sub with typed channels, presence tracking, broadcast | server, edge |

## forge:router

The primary routing module. Routes are defined explicitly in `app/routes.fx`.

```typescript
import { Router, Link, navigate, useParams } from 'forge:router'
```

Key exports: `Router.define()`, `<Link>`, `navigate()`, `useParams()`, `useLocation()`

## forge:data

Schema-first database layer. Tables are defined with `Schema.table()` and used to generate migrations and type-safe query builders.

```typescript
import { Schema, db, $db } from 'forge:data'
```

Key exports: `Schema.table()`, `Schema.text()`, `Schema.id()`, `db.query()`, `db.insert()`, `db.update()`, `db.delete()`, `$db()`

Supported adapters: `sqlite`, `postgresql`, `libsql`

## forge:auth

Session-based authentication with multiple provider strategies. Configure once in `app/auth.fx`, use everywhere.

```typescript
import { Auth, requireAuth, optionalAuth } from 'forge:auth'
```

Key exports: `Auth.configure()`, `auth.currentUser()`, `auth.requireUser()`, `auth.can()`, `requireAuth()`, `EmailProvider`, `OAuthProvider`, `PasskeyProvider`

## forge:test

The built-in test runner. Available only during `forge test` — not included in production builds.

```typescript
import { describe, it, expect, renderComponent, mockServerFunction } from 'forge:test'
```

Key exports: `describe()`, `it()`, `expect()`, `renderComponent()`, `mockServerFunction()`, `mockModule()`, `snapshot()`

## forge:email

Transactional email with `.fx` component templates. Emails are rendered to HTML using the same component model as UI, without browser-specific APIs.

```typescript
import { sendEmail, configureEmail, EmailLayout, EmailButton } from 'forge:email'
```

Key exports: `sendEmail()`, `configureEmail()`, `<EmailLayout>`, `<EmailButton>`, `<EmailText>`

## forge:jobs

Durable background job queue backed by `forge:data`. Jobs are typed, persistent, and retry-capable.

```typescript
import { Job, Scheduler, startWorker } from 'forge:jobs'
```

Key exports: `Job.define()`, `job.enqueue()`, `job.cancel()`, `Scheduler.cron()`, `startWorker()`

## forge:storage

File storage with a uniform API across S3, Cloudflare R2, and local filesystem. Image transforms are handled natively in Rust.

```typescript
import { Storage, FileRef } from 'forge:storage'
```

Key exports: `Storage.configure()`, `storage.upload()`, `storage.download()`, `storage.presignedUrl()`, `storage.delete()`, `FileRef.transform()`

## forge:realtime

WebSocket pub/sub for typed real-time messaging. Server broadcasts; clients subscribe. Presence tracking included.

```typescript
import { Channel, Presence, broadcast } from 'forge:realtime'
```

Key exports: `Channel.define()`, `channel.broadcast()`, `channel.subscribe()`, `channel.topic()`, `Presence.get()`, `broadcast()`, `broadcastToUser()`, `broadcastToRoom()`
