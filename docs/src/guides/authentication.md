# Authentication

Authentication is provided by `forge:auth`. Configure a single `Auth` instance in `app/auth.fx`, then import it from server functions and middleware throughout your application.

## Setup

Install `forge:auth`:

```sh
forge install forge:auth
```

Configure the auth instance in `app/auth.fx`:

```typescript
// app/auth.fx
"use module server"

import { Auth } from 'forge:auth'

export const auth = Auth.configure({
  session: {
    secret: env.SESSION_SECRET,
    maxAge: 60 * 60 * 24 * 30, // 30 days
    secure: env.NODE_ENV === 'production',
  },
  providers: [
    Auth.email(),
    Auth.oauth('github', {
      clientId: env.GITHUB_CLIENT_ID,
      clientSecret: env.GITHUB_CLIENT_SECRET,
    }),
  ],
  permissions: {
    'post:edit': ({ user, resource }) =>
      user.id === (resource as Post).authorId || user.role === 'admin',
    'post:delete': ({ user }) => user.role === 'admin',
  },
})
```

## Getting the Current User

In any server function:

```typescript
import { auth } from 'app/auth.fx'

export async function loadDashboard() {
  const user = await auth.requireUser() // throws redirect if not authenticated
  return { user, stats: await loadUserStats(user.id) }
}
```

Use `currentUser()` when the user may or may not be authenticated:

```typescript
const user = await auth.currentUser() // returns null if not authenticated
```

## Protecting Routes

Set `auth: true` on any route definition in `app/routes.fx`:

```typescript
{ path: '/dashboard', component: () => import('./pages/Dashboard.fx'), auth: true }
```

Unauthenticated users are redirected to `loginPath` (defaults to `/login`).

## Permission Checks

```typescript
const canEdit = await auth.can('post:edit', post)
if (!canEdit) throw new Error('Forbidden')
```

## Email + Password

The email provider handles sign-up, sign-in, and password management. Add the `Auth.email()` provider and implement the sign-in server function:

```typescript
import { auth } from 'app/auth.fx'
import { Users } from 'app/schema.fx'
import { db } from 'forge:data'
import { EmailProvider } from 'forge:auth'

export async function signIn(email: string, password: string) {
  const user = await db.query<User>(Users).where({ email }).first()
  if (!user) throw new Error('Invalid credentials')
  const valid = await EmailProvider.verify(email, password)
  if (!valid) throw new Error('Invalid credentials')
  await auth.createSession(user.id)
}
```

## OAuth

Configure OAuth providers in `app/auth.fx`. Forge handles the authorization code flow and token exchange. After a successful OAuth sign-in, your callback server function receives the provider's user profile.

## Passkeys (WebAuthn)

Add `Auth.passkey()` to the providers list. The `PasskeyProvider` from `forge:auth` implements the WebAuthn registration and authentication ceremonies. No additional configuration is required for passkeys.
