# Testing

Forge's built-in test runner is provided by `forge:test`. Tests live in `.test.fx` or `.spec.fx` files alongside the code they test. No configuration file is needed.

## Running Tests

```sh
forge test               # Run all tests once
forge test --watch       # Re-run tests on file changes
forge test --coverage    # Run with coverage report
forge test src/auth      # Run tests matching a path
```

## Writing Tests

```typescript
// app/server/posts.test.fx
import { describe, it, expect } from 'forge:test'
import { createPost } from './posts.fx'

describe('createPost', () => {
  it('creates a post with the given title and body', async () => {
    const post = await createPost({ title: 'Hello', body: 'World' })
    expect(post.title).toBe('Hello')
    expect(post.body).toBe('World')
  })
})
```

## Component Testing

`renderComponent` mounts a `.fx` component in a lightweight DOM environment. Server functions are automatically replaced with stubs that return empty values unless you provide mocks.

```typescript
import { describe, it, expect, renderComponent, mockServerFunction } from 'forge:test'
import { loadPosts } from 'app/server/posts.fx'
import PostList from 'app/pages/PostList.fx'

describe('PostList', () => {
  it('renders a list of posts', async () => {
    const { mock } = mockServerFunction(loadPosts, async () => [
      { id: '1', title: 'First Post', body: '...' },
      { id: '2', title: 'Second Post', body: '...' },
    ])

    const { getByText } = await renderComponent(<PostList />, { mocks: { loadPosts: mock } })

    expect(getByText('First Post')).toBeDefined()
    expect(getByText('Second Post')).toBeDefined()
  })
})
```

## Server Function Testing

Test server functions in isolation by running them directly in the test environment. Forge sets up a test database (SQLite in-memory) automatically:

```typescript
import { describe, it, expect, beforeEach } from 'forge:test'
import { createPost, deletePost } from 'app/server/posts.fx'

beforeEach(async () => {
  await forge.test.resetDatabase()
})

describe('deletePost', () => {
  it('returns true for an existing post', async () => {
    const post = await createPost({ title: 'To delete', body: 'body' })
    const result = await deletePost(post.id)
    expect(result).toBe(true)
  })

  it('returns false for a non-existent post id', async () => {
    const result = await deletePost('non-existent-id')
    expect(result).toBe(false)
  })
})
```

## Snapshot Testing

```typescript
import { snapshot } from 'forge:test'
import Button from 'app/components/Button.fx'

it('renders a primary button', async () => {
  const html = await snapshot(<Button variant="primary">Click me</Button>)
  expect(html).toMatchSnapshot()
})
```

Snapshots are stored in `__snapshots__/` directories next to the test file. Update them with `forge test --update-snapshots`.
