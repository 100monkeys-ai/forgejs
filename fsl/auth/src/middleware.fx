// Auth middleware for route protection.
//
// requireAuth() is used in route definitions to enforce authentication before
// the route component or server function is invoked. It redirects unauthenticated
// users to the login page rather than returning a 401.
//
// optionalAuth() populates the user context when a session exists but does not
// redirect — useful for pages that have both authenticated and public states.

"use module server"

import type { User } from './types'

export interface AuthContext {
  user: User | null
}

export async function requireAuth(): Promise<User> {
  // TODO: Integrate with request context and auth instance
  // Throw redirect to login if no valid session
  const user = null
  if (!user) {
    throw { type: 'redirect', location: '/login', status: 302 }
  }
  return user
}

export async function optionalAuth(): Promise<User | null> {
  // TODO: Integrate with request context and auth instance
  return null
}
