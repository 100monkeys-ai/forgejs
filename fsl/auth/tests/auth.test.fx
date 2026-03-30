// forge:auth tests
import { describe, it, expect } from 'forge:test'
import { Auth } from '../src/session.fx'

const auth = Auth.configure({
  session: { secret: 'test-secret-32-chars-minimum!!!' },
  providers: [Auth.email()],
  permissions: {
    'post:edit': ({ user, resource }) => user.id === (resource as { authorId: string }).authorId,
    'post:delete': ({ user }) => user.role === 'admin',
  },
})

describe('Auth.configure', () => {
  it('creates an AuthInstance', () => {
    expect(auth).toBeDefined()
    expect(typeof auth.currentUser).toBe('function')
    expect(typeof auth.can).toBe('function')
  })
})

describe('auth.can', () => {
  it('returns false when no user is authenticated', async () => {
    const result = await auth.can('post:edit', { authorId: 'user-1' })
    expect(result).toBe(false)
  })
})
