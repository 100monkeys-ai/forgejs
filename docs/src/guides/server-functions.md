# Server Functions

Server functions are the mechanism for calling server-side logic from client components. They are defined with `"use module server"` and imported by client code, which receives a generated RPC stub in place of the original implementation.

## Defining a Server Function

```typescript
// app/server/posts.fx
"use module server"

import { Posts } from 'app/schema.fx'
import { db } from 'forge:data'
import { auth } from 'app/auth.fx'

export async function createPost(input: { title: string; body: string }) {
  const user = await auth.requireUser()
  return db.insert(Posts, {
    title: input.title,
    body: input.body,
    authorId: user.id,
  })
}
```

## Calling from a Component

```typescript
// app/pages/NewPost.fx
"use module client"

import { createPost } from 'app/server/posts.fx'

export default component NewPost() {
  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()
    const data = new FormData(e.target as HTMLFormElement)
    await createPost({
      title: data.get('title') as string,
      body: data.get('body') as string,
    })
    navigate('/posts')
  }

  return (
    <form onSubmit={handleSubmit}>
      <input name="title" type="text" required />
      <textarea name="body" required />
      <button type="submit">Publish</button>
    </form>
  )
}
```

## The Boundary Contract

The compiler enforces these rules:

1. `"use module server"` files can import any module — server, client, or shared.
2. `"use module client"` files can import server modules only to call exported functions — not to access types, constants, or non-function exports.
3. Any value flowing from server to client across the RPC boundary must be JSON-serializable.
4. Any value flowing from client to server (function arguments) must be JSON-serializable.

Violations are compile-time errors, not runtime errors.

## Input Validation

Server functions should validate their inputs. Forge ships with a lightweight validation primitive:

```typescript
import { validate } from 'forge:validate'

export async function createPost(input: unknown) {
  const { title, body } = validate(input, {
    title: v => v.string().min(1).max(200),
    body: v => v.string().min(1),
  })
  // ...
}
```

## Error Handling

Errors thrown inside server functions are caught by the generated stub. By default, the error message is forwarded to the client. To return structured errors, throw a `ServerError`:

```typescript
import { ServerError } from 'forge:runtime'

throw new ServerError('VALIDATION_FAILED', { field: 'title', message: 'Too short' })
```

On the client, catch them via the promise:

```typescript
try {
  await createPost(input)
} catch (e) {
  if (e.code === 'VALIDATION_FAILED') showError(e.details.message)
}
```

## Streaming Responses

Server functions can return an `AsyncIterable` to stream data to the client:

```typescript
export async function* streamResponse() {
  for await (const chunk of llm.stream()) {
    yield chunk
  }
}
```

The client receives an `AsyncIterable` that consumes the stream.
