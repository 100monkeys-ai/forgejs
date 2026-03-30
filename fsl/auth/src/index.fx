// forge:auth — Forge Standard Library authentication module
//
// Provides session-based and token-based authentication with support for
// email+password, passkeys (WebAuthn), and OAuth providers. Permission
// checks are co-located with the auth layer rather than scattered across
// server functions.
//
// Example — app/auth.fx:
//   import { Auth } from 'forge:auth'
//
//   export const auth = Auth.configure({
//     session: { secret: env.SESSION_SECRET, maxAge: 60 * 60 * 24 * 30 },
//     providers: [
//       Auth.email(),
//       Auth.oauth('github', { clientId: env.GITHUB_CLIENT_ID, clientSecret: env.GITHUB_CLIENT_SECRET }),
//     ],
//     permissions: {
//       'post:edit': ({ user, resource }) => user.id === resource.authorId || user.role === 'admin',
//       'post:delete': ({ user }) => user.role === 'admin',
//     },
//   })
//
// Example — in a server function:
//   import { auth } from 'app/auth.fx'
//   const user = await auth.currentUser()
//   if (!user) throw new AuthError('Unauthenticated')
//   await auth.can('post:edit', post)

export { Auth } from './session.fx'
export { requireAuth, optionalAuth } from './middleware.fx'
export { EmailProvider, OAuthProvider, PasskeyProvider } from './providers.fx'
export type {
  User,
  Session,
  AuthConfig,
  Provider,
  PermissionRule,
  OAuthConfig,
} from './types'
