# 006 — WinterTC API Surface

**Status:** Normative
**Version:** 0.1.0-pre-alpha
**Last Updated:** 2026-03-30

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [Normative References](#2-normative-references)
3. [API Catalogue](#3-api-catalogue)
   - 3.1 [Fetch API](#31-fetch-api)
   - 3.2 [URL](#32-url)
   - 3.3 [Streams](#33-streams)
   - 3.4 [Encoding](#34-encoding)
   - 3.5 [Web Crypto](#35-web-crypto)
   - 3.6 [File API](#36-file-api)
   - 3.7 [Form Data](#37-form-data)
   - 3.8 [Events](#38-events)
   - 3.9 [Timers](#39-timers)
   - 3.10 [Microtasks](#310-microtasks)
   - 3.11 [Structured Clone](#311-structured-clone)
   - 3.12 [Base64](#312-base64)
   - 3.13 [Abort](#313-abort)
   - 3.14 [Message Channels](#314-message-channels)
   - 3.15 [WebSocket (Server)](#315-websocket-server)
4. [Explicitly Absent APIs](#4-explicitly-absent-apis)
5. [Target Availability Matrix](#5-target-availability-matrix)
6. [Forge FSL Replacements](#6-forge-fsl-replacements)
7. [Compliance Testing](#7-compliance-testing)

---

## 1. Purpose

The Forge server runtime and edge runtime implement the **WinterTC** (Winter Community Group for server-side JavaScript interoperability) API surface. This spec defines the normative list of APIs that are **guaranteed** to be present in every Forge non-browser runtime target (`server` binary and `edge`).

Code that only uses APIs defined in this spec is portable across:

- The Forge server binary runtime.
- The Forge edge runtime (Cloudflare Workers target).
- Any WinterTC-compliant runtime (Deno, Bun, Netlify Edge, etc.).

APIs outside this spec are **not portable** even if they happen to work in a specific runtime.

The static (browser) target is not a WinterTC runtime — it runs in the user's browser. The availability matrix in §5 notes browser availability where relevant.

---

## 2. Normative References

| Specification | URL | Relevant Section |
|--------------|-----|-----------------|
| WHATWG Fetch Living Standard | <https://fetch.spec.whatwg.org/> | Fetch API |
| WHATWG URL Living Standard | <https://url.spec.whatwg.org/> | URL, URLSearchParams |
| WHATWG Streams Living Standard | <https://streams.spec.whatwg.org/> | ReadableStream, WritableStream, TransformStream |
| WHATWG Encoding Living Standard | <https://encoding.spec.whatwg.org/> | TextEncoder, TextDecoder |
| W3C Web Cryptography API | <https://www.w3.org/TR/WebCryptoAPI/> | crypto.subtle |
| W3C File API | <https://www.w3.org/TR/FileAPI/> | Blob, File |
| WHATWG HTML Living Standard | <https://html.spec.whatwg.org/> | FormData, timers, queueMicrotask, structuredClone, atob, btoa, MessageChannel |
| DOM Living Standard | <https://dom.spec.whatwg.org/> | EventTarget, Event, AbortController |
| WinterTC Minimum Common API | <https://wintercg.org/work> | Minimum Common API baseline |

---

## 3. API Catalogue

### 3.1 Fetch API

**Spec source:** WHATWG Fetch Living Standard

| Name | Type | Description |
|------|------|-------------|
| `fetch` | `function` | Make HTTP requests. Returns `Promise<Response>`. |
| `Request` | `class` | Represents an HTTP request. Constructor: `new Request(url, init?)`. |
| `Response` | `class` | Represents an HTTP response. Includes `Response.json()`, `Response.text()`, `Response.error()` static methods. |
| `Headers` | `class` | Represents HTTP headers. Iterable. Constructor: `new Headers(init?)`. |

**Supported `RequestInit` fields:** `method`, `headers`, `body`, `signal`, `redirect`, `referrer`, `referrerPolicy`, `credentials` (edge only — server binary uses service credentials separately), `keepalive` (best-effort on server).

**Notes:**

- `fetch` in the server binary uses the system's TLS stack. CA certificates come from the OS certificate store.
- `fetch` in the edge runtime uses the provider's network stack (Cloudflare's in the CF Workers target).
- Streaming request bodies (`ReadableStream` as `body`) are supported on both targets.

---

### 3.2 URL

**Spec source:** WHATWG URL Living Standard

| Name | Type | Description |
|------|------|-------------|
| `URL` | `class` | Parses and manipulates URLs. Constructor: `new URL(url, base?)`. |
| `URLSearchParams` | `class` | Represents a URL query string. Constructor: `new URLSearchParams(init?)`. |

**All WHATWG URL properties are available:** `href`, `origin`, `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `searchParams`, `hash`.

---

### 3.3 Streams

**Spec source:** WHATWG Streams Living Standard

| Name | Type | Description |
|------|------|-------------|
| `ReadableStream<T>` | `class` | Represents a stream of data that can be read. |
| `WritableStream<T>` | `class` | Represents a stream of data that can be written to. |
| `TransformStream<I, O>` | `class` | A `{ readable, writable }` pair that transforms data. |
| `ReadableStreamDefaultReader<T>` | `class` | Obtained via `readableStream.getReader()`. |
| `WritableStreamDefaultWriter<T>` | `class` | Obtained via `writableStream.getWriter()`. |
| `CompressionStream` | `class` | `TransformStream` that compresses. Algorithms: `gzip`, `deflate`, `deflate-raw`. |
| `DecompressionStream` | `class` | `TransformStream` that decompresses. Same algorithms. |
| `ReadableStreamBYOBReader` | `class` | Bring-your-own-buffer reader. Available on server binary; edge support is provider-dependent. |

**Pipe operations:** `.pipeThrough()`, `.pipeTo()`, `.tee()` are all available.

---

### 3.4 Encoding

**Spec source:** WHATWG Encoding Living Standard

| Name | Type | Description |
|------|------|-------------|
| `TextEncoder` | `class` | Encodes strings to `Uint8Array`. Always UTF-8. |
| `TextDecoder` | `class` | Decodes `Uint8Array` to string. Constructor: `new TextDecoder(label?, options?)`. Supports UTF-8, UTF-16LE, UTF-16BE, and ASCII labels. |
| `TextEncoderStream` | `class` | `TransformStream` wrapping `TextEncoder`. |
| `TextDecoderStream` | `class` | `TransformStream` wrapping `TextDecoder`. |

---

### 3.5 Web Crypto

**Spec source:** W3C Web Cryptography API

The `crypto` global is available with the following surface:

#### `crypto.getRandomValues(array)`

```typescript
crypto.getRandomValues<T extends ArrayBufferView>(array: T): T
```

Fills `array` with cryptographically strong random bytes. Returns the same array. Throws `QuotaExceededError` if `array.byteLength > 65536`.

#### `crypto.randomUUID()`

```typescript
crypto.randomUUID(): string
```

Returns a RFC 4122 v4 UUID string.

#### `crypto.subtle`

The `SubtleCrypto` interface with the following methods:

| Method | Supported Algorithms |
|--------|---------------------|
| `subtle.encrypt(algo, key, data)` | AES-CBC, AES-CTR, AES-GCM, RSA-OAEP |
| `subtle.decrypt(algo, key, data)` | AES-CBC, AES-CTR, AES-GCM, RSA-OAEP |
| `subtle.sign(algo, key, data)` | HMAC, RSASSA-PKCS1-v1_5, RSA-PSS, ECDSA |
| `subtle.verify(algo, key, data, sig)` | HMAC, RSASSA-PKCS1-v1_5, RSA-PSS, ECDSA |
| `subtle.generateKey(algo, extractable, usages)` | AES-CBC, AES-CTR, AES-GCM, AES-KW, HMAC, RSA-OAEP, RSASSA-PKCS1-v1_5, RSA-PSS, ECDH, ECDSA |
| `subtle.importKey(format, keyData, algo, extractable, usages)` | All of the above |
| `subtle.exportKey(format, key)` | `"raw"`, `"pkcs8"`, `"spki"`, `"jwk"` |
| `subtle.deriveBits(algo, key, length)` | ECDH, HKDF, PBKDF2 |
| `subtle.deriveKey(algo, key, derivedKeyType, extractable, usages)` | ECDH, HKDF, PBKDF2 |
| `subtle.digest(algo, data)` | SHA-1 (deprecated), SHA-256, SHA-384, SHA-512 |
| `subtle.wrapKey(format, key, wrapKey, wrapAlgo)` | AES-KW, AES-GCM, RSA-OAEP |
| `subtle.unwrapKey(format, wrappedKey, unwrapKey, unwrapAlgo, unwrappedKeyAlgo, extractable, usages)` | AES-KW, AES-GCM, RSA-OAEP |

All `SubtleCrypto` methods return `Promise`.

---

### 3.6 File API

**Spec source:** W3C File API

| Name | Type | Description |
|------|------|-------------|
| `Blob` | `class` | Represents immutable raw binary data. Constructor: `new Blob(parts?, options?)`. Methods: `.arrayBuffer()`, `.bytes()`, `.text()`, `.stream()`, `.slice()`. |
| `File` | `class` | Extends `Blob` with `name` and `lastModified`. Constructor: `new File(parts, name, options?)`. |

**Note:** `FileReader` is a browser-only API that uses event listeners and is not present in WinterTC runtimes. Use `blob.arrayBuffer()`, `blob.text()`, or `blob.stream()` instead.

---

### 3.7 Form Data

**Spec source:** WHATWG HTML Living Standard

| Name | Type | Description |
|------|------|-------------|
| `FormData` | `class` | Represents `multipart/form-data` or `application/x-www-form-urlencoded` data. Constructor: `new FormData()`. Methods: `.append()`, `.delete()`, `.get()`, `.getAll()`, `.has()`, `.set()`. Iterable. |

---

### 3.8 Events

**Spec source:** DOM Living Standard, WHATWG HTML Living Standard

| Name | Type | Description |
|------|------|-------------|
| `EventTarget` | `class` | Base class for event dispatching. Methods: `.addEventListener()`, `.removeEventListener()`, `.dispatchEvent()`. |
| `Event` | `class` | Base event class. Constructor: `new Event(type, init?)`. |
| `CustomEvent<T>` | `class` | Event with custom detail. Constructor: `new CustomEvent(type, { detail? })`. |
| `ErrorEvent` | `class` | Represents an error event. Available for unhandled rejection reporting. |
| `MessageEvent<T>` | `class` | Data message across message channels. |

**Not available:** `MouseEvent`, `KeyboardEvent`, `PointerEvent`, `TouchEvent`, `FocusEvent`, `InputEvent`, `UIEvent` — these are browser-only events tied to user interaction.

---

### 3.9 Timers

**Spec source:** HTML Living Standard (§8.6)

| Name | Signature | Description |
|------|-----------|-------------|
| `setTimeout` | `(callback: () => void, ms?: number, ...args: unknown[]): number` | Schedules `callback` after at least `ms` milliseconds. Returns a numeric timer ID. |
| `clearTimeout` | `(id: number): void` | Cancels a pending `setTimeout`. |
| `setInterval` | `(callback: () => void, ms?: number, ...args: unknown[]): number` | Schedules `callback` every `ms` milliseconds until cancelled. |
| `clearInterval` | `(id: number): void` | Cancels a `setInterval`. |

**Minimum delay:** 0 ms is allowed. The actual resolution is runtime-dependent (typically 1–4 ms).

**Edge note:** Timers in edge runtimes are scoped to the request lifetime. A `setTimeout` that fires after the response is sent may be silently dropped. Use timers only for within-request scheduling in edge functions.

---

### 3.10 Microtasks

**Spec source:** HTML Living Standard (§8.1.7)

| Name | Signature | Description |
|------|-----------|-------------|
| `queueMicrotask` | `(callback: () => void): void` | Schedules `callback` as a microtask on the current microtask checkpoint. |

`queueMicrotask` is the preferred mechanism for scheduling reactive updates in the Forge signal system.

---

### 3.11 Structured Clone

**Spec source:** HTML Living Standard (§2.7.5)

| Name | Signature | Description |
|------|-----------|-------------|
| `structuredClone` | `<T>(value: T, options?: { transfer?: Transferable[] }): T` | Creates a deep clone using the structured clone algorithm. |

**Supported types:** Primitives, plain objects, Arrays, `Date`, `RegExp`, `Map`, `Set`, `ArrayBuffer`, `TypedArray`, `Blob`, `File`, `ReadableStream` (transferable), `MessagePort` (transferable).

**Not supported:** Functions, class instances (prototype chain is dropped), DOM nodes, `WeakMap`, `WeakSet`.

---

### 3.12 Base64

**Spec source:** HTML Living Standard (§2.4.6)

| Name | Signature | Description |
|------|-----------|-------------|
| `atob` | `(data: string): string` | Decodes a base64-encoded string to binary string. |
| `btoa` | `(data: string): string` | Encodes a binary string to base64. |

**Note:** `atob`/`btoa` operate on binary strings (Latin-1 encoding). For encoding arbitrary UTF-8 text to base64, use `TextEncoder` + `Uint8Array` + a base64 utility. The `forge:crypto` FSL package provides `encodeBase64(bytes: Uint8Array): string` and `decodeBase64(str: string): Uint8Array` as ergonomic alternatives.

---

### 3.13 Abort

**Spec source:** DOM Living Standard

| Name | Type | Description |
|------|------|-------------|
| `AbortController` | `class` | Controls an `AbortSignal`. Constructor: `new AbortController()`. Properties: `.signal`. Methods: `.abort(reason?)`. |
| `AbortSignal` | `class` | Represents an abort signal. Properties: `.aborted`, `.reason`. Methods: `.throwIfAborted()`. Static: `AbortSignal.timeout(ms)`, `AbortSignal.any(signals)`. |

`AbortSignal` integrates with `fetch` (`{ signal: controller.signal }`) and with the Forge streaming APIs.

---

### 3.14 Message Channels

**Spec source:** HTML Living Standard (§9.5)

| Name | Type | Description |
|------|------|-------------|
| `MessageChannel` | `class` | Creates a pair of connected `MessagePort`s. Constructor: `new MessageChannel()`. Properties: `.port1`, `.port2`. |
| `MessagePort` | `class` | One end of a `MessageChannel`. Methods: `.postMessage(data, transfer?)`, `.start()`, `.close()`. Events: `message`, `messageerror`. |

`MessageChannel` is used for inter-isolate communication in the Forge server runtime when spawning sub-workers. It is not used for browser-to-server communication.

---

### 3.15 WebSocket (Server)

**Spec source:** WHATWG WebSockets Living Standard (server-side behaviour)

| Name | Type | Description |
|------|------|-------------|
| `WebSocket` | `class` | Server-side WebSocket. Obtained from an HTTP upgrade request, not by direct construction. |

The server-side `WebSocket` API differs from the browser `WebSocket` API:

- Not constructed directly — obtained from the upgrade mechanism provided by `forge:router`.
- Properties: `.readyState`, `.protocol`, `.extensions`.
- Methods: `.send(data)`, `.close(code?, reason?)`, `.accept()`.
- Events: `message`, `close`, `error`.

The browser WebSocket (`new WebSocket(url)`) is a native browser API and is not part of the WinterTC surface. It is available in the static/client bundle without restriction.

---

## 4. Explicitly Absent APIs

The following are explicitly **not available** in the Forge server or edge runtime. Code that uses them will fail at runtime if it reaches a server or edge target. The boundary enforcement pass (spec 005 §7) detects many of these statically.

| Absent API | Forge/WinterTC Alternative |
|-----------|--------------------------|
| `process.env.*` | `forge:env` — `import { env } from 'forge:env'` |
| `process.exit()` | Throw an unrecoverable error; let the runtime handle shutdown |
| `process.argv` | Not available in server functions; use config or env |
| `process.version`, `process.platform` | Not available; don't rely on Node.js version strings |
| `node:*` imports | Use WinterTC APIs or FSL packages |
| `require()` | ESM only; use `import` |
| `Buffer` | `Uint8Array` — fully compatible for binary data |
| `__dirname`, `__filename` | `import.meta.url` + `new URL()` |
| `fs`, `path`, `http`, `https`, `net`, `os` | `forge:fs`, `forge:router` for the relevant subset |
| `node:crypto` | `crypto.subtle` (WinterTC Web Crypto) or `forge:crypto` |
| `XMLHttpRequest` | `fetch()` |
| `FileReader` | `blob.arrayBuffer()`, `blob.text()`, `blob.stream()` |
| `localStorage`, `sessionStorage` | `forge:kv` for server-side key-value storage |
| `alert()`, `confirm()`, `prompt()` | Not applicable in server context |
| `document`, `window`, `navigator` | Browser-only; boundary enforcement blocks these statically |
| `requestAnimationFrame` | Browser-only |
| `canvas`, `WebGL` | Browser-only |
| `Worker` (browser Web Worker) | `MessageChannel` + internal worker API in `forge:worker` |

---

## 5. Target Availability Matrix

The table below lists availability across Forge's three primary compilation targets.

| API | `server` binary | `edge` (CF Workers) | `static` (browser) |
|-----|:--------------:|:-------------------:|:------------------:|
| `fetch` | ✅ | ✅ | ✅ |
| `Request` / `Response` / `Headers` | ✅ | ✅ | ✅ |
| `URL` / `URLSearchParams` | ✅ | ✅ | ✅ |
| `ReadableStream` / `WritableStream` / `TransformStream` | ✅ | ✅ | ✅ |
| `CompressionStream` / `DecompressionStream` | ✅ | ✅ | ✅ |
| `TextEncoder` / `TextDecoder` | ✅ | ✅ | ✅ |
| `TextEncoderStream` / `TextDecoderStream` | ✅ | ✅ | ✅ |
| `crypto.getRandomValues` | ✅ | ✅ | ✅ |
| `crypto.randomUUID` | ✅ | ✅ | ✅ |
| `crypto.subtle` (all methods) | ✅ | ✅ | ✅ |
| `Blob` | ✅ | ✅ | ✅ |
| `File` | ✅ | ✅ | ✅ |
| `FormData` | ✅ | ✅ | ✅ |
| `EventTarget` / `Event` / `CustomEvent` | ✅ | ✅ | ✅ |
| `MessageEvent` | ✅ | ✅ | ✅ |
| `setTimeout` / `clearTimeout` | ✅ | ✅ (request-scoped) | ✅ |
| `setInterval` / `clearInterval` | ✅ | ✅ (request-scoped) | ✅ |
| `queueMicrotask` | ✅ | ✅ | ✅ |
| `structuredClone` | ✅ | ✅ | ✅ |
| `atob` / `btoa` | ✅ | ✅ | ✅ |
| `AbortController` / `AbortSignal` | ✅ | ✅ | ✅ |
| `MessageChannel` / `MessagePort` | ✅ | ✅ | ✅ |
| `WebSocket` (server-side upgrade) | ✅ | ✅ | ❌ (client WS is native) |
| `ReadableStreamBYOBReader` | ✅ | ⚠️ provider-dependent | ✅ |
| `import.meta.url` | ✅ | ✅ | ✅ |
| `globalThis` | ✅ | ✅ | ✅ |

**Legend:**

- ✅ — Available and spec-compliant.
- ⚠️ — Available in some configurations; not guaranteed.
- ❌ — Not available.

---

## 6. Forge FSL Replacements

For APIs absent from WinterTC, the Forge Standard Library provides equivalents:

| Missing API | FSL Replacement | Import |
|------------|-----------------|--------|
| `process.env.FOO` | `env.get('FOO')` | `import { env } from 'forge:env'` |
| `fs.readFile()` | `fs.read(path)` | `import { fs } from 'forge:fs'` |
| `fs.writeFile()` | `fs.write(path, data)` | `import { fs } from 'forge:fs'` |
| `node:crypto` HMAC | `crypto.subtle.sign('HMAC', ...)` | Built-in Web Crypto |
| `node:crypto` random bytes | `crypto.getRandomValues(new Uint8Array(n))` | Built-in Web Crypto |
| `node:path` join | `new URL(rel, base).pathname` | Built-in URL |
| `Buffer.from(str, 'base64')` | `forge:crypto` `decodeBase64()` | `import { decodeBase64 } from 'forge:crypto'` |
| `localStorage` | `kv.get(key)` / `kv.set(key, value)` | `import { kv } from 'forge:kv'` |
| `node:http` server | `router.handle(req)` | `import { router } from 'forge:router'` |

---

## 7. Compliance Testing

Forge runs the [WinterTC Minimum Common API conformance tests](https://github.com/wintercg/proposal-common-minimum-api) as part of its CI pipeline. The server binary and edge bundle must pass all mandatory tests before release.

Non-mandatory tests that require browser-specific behaviour (e.g. DOM event bubbling) are skipped for the server target.

The compliance test suite is run against:

- The Forge server binary (`forge:server` runtime).
- The Cloudflare Workers edge bundle output (using `wrangler dev`).

Any regression in WinterTC compliance is a release-blocking defect.
