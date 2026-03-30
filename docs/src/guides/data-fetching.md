# Data Fetching

Forge handles data loading through server functions and the `$async` signal. There is no client-side `useEffect` for data fetching; all data loading flows through the server boundary.

## Loading Data with Server Functions

The idiomatic pattern is to define a `load` server function per page and call it from the component:

```typescript
// app/server/posts.fx
"use module server"

import { Posts } from 'app/schema.fx'
import { db } from 'forge:data'

export async function loadPosts() {
  return db.query<Post>(Posts).orderBy('createdAt', 'desc').limit(20).all()
}
```

```typescript
// app/pages/PostList.fx
"use module client"

import { loadPosts } from 'app/server/posts.fx'
import { $async } from 'forge:signals'

export default component PostList() {
  const $posts = $async(loadPosts)

  return (
    <ul>
      {$posts.value.map(post => (
        <li key={post.id}>{post.title}</li>
      ))}
    </ul>
  )
}
```

The compiler detects the cross-boundary import and replaces `loadPosts` with an RPC stub on the client.

## $async Signal

`$async` wraps any server function in a reactive signal. The signal has three states:

| State | `$posts.loading` | `$posts.value` | `$posts.error` |
| --- | --- | --- | --- |
| Pending | `true` | `undefined` | `undefined` |
| Resolved | `false` | The data | `undefined` |
| Rejected | `false` | `undefined` | The error |

```typescript
const $posts = $async(loadPosts)

if ($posts.loading) return <Spinner />
if ($posts.error) return <ErrorMessage error={$posts.error} />
return <PostList posts={$posts.value} />
```

## Passing Arguments

```typescript
const $post = $async(() => loadPost(postId))
```

The `$async` signal re-fetches whenever the function reference changes, so wrapping it in an arrow function that captures reactive state will trigger re-fetches when that state changes.

## Mutations

For write operations, call server functions directly and invalidate the signal:

```typescript
import { createPost } from 'app/server/posts.fx'

async function handleSubmit(formData: FormData) {
  await createPost({ title: formData.get('title') as string })
  $posts.refresh()
}
```

## Server-Side Rendering

On the server, `$async` signals are awaited before rendering. The HTML sent to the client includes the resolved data, and the client hydrates without a second fetch.
