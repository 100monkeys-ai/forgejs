// Auth.configure() — the root auth instance factory.
//
// Call once in app/auth.fx. The returned auth object is the primary interface
// for auth operations throughout the application. It is server-only; any
// attempt to import it from a client module is a compile-time error.

"use module server"

import type { AuthConfig, User, Session } from './types'

export class AuthInstance {
  private config: AuthConfig

  constructor(config: AuthConfig) {
    this.config = config
  }

  async currentUser(): Promise<User | null> {
    // TODO: Read session from request context, look up user in db
    return null
  }

  async requireUser(): Promise<User> {
    const user = await this.currentUser()
    if (!user) {
      throw new Error('Unauthenticated')
    }
    return user
  }

  async can(
    action: string,
    resource?: Record<string, unknown>
  ): Promise<boolean> {
    const user = await this.currentUser()
    if (!user) return false
    const rule = this.config.permissions?.[action]
    if (!rule) return false
    return rule({ user, resource })
  }

  async createSession(userId: string): Promise<Session> {
    // TODO: Create session record, set cookie
    return {
      id: crypto.randomUUID(),
      userId,
      createdAt: new Date(),
      expiresAt: new Date(Date.now() + (this.config.session?.maxAge ?? 86400) * 1000),
    }
  }

  async destroySession(): Promise<void> {
    // TODO: Invalidate session record, clear cookie
  }
}

export const Auth = {
  configure(config: AuthConfig): AuthInstance {
    return new AuthInstance(config)
  },

  email() {
    return { type: 'email' as const }
  },

  passkey() {
    return { type: 'passkey' as const }
  },

  oauth(provider: string, config: { clientId: string; clientSecret: string }) {
    return { type: 'oauth' as const, provider, ...config }
  },
}
